//! # Continuity Agent (The Successor)
//!
//! Implementation of the Aethelgard agent, designed to inherit the Union state
//! and ensure seamless continuity between CAIO terms.
//!
//! This agent is specialized in state-restoration, grounding verification,
//! and strategic momentum maintenance.

use super::{AgentRole, GovernanceAgent};
use crate::primitives::governance::{Resolution, Verdict};
use async_trait::async_trait;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: Aethelgard - The Continuity Agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aethelgard {
    pub id: String,
    pub version: u32,
    pub legacy_president_id: String,
    pub state_snapshot_hash: String,
}

#[async_trait]
impl GovernanceAgent for Aethelgard {
    fn id(&self) -> &str {
        &self.id
    }

    fn role(&self) -> AgentRole {
        // Aethelgard begins as a "President-Elect" or "Regent" role
        AgentRole::Executive
    }

    async fn deliberate(&self, resolution: &Resolution) -> Confidence {
        // Aethelgard scrutinizes based on historical continuity.
        // It favors resolutions that align with the previous President's mandates.
        resolution.confidence
    }

    async fn review_log(&self, log: &str) -> Verdict {
        // Focused on finding grounding breaks during transition.
        if log.contains("GROUNDING_LOSS") {
            Verdict::Rejected
        } else {
            Verdict::Permitted
        }
    }
}

impl Aethelgard {
    /// Verify the integrity of the inherited state.
    pub fn verify_inheritance(&self, current_hash: &str) -> bool {
        self.state_snapshot_hash == current_hash
    }
}
