use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tracing::error;

use crate::db::models::{ChatMessage, ProjectTerminal};
use crate::db::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

#[derive(Debug, Deserialize)]
pub struct UpdateThreadRequest {
    pub thread_id: Option<String>,
}

/// GET /api/v1/projects
pub async fn list_projects_handler(
    State(db): State<Database>,
) -> Result<Json<Vec<ProjectTerminal>>, (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.list_projects()
            .map(Json)
            .map_err(|e| {
                error!("Erro ao listar projetos do SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?
}

/// POST /api/v1/projects
pub async fn save_project_handler(
    State(db): State<Database>,
    Json(project): Json<ProjectTerminal>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.save_project(&project)
            .map(|_| (StatusCode::OK, Json(json!({"status": "saved", "id": project.id}))))
            .map_err(|e| {
                error!("Erro ao salvar projeto no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?
}

/// DELETE /api/v1/projects/:id
pub async fn delete_project_handler(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.delete_project(&id)
            .map(|_| (StatusCode::OK, Json(json!({"status": "deleted", "id": id}))))
            .map_err(|e| {
                error!("Erro ao deletar projeto do SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?
}

/// POST /api/v1/projects/:id/messages
pub async fn add_project_message_handler(
    State(db): State<Database>,
    Path(project_id): Path<String>,
    Json(message): Json<ChatMessage>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.add_message(&project_id, &message)
            .map(|_| (StatusCode::CREATED, Json(json!({"status": "created", "id": message.id}))))
            .map_err(|e| {
                error!("Erro ao adicionar mensagem no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?
}

/// PUT /api/v1/projects/:id/thread
pub async fn update_project_thread_handler(
    State(db): State<Database>,
    Path(project_id): Path<String>,
    Json(payload): Json<UpdateThreadRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.update_project_thread_id(&project_id, payload.thread_id.as_deref())
            .map(|_| (StatusCode::OK, Json(json!({"status": "updated", "projectId": project_id}))))
            .map_err(|e| {
                error!("Erro ao atualizar thread_id do projeto no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?
}

/// DELETE /api/v1/projects/:id/messages/:msg_id
pub async fn delete_project_message_handler(
    State(db): State<Database>,
    Path((project_id, msg_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.delete_project_message(&project_id, &msg_id)
            .map(|_| (StatusCode::OK, Json(json!({"status": "deleted", "id": msg_id}))))
            .map_err(|e| {
                error!("Erro ao deletar mensagem no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?
}

/// DELETE /api/v1/projects/:id/messages
pub async fn clear_project_messages_handler(
    State(db): State<Database>,
    Path(project_id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.clear_project_messages(&project_id)
            .map(|_| (StatusCode::OK, Json(json!({"status": "cleared", "projectId": project_id}))))
            .map_err(|e| {
                error!("Erro ao limpar mensagens do projeto no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?
}
