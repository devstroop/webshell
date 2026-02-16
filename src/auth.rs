//! OS-native authentication
//!
//! Verifies credentials against the operating system.
//! - macOS: Uses `dscl . -authonly`
//! - Linux: Uses `su -c true` or PAM via command

use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session token with username
#[derive(Debug, Clone)]
pub struct Session {
    pub username: String,
    pub created_at: std::time::Instant,
}

/// Session store for authenticated users
#[derive(Debug, Clone, Default)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new session for authenticated user
    pub async fn create_session(&self, username: String) -> String {
        let token = generate_token();
        let session = Session {
            username,
            created_at: std::time::Instant::now(),
        };
        self.sessions.write().await.insert(token.clone(), session);
        token
    }

    /// Validate a session token and return the username
    pub async fn validate_session(&self, token: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions.get(token).map(|s| s.username.clone())
    }

    /// Remove a session
    pub async fn remove_session(&self, token: &str) {
        self.sessions.write().await.remove(token);
    }

    /// Clean up expired sessions (older than 24 hours)
    #[allow(dead_code)]
    pub async fn cleanup_expired(&self) {
        let max_age = std::time::Duration::from_secs(24 * 60 * 60);
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, s| s.created_at.elapsed() < max_age);
    }
}

/// Generate a secure random token
fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.update(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes(),
    );
    hex::encode(hasher.finalize())
}

/// Authenticate user against OS
/// Returns Ok(username) on success, Err(message) on failure
pub fn authenticate_os(username: &str, password: &str) -> Result<String, String> {
    // Validate input
    if username.is_empty() || password.is_empty() {
        return Err("Username and password required".to_string());
    }

    // Sanitize username (prevent command injection)
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err("Invalid username".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        authenticate_macos(username, password)
    }

    #[cfg(target_os = "linux")]
    {
        authenticate_linux(username, password)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err("Authentication not supported on this platform".to_string())
    }
}

/// macOS authentication using Directory Services
#[cfg(target_os = "macos")]
fn authenticate_macos(username: &str, password: &str) -> Result<String, String> {
    // dscl . -authonly username password
    // Note: password on command line is visible in `ps` briefly,
    // but this is acceptable for a local dev terminal
    let output = Command::new("dscl")
        .args([".", "-authonly", username, password])
        .output()
        .map_err(|e| format!("Failed to spawn auth process: {}", e))?;

    if output.status.success() {
        Ok(username.to_string())
    } else {
        Err("Invalid username or password".to_string())
    }
}

/// Linux authentication using PAM via su
#[cfg(target_os = "linux")]
fn authenticate_linux(username: &str, password: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::Stdio;

    // Use `su` with password via expect-like mechanism
    // Note: This requires the user running webshell to have appropriate permissions
    let mut child = Command::new("su")
        .args(["-c", "true", username])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn auth process: {}", e))?;

    // Write password to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .map_err(|e| format!("Failed to write password: {}", e))?;
    }

    let status = child
        .wait()
        .map_err(|e| format!("Auth process failed: {}", e))?;

    if status.success() {
        Ok(username.to_string())
    } else {
        Err("Invalid username or password".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_store() {
        let store = SessionStore::new();
        let token = store.create_session("testuser".to_string()).await;

        let username = store.validate_session(&token).await;
        assert_eq!(username, Some("testuser".to_string()));

        store.remove_session(&token).await;
        let username = store.validate_session(&token).await;
        assert_eq!(username, None);
    }
}
