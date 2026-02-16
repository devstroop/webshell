//! WebSocket message types

use serde::{Deserialize, Serialize};

/// Terminal open request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOpenRequest {
    pub id: String,
    pub cols: u16,
    pub rows: u16,
}

/// Terminal input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInput {
    pub id: String,
    pub input: String,
}

/// Terminal resize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalResize {
    pub id: String,
    pub cols: u16,
    pub rows: u16,
}

/// Terminal close request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalClose {
    pub id: String,
}

/// Shell output from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellOutput {
    pub id: String,
    pub output: String,
}

/// Shell exit notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellExit {
    pub id: String,
    pub code: Option<i32>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Client requests to open a terminal
    #[serde(rename = "term.open")]
    TerminalOpen(TerminalOpenRequest),

    /// Client sends input to terminal
    #[serde(rename = "term.input")]
    TerminalInput(TerminalInput),

    /// Client requests terminal resize
    #[serde(rename = "term.resize")]
    TerminalResize(TerminalResize),

    /// Client requests to close terminal
    #[serde(rename = "term.close")]
    TerminalClose(TerminalClose),

    /// Server sends shell output
    #[serde(rename = "shell.output")]
    ShellOutput(ShellOutput),

    /// Server notifies shell exit
    #[serde(rename = "shell.exit")]
    ShellExit(ShellExit),
}
