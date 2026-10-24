use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::state::SharedState;

/// Endpoint WebSocket para streaming de eventos em tempo real para o Canvas.
/// Rota: GET /ws
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Gerencia a conexão WebSocket individual do cliente frontend.
async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Cria uma assinatura no canal de eventos em tempo real
    let mut event_rx = {
        let world = state.read().await;
        world.subscribe()
    };

    // 2. Envia imediatamente o Snapshot do Estado Atual para sincronização do Canvas
    let initial_snapshot = {
        let world = state.read().await;
        json!({
            "topic": "WORLD_SNAPSHOT",
            "timestamp": match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(d) => d.as_millis() as i64,
                Err(_) => 0,
            },
            "payload": {
                "agents": world.list_agents(),
                "pods": world.list_pods(),
                "tasks": world.list_tasks(),
            }
        })
    };

    if let Ok(snapshot_json) = serde_json::to_string(&initial_snapshot) {
        if sender.send(Message::Text(snapshot_json.into())).await.is_err() {
            warn!("Cliente desconectou antes de receber o snapshot inicial do mundo");
            return;
        }
    }

    info!("Novo cliente conectado ao WebSocket Hub do The Grid Metropolis");

    // 3. Task para encaminhar eventos do canal de Broadcast Tokio para o WebSocket do cliente
    let mut send_task = tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => match serde_json::to_string(&event) {
                    Ok(json_str) => {
                        if sender.send(Message::Text(json_str.into())).await.is_err() {
                            // Cliente desconectou
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Falha ao serializar WebSocketEvent: {}", e);
                    }
                },
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!("Cliente atrasou e perdeu {} mensagens no canal de broadcast", skipped);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Canal de broadcast do servidor foi encerrado");
                    break;
                }
            }
        }
    });

    // 4. Task para processar mensagens recebidas do cliente (Ping, heartbeat, etc.)
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Mensagem recebida do cliente WS: {}", text);
                }
                Ok(Message::Ping(_)) => {
                    debug!("Ping recebido do cliente");
                }
                Ok(Message::Pong(_)) => {
                    debug!("Pong recebido do cliente");
                }
                Ok(Message::Close(_)) => {
                    info!("Cliente enviou sinal de fechamento de conexão WebSocket");
                    break;
                }
                Err(e) => {
                    warn!("Erro na recepção do WebSocket: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Se qualquer uma das tarefas terminar (desconexão ou erro), aborta a outra
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("Conexão WebSocket finalizada com sucesso");
}
