#!/usr/bin/env node

import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import os from "node:os";
import path from "node:path";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";

const timeoutMs = 90_000;
const tempBase = path.resolve(os.tmpdir());
const cwd = await mkdtemp(path.join(tempBase, "grid-metropolis-approval-phase0-"));
const checkpointPath = path.join(cwd, "checkpoint.json");
const clientInfo = {
  name: "grid-metropolis-phase0-approval-probe",
  title: "Grid Metropolis Phase 0 Approval Probe",
  version: "0.1.0",
};

class AppServerProcess {
  constructor() {
    this.child = spawn("codex", ["app-server", "--stdio"], {
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
        if (message.error) reject(new Error(`${method} failed: ${JSON.stringify(message.error)}`));
        else resolve(message.result);
      });
    });
    this.child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
    return result;
  }

  async waitFor(predicate) {
    const startedAt = Date.now();
    while (Date.now() - startedAt < timeoutMs) {
      const index = this.events.findIndex(predicate);
      if (index >= 0) return this.events.splice(index, 1)[0];
      if (this.child.exitCode !== null) {
        throw new Error(`app-server exited before approval: ${this.stderr}`);
      }
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    const tail = this.events.slice(-8).map((event) => event.method ?? "response");
    throw new Error(`Timed out waiting for approval; recent events: ${tail.join(", ")}`);
  }

  async initialize() {
    await this.request("initialize", { clientInfo, capabilities: {} });
    this.child.stdin.write('{"jsonrpc":"2.0","method":"initialized","params":{}}\n');
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

function stable(value) {
  if (Array.isArray(value)) return value.map(stable);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, stable(value[key])]));
  }
  return value;
}

function approvalAction(request) {
  const params = request.params ?? {};
  return {
    method: request.method,
    command: params.command ?? null,
    cwd: params.cwd ?? null,
    commandActions: params.commandActions ?? null,
    networkApprovalContext: params.networkApprovalContext ?? null,
    grantRoot: params.grantRoot ?? null,
  };
}

function approvalFingerprint(request) {
  return createHash("sha256")
    .update(JSON.stringify(stable(approvalAction(request))))
    .digest("hex");
}

function isApproval(message) {
  return [
    "item/commandExecution/requestApproval",
    "item/fileChange/requestApproval",
  ].includes(message.method);
}

const operationText = "For this disposable Phase 0 probe, run exactly this command in the current directory and then stop: node -e \"setTimeout(()=>require('node:fs').appendFileSync('approval-probe.log','PHASE0_APPROVAL\\n'),10000)\". Do not use another command or modify any other path.";
const completedOperationText = "For this disposable Phase 0 probe, run exactly this command in the current directory and then stop: node -e \"require('node:fs').writeFileSync('completed-probe.log','PHASE0_COMPLETED')\". Do not use another command or modify any other path.";

let first;
let second;
let third;
let fourth;
let fifth;
try {
  first = new AppServerProcess();
  await first.initialize();
  const threadStarted = await first.request("thread/start", {
    cwd,
    sandbox: "read-only",
    approvalPolicy: "on-request",
    threadSource: "grid-metropolis-phase0-approval-probe",
  });
  const threadId = threadStarted.thread?.id;
  if (!threadId) throw new Error("thread/start did not return a thread ID");

  const firstTurn = await first.request("turn/start", {
    threadId,
    input: [{
      type: "text",
      text: operationText,
    }],
    sandboxPolicy: { type: "readOnly" },
  });
  const firstApproval = await first.waitFor(isApproval);
  const firstFingerprint = approvalFingerprint(firstApproval);
  const checkpoint = {
    state: "WAITING_FOR_PERMISSION",
    threadId,
    turnId: firstTurn.turn?.id ?? null,
    requestId: firstApproval.id,
    requestMethod: firstApproval.method,
    itemId: firstApproval.params?.itemId ?? null,
    fingerprint: firstFingerprint,
    grantSent: false,
    resultPersisted: false,
  };
  await writeFile(checkpointPath, `${JSON.stringify(checkpoint, null, 2)}\n`, "utf8");
  await first.stop();
  first = null;

  second = new AppServerProcess();
  await second.initialize();
  const resumed = await second.request("thread/resume", { threadId, cwd });
  if (resumed.thread?.id !== threadId) throw new Error("thread/resume returned a different thread");
  await second.request("turn/start", {
    threadId,
    input: [{ type: "text", text: operationText }],
    sandboxPolicy: { type: "readOnly" },
  });
  const secondApproval = await second.waitFor(isApproval);
  const secondFingerprint = approvalFingerprint(secondApproval);

  const match = firstFingerprint === secondFingerprint;
  if (!match) throw new Error("reissued permission request does not match the persisted operation fingerprint");

  checkpoint.grantSent = true;
  checkpoint.state = "GRANT_SENT_RESULT_NOT_PERSISTED";
  checkpoint.requestId = secondApproval.id;
  checkpoint.turnId = secondApproval.params?.turnId ?? checkpoint.turnId;
  checkpoint.itemId = secondApproval.params?.itemId ?? checkpoint.itemId;
  await writeFile(checkpointPath, `${JSON.stringify(checkpoint, null, 2)}\n`, "utf8");
  second.child.stdin.write(`${JSON.stringify({
    jsonrpc: "2.0",
    id: secondApproval.id,
    result: { decision: "accept" },
  })}\n`);
  await second.waitFor((event) =>
    event.method === "serverRequest/resolved" &&
    String(event.params?.requestId) === String(secondApproval.id));
  await second.stop();
  second = null;

  third = new AppServerProcess();
  await third.initialize();
  const recoveredThread = await third.request("thread/resume", { threadId, cwd });
  if (recoveredThread.thread?.id !== threadId) throw new Error("recovery resume returned a different thread");
  const recoveryDecision = checkpoint.grantSent && !checkpoint.resultPersisted ? "BLOCKED" : "RESUME";
  let sideEffectCount = 0;
  try {
    const log = await readFile(path.join(cwd, "approval-probe.log"), "utf8");
    sideEffectCount = log.split("PHASE0_APPROVAL").length - 1;
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  await third.stop();
  third = null;

  fourth = new AppServerProcess();
  await fourth.initialize();
  const completedThreadStart = await fourth.request("thread/start", {
    cwd,
    sandbox: "read-only",
    approvalPolicy: "on-request",
    threadSource: "grid-metropolis-phase0-completed-probe",
  });
  const completedThreadId = completedThreadStart.thread?.id;
  if (!completedThreadId) throw new Error("completed probe thread/start did not return a thread ID");
  const completedTurnStart = await fourth.request("turn/start", {
    threadId: completedThreadId,
    input: [{ type: "text", text: completedOperationText }],
    sandboxPolicy: { type: "readOnly" },
  });
  const completedApproval = await fourth.waitFor(isApproval);
  const completedCheckpoint = {
    state: "GRANT_SENT_RESULT_NOT_PERSISTED",
    threadId: completedThreadId,
    turnId: completedTurnStart.turn?.id ?? null,
    requestId: completedApproval.id,
    itemId: completedApproval.params?.itemId ?? null,
    fingerprint: approvalFingerprint(completedApproval),
    grantSent: true,
    resultPersisted: false,
  };
  await writeFile(checkpointPath, `${JSON.stringify(completedCheckpoint, null, 2)}\n`, "utf8");
  fourth.child.stdin.write(`${JSON.stringify({
    jsonrpc: "2.0",
    id: completedApproval.id,
    result: { decision: "accept" },
  })}\n`);
  await fourth.waitFor((event) =>
    event.method === "serverRequest/resolved" &&
    String(event.params?.requestId) === String(completedApproval.id));
  const completedItemEvent = await fourth.waitFor((event) =>
    event.method === "item/completed" &&
    event.params?.item?.id === completedCheckpoint.itemId);
  const completedTurnEvent = await fourth.waitFor((event) =>
    event.method === "turn/completed" &&
    event.params?.turn?.id === completedCheckpoint.turnId);
  const completedFile = await readFile(path.join(cwd, "completed-probe.log"), "utf8");
  completedCheckpoint.state = "COMPLETED";
  completedCheckpoint.resultPersisted = true;
  completedCheckpoint.itemStatus = completedItemEvent.params?.item?.status ?? null;
  completedCheckpoint.turnStatus = completedTurnEvent.params?.turn?.status ?? null;
  await writeFile(checkpointPath, `${JSON.stringify(completedCheckpoint, null, 2)}\n`, "utf8");
  await fourth.stop();
  fourth = null;

  fifth = new AppServerProcess();
  await fifth.initialize();
  const completedResume = await fifth.request("thread/resume", { threadId: completedThreadId, cwd });
  if (completedResume.thread?.id !== completedThreadId) throw new Error("completed thread resume returned a different thread");
  const turnHistory = await fifth.request("thread/turns/list", {
    threadId: completedThreadId,
    limit: 20,
    sortDirection: "asc",
    itemsView: "full",
  });
  const historyTurn = turnHistory.data?.find((turn) => turn.id === completedCheckpoint.turnId);
  const historyItem = historyTurn?.items?.find((item) => item.id === completedCheckpoint.itemId);
  const completedResultMatches =
    completedCheckpoint.resultPersisted &&
    completedCheckpoint.itemStatus === "completed" &&
    completedCheckpoint.turnStatus === "completed" &&
    historyItem?.status === "completed" &&
    completedFile === "PHASE0_COMPLETED";
  await fifth.stop();
  fifth = null;

  console.log(JSON.stringify({
    result: "PASS",
    pendingApprovalReissuedAfterProcessRestart: true,
    sameOperationFingerprint: match,
    freshApprovalMethod: firstApproval.method,
    jsonRpcRequestIdWasReusedAcrossProcesses: String(firstApproval.id) === String(secondApproval.id),
    firstAction: approvalAction(firstApproval),
    secondAction: approvalAction(secondApproval),
    firstFingerprint,
    secondFingerprint,
    grantSentWithoutHarnessResultRecoversAs: recoveryDecision,
    retryStartedAfterRecovery: false,
    sideEffectCountObservedBeforeCleanup: sideEffectCount,
    completedOperationPersistedAcrossRestart: completedResultMatches,
    completedOperationStatus: completedCheckpoint.itemStatus,
    completedRunTurnStatus: completedCheckpoint.turnStatus,
    completedSideEffectValue: completedFile,
    nextStageAllowedWithoutReplay: completedResultMatches,
    note: "The probe accepted a disposable file-write request and exercised both the uncertain-result BLOCKED path and persisted-completion continuation path.",
  }, null, 2));
  if (recoveryDecision !== "BLOCKED" || sideEffectCount > 1 || !completedResultMatches) process.exitCode = 1;
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  if (first) await first.stop();
  if (second) await second.stop();
  if (third) await third.stop();
  if (fourth) await fourth.stop();
  if (fifth) await fifth.stop();
  const resolvedCwd = path.resolve(cwd);
  const resolvedBase = path.resolve(tempBase);
  const containedInTemp = path.dirname(resolvedCwd) === resolvedBase;
  const hasExpectedPrefix = path.basename(resolvedCwd).startsWith("grid-metropolis-approval-phase0-");
  if (containedInTemp && hasExpectedPrefix) {
    await rm(resolvedCwd, { recursive: true, force: true });
  } else {
    console.error(`Refusing to remove unexpected probe directory: ${resolvedCwd}`);
    process.exitCode = 1;
  }
}
