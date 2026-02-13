//! # PV Control Loop Implementation
//!
//! Pharmacovigilance-specific instantiation of the generic control loop.
//!
//! ## Domain Translation
//!
//! | Generic | PV Instantiation |
//! |---------|------------------|
//! | SENSE | ICSR intake, literature monitoring, trial data |
//! | COMPARE | Signal detection (PRR, ROR, IC, EBGM vs thresholds) |
//! | CONTROL | Causality assessment → action selection |
//! | ACTUATE | Label update, REMS, DHPC, withdrawal |
//! | FEEDBACK | Ongoing surveillance monitoring |

use super::{ControlError, ControlLoop, LoopMetrics, LoopStatus};
use crate::pv::signals::evaluate_signal_complete;
use crate::pv::thresholds::SignalCriteria;
use crate::pv::types::{CompleteSignalResult, ContingencyTable};
use serde::{Deserialize, Serialize};

// =============================================================================
// PV-Specific Types
// =============================================================================

/// PV safety state (analogous to spacecraft position/velocity)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PvSafetyState {
    /// Number of cases (a)
    pub case_count: u64,
    /// PRR value
    pub prr: f64,
    /// ROR value
    pub ror: f64,
    /// ROR lower 95% CI
    pub ror_lower_ci: f64,
    /// Information Component
    pub ic: f64,
    /// IC 2.5th percentile
    pub ic025: f64,
    /// EBGM
    pub ebgm: f64,
    /// EB05 (5th percentile)
    pub eb05: f64,
    /// Chi-square statistic
    pub chi_square: f64,
    /// Overall signal detected
    pub signal_detected: bool,
}

impl PvSafetyState {
    /// Create from 2x2 contingency table
    #[must_use]
    pub fn from_contingency(a: u64, b: u64, c: u64, d: u64) -> Self {
        let table = ContingencyTable { a, b, c, d };
        let criteria = SignalCriteria::evans();
        let result = evaluate_signal_complete(&table, &criteria);
        Self::from_complete_result(&result)
    }

    /// Create from complete signal result
    #[must_use]
    pub fn from_complete_result(result: &CompleteSignalResult) -> Self {
        // Evans signal = PRR >= 2.0 AND chi² >= 3.841 AND n >= 3
        let signal_detected = result.prr.is_signal && result.chi_square >= 3.841 && result.n >= 3;

        Self {
            case_count: u64::from(result.n),
            prr: result.prr.point_estimate,
            ror: result.ror.point_estimate,
            ror_lower_ci: result.ror.lower_ci,
            ic: result.ic.point_estimate,
            ic025: result.ic.lower_ci,
            ebgm: result.ebgm.point_estimate,
            eb05: result.ebgm.lower_ci,
            chi_square: result.chi_square,
            signal_detected,
        }
    }
}

/// Signal strength / error magnitude
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PvSignalStrength {
    /// PRR exceeds threshold
    pub prr_signal: bool,
    /// ROR lower CI > 1.0
    pub ror_signal: bool,
    /// IC025 > 0
    pub ic_signal: bool,
    /// EB05 >= 2.0
    pub ebgm_signal: bool,
    /// Chi-square >= 3.841
    pub chi_signal: bool,
    /// Count of positive signals
    pub signal_count: u8,
    /// Overall severity
    pub severity: SignalConfidence,
}

impl PvSignalStrength {
    /// Compute from state and thresholds
    #[must_use]
    pub fn compute(state: &PvSafetyState, thresholds: &PvThresholds) -> Self {
        let prr_signal = state.prr >= thresholds.prr_threshold;
        let ror_signal = state.ror_lower_ci > 1.0;
        let ic_signal = state.ic025 > 0.0;
        let ebgm_signal = state.eb05 >= thresholds.ebgm_threshold;
        let chi_signal = state.chi_square >= thresholds.chi_threshold;

        let signal_count = u8::from(prr_signal)
            + u8::from(ror_signal)
            + u8::from(ic_signal)
            + u8::from(ebgm_signal)
            + u8::from(chi_signal);

        let severity = match signal_count {
            5 => SignalConfidence::Critical,
            4 => SignalConfidence::High,
            2..=3 => SignalConfidence::Medium,
            1 => SignalConfidence::Low,
            _ => SignalConfidence::None,
        };

        Self {
            prr_signal,
            ror_signal,
            ic_signal,
            ebgm_signal,
            chi_signal,
            signal_count,
            severity,
        }
    }
}

/// Risk action (analogous to burn commands)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskAction {
    /// No action needed
    ContinueMonitoring,
    /// Log for routine PSUR
    RoutinePsurUpdate,
    /// Urgent label update required
    UrgentLabelUpdate,
    /// Issue DHPC (Dear Healthcare Professional Communication)
    IssueDhpc,
    /// Implement REMS
    ImplementRems,
    /// Emergency market withdrawal
    EmergencyWithdrawal,
}

impl RiskAction {
    /// Get action priority (higher = more urgent)
    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::ContinueMonitoring => 0,
            Self::RoutinePsurUpdate => 1,
            Self::UrgentLabelUpdate => 2,
            Self::IssueDhpc => 3,
            Self::ImplementRems => 4,
            Self::EmergencyWithdrawal => 5,
        }
    }
}

/// PV signal detection confidence from multi-metric concordance.
///
/// Tier: T2-P (κ + ∂ — comparison with boundary)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalConfidence {
    /// No signal
    #[default]
    None,
    /// Low confidence signal
    Low,
    /// Medium confidence signal
    Medium,
    /// High confidence signal
    High,
    /// Critical signal requiring immediate action
    Critical,
}

/// Backward-compatible alias.
#[deprecated(note = "use SignalConfidence — F2 equivocation fix")]
pub type Severity = SignalConfidence;

/// Detection thresholds (setpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvThresholds {
    /// PRR threshold (default: 2.0, Evans)
    pub prr_threshold: f64,
    /// EBGM threshold (default: 2.0)
    pub ebgm_threshold: f64,
    /// Chi-square threshold (default: 3.841, p<0.05)
    pub chi_threshold: f64,
    /// Minimum case count
    pub min_cases: u64,
}

impl Default for PvThresholds {
    fn default() -> Self {
        Self {
            prr_threshold: 2.0,
            ebgm_threshold: 2.0,
            chi_threshold: 3.841,
            min_cases: 3,
        }
    }
}

// =============================================================================
// PV Control Loop
// =============================================================================

/// Pharmacovigilance control loop implementation.
///
/// Translates the aerospace GNC pattern to drug safety surveillance:
/// - SENSE: Case intake, literature, trials
/// - COMPARE: Signal detection algorithms
/// - CONTROL: Action selection based on severity
/// - ACTUATE: Risk minimization measures
/// - FEEDBACK: Ongoing monitoring
#[derive(Debug)]
pub struct PvControlLoop {
    /// Current contingency data (a, b, c, d)
    contingency: (u64, u64, u64, u64),
    /// Detection thresholds
    thresholds: PvThresholds,
    /// Current state
    current_state: PvSafetyState,
    /// Last action taken
    last_action: Option<RiskAction>,
    /// Metrics
    metrics: LoopMetrics,
}

impl Default for PvControlLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl PvControlLoop {
    /// Create a new PV control loop with default thresholds
    #[must_use]
    pub fn new() -> Self {
        Self {
            contingency: (0, 0, 0, 0),
            thresholds: PvThresholds::default(),
            current_state: PvSafetyState::default(),
            last_action: None,
            metrics: LoopMetrics::default(),
        }
    }

    /// Create with custom thresholds
    #[must_use]
    pub fn with_thresholds(thresholds: PvThresholds) -> Self {
        Self {
            thresholds,
            ..Self::new()
        }
    }

    /// Update contingency data (new cases received)
    pub fn update_contingency(&mut self, a: u64, b: u64, c: u64, d: u64) {
        self.contingency = (a, b, c, d);
    }

    /// Get current metrics
    #[must_use]
    pub fn metrics(&self) -> &LoopMetrics {
        &self.metrics
    }

    /// Get last action taken
    #[must_use]
    pub fn last_action(&self) -> Option<&RiskAction> {
        self.last_action.as_ref()
    }

    /// Run one tick with current contingency data
    pub fn tick_with_data(
        &mut self,
        a: u64,
        b: u64,
        c: u64,
        d: u64,
    ) -> Result<PvSafetyState, ControlError> {
        self.update_contingency(a, b, c, d);
        self.tick(&self.thresholds.clone())
    }
}

impl ControlLoop for PvControlLoop {
    type State = PvSafetyState;
    type Action = RiskAction;
    type Error = PvSignalStrength;
    type Target = PvThresholds;

    /// SENSE: Compute signal metrics from contingency data
    fn sense(&self) -> Self::State {
        let (a, b, c, d) = self.contingency;
        PvSafetyState::from_contingency(a, b, c, d)
    }

    /// COMPARE: Check signals against thresholds
    fn compare(&self, current: &Self::State, target: &Self::Target) -> Self::Error {
        PvSignalStrength::compute(current, target)
    }

    /// CONTROL: Select action based on signal severity
    fn control(&self, error: &Self::Error) -> Self::Action {
        match error.severity {
            SignalConfidence::Critical => RiskAction::EmergencyWithdrawal,
            SignalConfidence::High => {
                if error.signal_count >= 4 {
                    RiskAction::ImplementRems
                } else {
                    RiskAction::UrgentLabelUpdate
                }
            }
            SignalConfidence::Medium => RiskAction::IssueDhpc,
            SignalConfidence::Low => RiskAction::RoutinePsurUpdate,
            SignalConfidence::None => RiskAction::ContinueMonitoring,
        }
    }

    /// ACTUATE: Record action (actual execution is external)
    fn actuate(&mut self, action: Self::Action) -> Result<(), ControlError> {
        self.last_action = Some(action.clone());
        self.metrics.iterations += 1;

        if action != RiskAction::ContinueMonitoring {
            self.metrics.actions_taken += 1;
        }

        // Update status based on action
        self.metrics.status = match action {
            RiskAction::EmergencyWithdrawal | RiskAction::ImplementRems => LoopStatus::Critical,
            RiskAction::UrgentLabelUpdate | RiskAction::IssueDhpc => LoopStatus::Warning,
            _ => LoopStatus::Nominal,
        };

        Ok(())
    }

    /// FEEDBACK: Return updated state
    fn feedback(&self) -> Self::State {
        self.current_state.clone()
    }

    /// Override tick to update current_state
    fn tick(&mut self, target: &Self::Target) -> Result<Self::State, ControlError> {
        let current = self.sense();
        self.current_state = current.clone();

        let error = self.compare(&current, target);
        self.metrics.current_error_magnitude = f64::from(error.signal_count);

        let action = self.control(&error);
        self.actuate(action)?;

        Ok(self.feedback())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pv_control_loop_no_signal() {
        let mut pv_loop = PvControlLoop::new();

        // No signal: a=1, background proportional
        let result = pv_loop.tick_with_data(1, 100, 50, 5000);

        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(!state.signal_detected);
        assert_eq!(pv_loop.last_action(), Some(&RiskAction::ContinueMonitoring));
    }

    #[test]
    fn test_pv_control_loop_critical_signal() {
        let mut pv_loop = PvControlLoop::new();

        // Strong signal: high PRR, ROR, IC, EBGM
        let result = pv_loop.tick_with_data(50, 100, 10, 10000);

        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(state.signal_detected);
        assert!(state.prr > 2.0);

        // Should recommend severe action
        let action = pv_loop.last_action();
        assert!(
            action == Some(&RiskAction::EmergencyWithdrawal)
                || action == Some(&RiskAction::ImplementRems)
                || action == Some(&RiskAction::UrgentLabelUpdate)
        );
    }

    #[test]
    fn test_pv_safety_state_from_contingency() {
        let state = PvSafetyState::from_contingency(15, 100, 20, 10000);

        assert!(state.prr > 0.0);
        assert!(state.ror > 0.0);
        assert!(state.case_count == 15);
    }

    #[test]
    fn test_pv_thresholds_default() {
        let thresholds = PvThresholds::default();

        assert!((thresholds.prr_threshold - 2.0).abs() < f64::EPSILON);
        assert!((thresholds.chi_threshold - 3.841).abs() < f64::EPSILON);
        assert_eq!(thresholds.min_cases, 3);
    }
}
