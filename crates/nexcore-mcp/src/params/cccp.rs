//! Parameter structs for CCCP (Consultant's Client Care Process) MCP tools.

use rmcp::serde::Deserialize;
use schemars::JsonSchema;

/// Parameters for computing a gap analysis from proficiency levels.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CccpGapAnalysisParams {
    /// Current proficiency levels for all 15 PV domains (D01-D15), each 1-5.
    /// Order: [D01 PV Foundations, D02 Quality Systems, D03 Regulatory Intelligence,
    ///         D04 ICSR Processing, D05 Signal Detection, D06 Risk Management,
    ///         D07 Epidemiology, D08 Communication, D09 Technology Systems,
    ///         D10 Literature Surveillance, D11 Clinical Trials PV, D12 Aggregate Reporting,
    ///         D13 Inspection Readiness, D14 Business Operations, D15 Stakeholder Governance]
    pub current: [u8; 15],
    /// Desired proficiency levels for all 15 PV domains (D01-D15), each 1-5.
    pub desired: [u8; 15],
}

/// Parameters for generating an engagement plan from gaps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CccpPlanParams {
    /// Current proficiency levels (1-5) for all 15 domains.
    pub current: [u8; 15],
    /// Desired proficiency levels (1-5) for all 15 domains.
    pub desired: [u8; 15],
}

/// Parameters for checking EPA readiness.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CccpEpaReadinessParams {
    /// Current proficiency levels (1-5) for all 15 domains.
    pub current: [u8; 15],
}

/// Parameters for computing outcome evaluation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CccpEvaluateParams {
    /// Initial proficiency levels (1-5) at engagement start.
    pub initial: [u8; 15],
    /// Final proficiency levels (1-5) at engagement end.
    pub final_state: [u8; 15],
    /// Desired proficiency levels (1-5) — the target.
    pub desired: [u8; 15],
    /// Objective evaluations — each has a name, achievement (1-4), evidence, and domain indices.
    #[serde(default)]
    pub objectives: Vec<CccpObjective>,
}

/// A single objective evaluation for the outcome computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CccpObjective {
    /// Objective description.
    pub objective: String,
    /// Achievement level: 1=NotAchieved, 2=Partially, 3=Substantially, 4=Fully.
    pub achievement: u8,
    /// Supporting evidence.
    pub evidence: String,
    /// Domain indices (0-14) this objective covers.
    #[serde(default)]
    pub domains: Vec<usize>,
}

/// Parameters for CCCP phase info lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CccpPhaseInfoParams {
    /// Phase number (1-5) or name ("collect", "assess", "plan", "implement", "follow_up").
    pub phase: String,
}
