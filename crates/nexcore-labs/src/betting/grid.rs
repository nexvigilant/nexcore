//! # Betting Constraint Satisfaction Grid
//!
//! Implements belief propagation for sports betting markets based on
//! Sentinel CSP (ToV §33) and Campion Signal Theory.
//!
//! ## Theory Application
//!
//! - **U (Market Anomaly)**: Unexpected line movement relative to public.
//! - **R (Source Reliability)**: Confidence in the odds provider/sharp signal.
//! - **T (Decay Window)**: Information half-life as game start approaches.
//!
//! # Codex Compliance
//! - **Tier**: T2-C / T3
//! - **Commandments**: I (Quantify), II (Classify), III (Ground), V (Wrap)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

use super::converter::SignalConverter;
use super::ecs::{EcsResult, ReliabilityInput, calculate_ecs};
use super::exchange::{MarketId, MarketOrder};
use super::temporal::SportType;

// =============================================================================
// CONSTANTS - T1 Primitives
// =============================================================================

/// Default weight for Structural adjacency (Spread ↔ ML).
pub const WEIGHT_STRUCTURAL: f64 = 0.85;
/// Default weight for Correlated adjacency (Same Team).
pub const WEIGHT_CORRELATED: f64 = 0.40;
/// Default weight for Microstructure adjacency (Related Games).
pub const WEIGHT_MICROSTRUCTURE: f64 = 0.25;

// =============================================================================
// ENUMS - Tier: T2-P / T2-C
// =============================================================================

/// Status of a betting market outcome.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MarketStatus {
    /// No significant action detected.
    #[default]
    Vacant = 0,
    /// Actively being analyzed.
    Analyzing = 1,
    /// Actionable signal detected (Direct evidence).
    Actionable = 2,
    /// High-value signal (Elite).
    HighValue = 3,
    /// Flagged via propagation (Indirect evidence).
    HiddenAlpha = 4,
    /// Automatically traded.
    AutoTraded = 5,
    /// Closed/Resulted.
    Settled = 6,
}

impl Ord for MarketStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for MarketStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Types of correlation between market outcomes.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdjacencyType {
    /// Same game, different market (e.g., Spread and Moneyline).
    Structural,
    /// Same team, different game (e.g., Team A Game 1 and Game 2).
    Correlated,
    /// Related games (e.g., Division rivals, Playoff implications).
    Microstructure,
}

/// Propagation weight between market cells.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdjacencyWeight(pub f64);

impl AdjacencyType {
    /// Default propagation weight for this adjacency.
    #[must_use]
    pub const fn default_weight(&self) -> AdjacencyWeight {
        match self {
            Self::Structural => AdjacencyWeight(WEIGHT_STRUCTURAL),
            Self::Correlated => AdjacencyWeight(WEIGHT_CORRELATED),
            Self::Microstructure => AdjacencyWeight(WEIGHT_MICROSTRUCTURE),
        }
    }
}

// =============================================================================
// TYPES - Tier: T2-C / T3
// =============================================================================

/// A single market outcome in the grid.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketCell {
    /// Unique identifier (game_id|market_type|outcome).
    pub id: String,
    /// Current intelligence score (Edge).
    pub edge_score: f64,
    /// Latest ECS components.
    pub ecs: Option<EcsResult>,
    /// Current status.
    pub status: MarketStatus,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Number of evidence points (e.g., sharp moves).
    pub evidence_count: u32,
}

impl MarketCell {
    /// Create a new market cell.
    #[must_use]
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            edge_score: 0.0,
            ecs: None,
            status: MarketStatus::Vacant,
            updated_at: Utc::now(),
            evidence_count: 0,
        }
    }

    /// Update cell with new direct evidence.
    pub fn update_direct(
        &mut self,
        public_pct: f64,
        line_move_dir: i8,
        steam_detected: bool,
        reliability: &ReliabilityInput,
        hours_to_game: f64,
        sport: SportType,
    ) {
        let ecs_res = calculate_ecs(
            public_pct,
            line_move_dir,
            steam_detected,
            reliability,
            hours_to_game,
            sport,
        );

        self.edge_score = ecs_res.ecs;
        self.status = if ecs_res.ecs >= 5.0 {
            MarketStatus::HighValue
        } else if ecs_res.is_actionable {
            MarketStatus::Actionable
        } else {
            MarketStatus::Analyzing
        };

        self.ecs = Some(ecs_res);
        self.updated_at = Utc::now();
        self.evidence_count += 1;
    }

    /// Apply propagated edge from a neighbor.
    ///
    /// Formula: E_new = E_current + (weight * E_neighbor)
    pub fn apply_propagation(&mut self, source_edge: f64, weight: f64) {
        let added_edge = source_edge * weight;
        self.edge_score += added_edge;

        if self.status == MarketStatus::Vacant && self.edge_score >= 2.0 {
            self.status = MarketStatus::HiddenAlpha;
        }

        self.updated_at = Utc::now();
    }
}

/// The Betting Constraint Satisfaction Grid.
///
/// # Tier: T3
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BettingGrid {
    /// Market outcomes indexed by ID.
    pub cells: HashMap<String, MarketCell>,
    /// Adjacency graph: source_id -> (target_id, weight).
    pub adjacency: HashMap<String, Vec<(String, f64)>>,
}

// Imports already at top of file

impl BettingGrid {
    /// Create a new empty grid.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a market cell to the grid.
    pub fn add_market(&mut self, id: &str) -> &mut MarketCell {
        self.cells
            .entry(id.to_string())
            .or_insert_with(|| MarketCell::new(id))
    }

    /// Set relationship between markets.
    pub fn set_correlation(&mut self, a: &str, b: &str, adj_type: AdjacencyType) {
        let weight = adj_type.default_weight();

        self.adjacency
            .entry(a.to_string())
            .or_default()
            .push((b.to_string(), weight.0));
        self.adjacency
            .entry(b.to_string())
            .or_default()
            .push((a.to_string(), weight.0));
    }

    /// Propagate intelligence from a source market.
    pub fn propagate(&mut self, source_id: &str) -> Vec<String> {
        let source_edge = match self.cells.get(source_id) {
            Some(cell) => cell.edge_score,
            None => return Vec::new(),
        };

        let mut newly_flagged = Vec::new();
        let targets = self.adjacency.get(source_id).cloned().unwrap_or_default();

        for (target_id, weight) in targets {
            if let Some(target_cell) = self.cells.get_mut(&target_id) {
                let was_vacant = target_cell.status == MarketStatus::Vacant;

                target_cell.apply_propagation(source_edge, weight);

                if was_vacant && target_cell.status == MarketStatus::HiddenAlpha {
                    newly_flagged.push(target_id);
                }
            }
        }

        newly_flagged
    }

    /// Search for actionable alpha and propose market orders.
    ///
    /// # Logic
    /// 1. Iterates over all cells.
    /// 2. If status is HighValue or HiddenAlpha, uses converter to generate orders.
    /// 3. Returns a list of proposed orders.
    pub fn search_alpha(
        &mut self,
        converter: &SignalConverter,
        base_quantity: u64,
    ) -> Vec<MarketOrder> {
        let mut proposals = Vec::new();

        for (id, cell) in self.cells.iter_mut() {
            if cell.status == MarketStatus::HighValue || cell.status == MarketStatus::HiddenAlpha {
                if let Some(ecs) = &cell.ecs {
                    if let Some(order) =
                        converter.convert_from_ecs(MarketId(id.clone()), ecs, base_quantity)
                    {
                        proposals.push(order);
                    }
                }
            }
        }

        proposals
    }
}
