//! # Cost Models (T1 Layer)
//!
//! Defines how much each edit operation costs. The cost model is the second
//! parameter in `edit_distance(ops, costs, solver)`.
//!
//! Built-in models:
//! - `UniformCost` — all operations cost 1.0 (Levenshtein)
//! - `WeightedCost` — per-operation-type weights
//! - `MatrixCost` — element-pair substitution matrix (bioinformatics)

use std::fmt;

use serde::{Deserialize, Serialize};

/// Computes the cost of an edit operation between two elements.
///
/// Cost models must be `Send + Sync + Clone` for parallel usage.
pub trait CostModel<E: Eq>: Clone + Send + Sync + fmt::Debug {
    /// Cost of inserting `elem` into the target
    fn insert_cost(&self, elem: &E) -> f64;

    /// Cost of deleting `elem` from the source
    fn delete_cost(&self, elem: &E) -> f64;

    /// Cost of substituting `from` with `to`
    ///
    /// Returns 0.0 if `from == to` (match, not a substitution).
    fn substitute_cost(&self, from: &E, to: &E) -> f64;

    /// Cost of transposing two adjacent elements
    fn transpose_cost(&self, first: &E, second: &E) -> f64;

    /// Human-readable name of this cost model
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// UniformCost: all ops cost 1.0 (Levenshtein)
// ---------------------------------------------------------------------------

/// All edit operations cost 1.0. This produces classic Levenshtein distance.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UniformCost;

impl<E: Eq> CostModel<E> for UniformCost {
    fn insert_cost(&self, _elem: &E) -> f64 {
        1.0
    }

    fn delete_cost(&self, _elem: &E) -> f64 {
        1.0
    }

    fn substitute_cost(&self, from: &E, to: &E) -> f64 {
        if from == to { 0.0 } else { 1.0 }
    }

    fn transpose_cost(&self, _first: &E, _second: &E) -> f64 {
        1.0
    }

    fn name(&self) -> &str {
        "uniform (all ops = 1.0)"
    }
}

// ---------------------------------------------------------------------------
// WeightedCost: per-operation-type weights
// ---------------------------------------------------------------------------

/// Per-operation-type weights. Useful for spell-checking where substitution
/// of nearby keys should cost less than distant keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedCost {
    /// Cost of insertion
    pub insert: f64,
    /// Cost of deletion
    pub delete: f64,
    /// Cost of substitution (applied when from != to)
    pub substitute: f64,
    /// Cost of transposition
    pub transpose: f64,
}

impl WeightedCost {
    /// Create a new weighted cost model.
    #[must_use]
    pub fn new(insert: f64, delete: f64, substitute: f64, transpose: f64) -> Self {
        Self {
            insert,
            delete,
            substitute,
            transpose,
        }
    }
}

impl Default for WeightedCost {
    fn default() -> Self {
        Self {
            insert: 1.0,
            delete: 1.0,
            substitute: 1.0,
            transpose: 1.0,
        }
    }
}

impl<E: Eq> CostModel<E> for WeightedCost {
    fn insert_cost(&self, _elem: &E) -> f64 {
        self.insert
    }

    fn delete_cost(&self, _elem: &E) -> f64 {
        self.delete
    }

    fn substitute_cost(&self, from: &E, to: &E) -> f64 {
        if from == to { 0.0 } else { self.substitute }
    }

    fn transpose_cost(&self, _first: &E, _second: &E) -> f64 {
        self.transpose
    }

    fn name(&self) -> &str {
        "weighted"
    }
}

// ---------------------------------------------------------------------------
// MatrixCost: element-pair substitution matrix (bioinformatics, NLP)
// ---------------------------------------------------------------------------

/// Substitution cost from a lookup function.
///
/// The lookup returns the cost of substituting one element for another.
/// Used for BLOSUM/PAM matrices (bioinformatics) or keyboard-distance
/// matrices (spell-checking).
///
/// # Type Parameter
///
/// - `F`: A closure `Fn(&E, &E) -> f64` providing pairwise substitution cost
#[derive(Clone)]
pub struct MatrixCost<F> {
    /// Substitution cost lookup
    lookup: F,
    /// Cost of insertion/deletion (gap penalty)
    gap_cost: f64,
    /// Transposition cost
    transpose: f64,
    /// Display name
    name: String,
}

impl<F> fmt::Debug for MatrixCost<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MatrixCost")
            .field("gap_cost", &self.gap_cost)
            .field("transpose", &self.transpose)
            .field("name", &self.name)
            .finish()
    }
}

impl<F> MatrixCost<F> {
    /// Create a matrix-based cost model.
    ///
    /// # Arguments
    ///
    /// * `lookup` - Function returning substitution cost for an element pair
    /// * `gap_cost` - Cost of insertions and deletions
    /// * `name` - Human-readable name (e.g., "BLOSUM62")
    #[must_use]
    pub fn new(lookup: F, gap_cost: f64, name: impl Into<String>) -> Self {
        Self {
            lookup,
            gap_cost,
            transpose: gap_cost,
            name: name.into(),
        }
    }

    /// Set transposition cost separately from gap cost.
    #[must_use]
    pub fn with_transpose_cost(mut self, cost: f64) -> Self {
        self.transpose = cost;
        self
    }
}

impl<E: Eq, F: Fn(&E, &E) -> f64 + Clone + Send + Sync> CostModel<E> for MatrixCost<F> {
    fn insert_cost(&self, _elem: &E) -> f64 {
        self.gap_cost
    }

    fn delete_cost(&self, _elem: &E) -> f64 {
        self.gap_cost
    }

    fn substitute_cost(&self, from: &E, to: &E) -> f64 {
        if from == to {
            0.0
        } else {
            (self.lookup)(from, to)
        }
    }

    fn transpose_cost(&self, _first: &E, _second: &E) -> f64 {
        self.transpose
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_cost_match_is_zero() {
        let cost = UniformCost;
        assert!((cost.substitute_cost(&'a', &'a') - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uniform_cost_mismatch_is_one() {
        let cost = UniformCost;
        assert!((cost.substitute_cost(&'a', &'b') - 1.0).abs() < f64::EPSILON);
        assert!((cost.insert_cost(&'x') - 1.0).abs() < f64::EPSILON);
        assert!((cost.delete_cost(&'x') - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_cost_custom_values() {
        let cost = WeightedCost::new(0.5, 0.5, 2.0, 1.5);
        assert!((cost.insert_cost(&'a') - 0.5).abs() < f64::EPSILON);
        assert!((cost.delete_cost(&'a') - 0.5).abs() < f64::EPSILON);
        assert!((cost.substitute_cost(&'a', &'b') - 2.0).abs() < f64::EPSILON);
        assert!((cost.substitute_cost(&'a', &'a') - 0.0).abs() < f64::EPSILON);
        assert!((cost.transpose_cost(&'a', &'b') - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn matrix_cost_lookup() {
        // Simple keyboard-distance mock: 'a'-'b' = 1.0, 'a'-'z' = 5.0
        let cost = MatrixCost::new(
            |from: &char, to: &char| {
                let dist = (*from as i32 - *to as i32).unsigned_abs();
                f64::from(dist)
            },
            2.0,
            "keyboard-mock",
        );

        assert!((cost.substitute_cost(&'a', &'b') - 1.0).abs() < f64::EPSILON);
        assert!((cost.substitute_cost(&'a', &'a') - 0.0).abs() < f64::EPSILON);
        assert!((cost.insert_cost(&'x') - 2.0).abs() < f64::EPSILON);
        assert_eq!(cost.name(), "keyboard-mock");
    }
}
