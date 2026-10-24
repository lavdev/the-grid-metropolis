use serde::{Deserialize, Serialize};

/// Estado exibido nos monitores e telas externas do Pod.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenStatus {
    Off,
    CodeStream,
    Dashboard,
    AlertRed,
}

/// Estado visual reativo do Pod no cenário isométrico.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VisualState {
    pub power_level: u8,
    pub chimney_smoke_color: String,
    pub screen_status: ScreenStatus,
}

impl Default for VisualState {
    fn default() -> Self {
        Self {
            power_level: 50,
            chimney_smoke_color: "#39ff14".to_string(), // Verde neon padrão
            screen_status: ScreenStatus::Dashboard,
        }
    }
}

/// Estação de trabalho interna dentro de um Pod.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InteriorStation {
    pub station_id: String,
    pub is_occupied: bool,
    pub coord_x: f64,
    pub coord_y: f64,
}

/// Entidade de Domínio representando um Project Pod (Casa / Laboratório de Projeto).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Pod {
    pub id: String,
    pub title: String,
    pub repository_url: Option<String>,
    pub max_capacity: usize,
    pub assigned_agents: Vec<String>,
    pub visual_state: VisualState,
    pub interior_stations: Vec<InteriorStation>,
}

impl Pod {
    /// Construtor de um novo Pod de projeto.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        max_capacity: usize,
        repository_url: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            repository_url,
            max_capacity,
            assigned_agents: Vec::new(),
            visual_state: VisualState::default(),
            interior_stations: Vec::new(),
        }
    }

    /// Vincula um agente ao Pod caso a capacidade permita.
    pub fn assign_agent(&mut self, agent_id: impl Into<String>) -> Result<(), &'static str> {
        let aid = agent_id.into();
        if self.assigned_agents.contains(&aid) {
            return Ok(());
        }
        if self.assigned_agents.len() >= self.max_capacity {
            return Err("Pod atingiu a capacidade máxima de agentes");
        }
        self.assigned_agents.push(aid);
        Ok(())
    }

    /// Desvincula um agente do Pod.
    pub fn unassign_agent(&mut self, agent_id: &str) {
        self.assigned_agents.retain(|id| id != agent_id);
    }

    /// Atualiza o status visual do Pod para modo de execução intensa.
    pub fn set_executing(&mut self) {
        self.visual_state.power_level = 100;
        self.visual_state.screen_status = ScreenStatus::CodeStream;
        self.visual_state.chimney_smoke_color = "#00f0ff".to_string(); // Ciano
    }

    /// Atualiza o status visual do Pod para modo de sucesso/conclusão.
    pub fn set_completed(&mut self) {
        self.visual_state.power_level = 70;
        self.visual_state.screen_status = ScreenStatus::Dashboard;
        self.visual_state.chimney_smoke_color = "#39ff14".to_string(); // Verde neon
    }

    /// Atualiza o status visual do Pod para modo de alerta/erro.
    pub fn set_alert(&mut self) {
        self.visual_state.power_level = 100;
        self.visual_state.screen_status = ScreenStatus::AlertRed;
        self.visual_state.chimney_smoke_color = "#ff007f".to_string(); // Magenta / Vermelho
    }
}
