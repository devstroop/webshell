//! WebShell Backend
//!
//! A lightweight web-based terminal service using native WebSockets.
//! Features OS-native authentication via PAM.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
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

mod auth;
mod config;
mod ssh;
mod terminal;
mod types;

use auth::{authenticate_os, SessionStore};
use config::{AuthMethod, Config};
use ssh::{SshAuth, SshConfig, SshSession};
use terminal::{PtyManager, SessionManager};
use types::{ShellOutput, WsMessage};

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    session_manager: Arc<SessionManager>,
    auth_sessions: SessionStore,
}

const SESSION_COOKIE: &str = "webshell_session";

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

    // Create auth session store
    let auth_sessions = SessionStore::new();

    let state = AppState {
        config: config.clone(),
        session_manager,
        auth_sessions,
    };

    // Resolve static files path
    let static_dir = std::env::var("WEBSHELL_STATIC_DIR")
        .unwrap_or_else(|_| "./static".to_string());

    tracing::info!("Serving static files from: {}", static_dir);

    // Log connection mode
    if config.auto_login() {
        tracing::info!("Auto-login enabled for user: {}", config.user.as_deref().unwrap_or(""));
    }
    if let Some(host) = &config.host {
        tracing::info!("Target host: {}", host);
    }

    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/config", get(config_handler))
        .route("/api/login", post(login_handler))
        .route("/api/logout", post(logout_handler))
        .route("/api/session", get(session_check))
        .route("/ws", get(ws_handler))
        .fallback_service(
            ServeDir::new(&static_dir)
                .fallback(ServeDir::new(format!("{}/index.html", static_dir))),
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

/// Config response - tells UI which fields to show
#[derive(Debug, Serialize)]
struct ConfigResponse {
    /// Pre-configured host (None = show field)
    host: Option<String>,
    /// Pre-configured user (None = show field)
    user: Option<String>,
    /// Auth method: "none", "password", "key_file", "key_data"
    auth_method: String,
    /// If true, auto-login (user + auth configured)
    auto_login: bool,
    /// Is this a local connection?
    is_local: bool,
}

/// Config handler - returns UI configuration
async fn config_handler(State(state): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        host: state.config.host.clone(),
        user: state.config.user.clone(),
        auth_method: state.config.auth_method_name().to_string(),
        auto_login: state.config.auto_login(),
        is_local: state.config.is_local(),
    })
}

/// Login request
#[derive(Debug, Deserialize)]
struct LoginRequest {
    host: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

/// Login response
#[derive(Debug, Serialize)]
struct LoginResponse {
    success: bool,
    message: String,
    username: Option<String>,
}

/// Login handler - authenticates against OS or SSH
/// Uses env vars if available, falling back to form values
async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(login): Form<LoginRequest>,
) -> impl IntoResponse {
    // Use configured values, fall back to form input
    let host = state.config.host.clone()
        .or(login.host)
        .unwrap_or_else(|| "localhost".to_string());
    let username = state.config.user.clone()
        .or(login.username)
        .unwrap_or_default();
    
    // Determine auth method
    let form_password = login.password.unwrap_or_default();
    let is_local = host == "localhost" || host == "127.0.0.1" || host.starts_with("127.");

    tracing::info!("Login attempt for user: {} on host: {} (local: {})", username, host, is_local);

    let auth_result = if is_local {
        // For local connections, use OS auth
        let password = match &state.config.auth {
            AuthMethod::Password(p) => p.clone(),
            _ => form_password.clone(),
        };
        
        if username.is_empty() || password.is_empty() {
            Err("Username and password required".to_string())
        } else {
            authenticate_os(&username, &password)
        }
    } else {
        // For remote connections, use SSH
        let ssh_auth = match &state.config.auth {
            AuthMethod::Password(p) => SshAuth::Password(p.clone()),
            AuthMethod::KeyFile { path, passphrase } => SshAuth::KeyFile { 
                path: path.clone(), 
                passphrase: passphrase.clone() 
            },
            AuthMethod::KeyData { data, passphrase } => SshAuth::KeyData { 
                data: data.clone(), 
                passphrase: passphrase.clone() 
            },
            AuthMethod::None => {
                // Use form password if no auth method configured
                if form_password.is_empty() {
                    return (
                        jar,
                        Json(LoginResponse {
                            success: false,
                            message: "Password required".to_string(),
                            username: None,
                        }),
                    );
                }
                SshAuth::Password(form_password)
            }
        };

        if username.is_empty() {
            Err("Username required".to_string())
        } else {
            // Test SSH connection
            let ssh_config = SshConfig {
                host: host.clone(),
                port: 22,
                user: username.clone(),
                auth: ssh_auth,
            };
            
            match ssh::test_connection(ssh_config).await {
                Ok(_) => Ok(username.clone()),
                Err(e) => Err(e),
            }
        }
    };

    match auth_result {
        Ok(username) => {
            let token = state.auth_sessions.create_session(username.clone()).await;
            tracing::info!("Login successful for user: {}", username);

            let cookie = Cookie::build((SESSION_COOKIE, token))
                .path("/")
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Strict)
                .build();

            (
                jar.add(cookie),
                Json(LoginResponse {
                    success: true,
                    message: "Login successful".to_string(),
                    username: Some(username),
                }),
            )
        }
        Err(e) => {
            tracing::warn!("Login failed for user {}: {}", username, e);
            (
                jar,
                Json(LoginResponse {
                    success: false,
                    message: e,
                    username: None,
                }),
            )
        }
    }
}

/// Logout handler
async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        state.auth_sessions.remove_session(cookie.value()).await;
    }

    let removal = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .build();

    (jar.remove(removal), Json(serde_json::json!({"success": true})))
}

/// Session check - returns current user if authenticated
async fn session_check(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        if let Some(username) = state.auth_sessions.validate_session(cookie.value()).await {
            return Json(serde_json::json!({
                "authenticated": true,
                "username": username
            }));
        }
    }

    Json(serde_json::json!({
        "authenticated": false
    }))
}

/// WebSocket handler - requires authentication
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    // Check authentication
    let session = if let Some(cookie) = jar.get(SESSION_COOKIE) {
        let token = cookie.value().to_string();
        state.auth_sessions.validate_session(&token).await
            .map(|user| (token, user))
    } else {
        None
    };

    match session {
        Some((token, user)) => {
            tracing::info!("WebSocket connection authenticated for user: {}", user);
            ws.on_upgrade(move |socket| handle_socket(socket, state, user, token))
                .into_response()
        }
        None => {
            tracing::warn!("Unauthenticated WebSocket connection attempt");
            (StatusCode::UNAUTHORIZED, "Authentication required").into_response()
        }
    }
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState, username: String, session_token: String) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    let connection_id = uuid::Uuid::new_v4().to_string();
    tracing::info!("WebSocket connected: {} (user: {})", connection_id, username);

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
    
    // Logout on disconnect
    state.auth_sessions.remove_session(&session_token).await;
    tracing::info!("WebSocket disconnected, session invalidated: {}", connection_id);
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
                let _ = tx_clone.send(WsMessage::ShellOutput(ShellOutput {
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
