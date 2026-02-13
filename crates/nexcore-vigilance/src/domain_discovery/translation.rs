//! # Translation Types
//!
//! Type-safe representations of cross-domain translations built via the
//! Domain Discovery Framework (Phase 5: TRANSLATE).
//!
//! ## Mapping Types
//!
//! | Type | Confidence | Description |
//! |------|------------|-------------|
//! | Identical | 1.0 | T1 universal - same concept in both domains |
//! | Analogous | 0.7-0.95 | T2 cross-domain - structural equivalence |
//! | Novel | 0.5-0.85 | T3 synthesized - creative mapping |

use serde::{Deserialize, Serialize};

/// Type of mapping between source and target concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MappingType {
    /// Identical concept in both domains (T1 universal).
    /// Confidence = 1.0.
    Identical,

    /// Analogous concept with structural equivalence (T2 cross-domain).
    /// Confidence typically 0.7-0.95.
    Analogous,

    /// Novel synthesis required (T3 domain-specific).
    /// Confidence typically 0.5-0.85.
    Novel,
}

impl MappingType {
    /// Returns expected confidence range for this mapping type.
    #[must_use]
    pub const fn confidence_range(&self) -> (f64, f64) {
        match self {
            Self::Identical => (1.0, 1.0),
            Self::Analogous => (0.70, 0.95),
            Self::Novel => (0.50, 0.85),
        }
    }

    /// Checks if a confidence value is valid for this mapping type.
    #[must_use]
    pub fn is_valid_confidence(&self, confidence: f64) -> bool {
        let (min, max) = self.confidence_range();
        confidence >= min && confidence <= max
    }
}

/// A T1 universal mapping (identity, confidence = 1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalMapping {
    /// Source concept name.
    pub source: String,
    /// Target concept name (usually same as source).
    pub target: String,
    /// Confidence score (always 1.0 for T1).
    pub confidence: f64,
    /// Mapping type (always Identical for T1).
    pub mapping_type: MappingType,
    /// Example in source domain.
    #[serde(default)]
    pub source_example: Option<String>,
    /// Example in target domain.
    #[serde(default)]
    pub target_example: Option<String>,
}

impl UniversalMapping {
    /// Creates a new T1 universal mapping.
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            confidence: 1.0,
            mapping_type: MappingType::Identical,
            source_example: None,
            target_example: None,
        }
    }

    /// Creates an identity mapping (source = target).
    #[must_use]
    pub fn identity(concept: impl Into<String>) -> Self {
        let name = concept.into();
        Self::new(name.clone(), name)
    }
}

/// Source or target concept in a transfer mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConcept {
    /// Concept name.
    pub name: String,
    /// Primitive tier.
    #[serde(default)]
    pub tier: Option<super::PrimitiveTier>,
    /// Meaning in this domain.
    #[serde(default)]
    pub meaning: Option<String>,
}

impl TransferConcept {
    /// Creates a new transfer concept.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tier: None,
            meaning: None,
        }
    }

    /// Adds tier information.
    #[must_use]
    pub fn with_tier(mut self, tier: super::PrimitiveTier) -> Self {
        self.tier = Some(tier);
        self
    }

    /// Adds meaning.
    #[must_use]
    pub fn with_meaning(mut self, meaning: impl Into<String>) -> Self {
        self.meaning = Some(meaning.into());
        self
    }
}

/// A T2 cross-domain transfer mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMapping {
    /// Source concept.
    pub source: TransferConcept,
    /// Target concept.
    pub target: TransferConcept,
    /// Confidence score (0.7-0.95 typical).
    pub confidence: f64,
    /// Mapping type (usually Analogous).
    pub mapping_type: MappingType,
    /// Rationale for the mapping.
    #[serde(default)]
    pub rationale: Option<String>,
}

impl TransferMapping {
    /// Creates a new transfer mapping.
    #[must_use]
    pub fn new(source: TransferConcept, target: TransferConcept, confidence: f64) -> Self {
        Self {
            source,
            target,
            confidence: confidence.clamp(0.0, 1.0),
            mapping_type: MappingType::Analogous,
            rationale: None,
        }
    }

    /// Adds rationale.
    #[must_use]
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }
}

/// A T3 novel synthesis mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelSynthesis {
    /// Source primitive name.
    pub source_primitive: String,
    /// Primitive tier (always T3).
    #[serde(default)]
    pub tier: Option<super::PrimitiveTier>,
    /// Meaning in source domain.
    #[serde(default)]
    pub source_meaning: Option<String>,
    /// Synthesized target expression.
    pub synthesized_target: String,
    /// How the synthesis was performed.
    #[serde(default)]
    pub synthesis_method: Option<String>,
    /// Confidence score.
    pub confidence: f64,
    /// Detailed mapping (for complex types like harm_type).
    #[serde(default)]
    pub mapping_detail: Option<serde_json::Value>,
}

impl NovelSynthesis {
    /// Creates a new novel synthesis.
    #[must_use]
    pub fn new(
        source_primitive: impl Into<String>,
        synthesized_target: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            source_primitive: source_primitive.into(),
            tier: Some(super::PrimitiveTier::T3DomainSpecific),
            source_meaning: None,
            synthesized_target: synthesized_target.into(),
            synthesis_method: None,
            confidence: confidence.clamp(0.0, 1.0),
            mapping_detail: None,
        }
    }
}

/// A reverse mapping (target → source).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseMapping {
    /// Concept in target domain.
    pub target_concept: String,
    /// Equivalent in source domain.
    pub source_equivalent: String,
    /// Confidence score.
    pub confidence: f64,
    /// Rationale for reverse mapping.
    #[serde(default)]
    pub rationale: Option<String>,
}

impl ReverseMapping {
    /// Creates a new reverse mapping.
    #[must_use]
    pub fn new(
        target_concept: impl Into<String>,
        source_equivalent: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            target_concept: target_concept.into(),
            source_equivalent: source_equivalent.into(),
            confidence: confidence.clamp(0.0, 1.0),
            rationale: None,
        }
    }
}

/// Feasibility score for cross-domain translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibilityScore {
    /// Overall feasibility (weighted average).
    pub overall: f64,
    /// T1 universal score (always 1.0 if T1 primitives exist).
    pub t1_score: f64,
    /// T2 cross-domain transferability score.
    pub t2_score: f64,
    /// T3 domain-specific synthesis score.
    pub t3_score: f64,
    /// Concepts that couldn't be mapped (barriers).
    #[serde(default)]
    pub barriers: Vec<String>,
    /// Recommended target domains for this source.
    #[serde(default)]
    pub recommended_targets: Vec<String>,
}

impl FeasibilityScore {
    /// Checks if translation is feasible (overall ≥ 0.70).
    #[must_use]
    pub fn is_feasible(&self) -> bool {
        self.overall >= 0.70
    }

    /// Checks if translation is highly feasible (overall ≥ 0.85).
    #[must_use]
    pub fn is_highly_feasible(&self) -> bool {
        self.overall >= 0.85
    }

    /// Returns the weakest tier score.
    #[must_use]
    pub fn weakest_tier(&self) -> (&'static str, f64) {
        let scores = [
            ("T1", self.t1_score),
            ("T2", self.t2_score),
            ("T3", self.t3_score),
        ];
        // SAFETY INVARIANT: scores array has exactly 3 elements (T1, T2, T3), always non-empty
        scores
            .into_iter()
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or(("T1", 0.0))
    }
}

/// Translation validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationValidation {
    /// Whether translation is valid.
    pub is_valid: bool,
    /// Coverage score (mapped concepts / total concepts).
    pub coverage_score: f64,
    /// Consistency score (no contradictions).
    pub consistency_score: f64,
    /// Grammar coverage (production rules translate).
    #[serde(default)]
    pub grammar_coverage: Option<f64>,
    /// Bidirectionality score (reverse mappings available).
    #[serde(default)]
    pub bidirectionality: Option<f64>,
    /// Unmapped concepts by direction.
    #[serde(default)]
    pub unmapped_concepts: Option<UnmappedConcepts>,
}

/// Concepts that couldn't be mapped in either direction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnmappedConcepts {
    /// Source concepts without target mapping.
    #[serde(default)]
    pub source_to_target: Vec<String>,
    /// Target concepts without source mapping.
    #[serde(default)]
    pub target_to_source: Vec<String>,
}

/// Complete translation record (Phase 5 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRecord {
    /// Source domain name.
    pub source_domain: String,
    /// Target domain name.
    pub target_domain: String,
    /// Translation version.
    #[serde(default)]
    pub version: Option<String>,

    /// T1 universal mappings (identity).
    #[serde(default)]
    pub t1_mappings: Vec<UniversalMapping>,

    /// T2 cross-domain transfer mappings.
    #[serde(default)]
    pub transfer_mappings: Vec<TransferMapping>,

    /// T3 novel synthesis mappings.
    #[serde(default)]
    pub novel_synthesis: Vec<NovelSynthesis>,

    /// Reverse mappings (target → source).
    #[serde(default)]
    pub reverse_mappings: Vec<ReverseMapping>,

    /// Feasibility assessment.
    #[serde(default)]
    pub feasibility: Option<FeasibilityScore>,

    /// Validation results.
    #[serde(default)]
    pub validation: Option<TranslationValidation>,
}

impl TranslationRecord {
    /// Loads translation from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let result: Self = serde_yaml::from_str(&content)?;
        Ok(result)
    }

    /// Total mapping count across all tiers.
    #[must_use]
    pub fn total_mappings(&self) -> usize {
        self.t1_mappings.len() + self.transfer_mappings.len() + self.novel_synthesis.len()
    }

    /// Average confidence across all mappings.
    #[must_use]
    pub fn avg_confidence(&self) -> f64 {
        let mut sum = 0.0;
        let mut count = 0;

        for m in &self.t1_mappings {
            sum += m.confidence;
            count += 1;
        }
        for m in &self.transfer_mappings {
            sum += m.confidence;
            count += 1;
        }
        for m in &self.novel_synthesis {
            sum += m.confidence;
            count += 1;
        }

        if count == 0 { 0.0 } else { sum / count as f64 }
    }

    /// Finds a mapping for a source concept.
    #[must_use]
    pub fn find_mapping(&self, source_name: &str) -> Option<(&str, f64)> {
        // Check T1 first
        if let Some(m) = self.t1_mappings.iter().find(|m| m.source == source_name) {
            return Some((&m.target, m.confidence));
        }

        // Check T2
        if let Some(m) = self
            .transfer_mappings
            .iter()
            .find(|m| m.source.name == source_name)
        {
            return Some((&m.target.name, m.confidence));
        }

        // Check T3
        if let Some(m) = self
            .novel_synthesis
            .iter()
            .find(|m| m.source_primitive == source_name)
        {
            return Some((&m.synthesized_target, m.confidence));
        }

        None
    }

    /// Finds a reverse mapping for a target concept.
    #[must_use]
    pub fn find_reverse_mapping(&self, target_name: &str) -> Option<(&str, f64)> {
        self.reverse_mappings
            .iter()
            .find(|m| m.target_concept == target_name)
            .map(|m| (m.source_equivalent.as_str(), m.confidence))
    }

    /// Checks if translation is feasible.
    #[must_use]
    pub fn is_feasible(&self) -> bool {
        self.feasibility.as_ref().map_or(false, |f| f.is_feasible())
    }
}

/// Pre-defined harm type mappings for ToV → AI Safety.
pub mod tov_ai_safety {
    /// AI Safety equivalents for ToV harm types A-H.
    pub const HARM_TYPE_MAPPINGS: [(&str, &str, f64); 8] = [
        ("A_Acute", "prompt_injection", 0.88),
        ("B_Cumulative", "capability_overhang", 0.82),
        ("C_OffTarget", "unintended_capability", 0.80),
        ("D_Cascade", "recursive_self_improvement", 0.85),
        ("E_Idiosyncratic", "edge_case_failure", 0.90),
        ("F_Saturation", "context_overflow", 0.85),
        ("G_Interaction", "multi_model_exploit", 0.78),
        ("H_Population", "disparate_impact", 0.88),
    ];

    /// AI Safety equivalents for ToV hierarchy levels.
    pub const HIERARCHY_LEVEL_MAPPINGS: [(&str, &str); 8] = [
        ("Molecular", "Token"),
        ("Cellular", "Attention_Head"),
        ("Tissue", "Layer"),
        ("Organ", "Circuit"),
        ("System", "Model"),
        ("Organism", "Agent"),
        ("Population", "Multi_Agent"),
        ("Societal", "Civilization"),
    ];

    /// Maps a ToV harm type to AI Safety equivalent.
    #[must_use]
    pub fn map_harm_type(tov_type: &str) -> Option<(&'static str, f64)> {
        HARM_TYPE_MAPPINGS
            .iter()
            .find(|(t, _, _)| *t == tov_type)
            .map(|(_, ai, conf)| (*ai, *conf))
    }

    /// Maps a ToV hierarchy level to AI Safety equivalent.
    #[must_use]
    pub fn map_hierarchy_level(tov_level: &str) -> Option<&'static str> {
        HIERARCHY_LEVEL_MAPPINGS
            .iter()
            .find(|(t, _)| *t == tov_level)
            .map(|(_, ai)| *ai)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_type_confidence() {
        assert!(MappingType::Identical.is_valid_confidence(1.0));
        assert!(!MappingType::Identical.is_valid_confidence(0.9));

        assert!(MappingType::Analogous.is_valid_confidence(0.85));
        assert!(!MappingType::Analogous.is_valid_confidence(0.5));
    }

    #[test]
    fn test_universal_mapping() {
        let m = UniversalMapping::identity("entity");
        assert_eq!(m.source, "entity");
        assert_eq!(m.target, "entity");
        assert_eq!(m.confidence, 1.0);
    }

    #[test]
    fn test_feasibility_score() {
        let score = FeasibilityScore {
            overall: 0.86,
            t1_score: 1.0,
            t2_score: 0.85,
            t3_score: 0.82,
            barriers: vec![],
            recommended_targets: vec!["ai_safety".to_string()],
        };

        assert!(score.is_feasible());
        assert!(score.is_highly_feasible());
        assert_eq!(score.weakest_tier(), ("T3", 0.82));
    }

    #[test]
    fn test_tov_ai_safety_mappings() {
        use tov_ai_safety::*;

        assert_eq!(map_harm_type("A_Acute"), Some(("prompt_injection", 0.88)));
        assert_eq!(map_hierarchy_level("Molecular"), Some("Token"));
        assert_eq!(map_hierarchy_level("Societal"), Some("Civilization"));
    }
}
