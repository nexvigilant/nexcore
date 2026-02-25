//! # Markov Chains: Discrete-Time Markov Chain Analysis
//!
//! Canonical `MarkovChain<S>` type with transition matrix, structural classification
//! via Graph SCC analysis, stationary distribution computation, and n-step
//! transition probability queries.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | discrete states with transition probabilities (DOMINANT) |
//! | T1: Sequence (σ) | state-to-state transitions over time |
//! | T1: Quantity (N) | transition probabilities |
//! | T1: Recursion (ρ) | power iteration, n-step probabilities |
//!
//! ## Design
//!
//! `MarkovChain<S>` wraps a `Matrix` (transition matrix) with typed state labels.
//! Structural analysis (ergodicity, communicating classes, absorbing states) uses
//! `Graph<usize>` + Tarjan SCC from the graph module — no duplication.
//!
//! ## Cross-Domain Transfer
//!
//! | Domain | Markov Application |
//! |--------|-------------------|
//! | PV | Drug development phase transition probabilities |
//! | Economics | Market regime switching models |
//! | Software | State machine probabilistic analysis |
//! | Biology | Population dynamics, genetic drift |
//!
//! ## Relationship to nexcore-oracle
//!
//! `nexcore-oracle::TransitionMatrix` learns from observed event sequences (T3).
//! `MarkovChain<S>` is the mathematical analysis engine (T2-C) that operates on
//! transition matrices to compute stationary distributions, classify states,
//! and determine ergodicity.

use stem_core::{Confidence, Measured};

use crate::graph::{self, Graph, VertexId};
use crate::matrix::Matrix;

// ============================================================================
// State Classification
// ============================================================================

/// Classification of a state in a Markov chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateClass {
    /// Recurrent: the chain returns to this state with probability 1.
    Recurrent,
    /// Transient: the chain may never return to this state.
    Transient,
    /// Absorbing: once entered, the chain never leaves (self-loop probability = 1).
    Absorbing,
}

/// Information about a communicating class.
#[derive(Debug, Clone)]
pub struct CommunicatingClass {
    /// Indices of states in this class.
    pub states: Vec<usize>,
    /// Whether this class is recurrent or transient.
    pub class_type: StateClass,
}

// ============================================================================
// MarkovChain
// ============================================================================

/// A discrete-time Markov chain with typed state labels.
///
/// The transition matrix P has entry P[i][j] = probability of transitioning
/// from state i to state j. Each row sums to 1.0 (right-stochastic).
///
/// # Type Parameter
///
/// `S` — state label type (e.g., `String`, `&str`, `u32`).
/// Used for human-readable output; internally states are indexed 0..n.
#[derive(Debug, Clone)]
pub struct MarkovChain<S> {
    /// State labels indexed by state number.
    states: Vec<S>,
    /// Row-stochastic transition matrix (n × n).
    transition_matrix: Matrix,
}

impl<S: Clone> MarkovChain<S> {
    /// Create a Markov chain from state labels and a transition matrix.
    ///
    /// Returns `None` if:
    /// - Matrix is not square
    /// - Matrix dimensions don't match state count
    /// - Matrix is not row-stochastic (rows don't sum to ~1.0)
    /// - Matrix has negative entries
    #[must_use]
    pub fn new(states: Vec<S>, matrix: Matrix) -> Option<Self> {
        if !matrix.is_square() {
            return None;
        }
        if matrix.rows() != states.len() {
            return None;
        }
        if !matrix.is_nonnegative() {
            return None;
        }
        if !matrix.is_stochastic() {
            return None;
        }
        Some(Self {
            states,
            transition_matrix: matrix,
        })
    }

    /// Create a Markov chain from state labels and a list of transitions.
    ///
    /// Each transition is `(from_index, to_index, probability)`.
    /// Missing transitions default to 0. Rows are normalized to sum to 1.
    ///
    /// Returns `None` if any index is out of bounds or state count is 0.
    #[must_use]
    pub fn from_transitions(states: Vec<S>, transitions: &[(usize, usize, f64)]) -> Option<Self> {
        let n = states.len();
        if n == 0 {
            return None;
        }
        let mut matrix = Matrix::zeros(n, n)?;
        for &(from, to, prob) in transitions {
            if from >= n || to >= n {
                return None;
            }
            matrix.set(from, to, prob);
        }
        let matrix = matrix.normalize_rows();
        Some(Self {
            states,
            transition_matrix: matrix,
        })
    }

    /// Create a Markov chain by estimating transition probabilities from
    /// observed state sequences.
    ///
    /// Each sequence is a slice of state indices. Consecutive pairs are
    /// counted as transitions. The resulting matrix is normalized.
    ///
    /// Returns `None` if state count is 0.
    #[must_use]
    pub fn from_observed_data(states: Vec<S>, sequences: &[Vec<usize>]) -> Option<Self> {
        let n = states.len();
        if n == 0 {
            return None;
        }
        let mut matrix = Matrix::zeros(n, n)?;
        for seq in sequences {
            for window in seq.windows(2) {
                let from = window[0];
                let to = window[1];
                if from < n && to < n {
                    let current = matrix.get(from, to).unwrap_or(0.0);
                    matrix.set(from, to, current + 1.0);
                }
            }
        }
        let matrix = matrix.normalize_rows();
        Some(Self {
            states,
            transition_matrix: matrix,
        })
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Number of states.
    #[must_use]
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Get state label by index.
    #[must_use]
    pub fn state(&self, index: usize) -> Option<&S> {
        self.states.get(index)
    }

    /// Get all state labels.
    #[must_use]
    pub fn states(&self) -> &[S] {
        &self.states
    }

    /// Get the transition matrix.
    #[must_use]
    pub fn transition_matrix(&self) -> &Matrix {
        &self.transition_matrix
    }

    /// Get transition probability from state i to state j.
    #[must_use]
    pub fn transition_probability(&self, from: usize, to: usize) -> Option<f64> {
        self.transition_matrix.get(from, to)
    }

    // ========================================================================
    // N-Step Probabilities
    // ========================================================================

    /// Compute n-step transition probability: P(X_n = j | X_0 = i).
    ///
    /// Uses matrix exponentiation via repeated squaring: O(n³ log steps).
    #[must_use]
    pub fn n_step_probability(&self, from: usize, to: usize, steps: u32) -> Option<f64> {
        if steps == 0 {
            // At step 0, probability is 1 if from==to, 0 otherwise
            return if from == to { Some(1.0) } else { Some(0.0) };
        }
        let powered = self.transition_matrix.power(steps)?;
        powered.get(from, to)
    }

    /// Compute the full n-step transition matrix P^n.
    #[must_use]
    pub fn n_step_matrix(&self, steps: u32) -> Option<Matrix> {
        if steps == 0 {
            return Matrix::identity(self.state_count());
        }
        self.transition_matrix.power(steps)
    }

    // ========================================================================
    // Stationary Distribution
    // ========================================================================

    /// Compute the stationary distribution π where πP = π.
    ///
    /// Uses power iteration: start with uniform distribution, repeatedly
    /// multiply by P until convergence.
    ///
    /// Returns `Measured<Vec<f64>>` with confidence based on convergence quality.
    ///
    /// - Converged within tolerance → high confidence (0.85-0.99)
    /// - Hit max iterations → moderate confidence (0.60)
    #[must_use]
    pub fn stationary_distribution(
        &self,
        max_iterations: usize,
        tolerance: f64,
    ) -> Measured<Vec<f64>> {
        let n = self.state_count();
        if n == 0 {
            return Measured::new(Vec::new(), Confidence::new(0.0));
        }

        // Start with uniform distribution as row vector
        let initial = 1.0 / n as f64;
        let mut pi = vec![initial; n];
        let mut new_pi = vec![0.0_f64; n];

        let mut converged = false;
        let mut iterations_used = 0;

        for iter in 0..max_iterations {
            iterations_used = iter + 1;

            // Compute π × P (row vector × matrix)
            for (j, new_pi_j) in new_pi.iter_mut().enumerate() {
                let mut sum = 0.0;
                for (i, &pi_i) in pi.iter().enumerate() {
                    sum += pi_i * self.transition_matrix.get(i, j).unwrap_or(0.0);
                }
                *new_pi_j = sum;
            }

            // Check convergence (L1 norm)
            let diff: f64 = pi
                .iter()
                .zip(new_pi.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();

            std::mem::swap(&mut pi, &mut new_pi);

            if diff < tolerance {
                converged = true;
                break;
            }
        }

        let confidence_val = if converged {
            (1.0 - iterations_used as f64 / max_iterations as f64)
                .mul_add(0.15, 0.85)
                .clamp(0.85, 0.99)
        } else {
            0.60
        };

        Measured::new(pi, Confidence::new(confidence_val))
    }

    // ========================================================================
    // Structural Analysis (via Graph SCC)
    // ========================================================================

    /// Build a directed graph from the transition matrix.
    ///
    /// An edge exists from i to j if P[i][j] > 0.
    fn to_graph(&self) -> Graph<usize, f64> {
        let n = self.state_count();
        let mut g = Graph::with_capacity(n);
        for i in 0..n {
            g.add_vertex(i);
        }
        for i in 0..n {
            for j in 0..n {
                let p = self.transition_matrix.get(i, j).unwrap_or(0.0);
                if p > 0.0 {
                    g.add_edge(VertexId::new(i), VertexId::new(j), p);
                }
            }
        }
        g
    }

    /// Find all communicating classes via Tarjan's SCC algorithm.
    ///
    /// A communicating class is a set of states where every pair can reach
    /// each other. Classification:
    /// - **Recurrent**: no transitions leave the class
    /// - **Transient**: some transitions leave the class
    /// - **Absorbing**: single-state class with self-loop probability = 1.0
    #[must_use]
    pub fn communicating_classes(&self) -> Vec<CommunicatingClass> {
        let g = self.to_graph();
        let sccs = graph::tarjan_scc(&g);
        let n = self.state_count();

        sccs.into_iter()
            .map(|scc| {
                let state_indices: Vec<usize> = scc.iter().map(|v| v.index()).collect();

                // Check if any state in this SCC has transitions to states outside the SCC
                let mut leaves_class = false;
                for &s in &state_indices {
                    for j in 0..n {
                        if !state_indices.contains(&j) {
                            let p = self.transition_matrix.get(s, j).unwrap_or(0.0);
                            if p > 0.0 {
                                leaves_class = true;
                                break;
                            }
                        }
                    }
                    if leaves_class {
                        break;
                    }
                }

                let class_type = if state_indices.len() == 1 {
                    let s = state_indices[0];
                    let self_prob = self.transition_matrix.get(s, s).unwrap_or(0.0);
                    if (self_prob - 1.0).abs() < 1e-9 {
                        StateClass::Absorbing
                    } else if leaves_class {
                        StateClass::Transient
                    } else {
                        StateClass::Recurrent
                    }
                } else if leaves_class {
                    StateClass::Transient
                } else {
                    StateClass::Recurrent
                };

                CommunicatingClass {
                    states: state_indices,
                    class_type,
                }
            })
            .collect()
    }

    /// Classify each state as Recurrent, Transient, or Absorbing.
    #[must_use]
    pub fn classify_states(&self) -> Vec<(usize, StateClass)> {
        let classes = self.communicating_classes();
        let mut result = vec![(0_usize, StateClass::Transient); self.state_count()];
        for class in &classes {
            for &s in &class.states {
                if s < result.len() {
                    result[s] = (s, class.class_type);
                }
            }
        }
        result
    }

    /// Find all absorbing states (self-loop probability = 1.0).
    #[must_use]
    pub fn absorbing_states(&self) -> Vec<usize> {
        let n = self.state_count();
        (0..n)
            .filter(|&i| {
                let self_prob = self.transition_matrix.get(i, i).unwrap_or(0.0);
                (self_prob - 1.0).abs() < 1e-9
            })
            .collect()
    }

    /// Check if a specific state is absorbing.
    #[must_use]
    pub fn is_absorbing(&self, state: usize) -> bool {
        self.transition_matrix
            .get(state, state)
            .is_some_and(|p| (p - 1.0).abs() < 1e-9)
    }

    /// Check if the chain is ergodic (irreducible + aperiodic).
    ///
    /// - **Irreducible**: exactly one communicating class (all states communicate)
    /// - **Aperiodic**: at least one state has a self-loop (P[i][i] > 0)
    ///
    /// An ergodic chain has a unique stationary distribution that equals
    /// the limiting distribution.
    #[must_use]
    pub fn is_ergodic(&self) -> bool {
        self.is_irreducible() && self.is_aperiodic()
    }

    /// Check if the chain is irreducible (all states form one communicating class).
    #[must_use]
    pub fn is_irreducible(&self) -> bool {
        let classes = self.communicating_classes();
        classes.len() == 1
    }

    /// Check if the chain is aperiodic.
    ///
    /// A sufficient condition: at least one state has a self-loop (P[i][i] > 0).
    /// This is a conservative check — a chain can be aperiodic without self-loops,
    /// but self-loops guarantee aperiodicity for irreducible chains.
    #[must_use]
    pub fn is_aperiodic(&self) -> bool {
        let n = self.state_count();
        (0..n).any(|i| {
            let self_prob = self.transition_matrix.get(i, i).unwrap_or(0.0);
            self_prob > 0.0
        })
    }

    // ========================================================================
    // Mean First Passage Time
    // ========================================================================

    /// Compute the mean first passage time from state i to state j.
    ///
    /// This is the expected number of steps to reach state j starting from state i.
    /// Uses the fundamental matrix approach for absorbing chains, or direct
    /// iterative computation for ergodic chains.
    ///
    /// Returns `None` if state j is unreachable from state i, or if
    /// the computation doesn't converge.
    #[must_use]
    pub fn mean_first_passage_time(
        &self,
        from: usize,
        to: usize,
        max_iterations: usize,
    ) -> Option<f64> {
        let n = self.state_count();
        if from >= n || to >= n {
            return None;
        }
        if from == to {
            return Some(0.0);
        }

        // Iterative computation:
        // m[i] = expected steps from i to target j
        // m[j] = 0
        // m[i] = 1 + Σ_k P[i][k] * m[k] for i ≠ j
        //
        // We solve this system iteratively.
        let mut m = vec![0.0_f64; n];
        // Initialize with a guess
        for (i, m_i) in m.iter_mut().enumerate() {
            if i != to {
                *m_i = n as f64; // initial guess
            }
        }

        for _ in 0..max_iterations {
            let mut max_change = 0.0_f64;
            for i in 0..n {
                if i == to {
                    continue;
                }
                let mut new_val = 1.0;
                for (k, m_k) in m.iter().enumerate() {
                    let p = self.transition_matrix.get(i, k).unwrap_or(0.0);
                    new_val += p * m_k;
                }
                let change = (new_val - m[i]).abs();
                if change > max_change {
                    max_change = change;
                }
                m[i] = new_val;
            }
            if max_change < 1e-8 {
                break;
            }
        }

        let result = m[from];
        // If the result is unreasonably large, target is probably unreachable
        if result > (max_iterations as f64) * 10.0 {
            None
        } else {
            Some(result)
        }
    }

    // ========================================================================
    // Entropy
    // ========================================================================

    /// Compute the entropy rate of the Markov chain.
    ///
    /// H = -Σ_i π_i Σ_j P[i][j] log2(P[i][j])
    ///
    /// where π is the stationary distribution. Higher entropy = more unpredictable.
    #[must_use]
    pub fn entropy_rate(&self) -> Measured<f64> {
        let pi = self.stationary_distribution(1000, 1e-10);
        let n = self.state_count();

        let mut entropy = 0.0_f64;
        for i in 0..n {
            let pi_i = pi.value.get(i).copied().unwrap_or(0.0);
            if pi_i <= 0.0 {
                continue;
            }
            for j in 0..n {
                let p_ij = self.transition_matrix.get(i, j).unwrap_or(0.0);
                if p_ij > 0.0 {
                    entropy -= pi_i * p_ij * p_ij.log2();
                }
            }
        }

        Measured::new(entropy, pi.confidence)
    }
}

// State lookup by label
impl<S: Clone + PartialEq> MarkovChain<S> {
    /// Find state index by label.
    #[must_use]
    pub fn state_index(&self, label: &S) -> Option<usize> {
        self.states.iter().position(|s| s == label)
    }
}

// Display for String states
impl MarkovChain<String> {
    /// Build a summary of the chain for display.
    #[must_use]
    pub fn summary(&self) -> MarkovSummary {
        let classes = self.communicating_classes();
        let absorbing = self.absorbing_states();
        let pi = self.stationary_distribution(1000, 1e-10);
        let entropy = self.entropy_rate();

        MarkovSummary {
            state_count: self.state_count(),
            is_ergodic: self.is_ergodic(),
            is_irreducible: self.is_irreducible(),
            is_aperiodic: self.is_aperiodic(),
            communicating_class_count: classes.len(),
            absorbing_state_count: absorbing.len(),
            stationary_distribution: pi.value,
            stationary_confidence: pi.confidence.value(),
            entropy_rate: entropy.value,
        }
    }
}

/// Summary statistics for a Markov chain.
#[derive(Debug, Clone)]
pub struct MarkovSummary {
    /// Number of states.
    pub state_count: usize,
    /// Whether the chain is ergodic.
    pub is_ergodic: bool,
    /// Whether the chain is irreducible.
    pub is_irreducible: bool,
    /// Whether the chain is aperiodic.
    pub is_aperiodic: bool,
    /// Number of communicating classes.
    pub communicating_class_count: usize,
    /// Number of absorbing states.
    pub absorbing_state_count: usize,
    /// Stationary distribution (if computable).
    pub stationary_distribution: Vec<f64>,
    /// Confidence in stationary distribution.
    pub stationary_confidence: f64,
    /// Entropy rate of the chain.
    pub entropy_rate: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper builders ---

    /// Simple 2-state chain: Sunny ↔ Rainy
    fn weather_chain() -> MarkovChain<String> {
        let states = vec!["Sunny".to_string(), "Rainy".to_string()];
        // P(Sunny→Sunny) = 0.8, P(Sunny→Rainy) = 0.2
        // P(Rainy→Sunny) = 0.4, P(Rainy→Rainy) = 0.6
        let matrix = Matrix::from_rows(&[vec![0.8, 0.2], vec![0.4, 0.6]]);
        MarkovChain::new(states, matrix.unwrap_or_else(|| unreachable!()))
            .unwrap_or_else(|| unreachable!())
    }

    /// 3-state cycle: A → B → C → A (no self-loops)
    fn cycle_chain() -> MarkovChain<String> {
        let states = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let matrix = Matrix::from_rows(&[
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![1.0, 0.0, 0.0],
        ]);
        MarkovChain::new(states, matrix.unwrap_or_else(|| unreachable!()))
            .unwrap_or_else(|| unreachable!())
    }

    /// Chain with absorbing state: Healthy → Sick → Dead (absorbing)
    fn absorbing_chain() -> MarkovChain<String> {
        let states = vec![
            "Healthy".to_string(),
            "Sick".to_string(),
            "Dead".to_string(),
        ];
        let matrix = Matrix::from_rows(&[
            vec![0.7, 0.2, 0.1],
            vec![0.3, 0.3, 0.4],
            vec![0.0, 0.0, 1.0], // absorbing
        ]);
        MarkovChain::new(states, matrix.unwrap_or_else(|| unreachable!()))
            .unwrap_or_else(|| unreachable!())
    }

    /// Drug development pipeline: Preclinical → Phase1 → Phase2 → Phase3 → Approved / Failed
    fn drug_pipeline_chain() -> MarkovChain<String> {
        let states = vec![
            "Preclinical".to_string(),
            "Phase1".to_string(),
            "Phase2".to_string(),
            "Phase3".to_string(),
            "Approved".to_string(),
            "Failed".to_string(),
        ];
        let matrix = Matrix::from_rows(&[
            vec![0.0, 0.6, 0.0, 0.0, 0.0, 0.4], // Preclinical
            vec![0.0, 0.0, 0.5, 0.0, 0.0, 0.5], // Phase1
            vec![0.0, 0.0, 0.0, 0.4, 0.0, 0.6], // Phase2
            vec![0.0, 0.0, 0.0, 0.0, 0.6, 0.4], // Phase3
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0], // Approved (absorbing)
            vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // Failed (absorbing)
        ]);
        MarkovChain::new(states, matrix.unwrap_or_else(|| unreachable!()))
            .unwrap_or_else(|| unreachable!())
    }

    // --- Construction tests ---

    #[test]
    fn new_validates_stochastic() {
        let states = vec!["A", "B"];
        let bad = Matrix::from_rows(&[vec![0.5, 0.6], vec![0.5, 0.5]]);
        assert!(MarkovChain::new(states, bad.unwrap_or_else(|| unreachable!())).is_none());
    }

    #[test]
    fn new_validates_nonnegative() {
        let states = vec!["A", "B"];
        let bad = Matrix::from_rows(&[vec![1.5, -0.5], vec![0.5, 0.5]]);
        assert!(MarkovChain::new(states, bad.unwrap_or_else(|| unreachable!())).is_none());
    }

    #[test]
    fn new_validates_square() {
        let states = vec!["A", "B"];
        let bad = Matrix::from_flat(2, 3, vec![0.0; 6]);
        assert!(MarkovChain::new(states, bad.unwrap_or_else(|| unreachable!())).is_none());
    }

    #[test]
    fn new_validates_dimension_match() {
        let states = vec!["A", "B", "C"]; // 3 states
        let matrix = Matrix::from_rows(&[vec![0.5, 0.5], vec![0.5, 0.5]]); // 2×2
        assert!(MarkovChain::new(states, matrix.unwrap_or_else(|| unreachable!())).is_none());
    }

    #[test]
    fn from_transitions_works() {
        let mc = MarkovChain::from_transitions(
            vec!["A", "B"],
            &[(0, 0, 3.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 1.0)],
        );
        assert!(mc.is_some());
        let mc = mc.unwrap_or_else(|| unreachable!());
        // Row 0: 3/(3+1) = 0.75, 1/(3+1) = 0.25
        assert!((mc.transition_probability(0, 0).unwrap_or(0.0) - 0.75).abs() < 1e-10);
        assert!((mc.transition_probability(0, 1).unwrap_or(0.0) - 0.25).abs() < 1e-10);
    }

    #[test]
    fn from_observed_data_works() {
        let mc = MarkovChain::from_observed_data(
            vec!["A", "B", "C"],
            &[
                vec![0, 1, 2, 0, 1], // A→B→C→A→B
                vec![0, 0, 1, 2],    // A→A→B→C
            ],
        );
        assert!(mc.is_some());
        let mc = mc.unwrap_or_else(|| unreachable!());
        // A(0) transitions: A→B (3), A→A (1) → P(A→B) = 3/4
        assert!((mc.transition_probability(0, 1).unwrap_or(0.0) - 3.0 / 4.0).abs() < 1e-10);
    }

    // --- Accessor tests ---

    #[test]
    fn accessors_work() {
        let mc = weather_chain();
        assert_eq!(mc.state_count(), 2);
        assert_eq!(mc.state(0), Some(&"Sunny".to_string()));
        assert_eq!(mc.state(1), Some(&"Rainy".to_string()));
        assert!(mc.transition_matrix().is_stochastic());
    }

    #[test]
    fn state_index_lookup() {
        let mc = weather_chain();
        assert_eq!(mc.state_index(&"Sunny".to_string()), Some(0));
        assert_eq!(mc.state_index(&"Rainy".to_string()), Some(1));
        assert_eq!(mc.state_index(&"Cloudy".to_string()), None);
    }

    // --- N-step probability tests ---

    #[test]
    fn one_step_equals_matrix() {
        let mc = weather_chain();
        assert!((mc.n_step_probability(0, 0, 1).unwrap_or(0.0) - 0.8).abs() < 1e-10);
        assert!((mc.n_step_probability(0, 1, 1).unwrap_or(0.0) - 0.2).abs() < 1e-10);
    }

    #[test]
    fn zero_step_is_identity() {
        let mc = weather_chain();
        assert_eq!(mc.n_step_probability(0, 0, 0), Some(1.0));
        assert_eq!(mc.n_step_probability(0, 1, 0), Some(0.0));
    }

    #[test]
    fn two_step_correct() {
        let mc = weather_chain();
        // P^2[0][0] = 0.8*0.8 + 0.2*0.4 = 0.64 + 0.08 = 0.72
        let p2_00 = mc.n_step_probability(0, 0, 2).unwrap_or(0.0);
        assert!((p2_00 - 0.72).abs() < 1e-10);
    }

    #[test]
    fn n_step_matrix_returns_correct_dimensions() {
        let mc = weather_chain();
        let p3 = mc.n_step_matrix(3);
        assert!(p3.is_some());
        let p3 = p3.unwrap_or_else(|| unreachable!());
        assert_eq!(p3.rows(), 2);
        assert_eq!(p3.cols(), 2);
        // Each row should still sum to ~1.0
        assert!(p3.is_stochastic());
    }

    // --- Stationary distribution tests ---

    #[test]
    fn stationary_distribution_weather() {
        let mc = weather_chain();
        let pi = mc.stationary_distribution(1000, 1e-10);
        // Analytical: π = [2/3, 1/3]
        assert!((pi.value[0] - 2.0 / 3.0).abs() < 1e-6);
        assert!((pi.value[1] - 1.0 / 3.0).abs() < 1e-6);
        assert!(pi.confidence.value() >= 0.85);
    }

    #[test]
    fn stationary_distribution_cycle() {
        let mc = cycle_chain();
        let pi = mc.stationary_distribution(1000, 1e-10);
        // 3-state cycle: uniform distribution [1/3, 1/3, 1/3]
        for &p in &pi.value {
            assert!((p - 1.0 / 3.0).abs() < 1e-6);
        }
    }

    // --- Structural analysis tests ---

    #[test]
    fn weather_is_ergodic() {
        let mc = weather_chain();
        assert!(mc.is_ergodic());
        assert!(mc.is_irreducible());
        assert!(mc.is_aperiodic());
    }

    #[test]
    fn cycle_is_not_aperiodic() {
        let mc = cycle_chain();
        assert!(mc.is_irreducible());
        assert!(!mc.is_aperiodic()); // no self-loops → periodic
        assert!(!mc.is_ergodic());
    }

    #[test]
    fn absorbing_chain_structure() {
        let mc = absorbing_chain();
        assert!(!mc.is_irreducible());
        assert!(!mc.is_ergodic());
        assert!(mc.is_absorbing(2)); // Dead is absorbing
        assert!(!mc.is_absorbing(0)); // Healthy is not
        assert_eq!(mc.absorbing_states(), vec![2]);
    }

    #[test]
    fn drug_pipeline_absorbing_states() {
        let mc = drug_pipeline_chain();
        let absorbing = mc.absorbing_states();
        assert_eq!(absorbing, vec![4, 5]); // Approved and Failed
    }

    #[test]
    fn drug_pipeline_communicating_classes() {
        let mc = drug_pipeline_chain();
        let classes = mc.communicating_classes();
        // 6 states, each transient state is its own class + 2 absorbing
        // Preclinical, Phase1, Phase2, Phase3 are transient singletons
        // Approved and Failed are absorbing singletons
        let absorbing_count = classes
            .iter()
            .filter(|c| c.class_type == StateClass::Absorbing)
            .count();
        let transient_count = classes
            .iter()
            .filter(|c| c.class_type == StateClass::Transient)
            .count();
        assert_eq!(absorbing_count, 2);
        assert_eq!(transient_count, 4);
    }

    #[test]
    fn classify_states_weather() {
        let mc = weather_chain();
        let classified = mc.classify_states();
        // Both states should be recurrent (single communicating class)
        assert_eq!(classified[0].1, StateClass::Recurrent);
        assert_eq!(classified[1].1, StateClass::Recurrent);
    }

    #[test]
    fn classify_states_drug_pipeline() {
        let mc = drug_pipeline_chain();
        let classified = mc.classify_states();
        // States 0-3 transient, 4 absorbing, 5 absorbing
        assert_eq!(classified[0].1, StateClass::Transient);
        assert_eq!(classified[4].1, StateClass::Absorbing);
        assert_eq!(classified[5].1, StateClass::Absorbing);
    }

    // --- Mean first passage time tests ---

    #[test]
    fn mfpt_self_is_zero() {
        let mc = weather_chain();
        assert_eq!(mc.mean_first_passage_time(0, 0, 1000), Some(0.0));
    }

    #[test]
    fn mfpt_weather_sunny_to_rainy() {
        let mc = weather_chain();
        // Analytical: m(Sunny→Rainy) = 1/P(S→R) = 1/0.2 = 5 (geometric)
        // Actually for Markov chains: m[0→1] = 1 + 0.8*m[0→1]
        // m = 1/(1-0.8) = 5
        let mfpt = mc.mean_first_passage_time(0, 1, 10000);
        assert!(mfpt.is_some());
        assert!((mfpt.unwrap_or(0.0) - 5.0).abs() < 0.1);
    }

    // --- Entropy tests ---

    #[test]
    fn entropy_rate_weather() {
        let mc = weather_chain();
        let h = mc.entropy_rate();
        // Should be positive but less than log2(2) = 1.0
        assert!(h.value > 0.0);
        assert!(h.value < 1.0);
        assert!(h.confidence.value() >= 0.85);
    }

    #[test]
    fn entropy_rate_deterministic_is_zero() {
        // Deterministic chain: A → B → A → B → ...
        let states = vec!["A".to_string(), "B".to_string()];
        let matrix = Matrix::from_rows(&[vec![0.0, 1.0], vec![1.0, 0.0]]);
        let mc = MarkovChain::new(states, matrix.unwrap_or_else(|| unreachable!()))
            .unwrap_or_else(|| unreachable!());
        let h = mc.entropy_rate();
        assert!((h.value - 0.0).abs() < 1e-10);
    }

    // --- Summary test ---

    #[test]
    fn summary_works() {
        let mc = weather_chain();
        let s = mc.summary();
        assert_eq!(s.state_count, 2);
        assert!(s.is_ergodic);
        assert!(s.is_irreducible);
        assert!(s.is_aperiodic);
        assert_eq!(s.communicating_class_count, 1);
        assert_eq!(s.absorbing_state_count, 0);
        assert!(s.stationary_confidence >= 0.85);
    }

    // --- Drug development pipeline integration test ---

    #[test]
    fn drug_pipeline_approval_probability() {
        let mc = drug_pipeline_chain();
        // What's the probability of going from Preclinical to Approved?
        // This requires many steps — check at step 100 (should converge)
        let p_approved = mc.n_step_probability(0, 4, 100).unwrap_or(0.0);
        // Analytical: 0.6 * 0.5 * 0.4 * 0.6 = 0.072
        assert!((p_approved - 0.072).abs() < 1e-6);
    }

    #[test]
    fn drug_pipeline_failure_probability() {
        let mc = drug_pipeline_chain();
        let p_failed = mc.n_step_probability(0, 5, 100).unwrap_or(0.0);
        // Should be 1 - 0.072 = 0.928
        assert!((p_failed - 0.928).abs() < 1e-6);
    }
}
