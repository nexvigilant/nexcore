//! # EditMetric Compositor (T2-C Factory)
//!
//! Binds operations + cost model + solver into a reusable, named metric.
//! This is the reified configuration space from the composition analysis:
//!
//! ```text
//! EditMetric<O, C, S> = edit_distance(ops=O, costs=C, solver=S)
//! ```
//!
//! Pre-built type aliases provide the T3 instantiations:
//! - `Levenshtein` = StdOps + UniformCost + TwoRowDp
//! - `DamerauLev` = DamerauOps + UniformCost + FullMatrixDp
//! - `Lcs` = IndelOps + UniformCost + TwoRowDp

use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::cost::{CostModel, UniformCost};
use crate::ops::{DamerauOps, EditOp, IndelOps, OperationSet, StdOps};
use crate::solver::{FullMatrixDp, SolveResult, Solver, TwoRowDp};

/// A complete edit distance metric binding ops + cost + solver.
///
/// This struct is the generic compositor — the T2-C factory from the
/// primitive extraction. Each field is a pluggable parameter.
#[derive(Clone)]
pub struct EditMetric<E, O, C, S>
where
    E: Clone + Eq,
    O: OperationSet,
    C: CostModel<E>,
    S: Solver<E, C>,
{
    /// Which edit operations are allowed
    pub ops: O,
    /// How much each operation costs
    pub cost: C,
    /// Algorithm used to compute the distance
    pub solver: S,
    _phantom: PhantomData<E>,
}

impl<E, O, C, S> fmt::Debug for EditMetric<E, O, C, S>
where
    E: Clone + Eq,
    O: OperationSet,
    C: CostModel<E>,
    S: Solver<E, C>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EditMetric")
            .field("ops", &self.ops.name())
            .field("cost", &self.cost.name())
            .field("solver", &self.solver.name())
            .finish()
    }
}

impl<E, O, C, S> EditMetric<E, O, C, S>
where
    E: Clone + Eq,
    O: OperationSet,
    C: CostModel<E>,
    S: Solver<E, C>,
{
    /// Create a new metric from components.
    #[must_use]
    pub fn new(ops: O, cost: C, solver: S) -> Self {
        Self {
            ops,
            cost,
            solver,
            _phantom: PhantomData,
        }
    }

    /// Compute edit distance between two slices.
    pub fn distance(&self, source: &[E], target: &[E]) -> f64 {
        self.solver
            .solve(source, target, &self.ops, &self.cost)
            .distance
    }

    /// Compute full result including optional traceback.
    pub fn solve(&self, source: &[E], target: &[E]) -> SolveResult<E> {
        self.solver.solve(source, target, &self.ops, &self.cost)
    }

    /// Compute normalized similarity in [0.0, 1.0].
    pub fn similarity(&self, source: &[E], target: &[E]) -> f64 {
        let d = self.distance(source, target);
        let max_len = source.len().max(target.len());
        if max_len == 0 {
            1.0
        } else {
            1.0 - (d / max_len as f64)
        }
    }

    /// Whether the underlying solver supports traceback.
    pub fn supports_traceback(&self) -> bool {
        self.solver.supports_traceback()
    }

    /// Get the operation sequence for a transformation (if solver supports it).
    pub fn operations(&self, source: &[E], target: &[E]) -> Option<Vec<EditOp<E>>> {
        self.solve(source, target).operations
    }

    /// Name of the operation set.
    pub fn ops_name(&self) -> &str {
        self.ops.name()
    }

    /// Name of the cost model.
    pub fn cost_name(&self) -> &str {
        self.cost.name()
    }

    /// Name of the solver.
    pub fn solver_name(&self) -> &str {
        self.solver.name()
    }
}

/// String-specific convenience methods.
impl<O, C, S> EditMetric<char, O, C, S>
where
    O: OperationSet,
    C: CostModel<char>,
    S: Solver<char, C>,
{
    /// Compute edit distance between two strings.
    pub fn str_distance(&self, source: &str, target: &str) -> f64 {
        let src: Vec<char> = source.chars().collect();
        let tgt: Vec<char> = target.chars().collect();
        self.distance(&src, &tgt)
    }

    /// Compute normalized similarity between two strings.
    pub fn str_similarity(&self, source: &str, target: &str) -> f64 {
        let src: Vec<char> = source.chars().collect();
        let tgt: Vec<char> = target.chars().collect();
        self.similarity(&src, &tgt)
    }

    /// Compute full result for two strings.
    pub fn str_solve(&self, source: &str, target: &str) -> SolveResult<char> {
        let src: Vec<char> = source.chars().collect();
        let tgt: Vec<char> = target.chars().collect();
        self.solve(&src, &tgt)
    }
}

// ---------------------------------------------------------------------------
// Typestate Builder — compile-time guarantee all fields are set
// ---------------------------------------------------------------------------

/// Marker: field not yet set.
#[derive(Debug)]
pub struct Missing;

/// Marker: field is set with value `T`.
#[derive(Debug)]
pub struct Set<T>(T);

/// Typestate builder for `EditMetric`. Only compiles when all three fields are `Set<_>`.
pub struct EditMetricBuilder<E, O, C, S> {
    ops: O,
    cost: C,
    solver: S,
    _phantom: PhantomData<E>,
}

impl<E: Clone + Eq> EditMetricBuilder<E, Missing, Missing, Missing> {
    /// Start building an `EditMetric`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ops: Missing,
            cost: Missing,
            solver: Missing,
            _phantom: PhantomData,
        }
    }
}

impl<E: Clone + Eq> Default for EditMetricBuilder<E, Missing, Missing, Missing> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Clone + Eq, C, S> EditMetricBuilder<E, Missing, C, S> {
    /// Set the operation set.
    pub fn ops<O: OperationSet>(self, ops: O) -> EditMetricBuilder<E, Set<O>, C, S> {
        EditMetricBuilder {
            ops: Set(ops),
            cost: self.cost,
            solver: self.solver,
            _phantom: PhantomData,
        }
    }
}

impl<E: Clone + Eq, O, S> EditMetricBuilder<E, O, Missing, S> {
    /// Set the cost model.
    pub fn cost<Cm: CostModel<E>>(self, cost: Cm) -> EditMetricBuilder<E, O, Set<Cm>, S> {
        EditMetricBuilder {
            ops: self.ops,
            cost: Set(cost),
            solver: self.solver,
            _phantom: PhantomData,
        }
    }
}

impl<E: Clone + Eq, O, C: CostModel<E>> EditMetricBuilder<E, O, Set<C>, Missing> {
    /// Set the solver. Cost must be set first (it constrains valid solvers).
    pub fn solver<Sv: Solver<E, C>>(self, solver: Sv) -> EditMetricBuilder<E, O, Set<C>, Set<Sv>> {
        EditMetricBuilder {
            ops: self.ops,
            cost: self.cost,
            solver: Set(solver),
            _phantom: PhantomData,
        }
    }
}

impl<E, O, C, S> EditMetricBuilder<E, Set<O>, Set<C>, Set<S>>
where
    E: Clone + Eq,
    O: OperationSet,
    C: CostModel<E>,
    S: Solver<E, C>,
{
    /// Build the `EditMetric`. Type system guarantees all fields are set.
    #[must_use]
    pub fn build(self) -> EditMetric<E, O, C, S> {
        EditMetric::new(self.ops.0, self.cost.0, self.solver.0)
    }
}

// ---------------------------------------------------------------------------
// Pre-built T3 type aliases
// ---------------------------------------------------------------------------

/// Classic Levenshtein: ins/del/sub with unit cost, two-row DP.
pub type Levenshtein = EditMetric<char, StdOps, UniformCost, TwoRowDp>;

/// Damerau-Levenshtein: adds transposition, requires full matrix for DP.
pub type DamerauLev = EditMetric<char, DamerauOps, UniformCost, FullMatrixDp>;

/// LCS distance: indel-only (no substitution), two-row DP.
pub type Lcs = EditMetric<char, IndelOps, UniformCost, TwoRowDp>;

/// Levenshtein with traceback support (full matrix).
pub type LevenshteinTraceback = EditMetric<char, StdOps, UniformCost, FullMatrixDp>;

// ---------------------------------------------------------------------------
// Default constructors for type aliases
// ---------------------------------------------------------------------------

impl Default for Levenshtein {
    fn default() -> Self {
        Self::new(StdOps, UniformCost, TwoRowDp)
    }
}

impl Default for DamerauLev {
    fn default() -> Self {
        Self::new(DamerauOps, UniformCost, FullMatrixDp)
    }
}

impl Default for Lcs {
    fn default() -> Self {
        Self::new(IndelOps, UniformCost, TwoRowDp)
    }
}

impl Default for LevenshteinTraceback {
    fn default() -> Self {
        Self::new(StdOps, UniformCost, FullMatrixDp)
    }
}

// ---------------------------------------------------------------------------
// Serialize/Deserialize
// ---------------------------------------------------------------------------

impl<E, O, C, S> Serialize for EditMetric<E, O, C, S>
where
    E: Clone + Eq,
    O: OperationSet + Serialize,
    C: CostModel<E> + Serialize,
    S: Solver<E, C> + Serialize,
{
    fn serialize<Ser: serde::Serializer>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("EditMetric", 3)?;
        s.serialize_field("ops", &self.ops)?;
        s.serialize_field("cost", &self.cost)?;
        s.serialize_field("solver", &self.solver)?;
        s.end()
    }
}

impl<'de, E, O, C, S> Deserialize<'de> for EditMetric<E, O, C, S>
where
    E: Clone + Eq,
    O: OperationSet + Deserialize<'de>,
    C: CostModel<E> + Deserialize<'de>,
    S: Solver<E, C> + Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Inner<O2, C2, S2> {
            ops: O2,
            cost: C2,
            solver: S2,
        }
        let inner = Inner::<O, C, S>::deserialize(deserializer)?;
        Ok(Self::new(inner.ops, inner.cost, inner.solver))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::WeightedCost;
    use crate::solver::BandedDp;

    #[test]
    fn levenshtein_type_alias() {
        let m = Levenshtein::default();
        assert!((m.str_distance("kitten", "sitting") - 3.0).abs() < f64::EPSILON);
        assert!(!m.supports_traceback());
    }

    #[test]
    fn damerau_type_alias() {
        let m = DamerauLev::default();
        assert!((m.str_distance("ab", "ba") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lcs_type_alias() {
        let m = Lcs::default();
        assert!((m.str_distance("abc", "abc") - 0.0).abs() < f64::EPSILON);
        let d = m.str_distance("ab", "ba");
        assert!((d - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn traceback_type_alias() {
        let m = LevenshteinTraceback::default();
        assert!(m.supports_traceback());
        let result = m.str_solve("kitten", "sitting");
        assert!(result.operations.is_some());
    }

    #[test]
    fn similarity_calculation() {
        let m = Levenshtein::default();
        let sim = m.str_similarity("hello", "hallo");
        assert!((sim - 0.8).abs() < f64::EPSILON);
        assert!((m.str_similarity("abc", "abc") - 1.0).abs() < f64::EPSILON);
        assert!((m.str_similarity("", "") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_pattern() {
        let m = EditMetricBuilder::<char, _, _, _>::new()
            .ops(StdOps)
            .cost(UniformCost)
            .solver(TwoRowDp)
            .build();
        assert!((m.str_distance("kitten", "sitting") - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_with_custom_cost() {
        let m = EditMetricBuilder::<char, _, _, _>::new()
            .ops(StdOps)
            .cost(WeightedCost::new(1.0, 1.0, 2.0, 1.0))
            .solver(TwoRowDp)
            .build();
        // "hello" -> "hallo": one substitution at cost 2.0
        assert!((m.str_distance("hello", "hallo") - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_with_banded_solver() {
        let m = EditMetricBuilder::<char, _, _, _>::new()
            .ops(StdOps)
            .cost(UniformCost)
            .solver(BandedDp::new(5))
            .build();
        assert!((m.str_distance("kitten", "sitting") - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn slice_api() {
        let m = Levenshtein::default();
        let src: Vec<char> = "abc".chars().collect();
        let tgt: Vec<char> = "axc".chars().collect();
        assert!((m.distance(&src, &tgt) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn debug_display() {
        let m = Levenshtein::default();
        let dbg = format!("{m:?}");
        assert!(dbg.contains("standard"));
        assert!(dbg.contains("uniform"));
        assert!(dbg.contains("two-row-dp"));
    }

    #[test]
    fn serialize_roundtrip() {
        let m = Levenshtein::default();
        let json = serde_json::to_string(&m).expect("serialize");
        let m2: Levenshtein = serde_json::from_str(&json).expect("deserialize");
        assert!(
            (m.str_distance("kitten", "sitting") - m2.str_distance("kitten", "sitting")).abs()
                < f64::EPSILON
        );
    }
}
