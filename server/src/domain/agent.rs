use serde::{Deserialize, Serialize};

/// Especialidades e papéis funcionais dos agentes na simulação.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentRole {
    Developer,
    Researcher,
    Devops,
    Security,
    BotRunner,
}

/// Estados operacionais da máquina de estados do agente.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentStatus {
    Idle,
    Walking,
    Working,
    Meeting,
    Error,
}

/// Configuração visual do sprite para renderização isométrica.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpriteConfig {
    pub texture_key: String,
    pub skin_theme: String,
    pub color_accent: String,
}

/// Coordenadas espaciais e zona geográfica no grid isométrico.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentLocation {
    pub x: f64,
    pub y: f64,
    pub current_zone: String,
}

/// Métricas de execução e consumo acumuladas pelo agente.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentMetrics {
    pub tasks_completed: u64,
    pub tokens_consumed: u64,
    pub uptime_seconds: u64,
}

/// Entidade de Domínio representando um Agente de IA na metrópole.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub sprite_config: SpriteConfig,
    pub home_pod_id: String,
    pub current_location: AgentLocation,
    pub status: AgentStatus,
    pub active_task_id: Option<String>,
    pub metrics: AgentMetrics,
}

impl Agent {
    /// Construtor de um novo Agente com estado inicial IDLE.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        role: AgentRole,
        sprite_config: SpriteConfig,
        home_pod_id: impl Into<String>,
        location: AgentLocation,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role,
            sprite_config,
            home_pod_id: home_pod_id.into(),
            current_location: location,
            status: AgentStatus::Idle,
            active_task_id: None,
            metrics: AgentMetrics::default(),
        }
    }

    /// Atribui uma tarefa ao agente e altera seu status para WALKING ou WORKING.
    pub fn assign_task(&mut self, task_id: impl Into<String>) {
        self.active_task_id = Some(task_id.into());
        self.status = AgentStatus::Walking;
    }

    /// Finaliza a tarefa ativa e incrementa métricas.
    pub fn complete_task(&mut self, tokens_used: u64) {
        self.active_task_id = None;
        self.status = AgentStatus::Idle;
        self.metrics.tasks_completed += 1;
        self.metrics.tokens_consumed += tokens_used;
    }

    /// Marca o agente em estado de falha/exceção.
    pub fn mark_error(&mut self) {
        self.status = AgentStatus::Error;
    }
}
