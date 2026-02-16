//! Socket.IO Server for Terminal Communication

use serde::{Deserialize, Serialize};
use socketioxide::{
    extract::{Data, SocketRef, State as SioState},
    SocketIo,
};
use std::sync::Arc;

use super::session::SessionManager;

/// Request to open a terminal
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTerminalRequest {
    pub id: String,
    pub cols: u16,
    pub rows: u16,
}

/// Request to send input to terminal
#[derive(Debug, Deserialize)]
pub struct InputRequest {
    pub id: String,
    pub input: String,
}

/// Request to resize terminal
#[derive(Debug, Deserialize)]
pub struct ResizeRequest {
    pub id: String,
    pub cols: u16,
    pub rows: u16,
}

/// Request to close terminal
#[derive(Debug, Deserialize)]
pub struct CloseRequest {
    pub id: String,
}

/// Terminal output response
#[derive(Debug, Serialize)]
pub struct OutputResponse {
    pub id: String,
    pub output: String,
}

/// Terminal error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub id: String,
    pub error: String,
}

/// Terminal exit response
#[derive(Debug, Serialize)]
pub struct ExitResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
}

/// Get the SocketIo layer for use with axum (also returns the SocketIo for routing)
pub fn create_terminal_socketio_layer(
    session_manager: Arc<SessionManager>,
) -> (socketioxide::layer::SocketIoLayer, SocketIo) {
    let (layer, io) = SocketIo::builder()
        .with_state(session_manager)
        .build_layer();

    io.ns("/", |socket: SocketRef, session_mgr: SioState<Arc<SessionManager>>| {
        handle_connection(socket, session_mgr);
    });

    (layer, io)
}

/// Handle a new socket connection
fn handle_connection(socket: SocketRef, session_mgr: SioState<Arc<SessionManager>>) {
    let socket_id = socket.id.to_string();
    tracing::info!("Terminal client connected: {}", socket_id);

    // Register event handlers
    register_term_open(socket.clone());
    register_term_input(socket.clone());
    register_term_resize(socket.clone());
    register_term_close(socket.clone());
    register_disconnect(socket);

    let _ = session_mgr;
}

/// Register term.open event handler
fn register_term_open(socket: SocketRef) {
    socket.on(
        "term.open",
        |socket: SocketRef,
         Data::<OpenTerminalRequest>(data),
         session_mgr: SioState<Arc<SessionManager>>| async move {
            tracing::info!("Opening terminal: {} ({}x{})", data.id, data.cols, data.rows);

            let socket_clone = socket.clone();
            let term_id = data.id.clone();

            // Create output callback that emits to socket
            let output_callback = move |output: Vec<u8>| {
                let output_str = String::from_utf8_lossy(&output).to_string();

                let response = OutputResponse {
                    id: term_id.clone(),
                    output: output_str,
                };

                if let Err(e) = socket_clone.emit("shell.output", &response) {
                    tracing::debug!("Failed to emit shell.output: {}", e);
                }
            };

            match session_mgr
                .create(data.id.clone(), data.cols, data.rows, output_callback)
                .await
            {
                Ok(_handle) => {
                    tracing::info!("Terminal {} opened successfully", data.id);
                }
                Err(e) => {
                    tracing::error!("Failed to open terminal {}: {}", data.id, e);
                    let _ = socket.emit(
                        "term.error",
                        &ErrorResponse {
                            id: data.id,
                            error: e.to_string(),
                        },
                    );
                }
            }
        },
    );
}

/// Register term.input event handler
fn register_term_input(socket: SocketRef) {
    socket.on(
        "term.input",
        |Data::<InputRequest>(data), session_mgr: SioState<Arc<SessionManager>>| async move {
            if let Err(e) = session_mgr.send_input(&data.id, data.input.into_bytes()).await {
                tracing::debug!("Failed to send input to terminal {}: {}", data.id, e);
            }
        },
    );
}

/// Register term.resize event handler
fn register_term_resize(socket: SocketRef) {
    socket.on(
        "term.resize",
        |Data::<ResizeRequest>(data), session_mgr: SioState<Arc<SessionManager>>| async move {
            tracing::debug!("Resizing terminal {}: {}x{}", data.id, data.cols, data.rows);

            if let Err(e) = session_mgr.resize(&data.id, data.cols, data.rows).await {
                tracing::debug!("Failed to resize terminal {}: {}", data.id, e);
            }
        },
    );
}

/// Register term.close event handler
fn register_term_close(socket: SocketRef) {
    socket.on(
        "term.close",
        |socket: SocketRef,
         Data::<CloseRequest>(data),
         session_mgr: SioState<Arc<SessionManager>>| async move {
            tracing::info!("Closing terminal: {}", data.id);

            if let Err(e) = session_mgr.close(&data.id).await {
                tracing::warn!("Failed to close terminal {}: {}", data.id, e);
            }

            // Emit exit event
            let _ = socket.emit(
                "shell.exit",
                &ExitResponse {
                    id: data.id,
                    code: Some(0),
                },
            );
        },
    );
}

/// Register disconnect handler
fn register_disconnect(socket: SocketRef) {
    socket.on_disconnect(|socket: SocketRef| async move {
        tracing::info!("Terminal client disconnected: {}", socket.id);
        // Note: We don't close terminals on disconnect to support reconnection
    });
}
