//! # Energy Balance (First Law of Thermodynamics)
//!
//! **Closed System**: ΔU = Q - W
//! **Open System**: dE/dt = Q̇ - Ẇ + Σṁ_in·h_in - Σṁ_out·h_out
//!
//! The First Law establishes energy conservation: energy cannot be created
//! or destroyed, only transferred or converted between forms.
//!
//! ## PV Application
//!
//! Energy balance maps to case processing workflows:
//! - **Internal Energy (U)**: Backlog state (cases pending)
//! - **Heat (Q)**: External case influx (spontaneous reports)
//! - **Work (W)**: Cases processed/resolved
//! - **Enthalpy Flow (ṁh)**: Inter-system case transfers
//!
//! ## Cross-Domain Transfer (Confidence: 0.85)
//!
//! | Thermodynamics | PV System | Software |
//! |----------------|-----------|----------|
//! | Internal energy U | Pending case count | Queue depth |
//! | Heat input Q | Incoming reports | Event ingestion |
//! | Work output W | Resolved cases | Processed requests |
//! | Enthalpy flow ṁh | Inter-agency transfers | Message passing |

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors for energy balance calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum EnergyBalanceError {
    /// Time step must be positive.
    #[error("Time step must be positive")]
    InvalidTimeStep,

    /// Mass flow rate cannot be negative.
    #[error("Mass flow rate must be non-negative")]
    NegativeMassFlow,

    /// Energy values produced invalid result.
    #[error("Energy balance calculation produced invalid result: {0}")]
    InvalidResult(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// FIRST LAW: CLOSED SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

/// First Law of Thermodynamics for closed systems.
///
/// ΔU = Q - W
///
/// Where:
/// - ΔU: Change in internal energy (J)
/// - Q: Heat added to system (J, positive = heat in)
/// - W: Work done by system (J, positive = work out)
///
/// # Arguments
///
/// * `heat_in` - Heat transferred into the system (J)
/// * `work_out` - Work done by the system (J)
///
/// # Returns
///
/// Change in internal energy ΔU (J)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::first_law_closed;
///
/// // 100 J heat in, 40 J work out -> 60 J increase in internal energy
/// let delta_u = first_law_closed(100.0, 40.0);
/// assert!((delta_u - 60.0).abs() < 0.001);
/// ```
#[must_use]
pub fn first_law_closed(heat_in: f64, work_out: f64) -> f64 {
    heat_in - work_out
}

/// Closed system energy balance result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedSystemBalance {
    /// Initial internal energy (J)
    pub u_initial: f64,
    /// Heat added to system (J)
    pub heat_in: f64,
    /// Work done by system (J)
    pub work_out: f64,
    /// Change in internal energy (J)
    pub delta_u: f64,
    /// Final internal energy (J)
    pub u_final: f64,
    /// Energy balance satisfied (conservation check)
    pub is_balanced: bool,
}

impl ClosedSystemBalance {
    /// Calculate closed system energy balance.
    ///
    /// # Arguments
    ///
    /// * `u_initial` - Initial internal energy (J)
    /// * `heat_in` - Heat added to system (J)
    /// * `work_out` - Work done by system (J)
    #[must_use]
    pub fn calculate(u_initial: f64, heat_in: f64, work_out: f64) -> Self {
        let delta_u = first_law_closed(heat_in, work_out);
        let u_final = u_initial + delta_u;

        Self {
            u_initial,
            heat_in,
            work_out,
            delta_u,
            u_final,
            is_balanced: true, // By definition, First Law always holds
        }
    }

    /// Verify energy conservation within tolerance.
    ///
    /// Checks: |U_final - (U_initial + Q - W)| < tolerance
    #[must_use]
    pub fn verify(&self, tolerance: f64) -> bool {
        let expected = self.u_initial + self.heat_in - self.work_out;
        (self.u_final - expected).abs() < tolerance
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FIRST LAW: OPEN SYSTEM (CONTROL VOLUME)
// ═══════════════════════════════════════════════════════════════════════════

/// Mass flow stream with enthalpy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassFlowStream {
    /// Mass flow rate (kg/s)
    pub mass_flow_rate: f64,
    /// Specific enthalpy (J/kg)
    pub specific_enthalpy: f64,
}

impl MassFlowStream {
    /// Create new mass flow stream.
    pub fn new(mass_flow_rate: f64, specific_enthalpy: f64) -> Result<Self, EnergyBalanceError> {
        if mass_flow_rate < 0.0 {
            return Err(EnergyBalanceError::NegativeMassFlow);
        }
        Ok(Self {
            mass_flow_rate,
            specific_enthalpy,
        })
    }

    /// Calculate enthalpy flow rate (J/s = W).
    #[must_use]
    pub fn enthalpy_flow_rate(&self) -> f64 {
        self.mass_flow_rate * self.specific_enthalpy
    }
}

/// First Law for open systems (control volume, rate form).
///
/// dE_cv/dt = Q̇ - Ẇ + Σṁ_in·h_in - Σṁ_out·h_out
///
/// Where:
/// - dE_cv/dt: Rate of change of control volume energy (W)
/// - Q̇: Heat transfer rate (W)
/// - Ẇ: Power (work rate) (W)
/// - ṁ: Mass flow rate (kg/s)
/// - h: Specific enthalpy (J/kg)
///
/// # Arguments
///
/// * `heat_rate` - Heat transfer rate into system (W)
/// * `power_out` - Power output by system (W)
/// * `inflows` - Inlet mass flow streams
/// * `outflows` - Outlet mass flow streams
///
/// # Returns
///
/// Rate of change of system energy dE/dt (W)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::{first_law_open, MassFlowStream};
///
/// let inflow = MassFlowStream::new(1.0, 3000.0).unwrap();  // 1 kg/s at 3000 J/kg
/// let outflow = MassFlowStream::new(1.0, 2500.0).unwrap(); // 1 kg/s at 2500 J/kg
///
/// // Steady state: dE/dt ≈ 0 when heat/work balance enthalpy change
/// let de_dt = first_law_open(0.0, 500.0, &[inflow], &[outflow]);
/// assert!((de_dt - 0.0).abs() < 1.0); // Near steady state
/// ```
#[must_use]
pub fn first_law_open(
    heat_rate: f64,
    power_out: f64,
    inflows: &[MassFlowStream],
    outflows: &[MassFlowStream],
) -> f64 {
    let enthalpy_in: f64 = inflows.iter().map(|s| s.enthalpy_flow_rate()).sum();
    let enthalpy_out: f64 = outflows.iter().map(|s| s.enthalpy_flow_rate()).sum();

    heat_rate - power_out + enthalpy_in - enthalpy_out
}

/// Open system energy balance result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSystemBalance {
    /// Heat transfer rate (W)
    pub heat_rate: f64,
    /// Power output (W)
    pub power_out: f64,
    /// Total inlet enthalpy flow (W)
    pub enthalpy_in: f64,
    /// Total outlet enthalpy flow (W)
    pub enthalpy_out: f64,
    /// Rate of energy change dE/dt (W)
    pub de_dt: f64,
    /// System is at steady state (dE/dt ≈ 0)
    pub is_steady_state: bool,
}

impl OpenSystemBalance {
    /// Calculate open system energy balance.
    pub fn calculate(
        heat_rate: f64,
        power_out: f64,
        inflows: &[MassFlowStream],
        outflows: &[MassFlowStream],
        steady_state_tolerance: f64,
    ) -> Self {
        let enthalpy_in: f64 = inflows.iter().map(|s| s.enthalpy_flow_rate()).sum();
        let enthalpy_out: f64 = outflows.iter().map(|s| s.enthalpy_flow_rate()).sum();
        let de_dt = first_law_open(heat_rate, power_out, inflows, outflows);

        Self {
            heat_rate,
            power_out,
            enthalpy_in,
            enthalpy_out,
            de_dt,
            is_steady_state: de_dt.abs() < steady_state_tolerance,
        }
    }

    /// Calculate mass balance (continuity).
    ///
    /// dm/dt = Σṁ_in - Σṁ_out
    #[must_use]
    pub fn mass_balance(inflows: &[MassFlowStream], outflows: &[MassFlowStream]) -> f64 {
        let mass_in: f64 = inflows.iter().map(|s| s.mass_flow_rate).sum();
        let mass_out: f64 = outflows.iter().map(|s| s.mass_flow_rate).sum();
        mass_in - mass_out
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STEADY-STATE ENERGY BALANCE
// ═══════════════════════════════════════════════════════════════════════════

/// Steady-state energy balance (dE/dt = 0).
///
/// 0 = Q̇ - Ẇ + Σṁ_in·h_in - Σṁ_out·h_out
///
/// Rearranged: Q̇ - Ẇ = Σṁ_out·h_out - Σṁ_in·h_in
///
/// # Arguments
///
/// * `inflows` - Inlet streams
/// * `outflows` - Outlet streams
///
/// # Returns
///
/// Required Q̇ - Ẇ to maintain steady state (W)
#[must_use]
pub fn steady_state_heat_work_balance(
    inflows: &[MassFlowStream],
    outflows: &[MassFlowStream],
) -> f64 {
    let enthalpy_in: f64 = inflows.iter().map(|s| s.enthalpy_flow_rate()).sum();
    let enthalpy_out: f64 = outflows.iter().map(|s| s.enthalpy_flow_rate()).sum();

    enthalpy_out - enthalpy_in
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM TYPE CLASSIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Thermodynamic system classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemType {
    /// No mass or energy exchange with surroundings
    Isolated,
    /// Energy exchange allowed, no mass exchange
    Closed,
    /// Both mass and energy exchange allowed
    Open,
}

impl SystemType {
    /// Determine system type from boundary conditions.
    #[must_use]
    pub fn classify(has_mass_flow: bool, has_energy_transfer: bool) -> Self {
        match (has_mass_flow, has_energy_transfer) {
            (false, false) => Self::Isolated,
            (false, true) => Self::Closed,
            (true, _) => Self::Open,
        }
    }

    /// Check if mass balance applies.
    #[must_use]
    pub fn has_mass_exchange(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Check if energy balance applies.
    #[must_use]
    pub fn has_energy_exchange(&self) -> bool {
        !matches!(self, Self::Isolated)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PV MAPPING: CASE PROCESSING WORKFLOW
// ═══════════════════════════════════════════════════════════════════════════

/// PV case processing energy balance analogy.
///
/// Maps thermodynamic terms to pharmacovigilance workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseProcessingBalance {
    /// Cases in backlog (internal energy analog)
    pub backlog: u64,
    /// New cases received (heat input analog)
    pub cases_received: u64,
    /// Cases resolved (work output analog)
    pub cases_resolved: u64,
    /// Cases transferred in from other systems
    pub transfers_in: u64,
    /// Cases transferred out to other systems
    pub transfers_out: u64,
    /// Net change in backlog
    pub delta_backlog: i64,
}

impl CaseProcessingBalance {
    /// Calculate case processing balance.
    ///
    /// ΔBacklog = Received - Resolved + Transfers_in - Transfers_out
    #[must_use]
    pub fn calculate(
        backlog: u64,
        cases_received: u64,
        cases_resolved: u64,
        transfers_in: u64,
        transfers_out: u64,
    ) -> Self {
        let delta_backlog = cases_received as i64 - cases_resolved as i64 + transfers_in as i64
            - transfers_out as i64;

        Self {
            backlog,
            cases_received,
            cases_resolved,
            transfers_in,
            transfers_out,
            delta_backlog,
        }
    }

    /// Final backlog after applying balance.
    #[must_use]
    pub fn final_backlog(&self) -> u64 {
        (self.backlog as i64 + self.delta_backlog).max(0) as u64
    }

    /// Check if system is at steady state (ΔBacklog ≈ 0).
    #[must_use]
    pub fn is_steady_state(&self) -> bool {
        self.delta_backlog == 0
    }

    /// Processing rate required to achieve steady state.
    #[must_use]
    pub fn required_resolution_rate(&self) -> u64 {
        self.cases_received + self.transfers_in
            - self
                .transfers_out
                .min(self.cases_received + self.transfers_in)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // --- Closed System Tests ---

    #[test]
    fn test_first_law_closed_basic() {
        // 100 J heat in, 40 J work out -> 60 J increase
        let delta_u = first_law_closed(100.0, 40.0);
        assert!((delta_u - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_first_law_closed_adiabatic() {
        // No heat transfer, work done
        let delta_u = first_law_closed(0.0, 50.0);
        assert!((delta_u - (-50.0)).abs() < 0.001);
    }

    #[test]
    fn test_first_law_closed_isochoric() {
        // No work (constant volume), heat added
        let delta_u = first_law_closed(75.0, 0.0);
        assert!((delta_u - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_closed_system_balance() {
        let balance = ClosedSystemBalance::calculate(100.0, 50.0, 20.0);
        assert!((balance.u_final - 130.0).abs() < 0.001);
        assert!(balance.is_balanced);
        assert!(balance.verify(0.001));
    }

    // --- Open System Tests ---

    #[test]
    fn test_mass_flow_stream() {
        let stream = MassFlowStream::new(2.0, 1500.0).unwrap();
        assert!((stream.enthalpy_flow_rate() - 3000.0).abs() < 0.001);
    }

    #[test]
    fn test_mass_flow_stream_negative() {
        assert!(MassFlowStream::new(-1.0, 1000.0).is_err());
    }

    #[test]
    fn test_first_law_open_steady_state() {
        let inflow = MassFlowStream::new(1.0, 3000.0).unwrap();
        let outflow = MassFlowStream::new(1.0, 2500.0).unwrap();

        // Heat and work balance the enthalpy change
        let de_dt = first_law_open(0.0, 500.0, &[inflow], &[outflow]);
        assert!((de_dt - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_first_law_open_accumulating() {
        let inflow = MassFlowStream::new(2.0, 1000.0).unwrap();
        let outflow = MassFlowStream::new(1.0, 1000.0).unwrap();

        // More mass in than out, energy accumulates
        let de_dt = first_law_open(0.0, 0.0, &[inflow], &[outflow]);
        assert!((de_dt - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_open_system_balance() {
        let inflow = MassFlowStream::new(1.0, 2000.0).unwrap();
        let outflow = MassFlowStream::new(1.0, 1800.0).unwrap();

        let balance = OpenSystemBalance::calculate(100.0, 300.0, &[inflow], &[outflow], 1.0);

        // dE/dt = 100 - 300 + 2000 - 1800 = 0
        assert!(balance.is_steady_state);
    }

    #[test]
    fn test_mass_balance() {
        let in1 = MassFlowStream::new(2.0, 1000.0).unwrap();
        let in2 = MassFlowStream::new(1.0, 1500.0).unwrap();
        let out1 = MassFlowStream::new(2.5, 1200.0).unwrap();

        let dm_dt = OpenSystemBalance::mass_balance(&[in1, in2], &[out1]);
        assert!((dm_dt - 0.5).abs() < 0.001); // 3 - 2.5 = 0.5 kg/s accumulation
    }

    // --- Steady State Tests ---

    #[test]
    fn test_steady_state_heat_work_balance() {
        let inflow = MassFlowStream::new(1.0, 1000.0).unwrap();
        let outflow = MassFlowStream::new(1.0, 1200.0).unwrap();

        let q_minus_w = steady_state_heat_work_balance(&[inflow], &[outflow]);
        // Need Q - W = 200 W to maintain steady state
        assert!((q_minus_w - 200.0).abs() < 0.001);
    }

    // --- System Type Tests ---

    #[test]
    fn test_system_type_classification() {
        assert_eq!(SystemType::classify(false, false), SystemType::Isolated);
        assert_eq!(SystemType::classify(false, true), SystemType::Closed);
        assert_eq!(SystemType::classify(true, false), SystemType::Open);
        assert_eq!(SystemType::classify(true, true), SystemType::Open);
    }

    // --- PV Case Processing Tests ---

    #[test]
    fn test_case_processing_balance_steady() {
        let balance = CaseProcessingBalance::calculate(100, 50, 50, 10, 10);
        assert!(balance.is_steady_state());
        assert_eq!(balance.final_backlog(), 100);
    }

    #[test]
    fn test_case_processing_balance_accumulating() {
        let balance = CaseProcessingBalance::calculate(100, 80, 50, 20, 10);
        // Delta = 80 - 50 + 20 - 10 = 40
        assert_eq!(balance.delta_backlog, 40);
        assert_eq!(balance.final_backlog(), 140);
    }

    #[test]
    fn test_case_processing_balance_depleting() {
        let balance = CaseProcessingBalance::calculate(100, 30, 70, 5, 15);
        // Delta = 30 - 70 + 5 - 15 = -50
        assert_eq!(balance.delta_backlog, -50);
        assert_eq!(balance.final_backlog(), 50);
    }
}
