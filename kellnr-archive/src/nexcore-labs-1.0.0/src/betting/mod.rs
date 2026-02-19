//! Betting domain module.
//!
//! Handles sports betting analytics and signal detection using
//! pharmacovigilance-grade methodology.
//!
//! # Modules
//! - `bdi` - Betting Disproportionality Index (PRR-adapted)
//! - `ecs` - Edge Confidence Score (EBGM-adapted)
//! - `classifier` - Combined BDI+ECS signal classification
//! - `proxy` - Proxy BDI when public betting % unavailable
//! - `temporal` - Temporal decay functions
//! - `thresholds` - Evans-adapted detection thresholds

pub mod bdi;
pub mod classifier;
pub mod converter;
pub mod ecs;
pub mod exchange;
pub mod grid;
pub mod market_data;
pub mod proxy;
pub mod proxy_bdi;
pub mod sentinel;
pub mod temporal;
pub mod thresholds;

// Re-exports - Core types
pub use bdi::{
    Bdi, BdiError, BdiResult, BdiSignalType, BdiThresholds, ChiSquare, CiBound, ContingencyTable,
    PValue, calculate_bdi, calculate_bdi_with_options,
};
pub use ecs::{
    CredibilityLower, CredibilityUpper, EcsComponents, EcsResult, EcsResultExtended, EcsScore,
    Reliability, ReliabilityInput, TemporalComponent, Unexpectedness, calculate_ecs,
    calculate_ecs_additive, calculate_ecs_full, calculate_ecs_simple,
};
pub use grid::{AdjacencyType, BettingGrid, MarketCell, MarketStatus};
pub use proxy::{BookClassification, ProxyBdiInput, calculate_proxy_bdi};
pub use proxy_bdi::{
    BookOdds as ProxyBookOdds, DataQualityScore, DirectionScore, DivergenceScore,
    LineMovementMetrics, MovementDirection, MovementMagnitude, ProxyBdiConfig, ProxyBdiResult,
    ProxySignalType, VelocityScore, calculate_movement_metrics,
    calculate_proxy_bdi as calculate_proxy_bdi_full, calculate_proxy_bdi_from_metrics,
};
pub use temporal::{
    ActionWindow, DecayFactor, DecayProfile, HoursToGame, LambdaCoefficient, SportType,
    TemporalDecay, TemporalError, TemporalZone, calculate_half_life, calculate_lambda_for_target,
    calculate_optimal_action_window, calculate_temporal_decay, temporal_decay,
};
pub use thresholds::{BDI_ELITE, BDI_MODERATE, BDI_STRONG, ECS_MODERATE, SignalStrength};

// Re-exports - Classifier (primary API)
pub use classifier::{
    BettingSignalInput, ClassificationConfig, LineMovement, PublicBetting, SignalClassification,
    SignalType, classify_quick, classify_signal,
};

/// Unique identifier for a betting market.
pub type MarketId = String;
/// Numerical representation of odds.
pub type Odds = f64;
/// Spread value (e.g. -3.5).
pub type Spread = f64;
