//! # Weighted Compositions
//!
//! Extends `PrimitiveComposition` with per-interaction weights,
//! contribution scoring, and compression to canonical form.
//!
//! ## Overview
//!
//! A `WeightedComposition` wraps a set of primitives with directed
//! interactions between them. Each interaction carries a weight,
//! enabling fine-grained contribution analysis:
//!
//! - **contribution(prim)**: Sum of interaction weights involving a primitive
//! - **total_weight()**: Sum of all interaction weights
//! - **compress()**: Remove zero-weight primitives
//! - **canonical_form()**: Normalize to nearest pattern
//!
//! ## Tier: T2-C (Quantity + Mapping + Comparison + Product)

use crate::grammar::{Interaction, InteractionGraph, InteractionType, PatternRegistry};
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::state_mode::StateMode;
use crate::tier::Tier;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A composition with per-interaction weights and contribution scoring.
///
/// Extends `PrimitiveComposition` with semantic weight information derived
/// from the interaction graph. Each primitive's contribution is computed
/// from the sum of weights on interactions that involve it.
///
/// Tier: T2-C (Quantity + Mapping + Comparison + Product)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WeightedComposition {
    /// The underlying primitives.
    pub primitives: Vec<LexPrimitiva>,
    /// Interactions between primitives in this composition.
    pub interactions: Vec<Interaction>,
    /// The dominant primitive.
    pub dominant: Option<LexPrimitiva>,
    /// Overall confidence (0.0-1.0).
    pub confidence: f64,
    /// Disambiguated State (ς) mode, propagated from composition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_mode: Option<StateMode>,
}

impl WeightedComposition {
    /// Create from a `PrimitiveComposition`, auto-populating interactions
    /// from the canonical interaction graph.
    #[must_use]
    pub fn from_composition(comp: &PrimitiveComposition) -> Self {
        let graph = InteractionGraph::canonical();
        let unique: BTreeSet<_> = comp.unique();
        let mut interactions = Vec::new();

        // For each pair of primitives in the composition, check the graph
        let prims: Vec<LexPrimitiva> = unique.iter().copied().collect();
        for &src in &prims {
            for &tgt in &prims {
                if src == tgt {
                    continue;
                }
                if let Some(rel) = graph.lookup(src, tgt) {
                    interactions.push(Interaction::new(src, tgt, rel));
                }
            }
        }

        Self {
            primitives: comp.primitives.clone(),
            interactions,
            dominant: comp.dominant,
            confidence: comp.confidence,
            state_mode: comp.state_mode,
        }
    }

    /// Create with explicit interactions.
    #[must_use]
    pub fn new(
        primitives: Vec<LexPrimitiva>,
        interactions: Vec<Interaction>,
        dominant: Option<LexPrimitiva>,
    ) -> Self {
        Self {
            primitives,
            interactions,
            dominant,
            confidence: 1.0,
            state_mode: None,
        }
    }

    /// Compute the contribution of a primitive (sum of interaction weights
    /// where it appears as source or target).
    #[must_use]
    pub fn contribution(&self, prim: LexPrimitiva) -> f64 {
        self.interactions
            .iter()
            .filter(|i| i.source == prim || i.target == prim)
            .map(|i| i.weight)
            .sum()
    }

    /// Total weight across all interactions.
    #[must_use]
    pub fn total_weight(&self) -> f64 {
        self.interactions.iter().map(|i| i.weight).sum()
    }

    /// Average weight per interaction (0.0 if no interactions).
    #[must_use]
    pub fn average_weight(&self) -> f64 {
        if self.interactions.is_empty() {
            return 0.0;
        }
        #[allow(
            clippy::as_conversions,
            reason = "interactions.len() bounded by 16*15=240, safe cast to f64"
        )]
        let len = self.interactions.len() as f64;
        self.total_weight() / len
    }

    /// Remove primitives with zero contribution (not involved in any interaction).
    ///
    /// Preserves the dominant even if it has zero contribution.
    #[must_use]
    pub fn compress(&self) -> Self {
        let mut keep: BTreeSet<LexPrimitiva> = BTreeSet::new();

        // Keep all primitives involved in interactions
        for interaction in &self.interactions {
            keep.insert(interaction.source);
            keep.insert(interaction.target);
        }

        // Always keep the dominant
        if let Some(dom) = self.dominant {
            keep.insert(dom);
        }

        let compressed: Vec<LexPrimitiva> = self
            .primitives
            .iter()
            .copied()
            .filter(|p| keep.contains(p))
            .collect();

        // Deduplicate
        let mut seen: BTreeSet<LexPrimitiva> = BTreeSet::new();
        let deduped: Vec<LexPrimitiva> =
            compressed.into_iter().filter(|p| seen.insert(*p)).collect();

        Self {
            primitives: deduped,
            interactions: self.interactions.clone(),
            dominant: self.dominant,
            confidence: self.confidence,
            state_mode: self.state_mode,
        }
    }

    /// Normalize to the nearest canonical pattern form.
    ///
    /// If the composition closely matches a canonical pattern, returns the
    /// pattern's composition. Otherwise returns a normalized version.
    #[must_use]
    pub fn canonical_form(&self) -> PrimitiveComposition {
        let registry = PatternRegistry::canonical();
        let comp = self.to_composition();

        if let Some((pattern, distance)) = registry.closest(&comp) {
            if distance < 0.3 {
                // Close enough to a canonical pattern — adopt its form
                return pattern.composition.clone();
            }
        }

        // Not close to any pattern — return normalized version
        let unique: BTreeSet<_> = comp.unique();
        let mut sorted: Vec<_> = unique.into_iter().collect();
        sorted.sort_by_key(|p: &LexPrimitiva| p.symbol());
        let mut result = PrimitiveComposition::new(sorted);
        if let Some(dom) = self.dominant {
            result = result.with_dominant(dom, self.confidence);
        }
        result
    }

    /// Lossless downcast to `PrimitiveComposition` (drops weight information).
    #[must_use]
    pub fn to_composition(&self) -> PrimitiveComposition {
        let mut comp = PrimitiveComposition::new(self.primitives.clone());
        if let Some(dom) = self.dominant {
            comp = comp.with_dominant(dom, self.confidence);
        }
        if let Some(mode) = self.state_mode {
            comp = comp.with_state_mode(mode);
        }
        comp
    }

    /// Unique primitives as a set.
    #[must_use]
    pub fn unique(&self) -> BTreeSet<LexPrimitiva> {
        self.primitives.iter().copied().collect()
    }

    /// Tier classification.
    #[must_use]
    pub fn tier(&self) -> Tier {
        Tier::classify(&self.to_composition())
    }

    /// Get the highest-contribution primitive.
    #[must_use]
    pub fn strongest_primitive(&self) -> Option<(LexPrimitiva, f64)> {
        let unique = self.unique();
        unique
            .into_iter()
            .map(|p| (p, self.contribution(p)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get primitives ranked by contribution (descending).
    #[must_use]
    pub fn ranked_primitives(&self) -> Vec<(LexPrimitiva, f64)> {
        let unique = self.unique();
        let mut ranked: Vec<(LexPrimitiva, f64)> = unique
            .into_iter()
            .map(|p| (p, self.contribution(p)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Count interactions by type.
    #[must_use]
    pub fn interaction_counts(&self) -> Vec<(InteractionType, usize)> {
        let mut counts: BTreeMap<InteractionType, usize> = BTreeMap::new();
        for interaction in &self.interactions {
            let entry = counts.entry(interaction.relation).or_insert(0);
            *entry = entry.saturating_add(1);
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
}

impl std::fmt::Display for WeightedComposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbols: Vec<&str> = self.unique().iter().map(LexPrimitiva::symbol).collect();
        write!(
            f,
            "[{}] (w={:.2}, i={})",
            symbols.join(" + "),
            self.total_weight(),
            self.interactions.len(),
        )
    }
}

/// Builder for constructing weighted compositions fluently.
///
/// Tier: T2-P (Sequence + Mapping)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct WeightedBuilder {
    primitives: Vec<LexPrimitiva>,
    interactions: Vec<Interaction>,
    dominant: Option<LexPrimitiva>,
    confidence: f64,
    auto_interactions: bool,
    state_mode: Option<StateMode>,
}

impl WeightedBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
            interactions: Vec::new(),
            dominant: None,
            confidence: 1.0,
            auto_interactions: false,
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

    /// Add an explicit interaction.
    #[must_use]
    pub fn interact(
        mut self,
        source: LexPrimitiva,
        target: LexPrimitiva,
        relation: InteractionType,
    ) -> Self {
        self.interactions
            .push(Interaction::new(source, target, relation));
        self
    }

    /// Add an interaction with custom weight.
    #[must_use]
    pub fn interact_weighted(
        mut self,
        source: LexPrimitiva,
        target: LexPrimitiva,
        relation: InteractionType,
        weight: f64,
    ) -> Self {
        self.interactions
            .push(Interaction::with_weight(source, target, relation, weight));
        self
    }

    /// Auto-populate interactions from the canonical graph.
    #[must_use]
    pub fn auto(mut self) -> Self {
        self.auto_interactions = true;
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

    /// Build the weighted composition.
    #[must_use]
    pub fn build(self) -> WeightedComposition {
        let mut interactions = self.interactions;

        if self.auto_interactions {
            let graph = InteractionGraph::canonical();
            let unique: BTreeSet<_> = self.primitives.iter().copied().collect();
            let prims: Vec<_> = unique.iter().copied().collect();

            for &src in &prims {
                for &tgt in &prims {
                    if src == tgt {
                        continue;
                    }
                    if let Some(rel) = graph.lookup(src, tgt) {
                        // Don't duplicate explicitly added interactions
                        let already = interactions
                            .iter()
                            .any(|i| i.source == src && i.target == tgt && i.relation == rel);
                        if !already {
                            interactions.push(Interaction::new(src, tgt, rel));
                        }
                    }
                }
            }
        }

        WeightedComposition {
            primitives: self.primitives,
            interactions,
            dominant: self.dominant,
            confidence: self.confidence,
            state_mode: self.state_mode,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitiva::LexPrimitiva::*;

    // ─── Construction tests ───

    #[test]
    fn test_from_composition_basic() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let wc = WeightedComposition::from_composition(&comp);
        assert_eq!(wc.primitives.len(), 2);
        // Boundary Guards Comparison is in the canonical graph
        assert!(!wc.interactions.is_empty());
    }

    #[test]
    fn test_from_composition_no_interactions() {
        // Two primitives with no canonical interaction
        let comp = PrimitiveComposition::new(vec![Product, Void]);
        let wc = WeightedComposition::from_composition(&comp);
        assert!(wc.interactions.is_empty());
    }

    #[test]
    fn test_new_explicit() {
        let interactions = vec![Interaction::new(
            Boundary,
            Comparison,
            InteractionType::Guards,
        )];
        let wc = WeightedComposition::new(vec![Boundary, Comparison], interactions, Some(Boundary));
        assert_eq!(wc.interactions.len(), 1);
        assert_eq!(wc.dominant, Some(Boundary));
    }

    // ─── Contribution tests ───

    #[test]
    fn test_contribution_involved_primitive() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let wc = WeightedComposition::from_composition(&comp);
        let contrib = wc.contribution(Boundary);
        assert!(contrib > 0.0, "Boundary should have nonzero contribution");
    }

    #[test]
    fn test_contribution_uninvolved_primitive() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let wc = WeightedComposition::from_composition(&comp);
        let contrib = wc.contribution(Quantity);
        assert!(contrib.abs() < f64::EPSILON, "Quantity not in composition");
    }

    #[test]
    fn test_total_weight() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let wc = WeightedComposition::from_composition(&comp);
        assert!(wc.total_weight() > 0.0);
    }

    #[test]
    fn test_average_weight_nonempty() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let wc = WeightedComposition::from_composition(&comp);
        if !wc.interactions.is_empty() {
            assert!(wc.average_weight() > 0.0);
        }
    }

    #[test]
    fn test_average_weight_empty() {
        let wc = WeightedComposition::new(vec![Quantity], vec![], Some(Quantity));
        assert!(wc.average_weight().abs() < f64::EPSILON);
    }

    // ─── Compress tests ───

    #[test]
    fn test_compress_removes_disconnected() {
        // Add a primitive not involved in any interaction
        let interactions = vec![Interaction::new(
            Boundary,
            Comparison,
            InteractionType::Guards,
        )];
        let wc = WeightedComposition::new(
            vec![Boundary, Comparison, Location], // Location has no interactions
            interactions,
            Some(Boundary),
        );
        let compressed = wc.compress();
        assert_eq!(compressed.unique().len(), 2);
        assert!(!compressed.primitives.contains(&Location));
    }

    #[test]
    fn test_compress_keeps_dominant() {
        // Dominant with no interactions should still be kept
        let wc =
            WeightedComposition::new(vec![Boundary, Comparison, Location], vec![], Some(Location));
        let compressed = wc.compress();
        assert!(compressed.primitives.contains(&Location));
    }

    #[test]
    fn test_compress_deduplicates() {
        let interactions = vec![Interaction::new(
            Boundary,
            Comparison,
            InteractionType::Guards,
        )];
        let wc = WeightedComposition::new(
            vec![Boundary, Comparison, Boundary], // duplicate
            interactions,
            Some(Boundary),
        );
        let compressed = wc.compress();
        assert_eq!(compressed.unique().len(), 2);
        assert_eq!(compressed.primitives.len(), 2);
    }

    // ─── Canonical form tests ───

    #[test]
    fn test_canonical_form_matches_pattern() {
        let comp =
            PrimitiveComposition::new(vec![Boundary, Comparison]).with_dominant(Boundary, 0.9);
        let wc = WeightedComposition::from_composition(&comp);
        let canonical = wc.canonical_form();
        // Should match Gatekeeper pattern
        let unique = canonical.unique();
        assert!(unique.contains(&Boundary));
        assert!(unique.contains(&Comparison));
    }

    #[test]
    fn test_canonical_form_novel() {
        // Composition far from any pattern
        let comp = PrimitiveComposition::new(vec![
            Frequency,
            Persistence,
            Location,
            Irreversibility,
            Product,
            Sum,
        ]);
        let wc = WeightedComposition::from_composition(&comp);
        let canonical = wc.canonical_form();
        // Should return normalized (sorted) version since no close pattern
        assert_eq!(canonical.unique().len(), 6);
    }

    // ─── to_composition tests ───

    #[test]
    fn test_to_composition_lossless() {
        let original =
            PrimitiveComposition::new(vec![Sequence, Mapping]).with_dominant(Sequence, 0.85);
        let wc = WeightedComposition::from_composition(&original);
        let back = wc.to_composition();
        assert_eq!(back.primitives, original.primitives);
        assert_eq!(back.dominant, original.dominant);
        assert!((back.confidence - original.confidence).abs() < f64::EPSILON);
    }

    // ─── Ranking tests ───

    #[test]
    fn test_strongest_primitive() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let wc = WeightedComposition::from_composition(&comp);
        let strongest = wc.strongest_primitive();
        assert!(strongest.is_some());
    }

    #[test]
    fn test_ranked_primitives_descending() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison, Quantity]);
        let wc = WeightedComposition::from_composition(&comp);
        let ranked = wc.ranked_primitives();
        // Verify descending order
        for window in ranked.windows(2) {
            assert!(window[0].1 >= window[1].1);
        }
    }

    #[test]
    fn test_interaction_counts() {
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison, Quantity]);
        let wc = WeightedComposition::from_composition(&comp);
        let counts = wc.interaction_counts();
        let total: usize = counts.iter().map(|(_, c)| *c).sum();
        assert_eq!(total, wc.interactions.len());
    }

    // ─── Tier tests ───

    #[test]
    fn test_tier_single() {
        let wc = WeightedComposition::new(vec![Quantity], vec![], Some(Quantity));
        assert_eq!(wc.tier(), Tier::T1Universal);
    }

    #[test]
    fn test_tier_multi() {
        let wc = WeightedComposition::new(vec![Boundary, Comparison], vec![], Some(Boundary));
        assert_eq!(wc.tier(), Tier::T2Primitive);
    }

    // ─── Builder tests ───

    #[test]
    fn test_builder_basic() {
        let wc = WeightedBuilder::new()
            .add(Boundary)
            .add(Comparison)
            .dominant(Boundary)
            .confidence(0.9)
            .build();
        assert_eq!(wc.primitives.len(), 2);
        assert_eq!(wc.dominant, Some(Boundary));
        assert!((wc.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_with_interaction() {
        let wc = WeightedBuilder::new()
            .add(Boundary)
            .add(Comparison)
            .interact(Boundary, Comparison, InteractionType::Guards)
            .build();
        assert_eq!(wc.interactions.len(), 1);
    }

    #[test]
    fn test_builder_auto_interactions() {
        let wc = WeightedBuilder::new()
            .add(Boundary)
            .add(Comparison)
            .auto()
            .build();
        // Should auto-populate Boundary Guards Comparison from canonical graph
        assert!(!wc.interactions.is_empty());
    }

    #[test]
    fn test_builder_auto_no_duplicates() {
        let wc = WeightedBuilder::new()
            .add(Boundary)
            .add(Comparison)
            .interact(Boundary, Comparison, InteractionType::Guards)
            .auto() // Should not duplicate the explicit interaction
            .build();
        let guards_count = wc
            .interactions
            .iter()
            .filter(|i| {
                i.source == Boundary
                    && i.target == Comparison
                    && i.relation == InteractionType::Guards
            })
            .count();
        assert_eq!(guards_count, 1);
    }

    #[test]
    fn test_builder_weighted_interaction() {
        let wc = WeightedBuilder::new()
            .add(Mapping)
            .add(Causality)
            .interact_weighted(Mapping, Causality, InteractionType::Transforms, 0.42)
            .build();
        assert_eq!(wc.interactions.len(), 1);
        assert!((wc.interactions[0].weight - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_add_all() {
        let wc = WeightedBuilder::new()
            .add_all(&[Sequence, Mapping, State])
            .auto()
            .build();
        assert_eq!(wc.primitives.len(), 3);
    }

    // ─── Display tests ───

    #[test]
    fn test_display() {
        let wc = WeightedBuilder::new()
            .add(Boundary)
            .add(Comparison)
            .auto()
            .build();
        let s = format!("{}", wc);
        assert!(s.contains("w="));
        assert!(s.contains("i="));
    }
}
