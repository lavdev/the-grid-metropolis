# API

## Overview

This module builds the Axum router for bridge dispatch, Codex CLI integration, project management, run snapshots and event replay, execution settings, file reads, and WebSocket updates (`mod.rs::api_routes`). The implemented API contract lives in [`server/openapi.yaml`](../../openapi.yaml).

## Interface

- `api_routes` merges the bridge/WebSocket router and project router (`mod.rs::api_routes`).
- Bridge: `POST /api/v1/bridge/dispatch` uses `dispatch_handler` and `RequireAuth` (`mod.rs::api_routes`, `dispatch.rs::dispatch_handler`, `auth.rs::RequireAuth`).
- Codex: `GET /api/v1/codex/status` and `POST /api/v1/codex/stream` (`mod.rs::api_routes`, `codex.rs::codex_status_handler`, `codex.rs::codex_stream_handler`).
- Projects: list/save, delete, message add/delete/clear, thread update, and project-file read handlers are registered by `api_routes` (`mod.rs::api_routes`, [`projects`](projects.rs)).
- Execution settings: `GET` and `PUT /api/v1/projects/{id}/execution-settings` read and persist project ceilings; the endpoints use strict `RequireRunAuth` and do not start runs ([`execution_settings`](execution_settings.rs), [`auth`](auth.rs)).
- Execution policy: `GET`/`PUT /api/v1/projects/{project_id}/execution-policy` save the project ceiling, and the corresponding `/droids/{droid_id}/execution-policy` routes save optional narrower overrides. Missing Droid overrides inherit; every configured override is checked against the project ceiling ([`execution_policy`](execution_policy.rs), [`run::policy`](../run/policy.rs)).
- Run submission: `POST /api/v1/projects/{project_id}/runs` requires a configured budget, project policy, assigned target Droid, supported server-owned verification preset, and `Idempotency-Key`. It snapshots the effective policy and returns `202` after durable creation. The worker is disabled unless `GRID_METROPOLIS_RUNS_ENABLED=true`.
- Run policy enforcement: `run::worker` resolves the saved policy snapshot and starts an isolated Codex app-server using `run::codex_app_server::start_with_policy`. Known command/file-change approval requests are persisted before the worker waits on `RunDispatcher`; unknown methods are declined. A grant/deny is sent to the exact live session and is marked sent only after `serverRequest/resolved` acknowledgement.
- Run reads: `GET /api/v1/runs/{run_id}` returns the persisted snapshot; `GET /api/v1/runs/{run_id}/events?afterSequence=<n>&limit=<n>` replays events exclusively after the cursor in sequence order ([`runs`](runs.rs)).
- Permission decisions: `POST /api/v1/runs/{run_id}/permission-requests/{request_id}/decisions` records an authenticated, idempotent `GRANT` or `DENY`; grants are scoped to the exact capability/action/resource tuple ([`runs`](runs.rs)).
- WebSocket: `GET /ws` is handled by `ws_handler` (`mod.rs::api_routes`, `ws.rs::ws_handler`).
- `ApiError` defines API rejection cases (`error.rs::ApiError`).

## Invariants

- Dispatch rejects blank `projectId`, `targetAgentId`, `taskId`, or task title and requires `RequireAuth` (`dispatch.rs::dispatch_handler`, `auth.rs::RequireAuth::from_request_parts`).
- `RequireAuth` accepts only an `Authorization` value starting with `Bearer ` and compares its token to `BRIDGE_AUTH_TOKEN` or the configured fallback (`auth.rs::RequireAuth::from_request_parts`).
- `RequireRunAuth` fails closed when `BRIDGE_AUTH_TOKEN` is absent or blank; execution-settings errors use a stable envelope with `code`, `message`, and `requestId` (`auth.rs::RequireRunAuth`, [`run_error`](run_error.rs)).
- Saved project budgets must be positive and within SQLite's integer range. Run submission is feature-gated and disabled by default.
- Run reads and permission decisions require strict local auth. Event replay uses a per-run cursor and caps page size at 500. A decision retry must reuse the same `Idempotency-Key`; a conflicting key or decision returns `409`.
- Recording a decision does not imply delivery to the app-server. If a live or recovered session has registered that exact approval, the dispatcher sends the JSON-RPC response and waits for its acknowledgement before marking it sent; otherwise the choice remains durable with `responseSent: false`. On restart, the worker resumes only an unsent approval whose saved item is present in an incomplete turn with no completed mutation; it compares the canonical action fingerprint before reusing the saved request ID.

## Side Effects

- Dispatch takes a write lock on shared world state, applies a task dispatch, and returns HTTP 202 on success (`dispatch.rs::dispatch_handler`).
- Codex status probes the discovered CLI with `--version`; the stream handler starts a Codex process and streams its output (`codex.rs::codex_status_handler`, `codex.rs::codex_stream_handler`).
- Project handlers read project files and delegate project/message persistence to `Database` (`projects.rs::read_project_file_handler`, `projects.rs::list_projects_handler`, `projects.rs::save_project_handler`).
- Execution settings are persisted per project in SQLite; the GET response explicitly marks an unset configuration as `configured: false`.
- Permission choices and their audit events are persisted atomically. The permission summary omits the canonical app-server payload; the exact grant scope is derived server-side from the displayed request.
- WebSocket clients receive an initial snapshot and subsequent serialized broadcast events (`ws.rs::handle_socket`).

## Verification

- Run from `server/`: `cargo test`

