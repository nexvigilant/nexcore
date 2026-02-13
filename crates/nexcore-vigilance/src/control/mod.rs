//! # Control Loop Abstraction
//!
//! Generic T2-C (Cross-Domain Composite) control loop that transfers across ALL safety-critical domains.
//!
//! ## The Universal Pattern
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    CONTROL LOOP (T2-C)                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │   ┌──────────┐      ┌───────────┐      ┌──────────┐            │
//! │   │  SENSE   │ ───► │  COMPUTE  │ ───► │ ACTUATE  │            │
//! │   │          │      │ (compare/ │      │          │            │
//! │   │          │      │  control) │      │          │            │
//! │   └──────────┘      └───────────┘      └──────────┘            │
//! │        ▲                                      │                 │
//! │        │              FEEDBACK                │                 │
//! │        └──────────────────────────────────────┘                 │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Domain Instantiations
//!
//! | Domain | Sense | Compute | Actuate | Feedback |
//! |--------|-------|---------|---------|----------|
//! | **Aerospace** | IMU, star trackers | GNC algorithms | Engines, gimbals | Telemetry |
//! | **Pharmacovigilance** | ICSRs, literature | Signal detection | Label updates, REMS | Surveillance |
//! | **Cybersecurity** | IDS, logs | Threat analysis | Firewall rules | Monitoring |
//! | **Manufacturing** | Sensors | SPC algorithms | Process adjustments | Inspection |
//!
//! ## Transfer Confidence
//!
//! - Structural: 0.95 (same components across domains)
//! - Functional: 0.95 (same purpose: maintain setpoint via correction)
//! - Contextual: 0.85 (timescales differ: milliseconds vs months)
//! - **Overall: 0.92**
//!
//! ## Example
//!
//! ```ignore
//! use nexcore_vigilance::control::{ControlLoop, PvControlLoop};
//!
//! let mut pv_loop = PvControlLoop::new();
//!
//! // Run one iteration
//! let state = pv_loop.tick();
//! println!("Current safety state: {:?}", state);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod fda_bridge;
pub mod pv;

pub use fda_bridge::{ContingencyInput, FaersBridge, FdaBridgeResult, FdaDataBridge};
pub use pv::{
    PvControlLoop, PvSafetyState, PvSignalStrength, PvThresholds, RiskAction,
    SignalConfidence as PvSeverity,
};

// =============================================================================
// Generic Control Loop Trait (T2-C Pattern)
// =============================================================================

/// Error type for control loop operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlError {
    /// Error category
    pub category: String,
    /// Error message
    pub message: String,
    /// Recoverable flag
    pub recoverable: bool,
}

impl std::fmt::Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.category, self.message)
    }
}

impl std::error::Error for ControlError {}

/// The universal control loop pattern.
///
/// This is a T2-C (Cross-Domain Composite) that transfers across ALL safety-critical domains.
/// It combines the following T2-P primitives:
/// - **Feedback**: Output affecting input
/// - **Gain**: Amplification ratio
/// - **Latency**: Time delay between cause/effect
///
/// # Type Parameters
///
/// - `S`: State type (what we're measuring)
/// - `A`: Action type (what we can do)
/// - `E`: Error/deviation type (difference from target)
///
/// # Transfer Confidence
///
/// | Dimension | Score | Rationale |
/// |-----------|-------|-----------|
/// | Structural | 0.95 | Same components: sensor, comparator, controller, actuator, plant |
/// | Functional | 0.95 | Same purpose: maintain setpoint via error-driven correction |
/// | Contextual | 0.85 | Timescales differ (milliseconds in aerospace, months in PV) |
/// | **Overall** | **0.92** | High-confidence cross-domain transfer |
pub trait ControlLoop {
    /// State type: What the system measures
    type State: Clone + Debug;

    /// Action type: What the system can do
    type Action: Clone + Debug;

    /// Error type: Deviation from target
    type Error: Clone + Debug;

    /// Target state for the system
    type Target: Clone + Debug;

    // =========================================================================
    // Core Control Loop Methods
    // =========================================================================

    /// **SENSE**: Collect current state from environment.
    ///
    /// Aerospace: Read IMU, star trackers, radar
    /// PV: Aggregate ICSRs, literature, trials
    fn sense(&self) -> Self::State;

    /// **COMPARE**: Calculate deviation from target.
    ///
    /// Aerospace: Position/velocity error
    /// PV: Signal strength vs. detection threshold
    fn compare(&self, current: &Self::State, target: &Self::Target) -> Self::Error;

    /// **CONTROL**: Determine corrective action based on error.
    ///
    /// Aerospace: Burn commands, attitude corrections
    /// PV: Label updates, REMS, withdrawal decisions
    fn control(&self, error: &Self::Error) -> Self::Action;

    /// **ACTUATE**: Execute the corrective action.
    ///
    /// Aerospace: Fire engines, adjust gimbals
    /// PV: Submit label change, issue DHPC
    fn actuate(&mut self, action: Self::Action) -> Result<(), ControlError>;

    /// **FEEDBACK**: Observe effect of action, update state.
    ///
    /// Aerospace: Read new telemetry
    /// PV: Monitor ongoing surveillance
    fn feedback(&self) -> Self::State;

    // =========================================================================
    // Composite Operations
    // =========================================================================

    /// Run one complete iteration of the control loop.
    ///
    /// This is the fundamental cybernetic cycle:
    /// SENSE → COMPARE → CONTROL → ACTUATE → FEEDBACK
    fn tick(&mut self, target: &Self::Target) -> Result<Self::State, ControlError> {
        // 1. SENSE: Get current state
        let current = self.sense();

        // 2. COMPARE: Calculate error
        let error = self.compare(&current, target);

        // 3. CONTROL: Determine action
        let action = self.control(&error);

        // 4. ACTUATE: Execute action
        self.actuate(action)?;

        // 5. FEEDBACK: Return new state
        Ok(self.feedback())
    }

    /// Get the system's current state without taking action.
    fn current_state(&self) -> Self::State {
        self.sense()
    }
}

// =============================================================================
// Loop Iteration Result
// =============================================================================

/// Result of a control loop iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopResult<S, A, E> {
    /// Iteration number
    pub iteration: u64,
    /// Timestamp (Unix epoch ms)
    pub timestamp_ms: u64,
    /// State before action
    pub state_before: S,
    /// Error/deviation computed
    pub error: E,
    /// Action taken
    pub action: A,
    /// State after action
    pub state_after: S,
    /// Duration of tick (ms)
    pub duration_ms: u64,
}

// =============================================================================
// Control Loop Metrics
// =============================================================================

/// Metrics for control loop performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoopMetrics {
    /// Total iterations
    pub iterations: u64,
    /// Actions taken
    pub actions_taken: u64,
    /// Errors encountered
    pub errors: u64,
    /// Average tick duration (ms)
    pub avg_duration_ms: f64,
    /// Current error magnitude
    pub current_error_magnitude: f64,
    /// System status
    pub status: LoopStatus,
}

/// Loop operational status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoopStatus {
    /// Normal operation
    #[default]
    Nominal,
    /// Warning threshold exceeded
    Warning,
    /// Critical threshold exceeded
    Critical,
    /// Loop is stopped
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple test implementation
    struct TestLoop {
        state: f64,
    }

    impl ControlLoop for TestLoop {
        type State = f64;
        type Action = f64;
        type Error = f64;
        type Target = f64;

        fn sense(&self) -> Self::State {
            self.state
        }

        fn compare(&self, current: &Self::State, target: &Self::Target) -> Self::Error {
            target - current
        }

        fn control(&self, error: &Self::Error) -> Self::Action {
            // Simple proportional control
            error * 0.5
        }

        fn actuate(&mut self, action: Self::Action) -> Result<(), ControlError> {
            self.state += action;
            Ok(())
        }

        fn feedback(&self) -> Self::State {
            self.state
        }
    }

    #[test]
    fn test_simple_control_loop() {
        let mut loop_ctrl = TestLoop { state: 0.0 };
        let target = 10.0;

        // Run several iterations
        for _ in 0..10 {
            let result = loop_ctrl.tick(&target);
            assert!(result.is_ok());
        }

        // State should approach target
        let final_state = loop_ctrl.current_state();
        assert!(
            (final_state - target).abs() < 0.1,
            "State {} should approach target {}",
            final_state,
            target
        );
    }
}
