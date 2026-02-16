//! SSH client for remote terminal connections

use async_trait::async_trait;
use russh::*;
use russh_keys::*;
use std::sync::Arc;

/// SSH authentication method
#[derive(Debug, Clone)]
pub enum SshAuth {
    Password(String),
    KeyFile { path: String, passphrase: Option<String> },
    KeyData { data: String, passphrase: Option<String> },
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

/// SSH terminal session
pub struct SshSession {
    session: client::Handle<ClientHandler>,
    channel: Channel<client::Msg>,
}

impl SshSession {
    /// Connect to SSH server and authenticate
    pub async fn connect(config: SshConfig) -> Result<Self, String> {
        let russh_config = client::Config::default();
        let config_arc = Arc::new(russh_config);
        
        let addr = format!("{}:{}", config.host, config.port);
        
        let mut session = client::connect(config_arc, &addr, ClientHandler)
            .await
            .map_err(|e| format!("SSH connection failed: {}", e))?;

        // Authenticate
        let auth_result = match config.auth {
            SshAuth::Password(password) => {
                session
                    .authenticate_password(&config.user, &password)
                    .await
                    .map_err(|e| format!("Password auth failed: {}", e))?
            }
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

        // Open a channel
        let channel = session
            .channel_open_session()
            .await
            .map_err(|e| format!("Failed to open channel: {}", e))?;

        Ok(Self { session, channel })
    }

    /// Request a PTY and start a shell
    pub async fn request_pty(&mut self, cols: u32, rows: u32) -> Result<(), String> {
        self.channel
            .request_pty(
                false,
                "xterm-256color",
                cols,
                rows,
                0,
                0,
                &[],
            )
            .await
            .map_err(|e| format!("PTY request failed: {}", e))?;

        self.channel
            .request_shell(false)
            .await
            .map_err(|e| format!("Shell request failed: {}", e))?;

        Ok(())
    }

    /// Resize the PTY
    pub async fn resize(&mut self, cols: u32, rows: u32) -> Result<(), String> {
        self.channel
            .window_change(cols, rows, 0, 0)
            .await
            .map_err(|e| format!("Resize failed: {}", e))?;
        Ok(())
    }

    /// Write data to the channel
    pub async fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.channel
            .data(data)
            .await
            .map_err(|e| format!("Write failed: {}", e))?;
        Ok(())
    }

    /// Wait for output data
    pub async fn read(&mut self) -> Option<Vec<u8>> {
        loop {
            match self.channel.wait().await {
                Some(ChannelMsg::Data { data }) => return Some(data.to_vec()),
                Some(ChannelMsg::ExtendedData { data, .. }) => return Some(data.to_vec()),
                Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => return None,
                _ => continue, // Skip other messages
            }
        }
    }

    /// Close the session
    pub async fn close(self) {
        let _ = self.channel.eof().await;
        let _ = self.channel.close().await;
    }
}

/// Test SSH connection without opening a shell
pub async fn test_connection(config: SshConfig) -> Result<String, String> {
    let session = SshSession::connect(config).await?;
    session.close().await;
    Ok("Connection successful".to_string())
}
