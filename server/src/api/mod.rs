pub mod auth;
pub mod codex;
pub mod dispatch;
pub mod error;
pub mod execution_policy;
pub mod execution_settings;
pub mod projects;
pub mod run_error;
pub mod runs;
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
pub use runs::{
    RunReadApiState, create_project_run, decide_run_permission, get_run_events, get_run_snapshot,
};
pub use ws::ws_handler;

use crate::api::execution_policy::{
    get_droid_execution_policy, get_project_execution_policy, put_droid_execution_policy,
    put_project_execution_policy,
};
use crate::db::Database;
use crate::run::dispatcher::RunDispatcher;
use crate::run::worker::RunWorker;
use crate::state::SharedState;

/// Cria o sub-roteador de rotas da API REST e WebSocket da ponte.
pub fn api_routes(state: SharedState, db: Database) -> Router {
    api_routes_with_run_auth(state, db, std::env::var("BRIDGE_AUTH_TOKEN").ok())
}

pub fn api_routes_with_dispatcher(
    state: SharedState,
    db: Database,
    dispatcher: RunDispatcher,
) -> Router {
    let worker = RunWorker::new_with_dispatcher(db.clone(), dispatcher.clone());
    api_routes_with_worker(state, db, dispatcher, worker)
}

pub fn api_routes_with_worker(
    state: SharedState,
    db: Database,
    dispatcher: RunDispatcher,
    worker: RunWorker,
) -> Router {
    api_routes_with_run_auth_and_dispatcher(
        state,
        db,
        std::env::var("BRIDGE_AUTH_TOKEN").ok(),
        dispatcher,
        worker,
    )
}

fn api_routes_with_run_auth(
    state: SharedState,
    db: Database,
    run_auth_token: Option<String>,
) -> Router {
    let dispatcher = RunDispatcher::default();
    let worker = RunWorker::new_with_dispatcher(db.clone(), dispatcher.clone());
    api_routes_with_run_auth_and_dispatcher(state, db, run_auth_token, dispatcher, worker)
}

fn api_routes_with_run_auth_and_dispatcher(
    state: SharedState,
    db: Database,
    run_auth_token: Option<String>,
    dispatcher: RunDispatcher,
    worker: RunWorker,
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
        .with_state(execution_settings::ExecutionSettingsApiState { db: db.clone() })
        .layer(Extension(auth::RunAuthConfig(run_auth_token.clone())));

    let execution_policy_routes = Router::new()
        .route(
            "/api/v1/projects/{project_id}/execution-policy",
            get(get_project_execution_policy).put(put_project_execution_policy),
        )
        .route(
            "/api/v1/projects/{project_id}/droids/{droid_id}/execution-policy",
            get(get_droid_execution_policy).put(put_droid_execution_policy),
        )
        .with_state(execution_policy::ExecutionPolicyApiState { db: db.clone() })
        .layer(Extension(auth::RunAuthConfig(run_auth_token.clone())));

    let run_read_routes = Router::new()
        .route(
            "/api/v1/projects/{project_id}/runs",
            post(create_project_run),
        )
        .route("/api/v1/runs/{run_id}", get(get_run_snapshot))
        .route("/api/v1/runs/{run_id}/events", get(get_run_events))
        .route(
            "/api/v1/runs/{run_id}/permission-requests/{request_id}/decisions",
            post(decide_run_permission),
        )
        .with_state(runs::RunReadApiState::with_worker(
            db.clone(),
            dispatcher,
            worker,
        ))
        .layer(Extension(auth::RunAuthConfig(run_auth_token)));

    ws_and_bridge
        .merge(project_routes)
        .merge(execution_settings_routes)
        .merge(execution_policy_routes)
        .merge(run_read_routes)
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

    use crate::domain::run::{NewRunRecord, RunBudgetLimits, RunStatus};
    use crate::domain::{DispatchPayload, DispatchTaskInput, TaskPayload, TaskPriority};
    use crate::run::coordinator::RunCoordinator;
    use crate::run::dispatcher::RunDispatcher;
    use crate::run::worker::RunWorker;
    use crate::state::create_shared_state;

    #[tokio::test]
    async fn run_submission_fails_closed_when_worker_is_disabled() {
        let db = Database::init(":memory:").unwrap();
        let project = db.list_projects().unwrap().into_iter().next().unwrap();
        let dispatcher = RunDispatcher::default();
        let worker = RunWorker::with_enabled(db.clone(), false);
        let app = api_routes_with_run_auth_and_dispatcher(
            create_shared_state(),
            db.clone(),
            Some("run-test-token".into()),
            dispatcher,
            worker,
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/projects/{}/runs", project.id))
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Idempotency-Key", "run-disabled-01")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"targetDroidId":"agent-rust-dev","taskTitle":"No execution","prompt":"This must not be queued","verificationPresetId":"cargo-test"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(db.list_nonterminal_runs().unwrap().is_empty());
    }

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

    #[tokio::test]
    async fn execution_policy_api_enforces_project_ceiling_and_droid_inheritance() {
        let db = Database::init(":memory:").unwrap();
        let project = db.list_projects().unwrap().into_iter().next().unwrap();
        let droid_id = project.droids[0].id.clone();
        let app =
            api_routes_with_run_auth(create_shared_state(), db, Some("run-test-token".into()));

        let inherited = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/projects/{}/droids/{}/execution-policy",
                        project.id, droid_id
                    ))
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(inherited.status(), StatusCode::OK);
        let inherited_json: serde_json::Value =
            serde_json::from_slice(&inherited.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(inherited_json["configured"], false);
        assert_eq!(inherited_json["inheritsProjectPolicy"], true);

        let project_policy = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!(
                        "/api/v1/projects/{}/execution-policy",
                        project.id
                    ))
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"readProject":true,"writeRoots":["src","docs"],"referenceProjectIds":[],"webSearch":true,"shellNetwork":false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(project_policy.status(), StatusCode::OK);

        let invalid_override = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!(
                        "/api/v1/projects/{}/droids/{}/execution-policy",
                        project.id, droid_id
                    ))
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"shellNetwork":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_override.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let narrowed_override = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!(
                        "/api/v1/projects/{}/droids/{}/execution-policy",
                        project.id, droid_id
                    ))
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"writeRoots":["src/private"],"webSearch":false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(narrowed_override.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn run_snapshot_and_event_replay_are_authenticated_and_cursor_ordered() {
        let db = Database::init(":memory:").unwrap();
        let project = db.list_projects().unwrap().into_iter().next().unwrap();
        let budgets = RunBudgetLimits {
            max_model_turns_per_run: 5,
            max_elapsed_seconds_per_run: 300,
            max_tokens_per_run: 30_000,
        };
        db.save_project_execution_settings(&project.id, budgets)
            .unwrap();
        let run = NewRunRecord {
            run_id: "run-api-read-01".into(),
            project_id: project.id,
            droid_ids: vec![project.droids[0].id.clone()],
            idempotency_key: "api-read-01".into(),
            task_title: "Read durable run state".into(),
            prompt: "Inspect the saved run".into(),
            verification_preset_id: "cargo-test".into(),
            policy_snapshot_json: "{}".into(),
            budget_limits: budgets,
        };
        let coordinator = RunCoordinator::new(db.clone());
        coordinator.create(&run).unwrap();
        coordinator
            .transition(
                &run.run_id,
                RunStatus::Planning,
                None,
                "RUN_PLANNING",
                &serde_json::json!({"stage":"planning"}),
            )
            .unwrap();
        let app =
            api_routes_with_run_auth(create_shared_state(), db, Some("run-test-token".into()));

        let snapshot = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/runs/run-api-read-01")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(snapshot.status(), StatusCode::OK);
        let snapshot_json: serde_json::Value =
            serde_json::from_slice(&snapshot.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(snapshot_json["status"], "PLANNING");
        assert_eq!(snapshot_json["lastSequence"], 2);

        let first_page = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/runs/run-api-read-01/events?limit=1")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let first_json: serde_json::Value =
            serde_json::from_slice(&first_page.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(first_json["events"][0]["sequence"], 1);
        assert_eq!(first_json["nextSequence"], 1);
        assert_eq!(first_json["hasMore"], true);

        let second_page = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/runs/run-api-read-01/events?afterSequence=1&limit=2")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let second_json: serde_json::Value =
            serde_json::from_slice(&second_page.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(second_json["events"][0]["sequence"], 2);
        assert_eq!(second_json["hasMore"], false);

        let unauthenticated = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/runs/run-api-read-01")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn permission_decisions_are_authenticated_scoped_and_idempotent() {
        use crate::domain::run::{NewRunPermissionRequest, PermissionStatus};

        let db = Database::init(":memory:").unwrap();
        let project = db.list_projects().unwrap().into_iter().next().unwrap();
        let budgets = RunBudgetLimits {
            max_model_turns_per_run: 5,
            max_elapsed_seconds_per_run: 300,
            max_tokens_per_run: 30_000,
        };
        db.save_project_execution_settings(&project.id, budgets)
            .unwrap();
        let run = NewRunRecord {
            run_id: "run-permission-api-01".into(),
            project_id: project.id,
            droid_ids: vec![project.droids[0].id.clone()],
            idempotency_key: "permission-api-01".into(),
            task_title: "Decide a permission request".into(),
            prompt: "Wait for a task-scoped grant".into(),
            verification_preset_id: "cargo-test".into(),
            policy_snapshot_json: "{}".into(),
            budget_limits: budgets,
        };
        let coordinator = RunCoordinator::new(db.clone());
        coordinator.create(&run).unwrap();
        coordinator
            .transition(
                &run.run_id,
                RunStatus::Planning,
                None,
                "RUN_PLANNING",
                &serde_json::json!({}),
            )
            .unwrap();
        coordinator
            .transition(
                &run.run_id,
                RunStatus::Executing,
                None,
                "RUN_EXECUTING",
                &serde_json::json!({}),
            )
            .unwrap();
        coordinator
            .request_permission(&NewRunPermissionRequest {
                request_id: "permission-api-request-01".into(),
                run_id: run.run_id.clone(),
                step_id: None,
                droid_id: run.droid_ids[0].clone(),
                capability: "write_project".into(),
                action: "write_file".into(),
                resource: "src/lib.rs".into(),
                rationale: "Write the requested implementation".into(),
                alternative_index: 0,
                app_server_generation: Some("api-test-generation".into()),
                thread_id: Some("api-test-thread".into()),
                turn_id: Some("api-test-turn".into()),
                item_id: Some("api-test-item".into()),
                json_rpc_request_id: Some("0".into()),
                app_server_approval_id: None,
                payload_json: r#"{"command":"write","path":"src/lib.rs"}"#.into(),
                payload_fingerprint: "api-test-fingerprint".into(),
            })
            .unwrap();
        let dispatcher = RunDispatcher::default();
        let mut approval = dispatcher
            .register_pending_approval(&run.run_id, "permission-api-request-01")
            .expect("register active app-server approval");
        let session = tokio::spawn(async move {
            let decision = approval.recv().await.expect("receive grant decision");
            assert!(decision.granted);
            decision
                .acknowledge
                .send(true)
                .expect("acknowledge app-server reply");
        });
        let worker = RunWorker::new(db.clone());
        let app = api_routes_with_run_auth_and_dispatcher(
            create_shared_state(),
            db,
            Some("run-test-token".into()),
            dispatcher,
            worker,
        );

        let unauthorized = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/runs/run-permission-api-01/permission-requests/permission-api-request-01/decisions")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"decision":"GRANT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let grant = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/runs/run-permission-api-01/permission-requests/permission-api-request-01/decisions")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Idempotency-Key", "permission-decision-01")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"decision":"GRANT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(grant.status(), StatusCode::OK);
        let grant_json: serde_json::Value =
            serde_json::from_slice(&grant.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(grant_json["permission"]["status"], "GRANTED");
        assert_eq!(grant_json["permission"]["resource"], "src/lib.rs");
        assert_eq!(
            grant_json["permission"]["payloadFingerprint"],
            "api-test-fingerprint"
        );
        assert_eq!(grant_json["responseSent"], true);
        assert_eq!(grant_json["run"]["status"], "EXECUTING");
        let decision_sequence = grant_json["run"]["lastSequence"].as_u64().unwrap();
        assert_eq!(decision_sequence, 6);
        session.await.unwrap();
        let acknowledged_sequence = decision_sequence;

        let retry = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/runs/run-permission-api-01/permission-requests/permission-api-request-01/decisions")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Idempotency-Key", "permission-decision-01")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"decision":"GRANT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(retry.status(), StatusCode::OK);
        let retry_json: serde_json::Value =
            serde_json::from_slice(&retry.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(retry_json["run"]["lastSequence"], acknowledged_sequence);
        assert_eq!(retry_json["run"]["status"], "EXECUTING");
        assert_eq!(retry_json["responseSent"], true);

        let conflicting_retry = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/runs/run-permission-api-01/permission-requests/permission-api-request-01/decisions")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Idempotency-Key", "different-decision-key")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"decision":"GRANT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(conflicting_retry.status(), StatusCode::CONFLICT);

        let permission = coordinator
            .permission_request("permission-api-request-01")
            .unwrap()
            .unwrap();
        assert_eq!(permission.status, PermissionStatus::Granted);
        assert_eq!(
            permission.grant_scope.as_deref(),
            Some("write_project:write_file:src/lib.rs")
        );
        assert!(permission.response_sent);

        coordinator
            .request_permission(&NewRunPermissionRequest {
                request_id: "permission-api-final-denial".into(),
                run_id: run.run_id.clone(),
                step_id: None,
                droid_id: run.droid_ids[0].clone(),
                capability: "external_access".into(),
                action: "open_url".into(),
                resource: "https://example.invalid".into(),
                rationale: "The last permission attempt was denied".into(),
                alternative_index: 3,
                app_server_generation: Some("api-test-generation".into()),
                thread_id: Some("api-test-thread".into()),
                turn_id: Some("api-test-turn-2".into()),
                item_id: Some("api-test-item-2".into()),
                json_rpc_request_id: Some("1".into()),
                app_server_approval_id: None,
                payload_json: r#"{"url":"https://example.invalid"}"#.into(),
                payload_fingerprint: "api-test-fingerprint-2".into(),
            })
            .unwrap();
        let denied = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/runs/run-permission-api-01/permission-requests/permission-api-final-denial/decisions")
                    .header(AUTHORIZATION, "Bearer run-test-token")
                    .header("Idempotency-Key", "permission-denial-01")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"decision":"DENY"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::OK);
        let denied_json: serde_json::Value =
            serde_json::from_slice(&denied.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(denied_json["permission"]["status"], "DENIED");
        assert_eq!(denied_json["run"]["status"], "BLOCKED");
        assert_eq!(denied_json["run"]["terminalOutcome"], "BLOCKED");
        assert_eq!(denied_json["responseSent"], false);
    }
}
