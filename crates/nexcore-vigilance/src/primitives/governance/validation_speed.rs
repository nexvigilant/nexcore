//! # Validation Speed (Bill of Rights)
//!
//! Implementation of Amendment VI: The right to a speedy and public
//! validation by an impartial jury of the domain.

use super::Verdict;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// T3: SpeedyValidation — The right of any type to be validated promptly.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeedyValidation {
    /// The type being validated
    pub type_name: String,
    /// Time taken for validation (in milliseconds)
    pub validation_duration_ms: u64,
    /// Maximum acceptable validation time (in milliseconds)
    pub speedy_threshold_ms: u64,
    /// Whether validation was public (observable)
    pub public: bool,
    /// Domain of the impartial jury
    pub jury_domain: String,
    /// Whether the jury was from the same domain as the error
    pub jury_impartial: bool,
}

impl SpeedyValidation {
    /// Default speedy threshold: 30 seconds for unit tests, 5 minutes for integration.
    pub const DEFAULT_UNIT_THRESHOLD_MS: u64 = 30_000;
    pub const DEFAULT_INTEGRATION_THRESHOLD_MS: u64 = 300_000;

    /// Check if the validation was constitutional.
    pub fn is_constitutional(&self) -> bool {
        self.validation_duration_ms <= self.speedy_threshold_ms
            && self.public
            && self.jury_impartial
    }

    /// How much the validation exceeded the speedy threshold.
    pub fn delay_factor(&self) -> f64 {
        if self.speedy_threshold_ms == 0 {
            return f64::MAX;
        }
        self.validation_duration_ms as f64 / self.speedy_threshold_ms as f64
    }

    /// Get the validation duration as a std::time::Duration.
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.validation_duration_ms)
    }

    /// Render a verdict.
    pub fn verdict(&self) -> Verdict {
        if self.is_constitutional() {
            Verdict::Permitted
        } else if self.delay_factor() > 5.0 {
            Verdict::Rejected // Egregiously slow
        } else {
            Verdict::Flagged
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speedy_validation_passed() {
        let sv = SpeedyValidation {
            type_name: "DrugId".to_string(),
            validation_duration_ms: 150,
            speedy_threshold_ms: SpeedyValidation::DEFAULT_UNIT_THRESHOLD_MS,
            public: true,
            jury_domain: "pharmacovigilance".to_string(),
            jury_impartial: true,
        };
        assert!(sv.is_constitutional());
        assert!(sv.delay_factor() < 1.0);
        assert_eq!(sv.verdict(), Verdict::Permitted);
    }

    #[test]
    fn slow_validation_flagged() {
        let sv = SpeedyValidation {
            type_name: "SlowType".to_string(),
            validation_duration_ms: 60_000,
            speedy_threshold_ms: 30_000,
            public: true,
            jury_domain: "testing".to_string(),
            jury_impartial: true,
        };
        assert!(!sv.is_constitutional());
        assert!(sv.delay_factor() > 1.0);
        assert_eq!(sv.verdict(), Verdict::Flagged);
    }

    #[test]
    fn egregiously_slow_rejected() {
        let sv = SpeedyValidation {
            type_name: "Glacial".to_string(),
            validation_duration_ms: 300_000,
            speedy_threshold_ms: 30_000,
            public: true,
            jury_domain: "testing".to_string(),
            jury_impartial: true,
        };
        assert!(sv.delay_factor() > 5.0);
        assert_eq!(sv.verdict(), Verdict::Rejected);
    }

    #[test]
    fn private_validation_unconstitutional() {
        let sv = SpeedyValidation {
            type_name: "Hidden".to_string(),
            validation_duration_ms: 100,
            speedy_threshold_ms: 30_000,
            public: false,
            jury_domain: "internal".to_string(),
            jury_impartial: true,
        };
        assert!(!sv.is_constitutional());
    }

    #[test]
    fn partial_jury_unconstitutional() {
        let sv = SpeedyValidation {
            type_name: "Biased".to_string(),
            validation_duration_ms: 100,
            speedy_threshold_ms: 30_000,
            public: true,
            jury_domain: "same-team".to_string(),
            jury_impartial: false,
        };
        assert!(!sv.is_constitutional());
    }

    #[test]
    fn duration_conversion() {
        let sv = SpeedyValidation {
            type_name: "Test".to_string(),
            validation_duration_ms: 1500,
            speedy_threshold_ms: 30_000,
            public: true,
            jury_domain: "test".to_string(),
            jury_impartial: true,
        };
        assert_eq!(sv.duration(), std::time::Duration::from_millis(1500));
    }
}
