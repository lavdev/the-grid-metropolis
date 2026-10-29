# Stage 5 — Initial Zoom Out Review

**Target:** `docs/designs/harness-pro.md` (lines 1–360)
**Reviewer:** `egm-zoom-out` — completed on retry after an incomplete first read.
**Persona and standard:** Read in full.

## Findings

### Major — Policy enforcement may miss operations that do not request approval

**Affected scope:** Lines 50, 192–196, 308.

The design says the coordinator checks app-server approval requests, while also promising policy enforcement before every operation. Operations already allowed by Codex's sandbox may not raise an approval request, so the described approval adapter alone does not establish that the project ceiling and droid subset are enforced for every operation.

**Recommendation:** State how every executable operation is constrained. Either prove that per-turn sandbox settings enforce the full project and droid policy for operations that need no approval, or route those operations through a coordinator-controlled check. Include both cases in the app-server prototype acceptance criteria.

### Nit — Repeated decisions and implementation detail make the design harder to maintain

**Affected scope:** Lines 60–79, 87–106, 152–184, 213–359.

Alternatives repeat decisions already described in the technical plan. The implementation section also enumerates files, endpoints, components, and release checks, creating more places for the same design to drift.

**Recommendation:** Keep the selected architecture and key trade-offs easy to find. Consider moving detailed file and contract material into a linked implementation plan or API specification.

## Reuse observations

The repository already has `RequireAuth` in `server/src/api/auth.rs`, SQLite initialization in `server/src/db/sqlite.rs`, project lookup and root checks in `server/src/api/projects.rs`, and a WebSocket event envelope/broadcast path in `server/src/domain/event.rs` and `server/src/api/ws.rs`. Existing WebSocket broadcasts are in-memory and can drop lagged events, so durable SQLite event records and replay are still needed. The current Codex route and client confirm the `working_dir` and sandbox-bypass risks described by the design.
