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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphTopology {
    /// Graph nodes.
    pub nodes: Vec<GraphNode>,
    /// Graph edges.
    pub edges: Vec<GraphEdge>,
}

/// A node in the graph.
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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

// ═══════════════════════════════════════════════════════════════════════════
// ATOMIC LEARNING OBJECTS (Micro-Learning)
// ═══════════════════════════════════════════════════════════════════════════

/// Atomic Learning Object — the fundamental micro-learning unit.
///
/// Each ALO covers exactly one concept in 2-15 minutes. ALOs connect
/// via typed dependency edges to form a computable learning DAG.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicLearningObject {
    /// Unique ALO identifier.
    /// Format: `{pathway}-{stage_seq}-{type_prefix}{seq}`
    /// Example: `tov-01-01-h01`, `tov-01-01-c01`
    pub id: String,

    /// Human-readable title (5-80 chars).
    pub title: String,

    /// ALO type: Hook, Concept, Activity, or Reflection.
    pub alo_type: AloType,

    /// Single, measurable learning objective.
    /// Must start with a Bloom-level verb.
    pub learning_objective: String,

    /// Estimated completion time in minutes (2-15 inclusive).
    pub estimated_duration: u16,

    /// Bloom's taxonomy level for this ALO.
    pub bloom_level: BloomLevel,

    /// Markdown content body.
    pub content: String,

    /// KSB identifiers this ALO addresses (0-5).
    #[serde(default)]
    pub ksb_refs: Vec<String>,

    /// Stage ID this ALO was decomposed from.
    pub source_stage_id: String,

    /// Source activity ID within the original stage.
    #[serde(default)]
    pub source_activity_id: Option<String>,

    /// Assessment data (Reflection ALOs only).
    #[serde(default)]
    pub assessment: Option<AloAssessment>,
}

/// ALO type classification.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AloType {
    /// Curiosity trigger, 2-3 min, no prereqs.
    Hook,
    /// Core knowledge delivery, 5-10 min, single concept.
    Concept,
    /// Hands-on application, 5-15 min.
    Activity,
    /// Metacognitive synthesis, 3-5 min.
    Reflection,
}

impl AloType {
    /// ID prefix character for this type.
    pub fn prefix(&self) -> char {
        match self {
            Self::Hook => 'h',
            Self::Concept => 'c',
            Self::Activity => 'a',
            Self::Reflection => 'r',
        }
    }

    /// Minimum duration in minutes.
    pub fn min_duration(&self) -> u16 {
        match self {
            Self::Hook => 2,
            Self::Concept => 5,
            Self::Activity => 5,
            Self::Reflection => 3,
        }
    }

    /// Maximum duration in minutes.
    pub fn max_duration(&self) -> u16 {
        match self {
            Self::Hook => 3,
            Self::Concept => 10,
            Self::Activity => 15,
            Self::Reflection => 5,
        }
    }
}

/// Bloom's taxonomy levels.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BloomLevel {
    /// Recall facts and basic concepts.
    Remember,
    /// Explain ideas or concepts.
    Understand,
    /// Use information in new situations.
    Apply,
    /// Draw connections among ideas.
    Analyze,
    /// Justify a stand or decision.
    Evaluate,
    /// Produce new or original work.
    Create,
}

impl BloomLevel {
    /// Parse from string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "remember" => Some(Self::Remember),
            "understand" => Some(Self::Understand),
            "apply" => Some(Self::Apply),
            "analyze" => Some(Self::Analyze),
            "evaluate" => Some(Self::Evaluate),
            "create" => Some(Self::Create),
            _ => None,
        }
    }

    /// Numeric rank (0-5) for spatial X-axis mapping.
    pub fn rank(&self) -> u8 {
        match self {
            Self::Remember => 0,
            Self::Understand => 1,
            Self::Apply => 2,
            Self::Analyze => 3,
            Self::Evaluate => 4,
            Self::Create => 5,
        }
    }
}

/// Assessment embedded in a Reflection ALO.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AloAssessment {
    /// Minimum passing percentage (0-100).
    pub passing_score: u8,
    /// Assessment questions (1-4 per ALO).
    pub questions: Vec<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════════════════════
// DEPENDENCY EDGES
// ═══════════════════════════════════════════════════════════════════════════

/// A typed dependency edge between two ALOs.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AloEdge {
    /// Source ALO ID (the prerequisite / reinforcing ALO).
    pub from: String,
    /// Target ALO ID (the dependent ALO).
    pub to: String,
    /// Edge classification.
    pub edge_type: AloEdgeType,
    /// Connection strength: 0.0 (weak) to 1.0 (hard gate).
    pub strength: f32,
}

/// Edge type between ALOs.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AloEdgeType {
    /// Hard prerequisite — must complete source before target.
    Prereq,
    /// Soft co-requisite — best taken in proximity.
    Coreq,
    /// Reinforcement — source strengthens understanding of target.
    Strengthens,
    /// Assessment — source evaluates knowledge from target.
    Assesses,
    /// Extension — source deepens/broadens target concept.
    Extends,
}

// ═══════════════════════════════════════════════════════════════════════════
// ATOMIZED PATHWAY
// ═══════════════════════════════════════════════════════════════════════════

/// A pathway decomposed into ALOs with intra-pathway dependency edges.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomizedPathway {
    /// Pathway identifier (same as source).
    pub id: String,
    /// Pathway title.
    pub title: String,
    /// Original StaticPathway ID.
    pub source_pathway_id: String,
    /// All ALOs in this pathway.
    pub alos: Vec<AtomicLearningObject>,
    /// Intra-pathway dependency edges.
    pub edges: Vec<AloEdge>,
}

// ═══════════════════════════════════════════════════════════════════════════
// LEARNING GRAPH (Cross-Pathway)
// ═══════════════════════════════════════════════════════════════════════════

/// A complete learning graph across one or more pathways.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningGraph {
    /// All ALOs across included pathways.
    pub nodes: Vec<AtomicLearningObject>,
    /// All dependency edges (intra- and cross-pathway).
    pub edges: Vec<AloEdge>,
    /// Pathway IDs included in this graph.
    pub pathways: Vec<String>,
    /// Detected overlap clusters.
    pub overlap_clusters: Vec<OverlapCluster>,
    /// Graph metadata.
    pub metadata: GraphMetadata,
}

/// A cluster of ALOs covering the same concept across pathways.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlapCluster {
    /// Shared concept identifier (KSB ID or concept name).
    pub concept: String,
    /// ALO IDs that cover this concept.
    pub alo_ids: Vec<String>,
    /// Pathway IDs containing these ALOs.
    pub pathways: Vec<String>,
    /// The canonical ALO (highest Bloom, most complete).
    pub canonical_alo_id: String,
}

/// Graph-level metrics.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Total ALO count.
    pub node_count: usize,
    /// Total edge count.
    pub edge_count: usize,
    /// Number of connected components.
    pub connected_components: usize,
    /// Longest shortest-path (graph diameter).
    pub diameter: usize,
    /// Average ALO duration in minutes.
    pub avg_duration_min: f32,
    /// Total learning time in minutes.
    pub total_duration_min: u32,
    /// Overlap ratio: ALOs with cross-pathway overlap / total.
    pub overlap_ratio: f32,
}
