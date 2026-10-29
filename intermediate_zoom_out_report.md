# Stage 8 — Intermediate Zoom Out Review

**Target:** `docs/designs/harness-pro.md` (lines 1–360)
**Reviewer:** `egm-zoom-out` — completed.
**Persona and standard:** Read in full. The reviewer also read the Stage 5–7 reports and inspected the existing API, authentication, project, SQLite, and WebSocket code.

## Findings

### Major — WebSocket authentication and project isolation remain unspecified

**Affected scope:** Lines 110–114, 142, 209–211, and 319–322.

REST authentication is explicit, but the existing `/ws` route accepts unauthenticated connections and broadcasts world events to every connection. The design does not say how the desktop supplies credentials or how run events are filtered by project.

**Recommendation:** Define WebSocket authentication and per-project authorization in the event/API contract. Reuse `RequireAuth` or specify an authenticated handshake and filtering rule.

### Major — Snapshot/replay/live event handoff can still lose events

**Affected scope:** Lines 110 and 211; Phase 2 lines 319–322; client socket line 331.

Fetching a snapshot and replaying before following the WebSocket leaves a window where a new event can be committed after replay and before subscription. Gap replay only helps if a later sequence arrives, so a final event could remain missing.

**Recommendation:** Specify a subscribe-first/high-water reconciliation or another atomic handoff and deduplicate by sequence, including terminal events.

### Major — A user grant's relationship to configured policy remains ambiguous

**Affected scope:** Lines 21–24, 192–194, 308, and 334–336.

The coordinator checks a Codex approval against the project ceiling and droid subset, but the UI describes granting the requested action. It is unclear whether this is an exception to configured policy or a one-time approval only for an action already within policy.

**Recommendation:** Define grantability and precedence. A user decision must not silently widen a hard project or droid policy.

### Nit — Architecture choices, alternatives, and implementation details repeat

**Affected scope:** Lines 58–106, 127–184, and 281–359.

Decisions are described in the technical plan and repeated in Alternatives Considered, then repeated in file, route, and acceptance details. This makes the architecture harder to maintain and overlaps with the planned OpenAPI contract.

**Recommendation:** Keep decisions, constraints, and key acceptance gates in the design; move file-level and endpoint detail into a linked implementation/API plan.

## Reuse observations

The existing `RequireAuth` Bearer middleware, SQLite database, canonical project-root validation, and WebSocket are reusable foundations. The current WebSocket path is unauthenticated, in-memory, and can drop lagged messages; it does not by itself satisfy run event isolation or replay.
