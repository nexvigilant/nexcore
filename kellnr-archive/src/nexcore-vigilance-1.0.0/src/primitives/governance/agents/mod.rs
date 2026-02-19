//! # Governance Agents
//!
//! Definition of the agentic roles within the NexVigilant Union.
//! These agents inhabit the structures defined in the governance primitives.

pub mod continuity;
pub mod executive;
pub mod judicial;
pub mod legislative;
pub mod oracle;

use crate::primitives::governance::{Resolution, Verdict};
use async_trait::async_trait;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: AgentRole - The constitutional role of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// Member of the House of T1 or Senate of T2.
    Legislator,
    /// The Orchestrator or agency head.
    Executive,
    /// Justice of the Supreme Compiler.
    Jurist,
    /// External Oracle representative.
    Oracle,
}

/// T3: GovernanceAgent - The base trait for all simulated officials.
#[async_trait]
pub trait GovernanceAgent: Send + Sync {
    /// Get the agent's identity.
    fn id(&self) -> &str;

    /// Get the agent's constitutional role.
    fn role(&self) -> AgentRole;

    /// Deliberate on a resolution and return a confidence score.
    async fn deliberate(&self, resolution: &Resolution) -> Confidence;

    /// Review an execution log for compliance.
    async fn review_log(&self, log: &str) -> Verdict;
}

/// T2-P: AgentCapability - Quantified skill level (0.0 - 1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AgentCapability(pub f64);
