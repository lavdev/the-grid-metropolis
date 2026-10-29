use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Component, Path as FsPath, PathBuf};
use tracing::error;

use crate::db::Database;
use crate::db::models::{ChatMessage, ProjectTerminal};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

#[derive(Debug, Deserialize)]
pub struct UpdateThreadRequest {
    pub thread_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadProjectFileQuery {
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileResponse {
    pub path: String,
    pub name: String,
    pub language: String,
    pub content: String,
    pub lines_count: usize,
    pub last_modified: String,
}

fn file_language(path: &FsPath) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "json" | "jsonc" => "json",
        "py" => "python",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "bash",
        "html" | "htm" | "xml" | "svg" => "html",
        "css" | "scss" | "less" => "css",
        "md" | "markdown" => "markdown",
        _ => "text",
    }
}

fn read_project_file(
    db: Database,
    project_id: String,
    requested_path: String,
) -> Result<ProjectFileResponse, (StatusCode, Json<serde_json::Value>)> {
    const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

    if requested_path.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "path é obrigatório"})),
        ));
    }

    let relative = PathBuf::from(&requested_path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "caminho de arquivo inválido"})),
        ));
    }

    let root_string = db
        .project_root(&project_id)
        .map_err(|error| {
            error!("Erro ao buscar raiz do projeto {}: {}", project_id, error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "falha ao buscar projeto"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "projeto não encontrado"})),
            )
        })?;

    let root = FsPath::new(&root_string).canonicalize().map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "workspace do projeto não encontrado"})),
        )
    })?;
    let candidate = root.join(&relative).canonicalize().map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "artefato não encontrado"})),
        )
    })?;

    if !candidate.starts_with(&root) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "caminho de arquivo inválido"})),
        ));
    }

    let metadata = std::fs::metadata(&candidate).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "artefato não encontrado"})),
        )
    })?;
    if !metadata.is_file() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "artefato não encontrado"})),
        ));
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({"error": "artefato excede o limite de 2 MiB"})),
        ));
    }

    let content = std::fs::read_to_string(&candidate).map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": "artefato não é texto UTF-8"})),
        )
    })?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|| "—".into());

    Ok(ProjectFileResponse {
        path: requested_path,
        name: candidate
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .into(),
        language: file_language(&candidate).into(),
        lines_count: content.lines().count().max(1),
        content,
        last_modified: modified,
    })
}

/// GET /api/v1/projects/:id/files?path=src/main.rs
pub async fn read_project_file_handler(
    State(db): State<Database>,
    Path(project_id): Path<String>,
    Query(query): Query<ReadProjectFileQuery>,
) -> Result<Json<ProjectFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || read_project_file(db, project_id, query.path))
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": error.to_string()})),
            )
        })?
        .map(Json)
}

/// GET /api/v1/projects
pub async fn list_projects_handler(
    State(db): State<Database>,
) -> Result<Json<Vec<ProjectTerminal>>, (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.list_projects().map(Json).map_err(|e| {
            error!("Erro ao listar projetos do SQLite: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
        })
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?
}

/// POST /api/v1/projects
pub async fn save_project_handler(
    State(db): State<Database>,
    Json(project): Json<ProjectTerminal>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.save_project(&project)
            .map(|_| {
                (
                    StatusCode::OK,
                    Json(json!({"status": "saved", "id": project.id})),
                )
            })
            .map_err(|e| {
                error!("Erro ao salvar projeto no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
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
            .map(|_| {
                (
                    StatusCode::CREATED,
                    Json(json!({"status": "created", "id": message.id})),
                )
            })
            .map_err(|e| {
                error!("Erro ao adicionar mensagem no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
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
            .map(|_| {
                (
                    StatusCode::OK,
                    Json(json!({"status": "updated", "projectId": project_id})),
                )
            })
            .map_err(|e| {
                error!("Erro ao atualizar thread_id do projeto no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?
}

/// DELETE /api/v1/projects/:id/messages/:msg_id
pub async fn delete_project_message_handler(
    State(db): State<Database>,
    Path((project_id, msg_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.delete_project_message(&project_id, &msg_id)
            .map(|_| {
                (
                    StatusCode::OK,
                    Json(json!({"status": "deleted", "id": msg_id})),
                )
            })
            .map_err(|e| {
                error!("Erro ao deletar mensagem no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?
}

/// DELETE /api/v1/projects/:id/messages
pub async fn clear_project_messages_handler(
    State(db): State<Database>,
    Path(project_id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        db.clear_project_messages(&project_id)
            .map(|_| {
                (
                    StatusCode::OK,
                    Json(json!({"status": "cleared", "projectId": project_id})),
                )
            })
            .map_err(|e| {
                error!("Erro ao limpar mensagens do projeto no SQLite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
            })
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?
}
