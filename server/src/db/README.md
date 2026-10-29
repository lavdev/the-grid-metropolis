# Database

## Overview

The database module exposes `Database` for SQLite initialization and project data operations, and re-exports the serialized model types ([`db`](mod.rs), `sqlite.rs::Database`).

## Interface

- `Database::init` opens or initializes a SQLite database; `default_db_path` returns the configured default location (`sqlite.rs::Database::init`, `sqlite.rs::Database::default_db_path`).
- Project operations include `list_projects`, `project_root`, `save_project`, and `delete_project` (`sqlite.rs::Database`).
- Message operations include `add_message`, `update_project_thread_id`, `delete_project_message`, and `clear_project_messages` (`sqlite.rs::Database`).
- Execution budget operations include `get_project_execution_settings` and `save_project_execution_settings`; settings are project-scoped and are not implicitly defaulted (`sqlite.rs::Database`).
- Data structures: `DroidInfo`, `ChatMessage`, `ProjectMetrics`, `PipelineState`, `ProjectTerminal`, and `RosterDroid` ([`models`](models.rs)).

## Invariants

- Database methods return errors through `Result`; callers must handle SQLite errors or the string errors returned by project operations (`sqlite.rs::Database`).
- `save_project` and related database operations use the project ID to locate and update records (`sqlite.rs::Database::save_project`, `sqlite.rs::Database::delete_project`).

## Side Effects

- Initialization can create the parent directory, open a SQLite file, initialize tables, apply ordered `user_version` migrations, and run legacy database migration (`sqlite.rs::Database::init`, `sqlite.rs::Database::apply_migrations`, `sqlite.rs::migrate_legacy_database`).
- Project and message methods execute SQLite queries and mutations (`sqlite.rs::Database`).

## Verification

- Run from `server/`: `cargo test`

