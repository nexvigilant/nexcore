//! # NexVigilant Core — insight
//!
//! Insight engine: pattern detection, novelty recognition, connection mapping,
//! and observation compression grounded in Lex Primitiva T1 primitives.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod adapters;
pub mod composites;
pub mod engine;
pub mod grounding;
pub mod orchestrator;
pub mod persist;
pub mod traits;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::adapters::{
        BrainInsightAdapter, FaersInsightAdapter, GuardianInsightAdapter, PvInsightAdapter,
    };
    pub use crate::composites::{
        Compression, Connection, Novelty, NoveltyReason, Pattern, Recognition, Suddenness,
    };
    pub use crate::engine::{InsightConfig, InsightEngine, InsightEvent, Observation};
    pub use crate::orchestrator::{NexCoreInsight, NexCoreInsightSummary};
    pub use crate::traits::Insight;
}

// Re-export main types at crate root.
pub use adapters::{
    BrainInsightAdapter, FaersInsightAdapter, GuardianInsightAdapter, PvInsightAdapter,
};
pub use composites::{Compression, Connection, Novelty, Pattern, Recognition, Suddenness};
pub use engine::{InsightConfig, InsightEngine, InsightEvent, Observation};
pub use orchestrator::{NexCoreInsight, NexCoreInsightSummary};
pub use traits::Insight;
