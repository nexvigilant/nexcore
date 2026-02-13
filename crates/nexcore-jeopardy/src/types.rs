//! Core domain types: Category, ClueValue, Round, Clue, Confidence.
//!
//! These are the T2-P and T2-C primitives that compose the Jeopardy domain.

use serde::{Deserialize, Serialize};

/// Algorithm development categories mapped to Jeopardy board columns.
///
/// Each category represents a domain of algorithm R&D investment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    /// PRR, ROR, IC, EBGM, Chi-squared signal detection algorithms.
    SignalDetection,
    /// T1/T2-P/T2-C primitive mining and extraction.
    PrimitiveExtraction,
    /// Chemistry-to-PV, STEM-to-PV knowledge transfer.
    CrossDomainTransfer,
    /// CTVP 5-phase validation pipeline.
    ValidationPhasing,
    /// V(t) = B * eta * r compound growth modeling.
    CompoundGrowth,
    /// Aggregation, routing, and orchestration patterns.
    PipelineOrchestration,
}

impl Category {
    /// Returns all categories in standard board order.
    pub fn all() -> &'static [Category] {
        &[
            Category::SignalDetection,
            Category::PrimitiveExtraction,
            Category::CrossDomainTransfer,
            Category::ValidationPhasing,
            Category::CompoundGrowth,
            Category::PipelineOrchestration,
        ]
    }

    /// Returns the compound growth multiplier for this category.
    ///
    /// T2-P categories compound 29% faster than T2-C per the triangular transfer law.
    pub fn compound_multiplier(&self) -> f64 {
        match self {
            // T2-P categories: higher compound rate (0.9 cost = faster compounding)
            Category::PrimitiveExtraction => 1.29,
            Category::SignalDetection => 1.29,
            // T2-C categories: standard compound rate
            Category::CrossDomainTransfer => 1.0,
            Category::ValidationPhasing => 1.0,
            Category::CompoundGrowth => 1.0,
            Category::PipelineOrchestration => 1.0,
        }
    }

    /// Display name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            Category::SignalDetection => "Signal Detection",
            Category::PrimitiveExtraction => "Primitive Extraction",
            Category::CrossDomainTransfer => "Cross-Domain Transfer",
            Category::ValidationPhasing => "Validation Phasing",
            Category::CompoundGrowth => "Compound Growth",
            Category::PipelineOrchestration => "Pipeline Orchestration",
        }
    }
}

/// Clue dollar values, mapped to investment units.
///
/// In standard Jeopardy round: 200, 400, 600, 800, 1000.
/// In Double Jeopardy: 400, 800, 1200, 1600, 2000.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClueValue(pub u64);

impl ClueValue {
    /// Standard Jeopardy round values.
    pub const JEOPARDY_VALUES: [ClueValue; 5] = [
        ClueValue(200),
        ClueValue(400),
        ClueValue(600),
        ClueValue(800),
        ClueValue(1000),
    ];

    /// Double Jeopardy round values (2x).
    pub const DOUBLE_JEOPARDY_VALUES: [ClueValue; 5] = [
        ClueValue(400),
        ClueValue(800),
        ClueValue(1200),
        ClueValue(1600),
        ClueValue(2000),
    ];

    /// Values for a given round.
    pub fn for_round(round: Round) -> &'static [ClueValue; 5] {
        match round {
            Round::Jeopardy => &Self::JEOPARDY_VALUES,
            Round::DoubleJeopardy => &Self::DOUBLE_JEOPARDY_VALUES,
            Round::FinalJeopardy => &Self::DOUBLE_JEOPARDY_VALUES, // Not used, but defined
        }
    }
}

/// Game rounds mapped to development phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Round {
    /// Preclinical phase: low stakes, build foundation.
    Jeopardy,
    /// Clinical phase: higher stakes, 2x investment.
    DoubleJeopardy,
    /// Post-market phase: all-in wagering on proven algorithms.
    FinalJeopardy,
}

impl Round {
    /// Multiplier applied to base values in this round.
    pub fn value_multiplier(&self) -> u64 {
        match self {
            Round::Jeopardy => 1,
            Round::DoubleJeopardy => 2,
            Round::FinalJeopardy => 1, // Final uses wager-based scoring
        }
    }

    /// The next round after this one, if any.
    pub fn next(&self) -> Option<Round> {
        match self {
            Round::Jeopardy => Some(Round::DoubleJeopardy),
            Round::DoubleJeopardy => Some(Round::FinalJeopardy),
            Round::FinalJeopardy => None,
        }
    }
}

/// A problem on the algorithm development board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clue {
    /// Which category this clue belongs to.
    pub category: Category,
    /// Dollar value (investment units).
    pub value: ClueValue,
    /// Difficulty rating from 0.0 (trivial) to 1.0 (extremely hard).
    pub difficulty: f64,
    /// Whether this is a Daily Double (hidden high-wager opportunity).
    pub is_daily_double: bool,
}

/// Tier: T2-P (N + ∂ — bounded probability)
///
/// Canonical confidence score in [0.0, 1.0].
/// Re-exported from `nexcore-constants` to eliminate F2 equivocation.
///
/// Note: previous jeopardy-local Confidence used validated `new()` returning
/// `Result`. The canonical version uses clamping `new()` returning `Self`.
/// Callers that passed in-range values are unaffected. Out-of-range values
/// are now clamped instead of rejected.
pub use nexcore_constants::Confidence;

/// Position of a clue on the board (row, category_index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CluePosition {
    /// Row index (0 = lowest value, 4 = highest value).
    pub row: usize,
    /// Category column index (0-5).
    pub col: usize,
}

impl CluePosition {
    /// Create a new position.
    pub fn new(row: usize, col: usize) -> Self {
        CluePosition { row, col }
    }
}
