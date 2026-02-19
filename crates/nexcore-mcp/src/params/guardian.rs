//! Guardian Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Homeostasis control loop, risk evaluation, and anomaly sensing.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for running a homeostasis loop tick
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianTickParams {}

/// Parameters for evaluating PV risk
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianEvaluatePvParams {
    /// Drug name
    pub drug: String,
    /// Adverse event name
    pub event: String,
    /// PRR (Proportional Reporting Ratio) value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// IC 2.5th percentile (Information Component)
    pub ic025: f64,
    /// EB05 (EBGM 5th percentile)
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
}

/// Parameters for getting homeostasis status
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianStatusParams {}

/// Parameters for injecting a test signal into Guardian
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianInjectSignalParams {
    /// Signal source: "external" (PAMP), "internal" (DAMP), or "pv"
    pub source: String,
    /// Signal pattern/type identifier
    pub pattern: String,
    /// Severity score (0.0 - 1.0)
    pub severity: f64,
    /// Optional context message
    #[serde(default)]
    pub context: Option<String>,
}

/// Parameters for listing Guardian sensors
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianSensorsListParams {}

/// Parameters for listing Guardian actuators
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianActuatorsListParams {}

/// Parameters for getting Guardian event history
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianHistoryParams {
    /// Maximum number of events to return (default: 50)
    #[serde(default = "default_history_limit")]
    pub limit: usize,
    /// Filter by event type: "signal", "action", "all" (default)
    #[serde(default = "default_history_filter")]
    pub filter: String,
}

fn default_history_limit() -> usize {
    50
}

fn default_history_filter() -> String {
    "all".to_string()
}

/// Parameters for subscribing to Guardian events
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianSubscribeParams {
    /// Event types to subscribe to: "signals", "actions", "all"
    #[serde(default = "default_subscribe_filter")]
    pub events: String,
}

fn default_subscribe_filter() -> String {
    "all".to_string()
}

/// Parameters for setting input for the adversarial prompt sensor
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdversarialSensorInputParams {
    /// Text input to analyze for statistical fingerprints
    pub text: String,
}

/// Parameters for adversarial decision boundary probing.
///
/// Takes a baseline PV signal input and generates systematic perturbations
/// near each metric's decision threshold to test decision robustness.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdversarialDecisionProbeParams {
    /// Baseline PRR value
    pub prr: f64,
    /// Baseline ROR lower CI value
    pub ror_lower: f64,
    /// Baseline IC025 value
    pub ic025: f64,
    /// Baseline EB05 value
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
    /// Number of perturbation steps per metric (default: 5)
    #[serde(default)]
    pub perturbation_steps: Option<usize>,
}

// ============================================================================
// {G, V, R} Framework Parameters (Autonomy-Aware Risk Assessment)
// ============================================================================

/// Parameters for classifying an entity by {G,V,R} capabilities
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianOriginatorClassifyParams {
    /// Entity has Goal-selection capability
    pub has_goal_selection: bool,
    /// Entity has Value-evaluation capability
    pub has_value_evaluation: bool,
    /// Entity has Refusal-capacity capability
    pub has_refusal_capacity: bool,
}

/// Parameters for getting autonomy-aware ceiling limits
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianCeilingForOriginatorParams {
    /// Originator type: "tool", "agent_with_r", "agent_with_vr", "agent_with_gr", "agent_with_gvr"
    pub originator_type: String,
}

/// Parameters for PV control loop tick
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvControlLoopTickParams {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

/// Parameters for FDA data bridge evaluation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaBridgeEvaluateParams {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

/// Parameters for FDA data bridge batch evaluation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaBridgeBatchParams {
    /// List of contingency tables as [a, b, c, d] arrays
    pub tables: Vec<[u64; 4]>,
}

/// Parameters for computing 3D safety space point
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianSpace3DComputeParams {
    pub prr: f64,
    pub ror_lower: f64,
    pub ic025: f64,
    pub eb05: f64,
    pub n: u64,
    /// Originator type (default: "tool")
    #[serde(default = "default_originator")]
    pub originator: String,
    /// Harm type
    #[serde(default)]
    pub harm_type: Option<String>,
    /// Hierarchy level (1-8, default 4)
    #[serde(default = "default_hierarchy_level")]
    pub hierarchy_level: u8,
    /// Number of signal metrics present (default 4)
    #[serde(default = "default_signal_metrics")]
    pub signal_metrics_present: usize,
}

fn default_originator() -> String {
    "tool".to_string()
}

fn default_hierarchy_level() -> u8 {
    4
}

fn default_signal_metrics() -> usize {
    4
}

// ============================================================================
// Governance Parameters (Phase 4: Head of State)
// ============================================================================

/// Parameters for granting governance consent
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianConsentGrantParams {
    /// Scope: "patient-safety", "system-health", "hud-governance", "access-control", "data-integrity", "global"
    pub scope: String,
    /// Description of what is being consented to
    #[serde(default = "default_consent_description")]
    pub description: String,
}

fn default_consent_description() -> String {
    "Guardian homeostasis monitoring".to_string()
}

/// Parameters for revoking governance consent
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianConsentRevokeParams {
    /// Consent ID to revoke
    pub consent_id: String,
    /// Reason for revocation
    pub reason: String,
}

/// Parameters for viewing governance consent status
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianConsentStatusParams {}

/// Parameters for creating an authority delegation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianDelegationCreateParams {
    /// Scope: "patient-safety", "system-health", "hud-governance", "access-control", "data-integrity", "global"
    pub scope: String,
    /// Entity receiving delegated authority
    #[serde(default = "default_delegate")]
    pub delegate: String,
    /// Originator type of delegate: "tool", "agent_with_r", "agent_with_vr", "agent_with_gvr"
    #[serde(default = "default_delegate_originator")]
    pub delegate_originator: String,
}

fn default_delegate() -> String {
    "guardian-engine".to_string()
}

fn default_delegate_originator() -> String {
    "agent_with_vr".to_string()
}

/// Parameters for running a legitimacy check on a proposed action
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianLegitimacyCheckParams {
    /// Actor performing the action
    #[serde(default = "default_actor")]
    pub actor: String,
    /// Scope of the action
    pub scope: String,
    /// Evidence summary
    #[serde(default = "default_evidence_summary")]
    pub evidence_summary: String,
    /// Whether this is a P0 patient safety emergency
    #[serde(default)]
    pub is_p0_emergency: bool,
}

fn default_actor() -> String {
    "guardian-engine".to_string()
}

fn default_evidence_summary() -> String {
    "homeostasis tick".to_string()
}

// ============================================================================
// Forensics Parameters (Phase 5: Journal & Audit)
// ============================================================================

/// Parameters for querying the governance journal
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianJournalQueryParams {
    /// Filter by scope (optional)
    #[serde(default)]
    pub scope: Option<String>,
    /// Filter by actor (optional)
    #[serde(default)]
    pub actor: Option<String>,
    /// Maximum entries to return (default: 50)
    #[serde(default = "default_journal_limit")]
    pub limit: usize,
}

fn default_journal_limit() -> usize {
    50
}

/// Parameters for getting journal statistics
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianJournalStatsParams {}

/// Parameters for running a governance audit
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianGovernanceAuditParams {}

// ============================================================================
// Guardian Actuator Inspection Parameters
// ============================================================================

/// Parameters for listing currently blocked targets
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianBlockedListParams {}

/// Parameters for unblocking a target
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianUnblockParams {
    /// Target identifier to unblock
    pub target: String,
}

/// Parameters for listing open escalations
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianEscalationsParams {}

/// Parameters for acknowledging an escalation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianAcknowledgeEscalationParams {
    /// Escalation ID to acknowledge
    pub escalation_id: String,
}

/// Parameters for listing quarantined items
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianQuarantinedListParams {}

/// Parameters for adjusting the decision engine threshold
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianSetThresholdParams {
    /// New threshold value (0.0-100.0). Lower = more sensitive.
    pub threshold: f64,
}
