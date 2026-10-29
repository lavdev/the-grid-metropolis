pub mod auth;
pub mod codex;
pub mod dispatch;
pub mod error;
pub mod execution_settings;
pub mod projects;
pub mod run_error;
pub mod ws;

use axum::{
    Extension, Router,
    routing::{delete, get, post, put},
};

pub use auth::{RequireAuth, RequireRunAuth};
pub use codex::{codex_status_handler, codex_stream_handler};
pub use dispatch::dispatch_handler;
pub use error::ApiError;
pub use execution_settings::{get_execution_settings, put_execution_settings};
pub use projects::{
    add_project_message_handler, clear_project_messages_handler, delete_project_handler,
    delete_project_message_handler, list_projects_handler, read_project_file_handler,
    save_project_handler, update_project_thread_handler,
};
pub use ws::ws_handler;

use crate::db::Database;
use crate::state::SharedState;

/// Cria o sub-roteador de rotas da API REST e WebSocket da ponte.
pub fn api_routes(state: SharedState, db: Database) -> Router {
    api_routes_with_run_auth(state, db, std::env::var("BRIDGE_AUTH_TOKEN").ok())
}

fn api_routes_with_run_auth(
    state: SharedState,
    db: Database,
    run_auth_token: Option<String>,
) -> Router {
    let ws_and_bridge = Router::new()
        .route("/api/v1/bridge/dispatch", post(dispatch_handler))
        .route("/api/v1/codex/status", get(codex_status_handler))
        .route("/api/v1/codex/stream", post(codex_stream_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let project_routes = Router::new()
        .route(
            "/api/v1/projects",
            get(list_projects_handler).post(save_project_handler),
        )
        .route("/api/v1/projects/{id}", delete(delete_project_handler))
        .route(
            "/api/v1/projects/{id}/messages",
            post(add_project_message_handler).delete(clear_project_messages_handler),
        )
        .route(
            "/api/v1/projects/{id}/messages/{msg_id}",
            delete(delete_project_message_handler),
        )
        .route(
            "/api/v1/projects/{id}/thread",
            put(update_project_thread_handler),
        )
        .route(
            "/api/v1/projects/{id}/files",
            get(read_project_file_handler),
        )
        .with_state(db.clone());

    let execution_settings_routes = Router::new()
        .route(
            "/api/v1/projects/{id}/execution-settings",
            get(get_execution_settings).put(put_execution_settings),
        )
        .with_state(execution_settings::ExecutionSettingsApiState { db })
        .layer(Extension(auth::RunAuthConfig(run_auth_token)));

    ws_and_bridge
        .merge(project_routes)
        .merge(execution_settings_routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::AUTHORIZATION},
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
        fs::write(
            root.join("src/main.rs"),
            "fn main() { println!(\"ok\"); }\n",
        )
        .unwrap();

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

        let save = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/projects")
                    .header("Content-Type", "application/json")
                    .body(Body::from(project.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save.status(), StatusCode::OK);

        let read = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/proj-artifact-test/files?path=src/main.rs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(read.status(), StatusCode::OK);
        let payload: serde_json::Value =
            serde_json::from_slice(&read.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(payload["language"], "rust");
        assert_eq!(payload["content"], "fn main() { println!(\"ok\"); }\n");

        let traversal = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/proj-artifact-test/files?path=../secret.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
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
        let (mut ws_stream, _) = connect_async(ws_url)
            .await
            .expect("Falha ao conectar no WS");

        // 1. O primeiro evento deve ser o WORLD_SNAPSHOT inicial
        let first_msg = ws_stream.next().await.unwrap().unwrap();
        let snapshot_json: serde_json::Value =
            serde_json::from_str(&first_msg.to_string()).unwrap();
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

    #[tokio::test]
    async fn project_execution_settings_are_authenticated_and_persisted() {
        let app = api_routes_with_run_auth(
            create_shared_state(),
            Database::init(":memory:").unwrap(),
            Some("run-test-token".into()),
        );

        let unauthorized = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/proj-notex/execution-settings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let initial = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/proj-notex/execution-settings")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(initial.status(), StatusCode::OK);
        let initial_body: serde_json::Value =
            serde_json::from_slice(&initial.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(
            initial_body,
            serde_json::json!({"configured": false, "budgets": null})
        );

        let saved = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/projects/proj-notex/execution-settings")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"budgets":{"maxModelTurnsPerRun":18,"maxElapsedSecondsPerRun":900,"maxTokensPerRun":120000}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(saved.status(), StatusCode::OK);
        let saved_body: serde_json::Value =
            serde_json::from_slice(&saved.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(saved_body["configured"], true);
        assert_eq!(saved_body["budgets"]["maxTokensPerRun"], 120000);

        let loaded = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/proj-notex/execution-settings")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let loaded_body: serde_json::Value =
            serde_json::from_slice(&loaded.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(loaded_body, saved_body);
    }

    #[tokio::test]
    async fn project_execution_settings_fail_closed_and_validate_requests() {
        let unconfigured = api_routes_with_run_auth(
            create_shared_state(),
            Database::init(":memory:").unwrap(),
            None,
        );
        let unavailable = unconfigured
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/proj-notex/execution-settings")
                    .header(AUTHORIZATION, "Bearer anything")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);

        let app = api_routes_with_run_auth(
            create_shared_state(),
            Database::init(":memory:").unwrap(),
            Some("run-test-token".into()),
        );
        let invalid = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/projects/proj-notex/execution-settings")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"budgets":{"maxModelTurnsPerRun":1,"maxElapsedSecondsPerRun":30,"maxTokensPerRun":0}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let invalid_body: serde_json::Value =
            serde_json::from_slice(&invalid.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(invalid_body["error"]["code"], "INVALID_BUDGET");
        assert!(invalid_body["error"]["requestId"].is_string());

        let malformed = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/projects/proj-notex/execution-settings")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(malformed.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
