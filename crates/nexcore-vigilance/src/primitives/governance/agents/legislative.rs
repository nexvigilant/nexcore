//! # Legislative Agents
//!
//! Implementation of Representatives and Senators.

use super::{AgentRole, GovernanceAgent};
use crate::primitives::governance::{Resolution, Verdict, VoteWeight};
use async_trait::async_trait;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: Legislator - A member of Congress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Legislator {
    pub id: String,
    pub weight: VoteWeight,
    pub focus_domain: Option<String>,
}

#[async_trait]
impl GovernanceAgent for Legislator {
    fn id(&self) -> &str {
        &self.id
    }

    fn role(&self) -> AgentRole {
        AgentRole::Legislator
    }

    async fn deliberate(&self, resolution: &Resolution) -> Confidence {
        // Simulation: If focus domain matches, increase scrutiny.
        let base_confidence = resolution.confidence.value();
        Confidence::new(base_confidence * 0.9) // Legislators are naturally skeptical
    }

    async fn review_log(&self, _log: &str) -> Verdict {
        // Legislators review logs primarily for budget/resource usage
        Verdict::Permitted
    }
}

impl Legislator {
    /// Cast a vote based on deliberation.
    pub fn cast_vote(&self, deliberation: Confidence) -> bool {
        deliberation.value() > 0.5
    }
}
