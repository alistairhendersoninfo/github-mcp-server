use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, warn};

// Metrics
use prometheus::{Counter, Histogram, Gauge, Registry, Encoder, TextEncoder};
use std::sync::Mutex;

mod auth;
mod config;
mod error;
mod github;
mod mcp;
mod security;
mod metrics;

use config::Config;
use error::AppError;
use metrics::Metrics;

type AppState = Arc<AppStateInner>;

#[derive(Clone)]
struct AppStateInner {
    config: Config,
    db: sqlx::SqlitePool,
    metrics: Arc<Metrics>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting GitHub MCP Server");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize database
    let db = sqlx::SqlitePool::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    info!("Database initialized and migrations applied");

    // Initialize metrics
    let metrics = Arc::new(Metrics::new().expect("Failed to create metrics"));
    info!("Metrics initialized");

    // Create application state
    let state = Arc::new(AppStateInner { 
        config: config.clone(), 
        db,
        metrics: metrics.clone(),
    });

    // Build application router
    let app = create_router(state);

    // Start server
    let listener = TcpListener::bind(&format!("{}:{}", config.host, config.port)).await?;
    info!("Server listening on {}:{}", config.host, config.port);

    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        
        // Metrics endpoint
        .route("/metrics", get(metrics::metrics_handler))
        
        // Authentication routes
        .route("/auth/github", get(auth::github_oauth_start))
        .route("/auth/github/callback", get(auth::github_oauth_callback))
        .route("/auth/token/refresh", post(auth::refresh_token))
        
        // MCP protocol endpoints
        .route("/mcp", post(mcp::handle_mcp_request))
        .route("/mcp/ws", get(mcp::websocket_handler))
        
        // GitHub workflow endpoints
        .route("/github/push", post(github::handle_push))
        .route("/github/scan-tasks", post(github::handle_scan_tasks))
        .route("/github/merge", post(github::handle_merge))
        
        // Static file serving for web interface
        .nest_service("/", ServeDir::new("web"))
        
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(security::rate_limiting_layer())
        .layer(security::security_headers_layer())
        
        // Application state
        .with_state(state)
}

async fn health_check() -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "github-mcp-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}