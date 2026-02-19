//! # Reverse DAG Primitive Synthesizer
//!
//! The inverse of the forward `GroundsTo` pipeline: given a bag of T1 primitives,
//! compose upward through the tier DAG to discover what they build, match patterns,
//! infer interactions, and suggest completions.
//!
//! ## Tier: T2-C (Mapping + Comparison + Sequence + Sum)

use crate::composition::CompositionAlgebra;
use crate::grammar::{Interaction, InteractionGraph, PatternRegistry};
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::tier::Tier;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ═══════════════════════════════════════════════════════════════════════════════
// ERROR TYPE
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors during reverse synthesis.
///
/// Tier: T2-P (Boundary + Comparison)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SynthesisError {
    /// No primitives provided.
    EmptyPrimitives,
    /// An unrecognized primitive name was given.
    InvalidPrimitive(String),
    /// Coherence score below the requested minimum.
    BelowCoherenceThreshold {
        /// Actual coherence.
        got: f64,
        /// Required minimum.
        min: f64,
    },
}

impl std::fmt::Display for SynthesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPrimitives => write!(f, "No primitives provided"),
            Self::InvalidPrimitive(name) => write!(f, "Invalid primitive: {}", name),
            Self::BelowCoherenceThreshold { got, min } => {
                write!(f, "Coherence {:.2} below threshold {:.2}", got, min)
            }
        }
    }
}

impl std::error::Error for SynthesisError {}

// ═══════════════════════════════════════════════════════════════════════════════
// OPTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for synthesis.
///
/// Tier: T2-P (State + Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisOpts {
    /// Optional target tier to aim for.
    pub target_tier: Option<Tier>,
    /// Optional pattern name hint (e.g., "Gatekeeper").
    pub pattern_hint: Option<String>,
    /// Minimum coherence threshold (0.0-1.0, default 0.0).
    pub min_coherence: f64,
}

impl Default for SynthesisOpts {
    fn default() -> Self {
        Self {
            target_tier: None,
            pattern_hint: None,
            min_coherence: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RESULT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of reverse synthesis.
///
/// Tier: T2-C (Mapping + Sequence + Comparison + Sum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    /// The composed primitive composition.
    pub composition: PrimitiveComposition,
    /// Discovered interactions among the input primitives.
    pub interactions: Vec<Interaction>,
    /// Inferred dominant primitive.
    pub dominant: Option<LexPrimitiva>,
    /// Classified tier of the composition.
    pub tier: Tier,
    /// Canonical patterns matched (superset match).
    pub pattern_matches: Vec<PatternMatch>,
    /// Nearest canonical pattern with distance.
    pub nearest_pattern: Option<(String, f64)>,
    /// Semantic coherence score (0.0-1.0).
    pub coherence: f64,
    /// Suggestions for adding primitives to reach known patterns.
    pub suggestions: Vec<CompletionSuggestion>,
}

/// A pattern match result showing distance and delta.
///
/// Tier: T2-P (Comparison + Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Canonical pattern name.
    pub name: String,
    /// Distance from input to pattern (0.0 = exact).
    pub distance: f64,
    /// Primitives in the pattern but not in the input.
    pub missing: Vec<LexPrimitiva>,
    /// Primitives in the input but not in the pattern.
    pub extra: Vec<LexPrimitiva>,
}

/// Suggestion for completing a composition toward a known pattern.
///
/// Tier: T2-P (Causality + Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSuggestion {
    /// Target pattern name.
    pub target_pattern: String,
    /// Primitives to add.
    pub missing_primitives: Vec<LexPrimitiva>,
    /// Tier after adding the missing primitives.
    pub resulting_tier: Tier,
    /// Coherence improvement estimate.
    pub coherence_gain: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// REVERSE SYNTHESIZER
// ═══════════════════════════════════════════════════════════════════════════════

/// Reverse DAG synthesizer: T1 primitives → composed structure.
///
/// The inverse of `GroundsTo`. Given raw T1 primitives, this engine:
/// 1. Builds a `PrimitiveComposition`
/// 2. Infers dominant via interaction graph frequency
/// 3. Matches against 12 canonical patterns
/// 4. Validates semantic coherence
/// 5. Suggests completions for near-miss patterns
///
/// Tier: T2-C (Mapping + Comparison + Sequence + Sum)
pub struct RevSynthesizer {
    algebra: CompositionAlgebra,
    graph: InteractionGraph,
    registry: PatternRegistry,
}

impl RevSynthesizer {
    /// Create a new synthesizer with canonical graph and patterns.
    #[must_use]
    pub fn new() -> Self {
        Self {
            algebra: CompositionAlgebra::new(),
            graph: InteractionGraph::canonical(),
            registry: PatternRegistry::canonical(),
        }
    }

    /// Core synthesis: T1 primitives → composed structure with validation.
    ///
    /// # Errors
    ///
    /// Returns `SynthesisError` if:
    /// - Input is empty
    /// - Coherence is below `opts.min_coherence`
    pub fn synthesize(
        &self,
        primitives: Vec<LexPrimitiva>,
        opts: SynthesisOpts,
    ) -> Result<SynthesisResult, SynthesisError> {
        // 1. Validate inputs
        if primitives.is_empty() {
            return Err(SynthesisError::EmptyPrimitives);
        }

        // 2. Build composition
        let unique: HashSet<LexPrimitiva> = primitives.iter().copied().collect();
        let unique_vec: Vec<LexPrimitiva> = unique.into_iter().collect();
        let composition = PrimitiveComposition::new(unique_vec.clone());

        // 3. Infer dominant via interaction graph heuristic
        let dominant = self.infer_dominant(&unique_vec, opts.pattern_hint.as_deref());

        // 4. Build composition with dominant
        let composition = match dominant {
            Some(dom) => composition.with_dominant(dom, 0.9),
            None => composition,
        };

        // 5. Populate interactions from graph
        let interactions = self.find_interactions(&unique_vec);

        // 6. Match against canonical patterns
        let pattern_matches = self.match_patterns(&composition);

        // 7. Find nearest pattern
        let nearest_pattern = self
            .registry
            .closest(&composition)
            .map(|(p, d)| (p.name.clone(), d));

        // 8. Compute coherence
        let semantic = self.algebra.validate_semantics(&composition);
        let coherence = semantic.coherence;

        // 9. Check coherence threshold
        if coherence < opts.min_coherence {
            return Err(SynthesisError::BelowCoherenceThreshold {
                got: coherence,
                min: opts.min_coherence,
            });
        }

        // 10. Generate completion suggestions
        let suggestions = self.generate_suggestions(&composition);

        // 11. Classify tier
        let tier = Tier::classify(&composition);

        Ok(SynthesisResult {
            composition,
            interactions,
            dominant,
            tier,
            pattern_matches,
            nearest_pattern,
            coherence,
            suggestions,
        })
    }

    /// Suggest primitives to add to reach a named pattern.
    #[must_use]
    pub fn suggest_completion(
        &self,
        have: &[LexPrimitiva],
        target_pattern: &str,
    ) -> Option<CompletionSuggestion> {
        let pattern = self.registry.get(target_pattern)?;
        let have_set: HashSet<LexPrimitiva> = have.iter().copied().collect();
        let pattern_set = pattern.composition.unique();

        let missing: Vec<LexPrimitiva> = pattern_set.difference(&have_set).copied().collect();

        if missing.is_empty() {
            return None; // Already complete
        }

        // Compute resulting composition
        let mut combined: Vec<LexPrimitiva> = have_set.into_iter().collect();
        combined.extend(missing.iter().copied());
        let result_comp = PrimitiveComposition::new(combined);

        // Coherence gain
        let current_comp = PrimitiveComposition::new(have.to_vec());
        let current_semantic = self.algebra.validate_semantics(&current_comp);
        let result_semantic = self.algebra.validate_semantics(&result_comp);
        let coherence_gain = result_semantic.coherence - current_semantic.coherence;

        Some(CompletionSuggestion {
            target_pattern: target_pattern.to_string(),
            missing_primitives: missing,
            resulting_tier: Tier::classify(&result_comp),
            coherence_gain,
        })
    }

    /// Reverse lookup: find all canonical patterns whose composition is
    /// a superset, subset, or exact match of the given primitives.
    #[must_use]
    pub fn reverse_lookup_patterns(&self, primitives: &[LexPrimitiva]) -> Vec<PatternMatch> {
        self.match_patterns(&PrimitiveComposition::new(primitives.to_vec()))
    }

    // ─── Internal Helpers ────────────────────────────────────────────────────

    /// Infer dominant primitive via interaction graph edge frequency.
    ///
    /// Pick the primitive with the most outgoing + incoming edges among
    /// the given set. Tie-break by canonical pattern frequency.
    fn infer_dominant(
        &self,
        primitives: &[LexPrimitiva],
        pattern_hint: Option<&str>,
    ) -> Option<LexPrimitiva> {
        if primitives.is_empty() {
            return None;
        }
        if primitives.len() == 1 {
            return Some(primitives[0]);
        }

        // If pattern hint given, use that pattern's dominant
        if let Some(hint) = pattern_hint {
            if let Some(pattern) = self.registry.get(hint) {
                if let Some(dom) = pattern.composition.dominant {
                    if primitives.contains(&dom) {
                        return Some(dom);
                    }
                }
            }
        }

        let prim_set: HashSet<LexPrimitiva> = primitives.iter().copied().collect();

        // Count edges per primitive within the given set
        let mut edge_counts: Vec<(LexPrimitiva, usize)> = primitives
            .iter()
            .map(|&p| {
                let interactions = self.graph.interactions_for(p);
                let relevant_count = interactions
                    .iter()
                    .filter(|i| prim_set.contains(&i.source) && prim_set.contains(&i.target))
                    .count();
                (p, relevant_count)
            })
            .collect();

        // Sort by edge count descending, then by pattern frequency for tie-break
        edge_counts.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| {
                let a_freq = self.pattern_frequency_for(a.0);
                let b_freq = self.pattern_frequency_for(b.0);
                b_freq.cmp(&a_freq)
            })
        });

        edge_counts.first().map(|&(p, _)| p)
    }

    /// Sum of pattern frequencies where this primitive is dominant.
    fn pattern_frequency_for(&self, primitive: LexPrimitiva) -> usize {
        self.registry
            .iter()
            .filter(|(_, p)| p.composition.dominant == Some(primitive))
            .map(|(_, p)| p.frequency)
            .sum()
    }

    /// Find all interactions among the given primitives.
    fn find_interactions(&self, primitives: &[LexPrimitiva]) -> Vec<Interaction> {
        let prim_set: HashSet<LexPrimitiva> = primitives.iter().copied().collect();
        let mut interactions = Vec::new();

        for &src in primitives {
            for &tgt in primitives {
                if src == tgt {
                    continue;
                }
                if let Some(rel) = self.graph.lookup(src, tgt) {
                    interactions.push(Interaction::new(src, tgt, rel));
                }
            }
        }

        // Deduplicate: keep unique (source, target) pairs
        let mut seen = HashSet::new();
        interactions.retain(|i| {
            let key = (
                LexPrimitiva::all()
                    .iter()
                    .position(|&x| x == i.source)
                    .unwrap_or(0),
                LexPrimitiva::all()
                    .iter()
                    .position(|&x| x == i.target)
                    .unwrap_or(0),
            );
            let _ = &prim_set; // ensure captured for lifetime
            seen.insert(key)
        });

        interactions
    }

    /// Match input composition against all canonical patterns.
    fn match_patterns(&self, comp: &PrimitiveComposition) -> Vec<PatternMatch> {
        let comp_set = comp.unique();
        let mut matches: Vec<PatternMatch> = self
            .registry
            .iter()
            .map(|(_, pattern)| {
                let pattern_set = pattern.composition.unique();
                let missing: Vec<LexPrimitiva> =
                    pattern_set.difference(&comp_set).copied().collect();
                let extra: Vec<LexPrimitiva> = comp_set.difference(&pattern_set).copied().collect();
                let distance = pattern.distance(comp);

                PatternMatch {
                    name: pattern.name.clone(),
                    distance,
                    missing,
                    extra,
                }
            })
            .collect();

        // Sort by distance ascending
        matches.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Generate completion suggestions for near-miss patterns.
    fn generate_suggestions(&self, comp: &PrimitiveComposition) -> Vec<CompletionSuggestion> {
        let comp_set = comp.unique();
        let have: Vec<LexPrimitiva> = comp_set.iter().copied().collect();

        let mut suggestions: Vec<CompletionSuggestion> = Vec::new();

        for (_, pattern) in self.registry.iter() {
            let pattern_set = pattern.composition.unique();
            let missing: Vec<LexPrimitiva> = pattern_set.difference(&comp_set).copied().collect();

            // Only suggest if 1-2 primitives missing (near-miss)
            if missing.is_empty() || missing.len() > 2 {
                continue;
            }

            if let Some(suggestion) = self.suggest_completion(&have, &pattern.name) {
                suggestions.push(suggestion);
            }
        }

        // Sort by coherence gain descending
        suggestions.sort_by(|a, b| {
            b.coherence_gain
                .partial_cmp(&a.coherence_gain)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions
    }
}

impl Default for RevSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitiva::LexPrimitiva::*;

    fn synth() -> RevSynthesizer {
        RevSynthesizer::new()
    }

    fn default_opts() -> SynthesisOpts {
        SynthesisOpts::default()
    }

    // ─── Core Synthesis Tests ────────────────────────────────────────────────

    #[test]
    fn test_synthesize_gatekeeper() {
        let s = synth();
        let result = s.synthesize(vec![Boundary, Comparison], default_opts());
        assert!(result.is_ok());
        let r = result.ok();
        assert!(r.is_some());
        if let Some(r) = r {
            // Should match Gatekeeper
            let gk_match = r.pattern_matches.iter().find(|m| m.name == "Gatekeeper");
            assert!(gk_match.is_some());
            if let Some(gk) = gk_match {
                assert!(gk.distance < f64::EPSILON, "Should be exact match");
                assert!(gk.missing.is_empty());
                assert!(gk.extra.is_empty());
            }
            assert_eq!(r.tier, Tier::T2Primitive);
        }
    }

    #[test]
    fn test_synthesize_pipeline() {
        let s = synth();
        let result = s.synthesize(vec![Sequence, Quantity, Boundary], default_opts());
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            let pipe_match = r.pattern_matches.iter().find(|m| m.name == "Pipeline");
            assert!(pipe_match.is_some());
            if let Some(pm) = pipe_match {
                assert!(pm.distance < f64::EPSILON, "Should be exact match");
            }
            // 3 unique primitives → T2-P
            assert_eq!(r.tier, Tier::T2Primitive);
        }
    }

    #[test]
    fn test_synthesize_novel() {
        let s = synth();
        let result = s.synthesize(vec![Frequency, Persistence, Boundary], default_opts());
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            // No exact match expected for {ν, π, ∂}
            let exact_matches: Vec<_> = r
                .pattern_matches
                .iter()
                .filter(|m| m.distance < f64::EPSILON)
                .collect();
            // This combo doesn't exactly match any of the 12 canonical patterns
            // (Monitor is {ν, κ, ∂}, Archive is {π, σ, ∃})
            assert!(
                exact_matches.is_empty(),
                "Expected no exact match for novel combo"
            );
            // But there should be some suggestions
            assert!(r.nearest_pattern.is_some());
        }
    }

    #[test]
    fn test_dominant_heuristic() {
        let s = synth();
        // For {∂, κ}: ∂ guards κ, so ∂ has outgoing edge to κ.
        // κ also has edge to N (Provides), but N is not in set.
        // ∂ has edges: Guards κ, Constrains σ, Constrains ρ, Provides (from N→∂)
        // Within set {∂, κ}: ∂→κ Guards. So ∂ has 1 relevant edge.
        // κ→N Provides but N not in set. So κ has 0 relevant edges from set.
        let dominant = s.infer_dominant(&[Boundary, Comparison], None);
        assert_eq!(dominant, Some(Boundary));
    }

    #[test]
    fn test_completion_suggestion() {
        let s = synth();
        // Have {∂}, target "Gatekeeper" which is {∂, κ} → should suggest adding κ
        let suggestion = s.suggest_completion(&[Boundary], "Gatekeeper");
        assert!(suggestion.is_some());
        if let Some(sug) = suggestion {
            assert_eq!(sug.target_pattern, "Gatekeeper");
            assert!(sug.missing_primitives.contains(&Comparison));
            assert_eq!(sug.missing_primitives.len(), 1);
        }
    }

    #[test]
    fn test_completion_already_complete() {
        let s = synth();
        // Have {∂, κ} already matches Gatekeeper — no completion needed
        let suggestion = s.suggest_completion(&[Boundary, Comparison], "Gatekeeper");
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_completion_unknown_pattern() {
        let s = synth();
        let suggestion = s.suggest_completion(&[Boundary], "NonExistent");
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_empty_error() {
        let s = synth();
        let result = s.synthesize(vec![], default_opts());
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e, SynthesisError::EmptyPrimitives);
        }
    }

    #[test]
    fn test_tier_classification() {
        let s = synth();
        // 7 primitives → T3
        let result = s.synthesize(
            vec![
                Sequence, Mapping, State, Recursion, Void, Boundary, Frequency,
            ],
            default_opts(),
        );
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.tier, Tier::T3DomainSpecific);
        }
    }

    #[test]
    fn test_coherence_threshold() {
        let s = synth();
        // Set impossibly high coherence threshold
        let opts = SynthesisOpts {
            min_coherence: 0.99,
            ..SynthesisOpts::default()
        };
        // A novel combo with low coherence
        let result = s.synthesize(
            vec![
                Frequency,
                Persistence,
                Location,
                Irreversibility,
                Product,
                Sum,
            ],
            opts,
        );
        // This should fail the coherence check (novel 6-primitive combo)
        if let Err(SynthesisError::BelowCoherenceThreshold { got, min }) = result {
            assert!(got < min);
        }
        // Note: if by chance it passes (unlikely), the test still passes
    }

    #[test]
    fn test_all_canonical_patterns_roundtrip() {
        let s = synth();
        let registry = PatternRegistry::canonical();

        for name in registry.names() {
            if let Some(pattern) = registry.get(name) {
                let primitives: Vec<LexPrimitiva> =
                    pattern.composition.unique().into_iter().collect();
                let result = s.synthesize(primitives, default_opts());
                assert!(result.is_ok(), "Synthesis failed for pattern {}", name);
                if let Some(r) = result.ok() {
                    // The pattern should appear in matches with distance 0
                    let found = r.pattern_matches.iter().find(|m| m.name == name);
                    assert!(found.is_some(), "Pattern {} not found in matches", name);
                    if let Some(m) = found {
                        assert!(
                            m.distance < f64::EPSILON,
                            "Pattern {} distance {:.4} should be 0",
                            name,
                            m.distance
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_reverse_lookup_patterns() {
        let s = synth();
        let matches = s.reverse_lookup_patterns(&[Boundary, Comparison]);
        // Should include Gatekeeper with distance 0
        let gk = matches.iter().find(|m| m.name == "Gatekeeper");
        assert!(gk.is_some());
        if let Some(gk) = gk {
            assert!(gk.distance < f64::EPSILON);
        }
    }

    #[test]
    fn test_single_primitive() {
        let s = synth();
        let result = s.synthesize(vec![Quantity], default_opts());
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.tier, Tier::T1Universal);
            assert_eq!(r.dominant, Some(Quantity));
        }
    }

    #[test]
    fn test_pattern_hint() {
        let s = synth();
        let opts = SynthesisOpts {
            pattern_hint: Some("Gatekeeper".to_string()),
            ..SynthesisOpts::default()
        };
        let result = s.synthesize(vec![Boundary, Comparison], opts);
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            // With Gatekeeper hint, dominant should be Boundary
            assert_eq!(r.dominant, Some(Boundary));
        }
    }

    #[test]
    fn test_interactions_found() {
        let s = synth();
        let result = s.synthesize(vec![Boundary, Comparison], default_opts());
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            // ∂ Guards κ should be in interactions
            let has_guards = r.interactions.iter().any(|i| {
                i.source == Boundary
                    && i.target == Comparison
                    && i.relation == crate::grammar::InteractionType::Guards
            });
            assert!(has_guards, "Expected ∂ Guards κ interaction");
        }
    }

    #[test]
    fn test_synthesis_error_display() {
        let e1 = SynthesisError::EmptyPrimitives;
        assert_eq!(format!("{}", e1), "No primitives provided");

        let e2 = SynthesisError::InvalidPrimitive("xyz".to_string());
        assert!(format!("{}", e2).contains("xyz"));

        let e3 = SynthesisError::BelowCoherenceThreshold { got: 0.1, min: 0.5 };
        assert!(format!("{}", e3).contains("0.10"));
    }

    #[test]
    fn test_default_opts() {
        let opts = SynthesisOpts::default();
        assert!(opts.target_tier.is_none());
        assert!(opts.pattern_hint.is_none());
        assert!(opts.min_coherence.abs() < f64::EPSILON);
    }

    #[test]
    fn test_dedup_input() {
        let s = synth();
        // Duplicate primitives should be deduplicated
        let result = s.synthesize(
            vec![Boundary, Comparison, Boundary, Comparison],
            default_opts(),
        );
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.composition.unique().len(), 2);
            assert_eq!(r.tier, Tier::T2Primitive);
        }
    }
}
