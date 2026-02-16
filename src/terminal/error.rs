//! Terminal error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("Terminal not found: {0}")]
    NotFound(String),

    #[error("Terminal already exists: {0}")]
    AlreadyExists(String),

    #[error("PTY error: {0}")]
    PtyError(#[from] std::io::Error),

    #[error("Terminal process exited")]
    #[allow(dead_code)]
    ProcessExited,

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Maximum terminals reached")]
    MaxTerminalsReached,

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}
