# Scripts

## Overview

This directory contains standalone Python utilities for Notion task synchronization, mock bridge dispatch, browser captures, and one-off edits to frontend source files. The utilities operate on repository-relative paths unless noted below; screenshot and session tools also use hard-coded local profile paths.

## Interface

- **Notion task utilities:** `notion_sync.py` exposes `get_env_var`, `query_tasks`, `update_task_status`, and `create_task`; its CLI supports `search`, `update`, `create`, and `list` ([scripts/notion_sync.py](./notion_sync.py)). `add_roadmap_tasks.py` posts the module-level `tasks_to_create` entries to the Notion API ([scripts/add_roadmap_tasks.py](./add_roadmap_tasks.py)).
- **Bridge CLI:** `mock_dispatch.py` provides `send_dispatch`, `listen_websocket`, `run_e2e_simulation`, and `main`, with `--dispatch`, `--listen`, and `--e2e` modes ([scripts/mock_dispatch.py](./mock_dispatch.py)).
- **Browser capture utilities:** `capture_new_ui.py`, `capture_work_and_handover.py`, and `take_screenshot.py` expose `main` entry points to navigate the local app with Playwright and save screenshots. `capture_refined_fonts.py` performs a synchronous Playwright capture at module scope ([scripts/capture_new_ui.py](./capture_new_ui.py), [scripts/capture_work_and_handover.py](./capture_work_and_handover.py), [scripts/take_screenshot.py](./take_screenshot.py), [scripts/capture_refined_fonts.py](./capture_refined_fonts.py)).
- **Source editing utilities:** `apply_ai_streaming.py` replaces a marked region in `client/src/lib/stores/projectStore.ts`; `reset_project_store.py` replaces `INITIAL_PROJECTS` and adds working-directory arguments; `refine_terminal_fonts.py` and `update_terminal.py` perform text replacements in `client/src/components/terminal/CrtTerminal.svelte` ([scripts/apply_ai_streaming.py:start_marker], [scripts/reset_project_store.py:start_marker], [scripts/refine_terminal_fonts.py:content], [scripts/update_terminal.py:path]).
- **Session inspection:** `inspect_session.py` parses a hard-coded Codex session JSONL file and prints selected record fields ([scripts/inspect_session.py:latest]).
- **Codex app-server Phase 0 probes:** `app_server_phase0_probe.mjs` checks local stdio initialization, thread resume across a process restart, and token-usage notifications. `app_server_phase0_approval_probe.mjs` checks pending-approval reissuance, fingerprint matching, uncertain-grant handling, and persisted completion history. Both require an authenticated Codex CLI and create disposable directories under the OS temp directory ([scripts/app_server_phase0_probe.mjs], [scripts/app_server_phase0_approval_probe.mjs]).

## Invariants

- `notion_sync.py` sources `NOTION_TOKEN` from the process environment or a repository `.env` file, and uses `NOTION_DATABASE_ID` with a built-in default ([scripts/notion_sync.py:get_env_var], [scripts/notion_sync.py:DATABASE_ID]).
- `apply_ai_streaming.py` writes the replacement only when both start and end markers are found; otherwise it exits before writing ([scripts/apply_ai_streaming.py:start_idx], [scripts/apply_ai_streaming.py:end_idx]).
- `reset_project_store.py` writes its replacement only when both boundary markers are found ([scripts/reset_project_store.py:start_idx], [scripts/reset_project_store.py:end_idx]).

## Side Effects

- `notion_sync.py` and `add_roadmap_tasks.py` make authenticated HTTP requests to the Notion API; the former can create or update pages, and the latter creates the listed tasks ([scripts/notion_sync.py:query_tasks], [scripts/notion_sync.py:update_task_status], [scripts/notion_sync.py:create_task], [scripts/add_roadmap_tasks.py:url]).
- `mock_dispatch.py` sends HTTP dispatch requests and can open a WebSocket connection; `run_e2e_simulation` dispatches a task and asserts received events ([scripts/mock_dispatch.py:send_dispatch], [scripts/mock_dispatch.py:listen_websocket], [scripts/mock_dispatch.py:run_e2e_simulation]).
- The Playwright capture utilities connect to a local web app, interact with pages, and write screenshot files to hard-coded output paths ([scripts/capture_new_ui.py:main], [scripts/capture_work_and_handover.py:main], [scripts/capture_refined_fonts.py:sync_playwright], [scripts/take_screenshot.py:main]).
- The source editing utilities read and write frontend files outside this directory: `projectStore.ts` and `CrtTerminal.svelte` ([scripts/apply_ai_streaming.py:open], [scripts/reset_project_store.py:open], [scripts/refine_terminal_fonts.py:open], [scripts/update_terminal.py:open]).
- `inspect_session.py` reads a user-specific JSONL file outside the repository and writes selected session contents to standard output ([scripts/inspect_session.py:open], [scripts/inspect_session.py:print]).
- The app-server probes start local Codex subprocesses and make model calls. The approval probe grants two file/command actions that are constrained to its generated temporary directory; it removes that directory after the run and refuses cleanup if the resolved target is outside the expected temp prefix.

## Verification

This directory has no test or build manifest. The root `package.json` defines no scripts, so no verification command is specified.
