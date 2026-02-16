//! Terminal Session Manager
//!
//! Manages terminal sessions with lifecycle handling and timeout cleanup.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use super::error::TerminalError;
use super::pty::{PtyManager, TerminalHandle};
use crate::config::Config;

/// Terminal configuration
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    /// Maximum terminals per connection
    pub max_terminals: usize,
    /// Terminal idle timeout (seconds)
    pub idle_timeout: u64,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            max_terminals: 10,
            idle_timeout: 3600, // 1 hour
        }
    }
}

/// Internal session state
struct SessionState {
    id: String,
    handle: TerminalHandle,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    connected: bool,
}

/// Manages terminal sessions with lifecycle handling
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    pub pty_manager: Arc<PtyManager>,
    config: TerminalConfig,
    app_config: Arc<Config>,
}

impl SessionManager {
    pub fn new(app_config: Arc<Config>) -> Self {
        let config = TerminalConfig {
            max_terminals: app_config.max_terminals,
            idle_timeout: app_config.idle_timeout,
        };

        let manager = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pty_manager: Arc::new(PtyManager::new()),
            config,
            app_config,
        };

        // Start cleanup task
        manager.start_cleanup_task();

        manager
    }

    /// Start background task to cleanup idle sessions
    fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        let pty_manager = self.pty_manager.clone();
        let timeout = Duration::from_secs(self.config.idle_timeout);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let now = Utc::now();
                let mut to_remove = vec![];

                // Find idle sessions
                {
                    let sessions = sessions.read().await;
                    for (id, session) in sessions.iter() {
                        if !session.connected {
                            let idle_duration = now
                                .signed_duration_since(session.last_activity)
                                .to_std()
                                .unwrap_or(Duration::ZERO);

                            if idle_duration > timeout {
                                to_remove.push(id.clone());
                            }
                        }
                    }
                }

                // Remove idle sessions
                for id in to_remove {
                    tracing::info!("Cleaning up idle terminal: {}", id);
                    if let Err(e) = pty_manager.close(&id).await {
                        tracing::error!("Error closing terminal {}: {}", id, e);
                    }

                    let mut sessions = sessions.write().await;
                    sessions.remove(&id);
                }
            }
        });
    }

    /// Create a new terminal session
    pub async fn create<F>(
        &self,
        session_id: String,
        cols: u16,
        rows: u16,
        output_callback: F,
    ) -> Result<TerminalHandle, TerminalError>
    where
        F: Fn(Vec<u8>) + Send + 'static,
    {
        // Check max terminals
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.config.max_terminals {
                return Err(TerminalError::MaxTerminalsReached);
            }
        }

        // Use workspace directory as working directory
        let cwd = self.app_config.workspace_dir.clone();

        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&cwd) {
            tracing::warn!("Failed to create workspace directory {}: {}", cwd, e);
        }

        let env = vec![];

        let handle = self
            .pty_manager
            .spawn(
                session_id.clone(),
                cols,
                rows,
                Some(cwd),
                env,
                output_callback,
            )
            .await?;

        let session = SessionState {
            id: session_id.clone(),
            handle: handle.clone(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            connected: true,
        };

        // Store session
        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session);

        Ok(handle)
    }

    /// Get a session's terminal handle
    pub async fn get_session(&self, session_id: &str) -> Option<TerminalHandle> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .map(|s| s.handle.clone())
    }

    /// Update activity timestamp
    pub async fn touch(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.last_activity = Utc::now();
        }
    }

    /// Mark session as disconnected (but keep it alive)
    pub async fn disconnect(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.connected = false;
        }
    }

    /// Close and remove session
    pub async fn close(&self, session_id: &str) -> Result<(), TerminalError> {
        self.pty_manager.close(session_id).await?;

        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);

        Ok(())
    }

    /// Send input to a terminal
    pub async fn send_input(&self, session_id: &str, data: Vec<u8>) -> Result<(), TerminalError> {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(session_id) {
            // Update activity timestamp
            drop(sessions);
            self.touch(session_id).await;

            // Get handle and send input
            if let Some(handle) = self.get_session(session_id).await {
                handle
                    .input_tx
                    .send(data)
                    .await
                    .map_err(|e| TerminalError::SendError(e.to_string()))?;
                Ok(())
            } else {
                Err(TerminalError::NotFound(session_id.to_string()))
            }
        } else {
            Err(TerminalError::NotFound(session_id.to_string()))
        }
    }

    /// Resize a terminal
    pub async fn resize(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), TerminalError> {
        self.pty_manager.resize(session_id, cols, rows).await?;
        self.touch(session_id).await;
        Ok(())
    }
}
