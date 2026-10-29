use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

use crate::state::StateError;

/// Erros da camada de transporte HTTP (API).
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Não autorizado: {0}")]
    Unauthorized(String),

    #[error("Requisição inválida: {0}")]
    BadRequest(String),

    #[error("Recurso não encontrado: {0}")]
    NotFound(String),

    #[error("Conflito de estado: {0}")]
    Conflict(String),

    #[error("Erro interno do servidor")]
    Internal,
}

impl From<StateError> for ApiError {
    fn from(err: StateError) -> Self {
        match err {
            StateError::AgentNotFound(msg) => {
                ApiError::NotFound(format!("Agente não encontrado: {}", msg))
            }
            StateError::PodNotFound(msg) => {
                ApiError::NotFound(format!("Pod não encontrado: {}", msg))
            }
            StateError::TaskNotFound(msg) => {
                ApiError::NotFound(format!("Tarefa não encontrada: {}", msg))
            }
            StateError::DuplicateAgent(msg)
            | StateError::DuplicatePod(msg)
            | StateError::DuplicateTask(msg) => {
                ApiError::Conflict(format!("Recurso duplicado: {}", msg))
            }
            StateError::PodCapacityExceeded(msg) => {
                ApiError::BadRequest(format!("Capacidade excedida: {}", msg))
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Ocorreu um erro interno no servidor".to_string(),
            ),
        };

        let body = Json(json!({
            "error": code,
            "message": message
        }));

        (status, body).into_response()
    }
}
