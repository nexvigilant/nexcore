//! # GroundsTo implementations for nexcore-jeopardy types
//!
//! Connects Jeopardy!-inspired game theory types to the Lex Primitiva type system.
//!
//! ## Key Primitive Distribution
//!
//! - Board traversal: sigma (Sequence) -- ordered selection
//! - Risk evaluation: kappa (Comparison) -- threshold comparison
//! - Score tracking: N (Quantity) -- numeric values
//! - Round dispatch: Sigma (Sum) -- categorical routing
//! - Buzz decisions: partial (Boundary) -- confidence thresholds

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::prelude::*;

// ---------------------------------------------------------------------------
// Atomic types -- T1/T2-P
// ---------------------------------------------------------------------------

/// Round: T1 (Sigma), pure sum
///
/// Jeopardy, DoubleJeopardy, FinalJeopardy -- categorical game phases.
impl GroundsTo for Round {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// ClueValue: T1 (N), pure quantity
///
/// Dollar value of a clue (200, 400, 600, 800, 1000).
impl GroundsTo for ClueValue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

// Confidence: GroundsTo impl is in nexcore-constants::grounding (canonical source).
// Removed local impl to eliminate F2 equivocation — see vocabulary::CONFIDENCE.

/// Category: T2-P (Sigma + sigma), dominant Sigma
///
/// A quiz category (e.g., "History", "Science").
/// Sum-dominant: it selects one from available categories.
impl GroundsTo for Category {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- category selection
            LexPrimitiva::Sequence, // sigma -- ordered in board layout
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// CluePosition: T2-P (lambda + N), dominant lambda
///
/// Grid position of a clue (category, row).
/// Location-dominant: it IS a position on the board.
impl GroundsTo for CluePosition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // lambda -- board position
            LexPrimitiva::Quantity, // N -- row/column indices
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Composite types
// ---------------------------------------------------------------------------

/// Clue: T2-C (N + Sigma + lambda + exists), dominant N
///
/// A single clue with category, value, answer, and daily double flag.
/// Quantity-dominant: the clue's value drives strategic decisions.
impl GroundsTo for Clue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- clue value
            LexPrimitiva::Sum,       // Sigma -- category routing
            LexPrimitiva::Location,  // lambda -- board position
            LexPrimitiva::Existence, // exists -- daily double flag
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// Board: T2-C (sigma + lambda + varsigma + N), dominant sigma
///
/// Game board grid with categories and clues.
/// Sequence-dominant: the board defines traversal order for strategy.
impl GroundsTo for Board {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- board traversal order
            LexPrimitiva::Location, // lambda -- grid positions
            LexPrimitiva::State,    // varsigma -- revealed/unrevealed state
            LexPrimitiva::Quantity, // N -- clue values
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// Cell: T2-P (varsigma + exists), dominant varsigma
///
/// A single board cell that can be empty or contain a clue.
/// State-dominant: the cell IS a state (Empty, Available, Revealed).
impl GroundsTo for Cell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- cell state
            LexPrimitiva::Existence, // exists -- clue existence
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// Player: T2-P (varsigma + N), dominant varsigma
///
/// A game player with name and running score.
impl GroundsTo for Player {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- player state
            LexPrimitiva::Quantity, // N -- score
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// BuzzDecision: T2-P (kappa + partial), dominant kappa
///
/// Result of should_buzz: Buzz or Pass with reasoning.
/// Comparison-dominant: the decision compares confidence against threshold.
impl GroundsTo for BuzzDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- confidence vs threshold
            LexPrimitiva::Boundary,   // partial -- buzz threshold boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Wager: T2-P (N + kappa), dominant N
///
/// A calculated wager amount for Daily Double or Final Jeopardy.
/// Quantity-dominant: the wager IS a numeric amount.
impl GroundsTo for Wager {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- wager amount
            LexPrimitiva::Comparison, // kappa -- risk assessment
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// SelectionScore: T2-P (N + kappa + lambda), dominant N
///
/// Score for a potential board selection, used in strategy optimization.
impl GroundsTo for SelectionScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric score
            LexPrimitiva::Comparison, // kappa -- score comparison
            LexPrimitiva::Location,   // lambda -- board position
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Domain types -- T3
// ---------------------------------------------------------------------------

/// GameState: T3 (varsigma + sigma + N + Sigma + kappa + lambda), dominant varsigma
///
/// Complete game state: players, board, round, current player.
/// State-dominant: this is the full mutable state of a game.
impl GroundsTo for GameState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- game state
            LexPrimitiva::Sequence,   // sigma -- turn progression
            LexPrimitiva::Quantity,   // N -- scores, values
            LexPrimitiva::Sum,        // Sigma -- round classification
            LexPrimitiva::Comparison, // kappa -- scoring, ranking
            LexPrimitiva::Location,   // lambda -- board positions
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// CompoundMetrics: T2-C (N + nu + sigma + kappa), dominant N
///
/// Cross-game compound velocity metrics.
/// Quantity-dominant: velocity, efficiency, accumulation are numeric.
impl GroundsTo for CompoundMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- velocity, efficiency numbers
            LexPrimitiva::Frequency,  // nu -- games-per-time frequency
            LexPrimitiva::Sequence,   // sigma -- game sequence
            LexPrimitiva::Comparison, // kappa -- threshold comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// GameResult: T2-P (N + kappa), dominant N
///
/// Outcome of a single game: scores, winner, accumulated primitives.
impl GroundsTo for GameResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- scores
            LexPrimitiva::Comparison, // kappa -- win/loss comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// JeopardyError: T2-P (partial + Sigma), dominant partial
impl GroundsTo for JeopardyError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn round_is_t1() {
        assert_eq!(Round::tier(), Tier::T1Universal);
        assert!(Round::is_pure_primitive());
    }

    #[test]
    fn clue_value_is_t1() {
        assert_eq!(ClueValue::tier(), Tier::T1Universal);
        assert!(ClueValue::is_pure_primitive());
    }

    #[test]
    fn board_is_t2c() {
        assert_eq!(Board::tier(), Tier::T2Composite);
        assert_eq!(Board::dominant_primitive(), Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn game_state_is_t3() {
        assert_eq!(GameState::tier(), Tier::T3DomainSpecific);
        assert_eq!(GameState::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn buzz_decision_comparison_dominant() {
        assert_eq!(
            BuzzDecision::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            Round::primitive_composition(),
            ClueValue::primitive_composition(),
            Confidence::primitive_composition(),
            Category::primitive_composition(),
            CluePosition::primitive_composition(),
            Clue::primitive_composition(),
            Board::primitive_composition(),
            Cell::primitive_composition(),
            Player::primitive_composition(),
            BuzzDecision::primitive_composition(),
            Wager::primitive_composition(),
            SelectionScore::primitive_composition(),
            GameState::primitive_composition(),
            CompoundMetrics::primitive_composition(),
            GameResult::primitive_composition(),
            JeopardyError::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
