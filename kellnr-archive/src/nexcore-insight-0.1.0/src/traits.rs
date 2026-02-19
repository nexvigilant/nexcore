//! # The Insight Trait — System-Level Behavioral Contract
//!
//! Any system that performs the 6 insight operations implements this trait:
//!
//! | Operation | Composite | Formula | Primitive Role |
//! |-----------|-----------|---------|----------------|
//! | `ingest` | (all 6) | full pipeline | Orchestration |
//! | detect | `Pattern` | σ + κ + μ | Co-occurrence detection |
//! | recognize | `Recognition` | κ + ∃ + σ | Match against prior knowledge |
//! | detect | `Novelty` | ∅ + ∃ + σ | Identify unprecedented |
//! | discover | `Connection` | μ + κ + ς | Link previously unrelated |
//! | compress | `Compression` | N + μ + κ | Many observations → few principles |
//! | detect | `Suddenness` | σ + ∂ + N + κ | Threshold crossing |
//!
//! ## Grammar Level
//!
//! Insight requires all 5 generators {σ, Σ, ρ, κ, ∃} → Chomsky Type-0.
//! - σ: Temporal ordering of observations
//! - Σ: Multiple event types (InsightEvent enum = coproduct)
//! - ρ: Patterns feed back into recognition (recursive refinement)
//! - κ: Threshold comparisons, identity matching
//! - ∃: Novelty detection (existence/absence in prior state)
//!
//! ## nexcore as InsightEngine
//!
//! nexcore's architecture IS an insight engine over pharmacovigilance data:
//! - `nexcore-pv-core` (Signal Detection) → Pattern
//! - `nexcore-guardian-engine` (Threat Sensing) → Recognition
//! - `nexcore-faers-etl` (Signal Velocity) → Novelty + Suddenness
//! - `nexcore-brain` (Implicit Learning) → Connection + Compression
//!
//! The concrete `InsightEngine` struct implements this trait domain-generically.
//! System-level implementations compose existing crates through this interface.

use crate::composites::{Compression, Connection, Pattern};
use crate::engine::InsightEvent;

/// The Insight trait — behavioral contract for any insight-producing system.
///
/// ## Tier: T3 (system-level trait)
///
/// INSIGHT ≡ ⟨σ, κ, μ, ∃, ς, ∅, N, ∂⟩
///
/// Implementors include:
/// - `InsightEngine` (domain-generic, in this crate)
/// - NexCore system-level (composes pv-core + guardian + brain + faers-etl)
///
/// # Associated Type
///
/// `Obs` — the observation type. Domain-generic engines use `Observation`.
/// PV systems might use `Icsr`, `AdverseEvent`, or `SignalCandidate`.
pub trait Insight {
    /// The observation type accepted by this engine.
    ///
    /// For the domain-generic engine: `Observation`
    /// For PV systems: could be `Icsr`, `SignalCandidate`, etc.
    type Obs;

    // ── Core Pipeline (all 6 composites) ──────────────────────────────────

    /// Ingest a single observation, running all detection pipelines.
    ///
    /// The 6-stage pipeline:
    /// 1. Suddenness detection (σ + ∂ + N + κ) — threshold crossing before state update
    /// 2. Recognition (κ + ∃ + σ) — match against known patterns
    /// 3. Novelty detection (∅ + ∃ + σ) — if not recognized, detect novelty
    /// 4. Co-occurrence update — prepare for pattern formation
    /// 5. Pattern detection (σ + κ + μ) — check for new patterns
    /// 6. State accumulation (ς) — store observation, record events
    ///
    /// Returns insight events produced by this observation.
    fn ingest(&mut self, observation: Self::Obs) -> Vec<InsightEvent>;

    /// Ingest a batch of observations.
    ///
    /// Default implementation processes sequentially. Override for
    /// parallel/streaming implementations.
    fn ingest_batch(&mut self, observations: Vec<Self::Obs>) -> Vec<InsightEvent> {
        let mut all_events = Vec::new();
        for obs in observations {
            all_events.extend(self.ingest(obs));
        }
        all_events
    }

    // ── Active Operations ─────────────────────────────────────────────────

    /// Establish a connection between two elements (Connection composite).
    ///
    /// μ + κ + ς: Maps a relationship, compares strength, changes state.
    fn connect(&mut self, from: &str, to: &str, relation: &str, strength: f64) -> Connection;

    /// Compress observations into a principle (Compression composite).
    ///
    /// N + μ + κ: Reduces quantity through mapping and comparison.
    fn compress(&mut self, keys: Vec<String>, principle: &str) -> Compression;

    // ── Queries ───────────────────────────────────────────────────────────

    /// Access accumulated insight events (append-only, ς-acc).
    fn events(&self) -> &[InsightEvent];

    /// Number of observations processed.
    fn observation_count(&self) -> usize;

    /// Number of detected patterns.
    fn pattern_count(&self) -> usize;

    /// Access all detected patterns.
    fn patterns(&self) -> Vec<&Pattern>;

    /// Access all discovered connections.
    fn connections(&self) -> &[Connection];

    /// Find connections involving a specific key.
    fn connections_for(&self, key: &str) -> Vec<&Connection> {
        self.connections()
            .iter()
            .filter(|c| c.involves(key))
            .collect()
    }

    /// Number of unique observation keys (distinct entities observed).
    fn unique_key_count(&self) -> usize;
}
