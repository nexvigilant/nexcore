//! # CEP Pipeline Stages
//!
//! Type definitions for each of the 8 CEP stages.

use crate::domain_discovery::{PrimitiveTier, PrimitivesByTier, TierCounts};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stage identifier in the CEP pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StageId {
    /// Stage 1: Observation
    See,
    /// Stage 2: Articulation
    Speak,
    /// Stage 3: Decomposition
    Decompose,
    /// Stage 4: Synthesis
    Compose,
    /// Stage 5: Translation
    Translate,
    /// Stage 6: Verification
    Validate,
    /// Stage 7: Implementation
    Deploy,
    /// Stage 8: Iteration
    Improve,
}

impl StageId {
    /// Returns the stage number (1-8).
    #[must_use]
    pub const fn number(&self) -> u8 {
        match self {
            Self::See => 1,
            Self::Speak => 2,
            Self::Decompose => 3,
            Self::Compose => 4,
            Self::Translate => 5,
            Self::Validate => 6,
            Self::Deploy => 7,
            Self::Improve => 8,
        }
    }

    /// Returns the stage name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::See => "SEE",
            Self::Speak => "SPEAK",
            Self::Decompose => "DECOMPOSE",
            Self::Compose => "COMPOSE",
            Self::Translate => "TRANSLATE",
            Self::Validate => "VALIDATE",
            Self::Deploy => "DEPLOY",
            Self::Improve => "IMPROVE",
        }
    }

    /// Returns the next stage in the pipeline.
    #[must_use]
    pub const fn next(&self) -> Option<Self> {
        match self {
            Self::See => Some(Self::Speak),
            Self::Speak => Some(Self::Decompose),
            Self::Decompose => Some(Self::Compose),
            Self::Compose => Some(Self::Translate),
            Self::Translate => Some(Self::Validate),
            Self::Validate => Some(Self::Deploy),
            Self::Deploy => Some(Self::Improve),
            Self::Improve => None, // Loops back to SEE via feedback
        }
    }
}

/// Output from any CEP stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageOutput {
    /// Which stage produced this output.
    pub stage: StageId,
    /// Timestamp when completed.
    pub timestamp: nexcore_chrono::DateTime,
    /// Confidence in the output (0.0-1.0).
    pub confidence: f64,
    /// Feedback signal for IMPROVE stage.
    pub feedback: Option<super::FeedbackSignal>,
}

/// Trait for CEP stage implementations.
pub trait CepStage {
    /// Input type for this stage.
    type Input;
    /// Output type for this stage.
    type Output;
    /// Error type.
    type Error: std::error::Error;

    /// Returns the stage identifier.
    fn stage_id(&self) -> StageId;

    /// Executes the stage.
    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Generates feedback signal from execution.
    fn feedback(&self, output: &Self::Output) -> super::FeedbackSignal;
}

// ============================================================================
// Stage 1: SEE (Observation)
// ============================================================================

/// Stage 1: Observe domain phenomena without prejudice.
#[derive(Debug, Clone, Default)]
pub struct SeeStage;

/// An observable phenomenon in the domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phenomenon {
    /// Unique identifier.
    pub id: String,
    /// Description of what was observed.
    pub description: String,
    /// Frequency of occurrence.
    #[serde(default)]
    pub frequency: Option<f64>,
    /// Source of observation.
    #[serde(default)]
    pub source: Option<String>,
}

/// A recurring pattern identified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Phenomena that exhibit this pattern.
    #[serde(default)]
    pub phenomena: Vec<String>,
    /// Confidence in pattern validity.
    pub confidence: f64,
}

/// An anomaly or surprise observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Description.
    pub description: String,
    /// Why it's surprising.
    #[serde(default)]
    pub rationale: Option<String>,
    /// Severity (0.0-1.0).
    pub severity: f64,
}

/// Output of Stage 1: SEE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionRecord {
    /// Domain being observed.
    pub domain: String,
    /// Observable phenomena.
    pub phenomena: Vec<Phenomenon>,
    /// Identified patterns.
    pub patterns: Vec<Pattern>,
    /// Anomalies/surprises.
    pub anomalies: Vec<Anomaly>,
    /// Overall confidence.
    pub confidence: f64,
}

impl PerceptionRecord {
    /// Creates a new perception record.
    #[must_use]
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            phenomena: Vec::new(),
            patterns: Vec::new(),
            anomalies: Vec::new(),
            confidence: 0.0,
        }
    }
}

// ============================================================================
// Stage 2: SPEAK (Articulation)
// ============================================================================

/// Stage 2: Articulate observations into structured vocabulary.
#[derive(Debug, Clone, Default)]
pub struct SpeakStage;

/// A glossary entry with preliminary tier assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    /// Term name.
    pub term: String,
    /// Definition.
    pub definition: String,
    /// Preliminary tier assignment.
    pub preliminary_tier: PrimitiveTier,
    /// Source phenomena this term describes.
    #[serde(default)]
    pub source_phenomena: Vec<String>,
}

/// Output of Stage 2: SPEAK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyRecord {
    /// Domain.
    pub domain: String,
    /// Extracted terms with definitions.
    pub terms: HashMap<String, String>,
    /// Preliminary tier assignments.
    pub tier_assignments: HashMap<String, PrimitiveTier>,
    /// Full glossary.
    pub glossary: Vec<GlossaryEntry>,
}

impl VocabularyRecord {
    /// Creates a new vocabulary record.
    #[must_use]
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            terms: HashMap::new(),
            tier_assignments: HashMap::new(),
            glossary: Vec::new(),
        }
    }

    /// Adds a term with definition and tier.
    pub fn add_term(
        &mut self,
        term: impl Into<String>,
        definition: impl Into<String>,
        tier: PrimitiveTier,
    ) {
        let t = term.into();
        let d = definition.into();
        self.terms.insert(t.clone(), d.clone());
        self.tier_assignments.insert(t.clone(), tier);
        self.glossary.push(GlossaryEntry {
            term: t,
            definition: d,
            preliminary_tier: tier,
            source_phenomena: Vec::new(),
        });
    }
}

// ============================================================================
// Stage 3: DECOMPOSE (Analysis)
// ============================================================================

/// Stage 3: Extract primitives via DAG analysis.
#[derive(Debug, Clone, Default)]
pub struct DecomposeStage;

/// Dependency graph for primitives.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Edges: primitive → list of primitives it depends on.
    pub edges: HashMap<String, Vec<String>>,
    /// In-degree for each node.
    pub in_degrees: HashMap<String, usize>,
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an edge: `from` depends on `to`.
    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        let f = from.into();
        let t = to.into();
        self.edges.entry(f).or_default().push(t.clone());
        *self.in_degrees.entry(t).or_default() += 1;
    }

    /// Returns root nodes (in-degree = 0).
    pub fn roots(&self) -> Vec<&String> {
        self.edges
            .keys()
            .filter(|k| self.in_degrees.get(*k).copied().unwrap_or(0) == 0)
            .collect()
    }
}

/// Distribution of primitives across levels (from topological sort).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LevelDistribution {
    /// Level → list of primitive names at that level.
    pub levels: HashMap<usize, Vec<String>>,
    /// Maximum depth.
    pub max_depth: usize,
}

/// Output of Stage 3: DECOMPOSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Domain.
    pub domain: String,
    /// Extracted primitives by tier.
    pub primitives: PrimitivesByTier,
    /// Dependency graph.
    pub dependency_graph: DependencyGraph,
    /// Level distribution from topological sort.
    pub level_distribution: LevelDistribution,
    /// Tier counts.
    pub tier_counts: TierCounts,
    /// Validation metrics.
    pub validation: super::ValidationMetrics,
}

// ============================================================================
// Stage 4: COMPOSE (Synthesis)
// ============================================================================

/// Stage 4: Synthesize novel structures from primitives.
#[derive(Debug, Clone, Default)]
pub struct ComposeStage;

/// A composite structure built from primitives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeStructure {
    /// Name of the composite.
    pub name: String,
    /// Component primitives.
    pub components: Vec<String>,
    /// How components are combined.
    pub composition_rule: String,
    /// Confidence in validity.
    pub confidence: f64,
}

/// A reusable composition template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template name.
    pub name: String,
    /// Required primitive slots.
    pub slots: Vec<String>,
    /// Template pattern.
    pub pattern: String,
}

/// A grammar rule for composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRule {
    /// Left-hand side (non-terminal).
    pub lhs: String,
    /// Right-hand side (production).
    pub rhs: Vec<String>,
    /// Semantic action.
    #[serde(default)]
    pub action: Option<String>,
}

/// Output of Stage 4: COMPOSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionResult {
    /// Domain.
    pub domain: String,
    /// Synthesized composites.
    pub composites: Vec<CompositeStructure>,
    /// Reusable templates.
    pub templates: Vec<Template>,
    /// Grammar rules.
    pub grammar_rules: Vec<GrammarRule>,
}

// ============================================================================
// Stage 5: TRANSLATE (Transfer)
// ============================================================================

/// Stage 5: Bidirectional cross-domain translation.
/// Uses types from `crate::domain_discovery::translation`.
#[derive(Debug, Clone, Default)]
pub struct TranslateStage;

// ============================================================================
// Stage 6: VALIDATE (Verification)
// ============================================================================

/// Stage 6: Verify translation quality.
#[derive(Debug, Clone, Default)]
pub struct ValidateStage;

/// Validation metrics for extraction/translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Coverage: proportion of concepts expressible from primitives.
    pub coverage: f64,
    /// Minimality: absence of redundant primitives.
    pub minimality: f64,
    /// Independence: absence of implied relationships.
    pub independence: f64,
    /// Whether validation passes all thresholds.
    pub is_valid: bool,
}

impl ValidationMetrics {
    /// Creates new metrics with validation check.
    #[must_use]
    pub fn new(coverage: f64, minimality: f64, independence: f64) -> Self {
        let is_valid = coverage >= super::DEFAULT_COVERAGE_THRESHOLD
            && minimality >= super::DEFAULT_MINIMALITY_THRESHOLD
            && independence >= super::DEFAULT_INDEPENDENCE_THRESHOLD;
        Self {
            coverage,
            minimality,
            independence,
            is_valid,
        }
    }
}

impl Default for ValidationMetrics {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// Output of Stage 6: VALIDATE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Core metrics.
    pub metrics: ValidationMetrics,
    /// Consistency check passed.
    pub consistency_score: f64,
    /// Completeness check passed.
    pub completeness_score: f64,
    /// Implementability check passed.
    pub implementability_score: f64,
    /// Falsifiability check passed.
    pub falsifiability_score: f64,
    /// Issues found.
    #[serde(default)]
    pub discrepancies: Vec<String>,
}

// ============================================================================
// Stage 7: DEPLOY (Implementation)
// ============================================================================

/// Stage 7: Generate operational artifacts.
#[derive(Debug, Clone, Default)]
pub struct DeployStage;

/// An operational artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact name.
    pub name: String,
    /// Artifact type (documentation, code, config, etc.).
    pub artifact_type: String,
    /// Content or path.
    pub content: String,
}

/// A monitoring metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name.
    pub name: String,
    /// How to measure.
    pub measurement: String,
    /// Target value.
    #[serde(default)]
    pub target: Option<f64>,
}

/// An edge case to handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCase {
    /// Description.
    pub description: String,
    /// Handling strategy.
    pub handling: String,
}

/// A feedback collection channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackChannel {
    /// Channel name.
    pub name: String,
    /// Channel type (webhook, log, etc.).
    pub channel_type: String,
    /// Configuration.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// Output of Stage 7: DEPLOY.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    /// Generated artifacts.
    pub artifacts: Vec<Artifact>,
    /// Monitoring metrics.
    pub monitoring_metrics: Vec<Metric>,
    /// Edge cases documented.
    pub edge_cases: Vec<EdgeCase>,
    /// Feedback channels enabled.
    pub feedback_channels: Vec<FeedbackChannel>,
}

// ============================================================================
// Stage 8: IMPROVE (Iteration)
// ============================================================================

/// Stage 8: Aggregate feedback for next cycle.
#[derive(Debug, Clone, Default)]
pub struct ImproveStage;

/// An improvement vector identified from feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementVector {
    /// What to improve.
    pub target: String,
    /// Which stage to improve.
    pub target_stage: StageId,
    /// Expected impact (0.0-1.0).
    pub impact: f64,
    /// Estimated effort (0.0-1.0).
    pub effort: f64,
    /// Impact/effort ratio.
    pub priority_score: f64,
}

impl ImprovementVector {
    /// Creates a new improvement vector with priority calculation.
    #[must_use]
    pub fn new(target: impl Into<String>, target_stage: StageId, impact: f64, effort: f64) -> Self {
        let priority_score = if effort > 0.0 { impact / effort } else { 0.0 };
        Self {
            target: target.into(),
            target_stage,
            impact,
            effort,
            priority_score,
        }
    }
}

/// Output of Stage 8: IMPROVE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPlan {
    /// Aggregated feedback signals.
    pub feedback_signals: Vec<super::FeedbackSignal>,
    /// Identified improvement vectors.
    pub improvement_vectors: Vec<ImprovementVector>,
    /// Priority order (indices into improvement_vectors).
    pub priority_order: Vec<usize>,
    /// New inputs to generate for next SEE cycle.
    pub new_cycle_inputs: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_id_sequence() {
        assert_eq!(StageId::See.number(), 1);
        assert_eq!(StageId::Improve.number(), 8);
        assert_eq!(StageId::See.next(), Some(StageId::Speak));
        assert_eq!(StageId::Improve.next(), None);
    }

    #[test]
    fn test_validation_metrics() {
        let valid = ValidationMetrics::new(0.98, 0.95, 0.92);
        assert!(valid.is_valid);

        let invalid = ValidationMetrics::new(0.80, 0.95, 0.92);
        assert!(!invalid.is_valid);
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("risk", "causation");
        graph.add_edge("risk", "threshold");

        assert!(graph.edges.contains_key("risk"));
        assert_eq!(graph.edges["risk"].len(), 2);
    }
}
