pub mod auth;
pub mod dispatch;
pub mod error;
pub mod ws;

use axum::{
    routing::{get, post},
    Router,
};

pub use auth::RequireAuth;
pub use dispatch::dispatch_handler;
pub use error::ApiError;
pub use ws::ws_handler;

use crate::state::SharedState;

/// Cria o sub-roteador de rotas da API REST e WebSocket da ponte.
pub fn api_routes(state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/bridge/dispatch", post(dispatch_handler))
        .route("/ws", get(ws_handler))
        .with_state(state)
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
    use tokio_tungstenite::connect_async;
    use tower::ServiceExt;

    use crate::domain::{DispatchPayload, DispatchTaskInput, TaskPayload, TaskPriority};
    use crate::state::create_shared_state;

    #[tokio::test]
    async fn test_dispatch_unauthorized_when_missing_header() {
        let state = create_shared_state();
        let app = api_routes(state);

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
        let app = api_routes(state);

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
        let app = api_routes(state);

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
        let app = api_routes(state);

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
    async fn test_websocket_streaming_and_snapshot_lifecycle() {
        let state = create_shared_state();
        let app = api_routes(state.clone());

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
