use axum::http::{
    HeaderName, HeaderValue, Method,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use server::api::api_routes_with_worker;
use server::run::dispatcher::RunDispatcher;
use server::run::{coordinator::RunCoordinator, recovery::recover_at_startup, worker::RunWorker};
use server::state::create_shared_state;
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializa logs estruturados com Tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Falha ao configurar global tracing subscriber");

    // Aceita apenas as origens usadas pelo Vite e pelo WebView local do Tauri.
    let allowed_origins = [
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("http://localhost:4173"),
        HeaderValue::from_static("http://tauri.localhost"),
        HeaderValue::from_static("tauri://localhost"),
    ];
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            HeaderName::from_static("idempotency-key"),
        ]);

    // Inicializa o banco de dados relacional SQLite local
    let db = server::db::Database::init(server::db::Database::default_db_path())?;

    // Reconcilia runs persistidos antes de abrir a API. A recuperação nunca inicia turns.
    let recovery_report = recover_at_startup(&db, &RunCoordinator::new(db.clone())).await?;
    info!(
        inspected = recovery_report.runs_inspected,
        queued_preserved = recovery_report.queued_runs_preserved,
        permissions_preserved = recovery_report.pending_permissions_preserved,
        threads_resumed = recovery_report.threads_resumed,
        thread_resume_failures = recovery_report.thread_resume_failures,
        blocked = recovery_report.runs_blocked,
        "Startup run recovery finished"
    );

    let dispatcher = RunDispatcher::default();
    let worker = RunWorker::new_with_dispatcher(db.clone(), dispatcher.clone());
    let requeued = worker.enqueue_queued_runs()?;
    let approvals_resumed = worker.enqueue_pending_approval_runs()?;
    info!(
        requeued,
        approvals_resumed,
        enabled = worker.enabled(),
        "Run worker initialized"
    );

    // Inicializa o estado compartilhado da simulação em memória
    let shared_state = create_shared_state();

    // Roteador consolidado da aplicação
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(api_routes_with_worker(
            shared_state,
            db.clone(),
            dispatcher,
            worker,
        ))
        .layer(cors);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!(
        "🚀 The Grid Metropolis Bridge Server ativo em http://{}",
        addr
    );

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
