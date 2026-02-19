//! Intermediate Representation (IR) types for Academy Forge.
//!
//! The IR serves two consumers:
//! - **Observatory**: 3D spatial knowledge rendering (R3F/Three.js)
//! - **Academy**: Experiential learning pathways (Next.js)
//!
//! Fields marked `[OBS]` carry spatial/visual metadata for Observatory.
//! Fields marked `[ACAD]` are Academy-specific. Unmarked fields serve both.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// GENERIC IR (any crate)
// ═══════════════════════════════════════════════════════════════════════════

/// Top-level analysis result for a Rust crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateAnalysis {
    /// Crate name from Cargo.toml.
    pub name: String,
    /// Crate version.
    pub version: String,
    /// Crate description from Cargo.toml.
    pub description: String,
    /// Module hierarchy.
    pub modules: Vec<ModuleInfo>,
    /// Public struct types.
    pub public_types: Vec<TypeInfo>,
    /// Public enum types.
    pub public_enums: Vec<EnumInfo>,
    /// Public constants.
    pub constants: Vec<ConstantInfo>,
    /// Public traits.
    pub traits: Vec<TraitInfo>,
    /// Internal workspace dependencies.
    pub dependencies: Vec<String>,
    /// Domain-specific analysis (if a domain plugin was applied).
    pub domain: Option<DomainAnalysis>,
    /// \[OBS\] Crate-level dependency DAG for Observatory rendering.
    pub dependency_graph: GraphTopology,
}

/// Information about a module in the crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Module path (e.g., "manifold::axiom4").
    pub path: String,
    /// Doc comment (if any).
    pub doc_comment: Option<String>,
    /// Names of public items in this module.
    pub public_items: Vec<String>,
    /// File path relative to crate root.
    pub file_path: String,
    /// \[OBS\] Line count — maps to node size via Stevens' Power Law.
    pub line_count: usize,
}

/// Information about a public struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type name.
    pub name: String,
    /// Doc comment (if any).
    pub doc_comment: Option<String>,
    /// Fields of the struct.
    pub fields: Vec<FieldInfo>,
    /// Derive macros applied.
    pub derives: Vec<String>,
}

/// Information about a struct field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    /// Field name (None for tuple struct fields).
    pub name: Option<String>,
    /// Field type as string.
    pub ty: String,
    /// Doc comment (if any).
    pub doc_comment: Option<String>,
}

/// Information about a public enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumInfo {
    /// Enum name.
    pub name: String,
    /// Doc comment (if any).
    pub doc_comment: Option<String>,
    /// Variants.
    pub variants: Vec<VariantInfo>,
}

/// Information about an enum variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantInfo {
    /// Variant name.
    pub name: String,
    /// Doc comment (if any).
    pub doc_comment: Option<String>,
    /// Fields (for struct/tuple variants).
    pub fields: Vec<FieldInfo>,
}

/// Information about a public constant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantInfo {
    /// Constant name.
    pub name: String,
    /// Type as string.
    pub ty: String,
    /// Value as string (if simple enough to extract).
    pub value: Option<String>,
    /// Doc comment (if any).
    pub doc_comment: Option<String>,
}

/// Information about a public trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitInfo {
    /// Trait name.
    pub name: String,
    /// Doc comment (if any).
    pub doc_comment: Option<String>,
    /// Method names.
    pub methods: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// GRAPH TOPOLOGY (Observatory + Academy)
// ═══════════════════════════════════════════════════════════════════════════

/// \[OBS\] Graph topology for Observatory's force-directed/hierarchical layouts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphTopology {
    /// Graph nodes.
    pub nodes: Vec<GraphNode>,
    /// Graph edges.
    pub edges: Vec<GraphEdge>,
}

/// A node in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node ID.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Node type: "axiom", "harm_type", "conservation_law", "module", "theorem".
    pub node_type: String,
    /// \[OBS\] Weight — maps to size via Stevens' Power Law.
    pub weight: f64,
    /// Arbitrary domain-specific metadata.
    pub metadata: serde_json::Value,
}

/// An edge in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Edge type: "depends_on", "supports", "violates".
    pub edge_type: String,
    /// \[OBS\] Weight — maps to edge thickness.
    pub weight: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// DOMAIN-SPECIFIC IR (ToV)
// ═══════════════════════════════════════════════════════════════════════════

/// Domain-specific analysis produced by a domain plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAnalysis {
    /// Axioms from the Theory of Vigilance.
    pub axioms: Vec<AxiomIR>,
    /// Harm types (A-H).
    pub harm_types: Vec<HarmTypeIR>,
    /// Conservation laws (11 types).
    pub conservation_laws: Vec<ConservationLawIR>,
    /// Principal theorems.
    pub theorems: Vec<TheoremIR>,
    /// \[OBS\] Full dependency DAG for 3D rendering.
    pub dependency_dag: GraphTopology,
    /// Signal detection thresholds.
    pub signal_thresholds: SignalThresholds,
}

/// Axiom information for the IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomIR {
    /// Axiom ID: "A1", "A2", etc.
    pub id: String,
    /// Axiom name: "System Decomposition".
    pub name: String,
    /// ToV section reference: "§2".
    pub section: String,
    /// Mathematical core assertion.
    pub core_assertion: String,
    /// Key definitions introduced by this axiom.
    pub definitions: Vec<String>,
    /// Theorems this axiom supports.
    pub theorems_supported: Vec<String>,
    /// Axiom IDs this depends on.
    pub dependencies: Vec<String>,
    /// \[OBS\] DAG depth for hierarchical Y-positioning.
    pub depth: usize,
}

/// Harm type information for the IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmTypeIR {
    /// Type letter: 'A' through 'H'.
    pub letter: char,
    /// Type name: "Acute", "Cumulative", etc.
    pub name: String,
    /// Conservation law number (None for theta-space types E, H).
    pub conservation_law: Option<u8>,
    /// Hierarchy levels affected.
    pub hierarchy_levels: Vec<u8>,
    /// Doc comment from source.
    pub doc_comment: Option<String>,
}

/// Conservation law information for the IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationLawIR {
    /// Law number (1-11).
    pub number: u8,
    /// Law name: "Mass/Amount", "Energy/Gradient", etc.
    pub name: String,
    /// Standard form formula.
    pub formula: String,
    /// Doc comment from source.
    pub doc_comment: Option<String>,
}

/// Principal theorem information for the IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremIR {
    /// Theorem name.
    pub name: String,
    /// Theorem statement.
    pub statement: String,
    /// Required axiom IDs.
    pub required_axioms: Vec<String>,
}

/// Signal detection threshold values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalThresholds {
    /// PRR threshold (>= signals).
    pub prr: f64,
    /// Chi-square threshold.
    pub chi_square: f64,
    /// ROR lower CI threshold (> signals).
    pub ror_lower_ci: f64,
    /// IC025 threshold (> signals).
    pub ic025: f64,
    /// EBGM/EB05 threshold (>= signals).
    pub eb05: f64,
}

impl Default for SignalThresholds {
    fn default() -> Self {
        Self {
            prr: 2.0,
            chi_square: 3.841,
            ror_lower_ci: 1.0,
            ic025: 0.0,
            eb05: 2.0,
        }
    }
}
