//! Configuration management

use std::env;

/// Authentication method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Password authentication
    Password(String),
    /// SSH key file path
    KeyFile {
        path: String,
        passphrase: Option<String>,
    },
    /// SSH key data (inline)
    KeyData {
        data: String,
        passphrase: Option<String>,
    },
    /// No pre-configured auth (prompt user)
    None,
}

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
    /// SSH port for remote connections (default: 22)
    pub ssh_port: u16,
    /// Pre-configured username (optional)
    pub user: Option<String>,
    /// Authentication method
    pub auth: AuthMethod,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 2222,
            workspace_dir: env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
            max_terminals: 10,
            idle_timeout: 3600,
            host: None,
            ssh_port: 22,
            user: None,
            auth: AuthMethod::None,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let default_workspace = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

        // Determine auth method from env vars
        let passphrase = env::var("WEBSHELL_SSH_PASSPHRASE")
            .ok()
            .filter(|s| !s.is_empty());

        let auth = if let Ok(key_data) = env::var("WEBSHELL_SSH_KEY_DATA") {
            if !key_data.is_empty() {
                AuthMethod::KeyData {
                    data: key_data,
                    passphrase,
                }
            } else {
                AuthMethod::None
            }
        } else if let Ok(key_path) = env::var("WEBSHELL_SSH_KEY") {
            if !key_path.is_empty() {
                AuthMethod::KeyFile {
                    path: key_path,
                    passphrase,
                }
            } else {
                AuthMethod::None
            }
        } else if let Ok(password) = env::var("WEBSHELL_PASSWORD") {
            if !password.is_empty() {
                AuthMethod::Password(password)
            } else {
                AuthMethod::None
            }
        } else {
            AuthMethod::None
        };

        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2222),
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
            ssh_port: env::var("WEBSHELL_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(22),
            user: env::var("WEBSHELL_USER").ok().filter(|s| !s.is_empty()),
            auth,
        }
    }

    /// Check if this is a local connection
    pub fn is_local(&self) -> bool {
        match &self.host {
            None => true,
            Some(h) => h == "localhost" || h == "127.0.0.1" || h.starts_with("127."),
        }
    }

    /// Check if auto-login is enabled (user + auth configured)
    pub fn auto_login(&self) -> bool {
        self.user.is_some() && !matches!(self.auth, AuthMethod::None)
    }

    /// Get auth method name for UI
    pub fn auth_method_name(&self) -> &'static str {
        match &self.auth {
            AuthMethod::Password(_) => "password",
            AuthMethod::KeyFile { .. } => "key_file",
            AuthMethod::KeyData { .. } => "key_data",
            AuthMethod::None => "none",
        }
    }
}
