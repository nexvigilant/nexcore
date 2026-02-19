//! # Validated Bounds (ToV Part I §0-§7)
//!
//! Wolfram-validated computational complexity bounds for Theory of Vigilance axioms.
//! Per Safety Axiom 4, all bounds ensure d(s) > 0 for safe operations.
//!
//! ## Safety Axioms Implemented
//!
//! - **Axiom 1 (System Decomposition)**: Complex systems decompose into finite elements
//! - **Axiom 2 (Hierarchical Organization)**: Systems exhibit ~10× scale separation
//! - **Axiom 3 (Conservation Constraints)**: Behavior governed by 11 Conservation Laws
//! - **Axiom 4 (Safety Manifold)**: Safe states form manifold M with d(s) > 0
//! - **Axiom 5 (Emergence)**: Harm probability factors across hierarchy levels
//!
//! ## Conservation Laws Referenced
//!
//! - Law 1 (Mass): Data volume conservation
//! - Law 2 (Energy): Resource conservation
//! - Law 4 (Flux): Throughput continuity
//! - Law 8 (Saturation): Capacity limits
//! - Law 11 (Structure): Architectural invariants
//!
//! ## Validation Evidence (2026-01-29)
//!
//! | Axiom | Property | Wolfram Result | Reference |
//! |-------|----------|----------------|-----------|
//! | A1 | Power set |𝒫(E)| = 2^n | 2^15 = 32768 | §2.3 |
//! | A2 | Scale separation | 10^8 / 10^7 = 10 | §3.2 |
//! | A4 | Safety margin d(s) | min(0.5, -0.2, 0.8) = -0.2 | §5.2 |
//! | A5 | Emergence attenuation | 0.9^8 ≈ 0.43 | §6.2 |
//! | S=URT | Uniqueness U | -log₂(0.001) ≈ 9.97 bits | §21 |

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 1: SYSTEM DECOMPOSITION BOUNDS (§2)
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 1 complexity bounds for composition function Φ: 𝒫(E) → S
///
/// **Wolfram Validation:** 2^15 = 32768
#[derive(Debug, Clone, Copy)]
pub struct Axiom1Bounds {
    /// Number of elements in decomposition
    pub n_elements: usize,
    /// Size of power set |𝒫(E)| = 2^n
    pub power_set_size: u64,
    /// Computational complexity class
    pub complexity_class: ComplexityClass,
}

impl Axiom1Bounds {
    /// Calculate bounds for element decomposition
    #[must_use]
    pub fn calculate(n_elements: usize) -> Self {
        let power_set_size = if n_elements < 64 {
            1u64 << n_elements
        } else {
            u64::MAX
        };

        let complexity_class = match n_elements {
            0..=10 => ComplexityClass::Tractable,
            11..=20 => ComplexityClass::Feasible,
            21..=30 => ComplexityClass::Expensive,
            _ => ComplexityClass::Intractable,
        };

        Self {
            n_elements,
            power_set_size,
            complexity_class,
        }
    }

    /// Check if exhaustive enumeration is tractable
    #[must_use]
    pub fn is_tractable(&self) -> bool {
        matches!(
            self.complexity_class,
            ComplexityClass::Tractable | ComplexityClass::Feasible
        )
    }
}

/// Computational complexity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityClass {
    /// O(2^n) where n ≤ 10: ~1K operations
    Tractable,
    /// O(2^n) where n ≤ 20: ~1M operations
    Feasible,
    /// O(2^n) where n ≤ 30: ~1B operations
    Expensive,
    /// O(2^n) where n > 30: exceeds practical limits
    Intractable,
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 2: HIERARCHICAL ORGANIZATION BOUNDS (§3)
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 2 scale separation validation (~10× between levels)
///
/// **Wolfram Validation:** 10^8 / 10^7 = 10
#[derive(Debug, Clone, Copy)]
pub struct Axiom2ScaleSeparation {
    /// Lower hierarchy level
    pub level_low: u8,
    /// Upper hierarchy level
    pub level_high: u8,
    /// Expected scale ratio (should be ~10× per level)
    pub expected_ratio: f64,
    /// Actual observed ratio
    pub actual_ratio: f64,
    /// Validation passed
    pub valid: bool,
}

impl Axiom2ScaleSeparation {
    /// Validate scale separation between hierarchy levels
    #[must_use]
    pub fn validate(level_low: u8, level_high: u8, units_low: f64, units_high: f64) -> Self {
        let level_diff = f64::from(level_high - level_low);
        let expected_ratio = 10.0_f64.powf(level_diff);
        let actual_ratio = units_high / units_low;

        let valid = actual_ratio >= expected_ratio * 0.5 && actual_ratio <= expected_ratio * 2.0;

        Self {
            level_low,
            level_high,
            expected_ratio,
            actual_ratio,
            valid,
        }
    }

    /// Standard 8-level hierarchy time scales (ToV §A.2)
    pub const TIMESCALES: [(&'static str, f64); 8] = [
        ("ps-μs", 1e-12),
        ("μs-ms", 1e-6),
        ("ms-s", 1e-3),
        ("s-min", 1.0),
        ("min-hr", 60.0),
        ("hr-days", 3600.0),
        ("days-wks", 86400.0),
        ("wks-yrs", 604_800.0),
    ];
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 3: CONSERVATION CONSTRAINT BOUNDS (§4)
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 3 feasible region computation
///
/// F(u,θ) = ⋂ᵢ {s : gᵢ(s,u,θ) ≤ 0}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibleRegion {
    /// Number of active constraints
    pub n_constraints: usize,
    /// Constraint values gᵢ(s,u,θ)
    pub constraint_values: Vec<f64>,
    /// Which constraints are satisfied (gᵢ ≤ 0)
    pub satisfied: Vec<bool>,
    /// State is in feasible region (all constraints satisfied)
    pub in_feasible_region: bool,
}

impl FeasibleRegion {
    /// Evaluate feasible region membership
    #[must_use]
    pub fn evaluate(constraint_values: Vec<f64>) -> Self {
        let satisfied: Vec<bool> = constraint_values.iter().map(|&g| g <= 0.0).collect();
        let in_feasible_region = satisfied.iter().all(|&s| s);

        Self {
            n_constraints: constraint_values.len(),
            constraint_values,
            satisfied,
            in_feasible_region,
        }
    }

    /// Count violated constraints
    #[must_use]
    pub fn n_violated(&self) -> usize {
        self.satisfied.iter().filter(|&&s| !s).count()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 4: SAFETY MANIFOLD BOUNDS (§5)
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 4 safety margin d(s) with validated computation
///
/// d(s) = minᵢ{-gᵢ(s,u,θ)} for active constraints
///
/// **Wolfram Validation:** min(0.5, -0.2, 0.8) = -0.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedSafetyMargin {
    /// Raw safety margin d(s)
    pub d_s: f64,
    /// Individual constraint margins
    pub margins: Vec<f64>,
    /// Index of binding (closest) constraint
    pub binding_constraint: usize,
    /// State classification
    pub state: SafetyState,
}

/// Safety state classification based on d(s)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyState {
    /// d(s) > 0.5: Robustly inside safe manifold
    RobustlySafe,
    /// 0 < d(s) ≤ 0.5: Safe but near boundary
    MarginallySafe,
    /// d(s) = 0: On harm boundary ∂M
    OnBoundary,
    /// d(s) < 0: Outside safe manifold (harm region)
    InHarmRegion,
}

impl ValidatedSafetyMargin {
    /// Calculate validated safety margin
    #[must_use]
    pub fn calculate(constraint_values: &[f64]) -> Self {
        if constraint_values.is_empty() {
            return Self {
                d_s: f64::INFINITY,
                margins: vec![],
                binding_constraint: 0,
                state: SafetyState::RobustlySafe,
            };
        }

        let margins: Vec<f64> = constraint_values.iter().map(|&g| -g).collect();

        let (binding_constraint, d_s) = margins.iter().enumerate().fold(
            (0_usize, f64::INFINITY),
            |(min_idx, min_val), (idx, &val)| match val
                .partial_cmp(&min_val)
                .unwrap_or(Ordering::Equal)
            {
                Ordering::Less => (idx, val),
                _ => (min_idx, min_val),
            },
        );

        #[allow(clippy::float_cmp)]
        let state = if d_s > 0.5 {
            SafetyState::RobustlySafe
        } else if d_s > 0.0 {
            SafetyState::MarginallySafe
        } else if d_s == 0.0 {
            SafetyState::OnBoundary
        } else {
            SafetyState::InHarmRegion
        };

        Self {
            d_s,
            margins,
            binding_constraint,
            state,
        }
    }

    /// Check if state is safe (d(s) > 0)
    #[must_use]
    pub fn is_safe(&self) -> bool {
        self.d_s > 0.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 5: EMERGENCE BOUNDS (§6)
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 5 emergence probability with attenuation
///
/// P(H|δs₁) = ∏ᵢ Pᵢ→ᵢ₊₁ (Markov factorization)
///
/// **Wolfram Validation:**
/// - Uniform α=0.9 over 8 levels: 0.9^8 ≈ 0.43
/// - Variable α=[0.95,0.90,...,0.60]: product ≈ 0.119
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceProbability {
    /// Per-level propagation probabilities
    pub propagation_probs: Vec<f64>,
    /// Per-level attenuation factors αᵢ
    pub attenuation_factors: Vec<f64>,
    /// Final emergence probability P(H|δs₁)
    pub emergence_prob: f64,
    /// Effective attenuation through hierarchy
    pub total_attenuation: f64,
}

impl EmergenceProbability {
    /// Calculate emergence probability with attenuation
    #[must_use]
    pub fn calculate(propagation_probs: Vec<f64>, attenuation_factors: Vec<f64>) -> Self {
        let total_attenuation: f64 = attenuation_factors.iter().product();
        let base_emergence: f64 = propagation_probs.iter().product();
        let emergence_prob = base_emergence * total_attenuation;

        Self {
            propagation_probs,
            attenuation_factors,
            emergence_prob,
            total_attenuation,
        }
    }

    /// Calculate with uniform attenuation factor
    #[must_use]
    pub fn with_uniform_attenuation(propagation_probs: Vec<f64>, alpha: f64) -> Self {
        let n_levels = propagation_probs.len();
        let attenuation_factors = vec![alpha; n_levels];
        Self::calculate(propagation_probs, attenuation_factors)
    }

    /// Standard 8-level attenuation (ToV §6.3 example)
    pub const STANDARD_ATTENUATION: [f64; 8] = [0.95, 0.90, 0.85, 0.80, 0.75, 0.70, 0.65, 0.60];
}

// ═══════════════════════════════════════════════════════════════════════════
// SIGNAL DETECTION BOUNDS (§19-§21)
// ═══════════════════════════════════════════════════════════════════════════

/// Signal uniqueness U = -log₂ P(C|H₀) with validated computation
///
/// **Wolfram Validation:**
/// - P = 0.001: U ≈ 9.97 bits
/// - P = 0.0001: U ≈ 13.29 bits
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignalUniqueness {
    /// Null hypothesis probability P(C|H₀)
    pub null_probability: f64,
    /// Uniqueness measure U (bits)
    pub u_bits: f64,
    /// Signal strength classification
    pub strength: SignalStrength,
}

/// Signal strength classification based on U
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalStrength {
    /// U < 3 bits: Weak signal
    Weak,
    /// 3 ≤ U < 8 bits: Moderate signal
    Moderate,
    /// 8 ≤ U < 15 bits: Strong signal
    Strong,
    /// U ≥ 15 bits: Very strong signal
    VeryStrong,
}

impl SignalUniqueness {
    /// Calculate signal uniqueness
    #[must_use]
    pub fn calculate(null_probability: f64) -> Self {
        let u_bits = if null_probability > 0.0 {
            -null_probability.log2()
        } else {
            f64::INFINITY
        };

        let strength = if u_bits < 3.0 {
            SignalStrength::Weak
        } else if u_bits < 8.0 {
            SignalStrength::Moderate
        } else if u_bits < 15.0 {
            SignalStrength::Strong
        } else {
            SignalStrength::VeryStrong
        };

        Self {
            null_probability,
            u_bits,
            strength,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// VALIDATION SUMMARY
// ═══════════════════════════════════════════════════════════════════════════

/// Summary of all validated bounds for ToV Part I
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Axiom 1: Element decomposition tractability
    pub axiom1_tractable: bool,
    /// Axiom 2: Scale separation valid
    pub axiom2_valid: bool,
    /// Axiom 3: In feasible region
    pub axiom3_feasible: bool,
    /// Axiom 4: Safety margin positive
    pub axiom4_safe: bool,
    /// Axiom 5: Emergence probability computed
    pub axiom5_computed: bool,
    /// Overall validation passed
    pub all_valid: bool,
}

impl ValidationSummary {
    /// Create validation summary from individual results
    #[must_use]
    pub fn from_results(
        a1: &Axiom1Bounds,
        a2: &Axiom2ScaleSeparation,
        a3: &FeasibleRegion,
        a4: &ValidatedSafetyMargin,
        _a5: &EmergenceProbability,
    ) -> Self {
        let axiom1_tractable = a1.is_tractable();
        let axiom2_valid = a2.valid;
        let axiom3_feasible = a3.in_feasible_region;
        let axiom4_safe = a4.is_safe();
        let axiom5_computed = true;

        let all_valid =
            axiom1_tractable && axiom2_valid && axiom3_feasible && axiom4_safe && axiom5_computed;

        Self {
            axiom1_tractable,
            axiom2_valid,
            axiom3_feasible,
            axiom4_safe,
            axiom5_computed,
            all_valid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axiom1_power_set_wolfram_validated() {
        let bounds = Axiom1Bounds::calculate(15);
        assert_eq!(bounds.power_set_size, 32768);
        assert_eq!(bounds.complexity_class, ComplexityClass::Feasible);
    }

    #[test]
    fn test_axiom2_scale_separation_wolfram_validated() {
        let sep = Axiom2ScaleSeparation::validate(7, 8, 1e7, 1e8);
        assert!((sep.expected_ratio - 10.0).abs() < 0.001);
        assert!((sep.actual_ratio - 10.0).abs() < 0.001);
        assert!(sep.valid);
    }

    #[test]
    fn test_axiom4_safety_margin_wolfram_validated() {
        let margin = ValidatedSafetyMargin::calculate(&[-0.5, 0.2, -0.8]);
        assert!((margin.d_s - (-0.2)).abs() < 0.001);
        assert_eq!(margin.binding_constraint, 1);
        assert_eq!(margin.state, SafetyState::InHarmRegion);
    }

    #[test]
    fn test_axiom5_emergence_wolfram_validated() {
        let probs = vec![1.0; 8];
        let emergence = EmergenceProbability::with_uniform_attenuation(probs, 0.9);
        assert!((emergence.total_attenuation - 0.430_467_21).abs() < 0.0001);
    }

    #[test]
    fn test_axiom5_variable_attenuation_wolfram_validated() {
        let probs = vec![1.0; 8];
        let alphas = vec![0.95, 0.90, 0.85, 0.80, 0.75, 0.70, 0.65, 0.60];
        let emergence = EmergenceProbability::calculate(probs, alphas);
        assert!((emergence.total_attenuation - 0.119_041_65).abs() < 0.0001);
    }

    #[test]
    fn test_signal_uniqueness_wolfram_validated() {
        let u1 = SignalUniqueness::calculate(0.001);
        assert!((u1.u_bits - 9.965_784).abs() < 0.001);
        assert_eq!(u1.strength, SignalStrength::Strong);

        let u2 = SignalUniqueness::calculate(0.0001);
        assert!((u2.u_bits - 13.287_71).abs() < 0.001);
        assert_eq!(u2.strength, SignalStrength::Strong);
    }

    #[test]
    fn test_feasible_region() {
        let fr = FeasibleRegion::evaluate(vec![-0.5, -0.3, -0.8]);
        assert!(fr.in_feasible_region);
        assert_eq!(fr.n_violated(), 0);

        let fr2 = FeasibleRegion::evaluate(vec![-0.5, 0.2, -0.8]);
        assert!(!fr2.in_feasible_region);
        assert_eq!(fr2.n_violated(), 1);
    }

    #[test]
    fn test_validation_summary() {
        let a1 = Axiom1Bounds::calculate(15);
        let a2 = Axiom2ScaleSeparation::validate(7, 8, 1e7, 1e8);
        let a3 = FeasibleRegion::evaluate(vec![-0.5, -0.3, -0.8]);
        let a4 = ValidatedSafetyMargin::calculate(&[-0.5, -0.3, -0.8]);
        let a5 = EmergenceProbability::with_uniform_attenuation(vec![1.0; 8], 0.9);

        let summary = ValidationSummary::from_results(&a1, &a2, &a3, &a4, &a5);
        assert!(summary.all_valid);
    }
}
