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

/// Internal session state
struct SessionState {
    #[allow(dead_code)]
    id: String,
    handle: TerminalHandle,
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    connected: bool,
}

/// Manages terminal sessions with lifecycle handling
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    pty_manager: Arc<PtyManager>,
    max_terminals: usize,
    idle_timeout: u64,
    app_config: Arc<Config>,
}

impl SessionManager {
    pub fn new(app_config: Arc<Config>) -> Self {
        let manager = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pty_manager: Arc::new(PtyManager::new()),
            max_terminals: app_config.max_terminals,
            idle_timeout: app_config.idle_timeout,
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
        let timeout = Duration::from_secs(self.idle_timeout);

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
    pub async fn create_terminal(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
        output_callback: Box<dyn Fn(String) + Send + 'static>,
    ) -> Result<TerminalHandle, TerminalError> {
        // Check max terminals
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.max_terminals {
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

        // Wrap the string callback to work with Vec<u8>
        let byte_callback = move |data: Vec<u8>| {
            if let Ok(s) = String::from_utf8(data) {
                output_callback(s);
            }
        };

        let handle = self
            .pty_manager
            .spawn(
                session_id.to_string(),
                cols,
                rows,
                Some(cwd),
                env,
                byte_callback,
            )
            .await?;

        let session = SessionState {
            id: session_id.to_string(),
            handle: handle.clone(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            connected: true,
        };

        // Store session
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), session);

        Ok(handle)
    }

    /// Write input to a terminal
    pub async fn write_to_terminal(&self, session_id: &str, input: &str) -> Result<(), TerminalError> {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(session_id) {
            let handle = session.handle.clone();
            drop(sessions);
            
            // Update activity
            self.touch(session_id).await;

            handle
                .input_tx
                .send(input.as_bytes().to_vec())
                .await
                .map_err(|e| TerminalError::SendError(e.to_string()))?;
            Ok(())
        } else {
            Err(TerminalError::NotFound(session_id.to_string()))
        }
    }

    /// Resize a terminal
    pub async fn resize_terminal(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), TerminalError> {
        self.pty_manager.resize(session_id, cols, rows).await?;
        self.touch(session_id).await;
        Ok(())
    }

    /// Close a terminal
    pub async fn close_terminal(&self, session_id: &str) {
        if let Err(e) = self.pty_manager.close(session_id).await {
            tracing::warn!("Error closing terminal {}: {}", session_id, e);
        }

        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }

    /// Update activity timestamp
    async fn touch(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.last_activity = Utc::now();
        }
    }
}
