use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tópicos de eventos transmitidos em tempo real pelo WebSocket Hub para o Canvas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventTopic {
    AgentSpawn,
    TaskAssigned,
    TaskProgress,
    AgentCollaborate,
    CollaborationRequest,
    TaskCompleted,
    TaskError,
    AgentRest,
}

/// Envelope de mensagem WebSocket padronizado para o cliente frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketEvent {
    pub topic: EventTopic,
    pub timestamp: i64,
    pub payload: Value,
}

impl WebSocketEvent {
    /// Cria um novo evento com timestamp Unix atual (em milissegundos).
    pub fn new(topic: EventTopic, payload: Value) -> Self {
        let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_millis() as i64,
            Err(_) => 0,
        };
        Self {
            topic,
            timestamp,
            payload,
        }
    }

    /// Cria um evento de surgimento de agente (AGENT_SPAWN).
    pub fn agent_spawn(agent_id: &str, role: &str, start_pod_id: &str) -> Self {
        Self::new(
            EventTopic::AgentSpawn,
            serde_json::json!({
                "agentId": agent_id,
                "role": role,
                "startPodId": start_pod_id
            }),
        )
    }

    /// Cria um evento de atribuição e início de deslocamento (TASK_ASSIGNED).
    pub fn task_assigned(
        agent_id: &str,
        source_pod: &str,
        target_pod: &str,
        task_id: &str,
    ) -> Self {
        Self::new(
            EventTopic::TaskAssigned,
            serde_json::json!({
                "agentId": agent_id,
                "sourcePod": source_pod,
                "targetPod": target_pod,
                "taskId": task_id
            }),
        )
    }

    /// Cria um evento de progresso na execução da tarefa (TASK_PROGRESS).
    pub fn task_progress(agent_id: &str, step: &str, tokens_used: u64) -> Self {
        Self::new(
            EventTopic::TaskProgress,
            serde_json::json!({
                "agentId": agent_id,
                "step": step,
                "tokensUsed": tokens_used
            }),
        )
    }

    /// Cria um evento de conclusão bem-sucedida (TASK_COMPLETED).
    pub fn task_completed(agent_id: &str, task_id: &str, artifacts: Vec<String>) -> Self {
        Self::new(
            EventTopic::TaskCompleted,
            serde_json::json!({
                "agentId": agent_id,
                "taskId": task_id,
                "artifacts": artifacts
            }),
        )
    }

    /// Cria um evento de falha ou exceção (TASK_ERROR).
    pub fn task_error(agent_id: &str, error_code: &str, details: &str) -> Self {
        Self::new(
            EventTopic::TaskError,
            serde_json::json!({
                "agentId": agent_id,
                "errorCode": error_code,
                "details": details
            }),
        )
    }

    /// Cria um evento de retorno ao descanso (AGENT_REST).
    pub fn agent_rest(agent_id: &str) -> Self {
        Self::new(
            EventTopic::AgentRest,
            serde_json::json!({
                "agentId": agent_id
            }),
        )
    }
}
