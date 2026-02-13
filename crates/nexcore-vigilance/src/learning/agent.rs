//! Teachable agent with teacher trait system.

use super::models::{AgentSession, Intervention, Observation, ObservationEventType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

/// Agent configuration.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    /// Description
    pub description: String,
    /// Model identifier
    pub model: String,
}

/// Trait for implementing teachers that observe and intervene.
#[async_trait]
pub trait Teacher: Send + Sync {
    /// Called when an observation is recorded.
    async fn observe(&self, observation: Observation);

    /// Called when an intervention is triggered. Returns optional response.
    async fn intervene(&self, intervention: Intervention) -> Option<String>;
}

/// A teachable agent that can have teachers attached.
pub struct TeachableAgent {
    config: AgentConfig,
    session: Option<AgentSession>,
    teachers: HashMap<String, Box<dyn Teacher>>,
    teaching_enabled: bool,
}

impl TeachableAgent {
    /// Create a new teachable agent.
    #[must_use]
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            session: None,
            teachers: HashMap::new(),
            teaching_enabled: false,
        }
    }

    /// Enable teaching and register a teacher.
    pub fn enable_teaching(&mut self, id: &str, teacher: Box<dyn Teacher>) {
        self.teaching_enabled = true;
        self.teachers.insert(id.to_string(), teacher);
    }

    /// Check if teaching is enabled.
    #[must_use]
    pub fn is_teaching_enabled(&self) -> bool {
        self.teaching_enabled
    }

    /// Get the current session.
    #[must_use]
    pub fn session(&self) -> Option<&AgentSession> {
        self.session.as_ref()
    }

    /// Start a new learning session.
    ///
    /// # Errors
    /// Returns error if session notification fails.
    pub async fn start_session(&mut self) -> Result<String> {
        let session_id = format!("session_{}", nexcore_id::NexId::v4());
        let session = AgentSession {
            id: session_id.clone(),
            agent_name: self.config.name.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: "active".to_string(),
            todos: Vec::new(),
            metadata: HashMap::new(),
        };

        self.session = Some(session.clone());

        if self.teaching_enabled {
            self.notify_teachers(
                ObservationEventType::SessionStarted,
                serde_json::to_value(&session)?,
            )
            .await;
        }

        Ok(session_id)
    }

    /// Notify all teachers of an observation.
    pub async fn notify_teachers(&self, event_type: ObservationEventType, data: serde_json::Value) {
        let observation = Observation {
            id: nexcore_id::NexId::v4(),
            timestamp: Utc::now(),
            event_type,
            agent_id: self.config.name.clone(),
            session_id: self
                .session
                .as_ref()
                .map_or_else(String::new, |s| s.id.clone()),
            data,
            flags: Vec::new(),
        };

        for teacher in self.teachers.values() {
            teacher.observe(observation.clone()).await;
        }
    }
}
