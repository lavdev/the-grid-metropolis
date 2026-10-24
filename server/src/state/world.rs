use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

use crate::domain::{
    Agent, AgentLocation, AgentRole, DispatchPayload, Pod, SpriteConfig, Task, WebSocketEvent,
};
use crate::state::error::StateError;

/// Capacidade padrão do buffer circular de broadcast do WebSocket.
pub const DEFAULT_BROADCAST_CAPACITY: usize = 1024;

/// Estado em memória consolidado do universo de simulação The Grid Metropolis.
pub struct WorldState {
    agents: HashMap<String, Agent>,
    pods: HashMap<String, Pod>,
    tasks: HashMap<String, Task>,
    event_tx: broadcast::Sender<WebSocketEvent>,
}

/// Tipo encapsulado para compartilhamento thread-safe do estado entre handlers do Axum.
pub type SharedState = Arc<RwLock<WorldState>>;

impl WorldState {
    /// Inicializa uma nova instância do WorldState com capacidade definida para o canal de eventos.
    pub fn new(broadcast_capacity: usize) -> Self {
        let (event_tx, _) = broadcast::channel(broadcast_capacity);
        Self {
            agents: HashMap::new(),
            pods: HashMap::new(),
            tasks: HashMap::new(),
            event_tx,
        }
    }

    /// Cria uma nova assinatura no canal de eventos em tempo real.
    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketEvent> {
        self.event_tx.subscribe()
    }

    /// Publica um evento WebSocket para todos os clientes conectados.
    pub fn broadcast(&self, event: WebSocketEvent) {
        if let Err(e) = self.event_tx.send(event) {
            // Em caso de zero clientes conectados, o broadcast retorna SendError normal
            warn!("Nenhum consumidor ativo no canal de broadcast WebSocket: {}", e);
        }
    }

    /// Registra um novo Agente na cidade e emite o evento AGENT_SPAWN.
    pub fn register_agent(&mut self, agent: Agent) -> Result<(), StateError> {
        if self.agents.contains_key(&agent.id) {
            return Err(StateError::DuplicateAgent(agent.id));
        }

        let spawn_event = WebSocketEvent::agent_spawn(
            &agent.id,
            &format!("{:?}", agent.role).to_uppercase(),
            &agent.home_pod_id,
        );

        let agent_id = agent.id.clone();
        self.agents.insert(agent_id, agent);
        self.broadcast(spawn_event);
        Ok(())
    }

    /// Registra uma nova estação/Pod de projeto na cidade.
    pub fn register_pod(&mut self, pod: Pod) -> Result<(), StateError> {
        if self.pods.contains_key(&pod.id) {
            return Err(StateError::DuplicatePod(pod.id));
        }
        self.pods.insert(pod.id.clone(), pod);
        Ok(())
    }

    /// Despacha uma nova tarefa externa (Bridge Dispatch):
    /// 1. Valida existência do agente e do pod de destino.
    /// 2. Cria a entidade Task.
    /// 3. Atualiza o status do Agente para WALKING.
    /// 4. Atualiza o estado visual do Pod para execução intensa.
    /// 5. Dispara evento TASK_ASSIGNED para o frontend.
    pub fn dispatch_task(&mut self, payload: DispatchPayload) -> Result<Task, StateError> {
        // Validação de existência do agente
        let agent = self
            .agents
            .get_mut(&payload.target_agent_id)
            .ok_or_else(|| StateError::AgentNotFound(payload.target_agent_id.clone()))?;

        // Validação de existência do pod do projeto
        let pod = self
            .pods
            .get_mut(&payload.project_id)
            .ok_or_else(|| StateError::PodNotFound(payload.project_id.clone()))?;

        if self.tasks.contains_key(&payload.task.task_id) {
            return Err(StateError::DuplicateTask(payload.task.task_id.clone()));
        }

        let now = chrono_timestamp_now();
        let task = Task::from_dispatch(&payload, now);
        let task_id = task.task_id.clone();

        // Transição de estado dos atores
        let source_pod = agent.home_pod_id.clone();
        agent.assign_task(&task_id);
        pod.set_executing();

        // Broadcast visual
        let event = WebSocketEvent::task_assigned(
            &payload.target_agent_id,
            &source_pod,
            &payload.project_id,
            &task_id,
        );
        self.broadcast(event);

        self.tasks.insert(task_id, task.clone());
        info!(
            "Tarefa '{}' despachada com sucesso para o agente '{}'",
            task.task_id, payload.target_agent_id
        );

        Ok(task)
    }

    /// Notifica progresso em uma tarefa, atualizando métricas e disparando TASK_PROGRESS.
    pub fn progress_task(
        &mut self,
        task_id: &str,
        step: &str,
        tokens_used: u64,
    ) -> Result<(), StateError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| StateError::TaskNotFound(task_id.to_string()))?;

        task.mark_in_progress();

        let event = WebSocketEvent::task_progress(&task.target_agent_id, step, tokens_used);
        self.broadcast(event);
        Ok(())
    }

    /// Conclui uma tarefa com sucesso, alterando fumaça do Pod e retornando agente para IDLE.
    pub fn complete_task(
        &mut self,
        task_id: &str,
        tokens_used: u64,
        artifacts: Vec<String>,
    ) -> Result<(), StateError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| StateError::TaskNotFound(task_id.to_string()))?;

        let now = chrono_timestamp_now();
        task.mark_completed(now);

        let target_agent_id = task.target_agent_id.clone();
        let project_id = task.project_id.clone();

        if let Some(agent) = self.agents.get_mut(&target_agent_id) {
            agent.complete_task(tokens_used);
        }

        if let Some(pod) = self.pods.get_mut(&project_id) {
            pod.set_completed();
        }

        let event = WebSocketEvent::task_completed(&target_agent_id, task_id, artifacts);
        self.broadcast(event);
        Ok(())
    }

    /// Marca uma tarefa como com falha e move o agente para estado de erro.
    pub fn fail_task(
        &mut self,
        task_id: &str,
        error_code: &str,
        details: &str,
    ) -> Result<(), StateError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| StateError::TaskNotFound(task_id.to_string()))?;

        let now = chrono_timestamp_now();
        task.mark_failed(now);

        let target_agent_id = task.target_agent_id.clone();
        let project_id = task.project_id.clone();

        if let Some(agent) = self.agents.get_mut(&target_agent_id) {
            agent.mark_error();
        }

        if let Some(pod) = self.pods.get_mut(&project_id) {
            pod.set_alert();
        }

        let event = WebSocketEvent::task_error(&target_agent_id, error_code, details);
        self.broadcast(event);
        Ok(())
    }

    // --- Métodos de Consulta (Query API) ---

    pub fn get_agent(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    pub fn get_pod(&self, id: &str) -> Option<&Pod> {
        self.pods.get(id)
    }

    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub fn list_agents(&self) -> Vec<Agent> {
        self.agents.values().cloned().collect()
    }

    pub fn list_pods(&self) -> Vec<Pod> {
        self.pods.values().cloned().collect()
    }

    pub fn list_tasks(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    /// Inicializa a metrópole com dados semente padrão (distritos e agentes padrão da especificação).
    pub fn seed_default_world(&mut self) {
        // Pods padrão
        let pod_nexus = Pod::new("pod-nexus", "The Nexus Central Hub", 10, None);
        let pod_notex = Pod::new(
            "proj-notex-backend",
            "Notex Core Services",
            3,
            Some("https://github.com/the-grid/notex".into()),
        );
        let pod_lounge = Pod::new("pod-battery-lounge", "The Battery Lounge", 20, None);
        let pod_glitch = Pod::new("pod-glitch-alleys", "The Glitch Alleys Quarantine", 5, None);

        let _ = self.register_pod(pod_nexus);
        let _ = self.register_pod(pod_notex);
        let _ = self.register_pod(pod_lounge);
        let _ = self.register_pod(pod_glitch);

        // Agente Rust padrão citado na documentação
        let agent_rust = Agent::new(
            "agent-rust-dev",
            "Rusty",
            AgentRole::Developer,
            SpriteConfig {
                texture_key: "hacker_neon_blue".into(),
                skin_theme: "street_hacker".into(),
                color_accent: "#00f0ff".into(),
            },
            "proj-notex-backend",
            AgentLocation {
                x: 10.0,
                y: 10.0,
                current_zone: "LOUNGE".into(),
            },
        );

        let _ = self.register_agent(agent_rust);
    }
}

/// Helper seguro para geração de timestamps ISO8601 sem pânico.
fn chrono_timestamp_now() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}

/// Cria um novo SharedState encapsulado para injeção de dependência na aplicação.
pub fn create_shared_state() -> SharedState {
    let mut world = WorldState::new(DEFAULT_BROADCAST_CAPACITY);
    world.seed_default_world();
    Arc::new(RwLock::new(world))
}
