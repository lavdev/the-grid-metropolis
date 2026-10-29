# API

## Overview

This module builds the Axum router for bridge dispatch, Codex CLI integration, project management, execution settings, file reads, and WebSocket updates (`mod.rs::api_routes`). The implemented API contract lives in [`server/openapi.yaml`](../../openapi.yaml).

## Interface

- `api_routes` merges the bridge/WebSocket router and project router (`mod.rs::api_routes`).
- Bridge: `POST /api/v1/bridge/dispatch` uses `dispatch_handler` and `RequireAuth` (`mod.rs::api_routes`, `dispatch.rs::dispatch_handler`, `auth.rs::RequireAuth`).
- Codex: `GET /api/v1/codex/status` and `POST /api/v1/codex/stream` (`mod.rs::api_routes`, `codex.rs::codex_status_handler`, `codex.rs::codex_stream_handler`).
- Projects: list/save, delete, message add/delete/clear, thread update, and project-file read handlers are registered by `api_routes` (`mod.rs::api_routes`, [`projects`](projects.rs)).
- Execution settings: `GET` and `PUT /api/v1/projects/{id}/execution-settings` read and persist project ceilings; the endpoints use strict `RequireRunAuth` and do not start runs ([`execution_settings`](execution_settings.rs), [`auth`](auth.rs)).
- WebSocket: `GET /ws` is handled by `ws_handler` (`mod.rs::api_routes`, `ws.rs::ws_handler`).
- `ApiError` defines API rejection cases (`error.rs::ApiError`).

## Invariants

- Dispatch rejects blank `projectId`, `targetAgentId`, `taskId`, or task title and requires `RequireAuth` (`dispatch.rs::dispatch_handler`, `auth.rs::RequireAuth::from_request_parts`).
- `RequireAuth` accepts only an `Authorization` value starting with `Bearer ` and compares its token to `BRIDGE_AUTH_TOKEN` or the configured fallback (`auth.rs::RequireAuth::from_request_parts`).
- `RequireRunAuth` fails closed when `BRIDGE_AUTH_TOKEN` is absent or blank; execution-settings errors use a stable envelope with `code`, `message`, and `requestId` (`auth.rs::RequireRunAuth`, [`run_error`](run_error.rs)).
- Saved project budgets must be positive and within SQLite's integer range. No run-start endpoint is exposed by this API slice.

## Side Effects

- Dispatch takes a write lock on shared world state, applies a task dispatch, and returns HTTP 202 on success (`dispatch.rs::dispatch_handler`).
- Codex status probes the discovered CLI with `--version`; the stream handler starts a Codex process and streams its output (`codex.rs::codex_status_handler`, `codex.rs::codex_stream_handler`).
- Project handlers read project files and delegate project/message persistence to `Database` (`projects.rs::read_project_file_handler`, `projects.rs::list_projects_handler`, `projects.rs::save_project_handler`).
- Execution settings are persisted per project in SQLite; the GET response explicitly marks an unset configuration as `configured: false`.
- WebSocket clients receive an initial snapshot and subsequent serialized broadcast events (`ws.rs::handle_socket`).

## Verification

- Run from `server/`: `cargo test`

