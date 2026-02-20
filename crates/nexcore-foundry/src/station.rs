//! Station types and pipeline ordering for The Foundry assembly line architecture.
//!
//! Defines the identifiers, run modes, per-station configuration, and ordered
//! pipeline sequences that govern how agents advance through the dual-pipeline
//! (builder + analyst) assembly line.
//!
//! # Pipelines
//!
//! - **Builder pipeline** — B1 → B2 → B3 (design, frame, finish)
//! - **Analyst pipeline** — A1 → A2 → A3 (measure, pattern, reason)
//! - **VDAG full** — both pipelines interleaved with seven bridge stations
//!
//! # Example
//!
//! ```
//! use nexcore_foundry::station::{PipelineOrder, StationId};
//!
//! let seq = PipelineOrder::builder_sequence();
//! assert_eq!(seq.len(), 3);
//! assert_eq!(seq.stages[0], StationId::B1);
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// StationId
// ---------------------------------------------------------------------------

/// Identifies one of the 14 positions in The Foundry assembly line.
///
/// Stations are grouped into three categories:
///
/// - **Builder stations** (`B1`–`B3`) — design, frame, and finish a deliverable.
/// - **Analyst stations** (`A1`–`A3`) — measure, pattern-match, and reason about it.
/// - **Bridge stations** (`Bridge*`) — handoff and coordination points between
///   the two pipelines in a full VDAG run.
///
/// # Example
///
/// ```
/// use nexcore_foundry::station::StationId;
///
/// let id = StationId::B1;
/// let serialised = serde_json::to_string(&id).ok();
/// assert_eq!(serialised.as_deref(), Some("\"B1\""));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationId {
    // -- Builder pipeline --------------------------------------------------

    /// B1 — Blueprint station. Produces a [`DesignSpec`] from a prompt.
    ///
    /// [`DesignSpec`]: crate::artifact::DesignSpec
    B1,

    /// B2 — Frame station. Implements the components described in a
    /// [`DesignSpec`] and produces a [`SourceArtifact`].
    ///
    /// [`DesignSpec`]: crate::artifact::DesignSpec
    /// [`SourceArtifact`]: crate::artifact::SourceArtifact
    B2,

    /// B3 — Finish station. Runs the quality gate and produces a
    /// [`ValidatedDeliverable`].
    ///
    /// [`ValidatedDeliverable`]: crate::artifact::ValidatedDeliverable
    B3,

    // -- Analyst pipeline --------------------------------------------------

    /// A1 — Measure station. Collects quantitative signals from the
    /// deliverable.
    A1,

    /// A2 — Pattern station. Identifies structural and behavioural patterns
    /// in the measurements.
    A2,

    /// A3 — Reason station. Synthesises patterns into actionable conclusions.
    A3,

    // -- Bridge stations ---------------------------------------------------

    /// Codification bridge — encodes builder output into a canonical form
    /// before B2 processing.
    BridgeCodify,

    /// Verification bridge — cross-checks encoded output before B3 processing.
    BridgeVerify,

    /// First extraction bridge — pulls primary signals from the finished
    /// deliverable.
    BridgeExtract1,

    /// Second extraction bridge — pulls secondary signals from the finished
    /// deliverable.
    BridgeExtract2,

    /// Third extraction bridge — pulls tertiary signals from the finished
    /// deliverable.
    BridgeExtract3,

    /// Crystal bridge — crystallises the extracted signals before A1
    /// processing.
    BridgeCrystal,

    /// Inference bridge — carries inferred patterns from A2 into A3.
    BridgeInfer,

    /// Feedback bridge — closes the loop by routing A3 conclusions back to
    /// the pipeline entry point.
    BridgeFeedback,
}

// ---------------------------------------------------------------------------
// RunMode
// ---------------------------------------------------------------------------

/// Execution mode for a station's agent.
///
/// `Foreground` stations block the pipeline until they complete; `Background`
/// stations are dispatched asynchronously and may proceed in parallel with
/// subsequent work.
///
/// # Example
///
/// ```
/// use nexcore_foundry::station::{RunMode, StationConfig};
///
/// let cfg = StationConfig::blueprint();
/// assert_eq!(cfg.run_mode, RunMode::Foreground);
///
/// let cfg = StationConfig::frame();
/// assert_eq!(cfg.run_mode, RunMode::Background);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunMode {
    /// The pipeline waits for this station to finish before advancing.
    Foreground,
    /// The station is dispatched without blocking the pipeline.
    Background,
}

// ---------------------------------------------------------------------------
// StationConfig
// ---------------------------------------------------------------------------

/// Configuration for a single station in the assembly line.
///
/// Each station names the agent that will execute it, the skills that agent
/// must load, and whether it runs in the foreground or background.
///
/// Prefer the constructor methods (`blueprint`, `frame`, etc.) over direct
/// struct construction to ensure canonical defaults are used.
///
/// # Example
///
/// ```
/// use nexcore_foundry::station::{StationConfig, StationId, RunMode};
///
/// let cfg = StationConfig::blueprint();
/// assert_eq!(cfg.id, StationId::B1);
/// assert_eq!(cfg.agent_name, "foundry-blueprint");
/// assert!(cfg.skills.contains(&"brainstorming".to_string()));
/// assert_eq!(cfg.run_mode, RunMode::Foreground);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationConfig {
    /// The station's position in the assembly line.
    pub id: StationId,
    /// Name of the agent that executes this station.
    pub agent_name: String,
    /// Ordered list of skill identifiers the agent must load.
    pub skills: Vec<String>,
    /// Whether this station blocks or is dispatched asynchronously.
    pub run_mode: RunMode,
}

impl StationConfig {
    // -- Builder pipeline constructors -------------------------------------

    /// Returns the canonical configuration for the **B1 Blueprint** station.
    ///
    /// The blueprint station designs the deliverable. It runs in the
    /// foreground because subsequent stations depend on its output before
    /// they can begin.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{StationConfig, StationId, RunMode};
    ///
    /// let cfg = StationConfig::blueprint();
    /// assert_eq!(cfg.id, StationId::B1);
    /// assert_eq!(cfg.run_mode, RunMode::Foreground);
    /// ```
    #[must_use]
    pub fn blueprint() -> Self {
        Self {
            id: StationId::B1,
            agent_name: "foundry-blueprint".to_string(),
            skills: vec![
                "brainstorming".to_string(),
                "strat-dev".to_string(),
                "code-architect".to_string(),
            ],
            run_mode: RunMode::Foreground,
        }
    }

    /// Returns the canonical configuration for the **B2 Frame** station.
    ///
    /// The frame station implements all components described in the design
    /// spec. It runs in the background so that the pipeline can track
    /// progress asynchronously.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{StationConfig, StationId, RunMode};
    ///
    /// let cfg = StationConfig::frame();
    /// assert_eq!(cfg.id, StationId::B2);
    /// assert_eq!(cfg.run_mode, RunMode::Background);
    /// ```
    #[must_use]
    pub fn frame() -> Self {
        Self {
            id: StationId::B2,
            agent_name: "foundry-frame".to_string(),
            skills: vec![
                "rust-dev".to_string(),
                "nextjs-dev".to_string(),
                "tailwind-dev".to_string(),
            ],
            run_mode: RunMode::Background,
        }
    }

    /// Returns the canonical configuration for the **B3 Finish** station.
    ///
    /// The finish station runs the quality gate (`cargo build`, `cargo test`,
    /// `cargo clippy`). It runs in the foreground because no further work
    /// should proceed until the gate result is known.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{StationConfig, StationId, RunMode};
    ///
    /// let cfg = StationConfig::finish();
    /// assert_eq!(cfg.id, StationId::B3);
    /// assert_eq!(cfg.run_mode, RunMode::Foreground);
    /// ```
    #[must_use]
    pub fn finish() -> Self {
        Self {
            id: StationId::B3,
            agent_name: "foundry-finish".to_string(),
            skills: vec!["guard-program".to_string()],
            run_mode: RunMode::Foreground,
        }
    }

    // -- Analyst pipeline constructors -------------------------------------

    /// Returns the canonical configuration for the **A1 Measure** station.
    ///
    /// The measure station collects quantitative signals from the deliverable.
    /// It runs in the background because measurements are I/O-bound and can
    /// proceed while the pipeline advances.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{StationConfig, StationId, RunMode};
    ///
    /// let cfg = StationConfig::measure();
    /// assert_eq!(cfg.id, StationId::A1);
    /// assert_eq!(cfg.run_mode, RunMode::Background);
    /// ```
    #[must_use]
    pub fn measure() -> Self {
        Self {
            id: StationId::A1,
            agent_name: "foundry-measure".to_string(),
            skills: vec![
                "data-transformer".to_string(),
                "craft-program".to_string(),
            ],
            run_mode: RunMode::Background,
        }
    }

    /// Returns the canonical configuration for the **A2 Pattern** station.
    ///
    /// The pattern station identifies structural and behavioural patterns in
    /// the measurements produced by A1. It runs in the background.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{StationConfig, StationId, RunMode};
    ///
    /// let cfg = StationConfig::pattern();
    /// assert_eq!(cfg.id, StationId::A2);
    /// assert_eq!(cfg.run_mode, RunMode::Background);
    /// ```
    #[must_use]
    pub fn pattern() -> Self {
        Self {
            id: StationId::A2,
            agent_name: "foundry-pattern".to_string(),
            skills: vec!["scope-program".to_string()],
            run_mode: RunMode::Background,
        }
    }

    /// Returns the canonical configuration for the **A3 Reason** station.
    ///
    /// The reason station synthesises the patterns found by A2 into
    /// actionable conclusions. It runs in the background.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{StationConfig, StationId, RunMode};
    ///
    /// let cfg = StationConfig::reason();
    /// assert_eq!(cfg.id, StationId::A3);
    /// assert_eq!(cfg.run_mode, RunMode::Background);
    /// ```
    #[must_use]
    pub fn reason() -> Self {
        Self {
            id: StationId::A3,
            agent_name: "foundry-reason".to_string(),
            skills: vec!["strat-dev".to_string()],
            run_mode: RunMode::Background,
        }
    }
}

// ---------------------------------------------------------------------------
// PipelineOrder
// ---------------------------------------------------------------------------

/// An ordered sequence of [`StationId`] values describing a pipeline run.
///
/// Three canonical sequences are provided:
///
/// - [`builder_sequence`] — the three-station builder pipeline.
/// - [`analyst_sequence`] — the three-station analyst pipeline.
/// - [`vdag_full`] — the complete 14-station VDAG interleaving both pipelines
///   with all seven bridge stations.
///
/// Custom sequences can be constructed by building a `Vec<StationId>` and
/// wrapping it: `PipelineOrder { stages: my_stages }`.
///
/// [`builder_sequence`]: PipelineOrder::builder_sequence
/// [`analyst_sequence`]: PipelineOrder::analyst_sequence
/// [`vdag_full`]: PipelineOrder::vdag_full
///
/// # Example
///
/// ```
/// use nexcore_foundry::station::{PipelineOrder, StationId};
///
/// let order = PipelineOrder::vdag_full();
/// assert_eq!(order.len(), 14);
/// assert_eq!(order.stages.first(), Some(&StationId::B1));
/// assert_eq!(order.stages.last(), Some(&StationId::BridgeFeedback));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineOrder {
    /// The ordered list of station identifiers for this pipeline run.
    pub stages: Vec<StationId>,
}

impl PipelineOrder {
    /// Returns the three-station **builder** pipeline: B1 → B2 → B3.
    ///
    /// Use this sequence when only the builder half of The Foundry is needed
    /// (e.g. code generation without analyst feedback).
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{PipelineOrder, StationId};
    ///
    /// let seq = PipelineOrder::builder_sequence();
    /// assert_eq!(seq.stages, vec![StationId::B1, StationId::B2, StationId::B3]);
    /// ```
    #[must_use]
    pub fn builder_sequence() -> Self {
        Self {
            stages: vec![StationId::B1, StationId::B2, StationId::B3],
        }
    }

    /// Returns the three-station **analyst** pipeline: A1 → A2 → A3.
    ///
    /// Use this sequence when only the analyst half of The Foundry is needed
    /// (e.g. post-hoc quality measurement without rebuilding).
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{PipelineOrder, StationId};
    ///
    /// let seq = PipelineOrder::analyst_sequence();
    /// assert_eq!(seq.stages, vec![StationId::A1, StationId::A2, StationId::A3]);
    /// ```
    #[must_use]
    pub fn analyst_sequence() -> Self {
        Self {
            stages: vec![StationId::A1, StationId::A2, StationId::A3],
        }
    }

    /// Returns the complete 14-station **VDAG** pipeline.
    ///
    /// The VDAG full sequence interleaves both pipelines with all seven bridge
    /// stations in the following order:
    ///
    /// B1 → BridgeCodify → B2 → BridgeVerify → B3 →
    /// BridgeExtract1 → BridgeExtract2 → BridgeExtract3 →
    /// A1 → BridgeCrystal → A2 → BridgeInfer → A3 → BridgeFeedback
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::{PipelineOrder, StationId};
    ///
    /// let order = PipelineOrder::vdag_full();
    /// assert_eq!(order.len(), 14);
    /// assert_eq!(order.stages[0], StationId::B1);
    /// assert_eq!(order.stages[13], StationId::BridgeFeedback);
    /// ```
    #[must_use]
    pub fn vdag_full() -> Self {
        Self {
            stages: vec![
                StationId::B1,
                StationId::BridgeCodify,
                StationId::B2,
                StationId::BridgeVerify,
                StationId::B3,
                StationId::BridgeExtract1,
                StationId::BridgeExtract2,
                StationId::BridgeExtract3,
                StationId::A1,
                StationId::BridgeCrystal,
                StationId::A2,
                StationId::BridgeInfer,
                StationId::A3,
                StationId::BridgeFeedback,
            ],
        }
    }

    /// Returns the number of stages in this pipeline.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::PipelineOrder;
    ///
    /// assert_eq!(PipelineOrder::builder_sequence().len(), 3);
    /// assert_eq!(PipelineOrder::vdag_full().len(), 14);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    /// Returns `true` when this pipeline contains no stages.
    ///
    /// A pipeline returned by any of the canonical constructors is never
    /// empty. This method is provided for custom pipelines constructed by
    /// the caller.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::station::PipelineOrder;
    ///
    /// assert!(!PipelineOrder::builder_sequence().is_empty());
    /// assert!(PipelineOrder { stages: vec![] }.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The builder sequence must contain exactly three stages in B1 → B2 → B3
    /// order.
    #[test]
    fn builder_pipeline_order() {
        let seq = PipelineOrder::builder_sequence();

        assert_eq!(seq.len(), 3);
        assert_eq!(seq.stages[0], StationId::B1);
        assert_eq!(seq.stages[1], StationId::B2);
        assert_eq!(seq.stages[2], StationId::B3);
    }

    /// The analyst sequence must contain exactly three stages in A1 → A2 → A3
    /// order.
    #[test]
    fn analyst_pipeline_order() {
        let seq = PipelineOrder::analyst_sequence();

        assert_eq!(seq.len(), 3);
        assert_eq!(seq.stages[0], StationId::A1);
        assert_eq!(seq.stages[1], StationId::A2);
        assert_eq!(seq.stages[2], StationId::A3);
    }

    /// The VDAG full sequence must contain all 14 stations, begin with B1,
    /// and end with BridgeFeedback.
    #[test]
    fn vdag_full_order_includes_bridges() {
        let order = PipelineOrder::vdag_full();

        assert_eq!(order.len(), 14);
        assert_eq!(order.stages.first(), Some(&StationId::B1));
        assert_eq!(order.stages.last(), Some(&StationId::BridgeFeedback));
    }

    /// The blueprint station config must list "brainstorming" and "strat-dev"
    /// among its skills.
    #[test]
    fn station_config_has_skills() {
        let cfg = StationConfig::blueprint();

        assert!(cfg.skills.contains(&"brainstorming".to_string()));
        assert!(cfg.skills.contains(&"strat-dev".to_string()));
    }

    /// Blueprint runs in the foreground; frame runs in the background.
    #[test]
    fn station_config_run_modes() {
        assert_eq!(StationConfig::blueprint().run_mode, RunMode::Foreground);
        assert_eq!(StationConfig::frame().run_mode, RunMode::Background);
    }
}
