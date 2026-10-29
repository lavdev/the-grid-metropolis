# Stage 6 — Comprehension Test

**Target:** `docs/designs/harness-pro.md`
**Ranges reviewed:** FeatureRange, lines 1–280; DetailedRange, lines 281–360.
**Reviewer:** `egm-comprehension` — completed.
**Persona and target:** Read in full.

## What this proposes

The document proposes moving project-changing work into a durable, server-owned run workflow. It keeps the Architect → Coder → Reviewer sequence, adds a configured verification command and one repair pass, and uses recorded run events to drive both the terminal and the city. Project and droid policies constrain execution; permission pauses, test evidence, and recovery state become part of the run record.

## How the existing system works, per this document

The current Harness has project conversations, a sequential multi-droid pipeline, a Codex SSE endpoint, and a separate Phaser city. The document says the pipeline can continue after errors or claim completion without independent test evidence, while city activity can be unrelated to real work. The Codex endpoint also disables sandbox and approval controls and accepts a caller-supplied working directory.

## Findings

### Major — The user grant's relationship to configured policy is unclear

**Affected scope:** Run coordinator boundaries and workflow, especially lines 190–194; Phase 1, line 308; permission panel, lines 334–336.

The policy description says Codex requests are checked against the project capability ceiling and droid subset, while the UI offers a grant for an action described as lacking permission and says that grant applies to that action only. It is unclear whether the user can authorize a one-operation exception to those configured limits, or only approve an action already permitted by them. Clarify which requests are grantable and how the decision interacts with the configured policy.

### Major — Snapshot, replay, and live WebSocket handoff can leave a gap

**Affected scope:** Selected event flow, lines 110–112; API operations, lines 138–142; run persistence and event contract, lines 211–212; Phase 2, lines 319–322.

The client fetches a snapshot and replays events before following the WebSocket. Events committed after replay but before live delivery resumes could be missed. Define an atomic cursor/subscription handoff or require sequence-gap detection followed by another replay so terminal and city state converge.
