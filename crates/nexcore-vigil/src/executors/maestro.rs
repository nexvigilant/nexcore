//! MaestroExecutor — dispatches tasks to Maestro sessions via REST API.
//!
//! Connects Vigil's event-driven orchestration to Maestro's multi-session
//! PTY management. When Vigil decides to dispatch work, this executor
//! finds or creates an idle Maestro session and writes the prompt.

use async_trait::async_trait;
use serde::Deserialize;

use crate::executors::Executor;
use crate::models::{ExecutorResult, ExecutorType};

/// Response from Maestro's session creation endpoint.
#[derive(Debug, Deserialize)]
struct SessionResponse {
    id: u32,
    status: String,
}

/// Executor that dispatches tasks to Maestro PTY sessions via HTTP.
pub struct MaestroExecutor {
    base_url: String,
    client: reqwest::Client,
}

impl MaestroExecutor {
    /// Create a new executor pointing at a Maestro REST API.
    /// Default: `http://localhost:7070`
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Find an idle session, or return None.
    async fn find_idle_session(&self) -> Option<u32> {
        let url = format!("{}/sessions", self.base_url);
        let resp = self.client.get(&url).send().await.ok()?;
        let sessions: Vec<SessionResponse> = resp.json().await.ok()?;

        sessions.iter().find(|s| s.status == "Idle").map(|s| s.id)
    }

    /// Create a new session for the given project.
    async fn create_session(&self, project_path: &str, mode: &str) -> nexcore_error::Result<u32> {
        let url = format!("{}/sessions", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "mode": mode,
                "project_path": project_path,
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            nexcore_error::bail!("Failed to create session: {}", resp.status());
        }

        let session: SessionResponse = resp.json().await?;
        Ok(session.id)
    }

    /// Write a prompt to a session's PTY.
    async fn dispatch_prompt(&self, session_id: u32, prompt: &str) -> nexcore_error::Result<()> {
        let url = format!("{}/sessions/{}/execute", self.base_url, session_id);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "prompt": prompt }))
            .send()
            .await?;

        if !resp.status().is_success() {
            nexcore_error::bail!(
                "Failed to execute in session {}: {}",
                session_id,
                resp.status()
            );
        }

        Ok(())
    }
}

#[async_trait]
impl Executor for MaestroExecutor {
    fn executor_type(&self) -> ExecutorType {
        ExecutorType::Maestro
    }

    async fn execute(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> nexcore_error::Result<ExecutorResult> {
        let project_path = params
            .get("project_path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let mode = params
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("claude");

        // Find idle session or create new one
        let session_id = match self.find_idle_session().await {
            Some(id) => id,
            None => self.create_session(project_path, mode).await?,
        };

        // Dispatch the action as a prompt
        self.dispatch_prompt(session_id, action).await?;

        Ok(ExecutorResult {
            executor: ExecutorType::Maestro,
            success: true,
            output: Some(format!("Dispatched to session {session_id}")),
            error: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }
}
