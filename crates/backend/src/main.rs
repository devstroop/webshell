//! WebShell Backend
//!
//! A lightweight web-based terminal service using native WebSockets.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
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
use terminal::{PtyManager, SessionManager};
use webshell_shared::WsMessage;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    session_manager: Arc<SessionManager>,
}

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

    let state = AppState {
        config: config.clone(),
        session_manager,
    };

    // Resolve frontend dist path (relative to workspace root or binary location)
    let frontend_dist = std::env::var("WEBSHELL_STATIC_DIR")
        .unwrap_or_else(|_| "./crates/frontend/dist".to_string());

    tracing::info!("Serving static files from: {}", frontend_dist);

    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(ws_handler))
        .fallback_service(
            ServeDir::new(&frontend_dist)
                .fallback(ServeDir::new(format!("{}/index.html", frontend_dist))),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("🚀 WebShell backend listening on http://{}", addr);
    tracing::info!("📡 WebSocket endpoint: /ws");

    axum::serve(listener, app).await.unwrap();
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// WebSocket handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    let connection_id = uuid::Uuid::new_v4().to_string();
    tracing::info!("WebSocket connected: {}", connection_id);

    // Spawn task to send messages to the WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    handle_message(ws_msg, &state, tx.clone()).await;
                }
            }
            Message::Close(_) => {
                tracing::info!("WebSocket closed: {}", connection_id);
                break;
            }
            _ => {}
        }
    }

    send_task.abort();
    tracing::info!("WebSocket disconnected: {}", connection_id);
}

/// Handle a WebSocket message
async fn handle_message(msg: WsMessage, state: &AppState, tx: mpsc::UnboundedSender<WsMessage>) {
    match msg {
        WsMessage::TerminalOpen(req) => {
            tracing::info!("Opening terminal: {}", req.id);

            let tx_clone = tx.clone();
            let terminal_id = req.id.clone();

            // Create output callback
            let output_callback = move |output: String| {
                let _ = tx_clone.send(WsMessage::ShellOutput(webshell_shared::ShellOutput {
                    id: terminal_id.clone(),
                    output,
                }));
            };

            // Create the terminal
            match state
                .session_manager
                .create_terminal(&req.id, req.cols, req.rows, Box::new(output_callback))
                .await
            {
                Ok(_) => {
                    tracing::info!("Terminal created: {}", req.id);
                }
                Err(e) => {
                    tracing::error!("Failed to create terminal {}: {}", req.id, e);
                }
            }
        }

        WsMessage::TerminalInput(input) => {
            if let Err(e) = state
                .session_manager
                .write_to_terminal(&input.id, &input.input)
                .await
            {
                tracing::error!("Failed to write to terminal {}: {}", input.id, e);
            }
        }

        WsMessage::TerminalResize(resize) => {
            if let Err(e) = state
                .session_manager
                .resize_terminal(&resize.id, resize.cols, resize.rows)
                .await
            {
                tracing::error!("Failed to resize terminal {}: {}", resize.id, e);
            }
        }

        WsMessage::TerminalClose(close) => {
            tracing::info!("Closing terminal: {}", close.id);
            state.session_manager.close_terminal(&close.id).await;
        }

        // Server-to-client messages - ignore if received from client
        WsMessage::ShellOutput(_) | WsMessage::ShellExit(_) => {}
    }
}
