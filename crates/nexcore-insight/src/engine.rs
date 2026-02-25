//! # InsightEngine
//!
//! The core engine that orchestrates pattern detection, novelty recognition,
//! connection mapping, observation compression, state tracking, and
//! optional suddenness detection.
//!
//! ## Tier: T3-Domain Specific
//!
//! INSIGHT = <sigma, kappa, mu, exists, state, void, N, boundary>
//! 8 unique T1 primitives => T3

use std::collections::HashMap;

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

// ── Custom serde for HashMap<(String,String), u64> ──────────────────
// JSON requires string keys. We join tuple keys as "a\x1Fb" (unit separator).
mod cooccurrence_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;

    const SEP: char = '\x1F'; // ASCII Unit Separator

    pub fn serialize<S>(
        map: &HashMap<(String, String), u64>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, u64> = map
            .iter()
            .map(|((a, b), v)| (format!("{a}{SEP}{b}"), *v))
            .collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<(String, String), u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, u64> = HashMap::deserialize(deserializer)?;
        Ok(string_map
            .into_iter()
            .filter_map(|(key, v)| {
                key.split_once(SEP)
                    .map(|(a, b)| ((a.to_string(), b.to_string()), v))
            })
            .collect())
    }
}

use crate::composites::{
    Compression, Connection, Novelty, NoveltyReason, Pattern, Recognition, Suddenness,
};

// ============================================================================
// Observation — the input unit
// ============================================================================

/// A single observation fed into the engine.
///
/// Tier: T2-P (Existence + Sequence)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// Unique key identifying this observation.
    pub key: String,
    /// The observed value (stringified for generality).
    pub value: String,
    /// Optional numeric value for threshold comparisons.
    pub numeric_value: Option<f64>,
    /// When this observation was made.
    pub timestamp: DateTime,
    /// Optional tags for grouping.
    pub tags: Vec<String>,
}

impl Observation {
    /// Create a new observation.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            numeric_value: None,
            timestamp: DateTime::now(),
            tags: Vec::new(),
        }
    }

    /// Create a new observation with a numeric value.
    #[must_use]
    pub fn with_numeric(key: impl Into<String>, value: f64) -> Self {
        Self {
            key: key.into(),
            value: format!("{value}"),
            numeric_value: Some(value),
            timestamp: DateTime::now(),
            tags: Vec::new(),
        }
    }

    /// Add a tag to this observation.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

// ============================================================================
// InsightConfig — engine configuration
// ============================================================================

/// Configuration for the InsightEngine.
///
/// Tier: T2-P (Boundary + Quantity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightConfig {
    /// Minimum number of co-occurrences to form a pattern.
    pub pattern_min_occurrences: u64,
    /// Confidence threshold for pattern confirmation.
    pub pattern_confidence_threshold: f64,
    /// Connection strength threshold for significance.
    pub connection_strength_threshold: f64,
    /// Minimum compression ratio to be considered meaningful.
    pub compression_min_ratio: f64,
    /// Whether to enable suddenness detection.
    pub enable_suddenness: bool,
    /// Default threshold for suddenness detection.
    pub suddenness_threshold: f64,
    /// Whether to enable recursive learning (ρ generator).
    ///
    /// When **on** (default): Detected patterns feed back into recognition —
    /// subsequent observations matching a pattern trigger `ObservationRecognized`
    /// instead of `NoveltyDetected`. This is the Chomsky Type-0 feedback loop.
    ///
    /// When **off**: Every observation goes through novelty detection regardless
    /// of existing patterns. Patterns still form (co-occurrence tracking runs),
    /// but they don't influence recognition. Engine operates at Chomsky Type-1.
    ///
    /// ## Grammar Impact
    ///
    /// | State | Generators | Chomsky Level |
    /// |-------|-----------|---------------|
    /// | `true` | {σ, Σ, ρ, κ, ∃} | Type-0 (Turing) |
    /// | `false` | {σ, Σ, κ, ∃} | Type-1 (Linear Bounded) |
    pub enable_recursive_learning: bool,
}

impl Default for InsightConfig {
    fn default() -> Self {
        Self {
            pattern_min_occurrences: 2,
            pattern_confidence_threshold: 0.6,
            connection_strength_threshold: 0.5,
            compression_min_ratio: 2.0,
            enable_suddenness: true,
            suddenness_threshold: 2.0,
            enable_recursive_learning: true,
        }
    }
}

impl InsightConfig {
    /// Apply selective overrides — only replace fields where the caller provided a value.
    ///
    /// This is the primary mechanism for MCP tools: load saved config, then patch
    /// only the fields the user explicitly set. Unspecified fields retain their
    /// persisted values.
    ///
    /// Tier: T2-P (μ + ∂ — mapping with boundary preservation)
    pub fn apply_overrides(
        &mut self,
        pattern_min_occurrences: Option<u64>,
        pattern_confidence_threshold: Option<f64>,
        connection_strength_threshold: Option<f64>,
        compression_min_ratio: Option<f64>,
        enable_suddenness: Option<bool>,
        suddenness_threshold: Option<f64>,
        enable_recursive_learning: Option<bool>,
    ) {
        if let Some(v) = pattern_min_occurrences {
            self.pattern_min_occurrences = v;
        }
        if let Some(v) = pattern_confidence_threshold {
            self.pattern_confidence_threshold = v;
        }
        if let Some(v) = connection_strength_threshold {
            self.connection_strength_threshold = v;
        }
        if let Some(v) = compression_min_ratio {
            self.compression_min_ratio = v;
        }
        if let Some(v) = enable_suddenness {
            self.enable_suddenness = v;
        }
        if let Some(v) = suddenness_threshold {
            self.suddenness_threshold = v;
        }
        if let Some(v) = enable_recursive_learning {
            self.enable_recursive_learning = v;
        }
    }
}

// ============================================================================
// InsightEvent — state change record
// ============================================================================

/// An event recording a state change in understanding.
///
/// Tier: T2-C (State + Sequence + Existence)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsightEvent {
    /// A new pattern was detected.
    PatternDetected(Pattern),
    /// An observation was recognized as part of a pattern.
    ObservationRecognized(Recognition),
    /// A novel observation was found.
    NoveltyDetected(Novelty),
    /// A new connection was established.
    ConnectionEstablished(Connection),
    /// Observations were compressed into a principle.
    ObservationsCompressed(Compression),
    /// A sudden threshold crossing was detected.
    SuddennessTrigger(Suddenness),
}

impl std::fmt::Display for InsightEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatternDetected(p) => write!(f, "EVENT: {p}"),
            Self::ObservationRecognized(r) => write!(f, "EVENT: {r}"),
            Self::NoveltyDetected(n) => write!(f, "EVENT: {n}"),
            Self::ConnectionEstablished(c) => write!(f, "EVENT: {c}"),
            Self::ObservationsCompressed(c) => write!(f, "EVENT: {c}"),
            Self::SuddennessTrigger(s) => write!(f, "EVENT: {s}"),
        }
    }
}

// ============================================================================
// InsightEngine — the T3 orchestrator
// ============================================================================

/// The core insight engine that processes observations and produces insights.
///
/// Orchestrates all 6 T2-C composites:
/// - Pattern detection (sigma + kappa + mu)
/// - Recognition (kappa + exists + sigma)
/// - Novelty detection (void + exists + sigma)
/// - Connection mapping (mu + kappa + state)
/// - Compression (N + mu + kappa)
/// - Suddenness detection (sigma + boundary + N + kappa)
///
/// ## Tier: T3-Domain Specific
///
/// INSIGHT = <sigma, kappa, mu, exists, state, void, N, boundary>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightEngine {
    /// Engine configuration.
    pub config: InsightConfig,
    /// Known patterns (label -> Pattern).
    patterns: HashMap<String, Pattern>,
    /// Known connections.
    connections: Vec<Connection>,
    /// History of observations by key.
    observation_history: HashMap<String, Vec<Observation>>,
    /// Accumulated co-occurrence counts: (key_a, key_b) -> count.
    #[serde(with = "cooccurrence_serde")]
    cooccurrence: HashMap<(String, String), u64>,
    /// Event log (accumulated, append-only).
    events: Vec<InsightEvent>,
    /// Previous numeric values for suddenness detection (key -> last_value).
    previous_values: HashMap<String, f64>,
}

impl InsightEngine {
    /// Create a new engine with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(InsightConfig::default())
    }

    /// Create a new engine with custom configuration.
    #[must_use]
    pub fn with_config(config: InsightConfig) -> Self {
        Self {
            config,
            patterns: HashMap::new(),
            connections: Vec::new(),
            observation_history: HashMap::new(),
            cooccurrence: HashMap::new(),
            events: Vec::new(),
            previous_values: HashMap::new(),
        }
    }

    /// Ingest a single observation, running all detection pipelines.
    ///
    /// Returns a list of events produced by this ingestion.
    pub fn ingest(&mut self, observation: Observation) -> Vec<InsightEvent> {
        let mut produced_events = Vec::new();

        // 1. Check for suddenness (before updating history)
        if self.config.enable_suddenness {
            if let Some(numeric) = observation.numeric_value {
                if let Some(suddenness) = self.check_suddenness(&observation.key, numeric) {
                    let event = InsightEvent::SuddennessTrigger(suddenness);
                    produced_events.push(event);
                }
                self.previous_values
                    .insert(observation.key.clone(), numeric);
            }
        }

        // 2. Try to recognize against existing patterns (ρ generator gate)
        //
        // When enable_recursive_learning is ON: patterns feed back into
        // recognition — Chomsky Type-0 (Turing) with full ρ feedback loop.
        //
        // When OFF: skip recognition entirely, always detect novelty —
        // Chomsky Type-1 (Linear Bounded), no ρ generator active.
        if self.config.enable_recursive_learning {
            let recognition = self.try_recognize(&observation);
            if let Some(rec) = recognition {
                let event = InsightEvent::ObservationRecognized(rec);
                produced_events.push(event);
            } else {
                let novelty = self.detect_novelty(&observation);
                let event = InsightEvent::NoveltyDetected(novelty);
                produced_events.push(event);
            }
        } else {
            // ρ disabled — every observation is treated as novel
            let novelty = self.detect_novelty(&observation);
            let event = InsightEvent::NoveltyDetected(novelty);
            produced_events.push(event);
        }

        // 4. Update co-occurrence counts with other recent observations
        self.update_cooccurrence(&observation);

        // 5. Check for new pattern formation
        if let Some(pattern) = self.detect_pattern(&observation) {
            let event = InsightEvent::PatternDetected(pattern);
            produced_events.push(event);
        }

        // 6. Store the observation
        self.observation_history
            .entry(observation.key.clone())
            .or_default()
            .push(observation);

        // 7. Record all events
        for event in &produced_events {
            self.events.push(event.clone());
        }

        produced_events
    }

    /// Ingest a batch of observations.
    ///
    /// Returns all events produced across all ingestions.
    pub fn ingest_batch(&mut self, observations: Vec<Observation>) -> Vec<InsightEvent> {
        let mut all_events = Vec::new();
        for obs in observations {
            let events = self.ingest(obs);
            all_events.extend(events);
        }
        all_events
    }

    /// Establish a connection between two observation keys.
    pub fn connect(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        relation: impl Into<String>,
        strength: f64,
    ) -> Connection {
        let conn = Connection::new(from, to, relation, strength);
        self.connections.push(conn.clone());
        let event = InsightEvent::ConnectionEstablished(conn.clone());
        self.events.push(event);
        conn
    }

    /// Compress observations with a given tag into a principle.
    ///
    /// Returns `Some(Compression)` if observations with the tag exist,
    /// `None` otherwise.
    pub fn compress_by_tag(
        &mut self,
        tag: &str,
        principle: impl Into<String>,
    ) -> Option<Compression> {
        let mut matching_keys = Vec::new();
        for (key, obs_list) in &self.observation_history {
            for obs in obs_list {
                if obs.tags.contains(&tag.to_string()) {
                    matching_keys.push(key.clone());
                    break;
                }
            }
        }

        if matching_keys.len() < 2 {
            return None;
        }

        let compression = Compression::new(principle, matching_keys);
        let event = InsightEvent::ObservationsCompressed(compression.clone());
        self.events.push(event);
        Some(compression)
    }

    /// Compress observations matching a predicate into a principle.
    pub fn compress_by_keys(
        &mut self,
        keys: Vec<String>,
        principle: impl Into<String>,
    ) -> Compression {
        let compression = Compression::new(principle, keys);
        let event = InsightEvent::ObservationsCompressed(compression.clone());
        self.events.push(event);
        compression
    }

    /// Automatically discover and compress observation groups.
    ///
    /// Three clustering strategies applied in priority order:
    /// 1. **Tag clustering** (κ + μ): Observations sharing tags form groups
    /// 2. **Prefix clustering** (κ): Keys with common prefixes form groups
    /// 3. **Singleton collapse**: Remaining ungrouped keys yield no compression
    ///
    /// Only groups with ≥ 2 members and ratio above `compression_min_ratio`
    /// are returned. Results sorted by quality score (descending).
    ///
    /// Tier: T2-C (N + μ + κ — quantity reduction via mapping and comparison)
    pub fn compress_auto(&mut self) -> Vec<Compression> {
        let mut compressions = Vec::new();
        let mut claimed_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Strategy 1: Tag-based clustering
        let mut tag_groups: HashMap<String, Vec<String>> = HashMap::new();
        for (key, obs_list) in &self.observation_history {
            for obs in obs_list {
                for tag in &obs.tags {
                    tag_groups.entry(tag.clone()).or_default().push(key.clone());
                }
            }
        }
        // Deduplicate within each group
        for members in tag_groups.values_mut() {
            members.sort();
            members.dedup();
        }
        // Sort tag groups by size (largest first)
        let mut sorted_tags: Vec<_> = tag_groups.into_iter().collect();
        sorted_tags.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (tag, members) in sorted_tags {
            let unclaimed: Vec<String> = members
                .into_iter()
                .filter(|k| !claimed_keys.contains(k))
                .collect();
            if unclaimed.len() >= 2 {
                let principle = format!("tag:{tag}");
                for k in &unclaimed {
                    claimed_keys.insert(k.clone());
                }
                let compression = Compression::new(principle, unclaimed);
                if compression.ratio >= self.config.compression_min_ratio {
                    let event = InsightEvent::ObservationsCompressed(compression.clone());
                    self.events.push(event);
                    compressions.push(compression);
                }
            }
        }

        // Strategy 2: Prefix-based clustering for unclaimed keys
        let unclaimed_keys: Vec<String> = self
            .observation_history
            .keys()
            .filter(|k| !claimed_keys.contains(*k))
            .cloned()
            .collect();

        if unclaimed_keys.len() >= 2 {
            let mut prefix_groups: HashMap<String, Vec<String>> = HashMap::new();
            for key in &unclaimed_keys {
                // Extract prefix (up to first '_', '.', or '-')
                let prefix = key
                    .find(|c: char| c == '_' || c == '.' || c == '-')
                    .map(|i| &key[..i])
                    .unwrap_or(key);
                if !prefix.is_empty() && prefix.len() < key.len() {
                    prefix_groups
                        .entry(prefix.to_string())
                        .or_default()
                        .push(key.clone());
                }
            }
            for (prefix, members) in prefix_groups {
                let unclaimed: Vec<String> = members
                    .into_iter()
                    .filter(|k| !claimed_keys.contains(k))
                    .collect();
                if unclaimed.len() >= 2 {
                    let principle = format!("prefix:{prefix}");
                    for k in &unclaimed {
                        claimed_keys.insert(k.clone());
                    }
                    let compression = Compression::new(principle, unclaimed);
                    if compression.ratio >= self.config.compression_min_ratio {
                        let event = InsightEvent::ObservationsCompressed(compression.clone());
                        self.events.push(event);
                        compressions.push(compression);
                    }
                }
            }
        }

        // Sort by quality (highest first)
        compressions.sort_by(|a, b| {
            b.quality()
                .partial_cmp(&a.quality())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        compressions
    }

    /// Returns all accumulated events.
    #[must_use]
    pub fn events(&self) -> &[InsightEvent] {
        &self.events
    }

    /// Returns the number of known patterns.
    #[must_use]
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Returns all known patterns.
    #[must_use]
    pub fn patterns(&self) -> Vec<&Pattern> {
        self.patterns.values().collect()
    }

    /// Returns all known connections.
    #[must_use]
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Returns connections involving a specific key.
    #[must_use]
    pub fn connections_for(&self, key: &str) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| c.involves(key))
            .collect()
    }

    /// Returns the total number of observations ingested.
    #[must_use]
    pub fn observation_count(&self) -> usize {
        self.observation_history.values().map(Vec::len).sum()
    }

    /// Returns the number of unique observation keys.
    #[must_use]
    pub fn unique_key_count(&self) -> usize {
        self.observation_history.len()
    }

    // ── Private detection methods ────────────────────────────────────────

    /// Try to recognize an observation against known patterns.
    fn try_recognize(&mut self, observation: &Observation) -> Option<Recognition> {
        for pattern in self.patterns.values_mut() {
            if pattern.members.contains(&observation.key) {
                pattern.record_occurrence();
                let strength = pattern.confidence;
                return Some(Recognition::new(
                    observation.key.clone(),
                    pattern.id,
                    strength,
                ));
            }
        }
        None
    }

    /// Detect novelty for an observation not recognized by any pattern.
    fn detect_novelty(&self, observation: &Observation) -> Novelty {
        if self.observation_history.contains_key(&observation.key) {
            // Key seen before but not in a pattern => mild novelty
            Novelty::deviation(
                observation.key.clone(),
                "known key",
                observation.value.clone(),
                0.3,
            )
        } else {
            // Completely new key
            Novelty::new_observation(observation.key.clone())
        }
    }

    /// Update co-occurrence counts between this observation and recent ones.
    fn update_cooccurrence(&mut self, observation: &Observation) {
        let current_key = &observation.key;
        for existing_key in self.observation_history.keys() {
            if existing_key != current_key {
                let pair = if current_key < existing_key {
                    (current_key.clone(), existing_key.clone())
                } else {
                    (existing_key.clone(), current_key.clone())
                };
                *self.cooccurrence.entry(pair).or_insert(0) += 1;
            }
        }
    }

    /// Check if co-occurrence counts warrant a new pattern.
    fn detect_pattern(&mut self, observation: &Observation) -> Option<Pattern> {
        let current_key = &observation.key;
        for (pair, count) in &self.cooccurrence {
            if *count >= self.config.pattern_min_occurrences {
                let involves_current = pair.0 == *current_key || pair.1 == *current_key;
                if involves_current {
                    let label = format!("{}+{}", pair.0, pair.1);
                    if !self.patterns.contains_key(&label) {
                        let members = vec![pair.0.clone(), pair.1.clone()];
                        let mut pattern = Pattern::new(label.clone(), members);
                        // Set confidence based on occurrence count
                        for _ in 1..*count {
                            pattern.record_occurrence();
                        }
                        self.patterns.insert(label, pattern.clone());
                        return Some(pattern);
                    }
                }
            }
        }
        None
    }

    /// Check for sudden threshold crossing.
    fn check_suddenness(&self, key: &str, current_value: f64) -> Option<Suddenness> {
        if let Some(&previous) = self.previous_values.get(key) {
            let change = (current_value - previous).abs();
            let threshold = self.config.suddenness_threshold;
            if change >= threshold {
                return Some(Suddenness::new(key, threshold, previous, current_value));
            }
        }
        None
    }
}

impl Default for InsightEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Insight trait implementation ────────────────────────────────────────────
//
// The concrete InsightEngine implements the system-level Insight trait.
// This makes InsightEngine interchangeable with any other Insight implementor
// (e.g., NexCore's system-level PV pipeline).

impl crate::traits::Insight for InsightEngine {
    type Obs = Observation;

    fn ingest(&mut self, observation: Self::Obs) -> Vec<InsightEvent> {
        // Delegates to the inherent method — same 6-stage pipeline.
        InsightEngine::ingest(self, observation)
    }

    fn ingest_batch(&mut self, observations: Vec<Self::Obs>) -> Vec<InsightEvent> {
        InsightEngine::ingest_batch(self, observations)
    }

    fn connect(&mut self, from: &str, to: &str, relation: &str, strength: f64) -> Connection {
        InsightEngine::connect(self, from, to, relation, strength)
    }

    fn compress(&mut self, keys: Vec<String>, principle: &str) -> Compression {
        InsightEngine::compress_by_keys(self, keys, principle)
    }

    fn events(&self) -> &[InsightEvent] {
        InsightEngine::events(self)
    }

    fn observation_count(&self) -> usize {
        InsightEngine::observation_count(self)
    }

    fn pattern_count(&self) -> usize {
        InsightEngine::pattern_count(self)
    }

    fn patterns(&self) -> Vec<&Pattern> {
        InsightEngine::patterns(self)
    }

    fn connections(&self) -> &[Connection] {
        InsightEngine::connections(self)
    }

    fn connections_for(&self, key: &str) -> Vec<&Connection> {
        InsightEngine::connections_for(self, key)
    }

    fn unique_key_count(&self) -> usize {
        InsightEngine::unique_key_count(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = InsightEngine::new();
        assert_eq!(engine.pattern_count(), 0);
        assert_eq!(engine.observation_count(), 0);
        assert_eq!(engine.unique_key_count(), 0);
        assert!(engine.events().is_empty());
    }

    #[test]
    fn test_engine_with_config() {
        let config = InsightConfig {
            pattern_min_occurrences: 5,
            ..InsightConfig::default()
        };
        let engine = InsightEngine::with_config(config);
        assert_eq!(engine.config.pattern_min_occurrences, 5);
    }

    #[test]
    fn test_ingest_single_observation() {
        let mut engine = InsightEngine::new();
        let obs = Observation::new("drug_a", "observed_effect");
        let events = engine.ingest(obs);

        assert_eq!(engine.observation_count(), 1);
        assert_eq!(engine.unique_key_count(), 1);
        // First observation should produce novelty
        assert!(!events.is_empty());
        let has_novelty = events
            .iter()
            .any(|e| matches!(e, InsightEvent::NoveltyDetected(_)));
        assert!(has_novelty);
    }

    #[test]
    fn test_novelty_for_new_observation() {
        let mut engine = InsightEngine::new();
        let obs = Observation::new("never_seen_before", "value");
        let events = engine.ingest(obs);

        let novelties: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                InsightEvent::NoveltyDetected(n) => Some(n),
                _ => None,
            })
            .collect();

        assert_eq!(novelties.len(), 1);
        assert!(novelties[0].is_highly_novel());
        assert_eq!(novelties[0].reason, NoveltyReason::NoMatchingPattern);
    }

    #[test]
    fn test_reduced_novelty_for_repeat_key() {
        let mut engine = InsightEngine::new();
        let _events1 = engine.ingest(Observation::new("key_a", "value1"));
        let events2 = engine.ingest(Observation::new("key_a", "value2"));

        let novelties: Vec<_> = events2
            .iter()
            .filter_map(|e| match e {
                InsightEvent::NoveltyDetected(n) => Some(n),
                _ => None,
            })
            .collect();

        // Second observation of same key has lower novelty
        if let Some(n) = novelties.first() {
            assert!(!n.is_highly_novel());
        }
    }

    #[test]
    fn test_pattern_detection_via_cooccurrence() {
        let mut config = InsightConfig::default();
        config.pattern_min_occurrences = 2;
        let mut engine = InsightEngine::with_config(config);

        // First round: key_a and key_b co-occur
        let _e1 = engine.ingest(Observation::new("key_a", "v1"));
        let _e2 = engine.ingest(Observation::new("key_b", "v2"));

        // Second ingestion of key_a bumps cooccurrence(key_a, key_b) to 2
        // which triggers pattern detection
        let e3 = engine.ingest(Observation::new("key_a", "v3"));

        let patterns: Vec<_> = e3
            .iter()
            .filter_map(|e| match e {
                InsightEvent::PatternDetected(p) => Some(p),
                _ => None,
            })
            .collect();

        assert_eq!(patterns.len(), 1);
        assert_eq!(engine.pattern_count(), 1);
    }

    #[test]
    fn test_recognition_after_pattern() {
        let mut config = InsightConfig::default();
        config.pattern_min_occurrences = 2;
        let mut engine = InsightEngine::with_config(config);

        // Build up a pattern
        let _e1 = engine.ingest(Observation::new("alpha", "v1"));
        let _e2 = engine.ingest(Observation::new("beta", "v2"));
        let _e3 = engine.ingest(Observation::new("alpha", "v3"));
        let _e4 = engine.ingest(Observation::new("beta", "v4"));

        // Now alpha should be recognized
        let e5 = engine.ingest(Observation::new("alpha", "v5"));
        let recognitions: Vec<_> = e5
            .iter()
            .filter_map(|e| match e {
                InsightEvent::ObservationRecognized(r) => Some(r),
                _ => None,
            })
            .collect();

        assert_eq!(recognitions.len(), 1);
        assert_eq!(recognitions[0].recognized_key, "alpha");
    }

    #[test]
    fn test_connection_establishment() {
        let mut engine = InsightEngine::new();
        let conn = engine.connect("drug_x", "side_effect_y", "causes", 0.85);

        assert_eq!(conn.from, "drug_x");
        assert_eq!(conn.to, "side_effect_y");
        assert!((conn.strength - 0.85).abs() < f64::EPSILON);
        assert_eq!(engine.connections().len(), 1);
    }

    #[test]
    fn test_connections_for_key() {
        let mut engine = InsightEngine::new();
        let _c1 = engine.connect("a", "b", "related", 0.5);
        let _c2 = engine.connect("a", "c", "related", 0.7);
        let _c3 = engine.connect("d", "e", "unrelated", 0.3);

        let conns = engine.connections_for("a");
        assert_eq!(conns.len(), 2);

        let conns_d = engine.connections_for("d");
        assert_eq!(conns_d.len(), 1);
    }

    #[test]
    fn test_compression_by_tag() {
        let mut engine = InsightEngine::new();

        let obs1 = Observation::new("obs_1", "val_1").with_tag("headache");
        let obs2 = Observation::new("obs_2", "val_2").with_tag("headache");
        let obs3 = Observation::new("obs_3", "val_3").with_tag("nausea");
        let obs4 = Observation::new("obs_4", "val_4").with_tag("headache");

        let _e1 = engine.ingest(obs1);
        let _e2 = engine.ingest(obs2);
        let _e3 = engine.ingest(obs3);
        let _e4 = engine.ingest(obs4);

        let compression = engine.compress_by_tag("headache", "Headache is a common adverse event");
        assert!(compression.is_some());
        if let Some(comp) = compression {
            assert_eq!(comp.input_count, 3);
            // ratio = 3/1 = 3.0 which is > 2.0
            assert!(comp.is_meaningful());
        }
    }

    #[test]
    fn test_compression_insufficient_data() {
        let mut engine = InsightEngine::new();
        let obs1 = Observation::new("obs_1", "val_1").with_tag("rare");
        let _e1 = engine.ingest(obs1);

        let compression = engine.compress_by_tag("rare", "Not enough data");
        assert!(compression.is_none());
    }

    #[test]
    fn test_compress_by_keys() {
        let mut engine = InsightEngine::new();
        let keys = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let compression = engine.compress_by_keys(keys, "ABC principle");
        assert_eq!(compression.input_count, 3);
        assert_eq!(compression.output_count, 1);
        assert!(compression.is_meaningful());
    }

    #[test]
    fn test_suddenness_detection() {
        let mut config = InsightConfig::default();
        config.suddenness_threshold = 5.0;
        let mut engine = InsightEngine::with_config(config);

        // First observation sets the baseline
        let _e1 = engine.ingest(Observation::with_numeric("metric_x", 10.0));

        // Second observation with a jump of 10 (above threshold of 5)
        let e2 = engine.ingest(Observation::with_numeric("metric_x", 20.0));

        let suddenness: Vec<_> = e2
            .iter()
            .filter_map(|e| match e {
                InsightEvent::SuddennessTrigger(s) => Some(s),
                _ => None,
            })
            .collect();

        assert_eq!(suddenness.len(), 1);
        assert!((suddenness[0].magnitude() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_suddenness_below_threshold() {
        let mut config = InsightConfig::default();
        config.suddenness_threshold = 5.0;
        let mut engine = InsightEngine::with_config(config);

        let _e1 = engine.ingest(Observation::with_numeric("metric_x", 10.0));
        let e2 = engine.ingest(Observation::with_numeric("metric_x", 12.0));

        let suddenness_count = e2
            .iter()
            .filter(|e| matches!(e, InsightEvent::SuddennessTrigger(_)))
            .count();

        assert_eq!(suddenness_count, 0);
    }

    #[test]
    fn test_suddenness_disabled() {
        let mut config = InsightConfig::default();
        config.enable_suddenness = false;
        let mut engine = InsightEngine::with_config(config);

        let _e1 = engine.ingest(Observation::with_numeric("metric_x", 10.0));
        let e2 = engine.ingest(Observation::with_numeric("metric_x", 100.0));

        let suddenness_count = e2
            .iter()
            .filter(|e| matches!(e, InsightEvent::SuddennessTrigger(_)))
            .count();

        assert_eq!(suddenness_count, 0);
    }

    #[test]
    fn test_ingest_batch() {
        let mut engine = InsightEngine::new();
        let batch = vec![
            Observation::new("a", "v1"),
            Observation::new("b", "v2"),
            Observation::new("c", "v3"),
        ];
        let events = engine.ingest_batch(batch);
        assert!(!events.is_empty());
        assert_eq!(engine.observation_count(), 3);
    }

    #[test]
    fn test_event_display() {
        let pattern = Pattern::new("test_pattern", vec!["a".to_string(), "b".to_string()]);
        let event = InsightEvent::PatternDetected(pattern);
        let display = format!("{event}");
        assert!(display.contains("EVENT:"));
        assert!(display.contains("Pattern"));
    }

    #[test]
    fn test_observation_with_tag() {
        let obs = Observation::new("key", "value")
            .with_tag("tag1")
            .with_tag("tag2");
        assert_eq!(obs.tags.len(), 2);
        assert!(obs.tags.contains(&"tag1".to_string()));
        assert!(obs.tags.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_default_engine() {
        let engine = InsightEngine::default();
        assert_eq!(engine.pattern_count(), 0);
    }

    // ── Compression algorithm tests ─────────────────────────────────────

    #[test]
    fn test_compress_auto_tag_clustering() {
        let mut engine = InsightEngine::new();

        // Ingest observations with shared tags
        let _e1 = engine.ingest(Observation::new("drug_a", "effect1").with_tag("hepatic"));
        let _e2 = engine.ingest(Observation::new("drug_b", "effect2").with_tag("hepatic"));
        let _e3 = engine.ingest(Observation::new("drug_c", "effect3").with_tag("hepatic"));
        let _e4 = engine.ingest(Observation::new("drug_d", "effect4").with_tag("renal"));
        let _e5 = engine.ingest(Observation::new("drug_e", "effect5").with_tag("renal"));

        let compressions = engine.compress_auto();

        // Should find at least the hepatic group (3 members)
        assert!(!compressions.is_empty());
        let hepatic = compressions.iter().find(|c| c.principle == "tag:hepatic");
        assert!(hepatic.is_some());
        let h = hepatic.unwrap();
        assert_eq!(h.input_count, 3);
        assert!(h.is_meaningful()); // ratio = 3.0 > 2.0
        assert!(h.quality() > 0.0);
    }

    #[test]
    fn test_compress_auto_prefix_clustering() {
        let mut engine = InsightEngine::new();

        // Ingest observations with shared key prefixes (no tags)
        let _e1 = engine.ingest(Observation::new("liver_ast", "high"));
        let _e2 = engine.ingest(Observation::new("liver_alt", "elevated"));
        let _e3 = engine.ingest(Observation::new("liver_bilirubin", "normal"));
        let _e4 = engine.ingest(Observation::new("kidney_creatinine", "high"));

        let compressions = engine.compress_auto();

        // Should find the liver prefix group
        let liver = compressions.iter().find(|c| c.principle == "prefix:liver");
        assert!(liver.is_some());
        let l = liver.unwrap();
        assert_eq!(l.input_count, 3);
    }

    #[test]
    fn test_compress_auto_empty_engine() {
        let mut engine = InsightEngine::new();
        let compressions = engine.compress_auto();
        assert!(compressions.is_empty());
    }

    #[test]
    fn test_compress_auto_single_observation() {
        let mut engine = InsightEngine::new();
        let _e1 = engine.ingest(Observation::new("solo", "value"));
        let compressions = engine.compress_auto();
        assert!(compressions.is_empty());
    }

    #[test]
    fn test_compress_auto_sorted_by_quality() {
        let mut engine = InsightEngine::new();

        // Create a large tag group (higher quality)
        for i in 0..5 {
            let obs = Observation::new(format!("big_{i}"), "val").with_tag("large_group");
            let _e = engine.ingest(obs);
        }

        // Create a small tag group (lower quality)
        let _e1 = engine.ingest(Observation::new("sm_a", "val").with_tag("small_group"));
        let _e2 = engine.ingest(Observation::new("sm_b", "val").with_tag("small_group"));

        let compressions = engine.compress_auto();
        assert!(compressions.len() >= 2);
        // First should have higher quality than second
        assert!(compressions[0].quality() >= compressions[1].quality());
    }

    #[test]
    fn test_compression_entropy_bits() {
        let comp = Compression::new("test", vec!["a".into(), "b".into(), "c".into(), "d".into()]);
        // log₂(4) = 2.0
        assert!((comp.entropy_bits() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compression_entropy_single() {
        let comp = Compression::new("test", vec!["a".into()]);
        assert!((comp.entropy_bits() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compression_quality_meaningful() {
        let comp = Compression::new(
            "test",
            vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()],
        );
        // ratio = 5, entropy = log₂(5) ≈ 2.32
        assert!(comp.quality() > 0.5);
    }

    #[test]
    fn test_compression_quality_trivial() {
        let comp = Compression::new("test", vec!["a".into()]);
        assert!((comp.quality() - 0.0).abs() < f64::EPSILON);
    }

    // ── Trait-based tests ─────────────────────────────────────────────────
    //
    // These tests exercise InsightEngine through the Insight trait interface,
    // proving the contract works for any implementor, not just the concrete struct.

    /// Helper that accepts any Insight implementor — proves trait polymorphism.
    fn run_through_trait<E: crate::traits::Insight<Obs = Observation>>(engine: &mut E) -> usize {
        let events = engine.ingest(Observation::new("trait_key", "trait_val"));
        let count = engine.observation_count();
        assert!(!events.is_empty());
        count
    }

    #[test]
    fn test_trait_polymorphism() {
        let mut engine = InsightEngine::new();
        let count = run_through_trait(&mut engine);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_trait_connect() {
        use crate::traits::Insight;
        let mut engine = InsightEngine::new();
        let conn = Insight::connect(&mut engine, "a", "b", "causes", 0.9);
        assert_eq!(conn.from, "a");
        assert_eq!(conn.to, "b");
        assert_eq!(Insight::connections(&engine).len(), 1);
    }

    #[test]
    fn test_trait_compress() {
        use crate::traits::Insight;
        let mut engine = InsightEngine::new();
        let comp = Insight::compress(
            &mut engine,
            vec!["x".into(), "y".into(), "z".into()],
            "XYZ principle",
        );
        assert_eq!(comp.input_count, 3);
        assert!(comp.is_meaningful());
    }

    #[test]
    fn test_trait_batch_ingest() {
        use crate::traits::Insight;
        let mut engine = InsightEngine::new();
        let batch = vec![Observation::new("t1", "v1"), Observation::new("t2", "v2")];
        let events = Insight::ingest_batch(&mut engine, batch);
        assert!(!events.is_empty());
        assert_eq!(Insight::observation_count(&engine), 2);
        assert_eq!(Insight::unique_key_count(&engine), 2);
    }

    // ── Recursive learning switch tests ────────────────────────────────

    #[test]
    fn test_recursive_learning_on_produces_recognition() {
        let config = InsightConfig {
            pattern_min_occurrences: 2,
            enable_recursive_learning: true,
            ..InsightConfig::default()
        };
        let mut engine = InsightEngine::with_config(config);

        // Build a pattern: alpha+beta co-occur twice
        let _e1 = engine.ingest(Observation::new("alpha", "v1"));
        let _e2 = engine.ingest(Observation::new("beta", "v2"));
        let _e3 = engine.ingest(Observation::new("alpha", "v3"));
        let _e4 = engine.ingest(Observation::new("beta", "v4"));

        // Pattern exists now — next alpha should be RECOGNIZED (ρ feedback)
        let e5 = engine.ingest(Observation::new("alpha", "v5"));
        let has_recognition = e5
            .iter()
            .any(|e| matches!(e, InsightEvent::ObservationRecognized(_)));
        assert!(
            has_recognition,
            "ρ ON: should produce ObservationRecognized"
        );
    }

    #[test]
    fn test_recursive_learning_off_always_novelty() {
        let config = InsightConfig {
            pattern_min_occurrences: 2,
            enable_recursive_learning: false,
            ..InsightConfig::default()
        };
        let mut engine = InsightEngine::with_config(config);

        // Build up the same co-occurrence pattern
        let _e1 = engine.ingest(Observation::new("alpha", "v1"));
        let _e2 = engine.ingest(Observation::new("beta", "v2"));
        let _e3 = engine.ingest(Observation::new("alpha", "v3"));
        let _e4 = engine.ingest(Observation::new("beta", "v4"));

        // Pattern exists — but ρ is OFF, so alpha should still be NOVELTY
        let e5 = engine.ingest(Observation::new("alpha", "v5"));
        let has_recognition = e5
            .iter()
            .any(|e| matches!(e, InsightEvent::ObservationRecognized(_)));
        let has_novelty = e5
            .iter()
            .any(|e| matches!(e, InsightEvent::NoveltyDetected(_)));
        assert!(
            !has_recognition,
            "ρ OFF: should NOT produce ObservationRecognized"
        );
        assert!(has_novelty, "ρ OFF: should produce NoveltyDetected instead");
    }

    #[test]
    fn test_recursive_learning_off_still_forms_patterns() {
        let config = InsightConfig {
            pattern_min_occurrences: 2,
            enable_recursive_learning: false,
            ..InsightConfig::default()
        };
        let mut engine = InsightEngine::with_config(config);

        // Co-occurrence should still track and form patterns
        let _e1 = engine.ingest(Observation::new("x", "v1"));
        let _e2 = engine.ingest(Observation::new("y", "v2"));
        let e3 = engine.ingest(Observation::new("x", "v3"));

        // Pattern detection still fires (co-occurrence threshold met)
        let has_pattern = e3
            .iter()
            .any(|e| matches!(e, InsightEvent::PatternDetected(_)));
        assert!(
            has_pattern,
            "ρ OFF: patterns still form (co-occurrence tracking runs)"
        );
        assert!(engine.pattern_count() > 0);
    }

    #[test]
    fn test_recursive_learning_default_is_on() {
        let config = InsightConfig::default();
        assert!(
            config.enable_recursive_learning,
            "default should enable recursive learning"
        );
    }

    #[test]
    fn test_trait_connections_for() {
        use crate::traits::Insight;
        let mut engine = InsightEngine::new();
        Insight::connect(&mut engine, "drug", "effect1", "causes", 0.8);
        Insight::connect(&mut engine, "drug", "effect2", "causes", 0.6);
        Insight::connect(&mut engine, "other", "effect3", "causes", 0.5);

        let drug_conns = Insight::connections_for(&engine, "drug");
        assert_eq!(drug_conns.len(), 2);
    }
}
