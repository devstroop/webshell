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
    /// Pre-configured host (optional)
    pub host: Option<String>,
    /// Pre-configured username (optional)
    pub user: Option<String>,
    /// Pre-configured password (optional) - enables auto-login
    pub password: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3000,
            workspace_dir: env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
            max_terminals: 10,
            idle_timeout: 3600,
            host: None,
            user: None,
            password: None,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let default_workspace = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            workspace_dir: env::var("WORKSPACE_DIR").unwrap_or(default_workspace),
            max_terminals: env::var("MAX_TERMINALS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            idle_timeout: env::var("IDLE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            host: env::var("WEBSHELL_HOST").ok().filter(|s| !s.is_empty()),
            user: env::var("WEBSHELL_USER").ok().filter(|s| !s.is_empty()),
            password: env::var("WEBSHELL_PASSWORD").ok().filter(|s| !s.is_empty()),
        }
    }

    /// Check if this is a local connection
    pub fn is_local(&self) -> bool {
        match &self.host {
            None => true,
            Some(h) => h == "localhost" || h == "127.0.0.1" || h.starts_with("127."),
        }
    }

    /// Check if auto-login is enabled (all credentials provided)
    pub fn auto_login(&self) -> bool {
        self.user.is_some() && self.password.is_some()
    }
}
