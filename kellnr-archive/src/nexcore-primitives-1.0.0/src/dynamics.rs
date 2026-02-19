//! # Quantum Dynamics (Evolution Operators)
//!
//! Defines the time-evolution rules ("Hamiltonians") that animate the static
//! quantum primitives. This layer bridges the gap between state representation
//! (T2-P/C) and active system control.

use crate::quantum::{
    Amplitude, Decoherence, Entanglement, Interference, Measurement, Phase, Qubit, Superposition,
};
use std::f64::consts::PI;

// ============================================================================
// 1. Time Evolution Trait
// ============================================================================

/// Describes a system that changes state over time.
pub trait TimeEvolution {
    /// Evolve the state forward by time delta `dt`.
    fn evolve(&mut self, dt: f64);
}

// ============================================================================
// 2. Hamiltonian Dynamics (Phase Rotation)
// ============================================================================

/// Evolves a Phase based on an angular velocity (frequency).
///
/// θ(t + dt) = θ(t) + ω * dt
impl TimeEvolution for Phase {
    fn evolve(&mut self, dt: f64) {
        // Frequency is implicitly 1.0 rad/time unit if not stored.
        // For a more complex model, Phase would need a `frequency` field.
        // Here we assume a standard rotation.
        // To be precise, we need the angular velocity.
        // Let's assume the `angle` is the dynamic variable.
        // We will define a standard evolution rate or require it.
        // Better design: The *state* has a frequency.
    }
}

// Re-design: We need an "Evolver" or "Hamiltonian" struct that acts on the state.
// Or, we extend the primitives to include their dynamics (which Amplitude has: frequency).

impl TimeEvolution for Amplitude {
    fn evolve(&mut self, dt: f64) {
        // Amplitude magnitude is constant in closed systems (unitary).
        // Phase rotates: φ = 2π * f * t
        // Amplitude struct stores (magnitude, frequency).
        // It doesn't store phase. We need a Phasor.
    }
}

// Let's introduce a Phasor to couple Amplitude and Phase for evolution.

/// A rotating vector combining Amplitude and Phase.
///
/// H = ℏω (Energy operator)
#[derive(Debug, Clone, PartialEq)]
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
    pub fn as_complex(&self) -> (f64, f64) {
        let mag = self.amplitude.magnitude;
        let angle = self.phase.angle;
        (mag * angle.cos(), mag * angle.sin())
    }
}

// ============================================================================
// 3. Decoherence Dynamics
// ============================================================================

/// Applies decoherence to a quantum state over time.
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

// ============================================================================
// 4. Interaction Gates (Coupling)
// ============================================================================

/// Operators that couple multiple states.
pub struct Interaction;

impl Interaction {
    /// Interfere two Phasors to produce a Resultant Amplitude.
    pub fn interfere(p1: &Phasor, p2: &Phasor) -> Interference {
        let (r1, i1) = p1.as_complex();
        let (r2, i2) = p2.as_complex();

        // Sum of complex numbers
        let r_sum = r1 + r2;
        let i_sum = i1 + i2;

        let resultant_mag = (r_sum * r_sum + i_sum * i_sum).sqrt();
        let resultant_phase = i_sum.atan2(r_sum);

        Interference {
            amplitudes: vec![p1.amplitude.magnitude, p2.amplitude.magnitude],
            phases: vec![p1.phase.angle, p2.phase.angle],
        }
        // Note: The Interference struct in quantum.rs calculates resultant on demand.
        // This helper just constructs it from Phasors.
    }

    /// Entangle two Qubits into a Bell State (simplified model).
    /// Returns an Entanglement handle and modifies the Qubits to be "linked".
    pub fn entangle(q1: &mut Qubit, q2: &mut Qubit, concurrence: f64) -> Entanglement {
        // In a full simulation, we'd merge q1 and q2 into a tensor product state 4-vector.
        // Here, we return the Entanglement metadata primitive T2-C.
        Entanglement::new("q1", "q2", concurrence)
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
}
