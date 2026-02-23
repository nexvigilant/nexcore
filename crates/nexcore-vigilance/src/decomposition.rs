//! # ToV §2: Axiom 1 - System Decomposition
//!
//! Formal implementation of Axiom 1 and its prerequisites (Definitions 2.1-2.8).
//!
//! ## Axiom Statement
//!
//! For every vigilance system 𝒱 = (𝒮, 𝒰, ℳ, ℋ) with state space (S, τ), there exists:
//! 1. A finite element set E with |E| = n < ∞
//! 2. A composition function Φ: 𝒫(E) → S
//!
//! such that the decomposition (E, Φ) is complete.
//!
//! ## Symbolic Formulation
//!
//! **∀𝒱 = (𝒮, 𝒰, ℳ, ℋ) : ∃E, Φ [ |E| < ∞ ∧ Φ: 𝒫(E) ↠ S_acc ]**
//!
//! ## Wolfram Validation (2026-01-29)
//!
//! | Property | Formula | Result |
//! |----------|---------|--------|
//! | Power set size | 2^15 | 32768 |
//! | Feasible limit | 2^20 | 1048576 |
//! | Expensive limit | 2^30 | 1073741824 |

use crate::grounded::{ComplexityChi, QuantityUnit, StabilityShell, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 2.1: ELEMENT
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 2.1: An element is a tuple e = (id, P).
///
/// - `id ∈ ℕ` is a unique identifier
/// - `P = (p₁, p₂, ..., pₖ) ∈ ℝᵏ` is a property vector
///
/// # Remark
/// The dimension k is assumed uniform across all elements within a given
/// decomposition. This can be relaxed when domain requirements dictate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Unique identifier (id ∈ ℕ)
    pub id: u64,
    /// Property vector P ∈ ℝᵏ
    pub properties: Vec<(f64, UnitId)>,
}

impl Element {
    /// Create a new element with given id and properties.
    #[must_use]
    pub fn new(id: u64, properties: Vec<(f64, UnitId)>) -> Self {
        Self { id, properties }
    }

    /// Property vector dimension k.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.properties.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 2.2: ELEMENT SET
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 2.2: An element set is a finite collection E = {e₁, e₂, ..., eₙ}.
///
/// Constraints:
/// - |E| = n < ∞ (finite)
/// - All identifiers are distinct
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementSet {
    /// The elements in the set
    elements: Vec<Element>,
    /// Index mapping id → position for O(1) lookup
    #[serde(skip)]
    id_index: HashMap<u64, usize>,
}

impl ElementSet {
    /// Create a new element set, validating uniqueness.
    ///
    /// # Errors
    /// Returns `Err` if duplicate element IDs are found.
    pub fn new(elements: Vec<Element>) -> Result<Self, DecompositionError> {
        let mut id_index = HashMap::with_capacity(elements.len());
        for (idx, elem) in elements.iter().enumerate() {
            if id_index.insert(elem.id, idx).is_some() {
                return Err(DecompositionError::DuplicateElementId(elem.id));
            }
        }
        Ok(Self { elements, id_index })
    }

    /// Number of elements |E| = n.
    #[must_use]
    pub fn cardinality(&self) -> usize {
        self.elements.len()
    }

    /// Size of the power set |𝒫(E)| = 2^n.
    #[must_use]
    pub fn power_set_size(&self) -> u64 {
        let n = self.elements.len();
        if n < 64 { 1u64 << n } else { u64::MAX }
    }

    /// Get element by id.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&Element> {
        self.id_index.get(&id).map(|&idx| &self.elements[idx])
    }

    /// Get all elements.
    #[must_use]
    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// Check if property dimensions are uniform across elements.
    #[must_use]
    pub fn has_uniform_dimension(&self) -> bool {
        if self.elements.is_empty() {
            return true;
        }
        let k = self.elements[0].dimension();
        self.elements.iter().all(|e| e.dimension() == k)
    }

    /// Iterate over all element IDs.
    pub fn ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.elements.iter().map(|e| e.id)
    }
    /// Check if the element set is in an Island of Stability (§66.1).
    #[must_use]
    pub fn is_stable(&self) -> bool {
        let chi = ComplexityChi(QuantityUnit(self.cardinality() as u64));
        chi.is_closed_shell()
    }

    /// Distance to the nearest magic number shell.
    #[must_use]
    pub fn distance_to_stability(&self) -> i64 {
        let chi = ComplexityChi(QuantityUnit(self.cardinality() as u64));
        chi.distance_to_stability()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 2.3: INTERACTION FUNCTION
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 2.3: An interaction function Iᵢ: E → ℝ.
///
/// For element eᵢ ∈ E, Iᵢ(eⱼ) quantifies the strength of interaction from eᵢ to eⱼ.
/// The collection {Iᵢ}ᵢ₌₁ⁿ defines all pairwise interactions on E.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMatrix {
    /// Interaction weights I[i][j] = strength from eᵢ to eⱼ
    weights: Vec<Vec<f64>>,
    /// Element IDs in order
    element_ids: Vec<u64>,
}

impl InteractionMatrix {
    /// Create interaction matrix for an element set.
    ///
    /// Initializes all interactions to zero.
    #[must_use]
    pub fn new(element_set: &ElementSet) -> Self {
        let n = element_set.cardinality();
        let element_ids: Vec<u64> = element_set.ids().collect();
        let weights = vec![vec![0.0; n]; n];
        Self {
            weights,
            element_ids,
        }
    }

    /// Set interaction strength from element i to element j.
    ///
    /// # Errors
    /// Returns `Err` if either element ID is not found.
    pub fn set_interaction(
        &mut self,
        from_id: u64,
        to_id: u64,
        strength: f64,
    ) -> Result<(), DecompositionError> {
        let from_idx = self.index_of(from_id)?;
        let to_idx = self.index_of(to_id)?;
        self.weights[from_idx][to_idx] = strength;
        Ok(())
    }

    /// Get interaction strength Iᵢ(eⱼ).
    #[must_use]
    pub fn get_interaction(&self, from_id: u64, to_id: u64) -> Option<f64> {
        let from_idx = self.index_of(from_id).ok()?;
        let to_idx = self.index_of(to_id).ok()?;
        Some(self.weights[from_idx][to_idx])
    }

    /// Get index of element by ID.
    fn index_of(&self, id: u64) -> Result<usize, DecompositionError> {
        self.element_ids
            .iter()
            .position(|&eid| eid == id)
            .ok_or(DecompositionError::ElementNotFound(id))
    }

    /// Number of non-zero interactions (edge count).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.weights
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&w| w != 0.0)
            .count()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 2.4: INTERACTION GRAPH
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 2.4: The interaction graph G_E = (E, ℰ, ω).
///
/// - E is the vertex set (elements as vertices)
/// - ℰ = {(eᵢ, eⱼ) ∈ E × E : Iᵢ(eⱼ) ≠ 0} is the edge set
/// - ω: ℰ → ℝ is the weight function with ω(eᵢ, eⱼ) = Iᵢ(eⱼ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionGraph {
    /// Element set (vertices)
    pub element_set: ElementSet,
    /// Interaction matrix (edges with weights)
    pub interactions: InteractionMatrix,
}

impl InteractionGraph {
    /// Create an interaction graph from an element set.
    #[must_use]
    pub fn new(element_set: ElementSet) -> Self {
        let interactions = InteractionMatrix::new(&element_set);
        Self {
            element_set,
            interactions,
        }
    }

    /// Number of vertices |E|.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.element_set.cardinality()
    }

    /// Number of edges |ℰ| (non-zero interactions).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.interactions.edge_count()
    }

    /// Check if graph is connected (every element reachable from every other).
    #[must_use]
    pub fn is_connected(&self) -> bool {
        if self.vertex_count() <= 1 {
            return true;
        }

        // BFS from first element
        let mut visited = HashSet::new();
        let mut queue = vec![0usize];
        visited.insert(0);

        while let Some(idx) = queue.pop() {
            for (j, &weight) in self.interactions.weights[idx].iter().enumerate() {
                if weight != 0.0 && !visited.contains(&j) {
                    visited.insert(j);
                    queue.push(j);
                }
            }
            // Also check incoming edges (undirected connectivity)
            for (i, row) in self.interactions.weights.iter().enumerate() {
                if row[idx] != 0.0 && !visited.contains(&i) {
                    visited.insert(i);
                    queue.push(i);
                }
            }
        }

        visited.len() == self.vertex_count()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 2.5: COMPOSITION FUNCTION
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 2.5: A composition function Φ: 𝒫(E) → S.
///
/// Maps each subset of elements A ⊆ E to a state s = Φ(A) ∈ S.
///
/// # Remark
/// Φ(∅) represents the baseline/null state—the state when no elements are
/// active. This corresponds to a quiescent state, ground state, or system
/// default depending on the domain.
pub trait CompositionFunction {
    /// The state type S.
    type State;

    /// Apply composition: Φ(A) → s ∈ S.
    ///
    /// Given a subset of element IDs, return the composed state.
    fn compose(&self, element_ids: &HashSet<u64>) -> Self::State;

    /// Baseline state Φ(∅).
    fn null_state(&self) -> Self::State {
        self.compose(&HashSet::new())
    }
}

/// A simple vector-sum composition function.
///
/// Φ(A) = Σᵢ∈A eᵢ.P (sum of property vectors)
#[derive(Debug, Clone)]
pub struct SumComposition {
    element_set: ElementSet,
    #[allow(dead_code)] // Cached for validation - may expose via getter
    dimension: usize,
    null_state: Vec<f64>,
}

impl SumComposition {
    /// Create a sum-based composition function.
    ///
    /// # Errors
    /// Returns `Err` if elements have non-uniform dimensions.
    pub fn new(element_set: ElementSet) -> Result<Self, DecompositionError> {
        if !element_set.has_uniform_dimension() {
            return Err(DecompositionError::NonUniformDimension);
        }
        let dimension = element_set.elements().first().map_or(0, Element::dimension);
        Ok(Self {
            element_set,
            dimension,
            null_state: vec![0.0; dimension],
        })
    }
}

impl CompositionFunction for SumComposition {
    type State = Vec<f64>;

    fn compose(&self, element_ids: &HashSet<u64>) -> Self::State {
        let mut state = self.null_state.clone();
        for &id in element_ids {
            if let Some(elem) = self.element_set.get(id) {
                for (i, (val, _unit)) in elem.properties.iter().enumerate() {
                    // INVARIANT: In a more rigorous implementation, we would verify
                    // that units match across elements for the same property index.
                    state[i] += val;
                }
            }
        }
        state
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 2.6: ACCESSIBLE STATE SPACE
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 2.6: The accessible state space S_acc ⊆ S.
///
/// The set of all states reachable from admissible initial conditions
/// under system dynamics and perturbations in 𝒰.
///
/// S_acc = {s ∈ S : ∃s₀ ∈ S_init, ∃u ∈ 𝒰, ∃t ≥ 0 such that s = φₜ(s₀; u)}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibleStateSpace<S> {
    /// Set of accessible states (may be sampled/approximate)
    pub states: Vec<S>,
    /// Whether this is a complete enumeration
    pub is_complete: bool,
}

impl<S: Clone + PartialEq> AccessibleStateSpace<S> {
    /// Create an empty accessible state space.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            states: Vec::new(),
            is_complete: true,
        }
    }

    /// Create from a vector of states.
    #[must_use]
    pub fn from_states(states: Vec<S>, is_complete: bool) -> Self {
        Self {
            states,
            is_complete,
        }
    }

    /// Check if a state is accessible.
    #[must_use]
    pub fn contains(&self, state: &S) -> bool {
        self.states.iter().any(|s| s == state)
    }

    /// Number of accessible states.
    #[must_use]
    pub fn cardinality(&self) -> usize {
        self.states.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 2.7 & 2.8: DECOMPOSITION (COMPLETE & MINIMAL)
// ═══════════════════════════════════════════════════════════════════════════

/// Definitions 2.7-2.8: A decomposition (E, Φ) of a system.
///
/// - **Complete**: Φ is surjective onto S_acc (every reachable state has a preimage)
/// - **Minimal**: No proper subset E' ⊂ E yields a complete decomposition
#[derive(Debug, Clone)]
pub struct Decomposition<F: CompositionFunction> {
    /// Element set E
    pub element_set: ElementSet,
    /// Composition function Φ
    pub composition: F,
}

impl<F: CompositionFunction> Decomposition<F>
where
    F::State: Clone + PartialEq,
{
    /// Create a decomposition.
    #[must_use]
    pub fn new(element_set: ElementSet, composition: F) -> Self {
        Self {
            element_set,
            composition,
        }
    }

    /// Element dimension |E|.
    #[must_use]
    pub fn element_dimension(&self) -> usize {
        self.element_set.cardinality()
    }

    /// Check if decomposition is complete (Definition 2.7).
    ///
    /// A decomposition is complete if Φ is surjective onto S_acc.
    /// For every s ∈ S_acc, ∃A ⊆ E such that Φ(A) = s.
    #[must_use]
    pub fn is_complete(&self, accessible_states: &AccessibleStateSpace<F::State>) -> bool {
        // Every accessible state must be reachable via some subset
        for state in &accessible_states.states {
            if !self.has_preimage(state) {
                return false;
            }
        }
        true
    }

    /// Check if a state has a preimage under Φ.
    ///
    /// Searches through power set (exponential, use for small |E|).
    fn has_preimage(&self, target: &F::State) -> bool {
        let n = self.element_set.cardinality();
        if n > 20 {
            // Too large for exhaustive search
            return false;
        }

        let ids: Vec<u64> = self.element_set.ids().collect();

        // Enumerate all 2^n subsets
        for mask in 0..(1u64 << n) {
            let subset: HashSet<u64> = ids
                .iter()
                .enumerate()
                .filter(|(i, _)| (mask >> i) & 1 == 1)
                .map(|(_, &id)| id)
                .collect();

            let state = self.composition.compose(&subset);
            if &state == target {
                return true;
            }
        }

        false
    }

    /// Find preimage of a state (subset A such that Φ(A) = s).
    #[must_use]
    pub fn find_preimage(&self, target: &F::State) -> Option<HashSet<u64>> {
        let n = self.element_set.cardinality();
        if n > 20 {
            return None;
        }

        let ids: Vec<u64> = self.element_set.ids().collect();

        for mask in 0..(1u64 << n) {
            let subset: HashSet<u64> = ids
                .iter()
                .enumerate()
                .filter(|(i, _)| (mask >> i) & 1 == 1)
                .map(|(_, &id)| id)
                .collect();

            let state = self.composition.compose(&subset);
            if &state == target {
                return Some(subset);
            }
        }

        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 1 VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 1 verification result.
///
/// ∀𝒱 : ∃E, Φ [ |E| < ∞ ∧ Φ: 𝒫(E) ↠ S_acc ]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axiom1Verification {
    /// Element set cardinality |E|
    pub element_count: usize,
    /// Power set size |𝒫(E)| = 2^n
    pub power_set_size: u64,
    /// Whether decomposition is complete (Φ surjective onto S_acc)
    pub is_complete: bool,
    /// Computational tractability
    pub is_tractable: bool,
    /// Stability status of the decomposition
    pub is_stable: bool,
    /// Distance to architectural stability
    pub stability_distance: i64,
    /// Axiom 1 satisfied
    pub axiom_satisfied: bool,
}

impl Axiom1Verification {
    /// Verify Axiom 1 for a decomposition.
    #[must_use]
    pub fn verify<F: CompositionFunction>(
        decomposition: &Decomposition<F>,
        accessible_states: &AccessibleStateSpace<F::State>,
    ) -> Self
    where
        F::State: Clone + PartialEq,
    {
        let element_count = decomposition.element_dimension();
        let power_set_size = decomposition.element_set.power_set_size();
        let is_complete = decomposition.is_complete(accessible_states);
        let is_tractable = element_count <= 20;
        let is_stable = decomposition.element_set.is_stable();
        let stability_distance = decomposition.element_set.distance_to_stability();

        // Axiom 1: |E| < ∞ (always true for finite element set) AND Φ surjective
        let axiom_satisfied = is_complete;

        Self {
            element_count,
            power_set_size,
            is_complete,
            is_tractable,
            is_stable,
            stability_distance,
            axiom_satisfied,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════════

/// Errors related to system decomposition.
#[derive(Debug, Clone, nexcore_error::Error)]
pub enum DecompositionError {
    /// Duplicate element ID in element set.
    #[error("Duplicate element ID: {0}")]
    DuplicateElementId(u64),

    /// Element not found in set.
    #[error("Element not found: {0}")]
    ElementNotFound(u64),

    /// Elements have non-uniform property dimensions.
    #[error("Elements have non-uniform property dimensions")]
    NonUniformDimension,

    /// Decomposition is not complete.
    #[error("Decomposition is not complete: missing preimages for {0} states")]
    NotComplete(usize),
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let e = Element::new(
            1,
            vec![
                (1.0, UnitId::Dimensionless),
                (2.0, UnitId::Dimensionless),
                (3.0, UnitId::Dimensionless),
            ],
        );
        assert_eq!(e.id, 1);
        assert_eq!(e.dimension(), 3);
        assert_eq!(
            e.properties,
            vec![
                (1.0, UnitId::Dimensionless),
                (2.0, UnitId::Dimensionless),
                (3.0, UnitId::Dimensionless)
            ]
        );
    }

    #[test]
    fn test_element_set_creation() {
        let elements = vec![
            Element::new(1, vec![(1.0, UnitId::Dimensionless)]),
            Element::new(2, vec![(2.0, UnitId::Dimensionless)]),
            Element::new(3, vec![(3.0, UnitId::Dimensionless)]),
        ];
        let set = ElementSet::new(elements).unwrap();

        assert_eq!(set.cardinality(), 3);
        assert_eq!(set.power_set_size(), 8); // 2^3 = 8
        assert!(set.has_uniform_dimension());
    }

    #[test]
    fn test_element_set_duplicate_error() {
        let elements = vec![
            Element::new(1, vec![(1.0, UnitId::Dimensionless)]),
            Element::new(1, vec![(2.0, UnitId::Dimensionless)]),
        ];
        let result = ElementSet::new(elements);
        assert!(matches!(
            result,
            Err(DecompositionError::DuplicateElementId(1))
        ));
    }

    #[test]
    fn test_power_set_size_wolfram_validated() {
        // Wolfram: 2^15 = 32768
        let elements: Vec<Element> = (0..15)
            .map(|i| Element::new(i, vec![(i as f64, UnitId::Dimensionless)]))
            .collect();
        let set = ElementSet::new(elements).unwrap();
        assert_eq!(set.power_set_size(), 32768);
    }

    #[test]
    fn test_interaction_matrix() {
        let elements = vec![
            Element::new(1, vec![(1.0, UnitId::Dimensionless)]),
            Element::new(2, vec![(2.0, UnitId::Dimensionless)]),
            Element::new(3, vec![(3.0, UnitId::Dimensionless)]),
        ];
        let set = ElementSet::new(elements).unwrap();
        let mut matrix = InteractionMatrix::new(&set);

        matrix.set_interaction(1, 2, 0.5).unwrap();
        matrix.set_interaction(2, 3, 0.7).unwrap();

        assert_eq!(matrix.get_interaction(1, 2), Some(0.5));
        assert_eq!(matrix.get_interaction(2, 3), Some(0.7));
        assert_eq!(matrix.get_interaction(1, 3), Some(0.0));
        assert_eq!(matrix.edge_count(), 2);
    }

    #[test]
    fn test_interaction_graph_connectivity() {
        let elements = vec![
            Element::new(1, vec![(1.0, UnitId::Dimensionless)]),
            Element::new(2, vec![(2.0, UnitId::Dimensionless)]),
            Element::new(3, vec![(3.0, UnitId::Dimensionless)]),
        ];
        let set = ElementSet::new(elements).unwrap();
        let mut graph = InteractionGraph::new(set);

        // Not connected initially
        assert!(!graph.is_connected());

        // Add edges to make connected
        graph.interactions.set_interaction(1, 2, 1.0).unwrap();
        graph.interactions.set_interaction(2, 3, 1.0).unwrap();

        assert!(graph.is_connected());
    }

    #[test]
    fn test_sum_composition() {
        let elements = vec![
            Element::new(
                1,
                vec![(1.0, UnitId::Dimensionless), (0.0, UnitId::Dimensionless)],
            ),
            Element::new(
                2,
                vec![(0.0, UnitId::Dimensionless), (2.0, UnitId::Dimensionless)],
            ),
            Element::new(
                3,
                vec![(3.0, UnitId::Dimensionless), (3.0, UnitId::Dimensionless)],
            ),
        ];
        let set = ElementSet::new(elements).unwrap();
        let comp = SumComposition::new(set).unwrap();

        // Empty set → null state
        let null = comp.compose(&HashSet::new());
        assert_eq!(null, vec![0.0, 0.0]);

        // Single element
        let single = comp.compose(&HashSet::from([1]));
        assert_eq!(single, vec![1.0, 0.0]);

        // Multiple elements
        let multiple = comp.compose(&HashSet::from([1, 2, 3]));
        assert_eq!(multiple, vec![4.0, 5.0]); // [1+0+3, 0+2+3]
    }

    #[test]
    fn test_decomposition_completeness() {
        let elements = vec![
            Element::new(1, vec![(1.0, UnitId::Dimensionless)]),
            Element::new(2, vec![(2.0, UnitId::Dimensionless)]),
        ];
        let set = ElementSet::new(elements).unwrap();
        let comp = SumComposition::new(set.clone()).unwrap();
        let decomp = Decomposition::new(set, comp);

        // Accessible states that CAN be composed
        let accessible = AccessibleStateSpace::from_states(
            vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0]],
            true,
        );

        assert!(decomp.is_complete(&accessible));

        // Add a state that cannot be composed
        let inaccessible = AccessibleStateSpace::from_states(
            vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0], vec![5.0]],
            true,
        );

        assert!(!decomp.is_complete(&inaccessible));
    }

    #[test]
    fn test_axiom1_verification() {
        let elements = vec![
            Element::new(1, vec![(1.0, UnitId::Dimensionless)]),
            Element::new(2, vec![(2.0, UnitId::Dimensionless)]),
        ];
        let set = ElementSet::new(elements).unwrap();
        let comp = SumComposition::new(set.clone()).unwrap();
        let decomp = Decomposition::new(set, comp);

        let accessible = AccessibleStateSpace::from_states(
            vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0]],
            true,
        );

        let verification = Axiom1Verification::verify(&decomp, &accessible);

        assert_eq!(verification.element_count, 2);
        assert_eq!(verification.power_set_size, 4); // 2^2
        assert!(verification.is_complete);
        assert!(verification.is_tractable);
        assert!(verification.axiom_satisfied);
    }

    #[test]
    fn test_find_preimage() {
        let elements = vec![
            Element::new(1, vec![(1.0, UnitId::Dimensionless)]),
            Element::new(2, vec![(2.0, UnitId::Dimensionless)]),
            Element::new(3, vec![(4.0, UnitId::Dimensionless)]),
        ];
        let set = ElementSet::new(elements).unwrap();
        let comp = SumComposition::new(set.clone()).unwrap();
        let decomp = Decomposition::new(set, comp);

        // State [3.0] = e1 + e2
        let preimage = decomp.find_preimage(&vec![3.0]);
        assert_eq!(preimage, Some(HashSet::from([1, 2])));

        // State [7.0] = e1 + e2 + e3
        let preimage2 = decomp.find_preimage(&vec![7.0]);
        assert_eq!(preimage2, Some(HashSet::from([1, 2, 3])));

        // State [10.0] has no preimage
        let preimage3 = decomp.find_preimage(&vec![10.0]);
        assert_eq!(preimage3, None);
    }
}
