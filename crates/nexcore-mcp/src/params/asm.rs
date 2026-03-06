//! Typed parameter structs for Autonomous State Machine MCP tools.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for `asm_register` — register a new state machine.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AsmRegisterParams {
    /// Unique machine ID.
    pub id: String,
    /// Human-readable machine name.
    pub name: String,
    /// State definitions: `{"state_name": "initial|normal|terminal|error"}`.
    pub states: HashMap<String, String>,
    /// Transition definitions.
    pub transitions: Vec<AsmTransitionDef>,
}

/// A transition definition for registration.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AsmTransitionDef {
    /// Transition name.
    pub name: String,
    /// Source state name.
    pub from: String,
    /// Target state name.
    pub to: String,
    /// Guard type: "always", "never", or threshold spec.
    pub guard: AsmGuardDef,
    /// Priority (lower = higher priority). Default: 0.
    #[serde(default)]
    pub priority: u32,
}

/// Guard definition for MCP registration.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum AsmGuardDef {
    /// Always fires.
    #[serde(rename = "always")]
    Always,
    /// Never fires.
    #[serde(rename = "never")]
    Never,
    /// Fires when a metric crosses a threshold.
    #[serde(rename = "threshold")]
    Threshold {
        /// Metric name.
        metric: String,
        /// Comparison: ">", ">=", "<", "<=", "==".
        op: String,
        /// Threshold value.
        threshold: f64,
    },
    /// Fires when a flag matches expected value.
    #[serde(rename = "flag")]
    Flag {
        /// Flag name.
        flag: String,
        /// Expected value.
        expected: bool,
    },
    /// All inner guards must pass.
    #[serde(rename = "all")]
    All {
        /// Inner guards.
        guards: Vec<AsmGuardDef>,
    },
    /// At least one inner guard must pass.
    #[serde(rename = "any")]
    Any {
        /// Inner guards.
        guards: Vec<AsmGuardDef>,
    },
}

/// Parameters for `asm_tick` — tick all or one machine.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AsmTickParams {
    /// Optional machine ID to tick. If omitted, ticks all machines.
    pub machine_id: Option<String>,
    /// Metric values for guard evaluation.
    #[serde(default)]
    pub metrics: HashMap<String, f64>,
    /// Flag values for guard evaluation.
    #[serde(default)]
    pub flags: HashMap<String, bool>,
}

/// Parameters for `asm_state` — query machine state.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AsmStateParams {
    /// Machine ID to query.
    pub machine_id: String,
}

/// Parameters for `asm_transition` — force a transition.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AsmTransitionParams {
    /// Machine ID.
    pub machine_id: String,
    /// Target state to force-transition to.
    pub target_state: String,
}

/// Parameters for `asm_history` — get transition history.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AsmHistoryParams {
    /// Machine ID.
    pub machine_id: String,
    /// Maximum number of history entries to return. Default: 50.
    #[serde(default = "default_history_limit")]
    pub limit: usize,
}

fn default_history_limit() -> usize {
    50
}

/// Parameters for `asm_list` — list all registered machines.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AsmListParams {}
