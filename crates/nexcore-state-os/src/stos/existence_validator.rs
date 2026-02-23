// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 10: Existence Validator (STOS-EX)
//!
//! **Dominant Primitive**: ∃ (Existence)
//!
//! Validates that states, transitions, and machines actually exist
//! before operations are performed on them.
//!
//! ## Tier Classification
//!
//! `ExistenceValidator` is T2-P (∃ + ς) — existence, state.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use super::state_registry::StateId;
use super::transition_engine::TransitionId;
use crate::MachineId;

/// Result of an existence check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistenceResult {
    /// Entity exists.
    Exists,
    /// Entity does not exist.
    NotExists,
    /// Entity was deleted.
    Deleted,
    /// Existence unknown (not yet registered).
    Unknown,
}

impl ExistenceResult {
    /// Whether the entity exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        matches!(self, Self::Exists)
    }

    /// Whether the entity is usable.
    #[must_use]
    pub fn usable(&self) -> bool {
        matches!(self, Self::Exists)
    }
}

/// A validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Entity type.
    pub entity_type: EntityType,
    /// Entity ID (as string for uniformity).
    pub entity_id: String,
    /// Error kind.
    pub kind: ValidationErrorKind,
}

/// Type of entity being validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    /// A state.
    State,
    /// A transition.
    Transition,
    /// A machine.
    Machine,
}

/// Kind of validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// Entity not found.
    NotFound,
    /// Entity was deleted.
    Deleted,
    /// Entity is invalid.
    Invalid,
    /// Reference to non-existent entity.
    DanglingReference,
}

/// The existence validator.
///
/// ## Tier: T2-P (∃ + ς)
///
/// Dominant primitive: ∃ (Existence)
#[derive(Debug, Clone)]
pub struct ExistenceValidator {
    /// Machine ID.
    _machine_id: MachineId,
    /// Known states.
    known_states: BTreeSet<StateId>,
    /// Known transitions.
    known_transitions: BTreeSet<TransitionId>,
    /// Deleted states.
    deleted_states: BTreeSet<StateId>,
    /// Deleted transitions.
    deleted_transitions: BTreeSet<TransitionId>,
    /// Validation errors.
    errors: Vec<ValidationError>,
}

impl ExistenceValidator {
    /// Create a new existence validator.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            _machine_id: machine_id,
            known_states: BTreeSet::new(),
            known_transitions: BTreeSet::new(),
            deleted_states: BTreeSet::new(),
            deleted_transitions: BTreeSet::new(),
            errors: Vec::new(),
        }
    }

    /// Register a state as existing.
    pub fn register_state(&mut self, state: StateId) {
        self.known_states.insert(state);
        self.deleted_states.remove(&state);
    }

    /// Register a transition as existing.
    pub fn register_transition(&mut self, transition: TransitionId) {
        self.known_transitions.insert(transition);
        self.deleted_transitions.remove(&transition);
    }

    /// Mark a state as deleted.
    pub fn delete_state(&mut self, state: StateId) {
        self.known_states.remove(&state);
        self.deleted_states.insert(state);
    }

    /// Mark a transition as deleted.
    pub fn delete_transition(&mut self, transition: TransitionId) {
        self.known_transitions.remove(&transition);
        self.deleted_transitions.insert(transition);
    }

    /// Check if a state exists.
    #[must_use]
    pub fn state_exists(&self, state: StateId) -> ExistenceResult {
        if self.known_states.contains(&state) {
            ExistenceResult::Exists
        } else if self.deleted_states.contains(&state) {
            ExistenceResult::Deleted
        } else {
            ExistenceResult::NotExists
        }
    }

    /// Check if a transition exists.
    #[must_use]
    pub fn transition_exists(&self, transition: TransitionId) -> ExistenceResult {
        if self.known_transitions.contains(&transition) {
            ExistenceResult::Exists
        } else if self.deleted_transitions.contains(&transition) {
            ExistenceResult::Deleted
        } else {
            ExistenceResult::NotExists
        }
    }

    /// Validate a state reference.
    pub fn validate_state(&mut self, state: StateId) -> bool {
        match self.state_exists(state) {
            ExistenceResult::Exists => true,
            ExistenceResult::Deleted => {
                self.errors.push(ValidationError {
                    entity_type: EntityType::State,
                    entity_id: state.to_string(),
                    kind: ValidationErrorKind::Deleted,
                });
                false
            }
            _ => {
                self.errors.push(ValidationError {
                    entity_type: EntityType::State,
                    entity_id: state.to_string(),
                    kind: ValidationErrorKind::NotFound,
                });
                false
            }
        }
    }

    /// Validate a transition reference.
    pub fn validate_transition(&mut self, transition: TransitionId) -> bool {
        match self.transition_exists(transition) {
            ExistenceResult::Exists => true,
            ExistenceResult::Deleted => {
                self.errors.push(ValidationError {
                    entity_type: EntityType::Transition,
                    entity_id: transition.to_string(),
                    kind: ValidationErrorKind::Deleted,
                });
                false
            }
            _ => {
                self.errors.push(ValidationError {
                    entity_type: EntityType::Transition,
                    entity_id: transition.to_string(),
                    kind: ValidationErrorKind::NotFound,
                });
                false
            }
        }
    }

    /// Validate multiple states.
    pub fn validate_states(&mut self, states: &[StateId]) -> bool {
        states.iter().all(|&s| self.validate_state(s))
    }

    /// Validate multiple transitions.
    pub fn validate_transitions(&mut self, transitions: &[TransitionId]) -> bool {
        transitions.iter().all(|&t| self.validate_transition(t))
    }

    /// Get validation errors.
    #[must_use]
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Clear validation errors.
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Check if validation passed (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Number of known states.
    #[must_use]
    pub fn state_count(&self) -> usize {
        self.known_states.len()
    }

    /// Number of known transitions.
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.known_transitions.len()
    }

    /// Reset validator.
    pub fn reset(&mut self) {
        self.known_states.clear();
        self.known_transitions.clear();
        self.deleted_states.clear();
        self.deleted_transitions.clear();
        self.errors.clear();
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_existence() {
        let mut validator = ExistenceValidator::new(1);

        validator.register_state(0);
        validator.register_state(1);

        assert!(validator.state_exists(0).exists());
        assert!(validator.state_exists(1).exists());
        assert!(!validator.state_exists(2).exists());
    }

    #[test]
    fn test_deleted_state() {
        let mut validator = ExistenceValidator::new(1);

        validator.register_state(0);
        assert!(validator.state_exists(0).exists());

        validator.delete_state(0);
        assert_eq!(validator.state_exists(0), ExistenceResult::Deleted);
    }

    #[test]
    fn test_validation_errors() {
        let mut validator = ExistenceValidator::new(1);

        validator.register_state(0);

        assert!(validator.validate_state(0));
        assert!(!validator.validate_state(1)); // Not registered

        assert!(!validator.is_valid());
        assert_eq!(validator.errors().len(), 1);
    }

    #[test]
    fn test_transition_existence() {
        let mut validator = ExistenceValidator::new(1);

        validator.register_transition(0);
        validator.register_transition(1);

        assert!(validator.transition_exists(0).exists());
        assert!(!validator.transition_exists(99).exists());
    }

    #[test]
    fn test_batch_validation() {
        let mut validator = ExistenceValidator::new(1);

        validator.register_state(0);
        validator.register_state(1);
        validator.register_state(2);

        assert!(validator.validate_states(&[0, 1, 2]));
        assert!(!validator.validate_states(&[0, 1, 99]));
    }
}
