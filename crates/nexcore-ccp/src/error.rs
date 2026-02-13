//! CCP error types.
//!
//! # T1 Grounding
//! - ∂ (boundary): Each variant guards a domain constraint
//! - ∅ (absence): Errors signal absence of valid state

use std::fmt;

/// All errors that can occur in the CCP pharmacokinetic engine.
///
/// Tier: T2-P (newtypes wrapping domain violations)
#[derive(Debug, Clone, PartialEq)]
pub enum CcpError {
    /// Time parameter must be non-negative.
    NegativeTime {
        /// The invalid time value provided.
        value: f64,
    },
    /// Bioavailability must be in (0, 1].
    InvalidBioavailability {
        /// The invalid bioavailability value provided.
        value: f64,
    },
    /// Therapeutic window lower bound must be < upper bound, both in [0, 1].
    InvalidWindow {
        /// Lower bound of the window.
        lower: f64,
        /// Upper bound of the window.
        upper: f64,
    },
    /// Phase transition is not allowed by the FSM.
    InvalidPhaseTransition {
        /// Source phase name.
        from: String,
        /// Target phase name.
        to: String,
    },
    /// Dose must be in [0, 1].
    InvalidDose {
        /// The invalid dose value provided.
        value: f64,
    },
    /// Plasma level is below the therapeutic threshold.
    SubtherapeuticLevel {
        /// Current plasma level.
        current: f64,
        /// Required minimum.
        threshold: f64,
    },
    /// Half-life must be positive.
    InvalidHalfLife {
        /// The invalid half-life value provided.
        value: f64,
    },
    /// Episode has no interventions to score.
    NoInterventions,
}

impl fmt::Display for CcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeTime { value } => {
                write!(f, "negative time: {value}")
            }
            Self::InvalidBioavailability { value } => {
                write!(f, "bioavailability must be in (0, 1], got {value}")
            }
            Self::InvalidWindow { lower, upper } => {
                write!(f, "invalid window: lower={lower} >= upper={upper}")
            }
            Self::InvalidPhaseTransition { from, to } => {
                write!(f, "invalid transition: {from} → {to}")
            }
            Self::InvalidDose { value } => {
                write!(f, "dose must be in [0, 1], got {value}")
            }
            Self::SubtherapeuticLevel { current, threshold } => {
                write!(f, "subtherapeutic: {current} < {threshold}")
            }
            Self::InvalidHalfLife { value } => {
                write!(f, "half-life must be positive, got {value}")
            }
            Self::NoInterventions => {
                write!(f, "episode has no interventions")
            }
        }
    }
}

impl std::error::Error for CcpError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_negative_time() {
        let e = CcpError::NegativeTime { value: -1.0 };
        assert!(e.to_string().contains("-1"));
    }

    #[test]
    fn display_invalid_bioavailability() {
        let e = CcpError::InvalidBioavailability { value: 1.5 };
        assert!(e.to_string().contains("1.5"));
    }

    #[test]
    fn display_invalid_window() {
        let e = CcpError::InvalidWindow {
            lower: 0.8,
            upper: 0.2,
        };
        assert!(e.to_string().contains("0.8"));
    }

    #[test]
    fn display_invalid_phase_transition() {
        let e = CcpError::InvalidPhaseTransition {
            from: "Collect".to_string(),
            to: "FollowUp".to_string(),
        };
        assert!(e.to_string().contains("Collect"));
    }

    #[test]
    fn display_invalid_dose() {
        let e = CcpError::InvalidDose { value: 2.0 };
        assert!(e.to_string().contains("2"));
    }

    #[test]
    fn display_subtherapeutic_level() {
        let e = CcpError::SubtherapeuticLevel {
            current: 0.1,
            threshold: 0.3,
        };
        assert!(e.to_string().contains("0.1"));
    }

    #[test]
    fn error_trait_is_implemented() {
        let e: Box<dyn std::error::Error> = Box::new(CcpError::NegativeTime { value: -1.0 });
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn display_no_interventions() {
        let e = CcpError::NoInterventions;
        assert!(e.to_string().contains("no interventions"));
    }
}
