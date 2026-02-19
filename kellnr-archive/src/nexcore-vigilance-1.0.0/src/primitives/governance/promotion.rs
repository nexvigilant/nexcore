//! # Agent Promotion System
//!
//! Implementation of the Agent_Promotion_Score algorithm.
//! Used for evaluating candidates for executive roles within the Union.

use serde::{Deserialize, Serialize};

/// T2-C: PromotionCriteria - The dimensions of agent performance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PromotionCriteria {
    pub strategic_alignment: u8,   // 0-25
    pub capability_breadth: u8,    // 0-20
    pub execution_reliability: u8, // 0-20
    pub integration_depth: u8,     // 0-15
    pub scalability_potential: u8, // 0-10
    pub autonomy_maturity: u8,     // 0-10
}

/// T2-P: AgentRank - The classification level of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AgentRank {
    Probationary,
    Standard,
    Senior,
    CeaEligible,
}

impl PromotionCriteria {
    /// Calculate the total promotion score (0-100).
    pub fn total_score(&self) -> u8 {
        self.strategic_alignment
            .saturating_add(self.capability_breadth)
            .saturating_add(self.execution_reliability)
            .saturating_add(self.integration_depth)
            .saturating_add(self.scalability_potential)
            .saturating_add(self.autonomy_maturity)
    }

    /// Determine the eligible rank.
    pub fn eligibility(&self) -> AgentRank {
        let score = self.total_score();
        if score >= 75 {
            AgentRank::CeaEligible
        } else if score >= 60 {
            AgentRank::Senior
        } else if score >= 40 {
            AgentRank::Standard
        } else {
            AgentRank::Probationary
        }
    }
}
