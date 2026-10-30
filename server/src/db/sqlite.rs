use rusqlite::backup::Backup;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{error, info};

use crate::db::models::{ChatMessage, DroidInfo, PipelineState, ProjectMetrics, ProjectTerminal};
use crate::domain::execution_policy::{
    DroidExecutionPolicyOverrides, ExecutionPolicyError, ProjectExecutionPolicy,
};
use crate::domain::run::{
    BudgetConfigError, NewRunPermissionRequest, NewRunRecord, NewRunStep, PermissionStatus,
    ProjectExecutionSettings, RecoveryDecision, RecoveryEvidence, RunBudgetLimits, RunBudgetUsage,
    RunEvent, RunOutcome, RunPermissionRequest, RunPermissionSummary, RunSnapshot, RunStatus,
    RunStep, RunStepStatus, evaluate_recovery,
};

#[derive(Debug, thiserror::Error)]
pub enum ExecutionSettingsStoreError {
    #[error("project not found")]
    ProjectNotFound,
    #[error("invalid project execution budget: {0}")]
    InvalidBudget(#[from] BudgetConfigError),
    #[error("execution settings storage error: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionPolicyStoreError {
    #[error("project not found")]
    ProjectNotFound,
    #[error("droid not found in project")]
    DroidNotFound,
    #[error("project execution policy must be configured first")]
    ProjectPolicyRequired,
    #[error("existing droid policy exceeds the changed project policy")]
    ExistingDroidPolicyConflict,
    #[error("invalid execution policy: {0}")]
    InvalidPolicy(#[from] ExecutionPolicyError),
    #[error("execution policy storage error: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RunStoreError {
    #[error("project not found")]
    ProjectNotFound,
    #[error("run not found")]
    RunNotFound,
    #[error("project execution budgets are required")]
    ExecutionBudgetsRequired,
    #[error("run budget exhausted")]
    BudgetExhausted,
    #[error("run idempotency key conflicts with a different request")]
    IdempotencyConflict,
    #[error("invalid run request: {0}")]
    InvalidRequest(String),
    #[error("invalid run transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
    #[error("run step conflicts with its current state")]
    StepConflict,
    #[error("run is already attached to a different app-server thread")]
    ThreadConflict,
    #[error("permission request not found")]
    PermissionNotFound,
    #[error("permission decision conflicts with the persisted decision")]
    PermissionDecisionConflict,
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if !path_ref.exists() {
            if let Some(legacy_path) = std::env::var_os("GRID_METROPOLIS_LEGACY_DB") {
                let legacy_path = PathBuf::from(legacy_path);
                if legacy_path.is_file() && legacy_path != path_ref {
                    if let Err(error) = migrate_legacy_database(&legacy_path, path_ref) {
                        tracing::warn!(
                            "Não foi possível migrar o banco legado {:?}: {}",
                            legacy_path,
                            error
                        );
                    }
                }
            }
        }

        let mut conn = Connection::open(path_ref)?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL,
                accent_color TEXT NOT NULL,
                active_droid_id TEXT NOT NULL,
                thread_id TEXT,
                metrics_json TEXT NOT NULL,
                pipeline_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_droids (
                project_id TEXT NOT NULL,
                id TEXT NOT NULL,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                model TEXT NOT NULL,
                codex_model TEXT,
                reasoning_effort TEXT,
                plan_badge TEXT NOT NULL,
                avatar_type TEXT NOT NULL,
                status TEXT NOT NULL,
                status_detail TEXT NOT NULL,
                tokens INTEGER NOT NULL,
                accent_color TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (project_id, id),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS project_messages (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                sender TEXT NOT NULL,
                agent_name TEXT,
                agent_role TEXT,
                agent_model TEXT,
                content TEXT NOT NULL,
                thinking TEXT,
                timestamp TEXT NOT NULL,
                status TEXT,
                files_modified_json TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS roster_droids (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                model TEXT NOT NULL,
                codex_model TEXT,
                reasoning_effort TEXT,
                plan_badge TEXT NOT NULL,
                avatar_type TEXT NOT NULL,
                accent_color TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                skills_json TEXT NOT NULL,
                max_tokens INTEGER NOT NULL,
                temperature REAL NOT NULL,
                is_custom INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL
            );
        ",
        )?;

        Self::apply_migrations(&mut conn)?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        // Seed dados padrão se banco estiver vazio
        db.seed_defaults_if_empty()?;

        info!(
            "💾 Banco de dados SQLite persistente inicializado em {:?}",
            path_ref
        );
        Ok(db)
    }

    pub fn default_db_path() -> PathBuf {
        if let Some(data_directory) = std::env::var_os("GRID_METROPOLIS_DATA_DIR") {
            return PathBuf::from(data_directory).join("grid.db");
        }

        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        base.join("data").join("grid.db")
    }

    fn apply_migrations(conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let current_version: i64 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current_version < 1 {
            let tx = conn.transaction()?;
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS project_execution_settings (
                    project_id TEXT PRIMARY KEY,
                    max_model_turns_per_run INTEGER NOT NULL CHECK (max_model_turns_per_run > 0),
                    max_elapsed_seconds_per_run INTEGER NOT NULL CHECK (max_elapsed_seconds_per_run > 0),
                    max_tokens_per_run INTEGER NOT NULL CHECK (max_tokens_per_run > 0),
                    updated_at INTEGER NOT NULL,
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
                );",
            )?;
            tx.pragma_update(None, "user_version", 1_i64)?;
            tx.commit()?;
        }
        let current_version: i64 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current_version < 2 {
            let tx = conn.transaction()?;
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS runs (
                    run_id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL,
                    droid_ids_json TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    task_title TEXT NOT NULL,
                    prompt TEXT NOT NULL,
                    verification_preset_id TEXT NOT NULL,
                    policy_snapshot_json TEXT NOT NULL,
                    status TEXT NOT NULL CHECK (status IN (
                        'QUEUED', 'PLANNING', 'EXECUTING', 'VERIFYING', 'REPAIRING',
                        'REVIEWING', 'WAITING_FOR_PERMISSION', 'SUCCEEDED', 'FAILED',
                        'BLOCKED', 'CANCELLED'
                    )),
                    terminal_outcome TEXT CHECK (terminal_outcome IS NULL OR terminal_outcome IN (
                        'SUCCEEDED', 'FAILED', 'BLOCKED', 'BUDGET_EXHAUSTED', 'CANCELLED'
                    )),
                    max_model_turns_per_run INTEGER NOT NULL CHECK (max_model_turns_per_run > 0),
                    max_elapsed_seconds_per_run INTEGER NOT NULL CHECK (max_elapsed_seconds_per_run > 0),
                    max_tokens_per_run INTEGER NOT NULL CHECK (max_tokens_per_run > 0),
                    model_turns_started INTEGER NOT NULL DEFAULT 0 CHECK (model_turns_started >= 0),
                    elapsed_seconds INTEGER NOT NULL DEFAULT 0 CHECK (elapsed_seconds >= 0),
                    reported_tokens INTEGER NOT NULL DEFAULT 0 CHECK (reported_tokens >= 0),
                    thread_id TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    completed_at INTEGER,
                    UNIQUE(project_id, idempotency_key),
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT
                );

                CREATE INDEX IF NOT EXISTS idx_runs_project_created
                    ON runs(project_id, created_at DESC);
                CREATE INDEX IF NOT EXISTS idx_runs_nonterminal
                    ON runs(status) WHERE terminal_outcome IS NULL;

                CREATE TABLE IF NOT EXISTS run_steps (
                    step_id TEXT PRIMARY KEY,
                    run_id TEXT NOT NULL,
                    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
                    stage TEXT NOT NULL,
                    droid_id TEXT NOT NULL,
                    attempt INTEGER NOT NULL CHECK (attempt > 0),
                    status TEXT NOT NULL,
                    started_at INTEGER,
                    ended_at INTEGER,
                    result_summary TEXT,
                    verification_json TEXT,
                    UNIQUE(run_id, ordinal, attempt),
                    FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS run_permission_requests (
                    request_id TEXT PRIMARY KEY,
                    run_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    step_id TEXT,
                    droid_id TEXT NOT NULL,
                    capability TEXT NOT NULL,
                    action TEXT NOT NULL,
                    resource TEXT NOT NULL,
                    rationale TEXT NOT NULL,
                    status TEXT NOT NULL CHECK (status IN ('PENDING', 'GRANTED', 'DENIED')),
                    alternative_index INTEGER NOT NULL DEFAULT 0 CHECK (alternative_index >= 0),
                    created_at INTEGER NOT NULL,
                    decided_at INTEGER,
                    grant_scope TEXT,
                    resume_status TEXT NOT NULL,
                    app_server_generation TEXT,
                    thread_id TEXT,
                    turn_id TEXT,
                    item_id TEXT,
                    json_rpc_request_id TEXT,
                    app_server_approval_id TEXT,
                    payload_json TEXT NOT NULL,
                    payload_fingerprint TEXT NOT NULL,
                    response_sent INTEGER NOT NULL DEFAULT 0 CHECK (response_sent IN (0, 1)),
                    decision_idempotency_key TEXT,
                    FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE,
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT,
                    FOREIGN KEY (step_id) REFERENCES run_steps(step_id) ON DELETE SET NULL
                );

                CREATE INDEX IF NOT EXISTS idx_run_permissions_pending
                    ON run_permission_requests(run_id, status);

                CREATE TABLE IF NOT EXISTS run_events (
                    run_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK (sequence > 0),
                    event_type TEXT NOT NULL,
                    timestamp INTEGER NOT NULL,
                    project_id TEXT NOT NULL,
                    step_id TEXT,
                    droid_id TEXT,
                    attempt INTEGER,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (run_id, sequence),
                    FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE,
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT,
                    FOREIGN KEY (step_id) REFERENCES run_steps(step_id) ON DELETE SET NULL
                );

                CREATE INDEX IF NOT EXISTS idx_run_events_project
                    ON run_events(project_id, timestamp);",
            )?;
            tx.pragma_update(None, "user_version", 2_i64)?;
            tx.commit()?;
        }
        let current_version: i64 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current_version < 3 {
            let tx = conn.transaction()?;
            tx.execute_batch(
                "ALTER TABLE run_steps ADD COLUMN app_server_turn_id TEXT;
                 ALTER TABLE run_steps ADD COLUMN app_server_item_id TEXT;
                 ALTER TABLE run_steps ADD COLUMN recovery_next_status TEXT;
                 ALTER TABLE run_steps ADD COLUMN recovery_terminal_outcome TEXT;
                 CREATE UNIQUE INDEX idx_run_step_app_server_turn
                    ON run_steps(run_id, app_server_turn_id)
                    WHERE app_server_turn_id IS NOT NULL;",
            )?;
            tx.pragma_update(None, "user_version", 3_i64)?;
            tx.commit()?;
        }
        let current_version: i64 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current_version < 4 {
            let tx = conn.transaction()?;
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS project_execution_policies (
                    project_id TEXT PRIMARY KEY,
                    policy_json TEXT NOT NULL,
                    updated_at INTEGER NOT NULL,
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
                 );
                 CREATE TABLE IF NOT EXISTS droid_execution_policy_overrides (
                    project_id TEXT NOT NULL,
                    droid_id TEXT NOT NULL,
                    policy_json TEXT NOT NULL,
                    updated_at INTEGER NOT NULL,
                    PRIMARY KEY (project_id, droid_id),
                    FOREIGN KEY (project_id, droid_id)
                        REFERENCES project_droids(project_id, id) ON DELETE CASCADE
                 );",
            )?;
            tx.pragma_update(None, "user_version", 4_i64)?;
            tx.commit()?;
        }
        let current_version: i64 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current_version < 5 {
            let tx = conn.transaction()?;
            tx.execute_batch("ALTER TABLE run_steps ADD COLUMN app_server_thread_id TEXT;")?;
            tx.pragma_update(None, "user_version", 5_i64)?;
            tx.commit()?;
        }
        Ok(())
    }

    fn seed_defaults_if_empty(&self) -> Result<(), rusqlite::Error> {
        let count: i64 = {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT count(*) FROM projects", [], |r| r.get(0))?
        };

        if count == 0 {
            info!(
                "🌱 Banco de dados novo detectado. Semeando projeto inicial 'The Notex' e Squad oficial."
            );
            let notex = ProjectTerminal {
                id: "proj-notex".into(),
                name: "The Notex".into(),
                path: "C:\\Workspace\\notex".into(),
                model: "CODEX CLI (ASSINATURA)".into(),
                status: "ACTIVE".into(),
                accent_color: "#00f0ff".into(),
                metrics: ProjectMetrics {
                    tokens_in: 0,
                    tokens_out: 0,
                    latency_ms: 0,
                    cpu: "||      ".into(),
                    mem: "|       ".into(),
                },
                active_droid_id: "spark-01".into(),
                thread_id: None,
                pipeline: PipelineState {
                    is_active: false,
                    stage: "IDLE".into(),
                    current_droid_id: "spark-01".into(),
                    prompt: "".into(),
                    progress_percent: 0,
                    status_text: "Aguardando instrução do operador para The Notex".into(),
                },
                droids: vec![
                    DroidInfo {
                        id: "spark-01".into(),
                        name: "SPARK-01".into(),
                        role: "ARQUITETO".into(),
                        model: "GPT-6.1 Sol (High)".into(),
                        codex_model: Some("gpt-6.1-sol".into()),
                        reasoning_effort: Some("high".into()),
                        plan_badge: "Lead Architect".into(),
                        avatar_type: "spark".into(),
                        status: "ONLINE".into(),
                        status_detail: "Pronto para planejamento de arquitetura e requisitos".into(),
                        tokens: 0,
                        accent_color: "#00f0ff".into(),
                    },
                    DroidInfo {
                        id: "agent-codex-01".into(),
                        name: "Rusty".into(),
                        role: "PROGRAMADOR".into(),
                        model: "GPT-6 Astra (Med)".into(),
                        codex_model: Some("gpt-6-astra".into()),
                        reasoning_effort: Some("medium".into()),
                        plan_badge: "Systems Coder".into(),
                        avatar_type: "hacker".into(),
                        status: "ONLINE".into(),
                        status_detail: "Pronto para codificação técnica em C:\\Workspace\\notex".into(),
                        tokens: 0,
                        accent_color: "#39ff14".into(),
                    },
                    DroidInfo {
                        id: "agent-sec-05".into(),
                        name: "Vanguard".into(),
                        role: "REVISOR_DE_CODIGO".into(),
                        model: "GPT-6.1 Sol (High)".into(),
                        codex_model: Some("gpt-6.1-sol".into()),
                        reasoning_effort: Some("high".into()),
                        plan_badge: "Code Reviewer & QA".into(),
                        avatar_type: "sentinel".into(),
                        status: "ONLINE".into(),
                        status_detail: "Pronto para revisão de código, AST e testes".into(),
                        tokens: 0,
                        accent_color: "#ff007f".into(),
                    },
                ],
                messages: vec![
                    ChatMessage {
                        id: "msg-notex-sys".into(),
                        project_id: Some("proj-notex".into()),
                        sender: "system".into(),
                        agent_name: None,
                        agent_role: None,
                        agent_model: None,
                        content: "Workspace \"The Notex\" conectado em C:\\Workspace\\notex. Tríade oficial de droids (SPARK-01, Rusty, Vanguard) pronta.".into(),
                        thinking: None,
                        timestamp: "16:58".into(),
                        status: None,
                        files_modified: None,
                    },
                    ChatMessage {
                        id: "msg-notex-spark".into(),
                        project_id: Some("proj-notex".into()),
                        sender: "agent".into(),
                        agent_name: Some("SPARK-01".into()),
                        agent_role: Some("ARQUITETO".into()),
                        agent_model: Some("GPT-6.1 Sol".into()),
                        content: "Tríade a postos para The Notex, Arquiteto! Aguardando sua próxima instrução para iniciarmos a esteira técnica.".into(),
                        thinking: None,
                        timestamp: "16:58".into(),
                        status: Some("DONE".into()),
                        files_modified: None,
                    },
                ],
            };

            self.save_project(&notex).map_err(|e| {
                error!("Falha ao semear projeto padrão: {}", e);
                rusqlite::Error::ExecuteReturnedResults
            })?;
        }

        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectTerminal>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, name, path, model, status, accent_color, active_droid_id, thread_id, metrics_json, pipeline_json FROM projects ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let path: String = row.get(2)?;
                let model: String = row.get(3)?;
                let status: String = row.get(4)?;
                let accent_color: String = row.get(5)?;
                let active_droid_id: String = row.get(6)?;
                let thread_id: Option<String> = row.get(7)?;
                let metrics_json: String = row.get(8)?;
                let pipeline_json: String = row.get(9)?;

                Ok((
                    id,
                    name,
                    path,
                    model,
                    status,
                    accent_color,
                    active_droid_id,
                    thread_id,
                    metrics_json,
                    pipeline_json,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut projects = Vec::new();

        for r in rows {
            let (
                id,
                name,
                path,
                model,
                status,
                accent_color,
                active_droid_id,
                thread_id,
                metrics_json,
                pipeline_json,
            ) = r.map_err(|e| e.to_string())?;

            let metrics: ProjectMetrics =
                serde_json::from_str(&metrics_json).unwrap_or(ProjectMetrics {
                    tokens_in: 0,
                    tokens_out: 0,
                    latency_ms: 0,
                    cpu: "||      ".into(),
                    mem: "|       ".into(),
                });

            let pipeline: PipelineState =
                serde_json::from_str(&pipeline_json).unwrap_or(PipelineState {
                    is_active: false,
                    stage: "IDLE".into(),
                    current_droid_id: active_droid_id.clone(),
                    prompt: "".into(),
                    progress_percent: 0,
                    status_text: "Aguardando instrução".into(),
                });

            // Carrega droids do projeto
            let mut d_stmt = conn
                .prepare("SELECT id, name, role, model, codex_model, reasoning_effort, plan_badge, avatar_type, status, status_detail, tokens, accent_color FROM project_droids WHERE project_id = ? ORDER BY sort_order ASC")
                .map_err(|e| e.to_string())?;

            let droids: Vec<DroidInfo> = d_stmt
                .query_map([&id], |d_row| {
                    Ok(DroidInfo {
                        id: d_row.get(0)?,
                        name: d_row.get(1)?,
                        role: d_row.get(2)?,
                        model: d_row.get(3)?,
                        codex_model: d_row.get(4)?,
                        reasoning_effort: d_row.get(5)?,
                        plan_badge: d_row.get(6)?,
                        avatar_type: d_row.get(7)?,
                        status: d_row.get(8)?,
                        status_detail: d_row.get(9)?,
                        tokens: d_row.get(10)?,
                        accent_color: d_row.get(11)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .filter_map(|x| x.ok())
                .collect();

            // Carrega mensagens do projeto
            let mut m_stmt = conn
                .prepare("SELECT id, sender, agent_name, agent_role, agent_model, content, thinking, timestamp, status, files_modified_json FROM project_messages WHERE project_id = ? ORDER BY created_at ASC")
                .map_err(|e| e.to_string())?;

            let messages: Vec<ChatMessage> = m_stmt
                .query_map([&id], |m_row| {
                    let files_json: Option<String> = m_row.get(9)?;
                    let files_modified = files_json.and_then(|j| serde_json::from_str(&j).ok());

                    Ok(ChatMessage {
                        id: m_row.get(0)?,
                        project_id: Some(id.clone()),
                        sender: m_row.get(1)?,
                        agent_name: m_row.get(2)?,
                        agent_role: m_row.get(3)?,
                        agent_model: m_row.get(4)?,
                        content: m_row.get(5)?,
                        thinking: m_row.get(6)?,
                        timestamp: m_row.get(7)?,
                        status: m_row.get(8)?,
                        files_modified,
                    })
                })
                .map_err(|e| e.to_string())?
                .filter_map(|x| x.ok())
                .collect();

            projects.push(ProjectTerminal {
                id,
                name,
                path,
                model,
                status,
                accent_color,
                metrics,
                droids,
                active_droid_id,
                messages,
                pipeline,
                thread_id,
            });
        }

        Ok(projects)
    }

    pub fn project_root(&self, project_id: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT path FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    pub fn get_project_execution_settings(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectExecutionSettings>, ExecutionSettingsStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))?;
        let project_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                params![project_id],
                |row| row.get(0),
            )
            .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))?;
        if !project_exists {
            return Err(ExecutionSettingsStoreError::ProjectNotFound);
        }

        conn.query_row(
            "SELECT max_model_turns_per_run, max_elapsed_seconds_per_run, max_tokens_per_run
             FROM project_execution_settings WHERE project_id = ?1",
            params![project_id],
            |row| {
                Ok(ProjectExecutionSettings {
                    budgets: RunBudgetLimits {
                        max_model_turns_per_run: row.get::<_, i64>(0)? as u64,
                        max_elapsed_seconds_per_run: row.get::<_, i64>(1)? as u64,
                        max_tokens_per_run: row.get::<_, i64>(2)? as u64,
                    },
                })
            },
        )
        .optional()
        .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))
    }

    pub fn save_project_execution_settings(
        &self,
        project_id: &str,
        budgets: RunBudgetLimits,
    ) -> Result<ProjectExecutionSettings, ExecutionSettingsStoreError> {
        budgets.validate()?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))?;
        let project_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                params![project_id],
                |row| row.get(0),
            )
            .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))?;
        if !project_exists {
            return Err(ExecutionSettingsStoreError::ProjectNotFound);
        }

        let updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        tx.execute(
            "INSERT INTO project_execution_settings (
                project_id, max_model_turns_per_run, max_elapsed_seconds_per_run,
                max_tokens_per_run, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(project_id) DO UPDATE SET
                max_model_turns_per_run = excluded.max_model_turns_per_run,
                max_elapsed_seconds_per_run = excluded.max_elapsed_seconds_per_run,
                max_tokens_per_run = excluded.max_tokens_per_run,
                updated_at = excluded.updated_at",
            params![
                project_id,
                budgets.max_model_turns_per_run as i64,
                budgets.max_elapsed_seconds_per_run as i64,
                budgets.max_tokens_per_run as i64,
                updated_at,
            ],
        )
        .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))?;
        tx.commit()
            .map_err(|error| ExecutionSettingsStoreError::Storage(error.to_string()))?;
        Ok(ProjectExecutionSettings { budgets })
    }

    pub fn get_project_execution_policy(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectExecutionPolicy>, ExecutionPolicyStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        ensure_project_exists(&conn, project_id)?;
        conn.query_row(
            "SELECT policy_json FROM project_execution_policies WHERE project_id = ?1",
            params![project_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?
        .map(|json| {
            serde_json::from_str(&json)
                .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))
        })
        .transpose()
    }

    pub fn save_project_execution_policy(
        &self,
        project_id: &str,
        policy: &ProjectExecutionPolicy,
    ) -> Result<ProjectExecutionPolicy, ExecutionPolicyStoreError> {
        policy.validate()?;
        let policy_json = serde_json::to_string(policy)
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        ensure_project_exists(&tx, project_id)?;
        for reference_id in &policy.reference_project_ids {
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1 AND id != ?2)",
                    params![reference_id, project_id],
                    |row| row.get(0),
                )
                .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
            if !exists {
                return Err(ExecutionPolicyStoreError::InvalidPolicy(
                    ExecutionPolicyError::InvalidReferenceProjects,
                ));
            }
        }

        let mut statement = tx
            .prepare(
                "SELECT policy_json FROM droid_execution_policy_overrides WHERE project_id = ?1",
            )
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        let overrides = statement
            .query_map(params![project_id], |row| row.get::<_, String>(0))
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?
            .map(|row| {
                let json =
                    row.map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
                serde_json::from_str::<DroidExecutionPolicyOverrides>(&json)
                    .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);
        if overrides
            .iter()
            .any(|overrides| policy.effective_for(overrides).is_err())
        {
            return Err(ExecutionPolicyStoreError::ExistingDroidPolicyConflict);
        }

        let updated_at = unix_time_millis();
        tx.execute(
            "INSERT INTO project_execution_policies (project_id, policy_json, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(project_id) DO UPDATE SET
                policy_json = excluded.policy_json,
                updated_at = excluded.updated_at",
            params![project_id, policy_json, updated_at],
        )
        .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        tx.commit()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        Ok(policy.clone())
    }

    pub fn get_droid_execution_policy_overrides(
        &self,
        project_id: &str,
        droid_id: &str,
    ) -> Result<Option<DroidExecutionPolicyOverrides>, ExecutionPolicyStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        ensure_project_exists(&conn, project_id)?;
        ensure_project_droid_exists(&conn, project_id, droid_id)?;
        conn.query_row(
            "SELECT policy_json FROM droid_execution_policy_overrides
             WHERE project_id = ?1 AND droid_id = ?2",
            params![project_id, droid_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?
        .map(|json| {
            serde_json::from_str(&json)
                .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))
        })
        .transpose()
    }

    pub fn save_droid_execution_policy_overrides(
        &self,
        project_id: &str,
        droid_id: &str,
        overrides: &DroidExecutionPolicyOverrides,
    ) -> Result<DroidExecutionPolicyOverrides, ExecutionPolicyStoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        ensure_project_exists(&tx, project_id)?;
        ensure_project_droid_exists(&tx, project_id, droid_id)?;
        let project_policy_json = tx
            .query_row(
                "SELECT policy_json FROM project_execution_policies WHERE project_id = ?1",
                params![project_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?
            .ok_or(ExecutionPolicyStoreError::ProjectPolicyRequired)?;
        let project_policy: ProjectExecutionPolicy = serde_json::from_str(&project_policy_json)
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        project_policy.effective_for(overrides)?;

        let policy_json = serde_json::to_string(overrides)
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        tx.execute(
            "INSERT INTO droid_execution_policy_overrides
                (project_id, droid_id, policy_json, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(project_id, droid_id) DO UPDATE SET
                policy_json = excluded.policy_json,
                updated_at = excluded.updated_at",
            params![project_id, droid_id, policy_json, unix_time_millis()],
        )
        .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        tx.commit()
            .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
        Ok(overrides.clone())
    }

    pub fn create_run_record(&self, run: &NewRunRecord) -> Result<RunSnapshot, RunStoreError> {
        validate_new_run(run)?;
        let droid_ids_json = serde_json::to_string(&run.droid_ids)
            .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;
        let conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let mut conn = conn;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;

        let project_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                params![run.project_id],
                |row| row.get(0),
            )
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        if !project_exists {
            return Err(RunStoreError::ProjectNotFound);
        }

        let project_budgets = tx
            .query_row(
                "SELECT max_model_turns_per_run, max_elapsed_seconds_per_run, max_tokens_per_run
                 FROM project_execution_settings WHERE project_id = ?1",
                params![run.project_id],
                |row| {
                    Ok(RunBudgetLimits {
                        max_model_turns_per_run: row.get::<_, i64>(0)? as u64,
                        max_elapsed_seconds_per_run: row.get::<_, i64>(1)? as u64,
                        max_tokens_per_run: row.get::<_, i64>(2)? as u64,
                    })
                },
            )
            .optional()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?
            .ok_or(RunStoreError::ExecutionBudgetsRequired)?;
        project_budgets
            .tighten_with(Some(run.budget_limits))
            .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;

        for droid_id in &run.droid_ids {
            let assigned: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM project_droids WHERE project_id = ?1 AND id = ?2)",
                    params![run.project_id, droid_id],
                    |row| row.get(0),
                )
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            if !assigned {
                return Err(RunStoreError::InvalidRequest(format!(
                    "droid {droid_id} is not assigned to the project"
                )));
            }
        }

        if let Some(existing_id) = tx
            .query_row(
                "SELECT run_id FROM runs WHERE project_id = ?1 AND idempotency_key = ?2",
                params![run.project_id, run.idempotency_key],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?
        {
            let snapshot = read_run_snapshot(&tx, &existing_id)?
                .ok_or_else(|| RunStoreError::Storage("idempotent run disappeared".into()))?;
            let existing_request_matches: bool = tx
                .query_row(
                    "SELECT droid_ids_json = ?2 AND task_title = ?3 AND prompt = ?4
                            AND verification_preset_id = ?5 AND policy_snapshot_json = ?6
                            AND max_model_turns_per_run = ?7 AND max_elapsed_seconds_per_run = ?8
                            AND max_tokens_per_run = ?9
                     FROM runs WHERE run_id = ?1",
                    params![
                        existing_id,
                        droid_ids_json,
                        run.task_title,
                        run.prompt,
                        run.verification_preset_id,
                        run.policy_snapshot_json,
                        run.budget_limits.max_model_turns_per_run as i64,
                        run.budget_limits.max_elapsed_seconds_per_run as i64,
                        run.budget_limits.max_tokens_per_run as i64,
                    ],
                    |row| row.get(0),
                )
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            if !existing_request_matches {
                return Err(RunStoreError::IdempotencyConflict);
            }
            tx.commit()
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            return Ok(snapshot);
        }

        let now = unix_time_millis();
        tx.execute(
            "INSERT INTO runs (
                run_id, project_id, droid_ids_json, idempotency_key, task_title, prompt,
                verification_preset_id, policy_snapshot_json, status, terminal_outcome,
                max_model_turns_per_run, max_elapsed_seconds_per_run, max_tokens_per_run,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'QUEUED', NULL, ?9, ?10, ?11, ?12, ?12)",
            params![
                run.run_id,
                run.project_id,
                droid_ids_json,
                run.idempotency_key,
                run.task_title,
                run.prompt,
                run.verification_preset_id,
                run.policy_snapshot_json,
                run.budget_limits.max_model_turns_per_run as i64,
                run.budget_limits.max_elapsed_seconds_per_run as i64,
                run.budget_limits.max_tokens_per_run as i64,
                now,
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            &run.run_id,
            &run.project_id,
            "RUN_QUEUED",
            None,
            None,
            None,
            &serde_json::json!({ "status": "QUEUED" }),
            now,
        )?;
        let snapshot = read_run_snapshot(&tx, &run.run_id)?
            .ok_or_else(|| RunStoreError::Storage("created run could not be read".into()))?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(snapshot)
    }

    pub fn get_run_snapshot(&self, run_id: &str) -> Result<RunSnapshot, RunStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        read_run_snapshot(&conn, run_id)?.ok_or(RunStoreError::RunNotFound)
    }

    pub fn reserve_run_model_turn(&self, run_id: &str) -> Result<RunSnapshot, RunStoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let current = read_run_snapshot(&tx, run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if current.status != RunStatus::Executing
            || current.usage.model_turns_started >= current.budget_limits.max_model_turns_per_run
            || current.usage.elapsed_seconds >= current.budget_limits.max_elapsed_seconds_per_run
            || current.usage.reported_tokens >= current.budget_limits.max_tokens_per_run
        {
            return Err(RunStoreError::BudgetExhausted);
        }
        let now = unix_time_millis();
        tx.execute(
            "UPDATE runs SET model_turns_started = model_turns_started + 1, updated_at = ?2
             WHERE run_id = ?1",
            params![run_id, now],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            run_id,
            &current.project_id,
            "MODEL_TURN_RESERVED",
            None,
            None,
            None,
            &serde_json::json!({ "modelTurnsStarted": current.usage.model_turns_started + 1 }),
            now,
        )?;
        let updated = read_run_snapshot(&tx, run_id)?.ok_or(RunStoreError::RunNotFound)?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(updated)
    }

    pub fn record_run_usage(
        &self,
        run_id: &str,
        usage: RunBudgetUsage,
    ) -> Result<RunSnapshot, RunStoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let current = read_run_snapshot(&tx, run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if usage.model_turns_started > i64::MAX as u64
            || usage.elapsed_seconds > i64::MAX as u64
            || usage.reported_tokens > i64::MAX as u64
        {
            return Err(RunStoreError::InvalidRequest(
                "run usage counter exceeds the supported storage range".into(),
            ));
        }
        if usage.model_turns_started < current.usage.model_turns_started
            || usage.elapsed_seconds < current.usage.elapsed_seconds
            || usage.reported_tokens < current.usage.reported_tokens
        {
            return Err(RunStoreError::InvalidRequest(
                "run usage counters cannot move backwards".into(),
            ));
        }
        let now = unix_time_millis();
        tx.execute(
            "UPDATE runs SET model_turns_started = ?2, elapsed_seconds = ?3,
                    reported_tokens = ?4, updated_at = ?5 WHERE run_id = ?1",
            params![
                run_id,
                usage.model_turns_started as i64,
                usage.elapsed_seconds as i64,
                usage.reported_tokens as i64,
                now
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            run_id,
            &current.project_id,
            "RUN_USAGE_UPDATED",
            None,
            None,
            None,
            &serde_json::json!({
                "modelTurnsStarted": usage.model_turns_started,
                "elapsedSeconds": usage.elapsed_seconds,
                "reportedTokens": usage.reported_tokens
            }),
            now,
        )?;
        let updated = read_run_snapshot(&tx, run_id)?.ok_or(RunStoreError::RunNotFound)?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(updated)
    }

    pub fn get_run_execution_input(
        &self,
        run_id: &str,
    ) -> Result<crate::domain::run::RunExecutionInput, RunStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        conn.query_row(
            "SELECT task_title, prompt, verification_preset_id, policy_snapshot_json
             FROM runs WHERE run_id = ?1",
            [run_id],
            |row| {
                Ok(crate::domain::run::RunExecutionInput {
                    task_title: row.get(0)?,
                    prompt: row.get(1)?,
                    verification_preset_id: row.get(2)?,
                    policy_snapshot_json: row.get(3)?,
                })
            },
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => RunStoreError::RunNotFound,
            _ => RunStoreError::Storage(error.to_string()),
        })
    }

    pub fn list_nonterminal_runs(&self) -> Result<Vec<RunSnapshot>, RunStoreError> {
        let run_ids = {
            let conn = self
                .conn
                .lock()
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            let mut statement = conn
                .prepare(
                    "SELECT run_id FROM runs WHERE terminal_outcome IS NULL
                     ORDER BY created_at, run_id",
                )
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| RunStoreError::Storage(error.to_string()))?
        };
        run_ids
            .iter()
            .map(|run_id| self.get_run_snapshot(run_id))
            .collect()
    }

    pub fn create_run_step(&self, step: &NewRunStep) -> Result<RunStep, RunStoreError> {
        if step.step_id.trim().is_empty()
            || step.stage.trim().is_empty()
            || step.droid_id.trim().is_empty()
            || step.attempt == 0
        {
            return Err(RunStoreError::InvalidRequest(
                "step ID, stage, droid and positive attempt are required".into(),
            ));
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let run = read_run_snapshot(&tx, &step.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if run.status.is_terminal() || !run.droid_ids.contains(&step.droid_id) {
            return Err(RunStoreError::InvalidRequest(
                "step droid must belong to a nonterminal run".into(),
            ));
        }
        let now = unix_time_millis();
        tx.execute(
            "INSERT INTO run_steps (step_id, run_id, ordinal, stage, droid_id, attempt, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'PENDING')",
            params![
                step.step_id,
                step.run_id,
                step.ordinal,
                step.stage,
                step.droid_id,
                step.attempt
            ],
        )
        .map_err(|error| {
            if matches!(error, rusqlite::Error::SqliteFailure(_, _)) {
                RunStoreError::StepConflict
            } else {
                RunStoreError::Storage(error.to_string())
            }
        })?;
        append_run_event_tx(
            &tx,
            &step.run_id,
            &run.project_id,
            "RUN_STEP_CREATED",
            Some(&step.step_id),
            Some(&step.droid_id),
            Some(step.attempt),
            &serde_json::json!({ "ordinal": step.ordinal, "stage": step.stage }),
            now,
        )?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(RunStep {
            step_id: step.step_id.clone(),
            run_id: step.run_id.clone(),
            ordinal: step.ordinal,
            stage: step.stage.clone(),
            droid_id: step.droid_id.clone(),
            attempt: step.attempt,
            status: RunStepStatus::Pending,
            started_at: None,
            ended_at: None,
            result_summary: None,
            verification: None,
            app_server_thread_id: None,
            app_server_turn_id: None,
            app_server_item_id: None,
            recovery_next_status: None,
            recovery_terminal_outcome: None,
        })
    }

    pub fn start_run_step(&self, step_id: &str) -> Result<RunStep, RunStoreError> {
        self.update_run_step(step_id, RunStepStatus::InProgress, None, None)
    }

    pub fn persist_run_step_thread_id(
        &self,
        step_id: &str,
        thread_id: &str,
    ) -> Result<RunStep, RunStoreError> {
        if thread_id.trim().is_empty() {
            return Err(RunStoreError::InvalidRequest(
                "app-server thread ID is required".into(),
            ));
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let step = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        if step.status != RunStepStatus::InProgress {
            return Err(RunStoreError::StepConflict);
        }
        if let Some(stored_thread_id) = step.app_server_thread_id.as_deref() {
            if stored_thread_id == thread_id {
                return Ok(step);
            }
            return Err(RunStoreError::ThreadConflict);
        }
        let run = read_run_snapshot(&tx, &step.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        let now = unix_time_millis();
        tx.execute(
            "UPDATE run_steps SET app_server_thread_id = ?2 WHERE step_id = ?1",
            params![step_id, thread_id],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            &step.run_id,
            &run.project_id,
            "RUN_STEP_APP_SERVER_THREAD_ATTACHED",
            Some(step_id),
            Some(&step.droid_id),
            Some(step.attempt),
            &serde_json::json!({ "threadId": thread_id }),
            now,
        )?;
        let updated = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(updated)
    }

    pub fn persist_run_step_turn(
        &self,
        step_id: &str,
        turn_id: &str,
        next_status: RunStatus,
        terminal_outcome: Option<RunOutcome>,
    ) -> Result<RunStep, RunStoreError> {
        if turn_id.trim().is_empty() {
            return Err(RunStoreError::InvalidRequest(
                "app-server turn ID is required".into(),
            ));
        }
        validate_terminal_outcome(next_status, terminal_outcome)?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let step = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        let run = read_run_snapshot(&tx, &step.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if step.status != RunStepStatus::InProgress || !run.status.can_transition_to(next_status) {
            return Err(RunStoreError::StepConflict);
        }
        if let Some(stored_turn_id) = step.app_server_turn_id.as_deref() {
            if stored_turn_id == turn_id
                && step.recovery_next_status == Some(next_status)
                && step.recovery_terminal_outcome == terminal_outcome
            {
                return Ok(step);
            }
            return Err(RunStoreError::StepConflict);
        }
        let now = unix_time_millis();
        tx.execute(
            "UPDATE run_steps SET app_server_turn_id = ?2, recovery_next_status = ?3,
                recovery_terminal_outcome = ?4 WHERE step_id = ?1",
            params![
                step_id,
                turn_id,
                next_status.as_str(),
                terminal_outcome.map(RunOutcome::as_str)
            ],
        )
        .map_err(|error| {
            if matches!(error, rusqlite::Error::SqliteFailure(_, _)) {
                RunStoreError::StepConflict
            } else {
                RunStoreError::Storage(error.to_string())
            }
        })?;
        append_run_event_tx(
            &tx,
            &step.run_id,
            &run.project_id,
            "RUN_STEP_APP_SERVER_TURN_ATTACHED",
            Some(step_id),
            Some(&step.droid_id),
            Some(step.attempt),
            &serde_json::json!({
                "turnId": turn_id,
                "recoveryNextStatus": next_status.as_str(),
                "recoveryTerminalOutcome": terminal_outcome.map(RunOutcome::as_str)
            }),
            now,
        )?;
        let updated = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(updated)
    }

    pub fn complete_app_server_run_step(
        &self,
        step_id: &str,
        turn_id: &str,
        item_id: &str,
        summary: &str,
    ) -> Result<RunStep, RunStoreError> {
        if turn_id.trim().is_empty() || item_id.trim().is_empty() {
            return Err(RunStoreError::InvalidRequest(
                "app-server turn and item IDs are required".into(),
            ));
        }
        if summary.len() > 8 * 1024 {
            return Err(RunStoreError::InvalidRequest(
                "step result summary exceeds 8192 bytes".into(),
            ));
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let step = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        let run = read_run_snapshot(&tx, &step.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if step.app_server_turn_id.as_deref() != Some(turn_id)
            || step.recovery_next_status.is_none()
            || step.status == RunStepStatus::Failed
            || step.status == RunStepStatus::Skipped
            || !step
                .recovery_next_status
                .is_some_and(|next_status| run.status.can_transition_to(next_status))
        {
            return Err(RunStoreError::StepConflict);
        }
        if step.status == RunStepStatus::Succeeded {
            if step.app_server_item_id.as_deref() == Some(item_id) {
                return Ok(step);
            }
            return Err(RunStoreError::StepConflict);
        }
        if step.status != RunStepStatus::InProgress {
            return Err(RunStoreError::StepConflict);
        }
        let now = unix_time_millis();
        tx.execute(
            "UPDATE run_steps SET status = 'SUCCEEDED', app_server_item_id = ?2,
                ended_at = ?3, result_summary = ?4 WHERE step_id = ?1",
            params![step_id, item_id, now, summary],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            &step.run_id,
            &run.project_id,
            "RUN_STEP_COMPLETED",
            Some(step_id),
            Some(&step.droid_id),
            Some(step.attempt),
            &serde_json::json!({
                "status": RunStepStatus::Succeeded.as_str(),
                "turnId": turn_id,
                "itemId": item_id,
                "summary": summary
            }),
            now,
        )?;
        let updated = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(updated)
    }

    pub fn complete_run_step(
        &self,
        step_id: &str,
        status: RunStepStatus,
        summary: &str,
        verification_json: Option<&str>,
    ) -> Result<RunStep, RunStoreError> {
        if !matches!(
            status,
            RunStepStatus::Succeeded | RunStepStatus::Failed | RunStepStatus::Skipped
        ) {
            return Err(RunStoreError::InvalidRequest(
                "completed step status must be SUCCEEDED, FAILED or SKIPPED".into(),
            ));
        }
        if summary.len() > 8 * 1024 {
            return Err(RunStoreError::InvalidRequest(
                "step result summary exceeds 8192 bytes".into(),
            ));
        }
        let verification = verification_json
            .map(|json| {
                let value: serde_json::Value = serde_json::from_str(json)
                    .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;
                if json.len() > 32 * 1024 {
                    return Err(RunStoreError::InvalidRequest(
                        "verification evidence exceeds 32768 bytes".into(),
                    ));
                }
                Ok(value)
            })
            .transpose()?;
        self.update_run_step(
            step_id,
            status,
            Some(summary),
            verification
                .as_ref()
                .map(serde_json::Value::to_string)
                .as_deref(),
        )
    }

    fn update_run_step(
        &self,
        step_id: &str,
        next_status: RunStepStatus,
        summary: Option<&str>,
        verification_json: Option<&str>,
    ) -> Result<RunStep, RunStoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let step = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        let valid = match (step.status, next_status) {
            (RunStepStatus::Pending, RunStepStatus::InProgress)
            | (RunStepStatus::InProgress, RunStepStatus::Succeeded)
            | (RunStepStatus::InProgress, RunStepStatus::Failed)
            | (RunStepStatus::InProgress, RunStepStatus::Skipped) => true,
            _ => false,
        };
        if !valid {
            return Err(RunStoreError::StepConflict);
        }
        let now = unix_time_millis();
        tx.execute(
            "UPDATE run_steps SET status = ?2,
                started_at = CASE WHEN ?2 = 'IN_PROGRESS' THEN ?3 ELSE started_at END,
                ended_at = CASE WHEN ?2 IN ('SUCCEEDED','FAILED','SKIPPED') THEN ?3 ELSE ended_at END,
                result_summary = COALESCE(?4, result_summary),
                verification_json = COALESCE(?5, verification_json)
             WHERE step_id = ?1",
            params![step_id, next_status.as_str(), now, summary, verification_json],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let event_type = match next_status {
            RunStepStatus::InProgress => "RUN_STEP_STARTED",
            RunStepStatus::Succeeded | RunStepStatus::Failed | RunStepStatus::Skipped => {
                "RUN_STEP_COMPLETED"
            }
            RunStepStatus::Pending => "RUN_STEP_CREATED",
        };
        append_run_event_tx(
            &tx,
            &step.run_id,
            &read_run_snapshot(&tx, &step.run_id)?
                .ok_or(RunStoreError::RunNotFound)?
                .project_id,
            event_type,
            Some(&step.step_id),
            Some(&step.droid_id),
            Some(step.attempt),
            &serde_json::json!({ "status": next_status.as_str(), "summary": summary }),
            now,
        )?;
        let updated = read_run_step(&tx, step_id)?.ok_or(RunStoreError::StepConflict)?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(updated)
    }

    pub fn create_pending_permission(
        &self,
        request: &NewRunPermissionRequest,
    ) -> Result<RunPermissionRequest, RunStoreError> {
        if request.request_id.trim().is_empty()
            || request.droid_id.trim().is_empty()
            || request.capability.trim().is_empty()
            || request.action.trim().is_empty()
            || request.resource.trim().is_empty()
            || request.payload_fingerprint.trim().is_empty()
        {
            return Err(RunStoreError::InvalidRequest(
                "permission ID, droid, capability, action, resource and fingerprint are required"
                    .into(),
            ));
        }
        let payload: serde_json::Value = serde_json::from_str(&request.payload_json)
            .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;
        if !payload.is_object() || request.payload_json.len() > 32 * 1024 {
            return Err(RunStoreError::InvalidRequest(
                "permission payload must be an object no larger than 32768 bytes".into(),
            ));
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let run = read_run_snapshot(&tx, &request.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if !run
            .status
            .can_transition_to(RunStatus::WaitingForPermission)
            || !run.droid_ids.contains(&request.droid_id)
        {
            return Err(RunStoreError::InvalidRequest(
                "run cannot enter permission wait for this droid".into(),
            ));
        }
        if let Some(step_id) = request.step_id.as_deref() {
            let belongs_to_run: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM run_steps WHERE step_id = ?1 AND run_id = ?2)",
                    params![step_id, request.run_id],
                    |row| row.get(0),
                )
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            if !belongs_to_run {
                return Err(RunStoreError::InvalidRequest(
                    "permission step does not belong to the run".into(),
                ));
            }
        }
        let now = unix_time_millis();
        tx.execute(
            "INSERT INTO run_permission_requests (
                request_id, run_id, project_id, step_id, droid_id, capability, action,
                resource, rationale, status, alternative_index, created_at, resume_status,
                app_server_generation, thread_id, turn_id, item_id, json_rpc_request_id,
                app_server_approval_id, payload_json, payload_fingerprint, response_sent
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'PENDING', ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, 0)",
            params![
                request.request_id,
                request.run_id,
                run.project_id,
                request.step_id,
                request.droid_id,
                request.capability,
                request.action,
                request.resource,
                request.rationale,
                request.alternative_index,
                now,
                run.status.as_str(),
                request.app_server_generation,
                request.thread_id,
                request.turn_id,
                request.item_id,
                request.json_rpc_request_id,
                request.app_server_approval_id,
                request.payload_json,
                request.payload_fingerprint,
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        tx.execute(
            "UPDATE runs SET status = 'WAITING_FOR_PERMISSION', updated_at = ?2 WHERE run_id = ?1",
            params![request.run_id, now],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            &request.run_id,
            &run.project_id,
            "PERMISSION_REQUESTED",
            request.step_id.as_deref(),
            Some(&request.droid_id),
            None,
            &serde_json::json!({
                "requestId": request.request_id,
                "capability": request.capability,
                "action": request.action,
                "resource": request.resource,
                "alternativeIndex": request.alternative_index,
                "payloadFingerprint": request.payload_fingerprint,
            }),
            now,
        )?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(RunPermissionRequest {
            request_id: request.request_id.clone(),
            run_id: request.run_id.clone(),
            project_id: run.project_id,
            step_id: request.step_id.clone(),
            droid_id: request.droid_id.clone(),
            capability: request.capability.clone(),
            action: request.action.clone(),
            resource: request.resource.clone(),
            rationale: request.rationale.clone(),
            status: PermissionStatus::Pending,
            alternative_index: request.alternative_index,
            created_at: now,
            decided_at: None,
            grant_scope: None,
            resume_status: run.status,
            app_server_generation: request.app_server_generation.clone(),
            thread_id: request.thread_id.clone(),
            turn_id: request.turn_id.clone(),
            item_id: request.item_id.clone(),
            json_rpc_request_id: request.json_rpc_request_id.clone(),
            app_server_approval_id: request.app_server_approval_id.clone(),
            payload,
            payload_fingerprint: request.payload_fingerprint.clone(),
            response_sent: false,
            decision_idempotency_key: None,
        })
    }

    pub fn refresh_unsent_permission_request(
        &self,
        request_id: &str,
        thread_id: &str,
        turn_id: &str,
        item_id: Option<&str>,
        json_rpc_request_id: &str,
        app_server_approval_id: Option<&str>,
        resumed_fingerprint: &str,
    ) -> Result<RunPermissionRequest, RunStoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let mut permission =
            read_permission_request(&tx, request_id)?.ok_or(RunStoreError::PermissionNotFound)?;
        if permission.response_sent
            || permission.payload_fingerprint != resumed_fingerprint
            || thread_id.trim().is_empty()
            || turn_id.trim().is_empty()
            || json_rpc_request_id.trim().is_empty()
        {
            return Err(RunStoreError::PermissionDecisionConflict);
        }
        let run = read_run_snapshot(&tx, &permission.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if run.status != permission.resume_status {
            return Err(RunStoreError::PermissionDecisionConflict);
        }
        let now = unix_time_millis();
        tx.execute(
            "UPDATE run_permission_requests SET thread_id = ?2, turn_id = ?3, item_id = ?4,
                    json_rpc_request_id = ?5, app_server_approval_id = ?6
             WHERE request_id = ?1 AND response_sent = 0",
            params![
                request_id,
                thread_id,
                turn_id,
                item_id,
                json_rpc_request_id,
                app_server_approval_id
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        tx.execute(
            "UPDATE runs SET status = 'WAITING_FOR_PERMISSION', updated_at = ?2
             WHERE run_id = ?1",
            params![permission.run_id, now],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            &permission.run_id,
            &permission.project_id,
            "PERMISSION_REISSUED",
            permission.step_id.as_deref(),
            Some(&permission.droid_id),
            None,
            &serde_json::json!({
                "requestId": request_id,
                "threadId": thread_id,
                "turnId": turn_id,
                "itemId": item_id,
                "payloadFingerprint": resumed_fingerprint,
            }),
            now,
        )?;
        permission.thread_id = Some(thread_id.to_owned());
        permission.turn_id = Some(turn_id.to_owned());
        permission.item_id = item_id.map(str::to_owned);
        permission.json_rpc_request_id = Some(json_rpc_request_id.to_owned());
        permission.app_server_approval_id = app_server_approval_id.map(str::to_owned);
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(permission)
    }

    pub fn decide_run_permission(
        &self,
        request_id: &str,
        granted: bool,
        idempotency_key: &str,
        grant_scope: Option<&str>,
    ) -> Result<(RunPermissionRequest, RunSnapshot), RunStoreError> {
        if idempotency_key.trim().is_empty()
            || (granted && grant_scope.unwrap_or_default().trim().is_empty())
        {
            return Err(RunStoreError::InvalidRequest(
                "decision idempotency key and explicit grant scope are required".into(),
            ));
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let mut request =
            read_permission_request(&tx, request_id)?.ok_or(RunStoreError::PermissionNotFound)?;
        let requested_status = if granted {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied
        };
        if request.status != PermissionStatus::Pending {
            if request.status != requested_status {
                return Err(RunStoreError::PermissionDecisionConflict);
            }
            if request.decision_idempotency_key.as_deref() != Some(idempotency_key) {
                return Err(RunStoreError::PermissionDecisionConflict);
            }
            if request.grant_scope.as_deref() != grant_scope {
                return Err(RunStoreError::PermissionDecisionConflict);
            }
            let snapshot =
                read_run_snapshot(&tx, &request.run_id)?.ok_or(RunStoreError::RunNotFound)?;
            tx.commit()
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            return Ok((request, snapshot));
        }
        let run = read_run_snapshot(&tx, &request.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if run.status != RunStatus::WaitingForPermission {
            return Err(RunStoreError::PermissionDecisionConflict);
        }
        let must_block = !granted && request.alternative_index >= 3;
        let outcome = must_block.then_some(RunOutcome::Blocked);
        let now = unix_time_millis();
        tx.execute(
            "UPDATE run_permission_requests SET status = ?2, decided_at = ?3, grant_scope = ?4,
                    decision_idempotency_key = ?5
             WHERE request_id = ?1 AND status = 'PENDING'",
            params![
                request_id,
                requested_status.as_str(),
                now,
                grant_scope,
                idempotency_key
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        tx.execute(
            "UPDATE runs SET status = ?2, terminal_outcome = ?3, updated_at = ?4,
                    completed_at = CASE WHEN ?3 IS NULL THEN NULL ELSE ?4 END
             WHERE run_id = ?1",
            params![
                request.run_id,
                if must_block {
                    RunStatus::Blocked.as_str()
                } else {
                    RunStatus::WaitingForPermission.as_str()
                },
                outcome.map(RunOutcome::as_str),
                now
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            &request.run_id,
            &run.project_id,
            if granted {
                "PERMISSION_GRANTED"
            } else {
                "PERMISSION_DENIED"
            },
            request.step_id.as_deref(),
            Some(&request.droid_id),
            None,
            &serde_json::json!({
                "requestId": request_id,
                "status": requested_status.as_str(),
                "alternativeIndex": request.alternative_index,
                "blocked": must_block,
            }),
            now,
        )?;
        request.status = requested_status;
        request.decided_at = Some(now);
        request.grant_scope = grant_scope.map(str::to_string);
        request.decision_idempotency_key = Some(idempotency_key.to_string());
        let snapshot =
            read_run_snapshot(&tx, &request.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok((request, snapshot))
    }

    pub fn mark_permission_response_sent(
        &self,
        request_id: &str,
    ) -> Result<RunPermissionRequest, RunStoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let mut request =
            read_permission_request(&tx, request_id)?.ok_or(RunStoreError::PermissionNotFound)?;
        if request.status == PermissionStatus::Pending {
            return Err(RunStoreError::PermissionDecisionConflict);
        }
        if request.response_sent {
            tx.commit()
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            return Ok(request);
        }
        let run = read_run_snapshot(&tx, &request.run_id)?.ok_or(RunStoreError::RunNotFound)?;
        let must_block =
            request.status == PermissionStatus::Denied && request.alternative_index >= 3;
        let next_status = if must_block {
            RunStatus::Blocked
        } else if run.status == RunStatus::WaitingForPermission {
            request.resume_status
        } else {
            run.status
        };
        let terminal_outcome = (next_status == RunStatus::Blocked).then_some(RunOutcome::Blocked);
        let now = unix_time_millis();
        tx.execute(
            "UPDATE run_permission_requests SET response_sent = 1 WHERE request_id = ?1",
            params![request_id],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        tx.execute(
            "UPDATE runs SET status = ?2, terminal_outcome = ?3,
                    completed_at = CASE WHEN ?3 IS NULL THEN NULL ELSE COALESCE(completed_at, ?4) END,
                    updated_at = ?4
             WHERE run_id = ?1",
            params![
                request.run_id,
                next_status.as_str(),
                terminal_outcome.map(RunOutcome::as_str),
                now
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            &request.run_id,
            &run.project_id,
            "PERMISSION_RESPONSE_SENT",
            request.step_id.as_deref(),
            Some(&request.droid_id),
            None,
            &serde_json::json!({
                "requestId": request_id,
                "status": request.status.as_str(),
                "nextStatus": next_status.as_str(),
                "blocked": next_status == RunStatus::Blocked,
            }),
            now,
        )?;
        request.response_sent = true;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(request)
    }

    pub fn get_permission_request(
        &self,
        request_id: &str,
    ) -> Result<Option<RunPermissionRequest>, RunStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        read_permission_request(&conn, request_id)
    }

    pub fn list_unresolved_permission_requests(
        &self,
        run_id: &str,
    ) -> Result<Vec<RunPermissionRequest>, RunStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let request_ids = {
            let mut statement = conn
                .prepare(
                    "SELECT request_id FROM run_permission_requests
                     WHERE run_id = ?1 AND response_sent = 0 ORDER BY created_at, request_id",
                )
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            let rows = statement
                .query_map(params![run_id], |row| row.get::<_, String>(0))
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| RunStoreError::Storage(error.to_string()))?
        };
        request_ids
            .iter()
            .map(|request_id| {
                read_permission_request(&conn, request_id)?.ok_or_else(|| {
                    RunStoreError::Storage("unresolved permission disappeared".into())
                })
            })
            .collect()
    }

    pub fn get_run_events(
        &self,
        run_id: &str,
        after_sequence: u64,
        limit: usize,
    ) -> Result<Vec<RunEvent>, RunStoreError> {
        if after_sequence > i64::MAX as u64 || limit == 0 || limit > 501 {
            return Err(RunStoreError::InvalidRequest(
                "run event cursor or page size is outside the supported range".into(),
            ));
        }
        let conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM runs WHERE run_id = ?1)",
                params![run_id],
                |row| row.get(0),
            )
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        if !exists {
            return Err(RunStoreError::RunNotFound);
        }

        let mut statement = conn
            .prepare(
                "SELECT sequence, event_type, timestamp, project_id, step_id, droid_id, attempt, payload_json
                 FROM run_events WHERE run_id = ?1 AND sequence > ?2
                 ORDER BY sequence ASC LIMIT ?3",
            )
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let rows = statement
            .query_map(
                params![run_id, after_sequence as i64, limit as i64],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<u32>>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        rows.map(|row| {
            let (sequence, event_type, timestamp, project_id, step_id, droid_id, attempt, payload) =
                row.map_err(|error| RunStoreError::Storage(error.to_string()))?;
            let payload = serde_json::from_str(&payload)
                .map_err(|error| RunStoreError::Storage(error.to_string()))?;
            Ok(RunEvent {
                run_id: run_id.to_string(),
                sequence: sequence as u64,
                event_type,
                timestamp,
                project_id,
                step_id,
                droid_id,
                attempt,
                payload,
            })
        })
        .collect()
    }

    pub fn transition_run(
        &self,
        run_id: &str,
        next_status: RunStatus,
        terminal_outcome: Option<RunOutcome>,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<RunSnapshot, RunStoreError> {
        if event_type.trim().is_empty() {
            return Err(RunStoreError::InvalidRequest(
                "event type is required".into(),
            ));
        }
        validate_terminal_outcome(next_status, terminal_outcome)?;
        let payload_json = serde_json::to_vec(payload)
            .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;
        if payload_json.len() > 64 * 1024 {
            return Err(RunStoreError::InvalidRequest(
                "run event payload exceeds 65536 bytes".into(),
            ));
        }

        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let current = read_run_snapshot(&tx, run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if !current.status.can_transition_to(next_status) {
            return Err(RunStoreError::InvalidTransition {
                from: current.status.as_str().into(),
                to: next_status.as_str().into(),
            });
        }

        let now = unix_time_millis();
        tx.execute(
            "UPDATE runs SET status = ?2, terminal_outcome = ?3, updated_at = ?4,
                completed_at = CASE WHEN ?3 IS NULL THEN NULL ELSE ?4 END
             WHERE run_id = ?1",
            params![
                run_id,
                next_status.as_str(),
                terminal_outcome.map(RunOutcome::as_str),
                now,
            ],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            run_id,
            &current.project_id,
            event_type,
            None,
            None,
            None,
            payload,
            now,
        )?;
        let snapshot = read_run_snapshot(&tx, run_id)?
            .ok_or_else(|| RunStoreError::Storage("transitioned run could not be read".into()))?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(snapshot)
    }

    pub fn persist_run_thread_id(
        &self,
        run_id: &str,
        thread_id: &str,
    ) -> Result<RunSnapshot, RunStoreError> {
        if thread_id.trim().is_empty() {
            return Err(RunStoreError::InvalidRequest(
                "app-server thread ID is required".into(),
            ));
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let current = read_run_snapshot(&tx, run_id)?.ok_or(RunStoreError::RunNotFound)?;
        if current.status.is_terminal() {
            return Err(RunStoreError::InvalidRequest(
                "cannot attach an app-server thread to a terminal run".into(),
            ));
        }
        if let Some(existing_thread_id) = current.thread_id.as_deref() {
            if existing_thread_id == thread_id {
                return Ok(current);
            }
            return Err(RunStoreError::ThreadConflict);
        }

        let now = unix_time_millis();
        tx.execute(
            "UPDATE runs SET thread_id = ?2, updated_at = ?3 WHERE run_id = ?1",
            params![run_id, thread_id, now],
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        append_run_event_tx(
            &tx,
            run_id,
            &current.project_id,
            "APP_SERVER_THREAD_ATTACHED",
            None,
            None,
            None,
            &serde_json::json!({ "threadId": thread_id }),
            now,
        )?;
        let snapshot = read_run_snapshot(&tx, run_id)?
            .ok_or_else(|| RunStoreError::Storage("updated run could not be read".into()))?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(snapshot)
    }

    pub fn recover_run(
        &self,
        run_id: &str,
        evidence: &RecoveryEvidence,
    ) -> Result<(RecoveryDecision, RunSnapshot), RunStoreError> {
        let decision = match evidence {
            RecoveryEvidence::PersistedUnsentApproval { request_id } => {
                let persisted = self
                    .get_permission_request(request_id)?
                    .ok_or(RunStoreError::PermissionNotFound)?;
                if persisted.run_id == run_id && !persisted.response_sent {
                    RecoveryDecision::KeepPendingApproval
                } else {
                    RecoveryDecision::Block("PENDING_APPROVAL_STATE_MISMATCH")
                }
            }
            RecoveryEvidence::PendingUnsentApproval {
                request_id,
                stored_fingerprint,
                resumed_fingerprint,
            } => {
                let persisted = self
                    .get_permission_request(request_id)?
                    .ok_or(RunStoreError::PermissionNotFound)?;
                if persisted.run_id != run_id
                    || persisted.status != PermissionStatus::Pending
                    || persisted.response_sent
                    || persisted.payload_fingerprint != *stored_fingerprint
                {
                    RecoveryDecision::Block("PENDING_APPROVAL_STATE_MISMATCH")
                } else {
                    evaluate_recovery(&RecoveryEvidence::PendingUnsentApproval {
                        request_id: request_id.clone(),
                        stored_fingerprint: persisted.payload_fingerprint,
                        resumed_fingerprint: resumed_fingerprint.clone(),
                    })
                }
            }
            RecoveryEvidence::GrantSentWithoutResult { request_id } => {
                let persisted = self
                    .get_permission_request(request_id)?
                    .ok_or(RunStoreError::PermissionNotFound)?;
                if persisted.run_id == run_id && persisted.status == PermissionStatus::Granted {
                    RecoveryDecision::Block("GRANT_RESULT_NOT_PERSISTED")
                } else {
                    RecoveryDecision::Block("GRANT_RECOVERY_STATE_MISMATCH")
                }
            }
            RecoveryEvidence::PersistedCompletion { step_id, .. } => {
                let persisted = {
                    let conn = self
                        .conn
                        .lock()
                        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
                    read_run_step(&conn, step_id)?
                }
                .ok_or(RunStoreError::StepConflict)?;
                if persisted.run_id == run_id && persisted.status == RunStepStatus::Succeeded {
                    evaluate_recovery(evidence)
                } else {
                    RecoveryDecision::Block("COMPLETED_RESULT_NOT_PERSISTED")
                }
            }
            RecoveryEvidence::PersistedAppServerCompletion {
                step_id,
                thread_id,
                turn_id,
                item_id,
                completed_operation_item_ids,
                next_status,
                terminal_outcome,
            } => {
                let (persisted, granted_items) = {
                    let conn = self
                        .conn
                        .lock()
                        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
                    let step = read_run_step(&conn, step_id)?.ok_or(RunStoreError::StepConflict)?;
                    let mut statement = conn
                        .prepare(
                            "SELECT thread_id, turn_id, item_id FROM run_permission_requests
                             WHERE run_id = ?1 AND status = 'GRANTED' AND response_sent = 1
                               AND (step_id = ?2 OR step_id IS NULL)",
                        )
                        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
                    let rows = statement
                        .query_map(params![run_id, step_id], |row| {
                            Ok((
                                row.get::<_, Option<String>>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                row.get::<_, Option<String>>(2)?,
                            ))
                        })
                        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
                    let grants = rows
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
                    (step, grants)
                };
                let run = self.get_run_snapshot(run_id)?;
                let all_grants_have_completed_items =
                    granted_items
                        .iter()
                        .all(|(grant_thread_id, grant_turn_id, grant_item_id)| {
                            grant_thread_id.as_deref() == Some(thread_id)
                                && grant_turn_id.as_deref() == Some(turn_id)
                                && grant_item_id.as_ref().is_some_and(|grant_item_id| {
                                    completed_operation_item_ids.contains(grant_item_id)
                                })
                        });
                if persisted.run_id == run_id
                    && persisted.status == RunStepStatus::Succeeded
                    && persisted.app_server_turn_id.as_deref() == Some(turn_id)
                    && persisted.app_server_item_id.as_deref() == Some(item_id)
                    && persisted.recovery_next_status == Some(*next_status)
                    && persisted.recovery_terminal_outcome == *terminal_outcome
                    && *terminal_outcome
                        == if *next_status == RunStatus::Succeeded {
                            Some(RunOutcome::Succeeded)
                        } else {
                            None
                        }
                    && run.thread_id.as_deref() == Some(thread_id)
                    && all_grants_have_completed_items
                {
                    evaluate_recovery(evidence)
                } else {
                    RecoveryDecision::Block("APP_SERVER_COMPLETION_EVIDENCE_MISMATCH")
                }
            }
            _ => evaluate_recovery(evidence),
        };
        match decision {
            RecoveryDecision::Advance(next_status) => {
                let current = self.get_run_snapshot(run_id)?;
                if !current.status.can_transition_to(next_status) {
                    return self.block_run_for_recovery(
                        run_id,
                        "INVALID_RECOVERY_TRANSITION",
                        decision,
                    );
                }
                let snapshot = self.transition_run(
                    run_id,
                    next_status,
                    (next_status == RunStatus::Succeeded).then_some(RunOutcome::Succeeded),
                    "RUN_RECOVERY_ADVANCED",
                    &serde_json::json!({ "nextStatus": next_status.as_str() }),
                )?;
                Ok((decision, snapshot))
            }
            RecoveryDecision::Block(reason) => {
                self.block_run_for_recovery(run_id, reason, decision)
            }
            RecoveryDecision::KeepPendingApproval | RecoveryDecision::Continue => {
                let current = self.get_run_snapshot(run_id)?;
                if !matches!(
                    (decision, current.status),
                    (
                        RecoveryDecision::KeepPendingApproval,
                        RunStatus::WaitingForPermission
                    ) | (RecoveryDecision::Continue, _)
                ) {
                    return self.block_run_for_recovery(
                        run_id,
                        "PENDING_APPROVAL_STATE_MISMATCH",
                        RecoveryDecision::Block("PENDING_APPROVAL_STATE_MISMATCH"),
                    );
                }
                self.append_run_event(
                    run_id,
                    "RUN_RECOVERY_CHECKED",
                    &serde_json::json!({ "decision": format!("{decision:?}") }),
                )?;
                Ok((decision, self.get_run_snapshot(run_id)?))
            }
        }
    }

    fn block_run_for_recovery(
        &self,
        run_id: &str,
        reason: &'static str,
        decision: RecoveryDecision,
    ) -> Result<(RecoveryDecision, RunSnapshot), RunStoreError> {
        let current = self.get_run_snapshot(run_id)?;
        if current.status.is_terminal() {
            return Ok((decision, current));
        }
        let snapshot = self.transition_run(
            run_id,
            RunStatus::Blocked,
            Some(RunOutcome::Blocked),
            "RUN_BLOCKED_DURING_RECOVERY",
            &serde_json::json!({ "reason": reason }),
        )?;
        Ok((decision, snapshot))
    }

    fn append_run_event(
        &self,
        run_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<RunEvent, RunStoreError> {
        let payload_bytes = serde_json::to_vec(payload)
            .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;
        if payload_bytes.len() > 64 * 1024 {
            return Err(RunStoreError::InvalidRequest(
                "run event payload exceeds 65536 bytes".into(),
            ));
        }
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let current = read_run_snapshot(&tx, run_id)?.ok_or(RunStoreError::RunNotFound)?;
        let now = unix_time_millis();
        append_run_event_tx(
            &tx,
            run_id,
            &current.project_id,
            event_type,
            None,
            None,
            None,
            payload,
            now,
        )?;
        let last_event = read_run_events_tx(&tx, run_id, current.last_sequence, 1)?
            .into_iter()
            .next()
            .ok_or_else(|| RunStoreError::Storage("appended run event could not be read".into()))?;
        tx.commit()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(last_event)
    }

    pub fn save_project(&self, proj: &ProjectTerminal) -> Result<(), String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let metrics_json = serde_json::to_string(&proj.metrics).unwrap_or_default();
        let pipeline_json = serde_json::to_string(&proj.pipeline).unwrap_or_default();

        tx.execute(
            "INSERT INTO projects (id, name, path, model, status, accent_color, active_droid_id, thread_id, metrics_json, pipeline_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                path = excluded.path,
                model = excluded.model,
                status = excluded.status,
                accent_color = excluded.accent_color,
                active_droid_id = excluded.active_droid_id,
                thread_id = COALESCE(excluded.thread_id, projects.thread_id),
                metrics_json = excluded.metrics_json,
                pipeline_json = excluded.pipeline_json,
                updated_at = excluded.updated_at",
            params![
                proj.id,
                proj.name,
                proj.path,
                proj.model,
                proj.status,
                proj.accent_color,
                proj.active_droid_id,
                proj.thread_id,
                metrics_json,
                pipeline_json,
                now
            ],
        ).map_err(|e| e.to_string())?;

        // Atualiza droids do projeto
        tx.execute(
            "DELETE FROM project_droids WHERE project_id = ?",
            params![proj.id],
        )
        .map_err(|e| e.to_string())?;

        for (idx, d) in proj.droids.iter().enumerate() {
            tx.execute(
                "INSERT INTO project_droids (project_id, id, name, role, model, codex_model, reasoning_effort, plan_badge, avatar_type, status, status_detail, tokens, accent_color, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    proj.id,
                    d.id,
                    d.name,
                    d.role,
                    d.model,
                    d.codex_model,
                    d.reasoning_effort,
                    d.plan_badge,
                    d.avatar_type,
                    d.status,
                    d.status_detail,
                    d.tokens as i64,
                    d.accent_color,
                    idx as i64
                ],
            ).map_err(|e| e.to_string())?;
        }

        // Salva mensagens (insere apenas as que não existirem)
        for (idx, m) in proj.messages.iter().enumerate() {
            let files_json = m
                .files_modified
                .as_ref()
                .map(|f| serde_json::to_string(f).unwrap_or_default());
            let msg_created = now + idx as i64;

            tx.execute(
                "INSERT INTO project_messages (id, project_id, sender, agent_name, agent_role, agent_model, content, thinking, timestamp, status, files_modified_json, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET
                    content = excluded.content,
                    thinking = excluded.thinking,
                    status = excluded.status,
                    files_modified_json = excluded.files_modified_json",
                params![
                    m.id,
                    proj.id,
                    m.sender,
                    m.agent_name,
                    m.agent_role,
                    m.agent_model,
                    m.content,
                    m.thinking,
                    m.timestamp,
                    m.status,
                    files_json,
                    msg_created
                ],
            ).map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM projects WHERE id = ?", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_message(&self, project_id: &str, msg: &ChatMessage) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let files_json = msg
            .files_modified
            .as_ref()
            .map(|f| serde_json::to_string(f).unwrap_or_default());

        conn.execute(
            "INSERT INTO project_messages (id, project_id, sender, agent_name, agent_role, agent_model, content, thinking, timestamp, status, files_modified_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                thinking = excluded.thinking,
                status = excluded.status,
                files_modified_json = excluded.files_modified_json",
            params![
                msg.id,
                project_id,
                msg.sender,
                msg.agent_name,
                msg.agent_role,
                msg.agent_model,
                msg.content,
                msg.thinking,
                msg.timestamp,
                msg.status,
                files_json,
                now
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn update_project_thread_id(
        &self,
        project_id: &str,
        thread_id: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE projects SET thread_id = ?1 WHERE id = ?2",
            params![thread_id, project_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn project_path_for_run(&self, project_id: &str) -> Result<PathBuf, RunStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        conn.query_row(
            "SELECT path FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get::<_, String>(0),
        )
        .map(PathBuf::from)
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => RunStoreError::ProjectNotFound,
            _ => RunStoreError::Storage(error.to_string()),
        })
    }

    pub fn delete_project_message(&self, project_id: &str, msg_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM project_messages WHERE project_id = ?1 AND id = ?2",
            params![project_id, msg_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_project_messages(&self, project_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM project_messages WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn ensure_project_exists(
    conn: &Connection,
    project_id: &str,
) -> Result<(), ExecutionPolicyStoreError> {
    let exists = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            params![project_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
    if exists {
        Ok(())
    } else {
        Err(ExecutionPolicyStoreError::ProjectNotFound)
    }
}

fn ensure_project_droid_exists(
    conn: &Connection,
    project_id: &str,
    droid_id: &str,
) -> Result<(), ExecutionPolicyStoreError> {
    let exists = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM project_droids WHERE project_id = ?1 AND id = ?2)",
            params![project_id, droid_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| ExecutionPolicyStoreError::Storage(error.to_string()))?;
    if exists {
        Ok(())
    } else {
        Err(ExecutionPolicyStoreError::DroidNotFound)
    }
}

fn migrate_legacy_database(source_path: &Path, destination_path: &Path) -> Result<(), String> {
    let source = Connection::open(source_path).map_err(|error| error.to_string())?;
    let mut destination = Connection::open(destination_path).map_err(|error| error.to_string())?;
    let backup = Backup::new(&source, &mut destination).map_err(|error| error.to_string())?;
    backup
        .run_to_completion(32, Duration::from_millis(50), None)
        .map_err(|error| error.to_string())
}

fn validate_new_run(run: &NewRunRecord) -> Result<(), RunStoreError> {
    if run.run_id.trim().is_empty()
        || run.project_id.trim().is_empty()
        || run.idempotency_key.trim().is_empty()
        || run.task_title.trim().is_empty()
        || run.prompt.trim().is_empty()
        || run.verification_preset_id.trim().is_empty()
        || run.droid_ids.is_empty()
    {
        return Err(RunStoreError::InvalidRequest(
            "run ID, project, idempotency key, title, prompt, preset and droids are required"
                .into(),
        ));
    }
    let unique_droids: std::collections::HashSet<_> = run.droid_ids.iter().collect();
    if unique_droids.len() != run.droid_ids.len() {
        return Err(RunStoreError::InvalidRequest(
            "droid IDs must be unique".into(),
        ));
    }
    run.budget_limits
        .validate()
        .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;
    let policy_snapshot: serde_json::Value = serde_json::from_str(&run.policy_snapshot_json)
        .map_err(|error| {
            RunStoreError::InvalidRequest(format!("invalid policy snapshot: {error}"))
        })?;
    if !policy_snapshot.is_object() {
        return Err(RunStoreError::InvalidRequest(
            "policy snapshot must be a JSON object".into(),
        ));
    }
    Ok(())
}

fn validate_terminal_outcome(
    status: RunStatus,
    outcome: Option<RunOutcome>,
) -> Result<(), RunStoreError> {
    let valid = match status {
        RunStatus::Succeeded => outcome == Some(RunOutcome::Succeeded),
        RunStatus::Failed => outcome == Some(RunOutcome::Failed),
        RunStatus::Blocked => matches!(
            outcome,
            Some(RunOutcome::Blocked | RunOutcome::BudgetExhausted)
        ),
        RunStatus::Cancelled => outcome == Some(RunOutcome::Cancelled),
        _ => outcome.is_none(),
    };
    if !valid {
        return Err(RunStoreError::InvalidRequest(
            "terminal status and outcome are inconsistent".into(),
        ));
    }
    Ok(())
}

fn run_status_from_str(value: &str) -> Result<RunStatus, RunStoreError> {
    match value {
        "QUEUED" => Ok(RunStatus::Queued),
        "PLANNING" => Ok(RunStatus::Planning),
        "EXECUTING" => Ok(RunStatus::Executing),
        "VERIFYING" => Ok(RunStatus::Verifying),
        "REPAIRING" => Ok(RunStatus::Repairing),
        "REVIEWING" => Ok(RunStatus::Reviewing),
        "WAITING_FOR_PERMISSION" => Ok(RunStatus::WaitingForPermission),
        "SUCCEEDED" => Ok(RunStatus::Succeeded),
        "FAILED" => Ok(RunStatus::Failed),
        "BLOCKED" => Ok(RunStatus::Blocked),
        "CANCELLED" => Ok(RunStatus::Cancelled),
        _ => Err(RunStoreError::Storage(format!(
            "unknown run status {value}"
        ))),
    }
}

fn run_outcome_from_str(value: &str) -> Result<RunOutcome, RunStoreError> {
    match value {
        "SUCCEEDED" => Ok(RunOutcome::Succeeded),
        "FAILED" => Ok(RunOutcome::Failed),
        "BLOCKED" => Ok(RunOutcome::Blocked),
        "BUDGET_EXHAUSTED" => Ok(RunOutcome::BudgetExhausted),
        "CANCELLED" => Ok(RunOutcome::Cancelled),
        _ => Err(RunStoreError::Storage(format!(
            "unknown run outcome {value}"
        ))),
    }
}

fn read_run_snapshot(
    conn: &Connection,
    run_id: &str,
) -> Result<Option<RunSnapshot>, RunStoreError> {
    type RunRow = (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        Option<String>,
        i64,
        i64,
        Option<i64>,
    );
    let row: Option<RunRow> = conn
        .query_row(
            "SELECT run_id, project_id, droid_ids_json, idempotency_key, task_title,
                    verification_preset_id, status, terminal_outcome,
                    max_model_turns_per_run, max_elapsed_seconds_per_run, max_tokens_per_run,
                    model_turns_started, elapsed_seconds, reported_tokens, thread_id,
                    created_at, updated_at, completed_at
             FROM runs WHERE run_id = ?1",
            params![run_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                ))
            },
        )
        .optional()
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let Some((
        run_id,
        project_id,
        droid_ids_json,
        idempotency_key,
        task_title,
        verification_preset_id,
        status,
        outcome,
        max_turns,
        max_elapsed,
        max_tokens,
        turns_started,
        elapsed,
        reported_tokens,
        thread_id,
        created_at,
        updated_at,
        completed_at,
    )) = row
    else {
        return Ok(None);
    };
    let droid_ids: Vec<String> = serde_json::from_str(&droid_ids_json)
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let status = run_status_from_str(&status)?;
    let terminal_outcome = outcome.as_deref().map(run_outcome_from_str).transpose()?;
    let last_sequence: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sequence), 0) FROM run_events WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let steps = read_run_steps(conn, &run_id)?;
    let pending_permissions = read_pending_permission_summaries(conn, &run_id)?;
    let reviewer_droid_id = droid_ids.get(1).cloned();
    Ok(Some(RunSnapshot {
        run_id,
        project_id,
        droid_ids,
        reviewer_droid_id,
        idempotency_key,
        task_title,
        verification_preset_id,
        status,
        terminal_outcome,
        budget_limits: RunBudgetLimits {
            max_model_turns_per_run: max_turns as u64,
            max_elapsed_seconds_per_run: max_elapsed as u64,
            max_tokens_per_run: max_tokens as u64,
        },
        usage: RunBudgetUsage {
            model_turns_started: turns_started as u64,
            elapsed_seconds: elapsed as u64,
            reported_tokens: reported_tokens as u64,
        },
        thread_id,
        steps,
        pending_permissions,
        created_at,
        updated_at,
        completed_at,
        last_sequence: last_sequence as u64,
    }))
}

#[allow(clippy::too_many_arguments)]
fn append_run_event_tx(
    tx: &rusqlite::Transaction<'_>,
    run_id: &str,
    project_id: &str,
    event_type: &str,
    step_id: Option<&str>,
    droid_id: Option<&str>,
    attempt: Option<u32>,
    payload: &serde_json::Value,
    timestamp: i64,
) -> Result<(), RunStoreError> {
    if event_type.trim().is_empty() {
        return Err(RunStoreError::InvalidRequest(
            "event type is required".into(),
        ));
    }
    if !payload.is_object() {
        return Err(RunStoreError::InvalidRequest(
            "run event payload must be a JSON object".into(),
        ));
    }
    let payload_json = serde_json::to_string(payload)
        .map_err(|error| RunStoreError::InvalidRequest(error.to_string()))?;
    if payload_json.len() > 64 * 1024 {
        return Err(RunStoreError::InvalidRequest(
            "run event payload exceeds 65536 bytes".into(),
        ));
    }
    let sequence: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(sequence), 0) + 1 FROM run_events WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    tx.execute(
        "INSERT INTO run_events
            (run_id, sequence, event_type, timestamp, project_id, step_id, droid_id, attempt, payload_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![run_id, sequence, event_type, timestamp, project_id, step_id, droid_id, attempt, payload_json],
    )
    .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    Ok(())
}

fn read_run_events_tx(
    conn: &Connection,
    run_id: &str,
    after_sequence: u64,
    limit: usize,
) -> Result<Vec<RunEvent>, RunStoreError> {
    let mut statement = conn
        .prepare(
            "SELECT sequence, event_type, timestamp, project_id, step_id, droid_id, attempt, payload_json
             FROM run_events WHERE run_id = ?1 AND sequence > ?2
             ORDER BY sequence ASC LIMIT ?3",
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let rows = statement
        .query_map(
            params![run_id, after_sequence as i64, limit as i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<u32>>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    rows.map(|row| {
        let (sequence, event_type, timestamp, project_id, step_id, droid_id, attempt, payload) =
            row.map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let payload = serde_json::from_str(&payload)
            .map_err(|error| RunStoreError::Storage(error.to_string()))?;
        Ok(RunEvent {
            run_id: run_id.to_string(),
            sequence: sequence as u64,
            event_type,
            timestamp,
            project_id,
            step_id,
            droid_id,
            attempt,
            payload,
        })
    })
    .collect()
}

fn read_run_step(conn: &Connection, step_id: &str) -> Result<Option<RunStep>, RunStoreError> {
    let row = conn
        .query_row(
            "SELECT step_id, run_id, ordinal, stage, droid_id, attempt, status,
                    started_at, ended_at, result_summary, verification_json,
                    app_server_turn_id, app_server_item_id, recovery_next_status,
                    recovery_terminal_outcome, app_server_thread_id
             FROM run_steps WHERE step_id = ?1",
            params![step_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, u32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, Option<String>>(14)?,
                    row.get::<_, Option<String>>(15)?,
                ))
            },
        )
        .optional()
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let Some((
        step_id,
        run_id,
        ordinal,
        stage,
        droid_id,
        attempt,
        status,
        started_at,
        ended_at,
        summary,
        verification,
        app_server_turn_id,
        app_server_item_id,
        recovery_next_status,
        recovery_terminal_outcome,
        app_server_thread_id,
    )) = row
    else {
        return Ok(None);
    };
    let status = match status.as_str() {
        "PENDING" => RunStepStatus::Pending,
        "IN_PROGRESS" => RunStepStatus::InProgress,
        "SUCCEEDED" => RunStepStatus::Succeeded,
        "FAILED" => RunStepStatus::Failed,
        "SKIPPED" => RunStepStatus::Skipped,
        _ => {
            return Err(RunStoreError::Storage(format!(
                "unknown run step status {status}"
            )));
        }
    };
    let verification = verification
        .map(|json| {
            serde_json::from_str(&json).map_err(|error| RunStoreError::Storage(error.to_string()))
        })
        .transpose()?;
    let recovery_next_status = recovery_next_status
        .map(|value| run_status_from_str(&value))
        .transpose()?;
    let recovery_terminal_outcome = recovery_terminal_outcome
        .map(|value| run_outcome_from_str(&value))
        .transpose()?;
    Ok(Some(RunStep {
        step_id,
        run_id,
        ordinal,
        stage,
        droid_id,
        attempt,
        status,
        started_at,
        ended_at,
        result_summary: summary,
        verification,
        app_server_thread_id,
        app_server_turn_id,
        app_server_item_id,
        recovery_next_status,
        recovery_terminal_outcome,
    }))
}

fn read_run_steps(conn: &Connection, run_id: &str) -> Result<Vec<RunStep>, RunStoreError> {
    let mut statement = conn
        .prepare("SELECT step_id FROM run_steps WHERE run_id = ?1 ORDER BY ordinal, attempt")
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let rows = statement
        .query_map(params![run_id], |row| row.get::<_, String>(0))
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let step_ids = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    step_ids
        .iter()
        .map(|step_id| read_run_step(conn, step_id)?.ok_or(RunStoreError::StepConflict))
        .collect()
}

fn read_pending_permission_summaries(
    conn: &Connection,
    run_id: &str,
) -> Result<Vec<RunPermissionSummary>, RunStoreError> {
    let mut statement = conn
        .prepare(
            "SELECT request_id, step_id, droid_id, capability, action, resource, rationale,
                    status, alternative_index, created_at, payload_fingerprint
             FROM run_permission_requests WHERE run_id = ?1 AND response_sent = 0
             ORDER BY created_at, request_id",
        )
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let rows = statement
        .query_map(params![run_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, u32>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, String>(10)?,
            ))
        })
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    rows.map(|row| {
        let (
            request_id,
            step_id,
            droid_id,
            capability,
            action,
            resource,
            rationale,
            status,
            alternative_index,
            created_at,
            payload_fingerprint,
        ) = row.map_err(|error| RunStoreError::Storage(error.to_string()))?;
        let status = match status.as_str() {
            "PENDING" => PermissionStatus::Pending,
            "GRANTED" => PermissionStatus::Granted,
            "DENIED" => PermissionStatus::Denied,
            _ => {
                return Err(RunStoreError::Storage(format!(
                    "unknown permission status {status}"
                )));
            }
        };
        Ok(RunPermissionSummary {
            request_id,
            step_id,
            droid_id,
            capability,
            action,
            resource,
            rationale,
            status,
            alternative_index,
            created_at,
            payload_fingerprint,
        })
    })
    .collect()
}

fn read_permission_request(
    conn: &Connection,
    request_id: &str,
) -> Result<Option<RunPermissionRequest>, RunStoreError> {
    type PermissionRow = (
        String,
        String,
        String,
        Option<String>,
        String,
        String,
        String,
        String,
        String,
        String,
        u32,
        i64,
        Option<i64>,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        bool,
        Option<String>,
    );
    let row: Option<PermissionRow> = conn
        .query_row(
            "SELECT request_id, run_id, project_id, step_id, droid_id, capability, action,
                    resource, rationale, status, alternative_index, created_at, decided_at,
                    grant_scope, resume_status, app_server_generation, thread_id, turn_id,
                    item_id, json_rpc_request_id, app_server_approval_id, payload_json,
                    payload_fingerprint, response_sent, decision_idempotency_key
             FROM run_permission_requests WHERE request_id = ?1",
            params![request_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                    row.get(19)?,
                    row.get(20)?,
                    row.get(21)?,
                    row.get(22)?,
                    row.get(23)?,
                    row.get(24)?,
                ))
            },
        )
        .optional()
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    let Some((
        request_id,
        run_id,
        project_id,
        step_id,
        droid_id,
        capability,
        action,
        resource,
        rationale,
        status,
        alternative_index,
        created_at,
        decided_at,
        grant_scope,
        resume_status,
        app_server_generation,
        thread_id,
        turn_id,
        item_id,
        json_rpc_request_id,
        app_server_approval_id,
        payload_json,
        payload_fingerprint,
        response_sent,
        decision_idempotency_key,
    )) = row
    else {
        return Ok(None);
    };
    let status = match status.as_str() {
        "PENDING" => PermissionStatus::Pending,
        "GRANTED" => PermissionStatus::Granted,
        "DENIED" => PermissionStatus::Denied,
        _ => {
            return Err(RunStoreError::Storage(format!(
                "unknown permission status {status}"
            )));
        }
    };
    let resume_status = run_status_from_str(&resume_status)?;
    let payload = serde_json::from_str(&payload_json)
        .map_err(|error| RunStoreError::Storage(error.to_string()))?;
    Ok(Some(RunPermissionRequest {
        request_id,
        run_id,
        project_id,
        step_id,
        droid_id,
        capability,
        action,
        resource,
        rationale,
        status,
        alternative_index,
        created_at,
        decided_at,
        grant_scope,
        resume_status,
        app_server_generation,
        thread_id,
        turn_id,
        item_id,
        json_rpc_request_id,
        app_server_approval_id,
        payload,
        payload_fingerprint,
        response_sent,
        decision_idempotency_key,
    }))
}

fn unix_time_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_run(db: &Database, run_id: &str, idempotency_key: &str) -> NewRunRecord {
        let project = db
            .list_projects()
            .unwrap()
            .into_iter()
            .find(|project| project.id == "proj-notex")
            .unwrap();
        let budgets = RunBudgetLimits {
            max_model_turns_per_run: 8,
            max_elapsed_seconds_per_run: 600,
            max_tokens_per_run: 60_000,
        };
        db.save_project_execution_settings(&project.id, budgets)
            .unwrap();
        NewRunRecord {
            run_id: run_id.into(),
            project_id: project.id,
            droid_ids: vec![project.droids[0].id.clone()],
            idempotency_key: idempotency_key.into(),
            task_title: "Build durable coordinator".into(),
            prompt: "Implement the run persistence foundation".into(),
            verification_preset_id: "cargo-test".into(),
            policy_snapshot_json: "{}".into(),
            budget_limits: budgets,
        }
    }

    fn new_permission(
        run: &NewRunRecord,
        request_id: &str,
        alternative_index: u32,
    ) -> NewRunPermissionRequest {
        NewRunPermissionRequest {
            request_id: request_id.into(),
            run_id: run.run_id.clone(),
            step_id: None,
            droid_id: run.droid_ids[0].clone(),
            capability: "write_project".into(),
            action: "write_file".into(),
            resource: "src/lib.rs".into(),
            rationale: "Implement the requested change".into(),
            alternative_index,
            app_server_generation: Some("generation-1".into()),
            thread_id: Some("thread-1".into()),
            turn_id: Some("turn-1".into()),
            item_id: Some("item-1".into()),
            json_rpc_request_id: Some("0".into()),
            app_server_approval_id: None,
            payload_json: r#"{"command":"write","path":"src/lib.rs"}"#.into(),
            payload_fingerprint: "fingerprint-1".into(),
        }
    }

    fn project(id: &str) -> ProjectTerminal {
        ProjectTerminal {
            id: id.into(),
            name: "Budget Test".into(),
            path: std::env::temp_dir().to_string_lossy().into_owned(),
            model: "test-model".into(),
            status: "ACTIVE".into(),
            accent_color: "#00f0ff".into(),
            metrics: ProjectMetrics {
                tokens_in: 0,
                tokens_out: 0,
                latency_ms: 0,
                cpu: String::new(),
                mem: String::new(),
            },
            droids: Vec::new(),
            active_droid_id: "test-droid".into(),
            messages: Vec::new(),
            pipeline: PipelineState {
                is_active: false,
                stage: "IDLE".into(),
                current_droid_id: "test-droid".into(),
                prompt: String::new(),
                progress_percent: 0,
                status_text: String::new(),
            },
            thread_id: None,
        }
    }

    #[test]
    fn project_execution_settings_are_required_and_persisted() {
        let db = Database::init(":memory:").unwrap();
        let project = project("proj-budget-test");
        db.save_project(&project).unwrap();

        assert_eq!(
            db.get_project_execution_settings(&project.id).unwrap(),
            None
        );

        let budgets = RunBudgetLimits {
            max_model_turns_per_run: 3,
            max_elapsed_seconds_per_run: 900,
            max_tokens_per_run: 80_000,
        };
        let saved = db
            .save_project_execution_settings(&project.id, budgets)
            .unwrap();
        assert_eq!(saved, ProjectExecutionSettings { budgets });
        assert_eq!(
            db.get_project_execution_settings(&project.id).unwrap(),
            Some(saved)
        );
    }

    #[test]
    fn settings_reject_zero_values_and_unknown_projects() {
        let db = Database::init(":memory:").unwrap();
        let invalid = RunBudgetLimits {
            max_model_turns_per_run: 0,
            max_elapsed_seconds_per_run: 900,
            max_tokens_per_run: 80_000,
        };
        assert!(
            db.save_project_execution_settings("proj-missing", invalid)
                .is_err()
        );
        assert!(db.get_project_execution_settings("proj-missing").is_err());

        let project = project("proj-budget-invalid");
        db.save_project(&project).unwrap();
        assert!(
            db.save_project_execution_settings(&project.id, invalid)
                .is_err()
        );
    }

    #[test]
    fn run_creation_is_idempotent_and_state_event_commit_together() {
        let db = Database::init(":memory:").unwrap();
        let request = new_run(&db, "run-atomic-01", "request-01");
        let created = db.create_run_record(&request).unwrap();
        assert_eq!(created.status, RunStatus::Queued);
        assert_eq!(created.last_sequence, 1);

        let duplicate = db
            .create_run_record(&NewRunRecord {
                run_id: "ignored-retry-id".into(),
                ..request.clone()
            })
            .unwrap();
        assert_eq!(duplicate.run_id, created.run_id);

        let mut conflicting = request.clone();
        conflicting.task_title = "Different task".into();
        assert!(matches!(
            db.create_run_record(&conflicting),
            Err(RunStoreError::IdempotencyConflict)
        ));

        let planning = db
            .transition_run(
                &request.run_id,
                RunStatus::Planning,
                None,
                "RUN_PLANNING",
                &serde_json::json!({"stage":"planning"}),
            )
            .unwrap();
        assert_eq!(planning.status, RunStatus::Planning);
        assert_eq!(planning.last_sequence, 2);
        let events = db.get_run_events(&request.run_id, 0, 10).unwrap();
        assert_eq!(
            events
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );

        assert!(matches!(
            db.transition_run(
                &request.run_id,
                RunStatus::Succeeded,
                Some(RunOutcome::Succeeded),
                "RUN_SUCCEEDED",
                &serde_json::json!({}),
            ),
            Err(RunStoreError::InvalidTransition { .. })
        ));
        let unchanged = db.get_run_snapshot(&request.run_id).unwrap();
        assert_eq!(unchanged.status, RunStatus::Planning);
        assert_eq!(unchanged.last_sequence, 2);
    }

    #[test]
    fn model_turn_reservation_and_run_usage_are_durable_and_monotonic() {
        let db = Database::init(":memory:").unwrap();
        let run = new_run(&db, "run-budget-reservation-01", "budget-reservation-01");
        db.create_run_record(&run).unwrap();
        db.transition_run(
            &run.run_id,
            RunStatus::Planning,
            None,
            "RUN_PLANNING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.transition_run(
            &run.run_id,
            RunStatus::Executing,
            None,
            "RUN_EXECUTING",
            &serde_json::json!({}),
        )
        .unwrap();

        let reserved = db.reserve_run_model_turn(&run.run_id).unwrap();
        assert_eq!(reserved.usage.model_turns_started, 1);
        let updated = db
            .record_run_usage(
                &run.run_id,
                RunBudgetUsage {
                    model_turns_started: 1,
                    elapsed_seconds: 12,
                    reported_tokens: 345,
                },
            )
            .unwrap();
        assert_eq!(updated.usage.elapsed_seconds, 12);
        assert_eq!(updated.usage.reported_tokens, 345);
        assert!(matches!(
            db.record_run_usage(
                &run.run_id,
                RunBudgetUsage {
                    model_turns_started: 1,
                    elapsed_seconds: 11,
                    reported_tokens: 345,
                }
            ),
            Err(RunStoreError::InvalidRequest(_))
        ));
    }

    #[test]
    fn recovery_keeps_matching_pending_approval_and_blocks_uncertain_grants() {
        let db = Database::init(":memory:").unwrap();
        let request = new_run(&db, "run-recovery-pending", "request-pending");
        db.create_run_record(&request).unwrap();
        db.transition_run(
            &request.run_id,
            RunStatus::Planning,
            None,
            "RUN_PLANNING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.transition_run(
            &request.run_id,
            RunStatus::Executing,
            None,
            "RUN_EXECUTING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.create_pending_permission(&new_permission(&request, "approval-pending-1", 0))
            .unwrap();

        let (decision, pending) = db
            .recover_run(
                &request.run_id,
                &RecoveryEvidence::PendingUnsentApproval {
                    request_id: "approval-pending-1".into(),
                    stored_fingerprint: "fingerprint-1".into(),
                    resumed_fingerprint: "fingerprint-1".into(),
                },
            )
            .unwrap();
        assert_eq!(decision, RecoveryDecision::KeepPendingApproval);
        assert_eq!(pending.status, RunStatus::WaitingForPermission);

        let request2 = new_run(&db, "run-recovery-grant", "request-grant");
        db.create_run_record(&request2).unwrap();
        db.transition_run(
            &request2.run_id,
            RunStatus::Planning,
            None,
            "RUN_PLANNING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.transition_run(
            &request2.run_id,
            RunStatus::Executing,
            None,
            "RUN_EXECUTING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.create_pending_permission(&new_permission(&request2, "approval-grant-1", 0))
            .unwrap();
        db.decide_run_permission("approval-grant-1", true, "decision-grant-1", Some("task"))
            .unwrap();
        let (decision, blocked) = db
            .recover_run(
                &request2.run_id,
                &RecoveryEvidence::GrantSentWithoutResult {
                    request_id: "approval-grant-1".into(),
                },
            )
            .unwrap();
        assert_eq!(
            decision,
            RecoveryDecision::Block("GRANT_RESULT_NOT_PERSISTED")
        );
        assert_eq!(blocked.status, RunStatus::Blocked);
        assert_eq!(blocked.terminal_outcome, Some(RunOutcome::Blocked));
    }

    #[test]
    fn persisted_completion_advances_without_replay_and_fingerprint_mismatch_blocks() {
        let db = Database::init(":memory:").unwrap();
        let completed = new_run(&db, "run-recovery-completed", "request-completed");
        db.create_run_record(&completed).unwrap();
        db.transition_run(
            &completed.run_id,
            RunStatus::Planning,
            None,
            "RUN_PLANNING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.transition_run(
            &completed.run_id,
            RunStatus::Executing,
            None,
            "RUN_EXECUTING",
            &serde_json::json!({}),
        )
        .unwrap();
        let completed_step = db
            .create_run_step(&NewRunStep {
                step_id: "step-completed-1".into(),
                run_id: completed.run_id.clone(),
                ordinal: 0,
                stage: "CODING".into(),
                droid_id: completed.droid_ids[0].clone(),
                attempt: 1,
            })
            .unwrap();
        db.start_run_step(&completed_step.step_id).unwrap();
        db.complete_run_step(
            &completed_step.step_id,
            RunStepStatus::Succeeded,
            "Change persisted",
            None,
        )
        .unwrap();
        let (decision, advanced) = db
            .recover_run(
                &completed.run_id,
                &RecoveryEvidence::PersistedCompletion {
                    step_id: completed_step.step_id,
                    next_status: RunStatus::Verifying,
                },
            )
            .unwrap();
        assert_eq!(decision, RecoveryDecision::Advance(RunStatus::Verifying));
        assert_eq!(advanced.status, RunStatus::Verifying);

        let pending = new_run(&db, "run-recovery-mismatch", "request-mismatch");
        db.create_run_record(&pending).unwrap();
        db.transition_run(
            &pending.run_id,
            RunStatus::Planning,
            None,
            "RUN_PLANNING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.transition_run(
            &pending.run_id,
            RunStatus::Executing,
            None,
            "RUN_EXECUTING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.create_pending_permission(&new_permission(&pending, "approval-mismatch-1", 0))
            .unwrap();
        let (_, blocked) = db
            .recover_run(
                &pending.run_id,
                &RecoveryEvidence::PendingUnsentApproval {
                    request_id: "approval-mismatch-1".into(),
                    stored_fingerprint: "fingerprint-1".into(),
                    resumed_fingerprint: "fingerprint-new".into(),
                },
            )
            .unwrap();
        assert_eq!(blocked.status, RunStatus::Blocked);
    }

    #[test]
    fn step_and_permission_records_are_durable_and_decisions_are_immutable() {
        let db = Database::init(":memory:").unwrap();
        let run = new_run(&db, "run-steps-permissions", "request-steps");
        db.create_run_record(&run).unwrap();
        let step = db
            .create_run_step(&NewRunStep {
                step_id: "step-architect-1".into(),
                run_id: run.run_id.clone(),
                ordinal: 0,
                stage: "PLANNING".into(),
                droid_id: run.droid_ids[0].clone(),
                attempt: 1,
            })
            .unwrap();
        assert_eq!(step.status, RunStepStatus::Pending);
        assert_eq!(
            db.start_run_step(&step.step_id).unwrap().status,
            RunStepStatus::InProgress
        );
        let completed = db
            .complete_run_step(
                &step.step_id,
                RunStepStatus::Succeeded,
                "Plan recorded",
                Some(r#"{"checks":["requirements"]}"#),
            )
            .unwrap();
        assert_eq!(completed.status, RunStepStatus::Succeeded);
        assert_eq!(completed.verification.unwrap()["checks"][0], "requirements");

        db.transition_run(
            &run.run_id,
            RunStatus::Planning,
            None,
            "RUN_PLANNING",
            &serde_json::json!({}),
        )
        .unwrap();
        db.transition_run(
            &run.run_id,
            RunStatus::Executing,
            None,
            "RUN_EXECUTING",
            &serde_json::json!({}),
        )
        .unwrap();
        let pending = db
            .create_pending_permission(&NewRunPermissionRequest {
                request_id: "approval-1".into(),
                run_id: run.run_id.clone(),
                step_id: Some(step.step_id),
                droid_id: run.droid_ids[0].clone(),
                capability: "write_project".into(),
                action: "write_file".into(),
                resource: "src/lib.rs".into(),
                rationale: "Implement the requested feature".into(),
                alternative_index: 0,
                app_server_generation: Some("generation-1".into()),
                thread_id: Some("thread-1".into()),
                turn_id: Some("turn-1".into()),
                item_id: Some("item-1".into()),
                json_rpc_request_id: Some("0".into()),
                app_server_approval_id: None,
                payload_json: r#"{"command":"write","path":"src/lib.rs"}"#.into(),
                payload_fingerprint: "sha256:fingerprint".into(),
            })
            .unwrap();
        assert_eq!(pending.status, PermissionStatus::Pending);
        assert_eq!(
            db.get_run_snapshot(&run.run_id).unwrap().status,
            RunStatus::WaitingForPermission
        );
        let waiting_snapshot = db.get_run_snapshot(&run.run_id).unwrap();
        assert_eq!(waiting_snapshot.pending_permissions.len(), 1);
        assert_eq!(
            serde_json::to_value(&waiting_snapshot).unwrap()["pendingPermissions"][0]
                .get("payload"),
            None
        );

        let (granted, resumed) = db
            .decide_run_permission("approval-1", true, "decision-1", Some("task"))
            .unwrap();
        assert_eq!(granted.status, PermissionStatus::Granted);
        assert_eq!(
            granted.decision_idempotency_key.as_deref(),
            Some("decision-1")
        );
        assert!(!granted.response_sent);
        assert_eq!(resumed.status, RunStatus::WaitingForPermission);

        let (retry, _) = db
            .decide_run_permission("approval-1", true, "decision-1", Some("task"))
            .unwrap();
        assert_eq!(retry.request_id, granted.request_id);
        assert!(matches!(
            db.decide_run_permission("approval-1", false, "decision-1", None),
            Err(RunStoreError::PermissionDecisionConflict)
        ));
        assert!(
            db.mark_permission_response_sent("approval-1")
                .unwrap()
                .response_sent
        );
        db.create_pending_permission(&new_permission(&run, "approval-denial-4", 3))
            .unwrap();
        let (denied, blocked) = db
            .decide_run_permission("approval-denial-4", false, "decision-deny-4", None)
            .unwrap();
        assert_eq!(denied.status, PermissionStatus::Denied);
        assert_eq!(blocked.status, RunStatus::Blocked);
        assert_eq!(blocked.terminal_outcome, Some(RunOutcome::Blocked));
        assert_eq!(
            db.get_run_events(&run.run_id, 0, 20)
                .unwrap()
                .iter()
                .find(|event| event.event_type == "PERMISSION_RESPONSE_SENT")
                .unwrap()
                .event_type,
            "PERMISSION_RESPONSE_SENT"
        );
    }

    #[test]
    fn project_policy_persists_and_droid_overrides_cannot_exceed_it() {
        let db = Database::init(":memory:").unwrap();
        let project = db.list_projects().unwrap().into_iter().next().unwrap();
        let droid_id = project.droids[0].id.clone();
        let overrides = DroidExecutionPolicyOverrides {
            web_search: Some(false),
            write_roots: Some(vec!["src/private".into()]),
            ..Default::default()
        };
        assert!(matches!(
            db.save_droid_execution_policy_overrides(&project.id, &droid_id, &overrides),
            Err(ExecutionPolicyStoreError::ProjectPolicyRequired)
        ));

        let project_policy = ProjectExecutionPolicy {
            read_project: true,
            write_roots: vec!["src".into(), "docs".into()],
            reference_project_ids: vec![],
            web_search: true,
            shell_network: false,
        };
        db.save_project_execution_policy(&project.id, &project_policy)
            .unwrap();
        db.save_droid_execution_policy_overrides(&project.id, &droid_id, &overrides)
            .unwrap();
        let loaded_project = db
            .get_project_execution_policy(&project.id)
            .unwrap()
            .unwrap();
        let loaded_overrides = db
            .get_droid_execution_policy_overrides(&project.id, &droid_id)
            .unwrap()
            .unwrap();
        let effective = loaded_project.effective_for(&loaded_overrides).unwrap();
        assert!(effective.allows_project_write("src/private/module.rs"));
        assert!(!effective.allows_project_write("src/public.rs"));
        assert!(!effective.web_search);
        assert!(!effective.shell_network);

        let narrowed_project = ProjectExecutionPolicy {
            write_roots: vec!["docs".into()],
            ..project_policy
        };
        assert!(matches!(
            db.save_project_execution_policy(&project.id, &narrowed_project),
            Err(ExecutionPolicyStoreError::ExistingDroidPolicyConflict)
        ));
        assert_eq!(
            db.get_project_execution_policy(&project.id).unwrap(),
            Some(loaded_project)
        );
    }
}
