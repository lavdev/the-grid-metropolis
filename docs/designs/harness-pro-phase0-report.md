# Harness PRO — App-Server Phase 0 Spike

**Date:** 2026-10-29
**Codex CLI:** 0.164.0
**Status:** Protocol behaviors verified in disposable probes; production recovery coordinator not implemented.

## Scope

The probes launched `codex app-server --stdio`, used the documented initialize/initialized handshake, and ran only in generated temporary directories. They used a minimal authenticated Codex turn and disposable command/file operations. Both probes cleaned their temporary directories after completion.

## Results

| Scenario | Observed result | What it proves |
| --- | --- | --- |
| Thread resume after process restart | Passed. A completed thread was reopened by `thread/resume`, and a later turn completed on the same thread ID. | Local stdio, thread persistence, and resume work in CLI 0.164.0. |
| Pending approval, no grant sent | Passed. After restarting the app-server, the probe reissued the same command; the canonical action fingerprint matched. | A new approval can be requested after resume when the regenerated operation is identical. |
| Grant sent, result not persisted by Harness | Passed in the probe policy. The disposable command approval was accepted, the process was closed before the probe saved an operation result, and recovery selected `BLOCKED` without starting another turn. The delayed side effect count was 0. | The conservative rule avoids replay when only a sent grant is durable. This exercised the probe checkpoint rule, not a production Rust coordinator. |
| Completed operation result persisted | Passed. A disposable file-change approval was accepted; both `item/completed` and `turn/completed` reported `completed`. After restart, `thread/turns/list` returned the completed item and the file contained exactly the expected value. | Persisted completion can be reconciled and followed by the next stage without replaying the mutation. |
| Token usage | Passed. `thread/tokenUsage/updated` reported input/output/total usage. In one short two-turn thread, the cumulative total reported 22,260 tokens after the first turn and 44,562 after the second. | Usage telemetry exists and is cumulative at thread level; use one thread per run or compute a per-run delta. Values vary by prompt and runtime. |

The JSON-RPC request ID was `0` for approval requests in separate app-server processes. It is connection-scoped and must not be used by itself as a durable or globally unique permission-request key. Store a Harness-owned request ID and an app-server process-generation identifier alongside thread, turn, and item IDs.

## Budget limitation

The generated stable schema for CLI 0.164.0 exposes `thread/tokenUsage/updated`, but `turn/start` has no `maxOutputTokens` field. The probe observed usage after dispatch. Therefore the app-server contract does not establish a hard token ceiling for an individual in-flight turn. A coordinator can count Harness-dispatched turns, enforce elapsed time, and stop or interrupt after observed usage, but token usage may overshoot within a turn. Keep real execution disabled until the owner accepts this best-effort behavior or an enforceable provider-level limit is proven.

## Validation performed

- `node --check scripts/app_server_phase0_probe.mjs`
- `node scripts/app_server_phase0_probe.mjs`
- `node --check scripts/app_server_phase0_approval_probe.mjs`
- `node scripts/app_server_phase0_approval_probe.mjs`
- `codex app-server generate-json-schema` and a schema scan for per-turn token limit fields.

These were protocol probes, not tests of the planned Rust run coordinator, SQLite recovery implementation, HTTP API, or UI. Those remain future work.

## Protocol references

- [Codex App Server: thread resume](https://learn.chatgpt.com/docs/app-server#start-or-resume-a-thread)
- [Codex App Server: streamed events and token usage](https://learn.chatgpt.com/docs/app-server#events)
- [Codex App Server: approval request/response flow](https://learn.chatgpt.com/docs/app-server#approvals)
