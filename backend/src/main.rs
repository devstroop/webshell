//! WebShell Backend
//!
//! A lightweight web-based terminal service.

use axum::{
    routing::get,
    Router,
};
use socketioxide::SocketIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod terminal;

use config::Config;
use terminal::{create_terminal_socketio_layer, SessionManager};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webshell_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env());
    tracing::info!("Starting WebShell backend on port {}", config.port);
    tracing::info!("Workspace directory: {}", config.workspace_dir);

    // Create terminal session manager
    let session_manager = Arc::new(SessionManager::new(config.clone()));

    // Create Socket.IO layer for terminal communication
    let (socketio_layer, io) = create_terminal_socketio_layer(session_manager.clone());

    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .nest_service("/", ServeDir::new("../frontend/dist").fallback(ServeDir::new("../frontend/dist/index.html")))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .layer(socketio_layer),
        )
        .with_state(io);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("🚀 WebShell backend listening on http://{}", addr);
    tracing::info!("📡 Socket.IO endpoint: /socket.io");

    axum::serve(listener, app).await.unwrap();
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
