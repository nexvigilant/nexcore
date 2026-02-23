//! # NexVigilant Core — antibodies
//!
//! Adaptive immune layer for targeted threat recognition and neutralization.
//! Complements `nexcore-immunity` (innate detection) with learned epitope-paratope
//! binding — the adaptive arm of the nexcore biological defense system.
//!
//! ## Immunological Mapping
//!
//! | Immune Concept | nexcore Analog |
//! |----------------|----------------|
//! | Antigen | Threat signature (code antipattern, signal anomaly) |
//! | Epitope | Specific binding site on antigen (matchable feature) |
//! | Paratope | Antibody's recognition site (detection rule) |
//! | Antibody | Recognition + neutralization unit |
//! | Immunoglobulin | Response class (IgG=targeted, IgM=broad, IgE=escalation) |
//! | Memory B-cell | Persisted learned pattern for future recognition |
//! | Affinity maturation | Iterative rule refinement via feedback |
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Dominant Primitives |
//! |------|------|---------------------|
//! | `ImmunoglobulinClass` | T2-P | Σ (Sum) — enum alternation |
//! | `AffinityScore` | T2-P | N (Quantity) — binding strength |
//! | `Epitope` | T2-P | κ (Comparison) — matchable feature |
//! | `Antigen` | T2-C | ∃ (Existence) + ∂ (Boundary) |
//! | `Antibody` | T3 | κ + ∂ + ∃ + → + Σ |
//! | `AntibodyRepertoire` | T3 | π (Persistence) + μ (Mapping) |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod grounding;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Errors ────────────────────────────────────────────────────────────────

/// Tier: T2-C | Grounding: Σ (Sum) + ∂ (Boundary)
#[derive(Debug, nexcore_error::Error)]
pub enum AntibodyError {
    #[error("antigen not recognized: {0}")]
    UnrecognizedAntigen(String),

    #[error("affinity below binding threshold: {score} < {threshold}")]
    InsufficientAffinity { score: f64, threshold: f64 },

    #[error("repertoire lookup failed: {0}")]
    RepertoireLookup(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// ─── Immunoglobulin Classes ────────────────────────────────────────────────

/// Immunoglobulin response class — determines escalation behavior.
///
/// Tier: T2-P | Dominant: Σ (Sum) — five-variant alternation
///
/// Maps to Chomsky Type-3 grammar: finite set of response categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImmunoglobulinClass {
    /// IgG: Targeted, high-affinity response (most common).
    /// Used for known, well-characterized threats.
    IgG,

    /// IgM: Broad, first-responder. Lower affinity but wider coverage.
    /// Activated on novel threats before affinity maturation.
    IgM,

    /// IgA: Boundary-specific defense (mucosal immunity).
    /// Guards interface surfaces: API boundaries, config ingress.
    IgA,

    /// IgD: Surveillance mode. Low-level monitoring without active response.
    /// Signals to other systems without direct neutralization.
    IgD,

    /// IgE: Escalation/alarm class. Triggers amplified response.
    /// Reserved for P0-P1 patient safety threats.
    IgE,
}

impl ImmunoglobulinClass {
    /// Default binding threshold for this class.
    ///
    /// Higher class = lower threshold (more sensitive).
    #[must_use]
    pub fn default_threshold(&self) -> f64 {
        match self {
            Self::IgE => 0.3, // Most sensitive — escalation triggers easily
            Self::IgM => 0.4, // Broad first-response
            Self::IgA => 0.5, // Boundary defense
            Self::IgG => 0.6, // Targeted — needs high confidence
            Self::IgD => 0.7, // Surveillance — only strong signals
        }
    }

    /// Whether this class requires immediate escalation to Guardian.
    #[must_use]
    pub const fn requires_escalation(&self) -> bool {
        matches!(self, Self::IgE)
    }
}

impl fmt::Display for ImmunoglobulinClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IgG => write!(f, "IgG (Targeted)"),
            Self::IgM => write!(f, "IgM (Broad)"),
            Self::IgA => write!(f, "IgA (Boundary)"),
            Self::IgD => write!(f, "IgD (Surveillance)"),
            Self::IgE => write!(f, "IgE (Escalation)"),
        }
    }
}

// ─── Affinity Score ────────────────────────────────────────────────────────

/// Binding affinity between paratope and epitope. Range: [0.0, 1.0].
///
/// Tier: T2-P | Dominant: N (Quantity)
///
/// 0.0 = no recognition, 1.0 = perfect complementary binding.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AffinityScore(f64);

impl AffinityScore {
    /// Create a new affinity score, clamped to [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Raw score value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Whether this score exceeds the given threshold.
    ///
    /// Grounding: κ (Comparison) — threshold gate.
    #[must_use]
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }
}

impl fmt::Display for AffinityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.0)
    }
}

// ─── Epitope & Paratope ───────────────────────────────────────────────────

/// An epitope: the specific feature on an antigen that can be matched.
///
/// Tier: T2-P | Dominant: κ (Comparison)
///
/// In NexCore terms: a matchable signature fragment — could be a code pattern,
/// a signal shape, or a behavioral fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Epitope {
    /// Unique identifier for this epitope.
    pub id: String,
    /// Pattern signature (e.g., regex, AST fragment hash, signal shape).
    pub signature: String,
    /// Source domain the epitope originates from.
    pub domain: String,
}

/// A paratope: the antibody's recognition site that binds to an epitope.
///
/// Tier: T2-P | Dominant: κ (Comparison)
///
/// The "lock" to the epitope's "key". Encodes the matching logic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Paratope {
    /// Unique identifier.
    pub id: String,
    /// Matching rule (the complementary pattern to an epitope).
    pub matcher: String,
    /// Specificity: how many epitopes this paratope can bind.
    /// Lower = more specific. 1 = perfectly targeted.
    pub specificity: u32,
}

// ─── Antigen ──────────────────────────────────────────────────────────────

/// Threat severity level.
///
/// Tier: T2-P | Dominant: Σ (Sum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatSeverity {
    /// Informational — log only.
    Low,
    /// Warrants monitoring and possible response.
    Medium,
    /// Active threat requiring immediate neutralization.
    High,
    /// Patient-safety-critical (P0). Triggers IgE escalation.
    Critical,
}

/// An antigen: the threat entity to be recognized and neutralized.
///
/// Tier: T2-C | Dominant: ∃ (Existence) + ∂ (Boundary)
///
/// Represents a detected threat with its epitopes (matchable features).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Antigen {
    /// Unique antigen identifier.
    pub id: NexId,
    /// Human-readable name (e.g., "unwrap-in-production", "signal-suppression").
    pub name: String,
    /// Threat severity classification.
    pub severity: ThreatSeverity,
    /// Epitopes exposed on this antigen (binding sites).
    pub epitopes: Vec<Epitope>,
    /// When this antigen was first detected.
    pub detected_at: DateTime<Utc>,
    /// Source system that reported this antigen.
    pub source: String,
}

// ─── Antibody ─────────────────────────────────────────────────────────────

/// Neutralization action to take upon successful binding.
///
/// Tier: T2-C | Dominant: → (Causality) + ∂ (Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NeutralizationAction {
    /// Block the operation that triggered the antigen.
    Block { reason: String },
    /// Warn but allow (with audit trail).
    Warn { message: String },
    /// Quarantine for human review.
    Quarantine { reviewer: String },
    /// Escalate to Guardian homeostasis loop.
    Escalate { priority: u8 },
    /// Log and monitor (IgD passive surveillance).
    Monitor { interval_secs: u64 },
}

/// An antibody: the complete recognition + neutralization unit.
///
/// Tier: T3 | Grounding: κ + ∂ + ∃ + → + Σ (5 primitives)
///
/// Combines a paratope (recognition) with a response class and action.
/// This is the core operational unit of the adaptive immune system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Antibody {
    /// Unique identifier.
    pub id: NexId,
    /// Human-readable label.
    pub name: String,
    /// Recognition site — what this antibody binds to.
    pub paratope: Paratope,
    /// Immunoglobulin class — determines response behavior.
    pub class: ImmunoglobulinClass,
    /// Action to take upon successful binding.
    pub action: NeutralizationAction,
    /// Number of successful bindings (maturation metric).
    pub binding_count: u64,
    /// When this antibody was created.
    pub created_at: DateTime<Utc>,
    /// Last successful binding time.
    pub last_bound_at: Option<DateTime<Utc>>,
}

// ─── Binding Result ───────────────────────────────────────────────────────

/// Result of attempting to bind an antibody to an antigen.
///
/// Tier: T2-C | Dominant: κ (Comparison) + → (Causality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingResult {
    /// The antibody that attempted binding.
    pub antibody_id: NexId,
    /// The antigen targeted.
    pub antigen_id: NexId,
    /// Epitope that was matched (if any).
    pub matched_epitope: Option<String>,
    /// Computed affinity score.
    pub affinity: AffinityScore,
    /// Whether binding threshold was met.
    pub bound: bool,
    /// Recommended action (if bound).
    pub action: Option<NeutralizationAction>,
    /// Timestamp of binding attempt.
    pub attempted_at: DateTime<Utc>,
}

// ─── Antibody Repertoire ──────────────────────────────────────────────────

/// The antibody repertoire: persistent memory of all known antibodies.
///
/// Tier: T3 | Dominant: π (Persistence) + μ (Mapping)
///
/// Analogous to memory B-cells — once an antigen is encountered and an
/// antibody is generated, it persists here for rapid future recognition.
/// Thread-safe via `DashMap` for concurrent access.
pub struct AntibodyRepertoire {
    /// Antibodies indexed by ID.
    antibodies: DashMap<NexId, Antibody>,
    /// Paratope → Antibody ID index for fast epitope lookup.
    paratope_index: DashMap<String, Vec<NexId>>,
}

impl AntibodyRepertoire {
    /// Create an empty repertoire.
    #[must_use]
    pub fn new() -> Self {
        Self {
            antibodies: DashMap::new(),
            paratope_index: DashMap::new(),
        }
    }

    /// Number of antibodies in the repertoire.
    #[must_use]
    pub fn len(&self) -> usize {
        self.antibodies.len()
    }

    /// Whether the repertoire is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.antibodies.is_empty()
    }

    /// Register a new antibody in the repertoire.
    pub fn register(&self, antibody: Antibody) {
        let id = antibody.id;
        let matcher = antibody.paratope.matcher.clone();

        self.antibodies.insert(id, antibody);
        self.paratope_index.entry(matcher).or_default().push(id);
    }

    /// Look up an antibody by ID.
    #[must_use]
    pub fn get(&self, id: &NexId) -> Option<Antibody> {
        self.antibodies.get(id).map(|entry| entry.clone())
    }

    /// Attempt to bind all antibodies against an antigen.
    ///
    /// Returns binding results sorted by affinity (highest first).
    /// This is the core recognition loop — O(epitopes × paratopes).
    ///
    /// Grounding: κ (Comparison) drives the matching, → (Causality) produces actions.
    pub fn bind(&self, antigen: &Antigen) -> Vec<BindingResult> {
        let mut results = Vec::new();
        let now = Utc::now();

        for epitope in &antigen.epitopes {
            // Look up antibodies whose paratope matches this epitope's signature
            if let Some(antibody_ids) = self.paratope_index.get(&epitope.signature) {
                for &ab_id in antibody_ids.value() {
                    if let Some(ab) = self.antibodies.get(&ab_id) {
                        let affinity = compute_affinity(&ab.paratope, epitope);
                        let threshold = ab.class.default_threshold();
                        let bound = affinity.exceeds_threshold(threshold);

                        results.push(BindingResult {
                            antibody_id: ab.id,
                            antigen_id: antigen.id,
                            matched_epitope: Some(epitope.id.clone()),
                            affinity,
                            bound,
                            action: if bound { Some(ab.action.clone()) } else { None },
                            attempted_at: now,
                        });
                    }
                }
            }
        }

        // Sort by affinity descending — strongest binders first
        results.sort_by(|a, b| {
            b.affinity
                .value()
                .partial_cmp(&a.affinity.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}

impl Default for AntibodyRepertoire {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Affinity Computation ─────────────────────────────────────────────────

/// Compute binding affinity between a paratope and an epitope.
///
/// Grounding: κ (Comparison) — string similarity as proxy for structural complementarity.
///
/// Uses normalized edit distance as the affinity metric:
/// affinity = 1.0 - (edit_distance / max_len)
///
/// In production, this would integrate with `nexcore-edit-distance` for
/// O(n*m) Levenshtein with early termination.
#[must_use]
pub fn compute_affinity(paratope: &Paratope, epitope: &Epitope) -> AffinityScore {
    let matcher = &paratope.matcher;
    let signature = &epitope.signature;

    if matcher == signature {
        return AffinityScore::new(1.0);
    }

    if matcher.is_empty() || signature.is_empty() {
        return AffinityScore::new(0.0);
    }

    // Normalized containment check as affinity proxy
    let (shorter, longer) = if matcher.len() <= signature.len() {
        (matcher.as_str(), signature.as_str())
    } else {
        (signature.as_str(), matcher.as_str())
    };

    if longer.contains(shorter) {
        let ratio = shorter.len() as f64 / longer.len() as f64;
        AffinityScore::new(ratio)
    } else {
        // Character overlap ratio
        let matcher_chars: std::collections::HashSet<char> = matcher.chars().collect();
        let sig_chars: std::collections::HashSet<char> = signature.chars().collect();
        let intersection = matcher_chars.intersection(&sig_chars).count();
        let union = matcher_chars.union(&sig_chars).count();

        if union == 0 {
            AffinityScore::new(0.0)
        } else {
            AffinityScore::new(intersection as f64 / union as f64)
        }
    }
}

/// Classify which immunoglobulin class should respond to a given threat.
///
/// Grounding: κ (Comparison) + Σ (Sum) — severity comparison selects class variant.
#[must_use]
pub fn classify_response(severity: ThreatSeverity, is_novel: bool) -> ImmunoglobulinClass {
    match (severity, is_novel) {
        (ThreatSeverity::Critical, _) => ImmunoglobulinClass::IgE,
        (ThreatSeverity::High, true) => ImmunoglobulinClass::IgM,
        (ThreatSeverity::High, false) => ImmunoglobulinClass::IgG,
        (ThreatSeverity::Medium, true) => ImmunoglobulinClass::IgM,
        (ThreatSeverity::Medium, false) => ImmunoglobulinClass::IgA,
        (ThreatSeverity::Low, _) => ImmunoglobulinClass::IgD,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_epitope(id: &str, sig: &str) -> Epitope {
        Epitope {
            id: id.to_string(),
            signature: sig.to_string(),
            domain: "test".to_string(),
        }
    }

    fn make_paratope(id: &str, matcher: &str) -> Paratope {
        Paratope {
            id: id.to_string(),
            matcher: matcher.to_string(),
            specificity: 1,
        }
    }

    fn make_antigen(name: &str, severity: ThreatSeverity, epitopes: Vec<Epitope>) -> Antigen {
        Antigen {
            id: NexId::v4(),
            name: name.to_string(),
            severity,
            epitopes,
            detected_at: Utc::now(),
            source: "test".to_string(),
        }
    }

    fn make_antibody(name: &str, matcher: &str, class: ImmunoglobulinClass) -> Antibody {
        Antibody {
            id: NexId::v4(),
            name: name.to_string(),
            paratope: make_paratope(&format!("p-{name}"), matcher),
            class,
            action: NeutralizationAction::Block {
                reason: format!("neutralize {name}"),
            },
            binding_count: 0,
            created_at: Utc::now(),
            last_bound_at: None,
        }
    }

    #[test]
    fn affinity_exact_match_is_one() {
        let p = make_paratope("p1", "unwrap_used");
        let e = make_epitope("e1", "unwrap_used");
        let score = compute_affinity(&p, &e);
        assert!((score.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn affinity_no_match_is_low() {
        let p = make_paratope("p1", "aaaa");
        let e = make_epitope("e1", "zzzz");
        let score = compute_affinity(&p, &e);
        assert!(score.value() < 0.5);
    }

    #[test]
    fn affinity_empty_is_zero() {
        let p = make_paratope("p1", "");
        let e = make_epitope("e1", "something");
        let score = compute_affinity(&p, &e);
        assert!((score.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn affinity_score_clamped() {
        let s = AffinityScore::new(1.5);
        assert!((s.value() - 1.0).abs() < f64::EPSILON);

        let s = AffinityScore::new(-0.5);
        assert!(s.value().abs() < f64::EPSILON);
    }

    #[test]
    fn ig_class_thresholds_descend_with_severity() {
        assert!(
            ImmunoglobulinClass::IgE.default_threshold()
                < ImmunoglobulinClass::IgD.default_threshold()
        );
        assert!(
            ImmunoglobulinClass::IgM.default_threshold()
                < ImmunoglobulinClass::IgG.default_threshold()
        );
    }

    #[test]
    fn only_ige_requires_escalation() {
        assert!(ImmunoglobulinClass::IgE.requires_escalation());
        assert!(!ImmunoglobulinClass::IgG.requires_escalation());
        assert!(!ImmunoglobulinClass::IgM.requires_escalation());
        assert!(!ImmunoglobulinClass::IgA.requires_escalation());
        assert!(!ImmunoglobulinClass::IgD.requires_escalation());
    }

    #[test]
    fn classify_critical_always_ige() {
        assert_eq!(
            classify_response(ThreatSeverity::Critical, false),
            ImmunoglobulinClass::IgE
        );
        assert_eq!(
            classify_response(ThreatSeverity::Critical, true),
            ImmunoglobulinClass::IgE
        );
    }

    #[test]
    fn classify_novel_high_uses_igm() {
        assert_eq!(
            classify_response(ThreatSeverity::High, true),
            ImmunoglobulinClass::IgM
        );
    }

    #[test]
    fn classify_known_high_uses_igg() {
        assert_eq!(
            classify_response(ThreatSeverity::High, false),
            ImmunoglobulinClass::IgG
        );
    }

    #[test]
    fn classify_low_always_igd() {
        assert_eq!(
            classify_response(ThreatSeverity::Low, true),
            ImmunoglobulinClass::IgD
        );
        assert_eq!(
            classify_response(ThreatSeverity::Low, false),
            ImmunoglobulinClass::IgD
        );
    }

    #[test]
    fn repertoire_register_and_lookup() {
        let repo = AntibodyRepertoire::new();
        assert!(repo.is_empty());

        let ab = make_antibody("unwrap-hunter", "unwrap_used", ImmunoglobulinClass::IgG);
        let id = ab.id;
        repo.register(ab);

        assert_eq!(repo.len(), 1);
        assert!(repo.get(&id).is_some());
    }

    #[test]
    fn repertoire_bind_exact_match() {
        let repo = AntibodyRepertoire::new();
        let ab = make_antibody("unwrap-hunter", "unwrap_used", ImmunoglobulinClass::IgG);
        repo.register(ab);

        let antigen = make_antigen(
            "bad-unwrap",
            ThreatSeverity::High,
            vec![make_epitope("e1", "unwrap_used")],
        );

        let results = repo.bind(&antigen);
        assert_eq!(results.len(), 1);
        assert!(results[0].bound);
        assert!((results[0].affinity.value() - 1.0).abs() < f64::EPSILON);
        assert!(results[0].action.is_some());
    }

    #[test]
    fn repertoire_bind_no_match() {
        let repo = AntibodyRepertoire::new();
        let ab = make_antibody("unwrap-hunter", "unwrap_used", ImmunoglobulinClass::IgG);
        repo.register(ab);

        let antigen = make_antigen(
            "something-else",
            ThreatSeverity::Low,
            vec![make_epitope("e1", "totally_different")],
        );

        let results = repo.bind(&antigen);
        // No antibody has a paratope matching "totally_different"
        assert!(results.is_empty());
    }

    #[test]
    fn repertoire_bind_multiple_epitopes() {
        let repo = AntibodyRepertoire::new();

        let ab1 = make_antibody("unwrap-hunter", "unwrap_used", ImmunoglobulinClass::IgG);
        let ab2 = make_antibody("panic-hunter", "panic_call", ImmunoglobulinClass::IgE);
        repo.register(ab1);
        repo.register(ab2);

        let antigen = make_antigen(
            "multi-threat",
            ThreatSeverity::Critical,
            vec![
                make_epitope("e1", "unwrap_used"),
                make_epitope("e2", "panic_call"),
            ],
        );

        let results = repo.bind(&antigen);
        assert_eq!(results.len(), 2);
        // Both should bind (exact matches)
        assert!(results.iter().all(|r| r.bound));
    }

    #[test]
    fn default_repertoire_is_empty() {
        let repo = AntibodyRepertoire::default();
        assert!(repo.is_empty());
        assert_eq!(repo.len(), 0);
    }

    #[test]
    fn neutralization_action_variants() {
        // Ensure all variants are constructible
        let _block = NeutralizationAction::Block {
            reason: "test".into(),
        };
        let _warn = NeutralizationAction::Warn {
            message: "test".into(),
        };
        let _quarantine = NeutralizationAction::Quarantine {
            reviewer: "test".into(),
        };
        let _escalate = NeutralizationAction::Escalate { priority: 0 };
        let _monitor = NeutralizationAction::Monitor { interval_secs: 60 };
    }

    #[test]
    fn ig_class_display() {
        assert_eq!(format!("{}", ImmunoglobulinClass::IgG), "IgG (Targeted)");
        assert_eq!(format!("{}", ImmunoglobulinClass::IgE), "IgE (Escalation)");
    }

    #[test]
    fn affinity_score_display() {
        let s = AffinityScore::new(0.85);
        assert_eq!(format!("{s}"), "0.850");
    }

    #[test]
    fn antigen_serialization_roundtrip() {
        let antigen = make_antigen(
            "test-antigen",
            ThreatSeverity::Medium,
            vec![make_epitope("e1", "sig1")],
        );
        let json = serde_json::to_string(&antigen);
        assert!(json.is_ok());
        let parsed: Result<Antigen, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(parsed.is_ok());
    }
}
