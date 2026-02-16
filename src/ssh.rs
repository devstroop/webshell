//! SSH client for remote terminal connections

use async_trait::async_trait;
use russh::*;
use russh_keys::*;
use std::sync::Arc;

/// SSH authentication method
#[derive(Debug, Clone)]
pub enum SshAuth {
    Password(String),
    KeyFile {
        path: String,
        passphrase: Option<String>,
    },
    KeyData {
        data: String,
        passphrase: Option<String>,
    },
}

/// SSH connection configuration
#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub auth: SshAuth,
}

/// SSH client handler
struct ClientHandler;

#[async_trait]
impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all server keys (like ssh -o StrictHostKeyChecking=no)
        // In production, you'd want to verify against known_hosts
        Ok(true)
    }
}

/// Test SSH connection and authentication
pub async fn test_connection(config: SshConfig) -> Result<String, String> {
    let russh_config = client::Config::default();
    let config_arc = Arc::new(russh_config);
    let addr = format!("{}:{}", config.host, config.port);

    let mut session = client::connect(config_arc, &addr, ClientHandler)
        .await
        .map_err(|e| format!("SSH connection failed: {}", e))?;

    // Authenticate
    let auth_result = match config.auth {
        SshAuth::Password(password) => session
            .authenticate_password(&config.user, &password)
            .await
            .map_err(|e| format!("Password auth failed: {}", e))?,
        SshAuth::KeyFile { path, passphrase } => {
            let key = load_secret_key(&path, passphrase.as_deref())
                .map_err(|e| format!("Failed to load key file: {}", e))?;
            session
                .authenticate_publickey(&config.user, Arc::new(key))
                .await
                .map_err(|e| format!("Key auth failed: {}", e))?
        }
        SshAuth::KeyData { data, passphrase } => {
            let key = decode_secret_key(&data, passphrase.as_deref())
                .map_err(|e| format!("Failed to decode key data: {}", e))?;
            session
                .authenticate_publickey(&config.user, Arc::new(key))
                .await
                .map_err(|e| format!("Key auth failed: {}", e))?
        }
    };

    if !auth_result {
        return Err("Authentication failed".to_string());
    }

    Ok("Connection successful".to_string())
}
