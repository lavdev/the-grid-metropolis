# Stage 9 — Implementation Readiness Test

**Target:** `docs/designs/harness-pro.md` (DetailedRange, lines 281–360)
**Reviewers:** `egm-implementation-readiness`, `egm-codebase-details`, and `egm-style` — all completed.
**Standard:** `implementation-readiness-standard.md` for readiness and codebase reviewers; no separate style standard assigned.

## Blockers

### Local authentication and credential lifecycle are not decided

**Reviewer lens:** `egm-implementation-readiness`
**Affected scope:** Phase 0 line 289; Phase 2 lines 315–318.

The plan says the authentication mechanism remains to be settled while requiring authentication on every execution route. An implementer must choose the mechanism and credential lifecycle.

**Guidance:** Specify how local credentials are provisioned and stored, how desktop/API requests authenticate, and expected failure responses before implementation.

### Run, persistence, and API contracts are incomplete

**Reviewer lens:** `egm-implementation-readiness`
**Affected scope:** Phase 1 lines 295–306; Phase 2 lines 314–320.

The plan names types, tables, routes, and an OpenAPI file without defining the run/step/event fields, transition rules, database columns and constraints, request/response schemas, error codes, idempotency keys, or event sequence behavior. Creating these contracts is left to the implementer.

**Guidance:** Specify the contracts and migration details or reference an already approved source of truth.

## Major findings

### Workflow, permissions, and budgets leave implementation choices open

**Reviewer lens:** `egm-implementation-readiness`
**Affected scope:** Phase 1 lines 297–301 and 308; Phase 3 line 336.

“Bounded alternatives,” a “specific new request,” and “at-most-one repair” do not define who selects alternatives, what happens when none are available, the exact transition limits, or what constitutes a repair. Budget and permission settings also lack defaults, units, allowed ranges, and server maximums.

**Guidance:** Define operation/transition rules, budget defaults and bounds, policy inheritance/override behavior, and the state entered on exhaustion.

### App-server resume and recovery cases are not reproducible

**Reviewer lens:** `egm-implementation-readiness`
**Affected scope:** Phase 0 line 287; Phase 1 lines 300 and 305.

The plan lacks protocol messages, persisted identifiers, reconciliation cases, safe-resume criteria, and a reproducible test for the Phase 0 restart claim.

**Guidance:** List scenarios and expected outcomes, then define when the adapter resumes, waits for permission, cancels, or marks a run `BLOCKED`.

### Test files, fixtures, layers, and assertions are missing

**Reviewer lens:** `egm-implementation-readiness`
**Affected scope:** Acceptance plan lines 349–353 and all phases.

Acceptance journeys are stated, but no test files, layers, fixtures, setup, or concrete assertions are named. Redaction, output bounds, usage accounting, restart, and replay are not assigned verification cases.

**Guidance:** Add file-by-file test changes and input/expected-result cases, including app-server recovery, authorization, and replay races.

### Canonical project-root validation is not an existing reusable helper

**Reviewer lens:** `egm-codebase-details`
**Affected scope:** `server/src/api/projects.rs`, Phase 2 line 316.

Canonicalization and containment checks currently exist inline in `read_project_file`; `Database::project_root` returns the stored path without canonicalizing it.

**Guidance:** Specify extracting and reusing a canonical-root validator, or describe it as new logic.

### Versioned SQLite migrations lack a specified mechanism

**Reviewer lens:** `egm-codebase-details`
**Affected scope:** `server/src/db/sqlite.rs`, Phase 1 line 303.

The current schema setup uses `CREATE TABLE IF NOT EXISTS` and has no migration version registry or ordered migration sequence.

**Guidance:** Specify the version tracking mechanism, migration ordering/transaction behavior, and preservation of existing project and conversation data.

## Nits

* **Reviewer lens:** `egm-style`; **Scope:** Phase 1 lines 299 and 306. “Prevent runaway work” is colloquial; the dependency row repeats the requirement to keep the dependency set minimal.
* **Reviewer lens:** `egm-style`; **Scope:** Phase 2 line 322. The paragraph repeats the API table; retain only additional trust/recovery rules there.
* **Reviewer lens:** `egm-style`; **Scope:** Acceptance plan lines 351–353. Split the dense sentences into grouped, checkable items.

No current-file references were found to be nonexistent, and all proposed new files in the reviewed range are marked `Create`.
