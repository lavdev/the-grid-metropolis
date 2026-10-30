# Database

## Overview

The database module exposes `Database` for SQLite initialization and project data operations, and re-exports the serialized model types ([`db`](mod.rs), `sqlite.rs::Database`).

## Interface

- `Database::init` opens or initializes a SQLite database; `default_db_path` returns the configured default location (`sqlite.rs::Database::init`, `sqlite.rs::Database::default_db_path`).
- Project operations include `list_projects`, `project_root`, `save_project`, and `delete_project` (`sqlite.rs::Database`).
- Message operations include `add_message`, `update_project_thread_id`, `delete_project_message`, and `clear_project_messages` (`sqlite.rs::Database`).
- Execution budget operations include `get_project_execution_settings` and `save_project_execution_settings`; settings are project-scoped and are not implicitly defaulted (`sqlite.rs::Database`).
- Execution policy operations store a project capability ceiling and optional per-Droid overrides. Migration v4 adds `project_execution_policies` and `droid_execution_policy_overrides`; project policy changes are rejected if they would leave a saved Droid profile broader than the new ceiling.
- Durable run storage includes run snapshots, steps, permission requests, and ordered events. Creation, transitions, step updates, permission decisions, and app-server response acknowledgments write their corresponding event in the same SQLite transaction (`sqlite.rs::Database::create_run_record`, `transition_run`, `create_run_step`, `create_pending_permission`, `decide_run_permission`). Migration v3 stores the app-server turn/final-item IDs and the planned next run state on each step for restart reconciliation.
- Data structures: `DroidInfo`, `ChatMessage`, `ProjectMetrics`, `PipelineState`, `ProjectTerminal`, and `RosterDroid` ([`models`](models.rs)).

## Invariants

- Database methods return errors through `Result`; callers must handle SQLite errors or the string errors returned by project operations (`sqlite.rs::Database`).
- `save_project` and related database operations use the project ID to locate and update records (`sqlite.rs::Database::save_project`, `sqlite.rs::Database::delete_project`).
- Run creation requires configured project budgets, a matching or new project-scoped idempotency key, and droids assigned to the project. Foreign keys prevent deleting a project with retained run history.
- Event payloads are capped at 64 KiB, step summaries at 8 KiB, verification evidence and permission request payloads at 32 KiB. Callers are responsible for redacting secrets before persisting event and permission payloads.

## Side Effects

- Initialization can create the parent directory, open a SQLite file, initialize tables, apply ordered `user_version` migrations, and run legacy database migration (`sqlite.rs::Database::init`, `sqlite.rs::Database::apply_migrations`, `sqlite.rs::migrate_legacy_database`).
- Project and message methods execute SQLite queries and mutations (`sqlite.rs::Database`).

## Verification

- Run from `server/`: `cargo test`

