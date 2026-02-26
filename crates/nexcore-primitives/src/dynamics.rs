//! # Quantum Dynamics (Evolution Operators)
//!
//! Time-evolution rules that animate the static quantum primitives (T2-P/C).
//! This layer bridges state representation and active system control.
//!
//! ## T1 Grounding
//!
//! - `Phasor`: → (Causality) + N (Quantity) + ν (Frequency)
//! - `EnvironmentalCoupling`: ∝ (Irreversibility) + N (Quantity) + → (Causality)
//! - `Interaction`: Σ (Sum) + κ (Comparison)
//! - `Observer`: κ (Comparison) + ∃ (Existence) + ∝ (Irreversibility)
//!
//! ## Transfer Domains
//!
//! | Type | PV Application | Systems Application |
//! |------|---------------|---------------------|
//! | `Phasor` | Signal oscillation in periodic reports | Cyclic workload patterns |
//! | `EnvironmentalCoupling` | Signal decay over time | Cache invalidation rates |
//! | `Interaction` | Constructive/destructive signal combination | Service mesh interference |
//! | `Observer` | Signal triage (collapse to decision) | Load balancer selection |

use crate::quantum::{
    Amplitude, Decoherence, Entanglement, Interference, Measurement, Phase, Qubit, Superposition,
};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::fmt;

// ============================================================================
// 2. Hamiltonian Dynamics (Phase Rotation)
// ============================================================================

/// A rotating vector combining Amplitude and Phase.
///
/// H = ℏω (Energy operator)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Phasor {
    pub amplitude: Amplitude,
    pub phase: Phase,
}

impl Phasor {
    pub fn new(amplitude: Amplitude, phase: Phase) -> Self {
        Self { amplitude, phase }
    }

    /// Evolve the phasor by time dt.
    /// Phase angle increases by 2π * frequency * dt
    pub fn evolve(&mut self, dt: f64) {
        let angular_velocity = 2.0 * PI * self.amplitude.frequency;
        self.phase.angle += angular_velocity * dt;
        // Normalize angle to keep it clean (optional but good practice)
        self.phase.angle %= 2.0 * PI;
    }

    /// Get the complex value (real, imaginary)
    #[must_use]
    pub fn as_complex(&self) -> (f64, f64) {
        let mag = self.amplitude.magnitude;
        let angle = self.phase.angle;
        (mag * angle.cos(), mag * angle.sin())
    }
}

impl fmt::Display for Phasor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Phasor(|A|={:.2}, φ={:.3}rad)",
            self.amplitude.magnitude, self.phase.angle
        )
    }
}

// ============================================================================
// 3. Decoherence Dynamics
// ============================================================================

/// Applies decoherence to a quantum state over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalCoupling {
    pub decoherence: Decoherence,
}

impl EnvironmentalCoupling {
    pub fn new(decoherence: Decoherence) -> Self {
        Self { decoherence }
    }

    /// Apply decoherence to an Amplitude (damping magnitude).
    ///
    /// |A|(t) = |A|(0) * e^(-t/T2)
    pub fn dampen(&self, amplitude: &mut Amplitude, dt: f64) {
        let decay_factor = (-dt * self.decoherence.decay_rate).exp();
        amplitude.magnitude *= decay_factor;
    }

    /// Apply decoherence to a Superposition (damping off-diagonal terms - purity loss).
    /// Simpler model: dampen all weights (non-unitary loss).
    pub fn degrade_superposition(&self, superposition: &mut Superposition, dt: f64) {
        let decay_factor = (-dt * self.decoherence.decay_rate).exp();
        for weight in &mut superposition.weights {
            *weight *= decay_factor;
        }
        // Note: This makes the sum < 1.0, representing information loss to environment.
    }
}

impl fmt::Display for EnvironmentalCoupling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnvCoupling(γ={:.3})", self.decoherence.decay_rate)
    }
}

// ============================================================================
// 4. Interaction Gates (Coupling)
// ============================================================================

/// Operators that couple multiple states.
pub struct Interaction;

impl Interaction {
    /// Interfere two Phasors to produce a composite `Interference` value.
    ///
    /// Constructs the `Interference` from the Phasors' amplitude magnitudes and phase
    /// angles. The `Interference` type computes resultant amplitude and constructive/
    /// destructive classification on demand via its own methods.
    #[must_use]
    pub fn interfere(p1: &Phasor, p2: &Phasor) -> Interference {
        Interference {
            amplitudes: vec![p1.amplitude.magnitude, p2.amplitude.magnitude],
            phases: vec![p1.phase.angle, p2.phase.angle],
        }
    }

    /// Entangle two Qubits into a Bell State (simplified model).
    ///
    /// Returns an `Entanglement` handle linking the two qubits by their field labels.
    /// The `subsystem_a` and `subsystem_b` fields capture which qubits are entangled.
    /// In a full simulation the four-component tensor product state would be tracked;
    /// here the T2-C metadata primitive captures the concurrence parameter.
    #[must_use]
    pub fn entangle(q1: &Qubit, q2: &Qubit, concurrence: f64) -> Entanglement {
        // Label the entangled pair by their Bloch-sphere angle to distinguish them.
        let label_a = format!("q(θ={:.3})", q1.theta());
        let label_b = format!("q(θ={:.3})", q2.theta());
        Entanglement::new(label_a, label_b, concurrence)
    }
}

// ============================================================================
// 5. Measurement Operator (Collapse)
// ============================================================================

/// The act of observing a quantum state.
pub struct Observer;

impl Observer {
    /// Measure a Superposition, collapsing it to a single state based on weights.
    ///
    /// Uses a random seed (in a real system) or deterministic selector for simulation.
    /// Here we implement a "max likelihood" collapse for deterministic behavior,
    /// or we could take a selector `rand: f64` [0.0, 1.0).
    #[must_use]
    pub fn measure(superposition: &Superposition, selector: f64) -> Measurement {
        let total_weight = superposition.total_weight();
        if total_weight <= 0.0 {
            return Measurement::new("null", 0.0);
        }

        let normalized_selector = selector * total_weight;
        let mut cumulative = 0.0;

        for (i, weight) in superposition.weights.iter().enumerate() {
            cumulative += weight;
            if cumulative >= normalized_selector {
                let label = superposition
                    .labels
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| i.to_string());
                let prob = weight / total_weight;
                let mut m = Measurement::new(label, prob);
                m.collapse();
                return m;
            }
        }

        // Fallback to last
        if let Some(last_w) = superposition.weights.last() {
            let label = superposition
                .labels
                .last()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let prob = last_w / total_weight;
            let mut m = Measurement::new(label, prob);
            m.collapse();
            return m;
        }

        Measurement::new("void", 0.0)
    }

    /// Measure a Qubit (Z-basis measurement).
    #[must_use]
    pub fn measure_qubit(qubit: &Qubit, selector: f64) -> Measurement {
        let p0 = qubit.prob_zero();
        // selector is [0.0, 1.0)
        if selector < p0 {
            let mut m = Measurement::new("0", p0);
            m.collapse();
            m
        } else {
            let mut m = Measurement::new("1", qubit.prob_one());
            m.collapse();
            m
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phasor_evolution() {
        let amp = Amplitude::new(1.0, 1.0); // f=1.0
        let phase = Phase::new(0.0, 0.0);
        let mut p = Phasor::new(amp, phase);

        // Evolve by 0.25 seconds (1/4 cycle) -> 90 degrees (PI/2)
        p.evolve(0.25);

        assert!((p.phase.angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_environmental_damping() {
        let mut amp = Amplitude::new(1.0, 1.0);
        let decay = Decoherence::new(10.0, "test"); // T2 = 10.0
        let env = EnvironmentalCoupling::new(decay);

        // Evolve by T2 -> amplitude should be 1/e
        env.dampen(&mut amp, 10.0);

        assert!((amp.magnitude - (-1.0_f64).exp()).abs() < 1e-5);
    }

    #[test]
    fn test_superposition_measurement() {
        let s = Superposition::new(vec![0.2, 0.8], vec!["A".to_string(), "B".to_string()]);

        // Select low range -> A
        let m1 = Observer::measure(&s, 0.1); // 0.1 * 1.0 = 0.1 < 0.2
        assert_eq!(m1.outcome, "A");
        assert!(m1.is_collapsed());

        // Select high range -> B
        let m2 = Observer::measure(&s, 0.5); // 0.5 > 0.2
        assert_eq!(m2.outcome, "B");
    }

    #[test]
    fn test_qubit_measurement() {
        // Equal superposition
        let s = 1.0 / 2.0_f64.sqrt();
        let q = Qubit::new(s, s);

        let m0 = Observer::measure_qubit(&q, 0.1); // 0.1 < 0.5
        assert_eq!(m0.outcome, "0");

        let m1 = Observer::measure_qubit(&q, 0.6); // 0.6 > 0.5
        assert_eq!(m1.outcome, "1");
    }

    #[test]
    fn test_phasor_complex_conversion() {
        let amp = Amplitude::new(2.0, 0.0);
        let phase = Phase::new(PI, 0.0); // 180 degrees
        let p = Phasor::new(amp, phase);

        let (re, im) = p.as_complex();
        // Should be -2.0 + 0i
        assert!((re - -2.0).abs() < 1e-10);
        assert!(im.abs() < 1e-10);
    }

    #[test]
    fn test_interaction_interfere_in_phase() {
        // Two in-phase phasors (angle=0): should produce constructive interference
        let p1 = Phasor::new(Amplitude::new(1.0, 1.0), Phase::new(0.0, 0.0));
        let p2 = Phasor::new(Amplitude::new(1.0, 1.0), Phase::new(0.0, 0.0));
        let i = Interaction::interfere(&p1, &p2);

        // Resultant: both vectors point same direction, amplitude = 2.0
        assert!((i.resultant_amplitude() - 2.0).abs() < 1e-10);
        assert!(i.is_constructive());
    }

    #[test]
    fn test_interaction_interfere_out_of_phase() {
        // Anti-phase phasors (180 degrees apart): destructive interference
        let p1 = Phasor::new(Amplitude::new(1.0, 1.0), Phase::new(0.0, 0.0));
        let p2 = Phasor::new(Amplitude::new(1.0, 1.0), Phase::new(PI, 0.0));
        let i = Interaction::interfere(&p1, &p2);

        // Resultant amplitude ≈ 0 (complete destructive)
        assert!(i.resultant_amplitude() < 1e-10);
        assert!(!i.is_constructive());
    }

    #[test]
    fn test_interaction_interfere_carries_input_amplitudes() {
        let p1 = Phasor::new(Amplitude::new(2.0, 1.0), Phase::new(0.0, 0.0));
        let p2 = Phasor::new(Amplitude::new(3.0, 1.0), Phase::new(0.0, 0.0));
        let i = Interaction::interfere(&p1, &p2);

        // The Interference records the source magnitudes
        assert_eq!(i.amplitudes.len(), 2);
        assert!((i.amplitudes[0] - 2.0).abs() < f64::EPSILON);
        assert!((i.amplitudes[1] - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interaction_entangle_links_qubits() {
        let q1 = Qubit::zero();
        let q2 = Qubit::one();
        let concurrence = 0.85;
        let ent = Interaction::entangle(&q1, &q2, concurrence);

        // Concurrence is preserved
        assert!((ent.concurrence - concurrence).abs() < f64::EPSILON);
        // Labels distinguish the two qubits by their Bloch angles
        assert_ne!(ent.subsystem_a, ent.subsystem_b);
    }

    #[test]
    fn test_interaction_entangle_clamps_concurrence() {
        let q1 = Qubit::zero();
        let q2 = Qubit::zero();
        // Over-range concurrence is clamped by Entanglement::new
        let ent = Interaction::entangle(&q1, &q2, 1.5);
        assert!((ent.concurrence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_degrade_superposition_reduces_weights() {
        let decay = Decoherence::new(1.0, "test");
        let env = EnvironmentalCoupling::new(decay);
        let mut s = Superposition::new(vec![0.5, 0.5], vec!["A".to_string(), "B".to_string()]);
        let total_before = s.total_weight();
        env.degrade_superposition(&mut s, 1.0);
        let total_after = s.total_weight();
        assert!(
            total_after < total_before,
            "Decoherence should reduce total weight"
        );
        assert!(
            total_after > 0.0,
            "Should not reduce to zero in finite time"
        );
    }

    #[test]
    fn test_observer_measure_empty_superposition() {
        let s = Superposition::new(vec![], vec![]);
        let m = Observer::measure(&s, 0.5);
        assert_eq!(m.outcome, "null");
    }

    #[test]
    fn test_phasor_full_cycle_returns_near_origin() {
        let amp = Amplitude::new(1.0, 1.0);
        let phase = Phase::new(0.0, 0.0);
        let mut p = Phasor::new(amp, phase);
        p.evolve(1.0); // Full cycle at f=1.0
        // Phase should be back near 0 (modulo 2π normalization = 0)
        assert!(p.phase.angle.abs() < 1e-10 || (p.phase.angle - 2.0 * PI).abs() < 1e-10);
    }
}
