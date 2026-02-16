//! Terminal/PTY module
//!
//! Provides terminal emulation with PTY support.

pub mod error;
pub mod pty;
pub mod session;

pub use error::TerminalError;
pub use pty::{PtyManager, TerminalHandle};
pub use session::SessionManager;
