//! Security Posture Assessment Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Compliance scorecards, threat readiness, and regulatory gap analysis.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for security posture assessment.
///
/// Evaluates a system against compliance frameworks (SOC 2, HIPAA, GDPR,
/// EU AI Act) and returns a scored posture report.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecurityPostureAssessParams {
    /// Target system or project to assess
    pub target: String,
    /// Compliance frameworks to check against (default: all)
    /// Options: "soc2", "hipaa", "gdpr", "eu_ai_act", "nist", "iso27001"
    #[serde(default)]
    pub frameworks: Option<Vec<String>>,
    /// Known security controls already in place (reduces gap count)
    #[serde(default)]
    pub existing_controls: Option<Vec<String>>,
}

/// Parameters for threat readiness scoring.
///
/// Scores readiness against AI-specific threats: data poisoning,
/// model inversion, adversarial attacks, prompt injection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecurityThreatReadinessParams {
    /// Target system
    pub target: String,
    /// Specific threats to assess (default: all AI threats)
    /// Options: "data_poisoning", "model_inversion", "adversarial", "prompt_injection", "supply_chain"
    #[serde(default)]
    pub threats: Option<Vec<String>>,
    /// Known defenses already deployed
    #[serde(default)]
    pub defenses: Option<Vec<String>>,
}

/// Parameters for compliance gap analysis.
///
/// Compares current controls against a target framework and
/// identifies gaps with remediation recommendations.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecurityComplianceGapParams {
    /// Target compliance framework
    /// Options: "soc2", "hipaa", "gdpr", "eu_ai_act", "nist", "iso27001"
    pub framework: String,
    /// Controls already implemented (will be checked off)
    #[serde(default)]
    pub implemented_controls: Option<Vec<String>>,
}
