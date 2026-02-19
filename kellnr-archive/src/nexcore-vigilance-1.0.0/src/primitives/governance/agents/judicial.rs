//! # Judicial Agents
//!
//! Implementation of Justices and Linters.

use super::{AgentRole, GovernanceAgent};
use crate::primitives::governance::{Resolution, Verdict};
use async_trait::async_trait;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: Jurist - A judicial reviewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jurist {
    pub id: String,
    pub rigor_threshold: f64,
}

#[async_trait]
impl GovernanceAgent for Jurist {
    fn id(&self) -> &str {
        &self.id
    }

    fn role(&self) -> AgentRole {
        AgentRole::Jurist
    }

    async fn deliberate(&self, resolution: &Resolution) -> Confidence {
        // Jurists are very skeptical of low rigor
        if resolution.confidence.value() < self.rigor_threshold {
            Confidence::new(0.1)
        } else {
            resolution.confidence
        }
    }

    async fn review_log(&self, log: &str) -> Verdict {
        // Jurists look for Heresy in logs
        if log.contains("BYPASS_TYPE_SAFETY") {
            Verdict::Rejected
        } else {
            Verdict::Permitted
        }
    }
}
