//! # Signal Alert
//!
//! Alert lifecycle management for detected signals.
//! Generates, transitions, and tracks alerts through states:
//! New → UnderReview → Confirmed/Escalated/FalsePositive → Closed.
//!
//! ## T1 Primitive: State
//! Alert is a state machine with defined transitions.

use crate::core::{Alert, AlertState, Alertable, DetectionResult, Result, SignalError, Threshold};
use nexcore_chrono::DateTime;
use nexcore_id::NexId;

/// Creates alerts from detection results that pass a threshold.
pub struct AlertGenerator<T: Threshold> {
    threshold: T,
}

impl<T: Threshold> AlertGenerator<T> {
    /// Create with a given threshold filter.
    pub fn new(threshold: T) -> Self {
        Self { threshold }
    }

    /// Generate alerts for all results passing the threshold.
    pub fn generate_all(&self, results: &[DetectionResult]) -> Result<Vec<Alert>> {
        results
            .iter()
            .filter(|r| self.threshold.apply(r))
            .map(|r| self.alert(r))
            .collect()
    }
}

impl<T: Threshold> Alertable for AlertGenerator<T> {
    fn alert(&self, result: &DetectionResult) -> Result<Alert> {
        Ok(Alert {
            id: NexId::v4(),
            detection: result.clone(),
            state: AlertState::New,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            notes: Vec::new(),
        })
    }
}

/// Manages alert state transitions.
pub struct AlertTransitions;

impl AlertTransitions {
    /// Transition an alert to a new state, validating the transition.
    pub fn transition(alert: &mut Alert, new_state: AlertState) -> Result<()> {
        let valid = matches!(
            (alert.state, new_state),
            (AlertState::New, AlertState::UnderReview)
                | (AlertState::UnderReview, AlertState::Confirmed)
                | (AlertState::UnderReview, AlertState::FalsePositive)
                | (AlertState::Confirmed, AlertState::Escalated)
                | (AlertState::Confirmed, AlertState::Closed)
                | (AlertState::Escalated, AlertState::Closed)
                | (AlertState::FalsePositive, AlertState::Closed)
        );
        if !valid {
            return Err(SignalError::Validation(format!(
                "invalid transition: {:?} → {:?}",
                alert.state, new_state
            )));
        }
        alert.state = new_state;
        alert.updated_at = DateTime::now();
        Ok(())
    }

    /// Add a note to an alert.
    pub fn add_note(alert: &mut Alert, note: impl Into<String>) {
        alert.notes.push(note.into());
        alert.updated_at = DateTime::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;

    struct AlwaysPass;
    impl Threshold for AlwaysPass {
        fn apply(&self, _: &DetectionResult) -> bool {
            true
        }
    }

    fn make_result() -> DetectionResult {
        DetectionResult {
            pair: DrugEventPair::new("drug", "event"),
            table: ContingencyTable {
                a: 10,
                b: 100,
                c: 20,
                d: 10_000,
            },
            prr: Some(Prr(3.0)),
            ror: Some(Ror(5.0)),
            ic: Some(Ic(1.5)),
            ebgm: Some(Ebgm(2.0)),
            chi_square: ChiSquare(10.0),
            strength: SignalStrength::Strong,
            detected_at: DateTime::now(),
        }
    }

    #[test]
    fn generate_alert() {
        let generator = AlertGenerator::new(AlwaysPass);
        let alert = generator.alert(&make_result()).unwrap();
        assert_eq!(alert.state, AlertState::New);
    }

    #[test]
    fn valid_transition() {
        let generator = AlertGenerator::new(AlwaysPass);
        let mut alert = generator.alert(&make_result()).unwrap();
        AlertTransitions::transition(&mut alert, AlertState::UnderReview).unwrap();
        assert_eq!(alert.state, AlertState::UnderReview);
    }

    #[test]
    fn invalid_transition() {
        let generator = AlertGenerator::new(AlwaysPass);
        let mut alert = generator.alert(&make_result()).unwrap();
        // New → Closed is not allowed
        let err = AlertTransitions::transition(&mut alert, AlertState::Closed);
        assert!(err.is_err());
    }
}
