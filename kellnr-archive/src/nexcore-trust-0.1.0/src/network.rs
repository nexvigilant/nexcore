/// Trust network with source attribution and transitivity.
///
/// Manages trust aggregated from multiple independent sources, supporting:
/// - Per-source trust tracking with independence (correlation discount) factors
/// - Transitive trust propagation (A→B→C) with configurable damping
/// - Source diversity metrics for Sybil resistance
///
/// Fixes: Fallacy #2 (Composition — correlated sources get discounted),
/// Fallacy #10 (Ecological — per-source scores preserved),
/// Gap #3 (Trust Networks / Transitivity),
/// Gap #11 (Sybil Resistance — diversity requirements).
///
/// Tier: T3 (μ Mapping + ρ Recursion + ∂ Boundary + ς State + → Causality + N Quantity)
use crate::engine::{TrustConfig, TrustEngine};
use crate::evidence::Evidence;
use crate::level::TrustLevel;

/// A source of trust evidence with a correlation discount.
///
/// Independence measures how much unique information this source provides
/// relative to other sources. Fully correlated sources (independence = 0)
/// contribute nothing to the aggregate — their evidence is already captured.
///
/// Tier: T2-P (μ Mapping + N Quantity)
#[derive(Debug, Clone, Copy)]
pub struct TrustSource {
    /// Unique identifier for this source
    pub id: u64,
    /// Independence factor in [0.0, 1.0].
    /// 1.0 = fully independent (full weight in aggregation)
    /// 0.0 = fully correlated with other sources (zero weight)
    pub independence: f64,
}

impl TrustSource {
    /// Create a source with explicit independence factor.
    pub fn new(id: u64, independence: f64) -> Self {
        Self {
            id,
            independence: independence.clamp(0.0, 1.0),
        }
    }

    /// Create a fully independent source (weight = 1.0).
    pub fn independent(id: u64) -> Self {
        Self::new(id, 1.0)
    }

    /// Create a partially correlated source.
    pub fn correlated(id: u64, independence: f64) -> Self {
        Self::new(id, independence)
    }
}

/// Trust aggregated from multiple attributed sources.
///
/// Each source has its own `TrustEngine` and an independence factor.
/// When computing the aggregate score, evidence from correlated sources
/// (low independence) is discounted to prevent double-counting.
///
/// This addresses the Composition Fallacy: naive merge of correlated
/// sources inflates confidence. Independence-weighted aggregation
/// ensures only unique information contributes.
///
/// Tier: T3 (μ Mapping + ∂ Boundary + ς State + N Quantity + → Causality)
#[derive(Debug, Clone)]
pub struct SourcedTrust {
    sources: Vec<(TrustSource, TrustEngine)>,
    config: TrustConfig,
}

impl SourcedTrust {
    /// Create a new sourced trust aggregator with the given config.
    pub fn new(config: TrustConfig) -> Self {
        Self {
            sources: Vec::new(),
            config,
        }
    }

    /// Add a new trust source. Returns false if the source ID already exists.
    pub fn add_source(&mut self, source: TrustSource) -> bool {
        if self.sources.iter().any(|(s, _)| s.id == source.id) {
            return false;
        }
        self.sources
            .push((source, TrustEngine::with_config(self.config)));
        true
    }

    /// Record evidence from a specific source.
    ///
    /// Returns false if the source is not registered.
    pub fn record(&mut self, source_id: u64, evidence: Evidence) -> bool {
        for (s, engine) in &mut self.sources {
            if s.id == source_id {
                engine.record(evidence);
                return true;
            }
        }
        false
    }

    /// Independence-weighted aggregate score.
    ///
    /// Each source's score is weighted by its independence factor.
    /// This naturally discounts correlated sources, preventing the
    /// Composition Fallacy of treating dependent evidence as independent.
    ///
    /// Returns 0.5 (neutral) if no sources or zero total weight.
    pub fn aggregate_score(&self) -> f64 {
        if self.sources.is_empty() {
            return 0.5;
        }

        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for (source, engine) in &self.sources {
            let w = source.independence;
            weighted_sum += w * engine.score();
            weight_total += w;
        }

        if weight_total <= 0.0 {
            return 0.5;
        }
        weighted_sum / weight_total
    }

    /// Aggregate trust level from the independence-weighted score.
    pub fn aggregate_level(&self) -> TrustLevel {
        TrustLevel::from_score(self.aggregate_score())
    }

    /// Number of independent sources with meaningful evidence.
    ///
    /// A source counts toward diversity when:
    /// 1. Its independence factor exceeds `independence_threshold`
    /// 2. Its engine has significant evidence (sufficient interactions)
    ///
    /// This is the key Sybil resistance metric: an attacker creating
    /// many correlated fake sources won't increase diversity.
    pub fn source_diversity(&self, independence_threshold: f64) -> usize {
        self.sources
            .iter()
            .filter(|(s, e)| s.independence >= independence_threshold && e.is_significant())
            .count()
    }

    /// Whether sufficient independent sources vouch for this entity.
    ///
    /// Requires at least `min_sources` independent (above threshold)
    /// sources with significant evidence. This guards against Sybil
    /// attacks where one entity creates many fake endorsers.
    pub fn has_sufficient_diversity(
        &self,
        min_sources: usize,
        independence_threshold: f64,
    ) -> bool {
        self.source_diversity(independence_threshold) >= min_sources
    }

    /// Number of sources registered.
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Score for a specific source (preserves per-source attribution).
    ///
    /// This prevents the Ecological Fallacy: the aggregate may look fine,
    /// but individual sources can reveal hidden problems.
    pub fn source_score(&self, source_id: u64) -> Option<f64> {
        self.sources
            .iter()
            .find(|(s, _)| s.id == source_id)
            .map(|(_, e)| e.score())
    }

    /// Score for each source, sorted by score ascending (weakest first).
    pub fn source_scores(&self) -> Vec<(u64, f64)> {
        let mut scores: Vec<(u64, f64)> = self
            .sources
            .iter()
            .map(|(s, e)| (s.id, e.score()))
            .collect();
        scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));
        scores
    }

    /// Advance time for all sources.
    pub fn advance_time(&mut self, dt: f64) {
        for (_, engine) in &mut self.sources {
            engine.advance_time(dt);
        }
    }

    /// Reset all sources to prior state.
    pub fn reset(&mut self) {
        for (_, engine) in &mut self.sources {
            engine.reset();
        }
    }
}

impl Default for SourcedTrust {
    fn default() -> Self {
        Self::new(TrustConfig::default())
    }
}

/// Compute transitive trust with damping.
///
/// If entity A trusts B with score `ab`, and B trusts C with score `bc`,
/// then A's transitive trust in C is:
///
/// `transitive = ab * bc * damping`
///
/// The damping factor (0, 1] models information loss through intermediaries.
/// Each hop reduces confidence because B's assessment of C is filtered
/// through A's (imperfect) trust in B's judgment.
///
/// Suggested values:
/// - 0.5: Conservative (trust halves per hop)
/// - 0.8: Moderate (20% loss per hop)
/// - 0.95: Permissive (minimal loss)
///
/// Tier: T2-P (→ Causality + N Quantity)
pub fn transitive_trust(ab: f64, bc: f64, damping: f64) -> f64 {
    let damping = damping.clamp(0.0, 1.0);
    (ab * bc * damping).clamp(0.0, 1.0)
}

/// Compute multi-hop transitive trust through a chain.
///
/// Given a chain of pairwise trust scores [A→B, B→C, C→D, ...],
/// computes the end-to-end transitive trust with per-hop damping.
///
/// Returns 0.5 (neutral) for an empty chain.
///
/// # Example
///
/// ```text
/// A trusts B (0.9), B trusts C (0.8), C trusts D (0.7)
/// chain_trust(&[0.9, 0.8, 0.7], 0.8)
///   = 0.9 * 0.8 * 0.8 * 0.7 * 0.8
///   = 0.9 * (0.8 * 0.8) * (0.7 * 0.8)
///   ≈ 0.323
/// ```
///
/// Tier: T2-C (→ Causality + ρ Recursion + N Quantity)
pub fn chain_trust(scores: &[f64], damping: f64) -> f64 {
    if scores.is_empty() {
        return 0.5;
    }
    let damping = damping.clamp(0.0, 1.0);
    let mut result = scores[0];
    for &score in &scores[1..] {
        result = result * score * damping;
    }
    result.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    // --- TrustSource ---

    #[test]
    fn source_independence_clamped() {
        let s = TrustSource::new(1, 2.0);
        assert!((s.independence - 1.0).abs() < EPS);
        let s = TrustSource::new(2, -1.0);
        assert!((s.independence).abs() < EPS);
    }

    // --- SourcedTrust ---

    #[test]
    fn empty_sourced_trust_is_neutral() {
        let st = SourcedTrust::default();
        assert!((st.aggregate_score() - 0.5).abs() < EPS);
        assert_eq!(st.source_count(), 0);
    }

    #[test]
    fn add_source_rejects_duplicates() {
        let mut st = SourcedTrust::default();
        assert!(st.add_source(TrustSource::independent(1)));
        assert!(!st.add_source(TrustSource::independent(1)));
        assert_eq!(st.source_count(), 1);
    }

    #[test]
    fn record_returns_false_for_unknown_source() {
        let mut st = SourcedTrust::default();
        assert!(!st.record(999, Evidence::positive()));
    }

    #[test]
    fn independent_sources_aggregate_equally() {
        let mut st = SourcedTrust::default();
        st.add_source(TrustSource::independent(1));
        st.add_source(TrustSource::independent(2));

        // Source 1: positive, Source 2: negative
        for _ in 0..10 {
            st.record(1, Evidence::positive());
            st.record(2, Evidence::negative());
        }

        // Source 1 should be high, Source 2 low
        let s1 = st.source_score(1);
        let s2 = st.source_score(2);
        assert!(s1.is_some());
        assert!(s2.is_some());
        if let (Some(score1), Some(score2)) = (s1, s2) {
            assert!(score1 > 0.7, "source 1 should be high: {score1:.3}");
            assert!(score2 < 0.3, "source 2 should be low: {score2:.3}");
        }
    }

    #[test]
    fn correlated_source_has_less_aggregate_influence() {
        // Setup: 1 independent positive source, 1 correlated negative source
        let mut st_equal = SourcedTrust::default();
        st_equal.add_source(TrustSource::independent(1));
        st_equal.add_source(TrustSource::independent(2));

        let mut st_discounted = SourcedTrust::default();
        st_discounted.add_source(TrustSource::independent(1));
        st_discounted.add_source(TrustSource::correlated(2, 0.2)); // Only 20% independent

        // Same evidence to both
        for _ in 0..10 {
            st_equal.record(1, Evidence::positive());
            st_equal.record(2, Evidence::negative());
            st_discounted.record(1, Evidence::positive());
            st_discounted.record(2, Evidence::negative());
        }

        // With discounting, the negative source has less pull
        assert!(
            st_discounted.aggregate_score() > st_equal.aggregate_score(),
            "discounted negative ({:.3}) should pull less than equal ({:.3})",
            st_discounted.aggregate_score(),
            st_equal.aggregate_score()
        );
    }

    #[test]
    fn source_diversity_counts_significant_independent() {
        let mut st = SourcedTrust::default();
        st.add_source(TrustSource::independent(1));
        st.add_source(TrustSource::correlated(2, 0.3));
        st.add_source(TrustSource::independent(3));

        // No evidence yet — no source is significant
        assert_eq!(st.source_diversity(0.5), 0);

        // Add enough evidence to make sources 1 and 3 significant
        for _ in 0..10 {
            st.record(1, Evidence::positive());
            st.record(3, Evidence::positive());
        }

        // Source 1 and 3 are independent + significant
        // Source 2 is below independence threshold (0.3 < 0.5)
        assert_eq!(st.source_diversity(0.5), 2);
    }

    #[test]
    fn sybil_resistance_requires_diversity() {
        let mut st = SourcedTrust::default();

        // Attacker creates 5 correlated fake sources
        for i in 0..5 {
            st.add_source(TrustSource::correlated(i, 0.1));
            for _ in 0..10 {
                st.record(i, Evidence::positive());
            }
        }

        // Despite 5 sources with high scores, diversity is low
        assert!(!st.has_sufficient_diversity(2, 0.5));

        // Add 2 genuine independent sources
        st.add_source(TrustSource::independent(100));
        st.add_source(TrustSource::independent(101));
        for _ in 0..10 {
            st.record(100, Evidence::positive());
            st.record(101, Evidence::positive());
        }

        // Now sufficient diversity exists
        assert!(st.has_sufficient_diversity(2, 0.5));
    }

    #[test]
    fn ecological_fallacy_detection() {
        let mut st = SourcedTrust::default();
        st.add_source(TrustSource::independent(1));
        st.add_source(TrustSource::independent(2));

        // Source 1: very positive. Source 2: very negative.
        for _ in 0..20 {
            st.record(1, Evidence::positive());
            st.record(2, Evidence::negative());
        }

        // Aggregate might look neutral, masking the problem
        let aggregate = st.aggregate_score();
        let scores = st.source_scores();

        // Per-source analysis reveals the divergence
        assert!(scores.len() == 2);
        let worst = scores[0].1;
        let best = scores[1].1;
        assert!(
            (best - worst) > 0.5,
            "per-source divergence should be large: best={best:.3}, worst={worst:.3}, aggregate={aggregate:.3}"
        );
    }

    // --- Transitive Trust ---

    #[test]
    fn transitive_trust_with_full_damping() {
        // A trusts B fully (1.0), B trusts C fully (1.0), damping 0.8
        let t = transitive_trust(1.0, 1.0, 0.8);
        assert!((t - 0.8).abs() < EPS);
    }

    #[test]
    fn transitive_trust_diminishes() {
        // A trusts B (0.9), B trusts C (0.9), damping 0.8
        let t = transitive_trust(0.9, 0.9, 0.8);
        assert!(t < 0.9 * 0.9, "transitive should be less than product");
        assert!((t - 0.9 * 0.9 * 0.8).abs() < EPS);
    }

    #[test]
    fn transitive_trust_clamped() {
        let t = transitive_trust(1.5, 1.5, 1.0);
        assert!(t <= 1.0);
    }

    #[test]
    fn chain_trust_empty_is_neutral() {
        assert!((chain_trust(&[], 0.8) - 0.5).abs() < EPS);
    }

    #[test]
    fn chain_trust_single_hop() {
        assert!((chain_trust(&[0.9], 0.8) - 0.9).abs() < EPS);
    }

    #[test]
    fn chain_trust_degrades_with_length() {
        let short = chain_trust(&[0.9, 0.9], 0.8);
        let long = chain_trust(&[0.9, 0.9, 0.9, 0.9], 0.8);
        assert!(
            long < short,
            "longer chain ({long:.3}) should have lower trust than shorter ({short:.3})"
        );
    }

    #[test]
    fn chain_trust_with_zero_link() {
        // One untrusted link breaks the chain
        let t = chain_trust(&[0.9, 0.0, 0.9], 0.8);
        assert!(t < EPS, "zero-trust link should break chain: {t:.4}");
    }

    #[test]
    fn source_scores_sorted_ascending() {
        let mut st = SourcedTrust::default();
        st.add_source(TrustSource::independent(1));
        st.add_source(TrustSource::independent(2));
        st.add_source(TrustSource::independent(3));

        for _ in 0..10 {
            st.record(1, Evidence::positive());
            st.record(2, Evidence::negative());
            // Source 3 stays neutral
        }

        let scores = st.source_scores();
        assert_eq!(scores.len(), 3);
        // Should be sorted: worst first
        assert!(scores[0].1 <= scores[1].1);
        assert!(scores[1].1 <= scores[2].1);
    }

    #[test]
    fn reset_clears_all_sources() {
        let mut st = SourcedTrust::default();
        st.add_source(TrustSource::independent(1));
        for _ in 0..10 {
            st.record(1, Evidence::positive());
        }
        assert!(st.aggregate_score() > 0.7);
        st.reset();
        assert!((st.aggregate_score() - 0.5).abs() < EPS);
    }
}
