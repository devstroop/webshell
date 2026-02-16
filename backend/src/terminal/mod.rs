//! Terminal/PTY module
//!
//! Extracted from webide project - provides terminal emulation with PTY support.

pub mod error;
pub mod pty;
pub mod session;
pub mod socketio;

pub use error::TerminalError;
pub use pty::{PtyManager, TerminalHandle};
pub use session::{SessionManager, TerminalConfig};
pub use socketio::create_terminal_socketio_layer;
