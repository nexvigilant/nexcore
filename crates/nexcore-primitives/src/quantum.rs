//! # Quantum Domain Primitives (10 T2-P, 3 T2-C)
//!
//! Universal computational patterns extracted from quantum mechanics.
//! Structured as a Directed Acyclic Graph (DAG) of dependencies, grounding
//! high-level quantum concepts into foundational Lex Primitiva (T1).
//!
//! ## Hierarchy
//! - **Layer 1 (Wave Foundations):** Amplitude, Phase, Interference
//! - **Layer 2 (Operators & Symmetry):** Eigenstate, Observable, Hermiticity, Unitarity
//! - **Layer 3 (State & Uncertainty):** Superposition, Uncertainty, Measurement
//! - **Layer 4 (Quantum Core):** Qubit, Entanglement, Decoherence

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Macro: Total Ordering for f64 Structs
// ============================================================================

macro_rules! impl_float_total_ord {
    ($name:ident { $($field:ident : $kind:tt),+ $(,)? }) => {
        impl Eq for $name {}

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                $(impl_float_total_ord!(@hash self, state, $field, $kind);)+
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                std::cmp::Ordering::Equal
                    $(.then_with(|| impl_float_total_ord!(@cmp self, other, $field, $kind)))+
            }
        }
    };

    // Hash arms
    (@hash $self:ident, $state:ident, $field:ident, float) => {
        std::hash::Hash::hash(&$self.$field.to_bits(), $state);
    };
    (@hash $self:ident, $state:ident, $field:ident, ord) => {
        std::hash::Hash::hash(&$self.$field, $state);
    };
    (@hash $self:ident, $state:ident, $field:ident, vec_float) => {
        std::hash::Hash::hash(&$self.$field.len(), $state);
        for v in &$self.$field {
            std::hash::Hash::hash(&v.to_bits(), $state);
        }
    };

    // Cmp arms
    (@cmp $self:ident, $other:ident, $field:ident, float) => {
        f64::total_cmp(&$self.$field, &$other.$field)
    };
    (@cmp $self:ident, $other:ident, $field:ident, ord) => {
        Ord::cmp(&$self.$field, &$other.$field)
    };
    (@cmp $self:ident, $other:ident, $field:ident, vec_float) => {{
        let len_ord = Ord::cmp(&$self.$field.len(), &$other.$field.len());
        if len_ord != std::cmp::Ordering::Equal {
            len_ord
        } else {
            $self.$field.iter().zip($other.$field.iter())
                .map(|(a, b)| f64::total_cmp(a, b))
                .find(|o| *o != std::cmp::Ordering::Equal)
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    }};
}

// ============================================================================
// Layer 1: Wave Foundations
// Grounding: N (Quantity), ν (Frequency), κ (Comparison), Σ (Sum)
// ============================================================================

pub mod wave {
    use super::*;

    /// A magnitude associated with a frequency component.
    /// Grounding: N (Quantity) + ν (Frequency)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Amplitude {
        pub magnitude: f64,
        pub frequency: f64,
    }

    impl Amplitude {
        pub fn new(magnitude: f64, frequency: f64) -> Self {
            Self {
                magnitude: magnitude.max(0.0),
                frequency: frequency.max(0.0),
            }
        }
        pub fn probability(&self) -> f64 {
            self.magnitude * self.magnitude
        }
        pub fn energy(&self) -> f64 {
            self.frequency * self.probability()
        }
    }

    impl fmt::Display for Amplitude {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "|A|={:.4} @ f={:.2} (P={:.4})",
                self.magnitude,
                self.frequency,
                self.probability()
            )
        }
    }

    /// An angular offset relative to a reference.
    /// Grounding: ν (Frequency) + κ (Comparison)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Phase {
        pub angle: f64,
        pub reference: f64,
    }

    impl Phase {
        pub fn new(angle: f64, reference: f64) -> Self {
            Self { angle, reference }
        }
        pub fn difference(&self) -> f64 {
            let diff = self.angle - self.reference;
            let two_pi = 2.0 * std::f64::consts::PI;
            (diff + std::f64::consts::PI).rem_euclid(two_pi) - std::f64::consts::PI
        }
        pub fn is_aligned(&self, tolerance: f64) -> bool {
            self.difference().abs() < tolerance
        }
        pub fn interference_factor(&self) -> f64 {
            self.difference().cos()
        }
    }

    impl fmt::Display for Phase {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "φ={:.3}rad (Δ={:.3}rad)", self.angle, self.difference())
        }
    }

    /// Combination of wave-like contributions.
    /// Grounding: Σ (Sum) + ν (Frequency) + N (Quantity)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Interference {
        pub amplitudes: Vec<f64>,
        pub phases: Vec<f64>,
    }

    impl Interference {
        pub fn new(amplitudes: Vec<f64>, phases: Vec<f64>) -> Self {
            let len = amplitudes.len().min(phases.len());
            Self {
                amplitudes: amplitudes[..len].to_vec(),
                phases: phases[..len].to_vec(),
            }
        }
        pub fn resultant_amplitude(&self) -> f64 {
            let real: f64 = self
                .amplitudes
                .iter()
                .zip(self.phases.iter())
                .map(|(a, p)| a * p.cos())
                .sum();
            let imag: f64 = self
                .amplitudes
                .iter()
                .zip(self.phases.iter())
                .map(|(a, p)| a * p.sin())
                .sum();
            (real * real + imag * imag).sqrt()
        }
        pub fn max_amplitude(&self) -> f64 {
            self.amplitudes.iter().map(|a| a.abs()).sum()
        }
        pub fn efficiency(&self) -> f64 {
            let max = self.max_amplitude();
            if max <= 0.0 {
                return 0.0;
            }
            self.resultant_amplitude() / max
        }
        pub fn is_constructive(&self) -> bool {
            self.efficiency() > 0.5
        }
    }

    impl fmt::Display for Interference {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let kind = if self.is_constructive() {
                "constructive"
            } else {
                "destructive"
            };
            write!(
                f,
                "I={:.4} ({}, eff={:.1}%)",
                self.resultant_amplitude(),
                kind,
                self.efficiency() * 100.0
            )
        }
    }
}

// ============================================================================
// Layer 2: Operators & Symmetry
// Grounding: π (Persistence), μ (Mapping), κ (Comparison), ρ (Recursion)
// ============================================================================

pub mod operators {
    use super::*;

    /// A stable state unchanged by an operation.
    /// Grounding: π (Persistence) + κ (Comparison) + μ (Mapping)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Eigenstate {
        pub eigenvalue: f64,
        pub label: String,
    }

    impl Eigenstate {
        pub fn new(eigenvalue: f64, label: impl Into<String>) -> Self {
            Self {
                eigenvalue,
                label: label.into(),
            }
        }
        pub fn is_fixed_point(&self, tolerance: f64) -> bool {
            (self.eigenvalue - 1.0).abs() < tolerance
        }
        pub fn is_stable(&self) -> bool {
            self.eigenvalue.abs() <= 1.0
        }
        pub fn growth_rate(&self) -> f64 {
            self.eigenvalue - 1.0
        }
    }

    impl fmt::Display for Eigenstate {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let stability = if self.is_stable() {
                "stable"
            } else {
                "unstable"
            };
            write!(
                f,
                "|{}> λ={:.4} [{}]",
                self.label, self.eigenvalue, stability
            )
        }
    }

    /// A measurable property with discrete outcomes.
    /// Grounding: μ (Mapping) + N (Quantity) + κ (Comparison)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Observable {
        pub name: String,
        pub eigenvalues: Vec<f64>,
    }

    impl Observable {
        pub fn new(name: impl Into<String>, eigenvalues: Vec<f64>) -> Self {
            Self {
                name: name.into(),
                eigenvalues,
            }
        }
        pub fn num_outcomes(&self) -> usize {
            self.eigenvalues.len()
        }
        pub fn range(&self) -> f64 {
            if self.eigenvalues.is_empty() {
                return 0.0;
            }
            let min = self
                .eigenvalues
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b));
            let max = self
                .eigenvalues
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            max - min
        }
        pub fn mean(&self) -> f64 {
            if self.eigenvalues.is_empty() {
                return 0.0;
            }
            self.eigenvalues.iter().sum::<f64>() / self.eigenvalues.len() as f64
        }
        pub fn variance(&self) -> f64 {
            if self.eigenvalues.is_empty() {
                return 0.0;
            }
            let m = self.mean();
            self.eigenvalues
                .iter()
                .map(|v| (v - m).powi(2))
                .sum::<f64>()
                / self.eigenvalues.len() as f64
        }
        pub fn std_dev(&self) -> f64 {
            self.variance().sqrt()
        }
    }

    impl fmt::Display for Observable {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "O({}) outcomes={} range={:.2}",
                self.name,
                self.num_outcomes(),
                self.range()
            )
        }
    }

    /// Self-adjointness: operation equals its transpose conjugate.
    /// Grounding: κ (Comparison) + μ (Mapping) + π (Persistence)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Hermiticity {
        pub is_hermitian: bool,
        pub deviation: f64,
    }

    impl Hermiticity {
        pub fn new(is_hermitian: bool, deviation: f64) -> Self {
            Self {
                is_hermitian,
                deviation: deviation.max(0.0),
            }
        }
        pub fn within_tolerance(&self, tolerance: f64) -> bool {
            self.deviation <= tolerance
        }
        pub fn symmetry_score(&self) -> f64 {
            1.0 / (1.0 + self.deviation)
        }
    }

    impl fmt::Display for Hermiticity {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let sym = if self.is_hermitian {
                "Hermitian"
            } else {
                "non-Hermitian"
            };
            write!(f, "{} (δ={:.6})", sym, self.deviation)
        }
    }

    /// Information-preserving transformation.
    /// Grounding: π (Persistence) + ρ (Recursion) + N (Quantity)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Unitarity {
        pub fidelity: f64,
        pub dimension: usize,
    }

    impl Unitarity {
        pub fn new(fidelity: f64, dimension: usize) -> Self {
            Self {
                fidelity: fidelity.clamp(0.0, 1.0),
                dimension: dimension.max(1),
            }
        }
        pub fn information_loss(&self) -> f64 {
            1.0 - self.fidelity
        }
        pub fn is_lossless(&self, threshold: f64) -> bool {
            self.fidelity >= threshold
        }
        pub fn max_entropy(&self) -> f64 {
            (self.dimension as f64).log2()
        }
    }

    impl fmt::Display for Unitarity {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "U(d={}) fidelity={:.4} loss={:.4}",
                self.dimension,
                self.fidelity,
                self.information_loss()
            )
        }
    }
}

// ============================================================================
// Layer 3: State & Uncertainty
// Grounding: ∃ (Existence), Σ (Sum), ∂ (Boundary), → (Causality)
// ============================================================================

pub mod state {
    use super::*;

    /// A system existing in multiple states simultaneously.
    /// Grounding: ∃ (Existence) + Σ (Sum) + ∂ (Boundary)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Superposition {
        pub weights: Vec<f64>,
        pub labels: Vec<String>,
    }

    impl Superposition {
        pub fn new(weights: Vec<f64>, labels: Vec<String>) -> Self {
            let len = weights.len().min(labels.len());
            Self {
                weights: weights[..len].to_vec(),
                labels: labels[..len].to_vec(),
            }
        }
        pub fn num_states(&self) -> usize {
            self.weights.len()
        }
        pub fn total_weight(&self) -> f64 {
            self.weights.iter().sum()
        }
        pub fn is_normalized(&self, tolerance: f64) -> bool {
            (self.total_weight() - 1.0).abs() < tolerance
        }
        pub fn most_probable(&self) -> Option<usize> {
            self.weights
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
        }
        pub fn entropy(&self) -> f64 {
            let total = self.total_weight();
            if total <= 0.0 {
                return 0.0;
            }
            self.weights
                .iter()
                .filter(|&&w| w > 0.0)
                .map(|w| {
                    let p = w / total;
                    -p * p.log2()
                })
                .sum()
        }
    }

    impl fmt::Display for Superposition {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Ψ = ")?;
            for (i, (w, l)) in self.weights.iter().zip(self.labels.iter()).enumerate() {
                if i > 0 {
                    write!(f, " + ")?;
                }
                write!(f, "{:.2}|{}>", w, l)?;
            }
            Ok(())
        }
    }

    /// Irreversible observation collapsing possibilities.
    /// Grounding: ∂ (Boundary) + → (Causality) + ∝ (Irreversibility)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Measurement {
        pub outcome: String,
        pub probability: f64,
        pub collapsed: bool,
    }

    impl Measurement {
        pub fn new(outcome: impl Into<String>, probability: f64) -> Self {
            Self {
                outcome: outcome.into(),
                probability: probability.clamp(0.0, 1.0),
                collapsed: false,
            }
        }
        pub fn collapse(&mut self) {
            self.collapsed = true;
        }
        pub fn is_likely(&self, threshold: f64) -> bool {
            self.probability >= threshold
        }
        pub fn is_collapsed(&self) -> bool {
            self.collapsed
        }
    }

    impl fmt::Display for Measurement {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let state = if self.collapsed {
                "collapsed"
            } else {
                "pending"
            };
            write!(
                f,
                "M({}) p={:.3} [{}]",
                self.outcome, self.probability, state
            )
        }
    }

    /// Fundamental limit on simultaneous precision.
    /// Grounding: ∂ (Boundary) + κ (Comparison) + ν (Frequency)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Uncertainty {
        pub position_spread: f64,
        pub momentum_spread: f64,
    }

    impl Uncertainty {
        pub fn new(position_spread: f64, momentum_spread: f64) -> Self {
            Self {
                position_spread: position_spread.max(0.0),
                momentum_spread: momentum_spread.max(0.0),
            }
        }
        pub fn product(&self) -> f64 {
            self.position_spread * self.momentum_spread
        }
        pub fn satisfies_bound(&self, min_product: f64) -> bool {
            self.product() >= min_product
        }
        pub fn balance(&self) -> f64 {
            let max = self.position_spread.max(self.momentum_spread);
            if max <= 0.0 {
                0.0
            } else {
                self.position_spread.min(self.momentum_spread) / max
            }
        }
    }

    impl fmt::Display for Uncertainty {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Δx={:.4} Δp={:.4} (ΔxΔp={:.4})",
                self.position_spread,
                self.momentum_spread,
                self.product()
            )
        }
    }
}

// ============================================================================
// Layer 4: Quantum Domain Core (T2-C)
// Grounding: ς (State), λ (Location), ∝ (Irreversibility)
// ============================================================================

pub mod domain {
    use super::*;

    /// Two-level system parameterized on the Bloch sphere.
    /// Grounding: ς (State) + ∂ (Boundary) + Σ (Sum) + N (Quantity)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Qubit {
        pub alpha: f64,
        pub beta: f64,
    }

    impl Qubit {
        pub fn new(alpha: f64, beta: f64) -> Self {
            Self { alpha, beta }
        }
        pub fn zero() -> Self {
            Self {
                alpha: 1.0,
                beta: 0.0,
            }
        }
        pub fn one() -> Self {
            Self {
                alpha: 0.0,
                beta: 1.0,
            }
        }
        pub fn hadamard() -> Self {
            let s = 1.0 / 2.0_f64.sqrt();
            Self { alpha: s, beta: s }
        }
        pub fn prob_zero(&self) -> f64 {
            self.alpha * self.alpha
        }
        pub fn prob_one(&self) -> f64 {
            self.beta * self.beta
        }
        pub fn total_probability(&self) -> f64 {
            self.prob_zero() + self.prob_one()
        }
        pub fn is_normalized(&self, tolerance: f64) -> bool {
            (self.total_probability() - 1.0).abs() < tolerance
        }
        pub fn theta(&self) -> f64 {
            2.0 * self.beta.atan2(self.alpha)
        }
        pub fn purity(&self) -> f64 {
            let tp = self.total_probability();
            tp * tp
        }
    }

    impl fmt::Display for Qubit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{:.4}|0> + {:.4}|1> (P0={:.3}, P1={:.3})",
                self.alpha,
                self.beta,
                self.prob_zero(),
                self.prob_one()
            )
        }
    }

    /// Non-local correlation between subsystems.
    /// Grounding: → (Causality) + λ (Location) + κ (Comparison) + ρ (Recursion)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Entanglement {
        pub subsystem_a: String,
        pub subsystem_b: String,
        pub concurrence: f64,
    }

    impl Entanglement {
        pub fn new(
            subsystem_a: impl Into<String>,
            subsystem_b: impl Into<String>,
            concurrence: f64,
        ) -> Self {
            Self {
                subsystem_a: subsystem_a.into(),
                subsystem_b: subsystem_b.into(),
                concurrence: concurrence.clamp(0.0, 1.0),
            }
        }
        pub fn is_separable(&self, tolerance: f64) -> bool {
            self.concurrence < tolerance
        }
        pub fn is_maximal(&self, tolerance: f64) -> bool {
            (self.concurrence - 1.0).abs() < tolerance
        }
        pub fn entropy(&self) -> f64 {
            let c = self.concurrence;
            if c <= 0.0 || c >= 1.0 {
                0.0
            } else {
                -(c * c.log2() + (1.0 - c) * (1.0 - c).log2())
            }
        }
    }

    impl fmt::Display for Entanglement {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "E({}, {}) C={:.4}",
                self.subsystem_a, self.subsystem_b, self.concurrence
            )
        }
    }

    /// Loss of quantum coherence.
    /// Grounding: ∝ (Irreversibility) + ∂ (Boundary) + ν (Frequency) + → (Causality)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Decoherence {
        pub coherence_time: f64,
        pub decay_rate: f64,
        pub channel: String,
    }

    impl Decoherence {
        pub fn new(coherence_time: f64, channel: impl Into<String>) -> Self {
            let ct = coherence_time.max(f64::EPSILON);
            Self {
                coherence_time: ct,
                decay_rate: 1.0 / ct,
                channel: channel.into(),
            }
        }
        pub fn coherence_at(&self, t: f64) -> f64 {
            (-t * self.decay_rate).exp()
        }
        pub fn time_to_threshold(&self, threshold: f64) -> f64 {
            if threshold <= 0.0 || threshold >= 1.0 {
                0.0
            } else {
                -threshold.ln() / self.decay_rate
            }
        }
        pub fn is_decohered(&self, t: f64, threshold: f64) -> bool {
            self.coherence_at(t) < threshold
        }
    }

    impl fmt::Display for Decoherence {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "T2={:.4} ({}, rate={:.4})",
                self.coherence_time, self.channel, self.decay_rate
            )
        }
    }
}

// ============================================================================
// Public Re-exports
// ============================================================================

pub use domain::{Decoherence, Entanglement, Qubit};
pub use operators::{Eigenstate, Hermiticity, Observable, Unitarity};
pub use state::{Measurement, Superposition, Uncertainty};
pub use wave::{Amplitude, Interference, Phase};

// ============================================================================
// Implementation of Total Ordering
// ============================================================================

impl_float_total_ord!(Amplitude {
    magnitude: float,
    frequency: float
});
impl_float_total_ord!(Phase {
    angle: float,
    reference: float
});
impl_float_total_ord!(Interference {
    amplitudes: vec_float,
    phases: vec_float
});

impl_float_total_ord!(Eigenstate {
    eigenvalue: float,
    label: ord
});
impl_float_total_ord!(Observable {
    name: ord,
    eigenvalues: vec_float
});
impl_float_total_ord!(Hermiticity {
    is_hermitian: ord,
    deviation: float
});
impl_float_total_ord!(Unitarity {
    fidelity: float,
    dimension: ord
});

impl_float_total_ord!(Superposition {
    weights: vec_float,
    labels: ord
});
impl_float_total_ord!(Measurement {
    outcome: ord,
    probability: float,
    collapsed: ord
});
impl_float_total_ord!(Uncertainty {
    position_spread: float,
    momentum_spread: float
});

impl_float_total_ord!(Qubit {
    alpha: float,
    beta: float
});
impl_float_total_ord!(Entanglement {
    subsystem_a: ord,
    subsystem_b: ord,
    concurrence: float
});
impl_float_total_ord!(Decoherence {
    coherence_time: float,
    decay_rate: float,
    channel: ord
});

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Amplitude tests

    #[test]
    fn test_amplitude_probability() {
        let a = Amplitude::new(0.5, 1.0);
        assert!((a.probability() - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_amplitude_energy() {
        let a = Amplitude::new(2.0, 3.0);
        // energy = frequency * |amplitude|^2 = 3.0 * 4.0 = 12.0
        assert!((a.energy() - 12.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_amplitude_clamps_negative() {
        let a = Amplitude::new(-1.0, -2.0);
        assert!((a.magnitude - 0.0).abs() < f64::EPSILON);
        assert!((a.frequency - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_amplitude_display() {
        let a = Amplitude::new(0.5, 1.0);
        let s = format!("{}", a);
        assert!(s.contains("|A|="));
        assert!(s.contains("P="));
    }

    // Phase tests

    #[test]
    fn test_phase_aligned() {
        let p = Phase::new(0.1, 0.0);
        assert!(p.is_aligned(0.2));
        assert!(!p.is_aligned(0.05));
    }

    #[test]
    fn test_phase_difference_normalized() {
        let p = Phase::new(7.0, 0.0); // > 2π
        let diff = p.difference();
        assert!(diff >= -std::f64::consts::PI && diff <= std::f64::consts::PI);
    }

    #[test]
    fn test_phase_large_angle_precision() {
        // 1000 * 2π + 0.1 should normalize to ~0.1
        let large = 1000.0 * 2.0 * std::f64::consts::PI + 0.1;
        let p = Phase::new(large, 0.0);
        let diff = p.difference();
        assert!(
            (diff - 0.1).abs() < 1e-8,
            "Large angle normalized to {} (expected ~0.1)",
            diff
        );
    }

    #[test]
    fn test_phase_negative_large_angle() {
        // -1000 * 2π - 0.1 should normalize to ~-0.1
        let large = -1000.0 * 2.0 * std::f64::consts::PI - 0.1;
        let p = Phase::new(large, 0.0);
        let diff = p.difference();
        assert!(
            (diff - (-0.1)).abs() < 1e-8,
            "Negative large angle normalized to {} (expected ~-0.1)",
            diff
        );
    }

    #[test]
    fn test_phase_constructive_interference() {
        let p = Phase::new(0.0, 0.0);
        assert!((p.interference_factor() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_phase_destructive_interference() {
        let p = Phase::new(std::f64::consts::PI, 0.0);
        assert!((p.interference_factor() - (-1.0)).abs() < 1e-10);
    }

    // Superposition tests

    #[test]
    fn test_superposition_normalized() {
        let s = Superposition::new(
            vec![0.5, 0.3, 0.2],
            vec!["a".into(), "b".into(), "c".into()],
        );
        assert!(s.is_normalized(0.01));
        assert_eq!(s.num_states(), 3);
    }

    #[test]
    fn test_superposition_most_probable() {
        let s = Superposition::new(
            vec![0.1, 0.7, 0.2],
            vec!["a".into(), "b".into(), "c".into()],
        );
        assert_eq!(s.most_probable(), Some(1));
    }

    #[test]
    fn test_superposition_entropy() {
        // Uniform distribution over 2 states: entropy = 1.0 bit
        let s = Superposition::new(vec![0.5, 0.5], vec!["0".into(), "1".into()]);
        assert!((s.entropy() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_superposition_mismatched_lengths() {
        let s = Superposition::new(vec![0.5, 0.5, 0.0], vec!["a".into(), "b".into()]);
        assert_eq!(s.num_states(), 2);
    }

    // Measurement tests

    #[test]
    fn test_measurement_collapse() {
        let mut m = Measurement::new("spin-up", 0.7);
        assert!(!m.is_collapsed());
        m.collapse();
        assert!(m.is_collapsed());
    }

    #[test]
    fn test_measurement_likely() {
        let m = Measurement::new("detected", 0.8);
        assert!(m.is_likely(0.5));
        assert!(!m.is_likely(0.9));
    }

    #[test]
    fn test_measurement_clamps_probability() {
        let m = Measurement::new("test", 1.5);
        assert!((m.probability - 1.0).abs() < f64::EPSILON);
    }

    // Interference tests

    #[test]
    fn test_interference_constructive() {
        let i = Interference::new(vec![1.0, 1.0], vec![0.0, 0.0]);
        assert!((i.resultant_amplitude() - 2.0).abs() < 1e-10);
        assert!(i.is_constructive());
        assert!((i.efficiency() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_interference_destructive() {
        let i = Interference::new(vec![1.0, 1.0], vec![0.0, std::f64::consts::PI]);
        assert!(i.resultant_amplitude() < 1e-10);
        assert!(!i.is_constructive());
    }

    #[test]
    fn test_interference_partial() {
        let i = Interference::new(vec![1.0, 1.0], vec![0.0, std::f64::consts::PI / 2.0]);
        let r = i.resultant_amplitude();
        // sqrt(1^2 + 1^2) = sqrt(2) ≈ 1.414
        assert!((r - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    // Uncertainty tests

    #[test]
    fn test_uncertainty_product() {
        let u = Uncertainty::new(2.0, 3.0);
        assert!((u.product() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_uncertainty_bound() {
        let u = Uncertainty::new(1.0, 1.0);
        assert!(u.satisfies_bound(0.5));
        assert!(!u.satisfies_bound(2.0));
    }

    #[test]
    fn test_uncertainty_balance() {
        let u = Uncertainty::new(1.0, 1.0);
        assert!((u.balance() - 1.0).abs() < f64::EPSILON);
        let u2 = Uncertainty::new(1.0, 4.0);
        assert!((u2.balance() - 0.25).abs() < f64::EPSILON);
    }

    // Unitarity tests

    #[test]
    fn test_unitarity_lossless() {
        let u = Unitarity::new(0.999, 4);
        assert!(u.is_lossless(0.99));
        assert!(!u.is_lossless(0.9999));
    }

    #[test]
    fn test_unitarity_information_loss() {
        let u = Unitarity::new(0.95, 2);
        assert!((u.information_loss() - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unitarity_max_entropy() {
        let u = Unitarity::new(1.0, 8);
        assert!((u.max_entropy() - 3.0).abs() < f64::EPSILON); // log2(8) = 3
    }

    // Eigenstate tests

    #[test]
    fn test_eigenstate_fixed_point() {
        let e = Eigenstate::new(1.0, "equilibrium");
        assert!(e.is_fixed_point(0.01));
        assert!(e.is_stable());
    }

    #[test]
    fn test_eigenstate_unstable() {
        let e = Eigenstate::new(2.0, "exponential-growth");
        assert!(!e.is_fixed_point(0.01));
        assert!(!e.is_stable());
    }

    #[test]
    fn test_eigenstate_growth_rate() {
        let e = Eigenstate::new(1.05, "slow-growth");
        assert!((e.growth_rate() - 0.05).abs() < f64::EPSILON);
    }

    // Observable tests

    #[test]
    fn test_observable_range() {
        let o = Observable::new("energy", vec![1.0, 3.0, 5.0]);
        assert!((o.range() - 4.0).abs() < f64::EPSILON);
        assert_eq!(o.num_outcomes(), 3);
    }

    #[test]
    fn test_observable_mean() {
        let o = Observable::new("spin", vec![-1.0, 1.0]);
        assert!((o.mean() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_observable_empty() {
        let o = Observable::new("void", vec![]);
        assert!((o.range() - 0.0).abs() < f64::EPSILON);
        assert!((o.mean() - 0.0).abs() < f64::EPSILON);
        assert_eq!(o.num_outcomes(), 0);
        assert!((o.variance() - 0.0).abs() < f64::EPSILON);
        assert!((o.std_dev() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_observable_variance() {
        // {1, 3, 5}: mean=3, var=((1-3)^2 + (3-3)^2 + (5-3)^2)/3 = 8/3
        let o = Observable::new("energy", vec![1.0, 3.0, 5.0]);
        assert!((o.variance() - 8.0 / 3.0).abs() < 1e-10);
        assert!((o.std_dev() - (8.0_f64 / 3.0).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_observable_variance_uniform() {
        let o = Observable::new("constant", vec![5.0, 5.0, 5.0]);
        assert!((o.variance() - 0.0).abs() < f64::EPSILON);
    }

    // Hermiticity tests

    #[test]
    fn test_hermiticity_perfect() {
        let h = Hermiticity::new(true, 0.0);
        assert!(h.within_tolerance(0.001));
        assert!((h.symmetry_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hermiticity_approximate() {
        let h = Hermiticity::new(false, 0.1);
        assert!(h.within_tolerance(0.2));
        assert!(!h.within_tolerance(0.05));
    }

    #[test]
    fn test_hermiticity_symmetry_score() {
        let h = Hermiticity::new(false, 1.0);
        assert!((h.symmetry_score() - 0.5).abs() < f64::EPSILON);
    }

    // Entanglement tests

    #[test]
    fn test_entanglement_separable() {
        let e = Entanglement::new("qubit-A", "qubit-B", 0.0);
        assert!(e.is_separable(0.01));
        assert!(!e.is_maximal(0.01));
    }

    #[test]
    fn test_entanglement_maximal() {
        let e = Entanglement::new("qubit-A", "qubit-B", 1.0);
        assert!(!e.is_separable(0.01));
        assert!(e.is_maximal(0.01));
    }

    #[test]
    fn test_entanglement_entropy() {
        let e = Entanglement::new("A", "B", 0.5);
        let entropy = e.entropy();
        assert!(entropy > 0.0);
        assert!(entropy <= 1.0);
    }

    #[test]
    fn test_entanglement_entropy_extremes() {
        let sep = Entanglement::new("A", "B", 0.0);
        assert!((sep.entropy() - 0.0).abs() < f64::EPSILON);
        let max = Entanglement::new("A", "B", 1.0);
        assert!((max.entropy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_entanglement_clamps() {
        let e = Entanglement::new("A", "B", 1.5);
        assert!((e.concurrence - 1.0).abs() < f64::EPSILON);
    }

    // Decoherence tests

    #[test]
    fn test_decoherence_initial() {
        let d = Decoherence::new(10.0, "dephasing");
        assert!((d.coherence_at(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_decoherence_at_coherence_time() {
        let d = Decoherence::new(10.0, "dephasing");
        let c = d.coherence_at(10.0);
        // At t=T2: e^(-1) ≈ 0.368
        assert!((c - (-1.0_f64).exp()).abs() < 1e-10);
    }

    #[test]
    fn test_decoherence_threshold() {
        let d = Decoherence::new(10.0, "T2");
        let t = d.time_to_threshold(0.5);
        assert!((d.coherence_at(t) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_decoherence_is_decohered() {
        let d = Decoherence::new(1.0, "fast");
        assert!(!d.is_decohered(0.1, 0.5));
        assert!(d.is_decohered(10.0, 0.5));
    }

    // Qubit tests

    #[test]
    fn test_qubit_zero_state() {
        let q = Qubit::zero();
        assert!((q.prob_zero() - 1.0).abs() < f64::EPSILON);
        assert!((q.prob_one() - 0.0).abs() < f64::EPSILON);
        assert!(q.is_normalized(0.001));
    }

    #[test]
    fn test_qubit_one_state() {
        let q = Qubit::one();
        assert!((q.prob_zero() - 0.0).abs() < f64::EPSILON);
        assert!((q.prob_one() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_qubit_superposition() {
        let s = 1.0 / std::f64::consts::SQRT_2;
        let q = Qubit::new(s, s);
        assert!(q.is_normalized(0.001));
        assert!((q.prob_zero() - 0.5).abs() < 0.001);
        assert!((q.prob_one() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_qubit_total_probability() {
        let q = Qubit::new(0.6, 0.8);
        assert!((q.total_probability() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_qubit_theta() {
        let q = Qubit::zero();
        assert!((q.theta() - 0.0).abs() < 1e-10);
        let q1 = Qubit::one();
        assert!((q1.theta() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_qubit_display() {
        let q = Qubit::zero();
        let s = format!("{}", q);
        assert!(s.contains("|0>"));
        assert!(s.contains("|1>"));
    }

    // Total ordering tests (Eq, Hash, PartialOrd, Ord)

    #[test]
    fn test_amplitude_ord() {
        let a1 = Amplitude::new(1.0, 2.0);
        let a2 = Amplitude::new(1.0, 3.0);
        let a3 = Amplitude::new(2.0, 1.0);
        assert!(a1 < a2); // same magnitude, lower frequency
        assert!(a1 < a3); // lower magnitude
        assert_eq!(a1.cmp(&a1), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_amplitude_hash_consistent() {
        use std::collections::HashSet;
        let a1 = Amplitude::new(1.0, 2.0);
        let a2 = Amplitude::new(1.0, 2.0);
        let a3 = Amplitude::new(1.0, 3.0);
        let mut set = HashSet::new();
        set.insert(a1.clone());
        assert!(set.contains(&a2));
        assert!(!set.contains(&a3));
    }

    #[test]
    fn test_qubit_eq_hash() {
        use std::collections::HashSet;
        let q1 = Qubit::zero();
        let q2 = Qubit::zero();
        let q3 = Qubit::one();
        let mut set = HashSet::new();
        set.insert(q1);
        assert!(set.contains(&q2));
        assert!(!set.contains(&q3));
    }

    #[test]
    fn test_superposition_ord_vec() {
        let s1 = Superposition::new(vec![0.5, 0.5], vec!["a".into(), "b".into()]);
        let s2 = Superposition::new(vec![0.5, 0.6], vec!["a".into(), "b".into()]);
        assert!(s1 < s2);
    }

    #[test]
    fn test_measurement_ord() {
        let m1 = Measurement::new("a", 0.5);
        let m2 = Measurement::new("b", 0.5);
        assert!(m1 < m2); // "a" < "b"
    }

    #[test]
    fn test_entanglement_hash() {
        use std::collections::HashSet;
        let e1 = Entanglement::new("A", "B", 0.8);
        let e2 = Entanglement::new("A", "B", 0.8);
        let e3 = Entanglement::new("A", "B", 0.9);
        let mut set = HashSet::new();
        set.insert(e1);
        assert!(set.contains(&e2));
        assert!(!set.contains(&e3));
    }

    #[test]
    fn test_observable_ord_by_name_then_values() {
        let o1 = Observable::new("energy", vec![1.0, 2.0]);
        let o2 = Observable::new("spin", vec![1.0, 2.0]);
        assert!(o1 < o2); // "energy" < "spin"
    }

    #[test]
    fn test_hermiticity_ord() {
        let h1 = Hermiticity::new(false, 0.1);
        let h2 = Hermiticity::new(true, 0.1);
        assert!(h1 < h2); // false < true
    }

    #[test]
    fn test_decoherence_ord() {
        let d1 = Decoherence::new(5.0, "dephasing");
        let d2 = Decoherence::new(10.0, "dephasing");
        assert!(d1 < d2); // lower coherence_time
    }
}
