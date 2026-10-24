use serde::{Deserialize, Serialize};

/// Níveis de prioridade de uma tarefa.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Estados do ciclo de vida de uma tarefa.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Review,
    Completed,
    Failed,
}

/// Dados contextuais e instruções carregadas pela tarefa.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskPayload {
    pub prompt: String,
    pub context_files: Option<Vec<String>>,
    pub scratchpad: Option<String>,
}

/// Entidade de Tarefa vinculada a um projeto e agente.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub task_id: String,
    pub project_id: String,
    pub target_agent_id: String,
    pub title: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub payload: TaskPayload,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Dados da tarefa embutida na requisição externa de despacho.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DispatchTaskInput {
    pub task_id: String,
    pub title: String,
    pub priority: TaskPriority,
    pub payload: TaskPayload,
}

/// Contrato HTTP para o endpoint POST /api/v1/bridge/dispatch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DispatchPayload {
    pub source: String,
    pub project_id: String,
    pub target_agent_id: String,
    pub task: DispatchTaskInput,
    pub callback_webhook: Option<String>,
}

impl Task {
    /// Constrói uma nova tarefa a partir de uma requisição de despacho.
    pub fn from_dispatch(
        dispatch: &DispatchPayload,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            task_id: dispatch.task.task_id.clone(),
            project_id: dispatch.project_id.clone(),
            target_agent_id: dispatch.target_agent_id.clone(),
            title: dispatch.task.title.clone(),
            priority: dispatch.task.priority.clone(),
            status: TaskStatus::Pending,
            payload: dispatch.task.payload.clone(),
            created_at: created_at.into(),
            completed_at: None,
        }
    }

    /// Atualiza status da tarefa para InProgress.
    pub fn mark_in_progress(&mut self) {
        self.status = TaskStatus::InProgress;
    }

    /// Finaliza a tarefa com timestamp de conclusão.
    pub fn mark_completed(&mut self, timestamp: impl Into<String>) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(timestamp.into());
    }

    /// Marca a tarefa como com falha.
    pub fn mark_failed(&mut self, timestamp: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(timestamp.into());
    }
}
