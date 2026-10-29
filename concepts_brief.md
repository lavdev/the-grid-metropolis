# The Grid Metropolis Harness — Concept Brief

* **Author:** Project owner and Codex brainstorm synthesis
* **Date:** 2026-10-29

## Overview

Make the Harness a trustworthy, inspectable workflow first. A run should produce real project changes, run an explicitly authorized verification command, record the outcome, and stop truthfully when it fails, pauses, or exhausts its repair limit. The city should visualize that same run state; it must not imply that work happened when it did not.

Reuse the existing architect → coder → reviewer flow as the starting point. Add a machine-run test/verification step and at most one bounded repair pass initially. Produce a factual run summary from recorded events and evidence. Keep a separate documenter droid optional until users need a specific generated artifact.

This advances the active roadmap item T-6.36 (Harness PRO). Treat the brief as concept direction, not a detailed implementation plan.

## Analysis: Problem and current context

The app already has a Svelte terminal, persisted project conversations, configurable droid profiles, a client-side sequential multi-droid pipeline, a Codex streaming endpoint, and a separate Phaser city. These are useful starting points, but they do not yet form one verifiable execution record:

- The current pipeline sequences architect, coder, and reviewer prompts in `projectStore.ts`. Stage errors may be caught while later stages continue, and finalization can report completion without independent verification.
- The Codex backend currently invokes `--dangerously-bypass-approvals-and-sandbox`, accepts a caller-provided working directory, and the stream route is not protected by the Bearer middleware used by bridge dispatch. A UI-only permission prompt would not enforce an execution boundary.
- The main `App.svelte` mounts the terminal, while `CanvasContainer.svelte` is separate. The Phaser population in `AgentManager.ts` is hard-coded and wanders independently. The current WebSocket event reducer updates Svelte stores but does not make the Phaser scene the source of execution truth.
- The AI service can return a simulated response when provider configuration is absent. That mode must never be presented as a real run or verified result.
- SQLite persistence, the project store, the Codex SSE boundary, WebSocket client, event feed, and game bridge are existing reuse points. No new queue, Redis layer, or telemetry platform is justified for the first milestone.

## Proposed user workflow

1. **Start a run.** The user gives a task in a selected project. The interface identifies the project root and the droid roles that will participate.
2. **Plan.** The architect proposes a short plan and relevant files. If the product wants an explicit human checkpoint, the user can approve or revise the plan before execution; do not add this pause without evidence that it improves safety or usefulness.
3. **Execute.** The coder makes changes only within the authorized project scope. Every requested operation is checked by the backend before it runs.
4. **Verify.** Run the project’s explicitly configured verification command under a timeout and resource limits. Store the exact command identity, exit status, bounded stdout/stderr, and duration. A model statement that tests passed is not evidence.
5. **Repair once.** If verification fails, allow one repair pass, rerun verification, and then stop. Show the attempt count and result. Make the limit configurable only after real runs establish a need.
6. **Review and summarize.** The reviewer can assess the diff and verification evidence. Generate a factual summary from recorded outcomes, changed files, and review findings. Add a documenter agent only when a concrete document deliverable requires generated prose.
7. **Finish visibly.** End in an explicit state such as succeeded, failed, paused for permission, cancelled, or repair limit reached. Never label an unverified or interrupted run successful.

## Authorization model

Authorization is a server-side execution rule, not a droid prompt or visual setting.

- A project defines the maximum capabilities and trusted project root. A droid profile can narrow that ceiling. Effective access is the intersection of the project ceiling and the droid’s allowed subset.
- Named reference projects are separate, explicit resources. Start with read-only access if reference access is needed; naming a project must not imply write or command access to it.
- When a required capability is missing, persist the run as `WAITING_FOR_PERMISSION` before the operation. Stop at the operation boundary; do not leave an unrestricted subprocess running behind a modal. Show the droid, project, exact requested action, resource/path, and consequence. The user can grant once, deny, or later edit the droid profile. Remembered grants are a later capability, not part of the first safe milestone.
- Denying a request refuses that exact action, not automatically the whole task. The droid may try up to three genuinely distinct alternatives using only capabilities already granted. If none works, it may make one new, specific permission request that explains why the alternatives failed and what action is now needed. If that request is not granted, stop the task as blocked and preserve prior work and evidence; do not repeat the same request or cycle through equivalent alternatives.
- A task-scoped grant expires with the run and authorizes only the displayed operation. The backend checks the grant immediately before execution. Add remembered grants after server-side policy enforcement is proven. A remembered grant must be scoped to one droid, one project, and one capability, cannot exceed the project ceiling, is auditable and revocable, and never silently applies to another project.
- Resolve project identity and canonical paths on the server. Enforce the same policy on every process-spawn path, including legacy clients; do not leave an alternate endpoint that can invoke unrestricted execution.
- Keep the Codex native sandbox/approval protections enabled. Do not run arbitrary caller-supplied commands or paths. Allow only explicitly configured verification commands and narrowly scoped operations.

## Run state and city connection

The backend should own the durable run and step record because it launches the process and is the authority for permissions and verification. Persist run identity, project/droid identity, step and attempt status, pending permission request, approval decisions, verification evidence, cancellation, and terminal outcome in the existing SQLite-backed server. Keep this additive and versioned so older project conversations and thread IDs remain readable. A pending request survives closing and reopening the Harness, but holds no active process or execution resources. The user can grant or deny it, or cancel the run. After a grant, resume only the blocked operation from a safe checkpoint. An interrupted run should reopen paused/reviewable, never auto-repeat edits.

Publish factual lifecycle events from that run record. Include a stable `run_id` and correlated project, droid, step, attempt, and timestamp fields. Store events so the client can recover after reconnect. Svelte and Phaser should render projections of these same events; animation errors must not alter run outcome. Begin with simple stage and status visuals. Do not build a generalized city simulation or elaborate choreography for this workflow.

Ambient wandering and simulation can remain as presentation features, but label them clearly. Task-related movement, success effects, and error states must be tied to actual run events. The terminal timeline remains a useful primary view even if the Phaser canvas is not mounted in the main app.

## User experience and observability

Make the workflow legible without requiring users to understand internal policy terms. Show the effective access a droid has in project settings and its profile, and explain each permission in plain language (for example, “Run project tests — allows this droid to run the configured test command here”). Keep pending approvals visible and resumable. After denial, cancellation, or exhausted repair, preserve prior work and evidence and explain the next safe choices.

For each run, make it possible to answer: what was requested; which role acted; what operation was attempted; what permission decision applied; which files changed; what command ran; what its exit status and output were; how many repair attempts occurred; and why the run stopped. Propagate a run ID through API/SSE, database records, structured Rust tracing, WebSocket events, and the UI. Distinguish provider-reported token usage from estimates. Track duration, stage failures, permission waits/denials, test outcomes, retries, cancellation, subprocess exit status, and stream disconnects.

Do not log credentials, secrets, or private model reasoning. Bound and redact captured process output, and make stored prompts/output subject to the project’s data handling expectations.

## Goals vs. Non-Goals

**Goals:** Prove one real, inspectable run in a selected project; enforce project and droid permissions on the server; execute a configured verification command; allow up to three distinct, permission-compliant alternatives after a denied operation; preserve pending permission requests across app restarts; and show the same actual run state in the terminal and city.

**Non-goals for the first milestone:** A general-purpose workflow engine, concurrent autonomous agents, unlimited repair or fallback loops, remembered grants, unrestricted host access, broad access to unlisted reference projects, a mandatory documenter droid, and a full simulation of the city’s economy or social life.

## Alternatives Weighed

| Option | Assessment |
| --- | --- |
| Extend the current three-role client pipeline and add server-verified execution | Best first step: reuses real code, preserves familiar handoffs, and closes the gap between claimed and verified outcomes. Requires moving authority and durable run state behind server-enforced boundaries. |
| Start with one droid and one verification command | Lowest complexity and useful as a safety baseline. It omits the existing role handoffs, so it can be retained as a fallback or narrow first slice if the pipeline blocks safe execution. |
| Build a configurable workflow engine with arbitrary stages, parallel agents, and retry policies | Defer. It multiplies state transitions, partial failures, permissions, costs, and recovery requirements before one safe run is proven. |
| Make a documenter droid generate every final report | Defer. A deterministic summary grounded in stored evidence is cheaper and more reliable; add an agent only for a clearly defined user-facing document artifact. |
| Remember permissions and support broad capability profiles immediately | Defer persistent grants and broad policy taxonomies until task-scoped enforcement and revocation are proven. Keep the model extensible, but implement only the specific operations required for v1. |
| Add Redis, queues, or a distributed tracing/metrics stack | Not justified for the desktop/local first workflow. Existing SQLite, Rust tracing, SSE, WebSocket events, and UI event history are sufficient starting points. |

## Risks and mitigations

- **Arbitrary code execution or path escape — critical.** Current Codex invocation bypasses approvals and sandboxing. Enforce canonical project roots, an operation allowlist, native sandbox policy, and capability checks in the backend before enabling real workflow runs. Apply enforcement to every existing execution route.
- **False success — critical.** Current stage completion is not equivalent to tests passing. Define typed outcomes and gate success on the configured command’s recorded exit status plus required review checks.
- **Run interruption and repeated side effects — high.** Persist step state and cancellation. Resume only from a defined safe checkpoint; never automatically repeat a potentially mutating operation after restart.
- **Grant confusion or excess scope — high.** Default to task-scoped approvals, display exact action and target, provide grant history/revocation, and prevent grants from exceeding project policy.
- **Cost and runaway repair — high.** Set per-run call, time, and token ceilings with a small repair cap. Stop before a call that would exceed the remaining budget; identify usage estimates separately from provider measurements.
- **Leaks through reference projects or logs — high.** Explicitly authorize named references, begin read-only, avoid logging secrets/reasoning, and bound/redact output capture.
- **Migration and rollback — high.** Use additive, versioned SQLite migrations with no broad defaults for legacy droids; old records should default to no execution capabilities. Roll out server enforcement before UI exposure. A rollback must not restore the unsafe bypass; if the safe server is unavailable, execution remains disabled.
- **Untrustworthy city state — medium.** Feed visuals from persisted run events, label ambient/simulated behavior, and keep run outcome independent of Phaser state.

## Key Decisions & Staging for Next Steps

### Staging and success criteria

Roll out in gated slices:

1. Enforce the safe server execution boundary for every Codex spawn path. Legacy projects receive no execution permissions until explicitly configured.
2. Record a durable run with correlated stages and truthful failure, pause, and cancellation states; first connect it to the current terminal timeline.
3. Add one configured verification command and one fixed repair pass.
4. Connect city visuals to the canonical events after the terminal workflow is trustworthy. Keep the feature usable if Phaser is disconnected or unavailable.
5. After the safe first run is proven, add remembered grants with clear scope, history, and revocation. Consider configurable repair counts, richer reference access, and a dedicated documenter only after usage demonstrates the need.

The concept is successful when a user can inspect one run and independently confirm its project, operations, permission decisions, changed files, verification command and exit status, repair count, and final outcome; a blocked or disconnected run never appears successful; and a city animation cannot invent execution evidence.

### Goldfish review synthesis

All six requested lenses completed. Their strongest shared objection is that the existing unrestricted Codex endpoint makes a front-end permission design cosmetic unless authorization and path policy are enforced at the backend. Reviewers also converged on truthful persisted outcomes, a minimal first workflow, and keeping the city a view of real events. The UX review additionally emphasized exact-action approval choices and persistent resumable pauses. Key codebase reuse and limitations are captured above.

