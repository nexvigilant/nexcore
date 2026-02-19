// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! The 7 axes of the Full Compilation Space and their dimension types.
//!
//! Each axis has a concrete type representing positions along that dimension,
//! and each axis grounds to a T1 Lex Primitiva symbol.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Axis — the 7 dimensions
// ═══════════════════════════════════════════════════════════════════

/// The 7 axes of the Full Compilation Space.
///
/// Tier: T2-P (Σ Sum — enumeration of orthogonal dimensions)
///
/// Compilation isn't a line. It's a 7-dimensional space. Every tool
/// operates at the intersection of one or more axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    /// Vertical movement through representations.
    /// Intent ↔ Spec ↔ Source ↔ Token ↔ AST ↔ IR ↔ Binary ↔ Execution
    Abstraction,
    /// Lateral movement between grammars.
    /// Rust ↔ C ↔ WASM ↔ JavaScript ↔ Prima
    Language,
    /// Movement through versions in time.
    /// v1 → diff → patch → v2
    Time,
    /// Static-to-dynamic resolution.
    /// Symbolic → Partial → Concrete
    Evaluation,
    /// Self-referential depth (strange loops).
    /// Code → Code-about-code → Code-about-code-about-code
    Reflection,
    /// Dimension reduction (structure collapse).
    /// Graph → Tree → Linear → Scalar
    Projection,
    /// Forking into parallel universes.
    /// cfg!, feature flags, conditional compilation
    Branching,
}

impl Axis {
    /// All 7 axes.
    pub const ALL: [Self; 7] = [
        Self::Abstraction,
        Self::Language,
        Self::Time,
        Self::Evaluation,
        Self::Reflection,
        Self::Projection,
        Self::Branching,
    ];

    /// The T1 Lex Primitiva symbol this axis grounds to.
    pub const fn primitive_symbol(&self) -> &'static str {
        match self {
            Self::Abstraction => "σ",  // Sequence — ordered pipeline stages
            Self::Language => "μ",     // Mapping — isomorphic transformation
            Self::Time => "ν",         // Frequency — versioned recurrence
            Self::Evaluation => "∂",   // Boundary — static/dynamic frontier
            Self::Reflection => "ρ",   // Recursion — self-reference
            Self::Projection => "Σ→σ", // Sum→Sequence — dimension collapse
            Self::Branching => "Σ",    // Sum — superposition of possibilities
        }
    }

    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Abstraction => "Abstraction",
            Self::Language => "Language",
            Self::Time => "Time",
            Self::Evaluation => "Evaluation",
            Self::Reflection => "Reflection",
            Self::Projection => "Projection",
            Self::Branching => "Branching",
        }
    }

    /// One-line description.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Abstraction => "Vertical movement through representations",
            Self::Language => "Lateral movement between grammars",
            Self::Time => "Movement through versions",
            Self::Evaluation => "Static-to-dynamic resolution",
            Self::Reflection => "Self-referential depth",
            Self::Projection => "Dimension reduction",
            Self::Branching => "Forking into parallel universes",
        }
    }
}

/// Direction of movement along an axis.
///
/// Tier: T2-P (σ Sequence — ordered direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Downward: higher abstraction to lower (compile, evaluate, project).
    Down,
    /// Upward: lower abstraction to higher (decompile, abstract, explain).
    Up,
    /// Lateral: same level, different position (transpile, translate).
    Lateral,
}

// ═══════════════════════════════════════════════════════════════════
// Axis 1: Abstraction Level
// ═══════════════════════════════════════════════════════════════════

/// Abstraction level — vertical position in the compilation pipeline.
///
/// Tier: T2-P (σ Sequence — ordered abstraction levels)
///
/// Ordered from closest-to-hardware (0) to closest-to-human (7).
/// "Compiling" moves downward. "Decompiling" moves upward.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum AbstractionLevel {
    /// Runtime values — fully executed.
    Execution = 0,
    /// Machine code or bytecode.
    Binary = 1,
    /// Intermediate representation (LLVM IR, MIR, etc.).
    Ir = 2,
    /// Abstract syntax tree.
    Ast = 3,
    /// Lexed token stream.
    Token = 4,
    /// Text representation (source code).
    #[default]
    Source = 5,
    /// Formal specification.
    Specification = 6,
    /// Human intent (natural language meaning).
    Intent = 7,
}

impl AbstractionLevel {
    /// All levels from lowest to highest.
    pub const ALL: [Self; 8] = [
        Self::Execution,
        Self::Binary,
        Self::Ir,
        Self::Ast,
        Self::Token,
        Self::Source,
        Self::Specification,
        Self::Intent,
    ];

    /// Whether this level is above (more abstract than) another.
    pub fn is_above(&self, other: &Self) -> bool {
        (*self as u8) > (*other as u8)
    }

    /// Whether this level is below (less abstract than) another.
    pub fn is_below(&self, other: &Self) -> bool {
        (*self as u8) < (*other as u8)
    }

    /// Distance (in levels) to another abstraction level.
    pub fn distance(&self, other: &Self) -> u8 {
        let a = *self as u8;
        let b = *other as u8;
        if a > b { a - b } else { b - a }
    }

    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Execution => "Execution",
            Self::Binary => "Binary",
            Self::Ir => "IR",
            Self::Ast => "AST",
            Self::Token => "Token",
            Self::Source => "Source",
            Self::Specification => "Specification",
            Self::Intent => "Intent",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Axis 2: Language
// ═══════════════════════════════════════════════════════════════════

/// Language identifier — lateral position in the compilation space.
///
/// Tier: T2-P (μ Mapping — which grammar the tokens belong to)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageId(String);

impl LanguageId {
    /// Create a new language identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Well-known languages
    /// Rust programming language.
    pub fn rust() -> Self {
        Self::new("rust")
    }
    /// Prima domain language (NexCore).
    pub fn prima() -> Self {
        Self::new("prima")
    }
    /// PVDSL bytecode VM language.
    pub fn pvdsl() -> Self {
        Self::new("pvdsl")
    }
    /// JavaScript / ECMAScript.
    pub fn javascript() -> Self {
        Self::new("javascript")
    }
    /// WebAssembly.
    pub fn wasm() -> Self {
        Self::new("wasm")
    }
    /// SQL (Structured Query Language).
    pub fn sql() -> Self {
        Self::new("sql")
    }
    /// C programming language.
    pub fn c_lang() -> Self {
        Self::new("c")
    }
    /// LLVM Intermediate Representation.
    pub fn llvm_ir() -> Self {
        Self::new("llvm-ir")
    }
    /// Natural language (human speech/text).
    pub fn natural() -> Self {
        Self::new("natural-language")
    }
    /// Unknown or unspecified language.
    pub fn unknown() -> Self {
        Self::new("unknown")
    }
}

impl Default for LanguageId {
    fn default() -> Self {
        Self::unknown()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Axis 3: Time (Temporal Coordinate)
// ═══════════════════════════════════════════════════════════════════

/// Temporal coordinate — position along the version timeline.
///
/// Tier: T2-P (ν Frequency — versioned recurrence)
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TemporalCoord {
    /// Monotonic revision number.
    pub revision: u64,
    /// Optional human-readable label (e.g., "v1.2.0").
    pub label: Option<String>,
}

impl TemporalCoord {
    /// Create a temporal coordinate.
    pub fn new(revision: u64) -> Self {
        Self {
            revision,
            label: None,
        }
    }

    /// Create with a label.
    pub fn with_label(revision: u64, label: impl Into<String>) -> Self {
        Self {
            revision,
            label: Some(label.into()),
        }
    }

    /// Distance to another temporal coordinate.
    pub fn distance(&self, other: &Self) -> u64 {
        if self.revision > other.revision {
            self.revision - other.revision
        } else {
            other.revision - self.revision
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Axis 4: Evaluation State
// ═══════════════════════════════════════════════════════════════════

/// Evaluation state — how much has been resolved from symbolic to concrete.
///
/// Tier: T2-P (∂ Boundary — the static/dynamic frontier)
///
/// `const fn`, `const` generics, monomorphization, and partial evaluation
/// all move along this axis.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum EvalState {
    /// Nothing resolved — fully symbolic.
    /// All names are references, no values computed.
    #[default]
    Symbolic = 0,
    /// Some symbols resolved — partially evaluated.
    /// const eval, monomorphization, inlining.
    Partial = 1,
    /// Fully evaluated — concrete runtime values.
    Concrete = 2,
}

impl EvalState {
    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Symbolic => "Symbolic",
            Self::Partial => "Partially Evaluated",
            Self::Concrete => "Concrete",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Axis 5: Reflection Depth
// ═══════════════════════════════════════════════════════════════════

/// Reflection depth — how many meta-levels deep.
///
/// Tier: T2-P (ρ Recursion — self-referential depth)
///
/// Depth 0 = normal code. Depth 1 = proc macros, code generators.
/// Depth 2 = macros that write macros. The strange loop deepens.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ReflectionDepth(pub u32);

impl ReflectionDepth {
    /// Normal code — not meta.
    pub const GROUND: Self = Self(0);
    /// Code about code (proc macros, code generators).
    pub const META: Self = Self(1);
    /// Code about code-about-code (macro-writing macros).
    pub const META_META: Self = Self(2);

    /// Whether this is ground-level (not meta).
    pub const fn is_ground(&self) -> bool {
        self.0 == 0
    }

    /// Whether this is meta (any level above ground).
    pub const fn is_meta(&self) -> bool {
        self.0 > 0
    }

    /// The depth level.
    pub const fn depth(&self) -> u32 {
        self.0
    }

    /// Distance to another reflection depth.
    pub const fn distance(&self, other: &Self) -> u32 {
        if self.0 > other.0 {
            self.0 - other.0
        } else {
            other.0 - self.0
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Axis 6: Dimensionality (Projection)
// ═══════════════════════════════════════════════════════════════════

/// Dimensionality — structural shape of the representation.
///
/// Tier: T2-P (Σ→σ — dimension reduction from rich to flat)
///
/// Every serialization is a projection — AST (tree) → token stream (linear).
/// Information about nesting, scope, and hierarchy becomes implicit.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Dimensionality {
    /// Single value (0D).
    Scalar = 0,
    /// Sequence / stream (1D).
    #[default]
    Linear = 1,
    /// Hierarchical tree structure (2D+).
    Tree = 2,
    /// Arbitrary connections — directed acyclic graph (nD).
    Graph = 3,
}

impl Dimensionality {
    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Scalar => "Scalar",
            Self::Linear => "Linear",
            Self::Tree => "Tree",
            Self::Graph => "Graph",
        }
    }

    /// Whether this is a reduction from another dimensionality.
    pub fn is_reduction_from(&self, other: &Self) -> bool {
        (*self as u8) < (*other as u8)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Axis 7: Branch Configuration
// ═══════════════════════════════════════════════════════════════════

/// Branch configuration — which universe of possibilities is selected.
///
/// Tier: T2-P (Σ Sum — superposition of possible compilation outcomes)
///
/// Conditional compilation (`#[cfg(...)]`, `#ifdef`, feature flags)
/// creates branching token universes. A branch config is a projection
/// operator that collapses the superposition into one concrete universe.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchConfig {
    /// Active feature flags.
    pub features: Vec<String>,
    /// Target triple (e.g., "aarch64-linux-gnu").
    pub target: Option<String>,
    /// Build profile (e.g., "release", "debug").
    pub profile: Option<String>,
}

impl BranchConfig {
    /// Create an empty (default) branch config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a feature flag.
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Set the target triple.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set the build profile.
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Whether a specific feature is active.
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    /// Number of active features.
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }

    /// Whether this config is compatible with another (subset check).
    /// A config is compatible if all its features are present in the other.
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.features.iter().all(|f| other.has_feature(f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Axis tests ──

    #[test]
    fn axis_count() {
        assert_eq!(Axis::ALL.len(), 7);
    }

    #[test]
    fn axis_primitives() {
        assert_eq!(Axis::Abstraction.primitive_symbol(), "σ");
        assert_eq!(Axis::Language.primitive_symbol(), "μ");
        assert_eq!(Axis::Reflection.primitive_symbol(), "ρ");
    }

    #[test]
    fn axis_labels() {
        assert_eq!(Axis::Time.label(), "Time");
        assert_eq!(Axis::Branching.label(), "Branching");
    }

    // ── AbstractionLevel tests ──

    #[test]
    fn abstraction_ordering() {
        assert!(AbstractionLevel::Intent > AbstractionLevel::Source);
        assert!(AbstractionLevel::Source > AbstractionLevel::Token);
        assert!(AbstractionLevel::Token > AbstractionLevel::Ast);
        assert!(AbstractionLevel::Ast > AbstractionLevel::Ir);
        assert!(AbstractionLevel::Ir > AbstractionLevel::Binary);
        assert!(AbstractionLevel::Binary > AbstractionLevel::Execution);
    }

    #[test]
    fn abstraction_above_below() {
        assert!(AbstractionLevel::Intent.is_above(&AbstractionLevel::Source));
        assert!(AbstractionLevel::Execution.is_below(&AbstractionLevel::Binary));
        assert!(!AbstractionLevel::Source.is_above(&AbstractionLevel::Intent));
    }

    #[test]
    fn abstraction_distance() {
        assert_eq!(
            AbstractionLevel::Source.distance(&AbstractionLevel::Token),
            1
        );
        assert_eq!(
            AbstractionLevel::Intent.distance(&AbstractionLevel::Execution),
            7
        );
        assert_eq!(AbstractionLevel::Ast.distance(&AbstractionLevel::Ast), 0);
    }

    #[test]
    fn abstraction_all_levels() {
        assert_eq!(AbstractionLevel::ALL.len(), 8);
    }

    #[test]
    fn abstraction_default_is_source() {
        assert_eq!(AbstractionLevel::default(), AbstractionLevel::Source);
    }

    // ── LanguageId tests ──

    #[test]
    fn language_well_known() {
        assert_eq!(LanguageId::rust().as_str(), "rust");
        assert_eq!(LanguageId::prima().as_str(), "prima");
        assert_eq!(LanguageId::natural().as_str(), "natural-language");
    }

    #[test]
    fn language_custom() {
        let lang = LanguageId::new("kotlin");
        assert_eq!(lang.as_str(), "kotlin");
    }

    #[test]
    fn language_default_is_unknown() {
        assert_eq!(LanguageId::default().as_str(), "unknown");
    }

    #[test]
    fn language_equality() {
        assert_eq!(LanguageId::rust(), LanguageId::new("rust"));
        assert_ne!(LanguageId::rust(), LanguageId::c_lang());
    }

    // ── TemporalCoord tests ──

    #[test]
    fn temporal_creation() {
        let t = TemporalCoord::new(5);
        assert_eq!(t.revision, 5);
        assert!(t.label.is_none());
    }

    #[test]
    fn temporal_with_label() {
        let t = TemporalCoord::with_label(3, "v1.2.0");
        assert_eq!(t.revision, 3);
        assert_eq!(t.label.as_deref(), Some("v1.2.0"));
    }

    #[test]
    fn temporal_distance() {
        let a = TemporalCoord::new(5);
        let b = TemporalCoord::new(12);
        assert_eq!(a.distance(&b), 7);
        assert_eq!(b.distance(&a), 7);
    }

    #[test]
    fn temporal_ordering() {
        let v1 = TemporalCoord::new(1);
        let v2 = TemporalCoord::new(2);
        assert!(v1 < v2);
    }

    // ── EvalState tests ──

    #[test]
    fn eval_ordering() {
        assert!(EvalState::Symbolic < EvalState::Partial);
        assert!(EvalState::Partial < EvalState::Concrete);
    }

    #[test]
    fn eval_default_is_symbolic() {
        assert_eq!(EvalState::default(), EvalState::Symbolic);
    }

    #[test]
    fn eval_labels() {
        assert_eq!(EvalState::Symbolic.label(), "Symbolic");
        assert_eq!(EvalState::Partial.label(), "Partially Evaluated");
        assert_eq!(EvalState::Concrete.label(), "Concrete");
    }

    // ── ReflectionDepth tests ──

    #[test]
    fn reflection_ground() {
        let d = ReflectionDepth::GROUND;
        assert!(d.is_ground());
        assert!(!d.is_meta());
        assert_eq!(d.depth(), 0);
    }

    #[test]
    fn reflection_meta() {
        let d = ReflectionDepth::META;
        assert!(!d.is_ground());
        assert!(d.is_meta());
        assert_eq!(d.depth(), 1);
    }

    #[test]
    fn reflection_distance() {
        assert_eq!(
            ReflectionDepth::GROUND.distance(&ReflectionDepth::META_META),
            2
        );
    }

    #[test]
    fn reflection_ordering() {
        assert!(ReflectionDepth::GROUND < ReflectionDepth::META);
        assert!(ReflectionDepth::META < ReflectionDepth::META_META);
    }

    // ── Dimensionality tests ──

    #[test]
    fn dimensionality_ordering() {
        assert!(Dimensionality::Scalar < Dimensionality::Linear);
        assert!(Dimensionality::Linear < Dimensionality::Tree);
        assert!(Dimensionality::Tree < Dimensionality::Graph);
    }

    #[test]
    fn dimensionality_reduction() {
        assert!(Dimensionality::Linear.is_reduction_from(&Dimensionality::Tree));
        assert!(Dimensionality::Scalar.is_reduction_from(&Dimensionality::Graph));
        assert!(!Dimensionality::Graph.is_reduction_from(&Dimensionality::Linear));
    }

    #[test]
    fn dimensionality_labels() {
        assert_eq!(Dimensionality::Graph.label(), "Graph");
        assert_eq!(Dimensionality::Scalar.label(), "Scalar");
    }

    // ── BranchConfig tests ──

    #[test]
    fn branch_default_empty() {
        let b = BranchConfig::new();
        assert!(b.features.is_empty());
        assert!(b.target.is_none());
        assert!(b.profile.is_none());
    }

    #[test]
    fn branch_builder() {
        let b = BranchConfig::new()
            .with_feature("async")
            .with_feature("serde")
            .with_target("aarch64-linux-gnu")
            .with_profile("release");
        assert_eq!(b.feature_count(), 2);
        assert!(b.has_feature("async"));
        assert!(b.has_feature("serde"));
        assert!(!b.has_feature("tokio"));
        assert_eq!(b.target.as_deref(), Some("aarch64-linux-gnu"));
        assert_eq!(b.profile.as_deref(), Some("release"));
    }

    #[test]
    fn branch_subset() {
        let small = BranchConfig::new().with_feature("serde");
        let big = BranchConfig::new()
            .with_feature("serde")
            .with_feature("async");
        assert!(small.is_subset_of(&big));
        assert!(!big.is_subset_of(&small));
    }
}
