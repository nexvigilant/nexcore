//! Referred pain diagnostics — mapping surface hook triggers to architectural root causes.
//!
//! ## Biology Analog
//!
//! Heart attack → left arm pain. Visceral afferents converge on the same spinal
//! segments as somatic afferents. The symptom appears at a different location
//! than the actual damage.
//!
//! ## Purpose
//!
//! When a hook fires, the surface-level detection (regex match, AST pattern)
//! may not be the root cause. This module maps surface symptoms to potential
//! architectural causes, generating diagnostic clues for the developer.
//!
//! ## Primitive Grounding: →(Causality) + λ(Location) + μ(Mapping)
//!
//! - `→` (Causality): A surface symptom *caused by* an architectural decision elsewhere.
//! - `λ` (Location): The symptom appears at one file location; the root cause lives at another.
//! - `μ` (Mapping): `DermatomeMap` and `PainMap` encode the referral topology.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_synapse::referred_pain::{
//!     ArchitecturalLayer, CauseCategory, DermatomeMap, PainMap,
//!     ReferralEngine, SurfaceSymptom,
//! };
//!
//! let engine = ReferralEngine::new(
//!     DermatomeMap::default_map(),
//!     PainMap::default_pain_map(),
//! );
//!
//! let symptom = SurfaceSymptom {
//!     hook_name: "unwrap-guardian".to_string(),
//!     pattern_matched: ".unwrap()".to_string(),
//!     file_path: "crates/nexcore-mcp/src/tools/mod.rs".to_string(),
//!     line_number: Some(42),
//!     severity: 0.8,
//! };
//!
//! let clue = engine.diagnose(&symptom);
//! assert!(!clue.causes.is_empty());
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// SURFACE SYMPTOM
// ============================================================================

/// What a hook detected at the surface level.
///
/// Tier: T2-P (grounded to λ: Location + ∃: Existence)
///
/// Analogous to the presenting symptom a clinician observes — left arm pain,
/// not the underlying cardiac ischemia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceSymptom {
    /// The hook binary that fired (e.g., `"unwrap-guardian"`).
    pub hook_name: String,

    /// The pattern the hook matched (e.g., `".unwrap()"`).
    pub pattern_matched: String,

    /// File path where the match was found.
    pub file_path: String,

    /// Line number of the match, if available.
    pub line_number: Option<u32>,

    /// Severity score in [0.0, 1.0].
    pub severity: f64,
}

impl SurfaceSymptom {
    /// Create a new surface symptom.
    #[must_use]
    pub fn new(
        hook_name: impl Into<String>,
        pattern_matched: impl Into<String>,
        file_path: impl Into<String>,
        line_number: Option<u32>,
        severity: f64,
    ) -> Self {
        Self {
            hook_name: hook_name.into(),
            pattern_matched: pattern_matched.into(),
            file_path: file_path.into(),
            line_number,
            severity: severity.clamp(0.0, 1.0),
        }
    }
}

// ============================================================================
// ARCHITECTURAL LAYER
// ============================================================================

/// NexCore's 4-layer architectural hierarchy.
///
/// Tier: T2-P (grounded to λ: Location + σ: Sequence)
///
/// Dependency flows DOWN only: Service → Orchestration → Domain → Foundation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitecturalLayer {
    /// Service layer — external interfaces, binary targets.
    ///
    /// Examples: `nexcore-mcp`, `nexcore-api`, `nexcore-cli`.
    Service,

    /// Orchestration layer — workflow coordination.
    ///
    /// Examples: `nexcore-vigil`, `nexcore-brain`, `nexcore-build-gate`.
    Orchestration,

    /// Domain layer — business logic, uses foundation types.
    ///
    /// Examples: `nexcore-vigilance`, `nexcore-guardian-engine`, `nexcore-skills-engine`.
    Domain,

    /// Foundation layer — core primitives, zero domain knowledge.
    ///
    /// Examples: `nexcore-primitives`, `stem-*`, `nexcore-lex-primitiva`.
    Foundation,
}

impl std::fmt::Display for ArchitecturalLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Service => "Service",
            Self::Orchestration => "Orchestration",
            Self::Domain => "Domain",
            Self::Foundation => "Foundation",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// CAUSE CATEGORY
// ============================================================================

/// Category of architectural root cause.
///
/// Tier: T2-P (grounded to →: Causality + ς: State)
///
/// Explains *why* a symptom is being forced at a distant location, analogous to
/// classifying whether pain is visceral, referred somatic, or neuropathic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseCategory {
    /// The upstream API makes the bad pattern inevitable.
    ///
    /// Example: A function returns `Option<T>` where `Result<T, E>` is needed,
    /// so callers are forced to `.unwrap()` or introduce lossy conversions.
    DesignForced,

    /// A transitive dependency forces the pattern.
    ///
    /// Example: An external crate only exposes `Option`-returning constructors;
    /// all downstream callers inherit the `.unwrap()` pressure.
    DependencyChain,

    /// No clean abstraction exists for the correct pattern.
    ///
    /// Example: Repeated `clone()` calls because there is no shared reference
    /// wrapper (e.g., `Arc<T>`) defined at the right abstraction boundary.
    MissingAbstraction,

    /// Old code pattern propagating through the codebase.
    ///
    /// Example: Pre-`?` operator idioms using `match` on `Result` that were
    /// never refactored after error propagation improved.
    LegacyDebt,

    /// Environment or configuration mismatch.
    ///
    /// Example: A path constant that differs between dev and prod causes
    /// panics that look like logic bugs but are actually config divergence.
    ConfigurationDrift,
}

impl std::fmt::Display for CauseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DesignForced => "DesignForced",
            Self::DependencyChain => "DependencyChain",
            Self::MissingAbstraction => "MissingAbstraction",
            Self::LegacyDebt => "LegacyDebt",
            Self::ConfigurationDrift => "ConfigurationDrift",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// ARCHITECTURAL CAUSE
// ============================================================================

/// A hypothesized root cause of a surface symptom.
///
/// Tier: T2-C (grounded to →: Causality + λ: Location + N: Quantity)
///
/// Each cause carries a confidence score so that multiple competing hypotheses
/// can be ranked — analogous to differential diagnosis in medicine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalCause {
    /// Classification of the cause type.
    pub category: CauseCategory,

    /// The architectural layer where the root cause lives.
    pub layer: ArchitecturalLayer,

    /// Human-readable explanation of what is causing the symptom.
    pub description: String,

    /// Confidence in this hypothesis, clamped to [0.0, 1.0].
    pub confidence: f64,

    /// The file most likely to contain the root cause, if known.
    pub likely_file: Option<String>,
}

impl ArchitecturalCause {
    /// Construct a new cause hypothesis.
    #[must_use]
    pub fn new(
        category: CauseCategory,
        layer: ArchitecturalLayer,
        description: impl Into<String>,
        confidence: f64,
        likely_file: Option<String>,
    ) -> Self {
        Self {
            category,
            layer,
            description: description.into(),
            confidence: confidence.clamp(0.0, 1.0),
            likely_file,
        }
    }
}

// ============================================================================
// DIAGNOSTIC CLUE
// ============================================================================

/// A confidence-weighted pointer from a surface symptom to its architectural causes.
///
/// Tier: T3 (diagnostic composition grounded to →, λ, μ)
///
/// The `dermatome` field names the "nerve segment" that connects the symptom
/// site to the organ: in neurology this is e.g. C8-T1; here it is a subsystem
/// label like `"mcp-service-layer"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticClue {
    /// The observed surface symptom.
    pub symptom: SurfaceSymptom,

    /// Hypothesized root causes, ranked by confidence (descending).
    pub causes: Vec<ArchitecturalCause>,

    /// The "nerve segment" label connecting the symptom location to the cause.
    pub dermatome: String,
}

impl DiagnosticClue {
    /// Return the highest-confidence cause, if any.
    #[must_use]
    pub fn primary_cause(&self) -> Option<&ArchitecturalCause> {
        self.causes.first()
    }

    /// Aggregate confidence — mean across all causes.
    #[must_use]
    pub fn aggregate_confidence(&self) -> f64 {
        if self.causes.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.causes.iter().map(|c| c.confidence).sum();
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        // cause count is small; precision loss negligible
        let count = self.causes.len() as f64;
        sum / count
    }
}

// ============================================================================
// DERMATOME MAP
// ============================================================================

/// Maps surface file regions to architectural layers and subsystem names.
///
/// Tier: T2-C (grounded to λ: Location + μ: Mapping)
///
/// In neurology a dermatome is the patch of skin innervated by a single spinal
/// nerve root. Here each entry maps a file-path prefix to the architectural
/// layer responsible for that region, enabling the referral engine to ask:
/// "which organ does this skin patch connect to?"
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DermatomeMap {
    entries: Vec<DermatomeEntry>,
}

/// A single entry in the dermatome map.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DermatomeEntry {
    /// File-path substring that triggers this mapping.
    file_pattern: String,
    /// The architectural layer responsible for files matching this pattern.
    architectural_layer: ArchitecturalLayer,
    /// Human-readable subsystem label.
    subsystem: String,
}

impl DermatomeMap {
    /// Create an empty dermatome map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new pattern → layer mapping.
    ///
    /// `file_pattern` is matched as a substring of the file path.
    /// More specific patterns should be registered first (first-match wins).
    pub fn register(
        &mut self,
        file_pattern: impl Into<String>,
        layer: ArchitecturalLayer,
        subsystem: impl Into<String>,
    ) {
        self.entries.push(DermatomeEntry {
            file_pattern: file_pattern.into(),
            architectural_layer: layer,
            subsystem: subsystem.into(),
        });
    }

    /// Look up the architectural layer and subsystem for a given file path.
    ///
    /// Returns the first match (registration order determines priority).
    #[must_use]
    pub fn lookup(&self, file_path: &str) -> Option<(ArchitecturalLayer, String)> {
        self.entries.iter().find_map(|entry| {
            if file_path.contains(entry.file_pattern.as_str()) {
                Some((entry.architectural_layer, entry.subsystem.clone()))
            } else {
                None
            }
        })
    }

    /// Construct a pre-populated map for the standard NexCore directory layout.
    ///
    /// Covers the four canonical layers and key subsystems.
    #[must_use]
    #[allow(
        clippy::too_many_lines,
        reason = "Long mapping tables and explicit diagnostic branches improve auditability"
    )]
    pub fn default_map() -> Self {
        let mut map = Self::new();

        // ── Service layer ────────────────────────────────────────────────────
        map.register(
            "crates/nexcore-mcp",
            ArchitecturalLayer::Service,
            "mcp-service",
        );
        map.register(
            "crates/nexcore-api",
            ArchitecturalLayer::Service,
            "rest-api",
        );
        map.register(
            "crates/nexcore-cli",
            ArchitecturalLayer::Service,
            "cli-service",
        );
        map.register(
            "crates/nexcore-guardian-cli",
            ArchitecturalLayer::Service,
            "guardian-cli",
        );

        // ── Orchestration layer ──────────────────────────────────────────────
        map.register(
            "crates/nexcore-vigil",
            ArchitecturalLayer::Orchestration,
            "vigil-orchestrator",
        );
        map.register(
            "crates/nexcore-brain",
            ArchitecturalLayer::Orchestration,
            "brain-memory",
        );
        map.register(
            "crates/nexcore-build-gate",
            ArchitecturalLayer::Orchestration,
            "build-gate",
        );

        // ── Domain layer ─────────────────────────────────────────────────────
        map.register(
            "crates/nexcore-vigilance",
            ArchitecturalLayer::Domain,
            "vigilance-domain",
        );
        map.register(
            "crates/nexcore-guardian-engine",
            ArchitecturalLayer::Domain,
            "guardian-domain",
        );
        map.register(
            "crates/nexcore-skills-engine",
            ArchitecturalLayer::Domain,
            "skills-domain",
        );
        map.register(
            "crates/nexcore-tov",
            ArchitecturalLayer::Domain,
            "tov-domain",
        );
        map.register(
            "crates/nexcore-faers-etl",
            ArchitecturalLayer::Domain,
            "faers-etl",
        );
        map.register(
            "crates/nexcore-pvos",
            ArchitecturalLayer::Domain,
            "pvos-domain",
        );
        map.register(
            "crates/nexcore-synapse",
            ArchitecturalLayer::Domain,
            "synapse-learning",
        );
        map.register(
            "crates/nexcore-energy",
            ArchitecturalLayer::Domain,
            "energy-budget",
        );
        map.register(
            "crates/nexcore-cytokine",
            ArchitecturalLayer::Domain,
            "cytokine-signaling",
        );
        map.register(
            "crates/nexcore-immunity",
            ArchitecturalLayer::Domain,
            "immunity-defense",
        );

        // ── Foundation layer ─────────────────────────────────────────────────
        map.register(
            "crates/nexcore-primitives",
            ArchitecturalLayer::Foundation,
            "primitives",
        );
        map.register(
            "crates/nexcore-lex-primitiva",
            ArchitecturalLayer::Foundation,
            "lex-primitiva",
        );
        map.register(
            "crates/stem-",
            ArchitecturalLayer::Foundation,
            "stem-science",
        );
        map.register(
            "crates/nexcore-id",
            ArchitecturalLayer::Foundation,
            "id-types",
        );
        map.register(
            "crates/nexcore-config",
            ArchitecturalLayer::Foundation,
            "config",
        );
        map.register(
            "crates/nucli",
            ArchitecturalLayer::Foundation,
            "nucli-codec",
        );
        map.register(
            "crates/prima-",
            ArchitecturalLayer::Foundation,
            "prima-lang",
        );

        map
    }
}

// ============================================================================
// PAIN MAP
// ============================================================================

/// Registry of known hook-pattern → architectural cause mappings.
///
/// Tier: T2-C (grounded to →: Causality + μ: Mapping)
///
/// Each entry encodes a known referral path: "when hook pattern P fires,
/// consider architectural cause C." Multiple causes per pattern are supported;
/// they are ranked by confidence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PainMap {
    entries: Vec<PainEntry>,
}

/// Internal entry in the pain map.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PainEntry {
    /// Hook name or pattern fragment to match (substring).
    hook_pattern: String,
    /// The cause associated with this pattern.
    cause: ArchitecturalCause,
}

impl PainMap {
    /// Create an empty pain map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a hook pattern → cause mapping.
    ///
    /// Multiple causes may be registered for the same hook pattern.
    /// All matching entries are returned by `lookup`, sorted by confidence.
    pub fn register(&mut self, hook_pattern: impl Into<String>, cause: ArchitecturalCause) {
        self.entries.push(PainEntry {
            hook_pattern: hook_pattern.into(),
            cause,
        });
    }

    /// Look up all causes for a given surface symptom.
    ///
    /// Matches on `hook_name` and `pattern_matched` (both as substrings).
    /// Returns causes sorted by confidence, descending.
    #[must_use]
    pub fn lookup(&self, symptom: &SurfaceSymptom) -> Vec<ArchitecturalCause> {
        let mut causes: Vec<ArchitecturalCause> = self
            .entries
            .iter()
            .filter(|entry| {
                symptom.hook_name.contains(entry.hook_pattern.as_str())
                    || symptom
                        .pattern_matched
                        .contains(entry.hook_pattern.as_str())
            })
            .map(|entry| entry.cause.clone())
            .collect();

        // Sort by descending confidence
        causes.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        causes
    }

    /// Construct a pre-populated pain map with known NexCore referral patterns.
    ///
    /// Covers:
    /// - `.unwrap()` in service layer → dependency-chain from domain returning `Option`
    /// - `clone()` spam → missing shared-reference abstraction
    /// - Repeated error-type conversions → design-forced inconsistent error types
    /// - `panic!` / `todo!` stubs → legacy debt
    /// - Config path mismatches → configuration drift
    #[must_use]
    #[allow(
        clippy::too_many_lines,
        reason = "Long mapping tables and explicit diagnostic branches improve auditability"
    )]
    pub fn default_pain_map() -> Self {
        let mut map = Self::new();

        // ── .unwrap() ────────────────────────────────────────────────────────
        map.register(
            ".unwrap()",
            ArchitecturalCause::new(
                CauseCategory::DependencyChain,
                ArchitecturalLayer::Domain,
                "Domain API returns Option<T> instead of Result<T,E>, \
                 forcing callers to unwrap or silently discard error context.",
                0.80,
                None,
            ),
        );
        map.register(
            ".unwrap()",
            ArchitecturalCause::new(
                CauseCategory::DesignForced,
                ArchitecturalLayer::Domain,
                "Constructor or factory function has no error channel; \
                 callers are architecturally forced to unwrap.",
                0.65,
                None,
            ),
        );
        map.register(
            "unwrap-guardian",
            ArchitecturalCause::new(
                CauseCategory::LegacyDebt,
                ArchitecturalLayer::Service,
                "Pre-? operator code was written before anyhow/thiserror adoption; \
                 `.unwrap()` is a relic of the early codebase style.",
                0.50,
                None,
            ),
        );

        // ── .clone() spam ────────────────────────────────────────────────────
        map.register(
            ".clone()",
            ArchitecturalCause::new(
                CauseCategory::MissingAbstraction,
                ArchitecturalLayer::Domain,
                "No shared-reference wrapper (Arc<T> or Cow<T>) defined at the \
                 correct abstraction boundary; consumers resort to cloning.",
                0.75,
                None,
            ),
        );
        map.register(
            ".clone()",
            ArchitecturalCause::new(
                CauseCategory::DesignForced,
                ArchitecturalLayer::Foundation,
                "Foundation type does not implement Copy where it could; \
                 downstream layers inherit unnecessary clone pressure.",
                0.55,
                None,
            ),
        );

        // ── Error type conversions ────────────────────────────────────────────
        map.register(
            "From<",
            ArchitecturalCause::new(
                CauseCategory::DesignForced,
                ArchitecturalLayer::Domain,
                "Inconsistent error types across crates require repeated From/Into \
                 conversions; unifying on anyhow or a shared error enum would eliminate these.",
                0.70,
                None,
            ),
        );
        map.register(
            ".map_err(",
            ArchitecturalCause::new(
                CauseCategory::DesignForced,
                ArchitecturalLayer::Domain,
                "Proliferating map_err chains indicate error-type fragmentation; \
                 a shared error interface would collapse these into single ? propagation.",
                0.65,
                None,
            ),
        );

        // ── panic! / todo! stubs ──────────────────────────────────────────────
        map.register(
            "panic!",
            ArchitecturalCause::new(
                CauseCategory::LegacyDebt,
                ArchitecturalLayer::Service,
                "Stub implementation not yet replaced; panic! was a placeholder \
                 that was never promoted to proper error handling.",
                0.85,
                None,
            ),
        );
        map.register(
            "todo!()",
            ArchitecturalCause::new(
                CauseCategory::LegacyDebt,
                ArchitecturalLayer::Domain,
                "todo!() marks an incomplete implementation; the architectural \
                 intent exists but the body was never written.",
                0.90,
                None,
            ),
        );

        // ── Configuration drift ───────────────────────────────────────────────
        map.register(
            "path",
            ArchitecturalCause::new(
                CauseCategory::ConfigurationDrift,
                ArchitecturalLayer::Orchestration,
                "Hard-coded path constant diverges between environments; \
                 centralise in nexcore-config or pass through AmplitudeConfig.",
                0.45,
                Some("crates/nexcore-config/src/lib.rs".to_string()),
            ),
        );

        map
    }
}

// ============================================================================
// REFERRAL ENGINE
// ============================================================================

/// The main engine that diagnoses surface symptoms into architectural causes.
///
/// Tier: T3 (orchestrates DermatomeMap + PainMap + DiagnosticClue)
///
/// Analogous to the specialist who reads the dermatomal chart and pain map to
/// deduce whether presenting left-arm pain is cardiac, cervical, or thoracic
/// in origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralEngine {
    dermatome_map: DermatomeMap,
    pain_map: PainMap,
}

impl ReferralEngine {
    /// Create a new referral engine with the given maps.
    #[must_use]
    pub fn new(dermatome_map: DermatomeMap, pain_map: PainMap) -> Self {
        Self {
            dermatome_map,
            pain_map,
        }
    }

    /// Create an engine pre-populated with default NexCore mappings.
    #[must_use]
    pub fn default_engine() -> Self {
        Self::new(DermatomeMap::default_map(), PainMap::default_pain_map())
    }

    /// Diagnose a single surface symptom into a `DiagnosticClue`.
    ///
    /// 1. Resolves the dermatome (λ: which layer owns this file?)
    /// 2. Retrieves ranked causes from the pain map (→: what could cause this?)
    /// 3. Packages them into a `DiagnosticClue` with the dermatome label
    #[must_use]
    pub fn diagnose(&self, symptom: &SurfaceSymptom) -> DiagnosticClue {
        // Resolve dermatome from file path
        let dermatome = self.dermatome_map.lookup(&symptom.file_path).map_or_else(
            || "unknown/unregistered".to_string(),
            |(layer, subsystem)| format!("{layer}/{subsystem}"),
        );

        // Retrieve ranked causes from the pain map
        let causes = self.pain_map.lookup(symptom);

        DiagnosticClue {
            symptom: symptom.clone(),
            causes,
            dermatome,
        }
    }

    /// Diagnose a batch of symptoms.
    ///
    /// Each symptom is diagnosed independently; results preserve input order.
    #[must_use]
    pub fn batch_diagnose(&self, symptoms: &[SurfaceSymptom]) -> Vec<DiagnosticClue> {
        symptoms.iter().map(|s| self.diagnose(s)).collect()
    }

    /// Correlate multiple diagnostic clues to find common root causes.
    ///
    /// Groups causes by (category, layer) and sums confidence-weighted
    /// occurrences. Returns the deduplicated list sorted by aggregate weight.
    ///
    /// This is the "cluster of symptoms → single organ" step: if three
    /// different files all show `.unwrap()`, the correlation elevates the
    /// DependencyChain/Domain hypothesis.
    #[must_use]
    pub fn correlate(&self, clues: &[DiagnosticClue]) -> Vec<ArchitecturalCause> {
        use std::collections::HashMap;

        // Key: (category, layer, description snippet)
        // Value: (accumulated_weight, representative_cause)
        let mut accumulator: HashMap<
            (CauseCategory, ArchitecturalLayer),
            (f64, ArchitecturalCause),
        > = HashMap::new();

        for clue in clues {
            for cause in &clue.causes {
                let key = (cause.category, cause.layer);
                let entry = accumulator
                    .entry(key)
                    .or_insert_with(|| (0.0, cause.clone()));
                entry.0 += cause.confidence;
            }
        }

        // Build output: set confidence = normalised aggregate weight
        let total_weight: f64 = accumulator.values().map(|(w, _)| w).sum();
        let mut result: Vec<ArchitecturalCause> = accumulator
            .into_values()
            .map(|(weight, mut cause)| {
                cause.confidence = if total_weight > 0.0 {
                    (weight / total_weight).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                cause
            })
            .collect();

        // Sort by descending normalised confidence
        result.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Test-only fixture setup with fixed constants"
)]
mod tests {
    use super::*;

    fn make_unwrap_symptom(file_path: &str) -> SurfaceSymptom {
        SurfaceSymptom::new("unwrap-guardian", ".unwrap()", file_path, Some(42), 0.8)
    }

    // ── SurfaceSymptom ───────────────────────────────────────────────────────

    #[test]
    fn surface_symptom_severity_clamped() {
        let s = SurfaceSymptom::new("hook", "pat", "file.rs", None, 2.5);
        assert!((s.severity - 1.0).abs() < f64::EPSILON);

        let s2 = SurfaceSymptom::new("hook", "pat", "file.rs", None, -0.3);
        assert!(s2.severity.abs() < f64::EPSILON);
    }

    // ── DermatomeMap ─────────────────────────────────────────────────────────

    #[test]
    fn dermatome_map_lookup_service() {
        let map = DermatomeMap::default_map();
        let result = map.lookup("crates/nexcore-mcp/src/tools/mod.rs");
        assert!(result.is_some());
        if let Some((layer, subsystem)) = result {
            assert_eq!(layer, ArchitecturalLayer::Service);
            assert_eq!(subsystem, "mcp-service");
        }
    }

    #[test]
    fn dermatome_map_lookup_foundation() {
        let map = DermatomeMap::default_map();
        let result = map.lookup("crates/nexcore-primitives/src/lib.rs");
        assert!(result.is_some());
        if let Some((layer, _)) = result {
            assert_eq!(layer, ArchitecturalLayer::Foundation);
        }
    }

    #[test]
    fn dermatome_map_lookup_unknown() {
        let map = DermatomeMap::default_map();
        assert!(map.lookup("external/unknown_lib/src/lib.rs").is_none());
    }

    #[test]
    fn dermatome_map_register_custom() {
        let mut map = DermatomeMap::new();
        map.register("myapp/", ArchitecturalLayer::Service, "my-service");
        let result = map.lookup("myapp/src/main.rs");
        assert!(result.is_some());
        if let Some((layer, sub)) = result {
            assert_eq!(layer, ArchitecturalLayer::Service);
            assert_eq!(sub, "my-service");
        }
    }

    // ── PainMap ──────────────────────────────────────────────────────────────

    #[test]
    fn pain_map_lookup_unwrap_returns_causes() {
        let pm = PainMap::default_pain_map();
        let symptom = make_unwrap_symptom("crates/nexcore-mcp/src/tools/mod.rs");
        let causes = pm.lookup(&symptom);
        assert!(!causes.is_empty(), "default map should have unwrap causes");
    }

    #[test]
    fn pain_map_causes_sorted_by_confidence() {
        let pm = PainMap::default_pain_map();
        let symptom = make_unwrap_symptom("any/file.rs");
        let causes = pm.lookup(&symptom);
        for window in causes.windows(2) {
            assert!(
                window[0].confidence >= window[1].confidence,
                "causes must be descending by confidence"
            );
        }
    }

    #[test]
    fn pain_map_lookup_no_match() {
        let pm = PainMap::default_pain_map();
        let symptom = SurfaceSymptom::new(
            "completely-unknown-hook",
            "~completely_unique_token~",
            "file.rs",
            None,
            0.5,
        );
        let causes = pm.lookup(&symptom);
        assert!(causes.is_empty());
    }

    #[test]
    fn pain_map_register_custom() {
        let mut pm = PainMap::new();
        pm.register(
            "my-hook",
            ArchitecturalCause::new(
                CauseCategory::LegacyDebt,
                ArchitecturalLayer::Domain,
                "Test cause",
                0.9,
                None,
            ),
        );
        let symptom = SurfaceSymptom::new("my-hook", "some-pattern", "file.rs", None, 1.0);
        let causes = pm.lookup(&symptom);
        assert_eq!(causes.len(), 1);
        assert!((causes[0].confidence - 0.9).abs() < f64::EPSILON);
    }

    // ── ArchitecturalCause ───────────────────────────────────────────────────

    #[test]
    fn architectural_cause_confidence_clamped() {
        let cause = ArchitecturalCause::new(
            CauseCategory::LegacyDebt,
            ArchitecturalLayer::Foundation,
            "desc",
            1.5,
            None,
        );
        assert!((cause.confidence - 1.0).abs() < f64::EPSILON);
    }

    // ── DiagnosticClue ───────────────────────────────────────────────────────

    #[test]
    fn diagnostic_clue_primary_cause_empty() {
        let clue = DiagnosticClue {
            symptom: make_unwrap_symptom("file.rs"),
            causes: vec![],
            dermatome: "unknown".to_string(),
        };
        assert!(clue.primary_cause().is_none());
        assert!((clue.aggregate_confidence()).abs() < f64::EPSILON);
    }

    #[test]
    fn diagnostic_clue_aggregate_confidence() {
        let causes = vec![
            ArchitecturalCause::new(
                CauseCategory::LegacyDebt,
                ArchitecturalLayer::Service,
                "a",
                0.8,
                None,
            ),
            ArchitecturalCause::new(
                CauseCategory::DesignForced,
                ArchitecturalLayer::Domain,
                "b",
                0.4,
                None,
            ),
        ];
        let clue = DiagnosticClue {
            symptom: make_unwrap_symptom("file.rs"),
            causes,
            dermatome: "test".to_string(),
        };
        let expected = 0.6_f64;
        assert!((clue.aggregate_confidence() - expected).abs() < 1e-10);
    }

    // ── ReferralEngine ───────────────────────────────────────────────────────

    #[test]
    fn referral_engine_diagnose_mcp_unwrap() {
        let engine = ReferralEngine::default_engine();
        let symptom = make_unwrap_symptom("crates/nexcore-mcp/src/tools/relay.rs");
        let clue = engine.diagnose(&symptom);

        assert!(
            !clue.causes.is_empty(),
            "should produce causes for unwrap in mcp"
        );
        assert!(
            clue.dermatome.contains("Service"),
            "mcp files belong to Service layer, got: {}",
            clue.dermatome
        );
    }

    #[test]
    fn referral_engine_diagnose_unknown_file() {
        let engine = ReferralEngine::default_engine();
        let symptom = SurfaceSymptom::new(
            "completely-unknown-hook",
            "completely-unique-pattern",
            "external/unknown/src/lib.rs",
            None,
            0.5,
        );
        let clue = engine.diagnose(&symptom);
        assert_eq!(clue.dermatome, "unknown/unregistered");
        assert!(clue.causes.is_empty());
    }

    #[test]
    fn referral_engine_batch_diagnose_preserves_order() {
        let engine = ReferralEngine::default_engine();
        let symptoms = vec![
            make_unwrap_symptom("crates/nexcore-mcp/src/lib.rs"),
            make_unwrap_symptom("crates/nexcore-primitives/src/lib.rs"),
            make_unwrap_symptom("crates/nexcore-brain/src/lib.rs"),
        ];
        let clues = engine.batch_diagnose(&symptoms);
        assert_eq!(clues.len(), 3);
        assert!(clues[0].dermatome.contains("Service"));
        assert!(clues[1].dermatome.contains("Foundation"));
        assert!(clues[2].dermatome.contains("Orchestration"));
    }

    #[test]
    fn referral_engine_correlate_elevates_common_cause() {
        let engine = ReferralEngine::default_engine();

        // Three .unwrap() symptoms in three different files
        let symptoms = vec![
            make_unwrap_symptom("crates/nexcore-mcp/src/tools/mod.rs"),
            make_unwrap_symptom("crates/nexcore-api/src/routes.rs"),
            make_unwrap_symptom("crates/nexcore-cli/src/main.rs"),
        ];
        let clues = engine.batch_diagnose(&symptoms);
        let correlated = engine.correlate(&clues);

        assert!(!correlated.is_empty(), "correlation should produce causes");

        // All causes have confidence in [0,1]
        for cause in &correlated {
            assert!(
                (0.0..=1.0).contains(&cause.confidence),
                "normalised confidence out of range: {}",
                cause.confidence
            );
        }

        // Sorted descending
        for window in correlated.windows(2) {
            assert!(
                window[0].confidence >= window[1].confidence,
                "correlation output must be sorted descending"
            );
        }
    }

    #[test]
    fn referral_engine_correlate_empty_input() {
        let engine = ReferralEngine::default_engine();
        let result = engine.correlate(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn cause_category_display() {
        assert_eq!(CauseCategory::DesignForced.to_string(), "DesignForced");
        assert_eq!(
            CauseCategory::DependencyChain.to_string(),
            "DependencyChain"
        );
        assert_eq!(
            CauseCategory::MissingAbstraction.to_string(),
            "MissingAbstraction"
        );
        assert_eq!(CauseCategory::LegacyDebt.to_string(), "LegacyDebt");
        assert_eq!(
            CauseCategory::ConfigurationDrift.to_string(),
            "ConfigurationDrift"
        );
    }

    #[test]
    fn architectural_layer_display() {
        assert_eq!(ArchitecturalLayer::Service.to_string(), "Service");
        assert_eq!(
            ArchitecturalLayer::Orchestration.to_string(),
            "Orchestration"
        );
        assert_eq!(ArchitecturalLayer::Domain.to_string(), "Domain");
        assert_eq!(ArchitecturalLayer::Foundation.to_string(), "Foundation");
    }

    // ── Serde round-trip ─────────────────────────────────────────────────────

    #[test]
    fn surface_symptom_serde_round_trip() {
        let s = SurfaceSymptom::new("hook", ".ok()", "crates/foo/src/lib.rs", Some(10), 0.7);
        let json = serde_json::to_string(&s).unwrap_or_default();
        assert!(!json.is_empty());
        let back: SurfaceSymptom = serde_json::from_str(&json).unwrap_or_else(|_| s.clone());
        assert_eq!(back.hook_name, "hook");
        assert_eq!(back.line_number, Some(10));
        assert!((back.severity - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn architectural_cause_serde_round_trip() {
        let c = ArchitecturalCause::new(
            CauseCategory::LegacyDebt,
            ArchitecturalLayer::Domain,
            "desc",
            0.75,
            Some("crates/nexcore-mcp/src/lib.rs".to_string()),
        );
        let json = serde_json::to_string(&c).unwrap_or_default();
        assert!(!json.is_empty());
        let back: ArchitecturalCause = serde_json::from_str(&json).unwrap_or_else(|_| c.clone());
        assert_eq!(back.category, CauseCategory::LegacyDebt);
        assert_eq!(back.layer, ArchitecturalLayer::Domain);
        assert!((back.confidence - 0.75).abs() < f64::EPSILON);
    }
}
