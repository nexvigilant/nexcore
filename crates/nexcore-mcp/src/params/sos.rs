//! State Operating System (SOS) Parameters
//! Tier: T2-T3 (State Machine Specification and Execution)
//!
//! State machines, transitions, history, and temporal scheduling.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// State specification for SOS machine creation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosStateSpec {
    /// State name
    pub name: String,
    /// State kind: "initial", "normal", "terminal", or "error"
    pub kind: String,
}

/// Transition specification for SOS machine creation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosTransitionSpec {
    /// Source state name
    pub from: String,
    /// Target state name
    pub to: String,
    /// Event name triggering the transition
    pub event: String,
}

/// Parameters for creating a new state machine.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosCreateParams {
    /// Machine name
    pub name: String,
    /// List of state specifications
    pub states: Vec<SosStateSpec>,
    /// List of transition specifications
    pub transitions: Vec<SosTransitionSpec>,
}

/// Parameters for executing a state transition.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosTransitionParams {
    /// Machine ID
    pub machine_id: u64,
    /// Event name to trigger
    pub event: String,
}

/// Parameters for querying machine state.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosStateParams {
    /// Machine ID
    pub machine_id: u64,
}

/// Parameters for querying machine transition history.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosHistoryParams {
    /// Machine ID
    pub machine_id: u64,
    /// Maximum number of entries to return
    #[serde(default = "default_sos_history_limit")]
    pub limit: usize,
}

fn default_sos_history_limit() -> usize {
    50
}

/// Parameters for validating a machine specification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosValidateParams {
    /// Machine name
    pub name: String,
    /// List of state specifications
    pub states: Vec<SosStateSpec>,
    /// List of transition specifications
    pub transitions: Vec<SosTransitionSpec>,
}

/// Parameters for listing active machines.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosListParams {
    /// Filter pattern
    #[serde(default)]
    pub filter: Option<String>,
}

/// Parameters for cycle detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosCyclesParams {
    /// Machine ID
    pub machine_id: u64,
    /// Include self-loops in results
    #[serde(default = "default_sos_include_self_loops")]
    pub include_self_loops: bool,
}

fn default_sos_include_self_loops() -> bool {
    true
}

/// Parameters for irreversibility audit trail query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosAuditParams {
    /// Machine ID
    pub machine_id: u64,
    /// Maximum number of audit entries
    #[serde(default = "default_sos_audit_limit")]
    pub limit: usize,
}

fn default_sos_audit_limit() -> usize {
    100
}

/// Parameters for temporal scheduling operations.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosScheduleParams {
    /// Machine ID
    pub machine_id: u64,
    /// Event to schedule
    pub event: String,
    /// Delay in ticks before firing
    pub delay_ticks: u64,
}

/// Parameters for location-based routing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosRouteParams {
    /// Machine ID to route
    pub machine_id: u64,
    /// Target location ID
    #[serde(default)]
    pub location_id: Option<u64>,
}
