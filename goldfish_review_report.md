# Goldfish Annotated Review Report

**Target:** `docs/designs/harness-pro.md` (full file, 348 lines at this follow-up)
**Source:** `consolidated_review_report.md`
**Status:** resolved
**Resolution:** All 17 PROPOSE FIX edits were applied to the target. No findings were contested, escalated, or declined. In a follow-up design decision, the owner approved required project-level per-run call, time, and token ceilings; droid/run overrides may only tighten them. No product-wide numeric defaults or hard maxima are prescribed. The design also now defines three app-server restart recovery cases: ungranted approval remains pending and is reissued only with a matching request fingerprint; a sent grant without a persisted result blocks without retry; a persisted completion continues without replay. Real execution remains gated on validating recovery and safe budget enforcement in Phase 0.

## Prioritized Findings & AI Suggestions

### 1. Local authentication and credential lifecycle — High

* **Reviewer:** `egm-implementation-readiness`
* **Affected Scope:** Phase 0 and Phase 2; [design](docs/designs/harness-pro.md#L285-L322)
* **Discrepancy Description:** The plan requires authentication on execution routes but leaves the local authentication mechanism to be settled. It does not describe desktop credential provisioning, storage, or failure responses.
* **Rationale:** The existing `RequireAuth` accepts a Bearer token from `BRIDGE_AUTH_TOKEN`, but falls back to a fixed token; current Tauri startup does not provide a client credential. Reusing the middleware alone does not complete local authentication.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** The document needs a selected, implementable local authentication path before execution routes are exposed. A concrete proposal is to keep `RequireAuth` Bearer validation, remove the fixed fallback, have Tauri provision a per-launch secret, deliver it to the WebView through a Tauri command, and fail closed when unavailable. This needs owner approval because it changes the local trust boundary.
* ***Proposed Edit:*** Replace the Phase 0 item about “settling” authentication with the concrete mechanism above, add its Tauri/client files and API error behavior to the file list, and require unauthenticated calls to return `401`.

### 2. Run, persistence, and API contracts are not complete — High

* **Reviewer:** `egm-implementation-readiness`
* **Affected Scope:** Phase 1 and Phase 2; [design](docs/designs/harness-pro.md#L291-L322)
* **Discrepancy Description:** The plan names types, tables, routes, and OpenAPI, but leaves fields, transition rules, constraints, request/response schemas, errors, idempotency behavior, and event sequencing to the implementer.
* **Rationale:** Naming artifacts does not define their contracts; a fresh implementation would still require architectural choices.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Add a compact contract section to the design: typed run/step/permission/event fields; allowed transitions and terminal-state rules; uniqueness, foreign-key, and retention behavior; all API request/response/error schemas; idempotency semantics; and WebSocket cursor/replay semantics. The OpenAPI file can then be generated from this approved contract during implementation.
* ***Proposed Edit:*** Add `Run and API Contracts` before Detailed Implementation and enumerate the contract fields and transition rules using the already selected SQLite and REST/WS architecture.

### 3. Policy checks may miss operations that do not request approval — Medium

* **Reviewers:** `egm-zoom-out` (Stage 5), `egm-design`
* **Affected Scope:** Technical Plan lines 50 and 192–196; Phase 1 policy and app-server adapter.
* **Discrepancy Description:** Checking approval requests does not prove that operations already allowed by the sandbox are checked against the project ceiling and droid subset.
* **Rationale:** App-server may execute an allowed operation without producing an approval event.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Keep Codex sandbox enforcement as the boundary for operations that need no prompt, and require the prototype to prove every exposed tool maps to the effective project/droid policy. Disable real runs if any capability cannot be constrained.
* ***Proposed Edit:*** Add an acceptance condition covering both approval-triggering and auto-allowed read, write, command, web-search, and network operations; state that an unmappable operation is unavailable in v1.

### 4. WebSocket authentication and project filtering are unspecified — Medium

* **Reviewers:** `egm-design`, `egm-zoom-out` (Stage 8)
* **Affected Scope:** Selected event flow and Phase 2; [design](docs/designs/harness-pro.md#L108-L120), [design](docs/designs/harness-pro.md#L310-L322)
* **Discrepancy Description:** REST routes require authentication, but the existing `/ws` accepts unauthenticated connections and broadcasts world events to every subscriber. No desktop credential or project-filtering behavior is defined.
* **Rationale:** Run events may contain private project/task details and must be protected consistently with REST snapshots.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Add authenticated, project-scoped run subscriptions. One concrete option is an authenticated REST call that mints a short-lived, single-use WebSocket ticket scoped to a project; the WebSocket consumes it once and emits only authorized run events. Keep public/ambient world events separate if they remain unauthenticated.
* ***Proposed Edit:*** Specify the ticket contract, expiry/use rules, project authorization, event filtering, and credential redaction in `server/openapi.yaml` and the WebSocket protocol.

### 5. Snapshot/replay/live subscription can miss events — Medium

* **Reviewers:** `egm-comprehension`, `egm-design`, `egm-zoom-out` (Stage 8)
* **Affected Scope:** Selected event flow, persistence contract, Phase 2, and client socket.
* **Discrepancy Description:** Fetching a snapshot and replaying before following the WebSocket leaves a gap where a committed event can be missed, including a terminal event.
* **Rationale:** A later sequence may never arrive to trigger gap recovery.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Define a race-free handoff with a high-water sequence and client deduplication.
* ***Proposed Edit:*** Specify: connect and subscribe first; receive the server's high-water cursor; fetch the snapshot at that cursor; replay persisted events through the current cursor; buffer and apply live events above the cursor; deduplicate by `(run_id, sequence)`; repeat replay on any gap.

### 6. One-time grants versus configured policy are ambiguous — Medium

* **Reviewers:** `egm-comprehension`, `egm-zoom-out` (Stage 8)
* **Affected Scope:** Authorization model, coordinator, and permission panel; [design](docs/designs/harness-pro.md#L21-L24), [design](docs/designs/harness-pro.md#L186-L196), Phase 1/3.
* **Discrepancy Description:** The backend checks requests against project and droid limits, while the UI offers a grant for a specific action. It is unclear whether the grant can override configured policy.
* **Rationale:** The brief's task-scoped grant is distinct from editing a droid profile, but its precedence over the profile is not stated.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Clarify that a one-time task grant may add the exact operation to the droid's effective access for this run only, but can never exceed the project ceiling. An operation outside the project ceiling cannot be granted in the task dialog; the user must change project policy first. A profile edit persists only after an explicit settings change.
* ***Proposed Edit:*** Add this precedence rule to Authorization Model, permission-decision API, and ProjectExecutionSettings.

### 7. Workflow and budget rules remain open — Medium

* **Reviewer:** `egm-implementation-readiness`
* **Affected Scope:** Phase 1 lines 297–301/308; Phase 3 settings.
* **Discrepancy Description:** Alternative selection, no-alternative behavior, repair definition, budget units/configuration, and exhaustion state are unspecified.
* **Rationale:** These affect state transitions, cost, and whether the run can proceed.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Define the actor and eligibility for alternatives, require a distinct operation/capability rationale, block when no permitted alternative remains, define repair as one Coder pass after failed verification followed by the same preset, and define budget fields/units and terminal behavior. Numeric project ceilings are configured by the owner; product-wide defaults and hard maxima are not required.
* ***Proposed Edit:*** Add exact behavioral rules and require project-level call, elapsed-time, and token ceilings before a run can start. Droid/run overrides may only tighten those limits; keep real execution gated on validating app-server usage enforcement during Phase 0.

### 8. App-server restart recovery lacks a test matrix — Medium

* **Reviewer:** `egm-implementation-readiness`
* **Affected Scope:** Phase 0 prototype and Phase 1 app-server adapter/recovery.
* **Discrepancy Description:** Protocol messages, persisted identifiers, reconciliation cases, safe-resume criteria, and required evidence are not defined.
* **Rationale:** The plan makes restart safety a release gate but gives no reproducible procedure for proving it.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Enumerate shutdown cases and expected outcomes so recovery logic can be built and reviewed without guessing.
* ***Proposed Edit:*** Add cases for a pending approval, an operation not yet started, an operation completed with a lost acknowledgement, an uncertain in-flight mutation, and a finished turn. Resume only from persisted evidence proving no duplicate side effect; otherwise mark `BLOCKED`.

### 9. Test files and assertions are not specified — Medium

* **Reviewer:** `egm-implementation-readiness`
* **Affected Scope:** Acceptance and verification plan; all phases.
* **Discrepancy Description:** The plan lists journeys but no test files, fixtures, layers, setup, or concrete assertions for redaction, limits, usage, restart, and replay.
* **Rationale:** The implementation-readiness standard requires specific happy-path and edge-case tests.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Name server unit and API integration coverage, reuse the existing bridge E2E conventions, and make expected results explicit.
* ***Proposed Edit:*** Add `server/src/run/tests.rs` and `server/tests/e2e_runs_test.rs` to the plan. Cover transition/policy/budget unit cases and end-to-end start, approval, denial, idempotency, WebSocket authorization, replay race, migration preservation, bounded/redacted output, and restart outcomes.

### 10. Canonical-root validation is inline, not a shared helper — Medium

* **Reviewer:** `egm-codebase-details`
* **Affected Scope:** Phase 2 `server/src/api/projects.rs`.
* **Discrepancy Description:** Canonicalization and containment checks are inline in `read_project_file`; `Database::project_root` returns a stored path without canonicalizing it.
* **Rationale:** The plan calls this an existing reusable point, so implementation would require inventing where shared validation belongs.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Plan one explicit shared helper for API and run policy use.
* ***Proposed Edit:*** Create `server/src/security/project_paths.rs` with a canonical root resolver/containment validator; call it from project file reads, run creation, and every execution adapter. Add the file to module registration and tests.

### 11. SQLite migration versioning is not specified — Medium

* **Reviewer:** `egm-codebase-details`
* **Affected Scope:** Phase 1 `server/src/db/sqlite.rs`.
* **Discrepancy Description:** Existing initialization uses `CREATE TABLE IF NOT EXISTS` and has no migration registry or sequence.
* **Rationale:** “Versioned, additive migrations” does not tell the implementer how to detect/apply versions or preserve existing data transactionally.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** SQLite already supplies `PRAGMA user_version`, which can version ordered migrations without introducing a dependency.
* ***Proposed Edit:*** Specify ordered migrations in `Database::init`, each applied transactionally according to `PRAGMA user_version`; preserve existing project, droid, message, and thread rows, and test upgrading a database created by the current schema.

### 12. Feature risks are not in a clearly labeled section — Medium

* **Reviewer:** `egm-structural`
* **Affected Scope:** FeatureRange, lines 1–280.
* **Discrepancy Description:** Risks and mitigations are scattered across decision trade-offs and failure conditions.
* **Rationale:** The feature-spec standard expects an explicit risks/mitigations section.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** A visible section will make the known security, recovery, cost, migration, and city-state risks easier to audit.
* ***Proposed Edit:*** Add `## Risks & Mitigations` before `Alternatives Considered`, consolidating the already documented operation/path risk, false success, restart duplication, permission scope, budgets, logging, migration, and visual-truth risks with their existing mitigations.

### 13. Alternatives repeat decision rationales — Low

* **Reviewers:** `egm-style`, `egm-zoom-out`
* **Affected Scope:** `Alternatives Considered`, lines 213–279.
* **Discrepancy Description:** Several alternatives repeat rationale already given under Technical Plan decisions.
* **Rationale:** Duplication makes the selected rationale harder to maintain.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Keep the required alternatives evaluation while removing duplicate prose.
* ***Proposed Edit:*** Condense each item to option, decisive trade-off, and link/reference to the corresponding decision instead of restating full analysis.

### 14. Legacy endpoint migration caveat is duplicated — Low

* **Reviewer:** `egm-style`
* **Affected Scope:** Technical Plan lines 131 and 209.
* **Discrepancy Description:** The same client migration caveat appears twice.
* **Rationale:** Repetition creates two locations to maintain.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Retain the caveat with the API contract only.
* ***Proposed Edit:*** Remove the duplicate from the other location and keep the full compatibility note in Decision 3.

### 15. Broad evaluative terms can be made observable — Low

* **Reviewer:** `egm-style`
* **Affected Scope:** Lines 9, 13, and 35.
* **Discrepancy Description:** “Trustworthy,” “truthful,” and “reliably” are broad terms where the document already defines observable outcomes.
* **Rationale:** Concrete conditions are easier to review than evaluative claims.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Replace each with its evidence condition.
* ***Proposed Edit:*** Replace with language about recorded operations, captured verification outcomes, durable run state, and terminal/city event agreement.

### 16. Comparison phrases lack explicit criteria — Low

* **Reviewer:** `egm-style`
* **Affected Scope:** Lines 64 and 79.
* **Discrepancy Description:** “Closest supported integration” and “costs more than the control it adds” do not state measurable comparison criteria.
* **Rationale:** The trade-off is difficult to assess without naming protocol capability or engineering cost.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** State the specific capability and maintenance burden being compared.
* ***Proposed Edit:*** Replace with the concrete fact that app-server surfaces approval events/decisions while requiring a long-lived JSON-RPC adapter; the Harness-owned loop requires implementing provider/tool adapters, sandboxing, and recovery.

### 17. Implementation summary repeats details and is hard to scan — Low

* **Reviewer:** `egm-style`
* **Affected Scope:** Detailed Implementation lines 299, 306, 322, and 349–353.
* **Discrepancy Description:** Dependency wording repeats itself, the Phase 2 paragraph repeats the table, and acceptance checks are dense sentences.
* **Rationale:** Grouped checks are easier to verify and duplicated text can drift.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Keep the additional trust/recovery rules while reducing repeated wording.
* ***Proposed Edit:*** State the dependency constraint once; trim the Phase 2 paragraph to requirements not in its table; convert acceptance checks into grouped bullets.

## Resolution gate

Reply **auto accept** to apply all proposed edits, **continue** to leave the design unchanged, or **interactive breakdown** to decide each item one at a time. Findings 1 and 7 include implementation choices that require owner judgment; review those carefully before accepting all proposals.
