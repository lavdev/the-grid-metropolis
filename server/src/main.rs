use std::net::SocketAddr;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use server::api::api_routes;
use server::state::create_shared_state;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializa logs estruturados com Tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Falha ao configurar global tracing subscriber");

    // Configuração de CORS permissivo para dev do frontend (Svelte/Phaser)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Inicializa o estado compartilhado da simulação em memória
    let shared_state = create_shared_state();

    // Roteador consolidado da aplicação
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(api_routes(shared_state))
        .layer(cors);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 The Grid Metropolis Bridge Server ativo em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Endpoint de Health Check da infraestrutura
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "online",
        "service": "the-grid-metropolis-bridge",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
