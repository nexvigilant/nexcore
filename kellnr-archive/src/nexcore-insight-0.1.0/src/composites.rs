//! # T2-C Composite Types for INSIGHT
//!
//! The 6 composites that decompose from the INSIGHT concept:
//!
//! | Composite | Formula | Tier |
//! |-----------|---------|------|
//! | Pattern | sigma + kappa + mu | T2-C |
//! | Recognition | kappa + exists + sigma | T2-C |
//! | Novelty | void + exists + sigma | T2-C |
//! | Connection | mu + kappa + state | T2-C |
//! | Compression | N + mu + kappa | T2-C |
//! | Suddenness | sigma + boundary + N + kappa | T2-C |

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// PATTERN = sigma + kappa + mu
// ============================================================================

/// A detected pattern across a sequence of observations.
///
/// Decomposes to: sigma (temporal ordering) + kappa (identity-matching)
/// + mu (mapping between elements).
///
/// Tier: T2-C (3 unique primitives: sigma, kappa, mu)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    /// Unique identifier for this pattern.
    pub id: Uuid,
    /// Human-readable label describing the pattern.
    pub label: String,
    /// The observation keys that participate in this pattern.
    pub members: Vec<String>,
    /// How many times this pattern has been observed.
    pub occurrence_count: u64,
    /// Confidence that this is a real pattern (0.0-1.0).
    pub confidence: f64,
    /// When the pattern was first detected.
    pub first_seen: DateTime<Utc>,
    /// When the pattern was most recently confirmed.
    pub last_seen: DateTime<Utc>,
}

impl Pattern {
    /// Create a new pattern from initial member keys.
    #[must_use]
    pub fn new(label: impl Into<String>, members: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            members,
            occurrence_count: 1,
            confidence: 0.5,
            first_seen: now,
            last_seen: now,
        }
    }

    /// Record another occurrence, increasing confidence.
    pub fn record_occurrence(&mut self) {
        self.occurrence_count += 1;
        self.last_seen = Utc::now();
        // Asymptotic confidence growth: approaches 1.0 but never reaches it
        self.confidence = 1.0 - (1.0 / (self.occurrence_count as f64 + 1.0));
    }

    /// Returns true if confidence exceeds the given threshold.
    #[must_use]
    pub fn is_above_threshold(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pattern({}, members={}, conf={:.2}, n={})",
            self.label,
            self.members.len(),
            self.confidence,
            self.occurrence_count,
        )
    }
}

// ============================================================================
// RECOGNITION = kappa + exists + sigma
// ============================================================================

/// Recognition of an element as matching prior knowledge.
///
/// Decomposes to: kappa (comparison/matching) + exists (something is present)
/// + sigma (temporal ordering of before vs now).
///
/// Tier: T2-C (3 unique primitives: kappa, exists, sigma)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recognition {
    /// The key that was recognized.
    pub recognized_key: String,
    /// Which pattern it was recognized as belonging to.
    pub pattern_id: Uuid,
    /// Match strength (0.0-1.0).
    pub match_strength: f64,
    /// When this recognition occurred.
    pub timestamp: DateTime<Utc>,
}

impl Recognition {
    /// Create a new recognition event.
    #[must_use]
    pub fn new(recognized_key: impl Into<String>, pattern_id: Uuid, match_strength: f64) -> Self {
        Self {
            recognized_key: recognized_key.into(),
            pattern_id,
            match_strength: match_strength.clamp(0.0, 1.0),
            timestamp: Utc::now(),
        }
    }

    /// Returns true if this is a strong recognition (above 0.8).
    #[must_use]
    pub fn is_strong(&self) -> bool {
        self.match_strength >= 0.8
    }
}

impl std::fmt::Display for Recognition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Recognition({} -> pattern {}, strength={:.2})",
            self.recognized_key, self.pattern_id, self.match_strength,
        )
    }
}

// ============================================================================
// NOVELTY = void + exists + sigma
// ============================================================================

/// Detection that an observation is novel (NOT in prior state).
///
/// Decomposes to: void (negation/absence in prior knowledge)
/// + exists (the observation itself exists) + sigma (temporal comparison).
///
/// Tier: T2-C (3 unique primitives: void, exists, sigma)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Novelty {
    /// The observation key that is novel.
    pub novel_key: String,
    /// Why it is considered novel (what was searched and not found).
    pub reason: NoveltyReason,
    /// Novelty score (0.0 = slightly novel, 1.0 = completely unprecedented).
    pub score: f64,
    /// When the novelty was detected.
    pub timestamp: DateTime<Utc>,
}

/// Reason an observation is considered novel.
///
/// Tier: T2-P (2 unique primitives: void + comparison)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NoveltyReason {
    /// No matching pattern exists in the knowledge base.
    NoMatchingPattern,
    /// The key exists but the value deviates significantly.
    SignificantDeviation {
        /// The expected value range.
        expected: String,
        /// What was actually observed.
        actual: String,
    },
    /// A new relationship between known elements.
    NewRelationship {
        /// First element.
        from: String,
        /// Second element.
        to: String,
    },
}

impl Novelty {
    /// Create a novelty detection for a completely new observation.
    #[must_use]
    pub fn new_observation(key: impl Into<String>) -> Self {
        Self {
            novel_key: key.into(),
            reason: NoveltyReason::NoMatchingPattern,
            score: 1.0,
            timestamp: Utc::now(),
        }
    }

    /// Create a novelty detection for a significant deviation.
    #[must_use]
    pub fn deviation(
        key: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
        score: f64,
    ) -> Self {
        Self {
            novel_key: key.into(),
            reason: NoveltyReason::SignificantDeviation {
                expected: expected.into(),
                actual: actual.into(),
            },
            score: score.clamp(0.0, 1.0),
            timestamp: Utc::now(),
        }
    }

    /// Returns true if this is highly novel (score >= 0.9).
    #[must_use]
    pub fn is_highly_novel(&self) -> bool {
        self.score >= 0.9
    }
}

impl std::fmt::Display for Novelty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Novelty({}, score={:.2}, reason={:?})",
            self.novel_key, self.score, self.reason,
        )
    }
}

// ============================================================================
// CONNECTION = mu + kappa + state
// ============================================================================

/// A connection between previously unlinked elements.
///
/// Decomposes to: mu (mapping/linking) + kappa (identity comparison)
/// + state (the connection changes understanding).
///
/// Tier: T2-C (3 unique primitives: mu, kappa, state)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    /// Unique identifier for this connection.
    pub id: Uuid,
    /// Source element key.
    pub from: String,
    /// Target element key.
    pub to: String,
    /// The nature of the connection.
    pub relation: String,
    /// Strength of the connection (0.0-1.0).
    pub strength: f64,
    /// When this connection was established.
    pub established: DateTime<Utc>,
}

impl Connection {
    /// Create a new connection between two elements.
    #[must_use]
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        relation: impl Into<String>,
        strength: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: from.into(),
            to: to.into(),
            relation: relation.into(),
            strength: strength.clamp(0.0, 1.0),
            established: Utc::now(),
        }
    }

    /// Returns true if this connection involves the given key.
    #[must_use]
    pub fn involves(&self, key: &str) -> bool {
        self.from == key || self.to == key
    }
}

impl std::fmt::Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Connection({} --[{}]--> {}, strength={:.2})",
            self.from, self.relation, self.to, self.strength,
        )
    }
}

// ============================================================================
// COMPRESSION = N + mu + kappa
// ============================================================================

/// Compression of many observations into fewer explanatory principles.
///
/// Decomposes to: N (quantity of observations reduced)
/// + mu (mapping many to few) + kappa (comparing to find commonalities).
///
/// Tier: T2-C (3 unique primitives: N, mu, kappa)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Compression {
    /// The principle that explains the compressed observations.
    pub principle: String,
    /// How many raw observations were compressed.
    pub input_count: usize,
    /// How many principles remain after compression.
    pub output_count: usize,
    /// The observation keys that were compressed.
    pub compressed_keys: Vec<String>,
    /// Compression ratio (input_count / output_count). Higher = more compression.
    pub ratio: f64,
}

impl Compression {
    /// Create a new compression result.
    #[must_use]
    pub fn new(principle: impl Into<String>, compressed_keys: Vec<String>) -> Self {
        let input_count = compressed_keys.len();
        let output_count = 1;
        let ratio = if output_count > 0 {
            input_count as f64 / output_count as f64
        } else {
            0.0
        };
        Self {
            principle: principle.into(),
            input_count,
            output_count,
            compressed_keys,
            ratio,
        }
    }

    /// Returns true if meaningful compression occurred (ratio > 2.0).
    #[must_use]
    pub fn is_meaningful(&self) -> bool {
        self.ratio > 2.0
    }

    /// Information entropy of the compressed keys (bits).
    ///
    /// Uses Shannon entropy: H = -Σ p(x) log₂ p(x) where p(x) = 1/n for
    /// uniformly distributed keys. This measures the information content
    /// that was reduced by compression.
    ///
    /// Tier: T1 (N — pure quantity computation)
    #[must_use]
    pub fn entropy_bits(&self) -> f64 {
        if self.input_count <= 1 {
            return 0.0;
        }
        // Uniform distribution entropy = log₂(n)
        (self.input_count as f64).log2()
    }

    /// Compression quality score (0.0 - 1.0).
    ///
    /// Combines ratio and entropy into a single quality metric:
    /// quality = (1 - 1/ratio) × (entropy / max_entropy)
    ///
    /// - High ratio + high entropy = high quality (many diverse items compressed)
    /// - Low ratio = low quality (not much compression)
    /// - Low entropy = lower quality (not much information reduced)
    #[must_use]
    pub fn quality(&self) -> f64 {
        if self.input_count <= 1 || self.ratio <= 1.0 {
            return 0.0;
        }
        let ratio_component = 1.0 - (1.0 / self.ratio);
        let entropy = self.entropy_bits();
        let max_entropy = (self.input_count as f64).log2();
        let entropy_component = if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        };
        ratio_component * entropy_component
    }
}

impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Compression({} -> {} principles, ratio={:.1}x, \"{}\")",
            self.input_count, self.output_count, self.ratio, self.principle,
        )
    }
}

// ============================================================================
// SUDDENNESS = sigma + boundary + N + kappa (optional composite)
// ============================================================================

/// Detection of a sudden threshold crossing that triggers recognition.
///
/// Decomposes to: sigma (temporal ordering) + boundary (threshold crossed)
/// + N (quantitative measure) + kappa (comparison against threshold).
///
/// Tier: T2-C (4 unique primitives: sigma, boundary, N, kappa)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suddenness {
    /// What quantity crossed a threshold.
    pub metric: String,
    /// The threshold that was crossed.
    pub threshold: f64,
    /// The value at the moment of crossing.
    pub value_at_crossing: f64,
    /// The value immediately before crossing.
    pub value_before: f64,
    /// When the crossing was detected.
    pub detected_at: DateTime<Utc>,
    /// Rate of change at the crossing point.
    pub rate_of_change: f64,
}

impl Suddenness {
    /// Create a new suddenness detection.
    #[must_use]
    pub fn new(
        metric: impl Into<String>,
        threshold: f64,
        value_before: f64,
        value_at_crossing: f64,
    ) -> Self {
        let rate = if (value_before - 0.0).abs() > f64::EPSILON {
            (value_at_crossing - value_before) / value_before.abs()
        } else {
            value_at_crossing
        };
        Self {
            metric: metric.into(),
            threshold,
            value_at_crossing,
            value_before,
            detected_at: Utc::now(),
            rate_of_change: rate,
        }
    }

    /// Returns the magnitude of the jump (absolute change).
    #[must_use]
    pub fn magnitude(&self) -> f64 {
        (self.value_at_crossing - self.value_before).abs()
    }

    /// Returns true if the threshold was crossed from below.
    #[must_use]
    pub fn crossed_from_below(&self) -> bool {
        self.value_before < self.threshold && self.value_at_crossing >= self.threshold
    }

    /// Returns true if the threshold was crossed from above.
    #[must_use]
    pub fn crossed_from_above(&self) -> bool {
        self.value_before > self.threshold && self.value_at_crossing <= self.threshold
    }
}

impl std::fmt::Display for Suddenness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Suddenness({}: {} -> {} crossing threshold {}, rate={:.2})",
            self.metric,
            self.value_before,
            self.value_at_crossing,
            self.threshold,
            self.rate_of_change,
        )
    }
}
