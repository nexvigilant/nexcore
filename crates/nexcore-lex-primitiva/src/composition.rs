//! # Primitive Composition Algebra
//!
//! Operators for combining and manipulating primitive compositions.
//!
//! ## Operators
//!
//! - `+` (Union): Combine primitives from both compositions
//! - `∘` (Compose): Sequential application
//! - `∩` (Intersect): Common primitives only
//! - `-` (Difference): Primitives in A but not B
//!
//! ## Tier: T2-P (Mapping + Sum + Causality)

use crate::grammar::{InteractionGraph, PatternRegistry};
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::state_mode::StateMode;
use crate::tier::Tier;
use crate::weighted::WeightedComposition;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Result of a composition operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompositionResult {
    /// The resulting composition.
    pub composition: PrimitiveComposition,
    /// Operation that produced this result.
    pub operation: CompositionOp,
    /// Tier of the result.
    pub tier: Tier,
    /// Whether the operation changed the composition.
    pub changed: bool,
}

/// Composition operation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CompositionOp {
    /// Union: A ∪ B
    Union,
    /// Intersection: A ∩ B
    Intersect,
    /// Difference: A - B
    Difference,
    /// Sequential composition: A ∘ B
    Compose,
    /// Projection to specific primitives
    Project,
    /// Filter by predicate
    Filter,
}

impl std::fmt::Display for CompositionOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sym = match self {
            Self::Union => "∪",
            Self::Intersect => "∩",
            Self::Difference => "−",
            Self::Compose => "∘",
            Self::Project => "π",
            Self::Filter => "σ",
        };
        write!(f, "{}", sym)
    }
}

/// Algebra for primitive compositions.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CompositionAlgebra;

impl CompositionAlgebra {
    /// Create a new algebra instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Union: combine primitives from both compositions.
    #[must_use]
    pub fn union(&self, a: &PrimitiveComposition, b: &PrimitiveComposition) -> CompositionResult {
        let a_set = a.unique();
        let b_set = b.unique();
        let combined: Vec<LexPrimitiva> = a_set.union(&b_set).copied().collect();
        let changed = combined.len() != a.primitives.len();
        let composition = self.build_composition(combined, a.dominant);
        CompositionResult {
            composition: composition.clone(),
            operation: CompositionOp::Union,
            tier: Tier::classify(&composition),
            changed,
        }
    }

    /// Intersection: common primitives only.
    #[must_use]
    pub fn intersect(
        &self,
        a: &PrimitiveComposition,
        b: &PrimitiveComposition,
    ) -> CompositionResult {
        let a_set = a.unique();
        let b_set = b.unique();
        let common: Vec<LexPrimitiva> = a_set.intersection(&b_set).copied().collect();
        let changed = common.len() != a.primitives.len();
        let dominant = a.dominant.filter(|d| common.contains(d));
        let composition = self.build_composition(common, dominant);
        CompositionResult {
            composition: composition.clone(),
            operation: CompositionOp::Intersect,
            tier: Tier::classify(&composition),
            changed,
        }
    }

    /// Difference: primitives in A but not B.
    #[must_use]
    pub fn difference(
        &self,
        a: &PrimitiveComposition,
        b: &PrimitiveComposition,
    ) -> CompositionResult {
        let a_set = a.unique();
        let b_set = b.unique();
        let diff: Vec<LexPrimitiva> = a_set.difference(&b_set).copied().collect();
        let changed = diff.len() != a.primitives.len();
        let dominant = a.dominant.filter(|d| diff.contains(d));
        let composition = self.build_composition(diff, dominant);
        CompositionResult {
            composition: composition.clone(),
            operation: CompositionOp::Difference,
            tier: Tier::classify(&composition),
            changed,
        }
    }

    /// Sequential composition: A followed by B.
    #[must_use]
    pub fn compose(&self, a: &PrimitiveComposition, b: &PrimitiveComposition) -> CompositionResult {
        // Sequential composition preserves order: A's primitives, then B's
        let mut combined = a.primitives.clone();
        combined.extend(b.primitives.iter().copied());
        let changed = combined.len() != a.primitives.len();
        // Dominant is from B (the "result" of composition)
        let dominant = b.dominant.or(a.dominant);
        let composition = self.build_composition(combined, dominant);
        CompositionResult {
            composition: composition.clone(),
            operation: CompositionOp::Compose,
            tier: Tier::classify(&composition),
            changed,
        }
    }

    /// Project to specific primitives.
    #[must_use]
    pub fn project(&self, comp: &PrimitiveComposition, keep: &[LexPrimitiva]) -> CompositionResult {
        let keep_set: BTreeSet<_> = keep.iter().copied().collect();
        let projected: Vec<LexPrimitiva> = comp
            .primitives
            .iter()
            .copied()
            .filter(|p| keep_set.contains(p))
            .collect();
        let changed = projected.len() != comp.primitives.len();
        let dominant = comp.dominant.filter(|d| keep_set.contains(d));
        let composition = self.build_composition(projected, dominant);
        CompositionResult {
            composition: composition.clone(),
            operation: CompositionOp::Project,
            tier: Tier::classify(&composition),
            changed,
        }
    }

    /// Filter by tier.
    #[must_use]
    pub fn filter_by_tier(&self, comp: &PrimitiveComposition, max_tier: Tier) -> CompositionResult {
        // Keep only primitives where single-primitive tier is <= max_tier
        let filtered: Vec<LexPrimitiva> = comp
            .primitives
            .iter()
            .copied()
            .filter(|p| {
                let single = PrimitiveComposition::new(vec![*p]);
                Tier::classify(&single) <= max_tier
            })
            .collect();
        let changed = filtered.len() != comp.primitives.len();
        let composition = self.build_composition(filtered, comp.dominant);
        CompositionResult {
            composition: composition.clone(),
            operation: CompositionOp::Filter,
            tier: Tier::classify(&composition),
            changed,
        }
    }

    /// Build composition with optional dominant.
    fn build_composition(
        &self,
        primitives: Vec<LexPrimitiva>,
        dominant: Option<LexPrimitiva>,
    ) -> PrimitiveComposition {
        let comp = PrimitiveComposition::new(primitives);
        match dominant {
            Some(dom) => {
                let conf = comp.confidence;
                comp.with_dominant(dom, conf)
            }
            None => comp,
        }
    }

    /// Check if composition is valid (non-empty, no duplicates).
    #[must_use]
    pub fn is_valid(&self, comp: &PrimitiveComposition) -> ValidationResult {
        let mut issues = Vec::new();

        if comp.primitives.is_empty() {
            issues.push("Empty composition".to_string());
        }

        // Check for dominant validity
        if let Some(dom) = comp.dominant {
            if !comp.primitives.contains(&dom) {
                issues.push(format!("Dominant {:?} not in primitives", dom));
            }
        }

        // Check confidence range
        if !(0.0..=1.0).contains(&comp.confidence) {
            issues.push(format!("Invalid confidence: {}", comp.confidence));
        }

        ValidationResult {
            valid: issues.is_empty(),
            issues,
        }
    }

    /// Normalize: deduplicate and sort.
    #[must_use]
    pub fn normalize(&self, comp: &PrimitiveComposition) -> PrimitiveComposition {
        let unique: BTreeSet<_> = comp.unique();
        let mut sorted: Vec<_> = unique.into_iter().collect();
        sorted.sort_by_key(|p: &LexPrimitiva| p.symbol());
        let mut result = PrimitiveComposition::new(sorted);
        if let Some(dom) = comp.dominant {
            result = result.with_dominant(dom, comp.confidence);
        }
        result
    }

    /// Validate a composition's semantic coherence.
    ///
    /// Beyond structural validity (non-empty, valid dominant), semantic
    /// validation checks that the primitives have defined interactions
    /// and form a coherent composition.
    ///
    /// Returns a `SemanticValidation` with a coherence score (0.0-1.0)
    /// and diagnostic notes.
    #[must_use]
    pub fn validate_semantics(&self, comp: &PrimitiveComposition) -> SemanticValidation {
        let structural = self.is_valid(comp);
        if !structural.valid {
            return SemanticValidation {
                coherence: 0.0,
                structural_valid: false,
                interaction_coverage: 0.0,
                pattern_match: None,
                notes: structural.issues,
            };
        }

        let graph = InteractionGraph::canonical();
        let unique: Vec<LexPrimitiva> = comp.unique().into_iter().collect();
        let pair_count = if unique.len() > 1 {
            unique.len().saturating_mul(unique.len().saturating_sub(1))
        } else {
            1 // Single primitive is trivially coherent
        };

        // Count how many primitive pairs have defined interactions
        let mut interaction_count: usize = 0;
        for &src in &unique {
            for &tgt in &unique {
                if src == tgt {
                    continue;
                }
                if graph.lookup(src, tgt).is_some() {
                    interaction_count = interaction_count.saturating_add(1);
                }
            }
        }

        #[allow(
            clippy::as_conversions,
            reason = "interaction_count and pair_count bounded by 16*15=240, safe cast to f64"
        )]
        let interaction_coverage = interaction_count as f64 / pair_count as f64;

        // Check for pattern match
        let registry = PatternRegistry::canonical();
        let matches = registry.find_matches(comp);
        let pattern_match = matches.first().map(|p| p.name.clone());

        // Coherence = weighted average of coverage + pattern bonus
        let pattern_bonus = if pattern_match.is_some() { 0.2 } else { 0.0 };
        let coherence = (interaction_coverage * 0.8 + pattern_bonus).min(1.0);

        let mut notes = Vec::new();
        if interaction_coverage < 0.3 && unique.len() > 1 {
            notes.push("Low interaction coverage — primitives may be loosely related".to_string());
        }
        if let Some(ref name) = pattern_match {
            notes.push(format!("Matches canonical pattern: {}", name));
        }

        SemanticValidation {
            coherence,
            structural_valid: true,
            interaction_coverage,
            pattern_match,
            notes,
        }
    }

    /// Canonicalize a composition to its simplest equivalent form.
    ///
    /// 1. Deduplicates and sorts
    /// 2. If close to a canonical pattern, adopts the pattern's form
    /// 3. Preserves dominant and confidence
    #[must_use]
    pub fn canonicalize(&self, comp: &PrimitiveComposition) -> PrimitiveComposition {
        let wc = WeightedComposition::from_composition(comp);
        wc.canonical_form()
    }

    /// Score a composition across three dimensions.
    ///
    /// - **Expressiveness**: How many distinct primitives (breadth)
    /// - **Compactness**: Inverse of redundancy (no duplicates = 1.0)
    /// - **Coherence**: Semantic validation score
    #[must_use]
    pub fn score(&self, comp: &PrimitiveComposition) -> CompositionScore {
        let unique_count = comp.unique().len();
        let total = comp.primitives.len().max(1);

        // Expressiveness: unique count / 16 (max possible)
        #[allow(
            clippy::as_conversions,
            reason = "unique_count bounded by 16, safe cast to f64"
        )]
        let expressiveness = unique_count as f64 / 16.0;

        // Compactness: unique / total (1.0 = no duplicates)
        #[allow(
            clippy::as_conversions,
            reason = "unique_count and total bounded by primitive count, safe cast to f64"
        )]
        let compactness = unique_count as f64 / total as f64;

        // Coherence from semantic validation
        let semantic = self.validate_semantics(comp);
        let coherence = semantic.coherence;

        CompositionScore {
            expressiveness,
            compactness,
            coherence,
            overall: (expressiveness + compactness + coherence) / 3.0,
        }
    }

    /// Find the best matching canonical pattern for a composition.
    #[must_use]
    pub fn find_pattern(&self, comp: &PrimitiveComposition) -> Option<String> {
        let registry = PatternRegistry::canonical();
        registry
            .closest(comp)
            .filter(|(_, distance)| *distance < 0.5)
            .map(|(pattern, _)| pattern.name.clone())
    }
}

/// Result of validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidationResult {
    /// Whether the composition is valid.
    pub valid: bool,
    /// Issues found (empty if valid).
    pub issues: Vec<String>,
}

/// Semantic validation result with coherence scoring.
///
/// Tier: T2-C (Comparison + Quantity + Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SemanticValidation {
    /// Overall coherence score (0.0-1.0).
    pub coherence: f64,
    /// Whether the composition passes structural validation.
    pub structural_valid: bool,
    /// Fraction of primitive pairs with defined interactions.
    pub interaction_coverage: f64,
    /// Name of matched canonical pattern, if any.
    pub pattern_match: Option<String>,
    /// Diagnostic notes.
    pub notes: Vec<String>,
}

impl std::fmt::Display for SemanticValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "coherence={:.2} coverage={:.0}% pattern={}",
            self.coherence,
            self.interaction_coverage * 100.0,
            self.pattern_match.as_deref().unwrap_or("none"),
        )
    }
}

/// Multi-dimensional composition quality score.
///
/// Tier: T2-P (Quantity + Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompositionScore {
    /// Breadth of primitives used (0.0-1.0, where 1.0 = all 16).
    pub expressiveness: f64,
    /// Absence of redundancy (1.0 = no duplicates).
    pub compactness: f64,
    /// Semantic coherence from interaction analysis (0.0-1.0).
    pub coherence: f64,
    /// Average of all three dimensions.
    pub overall: f64,
}

impl std::fmt::Display for CompositionScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expr={:.2} compact={:.2} cohere={:.2} overall={:.2}",
            self.expressiveness, self.compactness, self.coherence, self.overall,
        )
    }
}

/// Builder for creating compositions fluently.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CompositionBuilder {
    primitives: Vec<LexPrimitiva>,
    dominant: Option<LexPrimitiva>,
    confidence: f64,
    state_mode: Option<StateMode>,
}

impl CompositionBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
            dominant: None,
            confidence: 1.0,
            state_mode: None,
        }
    }

    /// Set the state mode.
    #[must_use]
    pub fn state_mode(mut self, mode: StateMode) -> Self {
        self.state_mode = Some(mode);
        self
    }

    /// Add a primitive.
    #[must_use]
    pub fn add(mut self, primitive: LexPrimitiva) -> Self {
        self.primitives.push(primitive);
        self
    }

    /// Add multiple primitives.
    #[must_use]
    pub fn add_all(mut self, primitives: &[LexPrimitiva]) -> Self {
        self.primitives.extend(primitives.iter().copied());
        self
    }

    /// Set the dominant primitive.
    #[must_use]
    pub fn dominant(mut self, primitive: LexPrimitiva) -> Self {
        self.dominant = Some(primitive);
        self
    }

    /// Set confidence.
    #[must_use]
    pub fn confidence(mut self, conf: f64) -> Self {
        self.confidence = conf.clamp(0.0, 1.0);
        self
    }

    /// Build the composition.
    #[must_use]
    pub fn build(self) -> PrimitiveComposition {
        let mut comp = PrimitiveComposition::new(self.primitives);
        if let Some(dom) = self.dominant {
            comp = comp.with_dominant(dom, self.confidence);
        } else {
            comp.confidence = self.confidence;
        }
        if let Some(mode) = self.state_mode {
            comp = comp.with_state_mode(mode);
        }
        comp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union() {
        let algebra = CompositionAlgebra::new();
        let a = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        let b = PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::State]);
        let result = algebra.union(&a, &b);
        assert_eq!(result.composition.unique().len(), 3);
        assert_eq!(result.operation, CompositionOp::Union);
    }

    #[test]
    fn test_intersect() {
        let algebra = CompositionAlgebra::new();
        let a = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        let b = PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::State]);
        let result = algebra.intersect(&a, &b);
        assert_eq!(result.composition.unique().len(), 1);
        assert!(
            result
                .composition
                .primitives
                .contains(&LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn test_difference() {
        let algebra = CompositionAlgebra::new();
        let a = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        let b = PrimitiveComposition::new(vec![LexPrimitiva::Mapping]);
        let result = algebra.difference(&a, &b);
        assert_eq!(result.composition.unique().len(), 1);
        assert!(
            result
                .composition
                .primitives
                .contains(&LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn test_compose() {
        let algebra = CompositionAlgebra::new();
        let a = PrimitiveComposition::new(vec![LexPrimitiva::Sequence]);
        let b = PrimitiveComposition::new(vec![LexPrimitiva::Mapping]);
        let result = algebra.compose(&a, &b);
        assert_eq!(result.composition.primitives.len(), 2);
    }

    #[test]
    fn test_project() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ]);
        let result = algebra.project(&comp, &[LexPrimitiva::Sequence, LexPrimitiva::State]);
        assert_eq!(result.composition.unique().len(), 2);
        assert!(
            !result
                .composition
                .primitives
                .contains(&LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn test_validation_valid() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let result = algebra.is_valid(&comp);
        assert!(result.valid);
    }

    #[test]
    fn test_validation_empty() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![]);
        let result = algebra.is_valid(&comp);
        assert!(!result.valid);
    }

    #[test]
    fn test_normalize() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
        ]);
        let normalized = algebra.normalize(&comp);
        assert_eq!(normalized.unique().len(), 2);
    }

    #[test]
    fn test_builder() {
        let comp = CompositionBuilder::new()
            .add(LexPrimitiva::Sequence)
            .add(LexPrimitiva::Mapping)
            .dominant(LexPrimitiva::Sequence)
            .confidence(0.9)
            .build();

        assert_eq!(comp.primitives.len(), 2);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_operation_display() {
        assert_eq!(format!("{}", CompositionOp::Union), "∪");
        assert_eq!(format!("{}", CompositionOp::Intersect), "∩");
        assert_eq!(format!("{}", CompositionOp::Compose), "∘");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SEMANTIC VALIDATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_semantics_coherent() {
        let algebra = CompositionAlgebra::new();
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison]);
        let result = algebra.validate_semantics(&comp);
        assert!(result.structural_valid);
        assert!(result.coherence > 0.0);
        assert!(result.interaction_coverage > 0.0);
    }

    #[test]
    fn test_validate_semantics_with_pattern() {
        let algebra = CompositionAlgebra::new();
        // Gatekeeper: ∂ + κ
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison]);
        let result = algebra.validate_semantics(&comp);
        assert!(result.pattern_match.is_some());
        assert_eq!(result.pattern_match.as_deref(), Some("Gatekeeper"));
    }

    #[test]
    fn test_validate_semantics_empty() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![]);
        let result = algebra.validate_semantics(&comp);
        assert!(!result.structural_valid);
        assert!(result.coherence.abs() < f64::EPSILON);
    }

    #[test]
    fn test_validate_semantics_single() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let result = algebra.validate_semantics(&comp);
        assert!(result.structural_valid);
        // Single primitive has trivial coherence
    }

    #[test]
    fn test_validate_semantics_display() {
        let algebra = CompositionAlgebra::new();
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison]);
        let result = algebra.validate_semantics(&comp);
        let s = format!("{}", result);
        assert!(s.contains("coherence="));
        assert!(s.contains("coverage="));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CANONICALIZE TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_canonicalize_near_pattern() {
        let algebra = CompositionAlgebra::new();
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison])
                .with_dominant(LexPrimitiva::Boundary, 0.9);
        let canonical = algebra.canonicalize(&comp);
        let unique = canonical.unique();
        assert!(unique.contains(&LexPrimitiva::Boundary));
        assert!(unique.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn test_canonicalize_novel() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Persistence,
            LexPrimitiva::Location,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Product,
            LexPrimitiva::Sum,
        ]);
        let canonical = algebra.canonicalize(&comp);
        // Should normalize — sorted, deduplicated
        assert_eq!(canonical.unique().len(), 6);
    }

    #[test]
    fn test_canonicalize_deduplicates() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
        ]);
        let canonical = algebra.canonicalize(&comp);
        assert_eq!(canonical.unique().len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SCORE TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_score_ranges() {
        let algebra = CompositionAlgebra::new();
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison]);
        let score = algebra.score(&comp);
        assert!(score.expressiveness >= 0.0 && score.expressiveness <= 1.0);
        assert!(score.compactness >= 0.0 && score.compactness <= 1.0);
        assert!(score.coherence >= 0.0 && score.coherence <= 1.0);
        assert!(score.overall >= 0.0 && score.overall <= 1.0);
    }

    #[test]
    fn test_score_perfect_compactness() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        let score = algebra.score(&comp);
        assert!((score.compactness - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_score_low_compactness() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
        ]);
        let score = algebra.score(&comp);
        assert!(score.compactness < 0.5);
    }

    #[test]
    fn test_score_display() {
        let score = CompositionScore {
            expressiveness: 0.5,
            compactness: 1.0,
            coherence: 0.8,
            overall: 0.77,
        };
        let s = format!("{}", score);
        assert!(s.contains("expr="));
        assert!(s.contains("compact="));
        assert!(s.contains("cohere="));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FIND PATTERN TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_find_pattern_gatekeeper() {
        let algebra = CompositionAlgebra::new();
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison]);
        let pattern = algebra.find_pattern(&comp);
        assert_eq!(pattern.as_deref(), Some("Gatekeeper"));
    }

    #[test]
    fn test_find_pattern_none() {
        let algebra = CompositionAlgebra::new();
        // Far from any pattern
        let comp = PrimitiveComposition::new(LexPrimitiva::all().to_vec());
        let pattern = algebra.find_pattern(&comp);
        // All 16 primitives — distance > 0.5 from any 2-3 primitive pattern
        assert!(pattern.is_none());
    }

    #[test]
    fn test_find_pattern_pipeline() {
        let algebra = CompositionAlgebra::new();
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ]);
        let pattern = algebra.find_pattern(&comp);
        assert_eq!(pattern.as_deref(), Some("Pipeline"));
    }
}
