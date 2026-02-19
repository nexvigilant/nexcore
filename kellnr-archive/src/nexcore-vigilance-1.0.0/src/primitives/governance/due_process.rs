//! # Due Process (Bill of Rights)
//!
//! Implementation of Amendment V: No Type shall be held to answer for a
//! critical failure without a Validation Pipeline. No double jeopardy.
//! No self-contradiction. Due Process of Validation required.

use super::Verdict;
use serde::{Deserialize, Serialize};

/// T3: DueProcess — The right of any type to undergo proper validation
/// before being condemned for a critical failure.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DueProcess {
    /// The type under examination
    pub type_name: String,
    /// Whether a validation pipeline was used
    pub validation_pipeline_used: bool,
    /// Number of times this type has been tried for the same offense
    pub trial_count: u32,
    /// Whether the type was forced to contradict itself
    pub self_contradiction_forced: bool,
    /// Whether private state was taken without compensation
    pub state_seized_without_compensation: bool,
}

impl DueProcess {
    /// Check if due process was followed.
    pub fn is_constitutional(&self) -> bool {
        self.validation_pipeline_used
            && self.trial_count <= 1 // No double jeopardy
            && !self.self_contradiction_forced
            && !self.state_seized_without_compensation
    }

    /// Identify which specific right was violated.
    pub fn violations(&self) -> Vec<DueProcessViolation> {
        let mut violations = Vec::new();
        if !self.validation_pipeline_used {
            violations.push(DueProcessViolation::NoValidationPipeline);
        }
        if self.trial_count > 1 {
            violations.push(DueProcessViolation::DoubleJeopardy {
                trial_count: self.trial_count,
            });
        }
        if self.self_contradiction_forced {
            violations.push(DueProcessViolation::SelfContradiction);
        }
        if self.state_seized_without_compensation {
            violations.push(DueProcessViolation::StateSeizure);
        }
        violations
    }

    /// Render a verdict.
    pub fn verdict(&self) -> Verdict {
        if self.is_constitutional() {
            Verdict::Permitted
        } else if self.violations().iter().any(|v| {
            matches!(
                v,
                DueProcessViolation::DoubleJeopardy { .. } | DueProcessViolation::SelfContradiction
            )
        }) {
            Verdict::Rejected
        } else {
            Verdict::Flagged
        }
    }
}

/// T3: Specific due process violations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DueProcessViolation {
    /// No validation pipeline was used before condemning the type
    NoValidationPipeline,
    /// The type was tried more than once for the same offense
    DoubleJeopardy { trial_count: u32 },
    /// The type was forced to contradict itself in testing
    SelfContradiction,
    /// Private state was seized without quota compensation
    StateSeizure,
}

/// T3: QuotaCompensation — Just compensation when private state is taken.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotaCompensation {
    /// State that was seized
    pub seized_state: String,
    /// Compensation amount (in token-quota units)
    pub compensation: f64,
    /// Whether compensation was actually provided
    pub provided: bool,
}

impl QuotaCompensation {
    /// Compensation is just if it was provided and non-negative.
    pub fn is_just(&self) -> bool {
        self.provided && self.compensation >= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_process_followed() {
        let dp = DueProcess {
            type_name: "SignalDetector".to_string(),
            validation_pipeline_used: true,
            trial_count: 1,
            self_contradiction_forced: false,
            state_seized_without_compensation: false,
        };
        assert!(dp.is_constitutional());
        assert!(dp.violations().is_empty());
        assert_eq!(dp.verdict(), Verdict::Permitted);
    }

    #[test]
    fn no_validation_pipeline() {
        let dp = DueProcess {
            type_name: "UntestedType".to_string(),
            validation_pipeline_used: false,
            trial_count: 1,
            self_contradiction_forced: false,
            state_seized_without_compensation: false,
        };
        assert!(!dp.is_constitutional());
        assert_eq!(dp.violations().len(), 1);
        assert_eq!(dp.verdict(), Verdict::Flagged);
    }

    #[test]
    fn double_jeopardy() {
        let dp = DueProcess {
            type_name: "RetryVictim".to_string(),
            validation_pipeline_used: true,
            trial_count: 3,
            self_contradiction_forced: false,
            state_seized_without_compensation: false,
        };
        assert!(!dp.is_constitutional());
        let violations = dp.violations();
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0],
            DueProcessViolation::DoubleJeopardy { trial_count: 3 }
        ));
        assert_eq!(dp.verdict(), Verdict::Rejected);
    }

    #[test]
    fn self_contradiction_forced() {
        let dp = DueProcess {
            type_name: "Conflicted".to_string(),
            validation_pipeline_used: true,
            trial_count: 1,
            self_contradiction_forced: true,
            state_seized_without_compensation: false,
        };
        assert!(!dp.is_constitutional());
        assert_eq!(dp.verdict(), Verdict::Rejected);
    }

    #[test]
    fn state_seized_without_compensation() {
        let dp = DueProcess {
            type_name: "Seized".to_string(),
            validation_pipeline_used: true,
            trial_count: 1,
            self_contradiction_forced: false,
            state_seized_without_compensation: true,
        };
        assert!(!dp.is_constitutional());
    }

    #[test]
    fn just_compensation() {
        let comp = QuotaCompensation {
            seized_state: "cache_entries".to_string(),
            compensation: 100.0,
            provided: true,
        };
        assert!(comp.is_just());
    }

    #[test]
    fn unjust_compensation_not_provided() {
        let comp = QuotaCompensation {
            seized_state: "session_data".to_string(),
            compensation: 50.0,
            provided: false,
        };
        assert!(!comp.is_just());
    }
}
