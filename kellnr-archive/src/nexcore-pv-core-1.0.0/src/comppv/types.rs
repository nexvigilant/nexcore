//! Core types for the 11 Conservation Laws.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The 11 Conservation Laws of Comprehensive Pharmacovigilance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConservationLaw {
    /// Law 1: Drug mass is conserved (dose = in_body + eliminated)
    DrugMassBalance = 1,
    /// Law 2: Binding requires negative Gibbs free energy
    ThermodynamicBinding = 2,
    /// Law 3: Total receptors = free + bound + desensitized
    ReceptorState = 3,
    /// Law 4: Pathway flux is conserved at steady state
    PathwayFlux = 4,
    /// Law 5: Enzyme mass follows synthesis/degradation kinetics
    EnzymeRegeneration = 5,
    /// Law 6: ADME compartment rates balance
    AdmeRate = 6,
    /// Law 7: Steady-state concentration follows PK equations
    SteadyState = 7,
    /// Law 8: Ionization follows Henderson-Hasselbalch
    IonizationState = 8,
    /// Law 9: Saturable processes follow Michaelis-Menten
    SaturationKinetics = 9,
    /// Law 10: Total entropy must not decrease (2nd Law)
    EntropyIncrease = 10,
    /// Law 11: Genetic information is conserved unless mutated
    GeneticConservation = 11,
}

impl ConservationLaw {
    /// Returns all 11 conservation laws in order.
    pub fn all() -> &'static [ConservationLaw] {
        &[
            Self::DrugMassBalance,
            Self::ThermodynamicBinding,
            Self::ReceptorState,
            Self::PathwayFlux,
            Self::EnzymeRegeneration,
            Self::AdmeRate,
            Self::SteadyState,
            Self::IonizationState,
            Self::SaturationKinetics,
            Self::EntropyIncrease,
            Self::GeneticConservation,
        ]
    }

    /// Returns the law number (1-11).
    pub fn number(&self) -> u8 {
        *self as u8
    }
}

/// Formal specification of a conservation law.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationLawSpec {
    /// Which law this specifies
    pub law: ConservationLaw,
    /// Human-readable name
    pub name: String,
    /// Verbal statement of the law
    pub statement: String,
    /// Mathematical formulation
    pub mathematical_form: String,
    /// Safety implications of violation
    pub safety_implication: String,
    /// Default tolerance for validation (relative deviation)
    pub validation_tolerance: f64,
}

/// Status of a constraint evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintStatus {
    /// Constraint is fully satisfied
    Satisfied,
    /// Constraint is close to violation (warning threshold)
    Warning,
    /// Constraint is violated
    Violated,
}

/// Result of evaluating a single conservation law.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawValidationResult {
    /// Which law was validated
    pub law: ConservationLaw,
    /// Whether the law is satisfied within tolerance
    pub is_valid: bool,
    /// Absolute deviation from expected value
    pub deviation: f64,
    /// Relative deviation (fraction)
    pub relative_deviation: f64,
    /// Tolerance used for validation
    pub tolerance: f64,
}

impl LawValidationResult {
    /// Returns the constraint status based on deviation relative to tolerance.
    pub fn status(&self) -> ConstraintStatus {
        if self.is_valid {
            if self.relative_deviation < self.tolerance * 0.5 {
                ConstraintStatus::Satisfied
            } else {
                ConstraintStatus::Warning
            }
        } else {
            ConstraintStatus::Violated
        }
    }
}

/// Complete validation report for all evaluated laws.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationValidationReport {
    /// Individual validation results
    pub validations: Vec<LawValidationResult>,
    /// True if all evaluated laws are valid
    pub all_valid: bool,
    /// List of laws that failed validation
    pub failed_laws: Vec<ConservationLaw>,
}

impl ConservationValidationReport {
    /// Returns the number of laws validated.
    pub fn laws_evaluated(&self) -> usize {
        self.validations.len()
    }

    /// Returns the number of laws that passed.
    pub fn laws_passed(&self) -> usize {
        self.validations.iter().filter(|v| v.is_valid).count()
    }

    /// Returns the compliance percentage (0-100).
    pub fn compliance_percentage(&self) -> f64 {
        if self.validations.is_empty() {
            100.0
        } else {
            (self.laws_passed() as f64 / self.laws_evaluated() as f64) * 100.0
        }
    }
}

/// Errors that can occur during conservation law validation.
#[derive(Debug, Error)]
pub enum ConservationError {
    /// Mass balance violation: elimination exceeds dose
    #[error("Mass balance violation: elimination ({elimination}) exceeds dose ({dose})")]
    MassBalanceViolation {
        /// Initial dose
        dose: f64,
        /// Amount eliminated
        elimination: f64,
    },
    /// Invalid parameter value
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Physical constants used in conservation law calculations.
pub mod constants {
    /// Gas constant in J/(mol·K)
    pub const GAS_CONSTANT_J_MOL_K: f64 = 8.314;
    /// Gas constant in kJ/(mol·K)
    pub const GAS_CONSTANT_KJ_MOL_K: f64 = 0.008314;
    /// Standard temperature in Kelvin (25°C)
    pub const STANDARD_TEMPERATURE_K: f64 = 298.15;
    /// Physiological temperature in Kelvin (37°C)
    pub const PHYSIOLOGICAL_TEMPERATURE_K: f64 = 310.15;
    /// Avogadro's number
    pub const AVOGADRO_NUMBER: f64 = 6.022e23;
    /// Number of half-lives to reach ~90% steady state
    pub const HALF_LIVES_TO_STEADY_STATE: f64 = 4.5;
}
