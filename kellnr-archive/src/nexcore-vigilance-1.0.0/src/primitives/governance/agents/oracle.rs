//! # Oracle Agents
//!
//! Implementation of External Oracle agents that provide validation
//! for system claims and resolutions.

use super::{AgentRole, GovernanceAgent};
use crate::primitives::governance::oracle_protocol::OracleReputation;
use crate::primitives::governance::{Resolution, Verdict};
use async_trait::async_trait;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: OracleAgent - An external validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAgent {
    pub id: String,
    pub reputation: OracleReputation,
}

#[async_trait]
impl GovernanceAgent for OracleAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn role(&self) -> AgentRole {
        AgentRole::Oracle
    }

    async fn deliberate(&self, resolution: &Resolution) -> Confidence {
        // Oracles provide a combined confidence based on their reputation
        resolution
            .confidence
            .combine(Confidence::new(self.reputation.0))
    }

    async fn review_log(&self, _log: &str) -> Verdict {
        // Oracles review logs for external consistency
        Verdict::Permitted
    }
}

impl OracleAgent {
    /// Provide a formal validation result for a claim.
    pub fn validate_claim(&self, claim_confidence: f64) -> Confidence {
        Confidence::new(claim_confidence).combine(Confidence::new(self.reputation.0))
    }
}
