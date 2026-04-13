//! # perception
//!
//! Environmental Perception pipeline — a 4-layer system for ingesting,
//! normalising, fusing, and maintaining a live world model of adverse-event
//! signals from heterogeneous data sources.
//!
//! ## Architecture
//!
//! ```text
//! [Layer 1] SourceConnectors  →  RawRecord stream
//! [Layer 2] Normalizer        →  NormalizedRecord + DeduplicationEngine
//! [Layer 3] FusionEngine      →  FusionResult (Singleton | Fused | Conflict)
//! [Layer 4] WorldModel        →  EntityState + ChangeEvent broadcast
//! ```
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use perception::{PerceptionPipeline, config::PerceptionConfig};
//! use std::sync::Arc;
//!
//! # async fn run() -> Result<(), perception::error::PerceptionError> {
//! let pipeline = PerceptionPipeline::from_config(PerceptionConfig::default());
//! let mut changes = pipeline.subscribe_changes();
//! // pipeline.run().await?;  // drives the full processing loop
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

pub mod config;
pub mod connector;
pub mod error;
pub mod fusion;
pub mod normalizer;
pub mod registry;
pub mod types;
pub mod world_model;

use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::config::PerceptionConfig;
use crate::connector::{IngestionOrchestrator, SourceConnector};
use crate::error::{PerceptionError, Result};
use crate::fusion::{ArbitrationQueue, FusionEngine};
use crate::normalizer::{DedupOutcome, DeduplicationEngine, HoldQueue, JsonNormalizer, Normalizer};
use crate::registry::SourceRegistry;
use crate::types::{ChangeEvent, FusionResult, RawRecord};
use crate::world_model::WorldModel;

// ── In-process arbitration queue ──────────────────────────────────────────────

/// Default in-process arbitration queue backed by a `Mutex<Vec>`.
pub struct InMemoryArbitrationQueue(std::sync::Mutex<Vec<FusionResult>>);

impl InMemoryArbitrationQueue {
    /// Create a new empty arbitration queue.
    pub fn new() -> Self {
        Self(std::sync::Mutex::new(Vec::new()))
    }
}

impl Default for InMemoryArbitrationQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ArbitrationQueue for InMemoryArbitrationQueue {
    fn push(&self, result: FusionResult) {
        self.0.lock().expect("mutex poisoned").push(result);
    }
    fn drain(&self) -> Vec<FusionResult> {
        self.0.lock().expect("mutex poisoned").drain(..).collect()
    }
    fn len(&self) -> usize {
        self.0.lock().expect("mutex poisoned").len()
    }
}

// ── In-process hold queue ──────────────────────────────────────────────────────

/// Default in-process hold queue backed by a `Mutex<Vec>`.
pub struct InMemoryHoldQueue(std::sync::Mutex<Vec<crate::types::NormalizedRecord>>);

impl InMemoryHoldQueue {
    /// Create a new empty hold queue.
    pub fn new() -> Self {
        Self(std::sync::Mutex::new(Vec::new()))
    }
}

impl Default for InMemoryHoldQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl HoldQueue for InMemoryHoldQueue {
    fn push(&self, record: crate::types::NormalizedRecord) {
        self.0.lock().expect("mutex poisoned").push(record);
    }
    fn drain(&self) -> Vec<crate::types::NormalizedRecord> {
        self.0.lock().expect("mutex poisoned").drain(..).collect()
    }
    fn len(&self) -> usize {
        self.0.lock().expect("mutex poisoned").len()
    }
}

// ── Perception pipeline ────────────────────────────────────────────────────────

/// The top-level 4-layer perception pipeline.
///
/// Wire up connectors, normalizers, registry, and world model via
/// [`from_config`][Self::from_config] (defaults) or construct the sub-systems
/// manually for full control.
pub struct PerceptionPipeline {
    config: PerceptionConfig,
    orchestrator: IngestionOrchestrator,
    normalizers: Vec<Arc<dyn Normalizer>>,
    registry: Arc<SourceRegistry>,
    world_model: Arc<WorldModel>,
    hold_queue: Arc<dyn HoldQueue>,
    arbitration: Arc<dyn ArbitrationQueue>,
}

impl PerceptionPipeline {
    /// Build a pipeline with default in-memory components and no connectors.
    ///
    /// Add connectors with [`add_connector`][Self::add_connector] before
    /// calling [`run`][Self::run].
    pub fn from_config(config: PerceptionConfig) -> Self {
        let registry = Arc::new(SourceRegistry::new());
        let world_model = Arc::new(WorldModel::new(
            Arc::new(world_model::InMemoryStateStore::new()),
            config.max_staleness_hours,
            1024,
        ));

        Self {
            orchestrator: IngestionOrchestrator::new(vec![], config.ingestion_buffer_size),
            normalizers: Vec::new(),
            registry,
            world_model,
            hold_queue: Arc::new(InMemoryHoldQueue::new()),
            arbitration: Arc::new(InMemoryArbitrationQueue::new()),
            config,
        }
    }

    /// Build a pipeline with explicit sub-system dependencies (for testing).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: PerceptionConfig,
        connectors: Vec<Arc<dyn SourceConnector>>,
        normalizers: Vec<Arc<dyn Normalizer>>,
        registry: Arc<SourceRegistry>,
        world_model: Arc<WorldModel>,
        hold_queue: Arc<dyn HoldQueue>,
        arbitration: Arc<dyn ArbitrationQueue>,
    ) -> Self {
        let orchestrator = IngestionOrchestrator::new(connectors, config.ingestion_buffer_size);
        Self {
            config,
            orchestrator,
            normalizers,
            registry,
            world_model,
            hold_queue,
            arbitration,
        }
    }

    /// Subscribe to world-model change events.
    pub fn subscribe_changes(&self) -> broadcast::Receiver<ChangeEvent> {
        self.world_model.subscribe_changes()
    }

    /// Borrow the world model for direct queries.
    pub fn world_model(&self) -> &WorldModel {
        &self.world_model
    }

    /// Run the pipeline until the ingestion channel is closed.
    ///
    /// Processing loop (8 steps per record):
    /// 1. Receive raw record from orchestrator channel.
    /// 2. Select normalizer by source ID (fall back to JsonNormalizer).
    /// 3. Normalize raw record → NormalizedRecord.
    /// 4. DeduplicationEngine: check fingerprint.
    /// 5. Unique → ingest into FusionEngine; Duplicate → drop; HoldForReview → hold queue.
    /// 6. Periodically flush fusion engine → FusionResult list.
    /// 7. Apply each FusionResult to the WorldModel.
    /// 8. Emit ChangeEvents to broadcast channel (handled inside WorldModel::update).
    pub async fn run(&self) -> Result<()> {
        let mut rx = self.orchestrator.start();
        let mut dedup =
            DeduplicationEngine::new(self.config.dedup_window_hours, self.config.merge_threshold);
        let mut fusion = FusionEngine::new(self.config.arbitration_threshold);
        let mut records_since_flush: usize = 0;
        const FLUSH_INTERVAL: usize = 100;

        info!("perception pipeline started");

        while let Some(raw) = rx.recv().await {
            // Step 2 — select normalizer
            let normalizer = self.select_normalizer(&raw);

            // Step 3 — normalize
            let normalized = match normalizer.normalize(raw).await {
                Ok(n) => n,
                Err(e) => {
                    error!(error = %e, "normalization failed — skipping record");
                    continue;
                }
            };

            // Step 4–5 — dedup
            match dedup.check(&normalized) {
                DedupOutcome::Unique => {
                    fusion.ingest(normalized);
                    records_since_flush += 1;
                }
                DedupOutcome::Duplicate => {
                    // Drop silently — telemetry hook point
                }
                DedupOutcome::HoldForReview => {
                    self.hold_queue.push(normalized);
                }
            }

            // Step 6 — periodic flush
            if records_since_flush >= FLUSH_INTERVAL {
                self.flush_fusion(&mut fusion).await;
                records_since_flush = 0;
            }
        }

        // Final flush on channel close
        self.flush_fusion(&mut fusion).await;

        info!("perception pipeline terminated — channel closed");
        Ok(())
    }

    // ── Private ────────────────────────────────────────────────────────────────

    fn select_normalizer(&self, raw: &RawRecord) -> Arc<dyn Normalizer> {
        self.normalizers
            .iter()
            .find(|n| n.source_id() == &raw.source_id)
            .cloned()
            .unwrap_or_else(|| Arc::new(JsonNormalizer::new(raw.source_id.clone())))
    }

    async fn flush_fusion(&self, fusion: &mut FusionEngine) {
        let results = fusion.fuse(&self.registry, self.arbitration.as_ref());
        for result in results {
            self.world_model.update(result);
        }
    }
}
