//! ToV-specific domain extraction.
//!
//! Extracts axioms, harm types, conservation laws, theorems, and the
//! axiom dependency DAG from `nexcore-vigilance` source code.
//!
//! Rather than parsing the source dynamically (which would require the source
//! to be present at runtime), this module encodes the canonical ToV domain
//! knowledge directly. The source is the ground truth; this module is kept
//! in sync with it.

use crate::ir::{
    AxiomIR, ConservationLawIR, DomainAnalysis, GraphEdge, GraphNode, GraphTopology, HarmTypeIR,
    SignalThresholds, TheoremIR,
};

/// Extract the complete ToV domain analysis.
///
/// This produces the canonical IR for the Theory of Vigilance domain,
/// covering 5 axioms, 8 harm types, 11 conservation laws, 3 principal
/// theorems, the axiom dependency DAG, and signal thresholds.
pub fn extract_vigilance_domain() -> DomainAnalysis {
    let axioms = extract_axioms();
    let harm_types = extract_harm_types();
    let conservation_laws = extract_conservation_laws();
    let theorems = extract_theorems();
    let dependency_dag = build_axiom_dag();
    let signal_thresholds = SignalThresholds::default();

    DomainAnalysis {
        axioms,
        harm_types,
        conservation_laws,
        theorems,
        dependency_dag,
        signal_thresholds,
    }
}

/// Extract the 5 axioms of the Theory of Vigilance.
///
/// Source: `nexcore-vigilance/src/axiom_summary.rs`
fn extract_axioms() -> Vec<AxiomIR> {
    vec![
        AxiomIR {
            id: "A1".to_string(),
            name: "System Decomposition".to_string(),
            section: "§2".to_string(),
            core_assertion:
                "Every vigilance system admits finite elemental decomposition (with measurable Φ)"
                    .to_string(),
            definitions: vec![
                "Element (e ∈ E)".to_string(),
                "Element Set (E ⊂ S)".to_string(),
                "Composition Function (Φ)".to_string(),
                "Accessible State Space".to_string(),
                "Interaction Graph".to_string(),
            ],
            theorems_supported: vec!["Finite Decomposition".to_string()],
            dependencies: vec![],
            depth: 0,
        },
        AxiomIR {
            id: "A2".to_string(),
            name: "Hierarchical Organization".to_string(),
            section: "§3".to_string(),
            core_assertion: "S ≅ S₁ with quotient spaces Sᵢ₊₁ ≅ Sᵢ/~ᵢ and emergent properties"
                .to_string(),
            definitions: vec![
                "Level (ℓ ∈ {1,...,L})".to_string(),
                "Coarse-Graining Map (πℓ)".to_string(),
                "Emergent Property".to_string(),
                "Scale Separation".to_string(),
            ],
            theorems_supported: vec![
                "Quotient Space Structure".to_string(),
                "Emergent Property Detection".to_string(),
            ],
            dependencies: vec!["A1".to_string()],
            depth: 1,
        },
        AxiomIR {
            id: "A3".to_string(),
            name: "Conservation Constraints".to_string(),
            section: "§4".to_string(),
            core_assertion: "Harm occurs iff conservation law constraints are violated".to_string(),
            definitions: vec![
                "Conservation Law (gᵢ)".to_string(),
                "Constraint Set (G)".to_string(),
                "Feasible Region".to_string(),
                "Violation Magnitude".to_string(),
            ],
            theorems_supported: vec![
                "Conservation-Harm Equivalence".to_string(),
                "11 Conservation Laws".to_string(),
            ],
            dependencies: vec![],
            depth: 0,
        },
        AxiomIR {
            id: "A4".to_string(),
            name: "Safety Manifold".to_string(),
            section: "§5".to_string(),
            core_assertion: "Safe states form stratified space; harm is boundary crossing"
                .to_string(),
            definitions: vec![
                "Safety Manifold (M)".to_string(),
                "Harm Boundary (∂M)".to_string(),
                "Signed Distance (d)".to_string(),
                "Safety Margin (Ω)".to_string(),
                "First Passage Time".to_string(),
            ],
            theorems_supported: vec![
                "Stratified Manifold Structure".to_string(),
                "Boundary Regularity".to_string(),
            ],
            dependencies: vec!["A1".to_string(), "A3".to_string()],
            depth: 1,
        },
        AxiomIR {
            id: "A5".to_string(),
            name: "Emergence".to_string(),
            section: "§6".to_string(),
            core_assertion: "Harm probability factors as product under Markov assumption"
                .to_string(),
            definitions: vec![
                "Level Perturbation (δsᵢ)".to_string(),
                "Buffering Capacity (bᵢ)".to_string(),
                "Propagation Function (Pᵢ→ᵢ₊₁)".to_string(),
                "Harm Level (ℓ_H)".to_string(),
            ],
            theorems_supported: vec![
                "Markov Product Formula".to_string(),
                "Attenuation Theorem".to_string(),
                "Non-Markovian Extension".to_string(),
            ],
            dependencies: vec!["A2".to_string(), "A4".to_string()],
            depth: 2,
        },
    ]
}

/// Extract the 8 harm types (A-H).
///
/// Source: `nexcore-vigilance/src/tov/mod.rs`
fn extract_harm_types() -> Vec<HarmTypeIR> {
    vec![
        HarmTypeIR {
            letter: 'A',
            name: "Acute".to_string(),
            conservation_law: Some(1),
            hierarchy_levels: vec![4, 5, 6],
            doc_comment: Some("Immediate, severe harm with clear temporal relationship. Conservation Law: Law 1 (Mass) - rapid accumulation.".to_string()),
        },
        HarmTypeIR {
            letter: 'B',
            name: "Cumulative".to_string(),
            conservation_law: Some(1),
            hierarchy_levels: vec![5, 6, 7],
            doc_comment: Some("Gradual harm from repeated/prolonged exposure. Conservation Law: Law 1 (Mass) - accumulated exposure over time.".to_string()),
        },
        HarmTypeIR {
            letter: 'C',
            name: "Off-Target".to_string(),
            conservation_law: Some(2),
            hierarchy_levels: vec![3, 4, 5],
            doc_comment: Some("Unintended effects on non-target components. Conservation Law: Law 2 (Energy) - favorable off-target interactions.".to_string()),
        },
        HarmTypeIR {
            letter: 'D',
            name: "Cascade".to_string(),
            conservation_law: Some(4),
            hierarchy_levels: vec![4, 5, 6, 7],
            doc_comment: Some("Propagating failure through interconnected components. Conservation Law: Law 4 (Flux) - imbalance propagation.".to_string()),
        },
        HarmTypeIR {
            letter: 'E',
            name: "Idiosyncratic".to_string(),
            conservation_law: None,
            hierarchy_levels: vec![3, 4, 5, 6],
            doc_comment: Some("Rare harm in individuals with unusual susceptibility. Mechanism: θ ∈ Θ_susceptible (parameter-space, not conservation law).".to_string()),
        },
        HarmTypeIR {
            letter: 'F',
            name: "Saturation".to_string(),
            conservation_law: Some(8),
            hierarchy_levels: vec![3, 4, 5],
            doc_comment: Some("Harm from exceeding processing capacity. Conservation Law: Law 8 (Capacity/Saturation) - rate-limiting exceeded.".to_string()),
        },
        HarmTypeIR {
            letter: 'G',
            name: "Interaction".to_string(),
            conservation_law: Some(5),
            hierarchy_levels: vec![4, 5, 6],
            doc_comment: Some("Harm from combining multiple perturbations. Conservation Law: Law 5 (Catalyst) - competitive inhibition.".to_string()),
        },
        HarmTypeIR {
            letter: 'H',
            name: "Population".to_string(),
            conservation_law: None,
            hierarchy_levels: vec![6, 7, 8],
            doc_comment: Some("Disparate impact across subgroups. Mechanism: θ-distribution heterogeneity (not conservation law).".to_string()),
        },
    ]
}

/// Extract the 11 conservation laws.
///
/// Source: `nexcore-vigilance/src/conservation.rs`
fn extract_conservation_laws() -> Vec<ConservationLawIR> {
    vec![
        ConservationLawIR {
            number: 1,
            name: "Mass/Amount".to_string(),
            formula: "Input - Output = Accumulation".to_string(),
            doc_comment: Some("Drug mass balance, data volume.".to_string()),
        },
        ConservationLawIR {
            number: 2,
            name: "Energy/Gradient".to_string(),
            formula: "ΔG < 0 for spontaneous".to_string(),
            doc_comment: Some("Binding thermodynamics, loss decrease.".to_string()),
        },
        ConservationLawIR {
            number: 3,
            name: "State Normalization".to_string(),
            formula: "Σ fractions = 1".to_string(),
            doc_comment: Some("Receptor occupancy, attention weights.".to_string()),
        },
        ConservationLawIR {
            number: 4,
            name: "Flux Continuity".to_string(),
            formula: "Σ J_in = Σ J_out".to_string(),
            doc_comment: Some("Pathway flux, network throughput.".to_string()),
        },
        ConservationLawIR {
            number: 5,
            name: "Catalyst Invariance".to_string(),
            formula: "[Catalyst] unchanged".to_string(),
            doc_comment: Some("Enzyme not consumed, competitive inhibition.".to_string()),
        },
        ConservationLawIR {
            number: 6,
            name: "Entropy Increase".to_string(),
            formula: "ΔS ≥ 0".to_string(),
            doc_comment: Some("Reaction directionality.".to_string()),
        },
        ConservationLawIR {
            number: 7,
            name: "Momentum".to_string(),
            formula: "Σ F = dp/dt".to_string(),
            doc_comment: Some("Rate constraints on state change.".to_string()),
        },
        ConservationLawIR {
            number: 8,
            name: "Capacity/Saturation".to_string(),
            formula: "v ≤ V_max".to_string(),
            doc_comment: Some("Enzyme saturation, memory limits.".to_string()),
        },
        ConservationLawIR {
            number: 9,
            name: "Charge Conservation".to_string(),
            formula: "Σ charges = const".to_string(),
            doc_comment: Some("Ion balance, signed quantity preservation.".to_string()),
        },
        ConservationLawIR {
            number: 10,
            name: "Stoichiometry".to_string(),
            formula: "Fixed ratios".to_string(),
            doc_comment: Some("Drug metabolism ratios.".to_string()),
        },
        ConservationLawIR {
            number: 11,
            name: "Structural Invariant".to_string(),
            formula: "Topology preserved".to_string(),
            doc_comment: Some("Architecture preserved, connectivity.".to_string()),
        },
    ]
}

/// Extract the 3 principal theorems.
///
/// Source: `nexcore-vigilance/src/axiom_summary.rs`
fn extract_theorems() -> Vec<TheoremIR> {
    vec![
        TheoremIR {
            name: "Predictability Theorem".to_string(),
            statement: "Given sufficient knowledge of system state, parameters, and perturbations, harm probability can be computed from the axioms".to_string(),
            required_axioms: vec!["A1".to_string(), "A2".to_string(), "A3".to_string(), "A4".to_string(), "A5".to_string()],
        },
        TheoremIR {
            name: "Attenuation Theorem".to_string(),
            statement: "The product structure of propagation probabilities implies exponential attenuation of harm probability with hierarchical depth".to_string(),
            required_axioms: vec!["A2".to_string(), "A5".to_string()],
        },
        TheoremIR {
            name: "Intervention Theorem".to_string(),
            statement: "Modifying constraint functions, buffering capacities, or perturbation magnitudes yields quantifiable changes in harm probability".to_string(),
            required_axioms: vec!["A3".to_string(), "A4".to_string(), "A5".to_string()],
        },
    ]
}

/// Build the axiom dependency DAG as a `GraphTopology`.
///
/// Source: `nexcore-vigilance/src/axiom_summary.rs` — `AxiomDependencyGraph::new()`
fn build_axiom_dag() -> GraphTopology {
    let nodes = vec![
        GraphNode {
            id: "A1".to_string(),
            label: "System Decomposition".to_string(),
            node_type: "axiom".to_string(),
            weight: 5.0,
            metadata: serde_json::json!({ "section": "§2", "depth": 0 }),
        },
        GraphNode {
            id: "A2".to_string(),
            label: "Hierarchical Organization".to_string(),
            node_type: "axiom".to_string(),
            weight: 4.0,
            metadata: serde_json::json!({ "section": "§3", "depth": 1 }),
        },
        GraphNode {
            id: "A3".to_string(),
            label: "Conservation Constraints".to_string(),
            node_type: "axiom".to_string(),
            weight: 5.0,
            metadata: serde_json::json!({ "section": "§4", "depth": 0 }),
        },
        GraphNode {
            id: "A4".to_string(),
            label: "Safety Manifold".to_string(),
            node_type: "axiom".to_string(),
            weight: 4.0,
            metadata: serde_json::json!({ "section": "§5", "depth": 1 }),
        },
        GraphNode {
            id: "A5".to_string(),
            label: "Emergence".to_string(),
            node_type: "axiom".to_string(),
            weight: 3.0,
            metadata: serde_json::json!({ "section": "§6", "depth": 2 }),
        },
    ];

    let edges = vec![
        GraphEdge {
            source: "A1".to_string(),
            target: "A2".to_string(),
            edge_type: "depends_on".to_string(),
            weight: 1.0,
        },
        GraphEdge {
            source: "A1".to_string(),
            target: "A4".to_string(),
            edge_type: "depends_on".to_string(),
            weight: 1.0,
        },
        GraphEdge {
            source: "A3".to_string(),
            target: "A4".to_string(),
            edge_type: "depends_on".to_string(),
            weight: 1.0,
        },
        GraphEdge {
            source: "A2".to_string(),
            target: "A5".to_string(),
            edge_type: "depends_on".to_string(),
            weight: 1.0,
        },
        GraphEdge {
            source: "A4".to_string(),
            target: "A5".to_string(),
            edge_type: "depends_on".to_string(),
            weight: 1.0,
        },
    ];

    GraphTopology { nodes, edges }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_vigilance_domain() {
        let domain = extract_vigilance_domain();
        assert_eq!(domain.axioms.len(), 5);
        assert_eq!(domain.harm_types.len(), 8);
        assert_eq!(domain.conservation_laws.len(), 11);
        assert_eq!(domain.theorems.len(), 3);
        assert_eq!(domain.dependency_dag.nodes.len(), 5);
        assert_eq!(domain.dependency_dag.edges.len(), 5);
    }

    #[test]
    fn test_axiom_ids() {
        let axioms = extract_axioms();
        let ids: Vec<&str> = axioms.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, vec!["A1", "A2", "A3", "A4", "A5"]);
    }

    #[test]
    fn test_harm_type_letters() {
        let types = extract_harm_types();
        let letters: Vec<char> = types.iter().map(|h| h.letter).collect();
        assert_eq!(letters, vec!['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H']);
    }

    #[test]
    fn test_conservation_law_count() {
        let laws = extract_conservation_laws();
        assert_eq!(laws.len(), 11);
        assert_eq!(laws[0].number, 1);
        assert_eq!(laws[10].number, 11);
    }

    #[test]
    fn test_axiom_dependencies() {
        let axioms = extract_axioms();
        // A1 has no dependencies
        assert!(axioms[0].dependencies.is_empty());
        // A2 depends on A1
        assert_eq!(axioms[1].dependencies, vec!["A1"]);
        // A3 has no dependencies
        assert!(axioms[2].dependencies.is_empty());
        // A4 depends on A1, A3
        assert_eq!(axioms[3].dependencies, vec!["A1", "A3"]);
        // A5 depends on A2, A4
        assert_eq!(axioms[4].dependencies, vec!["A2", "A4"]);
    }

    #[test]
    fn test_signal_thresholds() {
        let thresholds = SignalThresholds::default();
        assert!((thresholds.prr - 2.0).abs() < f64::EPSILON);
        assert!((thresholds.chi_square - 3.841).abs() < f64::EPSILON);
        assert!((thresholds.ror_lower_ci - 1.0).abs() < f64::EPSILON);
        assert!(thresholds.ic025.abs() < f64::EPSILON);
        assert!((thresholds.eb05 - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_theorem_axiom_requirements() {
        let theorems = extract_theorems();
        // Predictability requires all 5
        assert_eq!(theorems[0].required_axioms.len(), 5);
        // Attenuation requires A2, A5
        assert_eq!(theorems[1].required_axioms, vec!["A2", "A5"]);
        // Intervention requires A3, A4, A5
        assert_eq!(theorems[2].required_axioms, vec!["A3", "A4", "A5"]);
    }
}
