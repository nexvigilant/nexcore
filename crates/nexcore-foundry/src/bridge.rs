//! Bridge contract types for The Foundry assembly line architecture.
//!
//! A bridge is the typed boundary between two stations. Every bridge has a
//! [`BridgeContract`] that declares the source station, destination station,
//! and the [`nexcore_lex_primitiva::LexPrimitiva`] primitives that ground its
//! semantics. Bridges are load-tested via [`StructuralReport`], and the
//! feedback path back to the builder pipeline produces [`DesignConstraints`].
//!
//! # Pipeline Topology
//!
//! ```text
//! Builder pipeline          Analyst pipeline
//! ─────────────────         ────────────────
//!  Blueprint (B1)            Measure   (A1)
//!  Frame     (B2)  ←──────── Pattern   (A2)
//!  Finish    (B3)            Reason    (A3)
//!      │                         │
//!      └──── Extraction ─────────┘
//!                                │
//!                     Crystallization
//!                                │
//!                         Inference
//!                                │
//!                    Feedback ───┘
//!                        │
//!                        ↓
//!                    Blueprint (B1)  ← next iteration
//! ```
//!
//! # Example
//!
//! ```rust
//! use nexcore_foundry::bridge::{
//!     BridgeContract, BridgeKind, BuilderStation, AnalystStation, StationType,
//! };
//! use nexcore_lex_primitiva::LexPrimitiva;
//!
//! let contract = BridgeContract {
//!     kind: BridgeKind::Codification,
//!     from: StationType::Builder(BuilderStation::Blueprint),
//!     to:   StationType::Builder(BuilderStation::Frame),
//!     primitives: vec![
//!         LexPrimitiva::Mapping,
//!         LexPrimitiva::Causality,
//!         LexPrimitiva::Sequence,
//!     ],
//! };
//!
//! assert_eq!(contract.primitives.len(), 3);
//! ```

use nexcore_lex_primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────────────
// BridgeKind
// ──────────────────────────────────────────────────────────────────────────────

/// The six bridges that connect stations across The Foundry's dual pipeline.
///
/// Each variant names the direction and semantic role of the handoff:
///
/// | Variant | From → To | Semantic role |
/// |---------|-----------|---------------|
/// | [`Codification`](BridgeKind::Codification) | B1 → B2 | Design → Code |
/// | [`Verification`](BridgeKind::Verification) | B2 → B3 | Code → Validated |
/// | [`Extraction`](BridgeKind::Extraction) | B\* → A1 | Builder → Metrics |
/// | [`Crystallization`](BridgeKind::Crystallization) | A1 → A2 | Metrics → Patterns |
/// | [`Inference`](BridgeKind::Inference) | A2 → A3 | Patterns → Causal |
/// | [`Feedback`](BridgeKind::Feedback) | A3 → B1 | Insights → Constraints |
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeKind {
    /// B1 → B2: design artifacts are codified into executable code.
    Codification,
    /// B2 → B3: code is verified and promoted to a finished artifact.
    Verification,
    /// B\* → A1: builder outputs are extracted into measurable signals.
    Extraction,
    /// A1 → A2: raw metrics crystallise into recurring patterns.
    Crystallization,
    /// A2 → A3: patterns are elevated into causal inferences.
    Inference,
    /// A3 → B1: analyst insights are fed back as design constraints.
    Feedback,
}

impl BridgeKind {
    /// Returns all six bridge kinds in pipeline order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_foundry::bridge::BridgeKind;
    ///
    /// assert_eq!(BridgeKind::all().len(), 6);
    /// ```
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Codification,
            Self::Verification,
            Self::Extraction,
            Self::Crystallization,
            Self::Inference,
            Self::Feedback,
        ]
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Station enums
// ──────────────────────────────────────────────────────────────────────────────

/// A station in the builder pipeline.
///
/// The builder pipeline transforms a design blueprint into a finished artifact
/// through three ordered stations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuilderStation {
    /// B1 — Blueprint: the design phase where intent is captured as constraints.
    Blueprint,
    /// B2 — Frame: the construction phase where code is generated.
    Frame,
    /// B3 — Finish: the validation phase producing a releasable artifact.
    Finish,
}

/// A station in the analyst pipeline.
///
/// The analyst pipeline observes builder outputs and derives actionable insights
/// through three ordered stations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalystStation {
    /// A1 — Measure: raw signal collection from builder artifacts.
    Measure,
    /// A2 — Pattern: recurring structures extracted from measurements.
    Pattern,
    /// A3 — Reason: causal inference drawn from patterns.
    Reason,
}

/// A unified station type that covers both pipelines.
///
/// Used in [`BridgeContract`] fields so a single type describes either end of
/// any bridge regardless of which pipeline it belongs to.
///
/// # Example
///
/// ```rust
/// use nexcore_foundry::bridge::{BuilderStation, AnalystStation, StationType};
///
/// let from = StationType::Builder(BuilderStation::Blueprint);
/// let to   = StationType::Analyst(AnalystStation::Measure);
///
/// assert_ne!(from, to);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StationType {
    /// A station in the builder pipeline.
    Builder(BuilderStation),
    /// A station in the analyst pipeline.
    Analyst(AnalystStation),
}

// ──────────────────────────────────────────────────────────────────────────────
// BridgeContract
// ──────────────────────────────────────────────────────────────────────────────

/// The typed contract for a single bridge between two stations.
///
/// A [`BridgeContract`] declares which bridge kind it represents, the source
/// and destination stations, and the Lex Primitiva that ground its semantics.
/// Contracts are the inputs to load-testing via [`StructuralReport`].
///
/// # Example
///
/// ```rust
/// use nexcore_foundry::bridge::{
///     BridgeContract, BridgeKind, BuilderStation, StationType,
/// };
/// use nexcore_lex_primitiva::LexPrimitiva;
///
/// let contract = BridgeContract {
///     kind:       BridgeKind::Codification,
///     from:       StationType::Builder(BuilderStation::Blueprint),
///     to:         StationType::Builder(BuilderStation::Frame),
///     primitives: vec![LexPrimitiva::Mapping, LexPrimitiva::Causality],
/// };
///
/// assert_eq!(contract.primitives.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeContract {
    /// Which bridge this contract governs.
    pub kind: BridgeKind,
    /// The station that produces the artifact being handed off.
    pub from: StationType,
    /// The station that consumes the artifact.
    pub to: StationType,
    /// The Lex Primitiva that ground this bridge's semantics.
    pub primitives: Vec<LexPrimitiva>,
}

// ──────────────────────────────────────────────────────────────────────────────
// StructuralCheck / StructuralReport
// ──────────────────────────────────────────────────────────────────────────────

/// A single named check within a bridge load test.
///
/// # Example
///
/// ```rust
/// use nexcore_foundry::bridge::StructuralCheck;
///
/// let check = StructuralCheck { name: "schema_valid".to_owned(), passed: true };
/// assert!(check.passed);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralCheck {
    /// Human-readable name identifying what was checked.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
}

/// The aggregated result of a structural load test for a specific bridge.
///
/// # Example
///
/// ```rust
/// use nexcore_foundry::bridge::{BridgeKind, StructuralCheck, StructuralReport};
///
/// let report = StructuralReport {
///     bridge: BridgeKind::Codification,
///     checks: vec![
///         StructuralCheck { name: "schema_valid".to_owned(), passed: true },
///         StructuralCheck { name: "no_cycles".to_owned(),    passed: true },
///     ],
/// };
///
/// assert!(report.all_pass());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralReport {
    /// The bridge that was load-tested.
    pub bridge: BridgeKind,
    /// Individual check results.
    pub checks: Vec<StructuralCheck>,
}

impl StructuralReport {
    /// Returns `true` if every check in this report passed.
    ///
    /// An empty check list is considered a pass (vacuous truth).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_foundry::bridge::{BridgeKind, StructuralCheck, StructuralReport};
    ///
    /// let passing = StructuralReport {
    ///     bridge: BridgeKind::Verification,
    ///     checks: vec![StructuralCheck { name: "types_match".to_owned(), passed: true }],
    /// };
    /// assert!(passing.all_pass());
    ///
    /// let failing = StructuralReport {
    ///     bridge: BridgeKind::Verification,
    ///     checks: vec![StructuralCheck { name: "types_match".to_owned(), passed: false }],
    /// };
    /// assert!(!failing.all_pass());
    /// ```
    #[must_use]
    pub fn all_pass(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// DesignConstraints
// ──────────────────────────────────────────────────────────────────────────────

/// The output of the [`BridgeKind::Feedback`] bridge.
///
/// Analyst insights are translated into concrete constraints that are injected
/// back into the Blueprint station (B1) for the next iteration of the builder
/// pipeline.
///
/// # Example
///
/// ```rust
/// use nexcore_foundry::bridge::DesignConstraints;
///
/// let constraints = DesignConstraints {
///     new_components:  vec!["cache-layer".to_owned()],
///     new_constraints: vec!["p99 < 50ms".to_owned()],
///     iteration_trigger: true,
/// };
///
/// assert!(constraints.iteration_trigger);
/// assert_eq!(constraints.new_components.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignConstraints {
    /// New components the analyst pipeline determined should be introduced.
    pub new_components: Vec<String>,
    /// New constraints the analyst pipeline determined must be satisfied.
    pub new_constraints: Vec<String>,
    /// When `true`, the feedback bridge triggers a new builder iteration
    /// immediately rather than waiting for a scheduled cycle.
    pub iteration_trigger: bool,
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_contract_identifies_boundary() {
        let contract = BridgeContract {
            kind: BridgeKind::Codification,
            from: StationType::Builder(BuilderStation::Blueprint),
            to: StationType::Builder(BuilderStation::Frame),
            primitives: vec![
                LexPrimitiva::Mapping,
                LexPrimitiva::Causality,
                LexPrimitiva::Sequence,
            ],
        };

        assert_eq!(contract.primitives.len(), 3);
    }

    #[test]
    fn structural_report_pass() {
        let report = StructuralReport {
            bridge: BridgeKind::Verification,
            checks: vec![
                StructuralCheck {
                    name: "schema_valid".to_owned(),
                    passed: true,
                },
                StructuralCheck {
                    name: "no_cycles".to_owned(),
                    passed: true,
                },
                StructuralCheck {
                    name: "types_align".to_owned(),
                    passed: true,
                },
            ],
        };

        assert!(report.all_pass());
    }

    #[test]
    fn structural_report_fail() {
        let report = StructuralReport {
            bridge: BridgeKind::Extraction,
            checks: vec![StructuralCheck {
                name: "signal_present".to_owned(),
                passed: false,
            }],
        };

        assert!(!report.all_pass());
    }

    #[test]
    fn design_constraints_from_feedback() {
        let constraints = DesignConstraints {
            new_components: vec!["cache-layer".to_owned()],
            new_constraints: vec!["p99 < 50ms".to_owned()],
            iteration_trigger: true,
        };

        assert_eq!(constraints.new_components, vec!["cache-layer"]);
        assert_eq!(constraints.new_constraints, vec!["p99 < 50ms"]);
        assert!(constraints.iteration_trigger);
    }

    #[test]
    fn all_bridge_kinds_represented() {
        assert_eq!(BridgeKind::all().len(), 6);
    }
}
