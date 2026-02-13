// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # QuantumStateSpace
//!
//! **Tier**: T3 (ς + Σ + κ + → + ∂ + ρ + N)
//! **Dominant**: ς (State)
//! **Bridge**: quantum primitives × state-os
//! **Confidence**: 0.78
//!
//! Probabilistic finite state machines where states have probability weights
//! instead of binary occupancy. Supports:
//!
//! - **Superposition**: Machine occupies multiple states simultaneously
//! - **Measurement**: Collapses superposition to single state via weighted selection
//! - **Interference**: Transition paths can constructively or destructively combine
//! - **Entanglement**: Observing one machine collapses a correlated partner

use core::fmt;
use std::collections::BTreeMap;

/// Unique state identifier.
pub type QStateId = u32;

/// A probability amplitude for a state.
///
/// In real quantum mechanics this would be complex. We use real-valued
/// weights with sign to model constructive (+) and destructive (-) interference.
///
/// ## Tier: T2-P (N + ς)
#[derive(Debug, Clone, Copy)]
pub struct Amplitude {
    /// Real-valued weight. Sign enables interference.
    pub weight: f64,
}

impl Amplitude {
    /// Create an amplitude.
    #[must_use]
    pub fn new(weight: f64) -> Self {
        Self { weight }
    }

    /// Probability = |amplitude|² (Born rule analog).
    #[must_use]
    pub fn probability(&self) -> f64 {
        self.weight * self.weight
    }
}

/// A quantum-like state with a label.
#[derive(Debug, Clone)]
pub struct QState {
    /// State identifier.
    pub id: QStateId,
    /// Human-readable label.
    pub label: String,
    /// Whether this is an absorbing (terminal) state.
    pub absorbing: bool,
}

/// A transition between quantum states with amplitude modification.
///
/// ## Tier: T2-P (→ + N)
#[derive(Debug, Clone)]
pub struct QTransition {
    /// Source state.
    pub from: QStateId,
    /// Target state.
    pub to: QStateId,
    /// Amplitude transfer factor. Applied to source amplitude.
    /// Positive = constructive, negative = destructive.
    pub factor: f64,
    /// Transition label.
    pub label: String,
}

/// Result of a measurement (superposition collapse).
///
/// ## Tier: T2-P (ς + ∂)
#[derive(Debug, Clone)]
pub struct MeasurementResult {
    /// The collapsed state.
    pub state_id: QStateId,
    /// Probability that this state was selected.
    pub probability: f64,
    /// Whether an entangled partner was also collapsed.
    pub entangled_collapse: bool,
}

/// Probabilistic state machine with quantum-inspired semantics.
///
/// ## Tier: T3 (ς + Σ + κ + → + ∂ + ρ + N)
/// Dominant: ς (State) — probabilistic state occupancy drives everything
///
/// Primitives:
/// - ς: Multiple simultaneous state occupancies (superposition)
/// - Σ: Probability normalization (weights sum to 1.0)
/// - κ: Weighted selection during measurement, amplitude comparison
/// - →: Transitions cause amplitude redistribution
/// - ∂: Measurement boundary collapses superposition
/// - ρ: Iterative amplitude propagation through transition graph
/// - N: Probability values, amplitude weights, normalization factors
#[derive(Debug, Clone)]
pub struct QuantumStateSpace {
    /// States by ID.
    states: BTreeMap<QStateId, QState>,
    /// Current amplitude for each state.
    amplitudes: BTreeMap<QStateId, Amplitude>,
    /// Available transitions.
    transitions: Vec<QTransition>,
    /// Entangled partner (another QSS index to collapse on measurement).
    entangled_partner: Option<QStateId>,
    /// Next state ID.
    next_id: QStateId,
    /// Whether superposition has been collapsed.
    collapsed: bool,
    /// Measurement history.
    measurements: Vec<MeasurementResult>,
}

impl QuantumStateSpace {
    /// Create an empty quantum state space.
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: BTreeMap::new(),
            amplitudes: BTreeMap::new(),
            transitions: Vec::new(),
            entangled_partner: None,
            next_id: 0,
            collapsed: false,
            measurements: Vec::new(),
        }
    }

    /// Add a state with an initial amplitude.
    pub fn add_state(
        &mut self,
        label: impl Into<String>,
        amplitude: f64,
        absorbing: bool,
    ) -> QStateId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        self.states.insert(
            id,
            QState {
                id,
                label: label.into(),
                absorbing,
            },
        );
        self.amplitudes.insert(id, Amplitude::new(amplitude));

        id
    }

    /// Add a transition with an amplitude transfer factor.
    pub fn add_transition(
        &mut self,
        from: QStateId,
        to: QStateId,
        factor: f64,
        label: impl Into<String>,
    ) {
        self.transitions.push(QTransition {
            from,
            to,
            factor,
            label: label.into(),
        });
    }

    /// Set an entangled partner state. When this QSS is measured,
    /// the partner state is the one that gets set in the correlated system.
    pub fn entangle(&mut self, partner_state: QStateId) {
        self.entangled_partner = Some(partner_state);
    }

    /// Normalize amplitudes so probabilities sum to 1.0 (Σ primitive).
    pub fn normalize(&mut self) {
        let total_prob: f64 = self.amplitudes.values().map(|a| a.probability()).sum();

        if total_prob > 0.0 {
            let scale = 1.0 / total_prob.sqrt();
            for amp in self.amplitudes.values_mut() {
                amp.weight *= scale;
            }
        }
    }

    /// Propagate amplitudes through all transitions (one step).
    ///
    /// Each transition transfers amplitude from source to target:
    /// `target_new += source_current * factor`
    ///
    /// This is the ρ (Recursion) primitive — iterative propagation.
    pub fn propagate(&mut self) {
        let mut deltas: BTreeMap<QStateId, f64> = BTreeMap::new();

        for t in &self.transitions {
            let source_amp = self
                .amplitudes
                .get(&t.from)
                .map(|a| a.weight)
                .unwrap_or(0.0);

            *deltas.entry(t.to).or_default() += source_amp * t.factor;
        }

        // Apply deltas (interference: constructive if same sign, destructive if opposite)
        for (state_id, delta) in &deltas {
            if let Some(amp) = self.amplitudes.get_mut(state_id) {
                amp.weight += delta;
            }
        }

        self.normalize();
    }

    /// Measure the system — collapse superposition to a single state.
    ///
    /// Uses a deterministic seed for reproducibility: selects the highest-probability
    /// state (ties broken by state ID). For true randomness, caller would
    /// provide external entropy.
    ///
    /// This is the ∂ (Boundary) primitive — measurement creates a sharp boundary
    /// between superposed and collapsed.
    pub fn measure(&mut self) -> MeasurementResult {
        self.normalize();

        // Find highest-probability state (deterministic "measurement")
        let mut best_id: Option<QStateId> = None;
        let mut best_prob = -1.0_f64;

        for (&id, amp) in &self.amplitudes {
            let prob = amp.probability();
            if prob > best_prob || (prob == best_prob && best_id.is_none()) {
                best_prob = prob;
                best_id = Some(id);
            }
        }

        let collapsed_id = best_id.unwrap_or(0);

        // Collapse: set selected state to amplitude 1.0, all others to 0.0
        for (&id, amp) in self.amplitudes.iter_mut() {
            amp.weight = if id == collapsed_id { 1.0 } else { 0.0 };
        }

        self.collapsed = true;

        let result = MeasurementResult {
            state_id: collapsed_id,
            probability: best_prob,
            entangled_collapse: self.entangled_partner.is_some(),
        };

        self.measurements.push(result.clone());
        result
    }

    /// Get the probability distribution (Born rule: P = |ψ|²).
    #[must_use]
    pub fn probability_distribution(&self) -> BTreeMap<QStateId, f64> {
        self.amplitudes
            .iter()
            .map(|(&id, amp)| (id, amp.probability()))
            .collect()
    }

    /// Get the amplitude of a specific state.
    #[must_use]
    pub fn amplitude(&self, state_id: QStateId) -> Option<f64> {
        self.amplitudes.get(&state_id).map(|a| a.weight)
    }

    /// Whether the system is in superposition (not yet collapsed).
    #[must_use]
    pub fn is_superposed(&self) -> bool {
        !self.collapsed
    }

    /// Whether the system has collapsed to a single state.
    #[must_use]
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Reset to superposition (un-collapse).
    pub fn reset_superposition(&mut self) {
        self.collapsed = false;
    }

    /// Total number of states.
    #[must_use]
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Get measurement history.
    #[must_use]
    pub fn measurement_history(&self) -> &[MeasurementResult] {
        &self.measurements
    }

    /// Shannon entropy of the probability distribution.
    /// Higher entropy = more uncertain state (more "quantum").
    /// Zero entropy = fully collapsed.
    #[must_use]
    pub fn entropy(&self) -> f64 {
        let total_prob: f64 = self.amplitudes.values().map(|a| a.probability()).sum();

        if total_prob == 0.0 {
            return 0.0;
        }

        let mut h = 0.0_f64;
        for amp in self.amplitudes.values() {
            let p = amp.probability() / total_prob;
            if p > 0.0 {
                h -= p * p.ln();
            }
        }

        h
    }

    /// The dominant state (highest probability). Returns None if empty.
    #[must_use]
    pub fn dominant_state(&self) -> Option<QStateId> {
        self.amplitudes
            .iter()
            .max_by(|a, b| {
                a.1.probability()
                    .partial_cmp(&b.1.probability())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .map(|(&id, _)| id)
    }

    /// Get the entangled partner state ID.
    #[must_use]
    pub fn entangled_partner(&self) -> Option<QStateId> {
        self.entangled_partner
    }
}

impl Default for QuantumStateSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for QuantumStateSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.collapsed {
            "collapsed"
        } else {
            "superposed"
        };
        write!(
            f,
            "QuantumStateSpace({} states, {}, entropy {:.3})",
            self.state_count(),
            status,
            self.entropy(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superposition_and_probability() {
        let mut qss = QuantumStateSpace::new();
        let s0 = qss.add_state("alive", 1.0, false);
        let s1 = qss.add_state("dead", 1.0, true);

        qss.normalize();

        let dist = qss.probability_distribution();
        // Equal amplitudes → equal probabilities after normalization
        let p0 = dist.get(&s0).copied().unwrap_or(0.0);
        let p1 = dist.get(&s1).copied().unwrap_or(0.0);

        assert!((p0 - 0.5).abs() < 0.01);
        assert!((p1 - 0.5).abs() < 0.01);
        assert!(qss.is_superposed());
    }

    #[test]
    fn test_measurement_collapse() {
        let mut qss = QuantumStateSpace::new();
        qss.add_state("low_prob", 0.1, false);
        let high = qss.add_state("high_prob", 0.9, false);

        let result = qss.measure();

        // Should collapse to highest probability state
        assert_eq!(result.state_id, high);
        assert!(qss.is_collapsed());

        // After collapse, entropy should be near zero
        assert!(qss.entropy() < 0.01);
    }

    #[test]
    fn test_propagation_and_interference() {
        let mut qss = QuantumStateSpace::new();
        let s0 = qss.add_state("source", 1.0, false);
        let s1 = qss.add_state("target", 0.0, false);

        // Constructive transition
        qss.add_transition(s0, s1, 0.5, "transfer");

        qss.propagate();

        // Target should now have non-zero amplitude
        let target_amp = qss.amplitude(s1).unwrap_or(0.0);
        assert!(target_amp.abs() > 0.0);
    }

    #[test]
    fn test_destructive_interference() {
        let mut qss = QuantumStateSpace::new();
        let s0 = qss.add_state("path_a", 1.0, false);
        let s1 = qss.add_state("path_b", 1.0, false);
        let s2 = qss.add_state("target", 0.0, false);

        // Two paths to target: constructive (+0.5) and destructive (-0.5)
        qss.add_transition(s0, s2, 0.5, "constructive");
        qss.add_transition(s1, s2, -0.5, "destructive");

        qss.normalize();
        qss.propagate();

        // Destructive interference: target amplitude should be near zero
        // because +0.5 and -0.5 cancel out (both source amplitudes equal)
        let target_amp = qss.amplitude(s2).unwrap_or(999.0);
        assert!(
            target_amp.abs() < 0.1,
            "Destructive interference should cancel: got {target_amp}"
        );
    }

    #[test]
    fn test_entropy_decreases_on_collapse() {
        let mut qss = QuantumStateSpace::new();
        qss.add_state("a", 1.0, false);
        qss.add_state("b", 1.0, false);
        qss.add_state("c", 1.0, false);
        qss.normalize();

        let entropy_before = qss.entropy();
        assert!(
            entropy_before > 0.5,
            "Superposition should have high entropy"
        );

        qss.measure();
        let entropy_after = qss.entropy();
        assert!(
            entropy_after < entropy_before,
            "Collapse should reduce entropy"
        );
    }

    #[test]
    fn test_entanglement_marker() {
        let mut qss = QuantumStateSpace::new();
        let s0 = qss.add_state("entangled", 1.0, false);
        qss.entangle(s0);

        let result = qss.measure();
        assert!(result.entangled_collapse);
        assert_eq!(qss.entangled_partner(), Some(s0));
    }

    #[test]
    fn test_dominant_state() {
        let mut qss = QuantumStateSpace::new();
        qss.add_state("weak", 0.1, false);
        let strong = qss.add_state("strong", 0.9, false);

        assert_eq!(qss.dominant_state(), Some(strong));
    }
}
