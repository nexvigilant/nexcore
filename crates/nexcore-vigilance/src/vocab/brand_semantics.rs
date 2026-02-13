//! Brand Semantics: Primitive extraction and decomposition for brand names.
//!
//! Implements the NexVigilant brand decomposition methodology:
//! - Etymology extraction (Latin/Greek roots)
//! - Primitive tier classification (T1/T2-P/T2-C/T3)
//! - Three-part primitive testing
//! - Cross-domain transfer mappings
//!
//! # Tier System
//!
//! | Tier | Coverage | Description |
//! |------|----------|-------------|
//! | T1 | Universal | Irreducible concepts (σ, μ, ς, ρ, ∅, ∂, f, ∃, π, →, κ, N, λ, ∝) |
//! | T2-P | Cross-Domain Primitive | Atomic atoms in 2+ domains |
//! | T2-C | Cross-Domain Composite | Built from T1/T2-P components |
//! | T3 | Domain-Specific | Single-domain terminology |
//!
//! # Example
//!
//! ```
//! use nexcore_vigilance::vocab::brand_semantics::{
//!     BrandDecomposition, EtymologyRoot, SemanticTier,
//! };
//!
//! let nex = EtymologyRoot::latin("nex", "necis", "death, violent death");
//! let vigilant = EtymologyRoot::latin("vigilāre", "vigilans", "watching, alert");
//!
//! let decomposition = BrandDecomposition::new("NexVigilant")
//!     .with_roots(vec![nex, vigilant])
//!     .with_semantic_field("Death-watcher / Guardian against death");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// TIER CLASSIFICATION (Extended)
// ============================================================================

/// Extended semantic tier classification with T2 primitive/composite distinction.
///
/// Unlike the basic 3-tier vocabulary system, brand semantics requires
/// distinguishing between atomic cross-domain primitives (T2-P) and
/// composites built from primitives (T2-C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTier {
    /// T1: Universal primitives appearing in all human knowledge systems.
    /// Symbols: σ, μ, ς, ρ, ∅, ∂, f, ∃, π, →, κ, N, λ, ∝
    #[serde(rename = "T1_Universal")]
    T1Universal,

    /// T2-P: Cross-domain atomic atoms (cannot be decomposed within domains).
    /// Examples: observation, harm, anticipation, threshold
    #[serde(rename = "T2_Primitive")]
    T2Primitive,

    /// T2-C: Cross-domain composites (built from T1/T2-P components).
    /// Examples: vigilance (observation + duration + purpose)
    #[serde(rename = "T2_Composite")]
    T2Composite,

    /// T3: Domain-specific terminology.
    /// Examples: adverse event, ICSR, MedDRA
    #[serde(rename = "T3_DomainSpecific")]
    T3DomainSpecific,
}

impl SemanticTier {
    /// Human-readable tier name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::T1Universal => "T1 Universal",
            Self::T2Primitive => "T2-P Cross-Domain Primitive",
            Self::T2Composite => "T2-C Cross-Domain Composite",
            Self::T3DomainSpecific => "T3 Domain-Specific",
        }
    }

    /// Short label for display.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::T1Universal => "T1",
            Self::T2Primitive => "T2-P",
            Self::T2Composite => "T2-C",
            Self::T3DomainSpecific => "T3",
        }
    }

    /// Whether this tier represents a composite (built from components).
    #[must_use]
    pub const fn is_composite(&self) -> bool {
        matches!(self, Self::T2Composite | Self::T3DomainSpecific)
    }

    /// Whether this tier represents a primitive (irreducible within its scope).
    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        matches!(self, Self::T1Universal | Self::T2Primitive)
    }
}

impl std::fmt::Display for SemanticTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ============================================================================
// ETYMOLOGY ROOTS
// ============================================================================

/// Language of origin for etymology roots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RootLanguage {
    /// Latin origin.
    #[default]
    Latin,
    /// Greek origin.
    Greek,
    /// Old English / Germanic origin.
    OldEnglish,
    /// Other/mixed origin.
    Other,
}

impl std::fmt::Display for RootLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Latin => "Latin",
            Self::Greek => "Greek",
            Self::OldEnglish => "Old English",
            Self::Other => "Other",
        };
        write!(f, "{name}")
    }
}

/// An etymology root extracted from a compound word.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EtymologyRoot {
    /// The root word/morpheme.
    pub root: String,
    /// Original form in source language (e.g., "necis (f.)").
    pub original_form: String,
    /// Core meaning.
    pub meaning: String,
    /// Language of origin.
    pub language: RootLanguage,
    /// Extended semantic notes.
    #[serde(default)]
    pub semantic_notes: Option<String>,
}

impl EtymologyRoot {
    /// Create a Latin etymology root.
    #[must_use]
    pub fn latin(
        root: impl Into<String>,
        original: impl Into<String>,
        meaning: impl Into<String>,
    ) -> Self {
        Self {
            root: root.into(),
            original_form: original.into(),
            meaning: meaning.into(),
            language: RootLanguage::Latin,
            semantic_notes: None,
        }
    }

    /// Create a Greek etymology root.
    #[must_use]
    pub fn greek(
        root: impl Into<String>,
        original: impl Into<String>,
        meaning: impl Into<String>,
    ) -> Self {
        Self {
            root: root.into(),
            original_form: original.into(),
            meaning: meaning.into(),
            language: RootLanguage::Greek,
            semantic_notes: None,
        }
    }

    /// Add semantic notes.
    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.semantic_notes = Some(notes.into());
        self
    }
}

// ============================================================================
// PRIMITIVE TEST FRAMEWORK
// ============================================================================

/// Result of a single test in the primitive testing framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestResult {
    /// Test passed.
    Pass,
    /// Test failed.
    Fail,
    /// Borderline result (requires judgment).
    Borderline,
}

impl TestResult {
    /// Whether the test indicates primitive status.
    #[must_use]
    pub const fn indicates_primitive(&self) -> bool {
        matches!(self, Self::Pass | Self::Borderline)
    }
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Borderline => "BORDERLINE",
        };
        write!(f, "{label}")
    }
}

/// Three-part primitive test for determining irreducibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveTest {
    /// The term being tested.
    pub term: String,
    /// Definition used for testing.
    pub definition: String,

    // Test 1: No Domain-Internal Dependencies
    /// Domain terms found in the definition.
    pub domain_terms_found: Vec<String>,
    /// Result of Test 1.
    pub test1_no_domain_deps: TestResult,

    // Test 2: Grounds to External Concepts
    /// External concepts the term grounds to.
    pub external_grounding: Vec<String>,
    /// Result of Test 2.
    pub test2_external_grounding: TestResult,

    // Test 3: Not Merely a Synonym
    /// Synonym check explanation.
    pub synonym_analysis: String,
    /// Result of Test 3.
    pub test3_not_synonym: TestResult,

    /// Final verdict.
    pub verdict: PrimitiveVerdict,
    /// Assigned tier based on verdict and domain coverage.
    pub tier: SemanticTier,
    /// Additional notes.
    #[serde(default)]
    pub notes: Option<String>,
}

/// Verdict from primitive testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimitiveVerdict {
    /// Term is a primitive (irreducible).
    Primitive,
    /// Term is a composite (can be decomposed).
    Composite,
    /// Requires further analysis.
    Undetermined,
}

impl PrimitiveTest {
    /// Create a new primitive test.
    #[must_use]
    pub fn new(term: impl Into<String>, definition: impl Into<String>) -> Self {
        Self {
            term: term.into(),
            definition: definition.into(),
            domain_terms_found: Vec::new(),
            test1_no_domain_deps: TestResult::Pass,
            external_grounding: Vec::new(),
            test2_external_grounding: TestResult::Pass,
            synonym_analysis: String::new(),
            test3_not_synonym: TestResult::Pass,
            verdict: PrimitiveVerdict::Undetermined,
            tier: SemanticTier::T2Primitive,
            notes: None,
        }
    }

    /// Set Test 1 results.
    #[must_use]
    pub fn with_test1(mut self, result: TestResult, domain_terms: Vec<String>) -> Self {
        self.test1_no_domain_deps = result;
        self.domain_terms_found = domain_terms;
        self
    }

    /// Set Test 2 results.
    #[must_use]
    pub fn with_test2(mut self, result: TestResult, external: Vec<String>) -> Self {
        self.test2_external_grounding = result;
        self.external_grounding = external;
        self
    }

    /// Set Test 3 results.
    #[must_use]
    pub fn with_test3(mut self, result: TestResult, analysis: impl Into<String>) -> Self {
        self.test3_not_synonym = result;
        self.synonym_analysis = analysis.into();
        self
    }

    /// Set the verdict and tier.
    #[must_use]
    pub fn with_verdict(mut self, verdict: PrimitiveVerdict, tier: SemanticTier) -> Self {
        self.verdict = verdict;
        self.tier = tier;
        self
    }

    /// Add notes.
    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Compute verdict from test results.
    #[must_use]
    pub fn compute_verdict(&self) -> PrimitiveVerdict {
        let tests = [
            self.test1_no_domain_deps,
            self.test2_external_grounding,
            self.test3_not_synonym,
        ];

        if tests.iter().any(|t| *t == TestResult::Fail) {
            PrimitiveVerdict::Composite
        } else if tests.iter().all(|t| *t == TestResult::Pass) {
            PrimitiveVerdict::Primitive
        } else {
            PrimitiveVerdict::Undetermined
        }
    }

    /// Check if all tests pass.
    #[must_use]
    pub fn all_pass(&self) -> bool {
        self.test1_no_domain_deps == TestResult::Pass
            && self.test2_external_grounding == TestResult::Pass
            && self.test3_not_synonym == TestResult::Pass
    }
}

// ============================================================================
// SEMANTIC PRIMITIVES
// ============================================================================

/// A semantic primitive with tier classification and dependencies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticPrimitive {
    /// The primitive name.
    pub name: String,
    /// Definition.
    pub definition: String,
    /// Tier classification.
    pub tier: SemanticTier,
    /// Domains where this primitive appears (for T2+).
    #[serde(default)]
    pub domains_present: Vec<String>,
    /// External concepts this grounds to (for T1).
    #[serde(default)]
    pub grounds_to: Vec<String>,
    /// Components (for composites).
    #[serde(default)]
    pub components: Vec<String>,
    /// Additional notes.
    #[serde(default)]
    pub note: Option<String>,
}

impl SemanticPrimitive {
    /// Create a T1 universal primitive.
    #[must_use]
    pub fn t1_universal(
        name: impl Into<String>,
        definition: impl Into<String>,
        grounds_to: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            definition: definition.into(),
            tier: SemanticTier::T1Universal,
            domains_present: Vec::new(),
            grounds_to,
            components: Vec::new(),
            note: None,
        }
    }

    /// Create a T2-P cross-domain primitive.
    #[must_use]
    pub fn t2_primitive(name: impl Into<String>, domains: Vec<String>) -> Self {
        Self {
            name: name.into(),
            definition: String::new(),
            tier: SemanticTier::T2Primitive,
            domains_present: domains,
            grounds_to: Vec::new(),
            components: Vec::new(),
            note: None,
        }
    }

    /// Create a T2-C cross-domain composite.
    #[must_use]
    pub fn t2_composite(
        name: impl Into<String>,
        components: Vec<String>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            definition: String::new(),
            tier: SemanticTier::T2Composite,
            domains_present: Vec::new(),
            grounds_to: Vec::new(),
            components,
            note: Some(note.into()),
        }
    }

    /// Create a T3 domain-specific item.
    #[must_use]
    pub fn t3_domain_specific(name: impl Into<String>, components: Vec<String>) -> Self {
        Self {
            name: name.into(),
            definition: String::new(),
            tier: SemanticTier::T3DomainSpecific,
            domains_present: Vec::new(),
            grounds_to: Vec::new(),
            components,
            note: None,
        }
    }

    /// Add a definition.
    #[must_use]
    pub fn with_definition(mut self, definition: impl Into<String>) -> Self {
        self.definition = definition.into();
        self
    }

    /// Add a note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

// ============================================================================
// TRANSFER MAPPINGS
// ============================================================================

/// Confidence scores for cross-domain transfer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransferConfidence {
    /// Overall transfer confidence [0.0, 1.0].
    pub overall: f64,
    /// Structural similarity score.
    pub structural: f64,
    /// Functional similarity score.
    pub functional: f64,
    /// Contextual similarity score.
    pub contextual: f64,
}

impl TransferConfidence {
    /// Create a new transfer confidence.
    #[must_use]
    pub fn new(overall: f64, structural: f64, functional: f64, contextual: f64) -> Self {
        Self {
            overall: overall.clamp(0.0, 1.0),
            structural: structural.clamp(0.0, 1.0),
            functional: functional.clamp(0.0, 1.0),
            contextual: contextual.clamp(0.0, 1.0),
        }
    }

    /// Create from overall score only.
    #[must_use]
    pub fn from_overall(overall: f64) -> Self {
        let clamped = overall.clamp(0.0, 1.0);
        Self {
            overall: clamped,
            structural: clamped,
            functional: clamped,
            contextual: clamped,
        }
    }

    /// Returns the limiting factor (lowest dimension).
    #[must_use]
    pub fn limiting_factor(&self) -> &'static str {
        let min = self.structural.min(self.functional).min(self.contextual);
        if (min - self.contextual).abs() < f64::EPSILON {
            "contextual"
        } else if (min - self.functional).abs() < f64::EPSILON {
            "functional"
        } else {
            "structural"
        }
    }

    /// Check if transfer is high-confidence (≥0.8).
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.overall >= 0.8
    }
}

impl Default for TransferConfidence {
    fn default() -> Self {
        Self::from_overall(0.5)
    }
}

/// A cross-domain transfer mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {
    /// Source domain/primitive.
    pub source: String,
    /// Target domain equivalent.
    pub target: String,
    /// Target domain name.
    pub target_domain: String,
    /// Transfer confidence scores.
    pub confidence: TransferConfidence,
    /// Caveat or limitation note.
    #[serde(default)]
    pub caveat: Option<String>,
}

impl TransferMapping {
    /// Create a new transfer mapping.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        target_domain: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            target_domain: target_domain.into(),
            confidence: TransferConfidence::from_overall(confidence),
            caveat: None,
        }
    }

    /// Add detailed confidence scores.
    #[must_use]
    pub fn with_detailed_confidence(mut self, confidence: TransferConfidence) -> Self {
        self.confidence = confidence;
        self
    }

    /// Add a caveat.
    #[must_use]
    pub fn with_caveat(mut self, caveat: impl Into<String>) -> Self {
        self.caveat = Some(caveat.into());
        self
    }
}

// ============================================================================
// BRAND DECOMPOSITION
// ============================================================================

/// Primitives organized by tier.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PrimitivesBySemanticTier {
    /// T1 Universal primitives.
    #[serde(rename = "T1_Universal", default)]
    pub t1_universal: Vec<SemanticPrimitive>,

    /// T2-P Cross-domain primitives.
    #[serde(rename = "T2_Primitives", default)]
    pub t2_primitives: Vec<SemanticPrimitive>,

    /// T2-C Cross-domain composites.
    #[serde(rename = "T2_Composites", default)]
    pub t2_composites: Vec<SemanticPrimitive>,

    /// T3 Domain-specific items.
    #[serde(rename = "T3_DomainSpecific", default)]
    pub t3_domain_specific: Vec<SemanticPrimitive>,
}

impl PrimitivesBySemanticTier {
    /// Iterate over all primitives.
    pub fn all(&self) -> impl Iterator<Item = &SemanticPrimitive> {
        self.t1_universal
            .iter()
            .chain(self.t2_primitives.iter())
            .chain(self.t2_composites.iter())
            .chain(self.t3_domain_specific.iter())
    }

    /// Add a primitive to the appropriate tier.
    pub fn add(&mut self, primitive: SemanticPrimitive) {
        match primitive.tier {
            SemanticTier::T1Universal => self.t1_universal.push(primitive),
            SemanticTier::T2Primitive => self.t2_primitives.push(primitive),
            SemanticTier::T2Composite => self.t2_composites.push(primitive),
            SemanticTier::T3DomainSpecific => self.t3_domain_specific.push(primitive),
        }
    }

    /// Count total primitives.
    #[must_use]
    pub fn total(&self) -> usize {
        self.t1_universal.len()
            + self.t2_primitives.len()
            + self.t2_composites.len()
            + self.t3_domain_specific.len()
    }
}

/// Complete brand semantic decomposition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrandDecomposition {
    /// Brand name being analyzed.
    pub brand_name: String,
    /// Extraction date (ISO 8601).
    pub extraction_date: String,
    /// Source mode.
    #[serde(default)]
    pub source_mode: Option<String>,
    /// Warning about source validity.
    #[serde(default)]
    pub source_warning: Option<String>,

    // Phase 1: Vocabulary Extraction
    /// Etymology roots extracted.
    pub roots: Vec<EtymologyRoot>,
    /// Derived semantic field description.
    #[serde(default)]
    pub semantic_field: Option<String>,

    // Phase 2: Dependency Analysis
    /// Dependency tree as nested map.
    #[serde(default)]
    pub dependency_tree: HashMap<String, Vec<String>>,

    // Phase 3: Primitive Tests
    /// Primitive test results.
    #[serde(default)]
    pub primitive_tests: Vec<PrimitiveTest>,

    // Phase 4: Tier Classification
    /// Primitives organized by tier.
    pub primitives: PrimitivesBySemanticTier,

    // Phase 5: Semantic Synthesis
    /// Brand semantic formula.
    #[serde(default)]
    pub semantic_formula: Option<String>,
    /// Interpretation of the brand meaning.
    #[serde(default)]
    pub interpretation: Option<String>,

    // Transfer Mappings
    /// Cross-domain transfer mappings.
    #[serde(default)]
    pub transfer_mappings: Vec<TransferMapping>,

    // Validation Summary
    /// Number of terms analyzed.
    pub terms_analyzed: usize,
    /// Number of primitives identified.
    pub primitives_identified: usize,
    /// Number of composites identified.
    pub composites_identified: usize,
    /// Brand insight summary.
    #[serde(default)]
    pub brand_insight: Option<String>,
}

impl BrandDecomposition {
    /// Create a new brand decomposition.
    #[must_use]
    pub fn new(brand_name: impl Into<String>) -> Self {
        Self {
            brand_name: brand_name.into(),
            extraction_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            source_mode: None,
            source_warning: None,
            roots: Vec::new(),
            semantic_field: None,
            dependency_tree: HashMap::new(),
            primitive_tests: Vec::new(),
            primitives: PrimitivesBySemanticTier::default(),
            semantic_formula: None,
            interpretation: None,
            transfer_mappings: Vec::new(),
            terms_analyzed: 0,
            primitives_identified: 0,
            composites_identified: 0,
            brand_insight: None,
        }
    }

    /// Set extraction metadata.
    #[must_use]
    pub fn with_metadata(
        mut self,
        source_mode: impl Into<String>,
        source_warning: impl Into<String>,
    ) -> Self {
        self.source_mode = Some(source_mode.into());
        self.source_warning = Some(source_warning.into());
        self
    }

    /// Set etymology roots.
    #[must_use]
    pub fn with_roots(mut self, roots: Vec<EtymologyRoot>) -> Self {
        self.roots = roots;
        self
    }

    /// Set semantic field description.
    #[must_use]
    pub fn with_semantic_field(mut self, field: impl Into<String>) -> Self {
        self.semantic_field = Some(field.into());
        self
    }

    /// Add a dependency relationship.
    pub fn add_dependency(&mut self, parent: impl Into<String>, children: Vec<String>) {
        self.dependency_tree.insert(parent.into(), children);
    }

    /// Add a primitive test result.
    pub fn add_test(&mut self, test: PrimitiveTest) {
        self.primitive_tests.push(test);
    }

    /// Add a primitive.
    pub fn add_primitive(&mut self, primitive: SemanticPrimitive) {
        self.primitives.add(primitive);
    }

    /// Set semantic synthesis.
    #[must_use]
    pub fn with_synthesis(
        mut self,
        formula: impl Into<String>,
        interpretation: impl Into<String>,
    ) -> Self {
        self.semantic_formula = Some(formula.into());
        self.interpretation = Some(interpretation.into());
        self
    }

    /// Add transfer mappings.
    #[must_use]
    pub fn with_transfers(mut self, mappings: Vec<TransferMapping>) -> Self {
        self.transfer_mappings = mappings;
        self
    }

    /// Set brand insight.
    #[must_use]
    pub fn with_insight(mut self, insight: impl Into<String>) -> Self {
        self.brand_insight = Some(insight.into());
        self
    }

    /// Finalize with counts.
    #[must_use]
    pub fn finalize(mut self) -> Self {
        self.terms_analyzed = self.primitive_tests.len();
        self.primitives_identified =
            self.primitives.t1_universal.len() + self.primitives.t2_primitives.len();
        self.composites_identified =
            self.primitives.t2_composites.len() + self.primitives.t3_domain_specific.len();
        self
    }

    /// Serialize to YAML.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Deserialize from YAML.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

// ============================================================================
// NEXVIGILANT DECOMPOSITION (Pre-built)
// ============================================================================

/// Returns the pre-computed NexVigilant brand decomposition.
#[must_use]
pub fn nexvigilant_decomposition() -> BrandDecomposition {
    let mut decomposition = BrandDecomposition::new("NexVigilant")
        .with_metadata(
            "expert_generation",
            "Derived from Latin etymology + model knowledge. Requires validation.",
        )
        .with_roots(vec![
            EtymologyRoot::latin(
                "nex",
                "nex, necis (f.)",
                "death, violent death, slaughter, murder",
            ),
            EtymologyRoot::latin(
                "vigilant",
                "vigilāre → vigilans",
                "watching, wakeful, alert, on guard",
            ),
        ])
        .with_semantic_field("Death-watcher / Guardian against death / One who watches over death");

    // Add T1 Universal primitives grounded to symbols
    decomposition.add_primitive(SemanticPrimitive::t1_universal(
        "death/nex",
        "Permanent cessation of existence",
        vec!["Irreversibility (∝)".into(), "Void (∅)".into()],
    ));
    decomposition.add_primitive(SemanticPrimitive::t1_universal(
        "existence",
        "Simple presence; instantiation of being",
        vec!["Existence (∃)".into()],
    ));
    decomposition.add_primitive(SemanticPrimitive::t1_universal(
        "time",
        "Ordered sequence of states with duration",
        vec!["Sequence (σ)".into(), "Persistence (π)".into()],
    ));
    decomposition.add_primitive(SemanticPrimitive::t1_universal(
        "cause",
        "Producer (Cause) and consequence (Effect)",
        vec!["Causality (→)".into()],
    ));

    // Add T2-P Cross-domain primitives (Atomic Atoms)
    decomposition.add_primitive(SemanticPrimitive::t2_primitive(
        "observation",
        vec!["Mapping (μ)".into(), "State (ς)".into()],
    ));
    decomposition.add_primitive(SemanticPrimitive::t2_primitive(
        "harm",
        vec!["Boundary (∂)".into(), "Quantity (N)".into()],
    ));
    decomposition.add_primitive(SemanticPrimitive::t2_primitive(
        "anticipation",
        vec!["Recursion (ρ)".into(), "Causality (→)".into()],
    ));

    // Add T2-C Cross-domain composites (Architectural Patterns)
    decomposition.add_primitive(SemanticPrimitive::t2_composite(
        "vigilance",
        vec!["observation".into(), "time".into(), "cause".into()],
        "Sustained protective attention: Mapping (μ) + Sequence (σ) + Causality (→)",
    ));
    decomposition.add_primitive(SemanticPrimitive::t2_composite(
        "protection",
        vec!["Boundary (∂)".into(), "Causality (→)".into()],
        "Active harm prevention using enforced state boundaries",
    ));
    decomposition.add_primitive(SemanticPrimitive::t2_composite(
        "pharmacovigilance",
        vec!["drug".into(), "vigilance".into()],
        "Domain instantiation: T3 (Drug) grounded via T2-C (Vigilance)",
    ));

    // Add T3 Domain-specific items
    decomposition.add_primitive(SemanticPrimitive::t3_domain_specific(
        "adverse event",
        vec!["harm".into(), "drug".into(), "cause".into()],
    ));
    decomposition.add_primitive(SemanticPrimitive::t3_domain_specific(
        "signal (PV)",
        vec!["Frequency (f)".into(), "Threshold (T2-P)".into()],
    ));

    // Add primitive tests
    decomposition.add_test(
        PrimitiveTest::new("nex (death)", "Permanent cessation of existence")
            .with_test1(TestResult::Pass, vec![])
            .with_test2(
                TestResult::Pass,
                vec!["Irreversibility (∝)".into(), "Void (∅)".into()],
            )
            .with_test3(
                TestResult::Pass,
                String::from("Irreducible transition to null state"),
            )
            .with_verdict(PrimitiveVerdict::Primitive, SemanticTier::T1Universal),
    );

    decomposition.add_test(
        PrimitiveTest::new(
            "vigilare (to watch)",
            "Sustained directed attention with intent to detect",
        )
        .with_test1(TestResult::Fail, vec!["attention".into(), "detect".into()])
        .with_test2(
            TestResult::Pass,
            vec![
                "Sequence (σ)".into(),
                "Mapping (μ)".into(),
                "Causality (→)".into(),
            ],
        )
        .with_test3(
            TestResult::Pass,
            String::from("Composite of observation over duration with protective purpose"),
        )
        .with_verdict(PrimitiveVerdict::Composite, SemanticTier::T2Composite),
    );

    // Add transfer mappings
    decomposition = decomposition.with_transfers(vec![
        TransferMapping::new("death/nex", "fatality", "aviation", 0.95),
        TransferMapping::new("vigilance", "monitoring", "aviation", 0.90),
        TransferMapping::new("pharmacovigilance", "aviation safety", "aviation", 0.85),
    ]);

    // Semantic synthesis
    decomposition = decomposition
        .with_synthesis(
            "NexVigilant = f(∃, ∝, σ, →)",
            "Active watchfulness (σ + →) over the existence (∃) and irreversibility (∝) of harm",
        )
        .with_insight(
            "\"NexVigilant\" grounds directly to the 14 T1 primitives, \n             encoding the mission of watching over (Sequence σ) the \n             causal pathways (Causality →) leading to permanent (Irreversibility ∝) \n             cessation of existence (Void ∅).",
        );

    decomposition.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_tier() {
        assert_eq!(SemanticTier::T1Universal.label(), "T1");
        assert_eq!(SemanticTier::T2Primitive.label(), "T2-P");
        assert_eq!(SemanticTier::T2Composite.label(), "T2-C");
        assert_eq!(SemanticTier::T3DomainSpecific.label(), "T3");

        assert!(SemanticTier::T1Universal.is_primitive());
        assert!(SemanticTier::T2Primitive.is_primitive());
        assert!(SemanticTier::T2Composite.is_composite());
        assert!(SemanticTier::T3DomainSpecific.is_composite());
    }

    #[test]
    fn test_etymology_root() {
        let nex = EtymologyRoot::latin("nex", "necis (f.)", "death");
        assert_eq!(nex.language, RootLanguage::Latin);
        assert_eq!(nex.root, "nex");
    }

    #[test]
    fn test_primitive_test() {
        let test = PrimitiveTest::new("entity", "A thing that exists")
            .with_test1(TestResult::Pass, vec![])
            .with_test2(TestResult::Pass, vec!["Existence (∃)".into()])
            .with_test3(TestResult::Pass, "Distinct meaning");

        assert!(test.all_pass());
        assert_eq!(test.compute_verdict(), PrimitiveVerdict::Primitive);
    }

    #[test]
    fn test_transfer_confidence() {
        let conf = TransferConfidence::new(0.82, 0.85, 0.88, 0.65);
        assert_eq!(conf.limiting_factor(), "contextual");
        assert!(conf.is_high_confidence());
    }

    #[test]
    fn test_nexvigilant_decomposition() {
        let decomp = nexvigilant_decomposition();

        assert_eq!(decomp.brand_name, "NexVigilant");
        assert_eq!(decomp.roots.len(), 2);
        assert_eq!(decomp.primitives.t1_universal.len(), 4);
        assert_eq!(decomp.primitives.t2_primitives.len(), 3);
        assert_eq!(decomp.primitives.t2_composites.len(), 3);
        assert_eq!(decomp.primitives.t3_domain_specific.len(), 2);
    }
}
