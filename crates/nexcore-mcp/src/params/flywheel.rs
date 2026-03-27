//! Flywheel Loop Engine Parameters
//!
//! Typed parameter structs for flywheel_vitals and flywheel_cascade MCP tools.
//!
//! ## T1 Primitive Grounding: ς (State) + κ (Comparison) + ∂ (Boundary)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ============================================================================
// Tool 1: flywheel_vitals — FlywheelVitals field overrides (all optional)
// ============================================================================

/// Parameters for flywheel_vitals. All fields optional — omit to use defaults (0.0 / 0).
///
/// Corresponds to the 15-field `FlywheelVitals` struct grouped by loop.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelVitalsParams {
    // ── Loop 1: Rim Integrity ────────────────────────────────────────────
    /// Value density score (default 0.0).
    pub value_density: Option<f64>,
    /// Churn rate (default 0.0).
    pub churn_rate: Option<f64>,
    /// Switching cost index (default 0.0).
    pub switching_cost_index: Option<f64>,

    // ── Loop 2: Momentum Conservation ────────────────────────────────────
    /// Knowledge base growth rate (default 0.0).
    pub knowledge_base_growth: Option<f64>,
    /// Execution velocity (default 0.0).
    pub execution_velocity: Option<f64>,
    /// System momentum (default 0.0).
    pub momentum: Option<f64>,

    // ── Loop 3: Friction Dissipation ─────────────────────────────────────
    /// Automation coverage fraction [0.0, 1.0] (default 0.0).
    pub automation_coverage: Option<f64>,
    /// Manual touchpoints count (default 0).
    pub manual_touchpoints: Option<u32>,
    /// Overhead ratio (default 0.0).
    pub overhead_ratio: Option<f64>,

    // ── Loop 4: Gyroscopic Stability ─────────────────────────────────────
    /// Mission alignment score (default 0.0).
    pub mission_alignment_score: Option<f64>,
    /// Scope creep incident count (default 0).
    pub scope_creep_incidents: Option<u32>,
    /// Pivot resistance score (default 0.0).
    pub pivot_resistance: Option<f64>,

    // ── Loop 5: Elastic Equilibrium ──────────────────────────────────────
    /// Contributor load factor (default 0.0).
    pub contributor_load: Option<f64>,
    /// Fatigue cycle count (default 0).
    pub fatigue_cycle_count: Option<u32>,
    /// Recovery time in days (default 0.0).
    pub recovery_time_days: Option<f64>,
}

// ============================================================================
// Tool 2: flywheel_cascade — CascadeInput (flattened sub-loop inputs)
// ============================================================================

/// Parameters for flywheel_cascade — runs the full five-loop interaction cascade.
///
/// Cascade order: Loop 5 (Elastic) → Loop 1 (Rim) → Loop 3 (Friction) → Loop 2 (Momentum) → Loop 4 (Gyroscopic).
/// The friction net_drain is automatically fed into momentum; the momentum L is automatically fed into gyroscopic.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelCascadeParams {
    // ── Loop 1: Rim Integrity inputs ─────────────────────────────────────
    /// Tensile strength of the value network rim.
    pub rim_tensile_strength: f64,
    /// Centrifugal force pulling the rim outward.
    pub rim_centrifugal_force: f64,

    // ── Loop 2: Momentum Conservation inputs ─────────────────────────────
    /// System inertia (mass analog). L = inertia × omega − friction_drain.
    pub momentum_inertia: f64,
    /// Angular velocity (ω).
    pub momentum_omega: f64,
    /// Initial friction drain before Loop 3 contribution (usually 0.0).
    #[serde(default)]
    pub momentum_friction_drain: f64,

    // ── Loop 3: Friction Dissipation inputs ──────────────────────────────
    /// Count of manual processes (contact friction factor).
    pub friction_manual_processes: f64,
    /// Count of human touchpoints (contact friction factor).
    pub friction_human_touchpoints: f64,
    /// System velocity for aerodynamic (cubic) drag computation.
    pub friction_velocity: f64,
    /// Automation coverage fraction [0.0, 1.0].
    pub friction_automation_coverage: f64,

    // ── Loop 4: Gyroscopic Stability inputs ──────────────────────────────
    /// Initial angular momentum L (overridden in cascade by momentum result).
    pub gyroscopic_momentum_l: f64,
    /// External perturbation torque magnitude.
    pub gyroscopic_perturbation_torque: f64,
    /// Minimum L required before gyroscopic stabilization activates.
    pub gyroscopic_critical_momentum: f64,

    // ── Loop 5: Elastic Equilibrium inputs ───────────────────────────────
    /// Applied stress on system contributors.
    pub elastic_stress: f64,
    /// Yield point (stress at or above which permanent deformation occurs).
    pub elastic_yield_point: f64,
    /// Current fatigue cycle count.
    pub elastic_fatigue_cycles: u32,
    /// Maximum fatigue cycles before failure (0 = use threshold default of 1000).
    #[serde(default)]
    pub elastic_fatigue_limit: u32,
}

// ============================================================================
// Tool 3: flywheel_reality — Cascade + VDAG Reality Gradient
// ============================================================================

/// Parameters for flywheel_reality — cascade with VDAG evidence grading.
///
/// All cascade fields are required (same as FlywheelCascadeParams).
/// Optional VDAG fields control the goal and loop weights.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelRealityParams {
    // ── Cascade inputs (same as FlywheelCascadeParams) ─────────────────
    pub rim_tensile_strength: f64,
    pub rim_centrifugal_force: f64,
    pub momentum_inertia: f64,
    pub momentum_omega: f64,
    #[serde(default)]
    pub momentum_friction_drain: f64,
    pub friction_manual_processes: f64,
    pub friction_human_touchpoints: f64,
    pub friction_velocity: f64,
    pub friction_automation_coverage: f64,
    pub gyroscopic_momentum_l: f64,
    pub gyroscopic_perturbation_torque: f64,
    pub gyroscopic_critical_momentum: f64,
    pub elastic_stress: f64,
    pub elastic_yield_point: f64,
    pub elastic_fatigue_cycles: u32,
    #[serde(default)]
    pub elastic_fatigue_limit: u32,

    // ── VDAG goal fields (all optional) ────────────────────────────────
    /// Goal description (default: "All loops healthy").
    pub goal_description: Option<String>,
    /// Target system state: "thriving", "stressed", "critical", "failed" (default: "thriving").
    pub target_state: Option<String>,
    /// Weight for rim loop [0.0-1.0] (default: 0.2).
    pub rim_weight: Option<f64>,
    /// Weight for momentum loop [0.0-1.0] (default: 0.2).
    pub momentum_weight: Option<f64>,
    /// Weight for friction loop [0.0-1.0] (default: 0.2).
    pub friction_weight: Option<f64>,
    /// Weight for gyroscopic loop [0.0-1.0] (default: 0.2).
    pub gyroscopic_weight: Option<f64>,
    /// Weight for elastic loop [0.0-1.0] (default: 0.2).
    pub elastic_weight: Option<f64>,
}

// ============================================================================
// Tool 4a: flywheel_evaluate_live — Live Metrics → VDAG Reality Gradient
// ============================================================================

/// Parameters for flywheel_evaluate_live — feed live system metrics directly.
///
/// Maps Guardian, Immunity, and session observables into the 5-loop cascade
/// via [`nexcore_flywheel::live::LiveMetrics`], then grades with VDAG.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelEvaluateLiveParams {
    // -- Guardian surface --
    /// Registered Guardian sensors (from guardian_status).
    pub sensor_count: Option<u32>,
    /// Registered Guardian actuators (from guardian_status).
    pub actuator_count: Option<u32>,
    /// Guardian iteration count.
    pub guardian_iterations: Option<u32>,
    /// Signals detected in last homeostasis tick.
    pub signals_detected: Option<u32>,
    /// Actions taken in last homeostasis tick.
    pub actions_taken: Option<u32>,

    // -- Immunity surface --
    /// Total antibodies in the registry.
    pub antibody_count: Option<u32>,
    /// Critical-severity antibodies.
    pub critical_antibodies: Option<u32>,

    // -- Session surface --
    /// Tool calls in current session.
    pub tool_calls: Option<u32>,
    /// Commits in current session.
    pub commits: Option<u32>,
    /// Files modified in current session.
    pub files_modified: Option<u32>,
    /// Total sessions in brain.db.
    pub total_sessions: Option<u32>,
    /// Sessions in last 24 hours.
    pub sessions_last_24h: Option<u32>,

    // -- Automation surface --
    /// Active hooks count.
    pub active_hooks: Option<u32>,
    /// Total skills count.
    pub skill_count: Option<u32>,
    /// Automation coverage ratio (0.0-1.0).
    pub automation_coverage: Option<f64>,

    // -- Goal overrides --
    /// Goal description (default: "All loops healthy").
    pub goal_description: Option<String>,
    /// Target state: "thriving", "stressed", "critical", "failed" (default: "thriving").
    pub target_state: Option<String>,
}

// ============================================================================
// Tool 4: flywheel_learn — Learning Loop Analysis
// ============================================================================

/// Parameters for flywheel_learn — analyze cascade history for learning insights.
///
/// Pass an array of past CascadeRecord JSON objects (timestamp_ms, cascade, reality).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelLearnParams {
    /// Array of CascadeRecord JSON objects from prior evaluations.
    pub history: Vec<serde_json::Value>,
}

// ============================================================================
// Tool 6: flywheel_evaluate_extended — Live Metrics + Extension Loops
// ============================================================================

/// Parameters for flywheel_evaluate_extended — full 8-loop evaluation.
///
/// Combines LiveMetrics (5 core loops) with extension loops (trust, immunity,
/// skill maturation) for a comprehensive Reality Gradient score.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelEvaluateExtendedParams {
    // -- Core LiveMetrics (same as evaluate_live) --
    /// Registered Guardian sensors.
    pub sensor_count: Option<u32>,
    /// Registered Guardian actuators.
    pub actuator_count: Option<u32>,
    /// Guardian iteration count.
    pub guardian_iterations: Option<u32>,
    /// Signals detected in last homeostasis tick.
    pub signals_detected: Option<u32>,
    /// Actions taken in last homeostasis tick.
    pub actions_taken: Option<u32>,
    /// Total antibodies in the registry.
    pub antibody_count: Option<u32>,
    /// Critical-severity antibodies.
    pub critical_antibodies: Option<u32>,
    /// Tool calls in current session.
    pub tool_calls: Option<u32>,
    /// Commits in current session.
    pub commits: Option<u32>,
    /// Files modified in current session.
    pub files_modified: Option<u32>,
    /// Total sessions in brain.db.
    pub total_sessions: Option<u32>,
    /// Sessions in last 24 hours.
    pub sessions_last_24h: Option<u32>,
    /// Active hooks count.
    pub active_hooks: Option<u32>,
    /// Total skills count.
    pub skill_count: Option<u32>,
    /// Automation coverage ratio (0.0-1.0).
    pub automation_coverage: Option<f64>,

    // -- Trust extension --
    /// Global trust score (0.0-1.0).
    pub trust_score: Option<f64>,
    /// Number of verified operations.
    pub verified_operations: Option<u32>,
    /// Trust violations detected.
    pub trust_violations: Option<u32>,

    // -- Immunity extension --
    /// PAMP antibody count.
    pub pamp_count: Option<u32>,
    /// DAMP antibody count.
    pub damp_count: Option<u32>,

    // -- Skill maturation extension --
    /// Diamond-level skills count.
    pub diamond_skills: Option<u32>,
    /// Skill enforcement ratio (0.0-1.0).
    pub enforcement_ratio: Option<f64>,
}
