use thiserror::Error;

/// Erros tipados para operações sobre o estado em memória da simulação.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
    #[error("Agente com ID '{0}' não foi encontrado")]
    AgentNotFound(String),

    #[error("Pod com ID '{0}' não foi encontrado")]
    PodNotFound(String),

    #[error("Tarefa com ID '{0}' não foi encontrada")]
    TaskNotFound(String),

    #[error("Pod '{0}' atingiu a capacidade máxima de agentes")]
    PodCapacityExceeded(String),

    #[error("Agente com ID '{0}' já está cadastrado")]
    DuplicateAgent(String),

    #[error("Pod com ID '{0}' já está cadastrado")]
    DuplicatePod(String),

    #[error("Tarefa com ID '{0}' já está cadastrada")]
    DuplicateTask(String),
}
