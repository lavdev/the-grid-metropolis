#!/usr/bin/env node

import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import os from "node:os";
import path from "node:path";
import { mkdtemp, rm } from "node:fs/promises";

const timeoutMs = 90_000;
const appServerArgs = ["app-server", "--stdio"];
const clientInfo = {
  name: "grid-metropolis-phase0-probe",
  title: "Grid Metropolis Phase 0 Probe",
  version: "0.1.0",
};

class AppServerProcess {
  constructor(cwd) {
    this.child = spawn("codex", appServerArgs, {
      cwd,
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
    });
    this.nextId = 1;
    this.pending = new Map();
    this.events = [];
    this.stderr = "";
    this.closed = new Promise((resolve) => {
      this.child.once("close", (code, signal) => resolve({ code, signal }));
    });

    const lines = createInterface({ input: this.child.stdout });
    lines.on("line", (line) => this.#handleLine(line));
    this.child.stderr.on("data", (chunk) => {
      this.stderr = (this.stderr + chunk.toString()).slice(-12_000);
    });
  }

  #handleLine(line) {
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      this.events.push({ method: "protocol/non-json", text: line.slice(0, 500) });
      return;
    }

    if (Object.hasOwn(message, "id") && !message.method) {
      const resolve = this.pending.get(String(message.id));
      if (resolve) {
        this.pending.delete(String(message.id));
        resolve(message);
      }
      return;
    }

    this.events.push(message);
  }

  async request(method, params = {}) {
    const id = this.nextId++;
    const result = new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(String(id));
        reject(new Error(`Timed out waiting for ${method}`));
      }, timeoutMs);
      this.pending.set(String(id), (message) => {
        clearTimeout(timer);
        if (message.error) {
          reject(new Error(`${method} failed: ${JSON.stringify(message.error)}`));
        } else {
          resolve(message.result);
        }
      });
    });

    this.child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
    return result;
  }

  notify(method, params = {}) {
    this.child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method, params })}\n`);
  }

  async initialize() {
    await this.request("initialize", { clientInfo, capabilities: {} });
    this.notify("initialized");
  }

  async waitFor(method, predicate = () => true) {
    const startedAt = Date.now();
    while (Date.now() - startedAt < timeoutMs) {
      const matchIndex = this.events.findIndex(
        (event) => event.method === method && predicate(event.params ?? {}),
      );
      if (matchIndex >= 0) return this.events.splice(matchIndex, 1)[0];
      if (this.child.exitCode !== null) {
        throw new Error(`app-server exited while waiting for ${method}: ${this.stderr}`);
      }
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    throw new Error(`Timed out waiting for ${method}`);
  }

  async stop() {
    if (this.child.exitCode !== null) return this.closed;
    this.child.stdin.end();
    const exitAfterEof = await Promise.race([
      this.closed,
      new Promise((resolve) => setTimeout(() => resolve(null), 5_000)),
    ]);
    if (exitAfterEof) return exitAfterEof;
    this.child.kill();
    return this.closed;
  }
}

async function runTurn(appServer, threadId, prompt) {
  const started = await appServer.request("turn/start", {
    threadId,
    input: [{ type: "text", text: prompt }],
    approvalPolicy: "on-request",
    sandboxPolicy: { type: "readOnly" },
  });
  const turnId = started.turn?.id;
  if (!turnId) throw new Error("turn/start did not return a turn ID");

  const completed = await appServer.waitFor(
    "turn/completed",
    (params) => params.turn?.id === turnId,
  );
  const usageEvents = appServer.events.filter(
    (event) => event.method === "thread/tokenUsage/updated" && event.params?.threadId === threadId,
  );
  const agentMessages = appServer.events
    .filter((event) => event.method === "item/agentMessage/delta")
    .map((event) => event.params?.delta ?? "")
    .join("");

  return {
    turnId,
    status: completed.params?.turn?.status ?? "unknown",
    tokenUsageEventCount: usageEvents.length,
    latestTokenUsage: usageEvents.at(-1)?.params?.tokenUsage?.total ?? null,
    messageTail: agentMessages.slice(-120),
  };
}

const tempBase = path.resolve(os.tmpdir());
const cwd = await mkdtemp(path.join(tempBase, "grid-metropolis-app-server-phase0-"));
const threadResults = [];
let first;
let second;

try {
  first = new AppServerProcess(cwd);
  await first.initialize();
  const started = await first.request("thread/start", {
    cwd,
    sandbox: "read-only",
    approvalPolicy: "on-request",
    threadSource: "grid-metropolis-phase0-probe",
  });
  const threadId = started.thread?.id;
  if (!threadId) throw new Error("thread/start did not return a thread ID");

  const firstTurn = await runTurn(first, threadId, "Reply with PHASE0_OK only. Do not use tools.");
  await first.stop();
  first = null;
  threadResults.push(firstTurn);

  second = new AppServerProcess(cwd);
  await second.initialize();
  const resumed = await second.request("thread/resume", { threadId, cwd });
  if (resumed.thread?.id !== threadId) throw new Error("thread/resume returned a different thread");
  const secondTurn = await runTurn(second, threadId, "Reply with PHASE0_RESUMED only. Do not use tools.");
  await second.stop();
  second = null;
  threadResults.push(secondTurn);

  console.log(JSON.stringify({
    appServer: "codex app-server --stdio",
    result: "PASS",
    sameThreadResumedAcrossProcessRestart: true,
    turns: threadResults,
    note: "This probe confirms thread resumption and usage telemetry only; it does not exercise approval recovery or prove a hard pre-dispatch token cap.",
  }, null, 2));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  if (first) await first.stop();
  if (second) await second.stop();
  const resolvedCwd = path.resolve(cwd);
  const resolvedBase = path.resolve(tempBase);
  const containedInTemp = path.dirname(resolvedCwd) === resolvedBase;
  const hasExpectedPrefix = path.basename(resolvedCwd).startsWith("grid-metropolis-app-server-phase0-");
  if (containedInTemp && hasExpectedPrefix) {
    await rm(resolvedCwd, { recursive: true, force: true });
  } else {
    console.error(`Refusing to remove unexpected probe directory: ${resolvedCwd}`);
    process.exitCode = 1;
  }
}
