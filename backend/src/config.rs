//! Configuration management

use std::env;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP server port
    pub port: u16,
    /// Base workspace directory for terminal sessions
    pub workspace_dir: String,
    /// Maximum terminals per session
    pub max_terminals: usize,
    /// Terminal idle timeout (seconds)
    pub idle_timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3000,
            workspace_dir: "/workspace".to_string(),
            max_terminals: 10,
            idle_timeout: 3600,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            workspace_dir: env::var("WORKSPACE_DIR")
                .unwrap_or_else(|_| "/workspace".to_string()),
            max_terminals: env::var("MAX_TERMINALS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            idle_timeout: env::var("IDLE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
        }
    }
}
