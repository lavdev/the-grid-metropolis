# Goldfish Review Report — The Grid Metropolis Harness

**Target:** `docs/designs/harness-pro.md` (360 lines)
**Mode:** Consolidated report with all review rubrics
**Review flow:** Stages 5–9 completed. The first Stage 5 attempt did not read the full file and was retried; the retry read all 360 lines. No Goldfish modified the design.

## Reviewer status

| Stage | Reviewer | Status |
| --- | --- | --- |
| 5 — Initial Zoom Out | `egm-zoom-out` | Completed on retry; first attempt incomplete |
| 6 — Comprehension | `egm-comprehension` | Completed |
| 7 — Design Critic | `egm-design` | Completed |
| 7 — Structural | `egm-structural` | Completed |
| 7 — Style | `egm-style` | Completed |
| 8 — Intermediate Zoom Out | `egm-zoom-out` | Completed |
| 9 — Implementation Readiness | `egm-implementation-readiness` | Completed |
| 9 — Codebase Details | `egm-codebase-details` | Completed |
| 9 — Style | `egm-style` | Completed |

## Prioritized findings

### Blockers

#### B1 — Local authentication and credential lifecycle are undecided

**Reviewers:** `egm-implementation-readiness`
**Affected scope:** Detailed Implementation, Phase 0 and Phase 2.

The plan requires authentication on execution routes but leaves the local authentication mechanism “to be settled.” It does not define how the desktop obtains or stores credentials or what authentication failures look like.

**Required action:** Decide and document local credential provisioning/storage, request authentication, and failure responses before implementation.

**Traceability:** Stage 9 readiness finding 1.

#### B2 — Durable run, database, and API contracts are not specified enough to implement

**Reviewers:** `egm-implementation-readiness`
**Affected scope:** Detailed Implementation, backend phases 1–2.

The plan names tables, types, endpoints, and `server/openapi.yaml`, but leaves their fields, constraints, request/response schemas, transition rules, error codes, idempotency behavior, and event sequencing to the implementer.

**Required action:** Define the run/step/permission/event schemas, state transition table, SQLite columns/constraints, and complete API/event contract, or link an already approved contract.

**Traceability:** Stage 9 readiness finding 2.

### Major findings

#### M1 — Policy enforcement is not demonstrated for operations that do not request approval

**Reviewers:** `egm-zoom-out` (Stage 5), `egm-design`, `egm-zoom-out` (Stage 8)
**Affected scope:** Technical Plan lines 50 and 192–196; Detailed Implementation policy and app-server work.

The design checks approval requests before answering Codex, but an operation already allowed by the sandbox may not generate an approval request. The plan does not prove that per-turn sandbox policy fully enforces both the project ceiling and droid subset in that case.

**Required action:** Define and prototype enforcement for both approval-triggering and auto-allowed operations; include both cases in the release gate.

**Traceability:** Stage 5 Major 1; Stage 7 design finding 1.

#### M2 — WebSocket run events lack an authentication and project-isolation contract

**Reviewers:** `egm-design`, `egm-zoom-out` (Stage 8)
**Affected scope:** Technical Plan lines 110–114 and 142; event contract and Phase 2.

REST routes require authentication, but `/ws` currently accepts unauthenticated connections and broadcasts world events to subscribers. The plan does not explain desktop credentials or project-level filtering for private run events.

**Required action:** Specify WebSocket authentication and per-project authorization/filtering in the event/API contract, including how the desktop supplies credentials.

**Traceability:** Stage 7 design finding 3; Stage 8 Major 1.

#### M3 — Snapshot/replay/live subscription can miss events during reconnect

**Reviewers:** `egm-comprehension`, `egm-design`, `egm-zoom-out` (Stage 8)
**Affected scope:** Selected event flow, run event contract, client socket recovery.

The client fetches a snapshot, replays events, and then follows the WebSocket. An event committed after replay but before live subscription can be missed; recovery only on a later sequence would not necessarily catch a terminal event.

**Required action:** Define an atomic subscribe/high-water/snapshot reconciliation protocol or another race-free handoff. Require sequence-based deduplication and recovery of terminal events.

**Traceability:** Stage 6 Major 2; Stage 7 design finding 2; Stage 8 Major 2.

#### M4 — A one-time user grant's relationship to project and droid policy is ambiguous

**Reviewers:** `egm-comprehension`, `egm-zoom-out` (Stage 8)
**Affected scope:** Authorization model, run coordinator, permission panel.

The backend checks app-server approval requests against project and droid limits, while the UI offers a grant for a specific action. It is unclear whether a user can override configured policy or only approve an action already within it.

**Required action:** Define which requests are grantable and state explicitly whether project/droid limits are hard ceilings that a user decision cannot widen.

**Traceability:** Stage 6 Major 1; Stage 8 Major 3.

#### M5 — Workflow rules and operational budgets still require design decisions

**Reviewer:** `egm-implementation-readiness`
**Affected scope:** Detailed Implementation, Phases 1 and 3.

“Bounded alternatives,” one new request, and one repair pass do not fully specify who chooses alternatives, exact transition rules, or what qualifies as a repair. Call/time/token budgets lack defaults, units, allowed ranges, hard server maxima, and exhaustion behavior.

**Required action:** Define those rules and values/ranges before implementation; specify resulting run states and inheritance/override behavior.

**Traceability:** Stage 9 readiness finding 3.

#### M6 — App-server recovery lacks a reproducible protocol and test matrix

**Reviewer:** `egm-implementation-readiness`
**Affected scope:** Phase 0 prototype and Phase 1 adapter/recovery.

The plan does not specify protocol messages, persisted identifiers, reconciliation cases, safe-resume criteria, or evidence required to claim a restart is safe.

**Required action:** List restart scenarios and expected outcomes; define when the run resumes, stays permission-paused, cancels, or becomes `BLOCKED`.

**Traceability:** Stage 9 readiness finding 4.

#### M7 — Test implementation is not assigned to files or concrete assertions

**Reviewer:** `egm-implementation-readiness`
**Affected scope:** Acceptance and verification plan and all implementation phases.

The plan describes journeys and failure cases but does not name test files, fixtures, test layers, or exact inputs and expected results. Output redaction, bounds, usage accounting, restart, and replay are not assigned explicit checks.

**Required action:** Add file-by-file test changes and expected results for policies, idempotency, migration, replay races, budget exhaustion, redaction, and restart recovery.

**Traceability:** Stage 9 readiness finding 5.

#### M8 — The plan treats canonical-root validation as reusable when it is currently inline

**Reviewer:** `egm-codebase-details`
**Affected scope:** `server/src/api/projects.rs`, Phase 2.

Canonicalization and containment checks are inline in `read_project_file`; `Database::project_root` returns the stored path without canonicalization.

**Required action:** Specify extracting and reusing a canonical-root validator, or explicitly describe it as new logic.

**Traceability:** Stage 9 codebase finding 1.

#### M9 — The migration mechanism is unspecified and not present today

**Reviewer:** `egm-codebase-details`
**Affected scope:** `server/src/db/sqlite.rs`, Phase 1.

The current schema uses `CREATE TABLE IF NOT EXISTS`; there is no migration version registry or ordered migration sequence.

**Required action:** Choose a version-tracking mechanism and document migration order, transaction behavior, and preservation of project/message history.

**Traceability:** Stage 9 codebase finding 2.

#### M10 — The feature specification has no explicit risks section

**Reviewer:** `egm-structural`
**Affected scope:** FeatureRange, lines 1–280.

Risks and failure conditions appear within decisions and alternatives, but the feature-spec standard expects a visible risks/mitigations section.

**Required action:** Add `Risks & Mitigations` or a clearly labeled index to the existing risk statements.

**Traceability:** Stage 7 structural finding.

### Nits

#### N1 — Alternatives repeat the decision rationale

**Reviewers:** `egm-style`, `egm-zoom-out` (Stages 5, 8)
**Affected scope:** Alternatives Considered and Technical Plan.

Alternatives restate choices and trade-offs already explained in Decisions 1–5.

**Recommendation:** Shorten this section or make it an index to the selected decision sections.

#### N2 — Legacy endpoint migration caveat is duplicated

**Reviewer:** `egm-style`
**Affected scope:** Lines 131 and 209.

The same client migration requirement appears twice.

**Recommendation:** Keep it in one place.

#### N3 — Broad evaluative terms can be replaced with observable outcomes

**Reviewer:** `egm-style`
**Affected scope:** Lines 9, 13, 35.

“Trustworthy,” “truthful,” and “reliably” are less precise than the recorded operations and verification outcomes that define them.

**Recommendation:** Prefer those observable conditions.

#### N4 — Some comparison phrases lack criteria

**Reviewer:** `egm-style`
**Affected scope:** Lines 64 and 79.

“Closest supported integration” and “costs more than the control it adds” do not name the comparison criteria.

**Recommendation:** Name the capability or cost being compared.

#### N5 — Implementation section repeats details and could be easier to scan

**Reviewers:** `egm-zoom-out` and `egm-style`
**Affected scope:** Detailed Implementation.

The Phase 2 narrative repeats its table; the acceptance checks use long, dense sentences; some dependency wording repeats itself.

**Recommendation:** Remove duplicate prose, group checks into bullets, and state dependency constraints once.

## Traceability matrix

| Consolidated item | Raw source |
| --- | --- |
| B1 | Stage 9 readiness: auth mechanism |
| B2 | Stage 9 readiness: schemas and contracts |
| M1 | Stage 5 Major 1; Stage 7 design finding 1 |
| M2 | Stage 7 design finding 3; Stage 8 Major 1 |
| M3 | Stage 6 Major 2; Stage 7 design finding 2; Stage 8 Major 2 |
| M4 | Stage 6 Major 1; Stage 8 Major 3 |
| M5 | Stage 9 readiness: workflow and budgets |
| M6 | Stage 9 readiness: resume/recovery |
| M7 | Stage 9 readiness: test plan |
| M8 | Stage 9 codebase finding 1 |
| M9 | Stage 9 codebase finding 2 |
| M10 | Stage 7 structural finding |
| N1 | Stage 5 Nit 2; Stage 7 style Major; Stage 8 Nit 4 |
| N2 | Stage 7 style Nit 1 |
| N3 | Stage 7 style Nit 2 |
| N4 | Stage 7 style Nit 3 |
| N5 | Stage 9 style Nits 1–3 |

## Review artifacts

* [Stage 5 — Zoom Out](zoom_out_report.md)
* [Stage 6 — Comprehension](comprehension_report.md)
* [Stage 7 — Critic](critic_report.md)
* [Stage 8 — Intermediate Zoom Out](intermediate_zoom_out_report.md)
* [Stage 9 — Implementation Readiness](implementation_readiness_report.md)
