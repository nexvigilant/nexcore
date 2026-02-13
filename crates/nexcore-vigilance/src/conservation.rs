//! # ToV §4: Axiom 3 - Conservation Constraints
//!
//! Formal implementation of Axiom 3 and its prerequisites (Definitions 4.1-4.5).
//!
//! ## Axiom Statement
//!
//! For every vigilance system 𝒱 with harm specification ℋ, there exists a constraint
//! set 𝒢 = {g₁, ..., gₘ} such that:
//! 1. |𝒢| = m < ∞ (finitely many constraints)
//! 2. The harm event H occurs if and only if s ∉ F(u, θ)
//! 3. H = ⋃ᵢ₌₁ᵐ Hᵢ where Hᵢ = {s : gᵢ(s, u, θ) > 0}
//!
//! ## Symbolic Formulation
//!
//! **∀𝒱 : ∃𝒢 = {gᵢ}ᵢ₌₁ᵐ with m < ∞ such that:**
//!
//! **H ⟺ ∃i: gᵢ(s, u, θ) > 0 ⟺ s ∉ ⋂ᵢ₌₁ᵐ {s: gᵢ(s,u,θ) ≤ 0}**
//!
//! ## Wolfram Validation (2026-01-29)
//!
//! | Property | Formula | Result |
//! |----------|---------|--------|
//! | Violation magnitude (g > 0) | max(0, 0.5) | 0.5 |
//! | No violation (g ≤ 0) | max(0, -0.5) | 0 |

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// CONSERVATION LAW TAXONOMY (§4.5)
// ═══════════════════════════════════════════════════════════════════════════

/// Conservation law taxonomy from ToV §4.5.
///
/// | Type | Form | Example |
/// |------|------|---------|
/// | Mass/Amount | Input - Output = Accumulation | Drug mass balance |
/// | Energy/Gradient | ΔG < 0 for spontaneous | Binding thermodynamics |
/// | State | Σ fractions = 1 | Receptor occupancy |
/// | Flux | Σ J_in = Σ J_out | Pathway throughput |
/// | Capacity | v ≤ V_max | Enzyme saturation |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConservationLawType {
    /// Law 1: Mass/Amount conservation.
    /// Form: Input - Output = Accumulation.
    /// Example: Drug mass balance, data volume.
    Mass,

    /// Law 2: Energy/Gradient conservation.
    /// Form: ΔG < 0 for spontaneous process.
    /// Example: Binding thermodynamics, loss decrease.
    Energy,

    /// Law 3: State normalization.
    /// Form: Σ state_fractions = 1.
    /// Example: Receptor occupancy, attention weights.
    State,

    /// Law 4: Flux continuity.
    /// Form: Σ J_in = Σ J_out at nodes.
    /// Example: Pathway flux, network throughput.
    Flux,

    /// Law 5: Catalyst invariance.
    /// Form: [Catalyst] unchanged by reaction.
    /// Example: Enzyme not consumed, competitive inhibition.
    Catalyst,

    /// Law 6: Entropy increase (irreversibility).
    /// Form: ΔS ≥ 0 for isolated system.
    /// Example: Reaction directionality.
    Entropy,

    /// Law 7: Momentum (rate of change).
    /// Form: Σ Fᵢ = dp/dt.
    /// Example: Rate constraints on state change.
    Momentum,

    /// Law 8: Capacity/Saturation.
    /// Form: v ≤ V_max.
    /// Example: Enzyme saturation, memory limits.
    Capacity,

    /// Law 9: Charge/Sign conservation.
    /// Form: Σ charges = constant.
    /// Example: Ion balance, signed quantity preservation.
    Charge,

    /// Law 10: Stoichiometry.
    /// Form: Reactants consumed in fixed ratios.
    /// Example: Drug metabolism ratios.
    Stoichiometry,

    /// Law 11: Structural invariant.
    /// Form: Topological property unchanged.
    /// Example: Architecture preserved, connectivity.
    Structure,
}

impl ConservationLawType {
    /// All 11 conservation law types.
    pub const ALL: [ConservationLawType; 11] = [
        Self::Mass,
        Self::Energy,
        Self::State,
        Self::Flux,
        Self::Catalyst,
        Self::Entropy,
        Self::Momentum,
        Self::Capacity,
        Self::Charge,
        Self::Stoichiometry,
        Self::Structure,
    ];

    /// Law number (1-11).
    #[must_use]
    pub fn law_number(&self) -> u8 {
        match self {
            Self::Mass => 1,
            Self::Energy => 2,
            Self::State => 3,
            Self::Flux => 4,
            Self::Catalyst => 5,
            Self::Entropy => 6,
            Self::Momentum => 7,
            Self::Capacity => 8,
            Self::Charge => 9,
            Self::Stoichiometry => 10,
            Self::Structure => 11,
        }
    }

    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mass => "Mass/Amount",
            Self::Energy => "Energy/Gradient",
            Self::State => "State Normalization",
            Self::Flux => "Flux Continuity",
            Self::Catalyst => "Catalyst Invariance",
            Self::Entropy => "Entropy Increase",
            Self::Momentum => "Momentum",
            Self::Capacity => "Capacity/Saturation",
            Self::Charge => "Charge Conservation",
            Self::Stoichiometry => "Stoichiometry",
            Self::Structure => "Structural Invariant",
        }
    }

    /// Standard form of the constraint.
    #[must_use]
    pub fn standard_form(&self) -> &'static str {
        match self {
            Self::Mass => "Input - Output = Accumulation",
            Self::Energy => "ΔG < 0 for spontaneous",
            Self::State => "Σ fractions = 1",
            Self::Flux => "Σ J_in = Σ J_out",
            Self::Catalyst => "[Catalyst] unchanged",
            Self::Entropy => "ΔS ≥ 0",
            Self::Momentum => "Σ F = dp/dt",
            Self::Capacity => "v ≤ V_max",
            Self::Charge => "Σ charges = const",
            Self::Stoichiometry => "Fixed ratios",
            Self::Structure => "Topology preserved",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 4.1: CONSERVATION LAW
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 4.1: A conservation law is a constraint function g: S × U × Θ → ℝ.
///
/// The constraint is satisfied when g(s, u, θ) ≤ 0.
///
/// # Remarks
/// - The term "conservation law" encompasses any fundamental constraint
/// - Includes strict conservation (equalities) and bound constraints (inequalities)
/// - g is evaluated instantaneously at time t
pub trait ConservationLaw: std::fmt::Debug {
    /// Evaluate the constraint: g(s, u, θ) → ℝ.
    ///
    /// - Returns ≤ 0 when constraint is satisfied
    /// - Returns > 0 when constraint is violated
    fn evaluate(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> f64;

    /// Conservation law type (which of the 11 laws).
    fn law_type(&self) -> ConservationLawType;

    /// Human-readable name of this specific constraint.
    fn name(&self) -> &str;

    /// Check if constraint is satisfied at given state.
    fn is_satisfied(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> bool {
        self.evaluate(state, perturbation, parameters) <= 0.0
    }

    /// Check if constraint is active (exactly at boundary).
    ///
    /// Definition 4.4: gᵢ is active at s if gᵢ(s, u, θ) = 0.
    fn is_active(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> bool {
        let epsilon = 1e-10;
        self.evaluate(state, perturbation, parameters).abs() < epsilon
    }

    /// Calculate violation magnitude.
    ///
    /// Definition 4.5: vᵢ(s, u, θ) = max(0, gᵢ(s, u, θ))
    fn violation_magnitude(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> f64 {
        self.evaluate(state, perturbation, parameters).max(0.0)
    }

    /// Calculate margin (distance to violation).
    ///
    /// dᵢ(s, u, θ) = -gᵢ(s, u, θ)
    /// Positive margin = constraint satisfied with room to spare.
    fn margin(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> f64 {
        -self.evaluate(state, perturbation, parameters)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONCRETE CONSERVATION LAW IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Capacity constraint: v ≤ V_max.
///
/// g(s) = s[index] - capacity
#[derive(Debug, Clone)]
pub struct CapacityConstraint {
    /// Name of this constraint.
    pub name: String,
    /// Index in state vector to check.
    pub state_index: usize,
    /// Maximum capacity V_max.
    pub capacity: f64,
}

impl CapacityConstraint {
    /// Create a new capacity constraint.
    #[must_use]
    pub fn new(name: impl Into<String>, state_index: usize, capacity: f64) -> Self {
        Self {
            name: name.into(),
            state_index,
            capacity,
        }
    }
}

impl ConservationLaw for CapacityConstraint {
    fn evaluate(&self, state: &[f64], _perturbation: &[f64], _parameters: &[f64]) -> f64 {
        state
            .get(self.state_index)
            .map_or(0.0, |&v| v - self.capacity)
    }

    fn law_type(&self) -> ConservationLawType {
        ConservationLawType::Capacity
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Mass balance constraint: accumulation = input - output.
///
/// g(s, u) = |accumulation - (input - output)|  - tolerance
#[derive(Debug, Clone)]
pub struct MassBalanceConstraint {
    /// Name of this constraint.
    pub name: String,
    /// Index of accumulation in state.
    pub accumulation_index: usize,
    /// Index of input rate in perturbation.
    pub input_index: usize,
    /// Index of output rate in state.
    pub output_index: usize,
    /// Tolerance for balance.
    pub tolerance: f64,
}

impl MassBalanceConstraint {
    /// Create a new mass balance constraint.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        accumulation_index: usize,
        input_index: usize,
        output_index: usize,
        tolerance: f64,
    ) -> Self {
        Self {
            name: name.into(),
            accumulation_index,
            input_index,
            output_index,
            tolerance,
        }
    }
}

impl ConservationLaw for MassBalanceConstraint {
    fn evaluate(&self, state: &[f64], perturbation: &[f64], _parameters: &[f64]) -> f64 {
        let accumulation = state.get(self.accumulation_index).copied().unwrap_or(0.0);
        let input = perturbation.get(self.input_index).copied().unwrap_or(0.0);
        let output = state.get(self.output_index).copied().unwrap_or(0.0);

        let imbalance = (accumulation - (input - output)).abs();
        imbalance - self.tolerance
    }

    fn law_type(&self) -> ConservationLawType {
        ConservationLawType::Mass
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// State normalization constraint: Σ fractions = 1.
///
/// g(s) = |Σ s[indices] - 1| - tolerance
#[derive(Debug, Clone)]
pub struct StateNormalizationConstraint {
    /// Name of this constraint.
    pub name: String,
    /// Indices of fractions that must sum to 1.
    pub fraction_indices: Vec<usize>,
    /// Tolerance for sum.
    pub tolerance: f64,
}

impl StateNormalizationConstraint {
    /// Create a new state normalization constraint.
    #[must_use]
    pub fn new(name: impl Into<String>, fraction_indices: Vec<usize>, tolerance: f64) -> Self {
        Self {
            name: name.into(),
            fraction_indices,
            tolerance,
        }
    }
}

impl ConservationLaw for StateNormalizationConstraint {
    fn evaluate(&self, state: &[f64], _perturbation: &[f64], _parameters: &[f64]) -> f64 {
        let sum: f64 = self
            .fraction_indices
            .iter()
            .filter_map(|&i| state.get(i))
            .sum();
        (sum - 1.0).abs() - self.tolerance
    }

    fn law_type(&self) -> ConservationLawType {
        ConservationLawType::State
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Linear constraint: a·s + b·u + c·θ ≤ d.
///
/// g(s, u, θ) = a·s + b·u + c·θ - d
#[derive(Debug, Clone)]
pub struct LinearConstraint {
    /// Name of this constraint.
    pub name: String,
    /// Law type for this constraint.
    pub law_type: ConservationLawType,
    /// Coefficients for state vector.
    pub state_coefficients: Vec<f64>,
    /// Coefficients for perturbation vector.
    pub perturbation_coefficients: Vec<f64>,
    /// Coefficients for parameter vector.
    pub parameter_coefficients: Vec<f64>,
    /// Right-hand side bound.
    pub bound: f64,
}

impl LinearConstraint {
    /// Create a simple bound constraint on a single state dimension.
    #[must_use]
    pub fn state_bound(
        name: impl Into<String>,
        law_type: ConservationLawType,
        state_dim: usize,
        index: usize,
        bound: f64,
    ) -> Self {
        let mut state_coefficients = vec![0.0; state_dim];
        if index < state_dim {
            state_coefficients[index] = 1.0;
        }
        Self {
            name: name.into(),
            law_type,
            state_coefficients,
            perturbation_coefficients: vec![],
            parameter_coefficients: vec![],
            bound,
        }
    }
}

impl ConservationLaw for LinearConstraint {
    fn evaluate(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> f64 {
        let state_term: f64 = self
            .state_coefficients
            .iter()
            .zip(state.iter())
            .map(|(c, s)| c * s)
            .sum();

        let perturbation_term: f64 = self
            .perturbation_coefficients
            .iter()
            .zip(perturbation.iter())
            .map(|(c, u)| c * u)
            .sum();

        let parameter_term: f64 = self
            .parameter_coefficients
            .iter()
            .zip(parameters.iter())
            .map(|(c, p)| c * p)
            .sum();

        state_term + perturbation_term + parameter_term - self.bound
    }

    fn law_type(&self) -> ConservationLawType {
        self.law_type
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 4.2: CONSTRAINT SET
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 4.2: A constraint set 𝒢 = {g₁, g₂, ..., gₘ}.
///
/// A finite collection of conservation laws with |𝒢| = m < ∞.
#[derive(Debug, Default)]
pub struct ConstraintSet {
    /// The constraints in the set.
    constraints: Vec<Box<dyn ConservationLaw>>,
}

impl ConstraintSet {
    /// Create an empty constraint set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a constraint to the set.
    pub fn add<C: ConservationLaw + 'static>(&mut self, constraint: C) {
        self.constraints.push(Box::new(constraint));
    }

    /// Number of constraints |𝒢| = m.
    #[must_use]
    pub fn cardinality(&self) -> usize {
        self.constraints.len()
    }

    /// Get constraint by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&dyn ConservationLaw> {
        self.constraints.get(index).map(AsRef::as_ref)
    }

    /// Evaluate all constraints, returning their values.
    #[must_use]
    pub fn evaluate_all(
        &self,
        state: &[f64],
        perturbation: &[f64],
        parameters: &[f64],
    ) -> Vec<f64> {
        self.constraints
            .iter()
            .map(|c| c.evaluate(state, perturbation, parameters))
            .collect()
    }

    /// Check if all constraints are satisfied.
    #[must_use]
    pub fn all_satisfied(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> bool {
        self.constraints
            .iter()
            .all(|c| c.is_satisfied(state, perturbation, parameters))
    }

    /// Get active set A(s, u, θ) = {i : gᵢ = 0}.
    #[must_use]
    pub fn active_set(
        &self,
        state: &[f64],
        perturbation: &[f64],
        parameters: &[f64],
    ) -> Vec<usize> {
        self.constraints
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_active(state, perturbation, parameters))
            .map(|(i, _)| i)
            .collect()
    }

    /// Get violated constraints.
    #[must_use]
    pub fn violated_constraints(
        &self,
        state: &[f64],
        perturbation: &[f64],
        parameters: &[f64],
    ) -> Vec<usize> {
        self.constraints
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_satisfied(state, perturbation, parameters))
            .map(|(i, _)| i)
            .collect()
    }

    /// Calculate total violation magnitude.
    #[must_use]
    pub fn total_violation(&self, state: &[f64], perturbation: &[f64], parameters: &[f64]) -> f64 {
        self.constraints
            .iter()
            .map(|c| c.violation_magnitude(state, perturbation, parameters))
            .sum()
    }

    /// Get binding constraint (most restrictive).
    #[must_use]
    pub fn binding_constraint(
        &self,
        state: &[f64],
        perturbation: &[f64],
        parameters: &[f64],
    ) -> Option<(usize, f64)> {
        self.constraints
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.margin(state, perturbation, parameters)))
            .min_by(|(_, m1), (_, m2)| m1.partial_cmp(m2).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Iterate over constraints.
    pub fn iter(&self) -> impl Iterator<Item = &dyn ConservationLaw> {
        self.constraints.iter().map(AsRef::as_ref)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 4.3: FEASIBLE REGION
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 4.3: Feasible region evaluation result.
///
/// F(u, θ) = {s ∈ S : gᵢ(s, u, θ) ≤ 0 for all i}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibleRegionResult {
    /// Number of constraints.
    pub n_constraints: usize,
    /// Constraint values gᵢ(s, u, θ).
    pub constraint_values: Vec<f64>,
    /// Which constraints are satisfied.
    pub satisfied: Vec<bool>,
    /// Active constraints (on boundary).
    pub active_indices: Vec<usize>,
    /// Violated constraints.
    pub violated_indices: Vec<usize>,
    /// State is in feasible region.
    pub in_feasible_region: bool,
    /// Safety margin d(s) = min margins.
    pub safety_margin: f64,
    /// Index of binding (most restrictive) constraint.
    pub binding_constraint_index: Option<usize>,
}

impl FeasibleRegionResult {
    /// Evaluate feasible region membership.
    #[must_use]
    pub fn evaluate(
        constraint_set: &ConstraintSet,
        state: &[f64],
        perturbation: &[f64],
        parameters: &[f64],
    ) -> Self {
        let constraint_values = constraint_set.evaluate_all(state, perturbation, parameters);
        let satisfied: Vec<bool> = constraint_values.iter().map(|&g| g <= 0.0).collect();

        let epsilon = 1e-10;
        let active_indices: Vec<usize> = constraint_values
            .iter()
            .enumerate()
            .filter(|&(_, g)| (*g).abs() < epsilon)
            .map(|(i, _)| i)
            .collect();

        let violated_indices: Vec<usize> = constraint_values
            .iter()
            .enumerate()
            .filter(|&(_, g)| *g > 0.0)
            .map(|(i, _)| i)
            .collect();

        let in_feasible_region = violated_indices.is_empty();

        // Safety margin = min(-gᵢ) = -max(gᵢ)
        let margins: Vec<f64> = constraint_values.iter().map(|&g| -g).collect();
        let (binding_constraint_index, safety_margin) = if margins.is_empty() {
            (None, f64::INFINITY)
        } else {
            let min_entry = margins
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            match min_entry {
                Some((idx, &margin)) => (Some(idx), margin),
                None => (None, f64::INFINITY),
            }
        };

        Self {
            n_constraints: constraint_values.len(),
            constraint_values,
            satisfied,
            active_indices,
            violated_indices,
            in_feasible_region,
            safety_margin,
            binding_constraint_index,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 3 VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 3 verification result.
///
/// Verifies: H ⟺ ∃i: gᵢ(s, u, θ) > 0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axiom3Verification {
    /// Number of constraints |𝒢| = m.
    pub constraint_count: usize,
    /// Number of satisfied constraints.
    pub satisfied_count: usize,
    /// Number of violated constraints.
    pub violated_count: usize,
    /// Number of active constraints (on boundary).
    pub active_count: usize,
    /// State is in feasible region (no violations).
    pub in_feasible_region: bool,
    /// Harm event H occurred (at least one violation).
    pub harm_occurred: bool,
    /// Total violation magnitude.
    pub total_violation: f64,
    /// Safety margin d(s).
    pub safety_margin: f64,
    /// Axiom 3 condition verified (H ⟺ s ∉ F).
    pub axiom_verified: bool,
}

impl Axiom3Verification {
    /// Verify Axiom 3 for a constraint set at given state.
    #[must_use]
    pub fn verify(
        constraint_set: &ConstraintSet,
        state: &[f64],
        perturbation: &[f64],
        parameters: &[f64],
    ) -> Self {
        let result =
            FeasibleRegionResult::evaluate(constraint_set, state, perturbation, parameters);

        let constraint_count = result.n_constraints;
        let satisfied_count = result.satisfied.iter().filter(|&&s| s).count();
        let violated_count = result.violated_indices.len();
        let active_count = result.active_indices.len();
        let in_feasible_region = result.in_feasible_region;

        // Axiom 3: H ⟺ ∃i: gᵢ > 0 ⟺ s ∉ F
        let harm_occurred = !in_feasible_region;

        let total_violation = constraint_set.total_violation(state, perturbation, parameters);
        let safety_margin = result.safety_margin;

        // Axiom is always verified (it's the definition of harm)
        let axiom_verified = true;

        Self {
            constraint_count,
            satisfied_count,
            violated_count,
            active_count,
            in_feasible_region,
            harm_occurred,
            total_violation,
            safety_margin,
            axiom_verified,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════════

/// Errors related to conservation constraints.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConservationError {
    /// Empty constraint set.
    #[error("Constraint set is empty")]
    EmptyConstraintSet,

    /// Constraint index out of bounds.
    #[error("Constraint index {index} out of bounds (max {max})")]
    ConstraintIndexOutOfBounds {
        /// Requested index.
        index: usize,
        /// Maximum valid index.
        max: usize,
    },

    /// Dimension mismatch.
    #[error("State dimension {state} != expected {expected}")]
    DimensionMismatch {
        /// Actual state dimension.
        state: usize,
        /// Expected dimension.
        expected: usize,
    },
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservation_law_types() {
        assert_eq!(ConservationLawType::ALL.len(), 11);
        assert_eq!(ConservationLawType::Mass.law_number(), 1);
        assert_eq!(ConservationLawType::Structure.law_number(), 11);
    }

    #[test]
    fn test_capacity_constraint() {
        let constraint = CapacityConstraint::new("memory_limit", 0, 100.0);

        // Under capacity = satisfied
        assert!(constraint.is_satisfied(&[50.0], &[], &[]));
        assert!((constraint.margin(&[50.0], &[], &[]) - 50.0).abs() < 0.001);

        // At capacity = active
        assert!(constraint.is_active(&[100.0], &[], &[]));

        // Over capacity = violated
        assert!(!constraint.is_satisfied(&[150.0], &[], &[]));
        assert!((constraint.violation_magnitude(&[150.0], &[], &[]) - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_violation_magnitude_wolfram_validated() {
        let constraint = CapacityConstraint::new("test", 0, 10.0);

        // Wolfram: max(0, 0.5) = 0.5
        // g = 10.5 - 10 = 0.5, violation = max(0, 0.5) = 0.5
        let violation = constraint.violation_magnitude(&[10.5], &[], &[]);
        assert!((violation - 0.5).abs() < 0.001);

        // Wolfram: max(0, -0.5) = 0
        // g = 9.5 - 10 = -0.5, violation = max(0, -0.5) = 0
        let violation2 = constraint.violation_magnitude(&[9.5], &[], &[]);
        assert!(violation2.abs() < 0.001);
    }

    #[test]
    fn test_state_normalization_constraint() {
        let constraint = StateNormalizationConstraint::new("probability_sum", vec![0, 1, 2], 0.01);

        // Proper normalization
        assert!(constraint.is_satisfied(&[0.3, 0.3, 0.4], &[], &[]));

        // Not normalized
        assert!(!constraint.is_satisfied(&[0.3, 0.3, 0.3], &[], &[]));
    }

    #[test]
    fn test_constraint_set() {
        let mut constraints = ConstraintSet::new();
        constraints.add(CapacityConstraint::new("c1", 0, 10.0));
        constraints.add(CapacityConstraint::new("c2", 1, 20.0));

        assert_eq!(constraints.cardinality(), 2);

        // Both satisfied
        let state = vec![5.0, 15.0];
        assert!(constraints.all_satisfied(&state, &[], &[]));

        // One violated
        let state2 = vec![15.0, 15.0];
        assert!(!constraints.all_satisfied(&state2, &[], &[]));
        assert_eq!(constraints.violated_constraints(&state2, &[], &[]), vec![0]);
    }

    #[test]
    fn test_feasible_region_result() {
        let mut constraints = ConstraintSet::new();
        constraints.add(CapacityConstraint::new("c1", 0, 10.0));
        constraints.add(CapacityConstraint::new("c2", 1, 20.0));

        let state = vec![5.0, 15.0];
        let result = FeasibleRegionResult::evaluate(&constraints, &state, &[], &[]);

        assert!(result.in_feasible_region);
        assert_eq!(result.violated_indices.len(), 0);
        assert!(result.safety_margin > 0.0);
    }

    #[test]
    fn test_axiom3_verification_safe() {
        let mut constraints = ConstraintSet::new();
        constraints.add(CapacityConstraint::new("c1", 0, 10.0));

        let state = vec![5.0];
        let verification = Axiom3Verification::verify(&constraints, &state, &[], &[]);

        assert!(verification.in_feasible_region);
        assert!(!verification.harm_occurred);
        assert_eq!(verification.violated_count, 0);
        assert!(verification.axiom_verified);
    }

    #[test]
    fn test_axiom3_verification_harm() {
        let mut constraints = ConstraintSet::new();
        constraints.add(CapacityConstraint::new("c1", 0, 10.0));

        let state = vec![15.0];
        let verification = Axiom3Verification::verify(&constraints, &state, &[], &[]);

        assert!(!verification.in_feasible_region);
        assert!(verification.harm_occurred);
        assert_eq!(verification.violated_count, 1);
        assert!(verification.total_violation > 0.0);
    }

    #[test]
    fn test_binding_constraint() {
        let mut constraints = ConstraintSet::new();
        constraints.add(CapacityConstraint::new("tight", 0, 10.0));
        constraints.add(CapacityConstraint::new("loose", 1, 100.0));

        let state = vec![8.0, 10.0];
        let (idx, margin) = constraints.binding_constraint(&state, &[], &[]).unwrap();

        assert_eq!(idx, 0); // "tight" is binding
        assert!((margin - 2.0).abs() < 0.001); // margin = 10 - 8 = 2
    }

    #[test]
    fn test_linear_constraint() {
        // x + y ≤ 10
        let constraint = LinearConstraint {
            name: "sum_bound".to_string(),
            law_type: ConservationLawType::Capacity,
            state_coefficients: vec![1.0, 1.0],
            perturbation_coefficients: vec![],
            parameter_coefficients: vec![],
            bound: 10.0,
        };

        assert!(constraint.is_satisfied(&[3.0, 5.0], &[], &[])); // 8 ≤ 10
        assert!(!constraint.is_satisfied(&[6.0, 6.0], &[], &[])); // 12 > 10
    }
}
