use std::net::SocketAddr;
use axum::Router;
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;

use server::api::api_routes;
use server::state::create_shared_state;

/// Teste de Integração End-to-End:
/// 1. Sobe o servidor Axum com estado em memória em porta aleatória.
/// 2. Conecta um cliente WebSocket em /ws e recebe o WORLD_SNAPSHOT.
/// 3. Emite um HTTP POST /api/v1/bridge/dispatch com payload e token de autenticação.
/// 4. Valida resposta HTTP 202 Accepted.
/// 5. Valida recepção em tempo real do evento TASK_ASSIGNED no WebSocket.
#[tokio::test]
async fn test_full_bridge_dispatch_and_websocket_stream_e2e() {
    let state = create_shared_state();
    let app = Router::new().merge(api_routes(state.clone()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Falha ao abrir porta efêmera para teste");
    let addr: SocketAddr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 1. Conecta no WebSocket
    let ws_url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Falha ao conectar no WebSocket de teste");

    // Recebe e valida o snapshot inicial
    let msg_snapshot = ws_stream
        .next()
        .await
        .expect("Nenhuma mensagem recebida")
        .expect("Erro na mensagem WS");
    let snapshot: Value = serde_json::from_str(&msg_snapshot.to_string()).unwrap();
    assert_eq!(snapshot["topic"], "WORLD_SNAPSHOT");

    // 2. Faz o disparo HTTP externo (simulando Codex / Nuvem)
    let http_client = reqwest::Client::new();
    let dispatch_url = format!("http://{}/api/v1/bridge/dispatch", addr);

    let payload = json!({
        "source": "codex-e2e-runner",
        "projectId": "proj-notex-backend",
        "targetAgentId": "agent-rust-dev",
        "task": {
            "taskId": "task-e2e-999",
            "title": "Executar migracao SQLite",
            "priority": "HIGH",
            "payload": {
                "prompt": "Executar teste e2e completo...",
                "contextFiles": ["src/db/migration.rs"]
            }
        },
        "callbackWebhook": "https://cloud.engine.io/tasks/task-e2e-999/callback"
    });

    let resp = http_client
        .post(&dispatch_url)
        .header("Authorization", "Bearer secret-bridge-token")
        .json(&payload)
        .send()
        .await
        .expect("Falha na requisição HTTP de despacho");

    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let resp_body: Value = resp.json().await.expect("Falha ao parsear corpo de resposta");
    assert_eq!(resp_body["status"], "ACCEPTED");
    assert_eq!(resp_body["taskId"], "task-e2e-999");
    assert_eq!(resp_body["targetAgentId"], "agent-rust-dev");

    // 3. Valida a chegada do evento TASK_ASSIGNED no WebSocket conectado
    let msg_event = ws_stream
        .next()
        .await
        .expect("Nenhum evento transmitido pelo WS")
        .expect("Erro na mensagem WS");
    let event: Value = serde_json::from_str(&msg_event.to_string()).unwrap();

    assert_eq!(event["topic"], "TASK_ASSIGNED");
    assert_eq!(event["payload"]["taskId"], "task-e2e-999");
    assert_eq!(event["payload"]["agentId"], "agent-rust-dev");
    assert_eq!(event["payload"]["targetPod"], "proj-notex-backend");

    server_handle.abort();
}
