//! Shared types for WebShell
//!
//! Common types used by both the backend and frontend.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Terminal settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    pub font_size: u16,
    pub font_family: String,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    pub scrollback: u32,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            font_size: 14,
            font_family: "'JetBrains Mono', 'Fira Code', Consolas, monospace".to_string(),
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            scrollback: 10000,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

/// Terminal session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub name: String,
}

impl TerminalSession {
    pub fn new(number: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: format!("Terminal {}", number),
        }
    }
}

/// Generate a new terminal ID
pub fn generate_terminal_id() -> String {
    Uuid::new_v4().to_string()
}
