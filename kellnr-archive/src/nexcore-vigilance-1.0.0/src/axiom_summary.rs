//! # §7 Axiom Summary
//!
//! Theory of Vigilance axiom summary, dependency graph, and completeness verification.
//!
//! ## Overview
//!
//! The five axioms of the Theory of Vigilance form a complete foundation from which
//! the mathematical framework can be derived:
//!
//! | Axiom | Name | Core Assertion |
//! |-------|------|----------------|
//! | A1 | System Decomposition | Every vigilance system admits finite elemental decomposition (with measurable Φ) |
//! | A2 | Hierarchical Organization | S ≅ S₁ with quotient spaces Sᵢ₊₁ ≅ Sᵢ/~ᵢ and emergent properties |
//! | A3 | Conservation Constraints | Harm occurs iff conservation law constraints are violated |
//! | A4 | Safety Manifold | Safe states form stratified space; harm is boundary crossing |
//! | A5 | Emergence | Harm probability factors as product under Markov assumption |
//!
//! ## Dependency Structure
//!
//! ```text
//!     A1 (Decomposition)
//!      │
//!      ├────────────┬────────────┐
//!      ▼            ▼            │
//!     A2          A3            │
//!  (Hierarchy) (Conservation)   │
//!      │            │            │
//!      │            ▼            │
//!      │           A4 ◄──────────┘
//!      │        (Manifold)
//!      │            │
//!      ▼            ▼
//!     A5 ◄─────────┘
//!  (Emergence)
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM ENUMERATION
// ═══════════════════════════════════════════════════════════════════════════

/// The five axioms of the Theory of Vigilance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Axiom {
    /// A1: System Decomposition - Every vigilance system admits finite elemental decomposition
    A1Decomposition = 1,
    /// A2: Hierarchical Organization - S ≅ S₁ with quotient spaces and emergent properties
    A2Hierarchy = 2,
    /// A3: Conservation Constraints - Harm occurs iff conservation laws are violated
    A3Conservation = 3,
    /// A4: Safety Manifold - Safe states form stratified space; harm is boundary crossing
    A4Manifold = 4,
    /// A5: Emergence - Harm probability factors as product under Markov assumption
    A5Emergence = 5,
}

impl Axiom {
    /// Get all axioms in order.
    #[must_use]
    pub const fn all() -> [Axiom; 5] {
        [
            Axiom::A1Decomposition,
            Axiom::A2Hierarchy,
            Axiom::A3Conservation,
            Axiom::A4Manifold,
            Axiom::A5Emergence,
        ]
    }

    /// Get the axiom name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Axiom::A1Decomposition => "System Decomposition",
            Axiom::A2Hierarchy => "Hierarchical Organization",
            Axiom::A3Conservation => "Conservation Constraints",
            Axiom::A4Manifold => "Safety Manifold",
            Axiom::A5Emergence => "Emergence",
        }
    }

    /// Get the axiom's core assertion.
    #[must_use]
    pub const fn core_assertion(&self) -> &'static str {
        match self {
            Axiom::A1Decomposition => {
                "Every vigilance system admits finite elemental decomposition (with measurable Φ)"
            }
            Axiom::A2Hierarchy => {
                "S ≅ S₁ with quotient spaces Sᵢ₊₁ ≅ Sᵢ/~ᵢ and emergent properties"
            }
            Axiom::A3Conservation => "Harm occurs iff conservation law constraints are violated",
            Axiom::A4Manifold => "Safe states form stratified space; harm is boundary crossing",
            Axiom::A5Emergence => "Harm probability factors as product under Markov assumption",
        }
    }

    /// Get the ToV section reference.
    #[must_use]
    pub const fn section(&self) -> &'static str {
        match self {
            Axiom::A1Decomposition => "§2",
            Axiom::A2Hierarchy => "§3",
            Axiom::A3Conservation => "§4",
            Axiom::A4Manifold => "§5",
            Axiom::A5Emergence => "§6",
        }
    }

    /// Get axiom number (1-5).
    #[must_use]
    pub const fn number(&self) -> u8 {
        *self as u8
    }

    /// Get the axioms this axiom depends on.
    #[must_use]
    pub fn dependencies(&self) -> Vec<Axiom> {
        match self {
            // A1 is foundational - no dependencies
            Axiom::A1Decomposition => vec![],
            // A2 depends on A1 for element set over which hierarchy is defined
            Axiom::A2Hierarchy => vec![Axiom::A1Decomposition],
            // A3 is independent but interacts with A1
            Axiom::A3Conservation => vec![],
            // A4 depends on A1 (elements) and A3 (constraint functions)
            Axiom::A4Manifold => vec![Axiom::A1Decomposition, Axiom::A3Conservation],
            // A5 depends on A2 (hierarchy) and A4 (boundary crossing = harm)
            Axiom::A5Emergence => vec![Axiom::A2Hierarchy, Axiom::A4Manifold],
        }
    }

    /// Get axioms that depend on this axiom.
    #[must_use]
    pub fn dependents(&self) -> Vec<Axiom> {
        match self {
            // A1 is depended on by A2 and A4
            Axiom::A1Decomposition => vec![Axiom::A2Hierarchy, Axiom::A4Manifold],
            // A2 is depended on by A5
            Axiom::A2Hierarchy => vec![Axiom::A5Emergence],
            // A3 is depended on by A4
            Axiom::A3Conservation => vec![Axiom::A4Manifold],
            // A4 is depended on by A5
            Axiom::A4Manifold => vec![Axiom::A5Emergence],
            // A5 has no dependents - it's the final axiom
            Axiom::A5Emergence => vec![],
        }
    }

    /// Check if this axiom is foundational (no dependencies).
    #[must_use]
    pub fn is_foundational(&self) -> bool {
        self.dependencies().is_empty()
    }

    /// Check if this axiom is terminal (no dependents).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.dependents().is_empty()
    }
}

impl std::fmt::Display for Axiom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A{} ({})", self.number(), self.name())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM INFO
// ═══════════════════════════════════════════════════════════════════════════

/// Complete information about an axiom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomInfo {
    /// The axiom identifier.
    pub axiom: Axiom,
    /// The axiom name.
    pub name: String,
    /// The core assertion.
    pub core_assertion: String,
    /// ToV section reference.
    pub section: String,
    /// Axioms this depends on.
    pub dependencies: Vec<Axiom>,
    /// Axioms that depend on this.
    pub dependents: Vec<Axiom>,
    /// Key definitions introduced.
    pub definitions: Vec<String>,
    /// Key theorems supported.
    pub theorems: Vec<String>,
}

impl AxiomInfo {
    /// Create axiom info from an axiom.
    #[must_use]
    pub fn from_axiom(axiom: Axiom) -> Self {
        let (definitions, theorems) = match axiom {
            Axiom::A1Decomposition => (
                vec![
                    "Element (e ∈ E)".to_string(),
                    "Element Set (E ⊂ S)".to_string(),
                    "Composition Function (Φ)".to_string(),
                    "Accessible State Space".to_string(),
                    "Interaction Graph".to_string(),
                ],
                vec!["Finite Decomposition".to_string()],
            ),
            Axiom::A2Hierarchy => (
                vec![
                    "Level (ℓ ∈ {1,...,L})".to_string(),
                    "Coarse-Graining Map (πℓ)".to_string(),
                    "Emergent Property".to_string(),
                    "Scale Separation".to_string(),
                ],
                vec![
                    "Quotient Space Structure".to_string(),
                    "Emergent Property Detection".to_string(),
                ],
            ),
            Axiom::A3Conservation => (
                vec![
                    "Conservation Law (gᵢ)".to_string(),
                    "Constraint Set (G)".to_string(),
                    "Feasible Region".to_string(),
                    "Violation Magnitude".to_string(),
                ],
                vec![
                    "Conservation-Harm Equivalence".to_string(),
                    "11 Conservation Laws".to_string(),
                ],
            ),
            Axiom::A4Manifold => (
                vec![
                    "Safety Manifold (M)".to_string(),
                    "Harm Boundary (∂M)".to_string(),
                    "Signed Distance (d)".to_string(),
                    "Safety Margin (Ω)".to_string(),
                    "First Passage Time".to_string(),
                ],
                vec![
                    "Stratified Manifold Structure".to_string(),
                    "Boundary Regularity".to_string(),
                ],
            ),
            Axiom::A5Emergence => (
                vec![
                    "Level Perturbation (δsᵢ)".to_string(),
                    "Buffering Capacity (bᵢ)".to_string(),
                    "Propagation Function (Pᵢ→ᵢ₊₁)".to_string(),
                    "Harm Level (ℓ_H)".to_string(),
                ],
                vec![
                    "Markov Product Formula".to_string(),
                    "Attenuation Theorem".to_string(),
                    "Non-Markovian Extension".to_string(),
                ],
            ),
        };

        Self {
            axiom,
            name: axiom.name().to_string(),
            core_assertion: axiom.core_assertion().to_string(),
            section: axiom.section().to_string(),
            dependencies: axiom.dependencies(),
            dependents: axiom.dependents(),
            definitions,
            theorems,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM DEPENDENCY GRAPH
// ═══════════════════════════════════════════════════════════════════════════

/// Directed Acyclic Graph of axiom dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomDependencyGraph {
    /// Edges: (from, to) where 'from' is depended on by 'to'.
    edges: Vec<(Axiom, Axiom)>,
}

impl Default for AxiomDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl AxiomDependencyGraph {
    /// Create the canonical ToV axiom dependency graph.
    #[must_use]
    pub fn new() -> Self {
        // A1 → A2, A1 → A4
        // A3 → A4
        // A2 → A5, A4 → A5
        let edges = vec![
            (Axiom::A1Decomposition, Axiom::A2Hierarchy),
            (Axiom::A1Decomposition, Axiom::A4Manifold),
            (Axiom::A3Conservation, Axiom::A4Manifold),
            (Axiom::A2Hierarchy, Axiom::A5Emergence),
            (Axiom::A4Manifold, Axiom::A5Emergence),
        ];
        Self { edges }
    }

    /// Get all edges in the graph.
    #[must_use]
    pub fn edges(&self) -> &[(Axiom, Axiom)] {
        &self.edges
    }

    /// Get direct dependencies of an axiom.
    #[must_use]
    pub fn direct_dependencies(&self, axiom: Axiom) -> Vec<Axiom> {
        self.edges
            .iter()
            .filter(|(_, to)| *to == axiom)
            .map(|(from, _)| *from)
            .collect()
    }

    /// Get direct dependents of an axiom.
    #[must_use]
    pub fn direct_dependents(&self, axiom: Axiom) -> Vec<Axiom> {
        self.edges
            .iter()
            .filter(|(from, _)| *from == axiom)
            .map(|(_, to)| *to)
            .collect()
    }

    /// Get transitive closure of dependencies (all axioms this depends on).
    #[must_use]
    pub fn transitive_dependencies(&self, axiom: Axiom) -> HashSet<Axiom> {
        let mut visited = HashSet::new();
        let mut stack = vec![axiom];

        while let Some(current) = stack.pop() {
            for dep in self.direct_dependencies(current) {
                if visited.insert(dep) {
                    stack.push(dep);
                }
            }
        }

        visited
    }

    /// Get transitive closure of dependents (all axioms that depend on this).
    #[must_use]
    pub fn transitive_dependents(&self, axiom: Axiom) -> HashSet<Axiom> {
        let mut visited = HashSet::new();
        let mut stack = vec![axiom];

        while let Some(current) = stack.pop() {
            for dep in self.direct_dependents(current) {
                if visited.insert(dep) {
                    stack.push(dep);
                }
            }
        }

        visited
    }

    /// Topological sort of axioms (dependencies first).
    #[must_use]
    pub fn topological_order(&self) -> Vec<Axiom> {
        // Kahn's algorithm
        let mut in_degree: HashMap<Axiom, usize> = HashMap::new();
        for axiom in Axiom::all() {
            in_degree.insert(axiom, 0);
        }

        for (_, to) in &self.edges {
            if let Some(deg) = in_degree.get_mut(to) {
                *deg += 1;
            }
        }

        let mut queue: Vec<Axiom> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&ax, _)| ax)
            .collect();
        queue.sort(); // Deterministic ordering

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node);
            for dependent in self.direct_dependents(node) {
                if let Some(deg) = in_degree.get_mut(&dependent) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(dependent);
                        queue.sort();
                    }
                }
            }
        }

        result
    }

    /// Verify the graph is a valid DAG (no cycles).
    #[must_use]
    pub fn is_acyclic(&self) -> bool {
        self.topological_order().len() == 5
    }

    /// Get the depth of each axiom (longest path from a root).
    #[must_use]
    pub fn depths(&self) -> HashMap<Axiom, usize> {
        let topo = self.topological_order();
        let mut depths: HashMap<Axiom, usize> = HashMap::new();

        for axiom in &topo {
            let max_dep_depth = self
                .direct_dependencies(*axiom)
                .iter()
                .map(|dep| depths.get(dep).copied().unwrap_or(0))
                .max()
                .unwrap_or(0);

            let depth = if self.direct_dependencies(*axiom).is_empty() {
                0
            } else {
                max_dep_depth + 1
            };

            depths.insert(*axiom, depth);
        }

        depths
    }

    /// Get axioms at each depth level.
    #[must_use]
    pub fn levels(&self) -> Vec<Vec<Axiom>> {
        let depths = self.depths();
        let max_depth = depths.values().max().copied().unwrap_or(0);

        let mut levels: Vec<Vec<Axiom>> = vec![Vec::new(); max_depth + 1];

        for (axiom, depth) in depths {
            levels[depth].push(axiom);
        }

        for level in &mut levels {
            level.sort();
        }

        levels
    }

    /// Render as ASCII DAG.
    #[must_use]
    pub fn render_ascii(&self) -> String {
        r"    A1 (Decomposition)
     │
     ├────────────┬────────────┐
     ▼            ▼            │
    A2          A3            │
 (Hierarchy) (Conservation)   │
     │            │            │
     │            ▼            │
     │           A4 ◄──────────┘
     │        (Manifold)
     │            │
     ▼            ▼
    A5 ◄─────────┘
 (Emergence)"
            .to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPLETENESS VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Status of an individual axiom verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxiomStatus {
    /// Not yet verified.
    Unverified,
    /// Verified with all conditions satisfied.
    Verified,
    /// Partially verified (some conditions failed).
    Partial,
    /// Verification failed.
    Failed,
}

impl AxiomStatus {
    /// Check if fully verified.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        matches!(self, AxiomStatus::Verified)
    }
}

/// Result of completeness verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessResult {
    /// Status of each axiom.
    pub axiom_status: HashMap<Axiom, AxiomStatus>,
    /// Whether all axioms are verified.
    pub all_verified: bool,
    /// Axioms that are not verified.
    pub unverified: Vec<Axiom>,
    /// Axioms that are blocked (dependencies not satisfied).
    pub blocked: Vec<Axiom>,
    /// Principal theorems derivable from verified axioms.
    pub derivable_theorems: Vec<PrincipalTheorem>,
}

impl CompletenessResult {
    /// Get verification percentage.
    #[must_use]
    pub fn verification_percentage(&self) -> f64 {
        let verified = self
            .axiom_status
            .values()
            .filter(|s| s.is_verified())
            .count();
        (verified as f64 / 5.0) * 100.0
    }
}

/// Principal theorems of the Theory of Vigilance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrincipalTheorem {
    /// Given sufficient knowledge, harm probability can be computed from axioms.
    Predictability,
    /// Product structure implies exponential attenuation with hierarchical depth.
    Attenuation,
    /// Modifying constraints/buffering yields quantifiable changes in harm probability.
    Intervention,
}

impl PrincipalTheorem {
    /// Get theorem name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            PrincipalTheorem::Predictability => "Predictability Theorem",
            PrincipalTheorem::Attenuation => "Attenuation Theorem",
            PrincipalTheorem::Intervention => "Intervention Theorem",
        }
    }

    /// Get theorem statement.
    #[must_use]
    pub const fn statement(&self) -> &'static str {
        match self {
            PrincipalTheorem::Predictability => {
                "Given sufficient knowledge of system state, parameters, and perturbations, \
                 harm probability can be computed from the axioms"
            }
            PrincipalTheorem::Attenuation => {
                "The product structure of propagation probabilities implies exponential \
                 attenuation of harm probability with hierarchical depth"
            }
            PrincipalTheorem::Intervention => {
                "Modifying constraint functions, buffering capacities, or perturbation \
                 magnitudes yields quantifiable changes in harm probability"
            }
        }
    }

    /// Get required axioms for this theorem.
    #[must_use]
    pub fn required_axioms(&self) -> Vec<Axiom> {
        match self {
            PrincipalTheorem::Predictability => Axiom::all().to_vec(),
            PrincipalTheorem::Attenuation => vec![Axiom::A2Hierarchy, Axiom::A5Emergence],
            PrincipalTheorem::Intervention => {
                vec![Axiom::A3Conservation, Axiom::A4Manifold, Axiom::A5Emergence]
            }
        }
    }

    /// Check if theorem is derivable from verified axioms.
    #[must_use]
    pub fn is_derivable(&self, verified: &HashSet<Axiom>) -> bool {
        self.required_axioms().iter().all(|a| verified.contains(a))
    }
}

/// Framework completeness verifier.
#[derive(Debug, Clone)]
pub struct CompletenessVerifier {
    /// Current status of each axiom.
    status: HashMap<Axiom, AxiomStatus>,
    /// The dependency graph.
    graph: AxiomDependencyGraph,
}

impl Default for CompletenessVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletenessVerifier {
    /// Create a new completeness verifier.
    #[must_use]
    pub fn new() -> Self {
        let mut status = HashMap::new();
        for axiom in Axiom::all() {
            status.insert(axiom, AxiomStatus::Unverified);
        }

        Self {
            status,
            graph: AxiomDependencyGraph::new(),
        }
    }

    /// Mark an axiom as verified.
    pub fn mark_verified(&mut self, axiom: Axiom) {
        self.status.insert(axiom, AxiomStatus::Verified);
    }

    /// Mark an axiom as failed.
    pub fn mark_failed(&mut self, axiom: Axiom) {
        self.status.insert(axiom, AxiomStatus::Failed);
    }

    /// Mark an axiom as partial.
    pub fn mark_partial(&mut self, axiom: Axiom) {
        self.status.insert(axiom, AxiomStatus::Partial);
    }

    /// Get status of an axiom.
    #[must_use]
    pub fn get_status(&self, axiom: Axiom) -> AxiomStatus {
        self.status
            .get(&axiom)
            .copied()
            .unwrap_or(AxiomStatus::Unverified)
    }

    /// Check if an axiom's dependencies are all verified.
    #[must_use]
    pub fn dependencies_verified(&self, axiom: Axiom) -> bool {
        self.graph
            .direct_dependencies(axiom)
            .iter()
            .all(|dep| self.get_status(*dep).is_verified())
    }

    /// Get axioms that are blocked (dependencies not verified).
    #[must_use]
    pub fn blocked_axioms(&self) -> Vec<Axiom> {
        Axiom::all()
            .into_iter()
            .filter(|a| !self.get_status(*a).is_verified() && !self.dependencies_verified(*a))
            .collect()
    }

    /// Verify completeness and return result.
    #[must_use]
    pub fn verify(&self) -> CompletenessResult {
        let verified_set: HashSet<Axiom> = self
            .status
            .iter()
            .filter(|(_, s)| s.is_verified())
            .map(|(a, _)| *a)
            .collect();

        let unverified: Vec<Axiom> = Axiom::all()
            .into_iter()
            .filter(|a| !verified_set.contains(a))
            .collect();

        let blocked = self.blocked_axioms();

        let all_verified = unverified.is_empty();

        let derivable_theorems: Vec<PrincipalTheorem> = [
            PrincipalTheorem::Predictability,
            PrincipalTheorem::Attenuation,
            PrincipalTheorem::Intervention,
        ]
        .into_iter()
        .filter(|t| t.is_derivable(&verified_set))
        .collect();

        CompletenessResult {
            axiom_status: self.status.clone(),
            all_verified,
            unverified,
            blocked,
            derivable_theorems,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONSISTENCY VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Domain for which consistency is demonstrated via model existence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyDomain {
    /// Pharmacovigilance domain.
    Pharmacovigilance,
    /// Cloud/distributed systems domain.
    CloudSystems,
    /// AI/ML systems domain.
    AISystems,
}

impl ConsistencyDomain {
    /// Get domain name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            ConsistencyDomain::Pharmacovigilance => "Pharmacovigilance",
            ConsistencyDomain::CloudSystems => "Cloud Systems",
            ConsistencyDomain::AISystems => "AI Systems",
        }
    }

    /// Get example instantiations for this domain.
    #[must_use]
    pub fn example_instantiations(&self) -> Vec<&'static str> {
        match self {
            ConsistencyDomain::Pharmacovigilance => vec![
                "A1: Drug molecules, metabolites, enzymes as elements",
                "A2: Molecular → Cellular → Tissue → Organ → System → Organism",
                "A3: Mass balance, Michaelis-Menten saturation, thermodynamic directionality",
                "A4: Therapeutic window as safety manifold",
                "A5: Adverse event propagation through biological hierarchy",
            ],
            ConsistencyDomain::CloudSystems => vec![
                "A1: Microservices, containers, nodes as elements",
                "A2: Process → Container → Pod → Node → Cluster → Region",
                "A3: Resource conservation, request rate limits, consistency guarantees",
                "A4: SLO compliance region as safety manifold",
                "A5: Failure cascades through service dependency graph",
            ],
            ConsistencyDomain::AISystems => vec![
                "A1: Neurons, layers, attention heads as elements",
                "A2: Weight → Neuron → Layer → Block → Model → System",
                "A3: Gradient flow conservation, capacity limits, loss function descent",
                "A4: Alignment boundary as safety manifold",
                "A5: Error propagation through model architecture",
            ],
        }
    }
}

/// Result of consistency verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyResult {
    /// Whether the axioms are consistent.
    pub is_consistent: bool,
    /// Domains where models exist.
    pub model_domains: Vec<ConsistencyDomain>,
    /// Explanation of consistency proof method.
    pub proof_method: String,
}

/// Consistency verifier using model existence.
#[derive(Debug, Clone)]
pub struct ConsistencyVerifier {
    /// Domains with verified models.
    verified_domains: Vec<ConsistencyDomain>,
}

impl Default for ConsistencyVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsistencyVerifier {
    /// Create a new consistency verifier with all three domains.
    #[must_use]
    pub fn new() -> Self {
        // All three domains are pre-verified as part of ToV development
        Self {
            verified_domains: vec![
                ConsistencyDomain::Pharmacovigilance,
                ConsistencyDomain::CloudSystems,
                ConsistencyDomain::AISystems,
            ],
        }
    }

    /// Verify consistency.
    #[must_use]
    pub fn verify(&self) -> ConsistencyResult {
        let is_consistent = !self.verified_domains.is_empty();

        ConsistencyResult {
            is_consistent,
            model_domains: self.verified_domains.clone(),
            proof_method: "Model existence: Concrete instantiations (pharmacovigilance, \
                           cloud systems, AI systems) simultaneously satisfy all five axioms. \
                           The existence of models satisfying all axioms demonstrates \
                           consistency by the standard metamathematical argument."
                .to_string(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNIFIED AXIOM VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Unified verification result for the entire axiom system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axiom7Verification {
    /// Completeness result.
    pub completeness: CompletenessResult,
    /// Consistency result.
    pub consistency: ConsistencyResult,
    /// The dependency graph structure.
    pub dependency_structure: String,
    /// Information about each axiom.
    pub axiom_info: Vec<AxiomInfo>,
    /// Overall status.
    pub status: VerificationStatus,
}

/// Overall verification status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// All axioms verified, system is complete and consistent.
    Complete,
    /// Axioms are consistent but not all verified.
    Partial,
    /// Verification failed.
    Failed,
}

/// Perform complete §7 verification.
#[must_use]
pub fn verify_axiom_system(completeness_verifier: &CompletenessVerifier) -> Axiom7Verification {
    let completeness = completeness_verifier.verify();
    let consistency = ConsistencyVerifier::new().verify();
    let graph = AxiomDependencyGraph::new();

    let axiom_info: Vec<AxiomInfo> = Axiom::all()
        .into_iter()
        .map(AxiomInfo::from_axiom)
        .collect();

    let status = if completeness.all_verified && consistency.is_consistent {
        VerificationStatus::Complete
    } else if consistency.is_consistent {
        VerificationStatus::Partial
    } else {
        VerificationStatus::Failed
    };

    Axiom7Verification {
        completeness,
        consistency,
        dependency_structure: graph.render_ascii(),
        axiom_info,
        status,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axiom_enumeration() {
        let axioms = Axiom::all();
        assert_eq!(axioms.len(), 5);
        assert_eq!(axioms[0], Axiom::A1Decomposition);
        assert_eq!(axioms[4], Axiom::A5Emergence);
    }

    #[test]
    fn test_axiom_names() {
        assert_eq!(Axiom::A1Decomposition.name(), "System Decomposition");
        assert_eq!(Axiom::A2Hierarchy.name(), "Hierarchical Organization");
        assert_eq!(Axiom::A3Conservation.name(), "Conservation Constraints");
        assert_eq!(Axiom::A4Manifold.name(), "Safety Manifold");
        assert_eq!(Axiom::A5Emergence.name(), "Emergence");
    }

    #[test]
    fn test_axiom_sections() {
        assert_eq!(Axiom::A1Decomposition.section(), "§2");
        assert_eq!(Axiom::A5Emergence.section(), "§6");
    }

    #[test]
    fn test_axiom_dependencies() {
        // A1 is foundational
        assert!(Axiom::A1Decomposition.is_foundational());
        assert!(Axiom::A1Decomposition.dependencies().is_empty());

        // A2 depends on A1
        assert_eq!(
            Axiom::A2Hierarchy.dependencies(),
            vec![Axiom::A1Decomposition]
        );

        // A3 is independent
        assert!(Axiom::A3Conservation.is_foundational());

        // A4 depends on A1 and A3
        let a4_deps = Axiom::A4Manifold.dependencies();
        assert!(a4_deps.contains(&Axiom::A1Decomposition));
        assert!(a4_deps.contains(&Axiom::A3Conservation));

        // A5 depends on A2 and A4
        let a5_deps = Axiom::A5Emergence.dependencies();
        assert!(a5_deps.contains(&Axiom::A2Hierarchy));
        assert!(a5_deps.contains(&Axiom::A4Manifold));

        // A5 is terminal
        assert!(Axiom::A5Emergence.is_terminal());
    }

    #[test]
    fn test_dependency_graph_is_acyclic() {
        let graph = AxiomDependencyGraph::new();
        assert!(graph.is_acyclic());
    }

    #[test]
    fn test_dependency_graph_edges() {
        let graph = AxiomDependencyGraph::new();
        assert_eq!(graph.edges().len(), 5);
    }

    #[test]
    fn test_topological_order() {
        let graph = AxiomDependencyGraph::new();
        let order = graph.topological_order();

        assert_eq!(order.len(), 5);

        // A1 and A3 should come before their dependents
        let a1_pos = order
            .iter()
            .position(|a| *a == Axiom::A1Decomposition)
            .unwrap();
        let a2_pos = order.iter().position(|a| *a == Axiom::A2Hierarchy).unwrap();
        let a3_pos = order
            .iter()
            .position(|a| *a == Axiom::A3Conservation)
            .unwrap();
        let a4_pos = order.iter().position(|a| *a == Axiom::A4Manifold).unwrap();
        let a5_pos = order.iter().position(|a| *a == Axiom::A5Emergence).unwrap();

        assert!(a1_pos < a2_pos); // A1 before A2
        assert!(a1_pos < a4_pos); // A1 before A4
        assert!(a3_pos < a4_pos); // A3 before A4
        assert!(a2_pos < a5_pos); // A2 before A5
        assert!(a4_pos < a5_pos); // A4 before A5
    }

    #[test]
    fn test_transitive_dependencies() {
        let graph = AxiomDependencyGraph::new();

        // A5 transitively depends on A1, A2, A3, A4
        let a5_deps = graph.transitive_dependencies(Axiom::A5Emergence);
        assert_eq!(a5_deps.len(), 4);
        assert!(a5_deps.contains(&Axiom::A1Decomposition));
        assert!(a5_deps.contains(&Axiom::A2Hierarchy));
        assert!(a5_deps.contains(&Axiom::A3Conservation));
        assert!(a5_deps.contains(&Axiom::A4Manifold));
    }

    #[test]
    fn test_transitive_dependents() {
        let graph = AxiomDependencyGraph::new();

        // A1 transitively has A2, A4, A5 as dependents
        let a1_deps = graph.transitive_dependents(Axiom::A1Decomposition);
        assert_eq!(a1_deps.len(), 3);
        assert!(a1_deps.contains(&Axiom::A2Hierarchy));
        assert!(a1_deps.contains(&Axiom::A4Manifold));
        assert!(a1_deps.contains(&Axiom::A5Emergence));
    }

    #[test]
    fn test_graph_depths() {
        let graph = AxiomDependencyGraph::new();
        let depths = graph.depths();

        assert_eq!(depths[&Axiom::A1Decomposition], 0);
        assert_eq!(depths[&Axiom::A3Conservation], 0);
        assert_eq!(depths[&Axiom::A2Hierarchy], 1);
        assert_eq!(depths[&Axiom::A4Manifold], 1);
        assert_eq!(depths[&Axiom::A5Emergence], 2);
    }

    #[test]
    fn test_graph_levels() {
        let graph = AxiomDependencyGraph::new();
        let levels = graph.levels();

        assert_eq!(levels.len(), 3);
        assert!(levels[0].contains(&Axiom::A1Decomposition));
        assert!(levels[0].contains(&Axiom::A3Conservation));
        assert!(levels[1].contains(&Axiom::A2Hierarchy));
        assert!(levels[1].contains(&Axiom::A4Manifold));
        assert!(levels[2].contains(&Axiom::A5Emergence));
    }

    #[test]
    fn test_completeness_verifier() {
        let mut verifier = CompletenessVerifier::new();

        // Initially nothing verified
        let result = verifier.verify();
        assert!(!result.all_verified);
        assert_eq!(result.unverified.len(), 5);

        // Verify A1 and A3 (foundational)
        verifier.mark_verified(Axiom::A1Decomposition);
        verifier.mark_verified(Axiom::A3Conservation);

        let result = verifier.verify();
        assert!(!result.all_verified);
        assert_eq!(result.unverified.len(), 3);

        // Now A2 and A4 should not be blocked
        assert!(verifier.dependencies_verified(Axiom::A2Hierarchy));
        assert!(verifier.dependencies_verified(Axiom::A4Manifold));

        // Verify all
        verifier.mark_verified(Axiom::A2Hierarchy);
        verifier.mark_verified(Axiom::A4Manifold);
        verifier.mark_verified(Axiom::A5Emergence);

        let result = verifier.verify();
        assert!(result.all_verified);
        assert_eq!(result.verification_percentage(), 100.0);
    }

    #[test]
    fn test_principal_theorems() {
        let all_verified: HashSet<Axiom> = Axiom::all().into_iter().collect();

        // Predictability requires all axioms
        assert!(PrincipalTheorem::Predictability.is_derivable(&all_verified));

        // Attenuation requires A2 and A5
        let partial: HashSet<Axiom> = [Axiom::A2Hierarchy, Axiom::A5Emergence]
            .into_iter()
            .collect();
        assert!(PrincipalTheorem::Attenuation.is_derivable(&partial));
        assert!(!PrincipalTheorem::Predictability.is_derivable(&partial));
    }

    #[test]
    fn test_consistency_verification() {
        let verifier = ConsistencyVerifier::new();
        let result = verifier.verify();

        assert!(result.is_consistent);
        assert_eq!(result.model_domains.len(), 3);
    }

    #[test]
    fn test_consistency_domains() {
        assert_eq!(
            ConsistencyDomain::Pharmacovigilance.name(),
            "Pharmacovigilance"
        );

        let pv_examples = ConsistencyDomain::Pharmacovigilance.example_instantiations();
        assert_eq!(pv_examples.len(), 5); // One per axiom
    }

    #[test]
    fn test_full_axiom_verification() {
        let mut completeness = CompletenessVerifier::new();
        for axiom in Axiom::all() {
            completeness.mark_verified(axiom);
        }

        let result = verify_axiom_system(&completeness);

        assert_eq!(result.status, VerificationStatus::Complete);
        assert!(result.completeness.all_verified);
        assert!(result.consistency.is_consistent);
        assert_eq!(result.axiom_info.len(), 5);
    }

    #[test]
    fn test_partial_verification() {
        let mut completeness = CompletenessVerifier::new();
        completeness.mark_verified(Axiom::A1Decomposition);
        completeness.mark_verified(Axiom::A3Conservation);

        let result = verify_axiom_system(&completeness);

        assert_eq!(result.status, VerificationStatus::Partial);
        assert!(!result.completeness.all_verified);
        assert!(result.consistency.is_consistent);
    }

    #[test]
    fn test_axiom_info() {
        let info = AxiomInfo::from_axiom(Axiom::A1Decomposition);

        assert_eq!(info.name, "System Decomposition");
        assert!(!info.definitions.is_empty());
        assert!(!info.theorems.is_empty());
        assert!(info.dependencies.is_empty());
    }

    #[test]
    fn test_axiom_display() {
        assert_eq!(
            format!("{}", Axiom::A1Decomposition),
            "A1 (System Decomposition)"
        );
        assert_eq!(format!("{}", Axiom::A5Emergence), "A5 (Emergence)");
    }

    #[test]
    fn test_derivable_theorems_with_complete_verification() {
        let mut verifier = CompletenessVerifier::new();
        for axiom in Axiom::all() {
            verifier.mark_verified(axiom);
        }

        let result = verifier.verify();

        assert_eq!(result.derivable_theorems.len(), 3);
        assert!(
            result
                .derivable_theorems
                .contains(&PrincipalTheorem::Predictability)
        );
        assert!(
            result
                .derivable_theorems
                .contains(&PrincipalTheorem::Attenuation)
        );
        assert!(
            result
                .derivable_theorems
                .contains(&PrincipalTheorem::Intervention)
        );
    }
}
