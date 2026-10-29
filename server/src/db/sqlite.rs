use rusqlite::backup::Backup;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{error, info};

use crate::db::models::{ChatMessage, DroidInfo, PipelineState, ProjectMetrics, ProjectTerminal};
use crate::domain::run::{BudgetConfigError, ProjectExecutionSettings, RunBudgetLimits};

#[derive(Debug, thiserror::Error)]
pub enum ExecutionSettingsStoreError {
    #[error("project not found")]
    ProjectNotFound,
    #[error("invalid project execution budget: {0}")]
    InvalidBudget(#[from] BudgetConfigError),
    #[error("execution settings storage error: {0}")]
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

fn migrate_legacy_database(source_path: &Path, destination_path: &Path) -> Result<(), String> {
    let source = Connection::open(source_path).map_err(|error| error.to_string())?;
    let mut destination = Connection::open(destination_path).map_err(|error| error.to_string())?;
    let backup = Backup::new(&source, &mut destination).map_err(|error| error.to_string())?;
    backup
        .run_to_completion(32, Duration::from_millis(50), None)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
