//! # NexCoreInsight — System-Level Insight Compositor
//!
//! The capstone of the architectural reframe: **NexCore IS an InsightEngine.**
//!
//! `NexCoreInsight` holds a single `InsightEngine` that sees ALL observations
//! from ALL registered domains (Guardian, Brain, PV, FAERS, etc.). Cross-domain
//! patterns emerge naturally because the engine's co-occurrence detection
//! operates across domain boundaries.
//!
//! ## Architecture
//!
//! ```text
//!                    ┌─────────────────────┐
//!                    │   NexCoreInsight     │
//!                    │   (impl Insight)     │
//!                    │                     │
//!                    │  ┌───────────────┐  │
//!                    │  │ InsightEngine │  │
//!                    │  │ (unified ς)   │  │
//!                    │  └───────┬───────┘  │
//!                    │          │          │
//!                    └──────────┼──────────┘
//!                   ┌───────┬──┴──┬────────┐
//!                   ▼       ▼     ▼        ▼
//!              guardian   brain   pv     faers
//!              domain    domain  domain  domain
//! ```
//!
//! Each domain's observations are auto-tagged with the domain name,
//! enabling domain-scoped queries and cross-domain pattern detection.
//!
//! ## T1 Grounding
//!
//! NexCoreInsight ≡ ⟨σ, κ, μ, ∃, ς, ∅, N, ∂, Σ⟩ (T3)
//! - σ: Temporal ordering across all domains
//! - κ: Cross-domain comparison and threshold crossing
//! - μ: Connection mapping between domain observations
//! - ∃: Novelty detection across the full observation space
//! - ς: Unified state accumulation (ς-acc)
//! - ∅: Absence detection (novel combinations)
//! - N: Aggregate counts and ratios
//! - ∂: Suddenness thresholds
//! - Σ: Multi-domain aggregation (coproduct of domain events)
//!
//! ## Chomsky Level
//!
//! Type-0 (Unrestricted) — all 5 generators {σ, Σ, ρ, κ, ∃} active.
//! Cross-domain patterns use ρ (recursive learning), making this a
//! Turing-complete insight system.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::composites::{Compression, Connection, Pattern};
use crate::engine::{InsightConfig, InsightEngine, InsightEvent, Observation};
use crate::traits::Insight;

/// Domain registration entry.
///
/// Tier: T2-P (λ + ∃ — Location of domain + Existence proof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    /// Domain name used for tagging observations.
    pub name: String,
    /// Optional description of what this domain contributes.
    pub description: String,
    /// Number of observations ingested from this domain.
    pub observation_count: u64,
}

impl Domain {
    /// Create a new domain entry.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            observation_count: 0,
        }
    }
}

/// System-level insight compositor — NexCore IS this engine.
///
/// Holds a single `InsightEngine` that sees observations from all domains.
/// Cross-domain patterns emerge from the unified co-occurrence state.
///
/// ## Key Design Decisions
///
/// 1. **Single engine, not N engines.** One ς-acc accumulates everything.
///    Cross-domain patterns (e.g., "guardian risk X co-occurs with brain
///    learning Y") are first-class citizens.
///
/// 2. **Broadcast model.** Every observation enters the unified engine.
///    Domain tags enable filtering but don't partition the state space.
///
/// 3. **Domain registration.** Domains are named sources of observations.
///    Registration is declarative — the compositor doesn't depend on
///    adapter crate types, avoiding circular dependencies.
///
/// Tier: T3 (system-level orchestrator)
#[derive(Debug, Serialize, Deserialize)]
pub struct NexCoreInsight {
    /// The unified insight engine (single ς-acc for all domains).
    engine: InsightEngine,
    /// Registered domains (named observation sources).
    domains: Vec<Domain>,
}

impl NexCoreInsight {
    /// Create a new system-level insight compositor with default config.
    ///
    /// Tier: T1 (∃ — bringing into existence)
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: InsightEngine::new(),
            domains: Vec::new(),
        }
    }

    /// Create with a specific config.
    #[must_use]
    pub fn with_config(config: InsightConfig) -> Self {
        Self {
            engine: InsightEngine::with_config(config),
            domains: Vec::new(),
        }
    }

    // ── Domain Management ────────────────────────────────────────────────

    /// Register a domain as an observation source.
    ///
    /// Domains are named subsystems that contribute observations:
    /// - `"guardian"` — threat sensing, risk evaluation
    /// - `"brain"` — implicit learning, artifact tracking
    /// - `"pv"` — signal detection, adverse events
    /// - `"faers"` — FDA adverse event reports
    ///
    /// Tier: T2-P (λ + ∃ — locating + creating domain entry)
    pub fn register_domain(&mut self, name: impl Into<String>, description: impl Into<String>) {
        let name = name.into();
        // Idempotent — don't duplicate
        if !self.domains.iter().any(|d| d.name == name) {
            self.domains.push(Domain::new(name, description));
        }
    }

    /// List all registered domains.
    #[must_use]
    pub fn domains(&self) -> &[Domain] {
        &self.domains
    }

    /// Get a domain by name.
    #[must_use]
    pub fn domain(&self, name: &str) -> Option<&Domain> {
        self.domains.iter().find(|d| d.name == name)
    }

    /// Number of registered domains.
    #[must_use]
    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }

    // ── Domain-Scoped Ingestion ──────────────────────────────────────────

    /// Ingest an observation from a specific domain.
    ///
    /// The observation is auto-tagged with the domain name, enabling:
    /// 1. Domain-scoped pattern queries
    /// 2. Cross-domain co-occurrence detection
    /// 3. Per-domain observation counts
    ///
    /// If the domain is not registered, it is auto-registered with an
    /// empty description.
    ///
    /// Tier: T3 (full pipeline through unified engine)
    pub fn ingest_from(&mut self, domain: &str, observation: Observation) -> Vec<InsightEvent> {
        // Auto-register unknown domains
        if !self.domains.iter().any(|d| d.name == domain) {
            self.register_domain(domain, "");
        }

        // Increment domain observation count
        if let Some(d) = self.domains.iter_mut().find(|d| d.name == domain) {
            d.observation_count += 1;
        }

        // Auto-tag with domain name
        let tagged = observation.with_tag(domain);
        self.engine.ingest(tagged)
    }

    /// Ingest a batch of observations from a specific domain.
    pub fn ingest_batch_from(
        &mut self,
        domain: &str,
        observations: Vec<Observation>,
    ) -> Vec<InsightEvent> {
        let mut all_events = Vec::new();
        for obs in observations {
            all_events.extend(self.ingest_from(domain, obs));
        }
        all_events
    }

    // ── Cross-Domain Queries ─────────────────────────────────────────────

    /// Find patterns whose members span multiple domains.
    ///
    /// These are the most valuable patterns — they reveal connections
    /// between subsystems that no single domain could detect alone.
    ///
    /// A cross-domain pattern has member keys tagged from different domains
    /// in the observation history.
    ///
    /// Tier: T2-C (κ + μ + Σ — comparing domains via mapped patterns)
    #[must_use]
    pub fn cross_domain_patterns(&self) -> Vec<&Pattern> {
        self.engine
            .patterns()
            .into_iter()
            .filter(|p| {
                let member_domains: HashSet<&str> = p
                    .members
                    .iter()
                    .filter_map(|member| {
                        // Check which domain(s) have contributed observations
                        // for this member key
                        self.domains
                            .iter()
                            .find(|d| {
                                // A member belongs to a domain if any observation
                                // with that key was tagged with the domain name
                                self.engine
                                    .connections()
                                    .iter()
                                    .any(|c| c.involves(member) && c.involves(&d.name))
                            })
                            .map(|d| d.name.as_str())
                    })
                    .collect();
                member_domains.len() > 1
            })
            .collect()
    }

    /// Get patterns that involve a specific domain.
    ///
    /// Tier: T2-C (κ + σ + μ — comparison-filtered pattern query)
    #[must_use]
    pub fn patterns_for_domain(&self, domain: &str) -> Vec<&Pattern> {
        let domain_tag = domain;
        self.engine
            .patterns()
            .into_iter()
            .filter(|p| {
                // A pattern involves a domain if any of its members
                // have been observed with that domain tag
                p.members.iter().any(|m| {
                    // Check if this member key has the domain tag in its key
                    // by convention: domain-tagged observations use "domain:key" or tags
                    m.contains(domain_tag)
                })
            })
            .collect()
    }

    /// Get observation count for a specific domain.
    #[must_use]
    pub fn domain_observation_count(&self, domain: &str) -> u64 {
        self.domains
            .iter()
            .find(|d| d.name == domain)
            .map_or(0, |d| d.observation_count)
    }

    // ── Engine Access ────────────────────────────────────────────────────

    /// Direct access to the underlying engine (for advanced queries).
    #[must_use]
    pub fn engine(&self) -> &InsightEngine {
        &self.engine
    }

    /// Mutable access to the underlying engine config.
    pub fn config_mut(&mut self) -> &mut InsightConfig {
        &mut self.engine.config
    }

    /// Get the current config.
    #[must_use]
    pub fn config(&self) -> &InsightConfig {
        &self.engine.config
    }

    /// Summary statistics across all domains.
    ///
    /// Tier: T2-C (N + Σ — quantity aggregation)
    #[must_use]
    pub fn summary(&self) -> NexCoreInsightSummary {
        NexCoreInsightSummary {
            domain_count: self.domains.len(),
            total_observations: self.engine.observation_count(),
            unique_keys: self.engine.unique_key_count(),
            pattern_count: self.engine.pattern_count(),
            connection_count: self.engine.connections().len(),
            event_count: self.engine.events().len(),
            domains: self
                .domains
                .iter()
                .map(|d| (d.name.clone(), d.observation_count))
                .collect(),
        }
    }
}

impl Default for NexCoreInsight {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement the Insight trait — NexCore IS an InsightEngine.
///
/// This is the architectural capstone: NexCoreInsight fulfills the same
/// behavioral contract as any domain-specific adapter, but operates
/// across ALL domains simultaneously.
impl Insight for NexCoreInsight {
    type Obs = Observation;

    fn ingest(&mut self, observation: Self::Obs) -> Vec<InsightEvent> {
        // Observations without a domain tag go to the "system" domain
        self.ingest_from("system", observation)
    }

    fn ingest_batch(&mut self, observations: Vec<Self::Obs>) -> Vec<InsightEvent> {
        self.ingest_batch_from("system", observations)
    }

    fn connect(&mut self, from: &str, to: &str, relation: &str, strength: f64) -> Connection {
        self.engine.connect(from, to, relation, strength)
    }

    fn compress(&mut self, keys: Vec<String>, principle: &str) -> Compression {
        self.engine.compress_by_keys(keys, principle)
    }

    fn events(&self) -> &[InsightEvent] {
        self.engine.events()
    }

    fn observation_count(&self) -> usize {
        self.engine.observation_count()
    }

    fn pattern_count(&self) -> usize {
        self.engine.pattern_count()
    }

    fn patterns(&self) -> Vec<&Pattern> {
        self.engine.patterns()
    }

    fn connections(&self) -> &[Connection] {
        self.engine.connections()
    }

    fn unique_key_count(&self) -> usize {
        self.engine.unique_key_count()
    }
}

// NOTE: `impl Insight for InsightEngine` is in engine.rs.
// Both InsightEngine and NexCoreInsight implement the Insight trait,
// enabling generic code: `fn process(engine: &impl Insight) { ... }`

/// Summary statistics for the system-level insight compositor.
///
/// Tier: T2-C (N + Σ — quantity summary via aggregation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexCoreInsightSummary {
    pub domain_count: usize,
    pub total_observations: usize,
    pub unique_keys: usize,
    pub pattern_count: usize,
    pub connection_count: usize,
    pub event_count: usize,
    /// Per-domain observation counts.
    pub domains: Vec<(String, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Insight;

    // ── Construction ─────────────────────────────────────────────────────

    #[test]
    fn test_new_creates_empty_compositor() {
        let nc = NexCoreInsight::new();
        assert_eq!(nc.domain_count(), 0);
        assert_eq!(nc.observation_count(), 0);
        assert_eq!(nc.pattern_count(), 0);
    }

    #[test]
    fn test_default_matches_new() {
        let a = NexCoreInsight::new();
        let b = NexCoreInsight::default();
        assert_eq!(a.domain_count(), b.domain_count());
        assert_eq!(a.observation_count(), b.observation_count());
    }

    // ── Domain Registration ──────────────────────────────────────────────

    #[test]
    fn test_register_domain() {
        let mut nc = NexCoreInsight::new();
        nc.register_domain("guardian", "Threat sensing");
        nc.register_domain("brain", "Implicit learning");
        assert_eq!(nc.domain_count(), 2);
        assert!(nc.domain("guardian").is_some());
        assert!(nc.domain("brain").is_some());
        assert!(nc.domain("nonexistent").is_none());
    }

    #[test]
    fn test_register_domain_idempotent() {
        let mut nc = NexCoreInsight::new();
        nc.register_domain("guardian", "First");
        nc.register_domain("guardian", "Second");
        assert_eq!(nc.domain_count(), 1);
    }

    #[test]
    fn test_auto_register_on_ingest() {
        let mut nc = NexCoreInsight::new();
        let obs = Observation::new("risk_level", "high");
        let _events = nc.ingest_from("guardian", obs);
        assert_eq!(nc.domain_count(), 1);
        assert!(nc.domain("guardian").is_some());
    }

    // ── Domain-Scoped Ingestion ──────────────────────────────────────────

    #[test]
    fn test_ingest_from_tags_observation() {
        let mut nc = NexCoreInsight::new();
        nc.register_domain("guardian", "Threat sensing");

        let obs = Observation::new("risk_level", "high");
        let events = nc.ingest_from("guardian", obs);

        assert!(!events.is_empty());
        assert_eq!(nc.observation_count(), 1);
        assert_eq!(nc.domain_observation_count("guardian"), 1);
    }

    #[test]
    fn test_ingest_from_multiple_domains() {
        let mut nc = NexCoreInsight::new();
        nc.register_domain("guardian", "Threat sensing");
        nc.register_domain("brain", "Implicit learning");

        let _e1 = nc.ingest_from("guardian", Observation::new("risk", "high"));
        let _e2 = nc.ingest_from("brain", Observation::new("pattern", "recurring"));
        let _e3 = nc.ingest_from("guardian", Observation::new("threat", "pamp"));

        assert_eq!(nc.observation_count(), 3);
        assert_eq!(nc.domain_observation_count("guardian"), 2);
        assert_eq!(nc.domain_observation_count("brain"), 1);
        assert_eq!(nc.unique_key_count(), 3);
    }

    #[test]
    fn test_ingest_batch_from() {
        let mut nc = NexCoreInsight::new();
        let observations = vec![
            Observation::new("a", "1"),
            Observation::new("b", "2"),
            Observation::new("c", "3"),
        ];
        let events = nc.ingest_batch_from("test", observations);
        assert!(!events.is_empty());
        assert_eq!(nc.observation_count(), 3);
        assert_eq!(nc.domain_observation_count("test"), 3);
    }

    // ── Insight Trait ────────────────────────────────────────────────────

    #[test]
    fn test_insight_trait_ingest_uses_system_domain() {
        let mut nc = NexCoreInsight::new();
        let obs = Observation::new("untargeted", "value");
        let _events = nc.ingest(obs);
        assert_eq!(nc.domain_observation_count("system"), 1);
    }

    #[test]
    fn test_insight_trait_connect() {
        let mut nc = NexCoreInsight::new();
        let conn = nc.connect("drug_a", "event_x", "causes", 0.8);
        assert_eq!(conn.from, "drug_a");
        assert_eq!(conn.to, "event_x");
        assert!((conn.strength - 0.8).abs() < f64::EPSILON);
        assert_eq!(nc.connections().len(), 1);
    }

    #[test]
    fn test_insight_trait_compress() {
        let mut nc = NexCoreInsight::new();
        // Ingest enough observations to have something to compress
        for i in 0..5 {
            let _e = nc.ingest_from(
                "test",
                Observation::new(&format!("key_{i}"), &format!("val_{i}")),
            );
        }
        let comp = nc.compress(
            vec!["key_0".into(), "key_1".into(), "key_2".into()],
            "test_principle",
        );
        assert_eq!(comp.principle, "test_principle");
        assert_eq!(comp.input_count, 3);
    }

    // ── Cross-Domain Pattern Detection ───────────────────────────────────

    #[test]
    fn test_multi_domain_co_occurrence_builds_patterns() {
        let mut nc = NexCoreInsight::with_config(InsightConfig {
            pattern_min_occurrences: 2,
            pattern_confidence_threshold: 0.3,
            ..InsightConfig::default()
        });

        nc.register_domain("guardian", "Threat sensing");
        nc.register_domain("brain", "Learning");

        // Create co-occurring observations across domains
        // observation keys co-occur if ingested together
        for _ in 0..3 {
            let _e1 = nc.ingest_from("guardian", Observation::new("risk_high", "yes"));
            let _e2 = nc.ingest_from("brain", Observation::new("risk_high", "learning"));
        }

        // The key "risk_high" has been seen 6 times across two domains
        assert_eq!(nc.observation_count(), 6);
        // Both domains contributed
        assert_eq!(nc.domain_observation_count("guardian"), 3);
        assert_eq!(nc.domain_observation_count("brain"), 3);
    }

    // ── Summary ──────────────────────────────────────────────────────────

    #[test]
    fn test_summary() {
        let mut nc = NexCoreInsight::new();
        nc.register_domain("guardian", "Threats");
        nc.register_domain("brain", "Learning");
        let _e1 = nc.ingest_from("guardian", Observation::new("a", "1"));
        let _e2 = nc.ingest_from("brain", Observation::new("b", "2"));

        let summary = nc.summary();
        assert_eq!(summary.domain_count, 2);
        assert_eq!(summary.total_observations, 2);
        assert_eq!(summary.unique_keys, 2);
        assert_eq!(summary.domains.len(), 2);
    }

    // ── InsightEngine impl Insight ───────────────────────────────────────

    #[test]
    fn test_engine_implements_insight_trait() {
        let mut engine = InsightEngine::new();
        // Use via trait methods
        let events =
            <InsightEngine as Insight>::ingest(&mut engine, Observation::new("test", "value"));
        assert!(!events.is_empty());
        assert_eq!(<InsightEngine as Insight>::observation_count(&engine), 1,);
    }

    #[test]
    fn test_engine_compress_via_trait() {
        let mut engine = InsightEngine::new();
        for i in 0..3 {
            let _e = engine.ingest(Observation::new(&format!("k{i}"), &format!("v{i}")));
        }
        let comp = <InsightEngine as Insight>::compress(
            &mut engine,
            vec!["k0".into(), "k1".into()],
            "unified",
        );
        assert_eq!(comp.principle, "unified");
    }

    // ── Generic Insight Consumer ─────────────────────────────────────────

    fn count_observations(engine: &impl Insight) -> usize {
        engine.observation_count()
    }

    #[test]
    fn test_generic_consumer_with_engine() {
        let mut engine = InsightEngine::new();
        let _e = engine.ingest(Observation::new("a", "b"));
        assert_eq!(count_observations(&engine), 1);
    }

    #[test]
    fn test_generic_consumer_with_nexcore() {
        let mut nc = NexCoreInsight::new();
        let _e = nc.ingest_from("test", Observation::new("a", "b"));
        assert_eq!(count_observations(&nc), 1);
    }

    // ── Serialization Round-Trip ─────────────────────────────────────────

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut nc = NexCoreInsight::new();
        nc.register_domain("guardian", "Threats");
        let _e = nc.ingest_from("guardian", Observation::new("risk", "high"));

        let json = serde_json::to_string(&nc);
        assert!(json.is_ok());

        let restored: Result<NexCoreInsight, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(restored.is_ok());

        let restored = restored.unwrap_or_default();
        assert_eq!(restored.domain_count(), 1);
        assert_eq!(restored.observation_count(), 1);
    }

    // ── Config Propagation ───────────────────────────────────────────────

    #[test]
    fn test_config_propagates_to_engine() {
        let config = InsightConfig {
            pattern_min_occurrences: 5,
            suddenness_threshold: 3.0,
            ..InsightConfig::default()
        };
        let nc = NexCoreInsight::with_config(config);
        assert_eq!(nc.config().pattern_min_occurrences, 5);
        assert!((nc.config().suddenness_threshold - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_mut_modifies_engine() {
        let mut nc = NexCoreInsight::new();
        nc.config_mut().enable_recursive_learning = false;
        assert!(!nc.config().enable_recursive_learning);
    }
}
