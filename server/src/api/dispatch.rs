use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use tracing::info;

use crate::api::auth::RequireAuth;
use crate::api::error::ApiError;
use crate::domain::DispatchPayload;
use crate::state::SharedState;

/// Endpoint de Ingestão da Ponte (Bridge Dispatch).
/// Rota: POST /api/v1/bridge/dispatch
/// Autenticação: Bearer <TOKEN>
/// Retorno: 202 Accepted em caso de sucesso.
pub async fn dispatch_handler(
    _auth: RequireAuth,
    State(state): State<SharedState>,
    Json(payload): Json<DispatchPayload>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    // Validações básicas de integridade do payload
    if payload.project_id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "O campo 'projectId' não pode ser vazio".into(),
        ));
    }
    if payload.target_agent_id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "O campo 'targetAgentId' não pode ser vazio".into(),
        ));
    }
    if payload.task.task_id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "O campo 'task.taskId' não pode ser vazio".into(),
        ));
    }
    if payload.task.title.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "O campo 'task.title' não pode ser vazio".into(),
        ));
    }

    info!(
        source = %payload.source,
        project = %payload.project_id,
        agent = %payload.target_agent_id,
        task = %payload.task.task_id,
        "Recebida nova requisição de despacho externo na ponte"
    );

    let mut world = state.write().await;
    let task = world.dispatch_task(payload)?;

    let response_body = json!({
        "status": "ACCEPTED",
        "message": "Tarefa enfileirada e despachada para a simulação com sucesso",
        "taskId": task.task_id,
        "targetAgentId": task.target_agent_id,
        "projectId": task.project_id,
        "taskStatus": task.status
    });

    Ok((StatusCode::ACCEPTED, Json(response_body)))
}
