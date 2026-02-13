// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # NexVigilant Core — Value Mining
//!
//! Economic value signal detection using pharmacovigilance algorithms.
//!
//! ## Signal Types
//!
//! | Signal | PV Analog | Description |
//! |--------|-----------|-------------|
//! | Sentiment | PRR | Unexpected positive/negative sentiment vs baseline |
//! | Trend | IC | Directional momentum over time |
//! | Engagement | ROR | Unusual engagement rate vs baseline |
//! | Virality | EBGM | Exponential growth phase detection |
//! | Controversy | Chi² | High sentiment variance (polarization) |
//!
//! ## Primitive Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | Sentiment Value | Quantity + Comparison | N + κ |
//! | Trend Direction | Sequence + Causality | σ + → |
//! | Engagement Rate | Frequency + Quantity | ν + N |
//! | Virality Threshold | Boundary + Irreversibility | ∂ + ∝ |
//! | Controversy Index | Comparison + State | κ + ς |
//! | Signal Persistence | Persistence + Frequency | π + ν |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

pub mod error;
pub mod goals;
pub mod grounding;
pub mod intelligence;
pub mod signals;
pub mod types;

// Re-exports
pub use error::{MiningError, MiningResult};
pub use goals::{GoalMetric, GoalPortfolio, GoalProgress, GoalStatus, IntelligenceGoal, Priority};
pub use intelligence::{
    ActionRecommendation, IntelligenceConfidence, IntelligenceState, MonitoringDashboard,
    ResponseAction, SignalConvergence, SourceDiversity, TemporalAlignment, ValueIntelligence,
};
pub use signals::{
    ControversyDetector, EngagementDetector, SentimentDetector, SignalDetector, TrendDetector,
    ViralityDetector,
};
pub use types::{Baseline, SignalStrength, SignalType, ValueSignal};

/// Prelude for common imports.
pub mod prelude {
    //! Common imports for value mining.
    pub use crate::error::{MiningError, MiningResult};
    pub use crate::goals::{GoalMetric, GoalPortfolio, GoalStatus, IntelligenceGoal, Priority};
    pub use crate::intelligence::{
        ActionRecommendation, IntelligenceState, ResponseAction, ValueIntelligence,
    };
    pub use crate::signals::SignalDetector;
    pub use crate::types::{SignalStrength, SignalType, ValueSignal};
}
