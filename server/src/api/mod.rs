pub mod auth;
pub mod dispatch;
pub mod error;

use axum::{routing::post, Router};

pub use auth::RequireAuth;
pub use dispatch::dispatch_handler;
pub use error::ApiError;

use crate::state::SharedState;

/// Cria o sub-roteador de rotas da API REST da ponte.
pub fn api_routes(state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/bridge/dispatch", post(dispatch_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header::AUTHORIZATION, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

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
}
