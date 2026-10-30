# Server Blueprint

## System Overview

The `server` Rust package exposes the bridge and project APIs, holds simulation state in memory, and stores project data through SQLite. Its binary assembles the Axum router and listens on `127.0.0.1` using `PORT` or port `3000` (`src/main.rs::main`, `src/db/sqlite.rs::Database`, `src/api/mod.rs::api_routes`).

## Subdirectories & System Boundaries

- [`src`](src/README.md) — library and executable modules for API transport, persistence, domain types, and shared simulation state.

## Global Invariants

- The bridge dispatch route requires a Bearer token checked by `RequireAuth`; the expected value comes from `BRIDGE_AUTH_TOKEN`, with a source-defined fallback (`src/api/auth.rs::RequireAuth::from_request_parts`, `src/api/dispatch.rs::dispatch_handler`).
- Durable run submission requires `BRIDGE_AUTH_TOKEN` and `GRID_METROPOLIS_RUNS_ENABLED=true`; the worker stays disabled when the latter is unset. The worker accepts only the server-owned `cargo-test` and `npm-check` verification presets. Known command/file-change approvals are durably recorded and can pause for an authenticated grant/deny. After restart, the worker resumes the saved thread and reissues the original turn only when the incomplete history contains the saved approval item and no completed mutation; it delivers a new request only if the canonical fingerprint matches. One repair attempt is allowed after verification failure, bounded by the run's turn, token, and elapsed-time budgets. Every run then requires a separate read-only Reviewer turn, selected by `reviewerDroidId` or the first distinct `REVISOR_DE_CODIGO` Droid. UI and end-to-end validation remain pending; a reviewer interrupted by restart is conservatively blocked unless its approval was durably recorded.
- Dispatch validates that the project, target agent, task ID, and title are non-empty before calling `WorldState::dispatch_task` (`src/api/dispatch.rs::dispatch_handler`).

## Global Side Effects

- Startup initializes the local SQLite database, creates shared simulation state, and binds an HTTP listener on loopback (`src/main.rs::main`, `src/db/sqlite.rs::Database::init`).
- API operations can persist project records and messages in SQLite; bridge operations mutate in-memory world state and broadcast events (`src/db/sqlite.rs::Database`, `src/state/world.rs::WorldState`).

## System Verification

- Run from `server/`: `cargo test`

