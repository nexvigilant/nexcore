//! # FDA Data Bridge
//!
//! Connects FDA FAERS data to the PvControlLoop for automated signal detection
//! and action recommendation.
//!
//! ## Architecture
//!
//! ```text
//! OpenFDA API → Contingency Table → PvControlLoop → RiskAction
//!     ↓              ↓                   ↓              ↓
//!  FAERS data     2x2 table         Signal eval    Recommended action
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_vigilance::control::{FdaDataBridge, RiskAction};
//!
//! let bridge = FdaDataBridge::new();
//! let result = bridge.evaluate_drug_event(50, 1000, 100, 50000);
//! assert_eq!(result.action, RiskAction::UrgentLabelUpdate);
//! ```

use super::ControlError;
use super::pv::{PvControlLoop, PvSafetyState, PvThresholds, RiskAction, SignalConfidence};
use serde::{Deserialize, Serialize};

// =============================================================================
// FDA Data Bridge
// =============================================================================

/// FDA Data Bridge evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FdaBridgeResult {
    /// Input contingency values
    pub contingency: ContingencyInput,
    /// Computed safety state
    pub safety_state: PvSafetyState,
    /// Signal severity
    pub severity: SignalConfidence,
    /// Recommended regulatory action
    pub action: RiskAction,
    /// Action priority (0-5, higher = more urgent)
    pub action_priority: u8,
    /// Human-readable summary
    pub summary: String,
}

/// Input contingency table values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContingencyInput {
    /// Drug+Event cases
    pub a: u64,
    /// Drug, no Event
    pub b: u64,
    /// Event, no Drug
    pub c: u64,
    /// Neither Drug nor Event
    pub d: u64,
}

/// FDA Data Bridge - connects FAERS data to PV Control Loop
#[derive(Debug)]
pub struct FdaDataBridge {
    loop_instance: PvControlLoop,
}

impl Default for FdaDataBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl FdaDataBridge {
    /// Create with default (Evans) thresholds
    #[must_use]
    pub fn new() -> Self {
        Self {
            loop_instance: PvControlLoop::new(),
        }
    }

    /// Create with custom thresholds
    #[must_use]
    pub fn with_thresholds(thresholds: PvThresholds) -> Self {
        Self {
            loop_instance: PvControlLoop::with_thresholds(thresholds),
        }
    }

    /// Evaluate a drug-event pair from contingency table values
    ///
    /// # Arguments
    /// * `a` - Cases with drug AND event
    /// * `b` - Cases with drug but NOT event
    /// * `c` - Cases with event but NOT drug
    /// * `d` - Cases with neither drug nor event
    ///
    /// # Returns
    /// Complete evaluation result with action recommendation
    ///
    /// # Errors
    /// Returns `ControlError` if the control loop tick fails
    pub fn evaluate(
        &mut self,
        a: u64,
        b: u64,
        c: u64,
        d: u64,
    ) -> Result<FdaBridgeResult, ControlError> {
        // Run one tick of the control loop
        let state = self.loop_instance.tick_with_data(a, b, c, d)?;

        // Get the recommended action
        let action = self
            .loop_instance
            .last_action()
            .cloned()
            .unwrap_or(RiskAction::ContinueMonitoring);

        // Determine severity from metrics
        let severity = self.compute_severity(&state);

        Ok(FdaBridgeResult {
            contingency: ContingencyInput { a, b, c, d },
            safety_state: state.clone(),
            severity: severity.clone(),
            action: action.clone(),
            action_priority: action.priority(),
            summary: self.generate_summary(&state, &severity, &action),
        })
    }

    /// Batch evaluation of multiple drug-event pairs
    ///
    /// # Errors
    /// Returns error if any evaluation fails
    pub fn evaluate_batch(
        &mut self,
        pairs: Vec<(u64, u64, u64, u64)>,
    ) -> Result<Vec<FdaBridgeResult>, ControlError> {
        pairs
            .into_iter()
            .map(|(a, b, c, d)| self.evaluate(a, b, c, d))
            .collect()
    }

    /// Compute severity from safety state
    fn compute_severity(&self, state: &PvSafetyState) -> SignalConfidence {
        let mut signal_count = 0u8;

        if state.prr >= 2.0 {
            signal_count += 1;
        }
        if state.ror_lower_ci > 1.0 {
            signal_count += 1;
        }
        if state.ic025 > 0.0 {
            signal_count += 1;
        }
        if state.eb05 >= 2.0 {
            signal_count += 1;
        }
        if state.chi_square >= 3.841 {
            signal_count += 1;
        }

        match signal_count {
            5 => SignalConfidence::Critical,
            4 => SignalConfidence::High,
            2..=3 => SignalConfidence::Medium,
            1 => SignalConfidence::Low,
            _ => SignalConfidence::None,
        }
    }

    /// Generate human-readable summary
    fn generate_summary(
        &self,
        state: &PvSafetyState,
        severity: &SignalConfidence,
        action: &RiskAction,
    ) -> String {
        let signal_status = if state.signal_detected {
            "SIGNAL DETECTED"
        } else {
            "No signal"
        };

        let action_str = match action {
            RiskAction::ContinueMonitoring => "Continue routine monitoring",
            RiskAction::RoutinePsurUpdate => "Include in next PSUR",
            RiskAction::UrgentLabelUpdate => "Urgent label update required",
            RiskAction::IssueDhpc => "Issue Dear Healthcare Provider Communication",
            RiskAction::ImplementRems => "Implement Risk Evaluation and Mitigation Strategy",
            RiskAction::EmergencyWithdrawal => "EMERGENCY: Consider market withdrawal",
        };

        format!(
            "{signal_status} | SignalConfidence: {severity:?} | PRR: {:.2} | ROR: {:.2} | Cases: {} | Action: {action_str}",
            state.prr, state.ror, state.case_count
        )
    }

    /// Get the control loop metrics
    #[must_use]
    pub fn metrics(&self) -> &super::LoopMetrics {
        self.loop_instance.metrics()
    }
}

// =============================================================================
// FAERS Bridge (async wrapper for HTTP integration)
// =============================================================================

/// Async FAERS bridge for direct OpenFDA integration
///
/// This struct is designed for use in the MCP layer where async HTTP
/// calls fetch data from OpenFDA before evaluation.
#[derive(Debug)]
pub struct FaersBridge {
    inner: FdaDataBridge,
}

impl Default for FaersBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl FaersBridge {
    /// Create new FAERS bridge
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: FdaDataBridge::new(),
        }
    }

    /// Evaluate from pre-fetched contingency counts
    ///
    /// Used after `get_contingency_counts()` in faers.rs fetches data
    ///
    /// # Errors
    /// Returns `ControlError` if evaluation fails
    pub fn evaluate_counts(
        &mut self,
        a: u64,
        b: u64,
        c: u64,
        d: u64,
    ) -> Result<FdaBridgeResult, ControlError> {
        self.inner.evaluate(a, b, c, d)
    }

    /// Get inner bridge for customization
    #[must_use]
    pub fn inner(&self) -> &FdaDataBridge {
        &self.inner
    }

    /// Get mutable inner bridge
    pub fn inner_mut(&mut self) -> &mut FdaDataBridge {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fda_bridge_no_signal() {
        let mut bridge = FdaDataBridge::new();

        // Low PRR, no signal expected
        let result = bridge
            .evaluate(2, 1000, 500, 100_000)
            .expect("should evaluate");

        assert!(!result.safety_state.signal_detected);
        assert_eq!(result.action, RiskAction::ContinueMonitoring);
        assert_eq!(result.action_priority, 0);
    }

    #[test]
    fn test_fda_bridge_strong_signal() {
        let mut bridge = FdaDataBridge::new();

        // Strong signal: high PRR, high chi-square
        let result = bridge
            .evaluate(50, 100, 20, 50_000)
            .expect("should evaluate");

        assert!(result.safety_state.signal_detected);
        assert!(result.safety_state.prr > 2.0);
        assert!(result.action_priority >= 2); // At least urgent label update
    }

    #[test]
    fn test_fda_bridge_batch() {
        let mut bridge = FdaDataBridge::new();

        let pairs = vec![
            (10, 1000, 100, 100_000), // Marginal
            (50, 100, 20, 50_000),    // Strong signal
            (1, 5000, 2000, 200_000), // No signal
        ];

        let results = bridge.evaluate_batch(pairs).expect("batch should work");

        assert_eq!(results.len(), 3);
        assert!(results[1].safety_state.signal_detected); // Strong signal case
        assert!(!results[2].safety_state.signal_detected); // No signal case
    }

    #[test]
    fn test_fda_bridge_severity_levels() {
        let mut bridge = FdaDataBridge::new();

        // Critical signal (all 5 algorithms positive)
        let critical = bridge
            .evaluate(100, 50, 10, 50_000)
            .expect("should evaluate");
        assert!(matches!(
            critical.severity,
            SignalConfidence::High | SignalConfidence::Critical
        ));

        // No signal
        let none = bridge
            .evaluate(1, 10_000, 5_000, 500_000)
            .expect("should evaluate");
        assert!(matches!(
            none.severity,
            SignalConfidence::None | SignalConfidence::Low
        ));
    }

    #[test]
    fn test_faers_bridge_wrapper() {
        let mut bridge = FaersBridge::new();

        let result = bridge
            .evaluate_counts(30, 500, 100, 80_000)
            .expect("should work");

        assert!(result.contingency.a == 30);
        assert!(!result.summary.is_empty());
    }
}
