pub mod auth;
pub mod codex;
pub mod dispatch;
pub mod error;
pub mod projects;
pub mod ws;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub use auth::RequireAuth;
pub use codex::{codex_status_handler, codex_stream_handler};
pub use dispatch::dispatch_handler;
pub use error::ApiError;
pub use projects::{
    add_project_message_handler, clear_project_messages_handler, delete_project_handler,
    delete_project_message_handler, list_projects_handler, read_project_file_handler, save_project_handler,
    update_project_thread_handler,
};
pub use ws::ws_handler;

use crate::db::Database;
use crate::state::SharedState;

/// Cria o sub-roteador de rotas da API REST e WebSocket da ponte.
pub fn api_routes(state: SharedState, db: Database) -> Router {
    let ws_and_bridge = Router::new()
        .route("/api/v1/bridge/dispatch", post(dispatch_handler))
        .route("/api/v1/codex/status", get(codex_status_handler))
        .route("/api/v1/codex/stream", post(codex_stream_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let project_routes = Router::new()
        .route("/api/v1/projects", get(list_projects_handler).post(save_project_handler))
        .route("/api/v1/projects/{id}", delete(delete_project_handler))
        .route(
            "/api/v1/projects/{id}/messages",
            post(add_project_message_handler).delete(clear_project_messages_handler),
        )
        .route(
            "/api/v1/projects/{id}/messages/{msg_id}",
            delete(delete_project_message_handler),
        )
        .route("/api/v1/projects/{id}/thread", put(update_project_thread_handler))
        .route("/api/v1/projects/{id}/files", get(read_project_file_handler))
        .with_state(db);

    ws_and_bridge.merge(project_routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header::AUTHORIZATION, Request, StatusCode},
    };
    use futures_util::StreamExt;
    use http_body_util::BodyExt;
    use std::fs;
    use tokio_tungstenite::connect_async;
    use tower::ServiceExt;

    use crate::domain::{DispatchPayload, DispatchTaskInput, TaskPayload, TaskPriority};
    use crate::state::create_shared_state;

    #[tokio::test]
    async fn test_dispatch_unauthorized_when_missing_header() {
        let state = create_shared_state();
        let db = Database::init(":memory:").unwrap();
        let app = api_routes(state, db);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bridge/dispatch")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_dispatch_unauthorized_when_invalid_token() {
        let state = create_shared_state();
        let db = Database::init(":memory:").unwrap();
        let app = api_routes(state, db);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bridge/dispatch")
            .header(AUTHORIZATION, "Bearer token-invalido")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_dispatch_accepted_when_valid_contract() {
        let state = create_shared_state();
        let db = Database::init(":memory:").unwrap();
        let app = api_routes(state, db);

        let valid_payload = r#"{
            "source": "codex-cloud-runner",
            "projectId": "proj-notex-backend",
            "targetAgentId": "agent-rust-dev",
            "task": {
                "taskId": "task-8841",
                "title": "Implementar driver SQLite OPFS",
                "priority": "HIGH",
                "payload": {
                    "prompt": "Escreva a camada de persistência...",
                    "contextFiles": ["src/db/schema.rs"]
                }
            },
            "callbackWebhook": "https://cloud.engine.io/tasks/task-8841/callback"
        }"#;

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bridge/dispatch")
            .header(AUTHORIZATION, "Bearer secret-bridge-token")
            .header("Content-Type", "application/json")
            .body(Body::from(valid_payload))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["status"], "ACCEPTED");
        assert_eq!(json["taskId"], "task-8841");
        assert_eq!(json["targetAgentId"], "agent-rust-dev");
    }

    #[tokio::test]
    async fn test_dispatch_not_found_when_agent_does_not_exist() {
        let state = create_shared_state();
        let db = Database::init(":memory:").unwrap();
        let app = api_routes(state, db);

        let payload_nonexistent_agent = r#"{
            "source": "codex",
            "projectId": "proj-notex-backend",
            "targetAgentId": "agent-fantasma",
            "task": {
                "taskId": "task-9999",
                "title": "Tarefa fantasma",
                "priority": "LOW",
                "payload": { "prompt": "teste" }
            }
        }"#;

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bridge/dispatch")
            .header(AUTHORIZATION, "Bearer secret-bridge-token")
            .header("Content-Type", "application/json")
            .body(Body::from(payload_nonexistent_agent))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_project_file_read_and_traversal_protection() {
        let root = std::env::temp_dir().join(format!(
            "grid-metropolis-artifact-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() { println!(\"ok\"); }\n").unwrap();

        let state = create_shared_state();
        let db = Database::init(":memory:").unwrap();
        let app = api_routes(state, db);
        let project = serde_json::json!({
            "id": "proj-artifact-test",
            "name": "Artifact Test",
            "path": root.to_string_lossy(),
            "model": "test",
            "status": "ACTIVE",
            "accentColor": "#00f0ff",
            "metrics": { "tokensIn": 0, "tokensOut": 0, "latencyMs": 0, "cpu": "", "mem": "" },
            "droids": [],
            "activeDroidId": "test",
            "messages": [],
            "pipeline": { "isActive": false, "stage": "IDLE", "currentDroidId": "test", "prompt": "", "progressPercent": 0, "statusText": "" }
        });

        let save = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Content-Type", "application/json")
                .body(Body::from(project.to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(save.status(), StatusCode::OK);

        let read = app.clone().oneshot(
            Request::builder()
                .uri("/api/v1/projects/proj-artifact-test/files?path=src/main.rs")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(read.status(), StatusCode::OK);
        let payload: serde_json::Value = serde_json::from_slice(
            &read.into_body().collect().await.unwrap().to_bytes(),
        ).unwrap();
        assert_eq!(payload["language"], "rust");
        assert_eq!(payload["content"], "fn main() { println!(\"ok\"); }\n");

        let traversal = app.oneshot(
            Request::builder()
                .uri("/api/v1/projects/proj-artifact-test/files?path=../secret.txt")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(traversal.status(), StatusCode::BAD_REQUEST);
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn test_websocket_streaming_and_snapshot_lifecycle() {
        let state = create_shared_state();
        let db = Database::init(":memory:").unwrap();
        let app = api_routes(state.clone(), db);

        // Inicia servidor TCP efêmero para testar upgrade real com tokio-tungstenite
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let ws_url = format!("ws://{}/ws", local_addr);
        let (mut ws_stream, _) = connect_async(ws_url).await.expect("Falha ao conectar no WS");

        // 1. O primeiro evento deve ser o WORLD_SNAPSHOT inicial
        let first_msg = ws_stream.next().await.unwrap().unwrap();
        let snapshot_json: serde_json::Value = serde_json::from_str(&first_msg.to_string()).unwrap();
        assert_eq!(snapshot_json["topic"], "WORLD_SNAPSHOT");
        assert!(snapshot_json["payload"]["agents"].as_array().unwrap().len() >= 1);
        assert!(snapshot_json["payload"]["pods"].as_array().unwrap().len() >= 1);

        // 2. Dispara uma tarefa via state para validar o envio em tempo real
        let dispatch = DispatchPayload {
            source: "ws-test".into(),
            project_id: "proj-notex-backend".into(),
            target_agent_id: "agent-rust-dev".into(),
            task: DispatchTaskInput {
                task_id: "task-ws-001".into(),
                title: "Test WS Streaming".into(),
                priority: TaskPriority::Critical,
                payload: TaskPayload::default(),
            },
            callback_webhook: None,
        };

        {
            let mut world = state.write().await;
            world.dispatch_task(dispatch).unwrap();
        }

        // 3. O cliente WS deve receber imediatamente o evento TASK_ASSIGNED
        let second_msg = ws_stream.next().await.unwrap().unwrap();
        let event_json: serde_json::Value = serde_json::from_str(&second_msg.to_string()).unwrap();
        assert_eq!(event_json["topic"], "TASK_ASSIGNED");
        assert_eq!(event_json["payload"]["agentId"], "agent-rust-dev");
        assert_eq!(event_json["payload"]["taskId"], "task-ws-001");

        server_handle.abort();
    }
}
