//! # GroundsTo implementations for nexcore-labs types
//!
//! Connects experimental lab module types to the Lex Primitiva type system.
//!
//! ## Key Domain Mapping
//!
//! - **Quiz**: Sequence (sigma) for question ordering, Sum (Sigma) for answer types
//! - **Betting**: Comparison (kappa) for threshold detection, Quantity (N) for odds
//! - **Academy**: Mapping (mu) for competency assessment, Persistence (pi) for learning

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::betting::classifier::SignalClassification;
use crate::betting::exchange::{ExchangeResult, ExchangeStatus, MarketOrder, OrderSide};
use crate::crypto::CryptoEngine;
use crate::quiz::config::{QuizConfig, StorageBackend};
use crate::quiz::domain::game::{GamePlayer, GameResults, GameSession, PlayGame};
use crate::quiz::domain::quiz::{QuestionType, Quiz, QuizQuestion};
use crate::quiz::domain::user::User;
use crate::quiz::error::QuizError;
use crate::quiz::question_types::QuizAnswer;

// ===========================================================================
// Quiz Module -- sigma (Sequence) dominant
// ===========================================================================

/// QuizAnswer: T2-P (Sigma + N), dominant Sigma
///
/// Multi-variant answer type: ABCD, Range, Voting, Text, Check, Order.
impl GroundsTo for QuizAnswer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- answer type variant
            LexPrimitiva::Quantity, // N -- numeric answer content
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// QuizConfig: T2-P (varsigma + partial), dominant varsigma
///
/// Configuration for a quiz session.
impl GroundsTo for QuizConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- configuration state
            LexPrimitiva::Boundary, // partial -- config limits
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// StorageBackend: T1 (Sigma), pure sum
///
/// Storage backend selection: InMemory, FileSystem.
impl GroundsTo for StorageBackend {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// QuestionType: T1 (Sigma), pure sum
///
/// Categorical question type: MultipleChoice, TrueFalse, OpenEnded, etc.
impl GroundsTo for QuestionType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// Quiz: T2-C (sigma + Sigma + varsigma + pi), dominant sigma
///
/// A quiz with ordered questions and metadata.
/// Sequence-dominant: questions follow a defined order.
impl GroundsTo for Quiz {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // sigma -- question ordering
            LexPrimitiva::Sum,         // Sigma -- question type routing
            LexPrimitiva::State,       // varsigma -- quiz state
            LexPrimitiva::Persistence, // pi -- stored quiz data
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// QuizQuestion: T2-P (Sigma + sigma + N), dominant Sigma
///
/// A single question with type, content, and point value.
impl GroundsTo for QuizQuestion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- question type
            LexPrimitiva::Sequence, // sigma -- position in quiz
            LexPrimitiva::Quantity, // N -- point value
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// PlayGame: T2-C (varsigma + sigma + N + Sigma), dominant varsigma
///
/// Active game session with players and state tracking.
impl GroundsTo for PlayGame {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- game state
            LexPrimitiva::Sequence, // sigma -- question progression
            LexPrimitiva::Quantity, // N -- scores
            LexPrimitiva::Sum,      // Sigma -- game mode
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// GamePlayer: T2-P (varsigma + N), dominant varsigma
///
/// A player in a game session with score and status.
impl GroundsTo for GamePlayer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- player state
            LexPrimitiva::Quantity, // N -- score
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// GameSession: T2-P (varsigma + sigma + N), dominant varsigma
///
/// A game session tracking rounds and timing.
impl GroundsTo for GameSession {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- session state
            LexPrimitiva::Sequence, // sigma -- round progression
            LexPrimitiva::Quantity, // N -- timing, counts
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// GameResults: T2-P (N + kappa + sigma), dominant N
///
/// Final game results with scores and rankings.
impl GroundsTo for GameResults {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- scores
            LexPrimitiva::Comparison, // kappa -- ranking comparison
            LexPrimitiva::Sequence,   // sigma -- result ordering
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// User: T2-C (varsigma + pi + partial + exists), dominant varsigma
///
/// User with auth state, credentials, and session data.
impl GroundsTo for User {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- user state
            LexPrimitiva::Persistence, // pi -- stored credentials
            LexPrimitiva::Boundary,    // partial -- auth boundaries
            LexPrimitiva::Existence,   // exists -- session existence
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// QuizError: T2-P (partial + Sigma), dominant partial
impl GroundsTo for QuizError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ===========================================================================
// Betting Module -- kappa (Comparison) dominant
// ===========================================================================

/// OrderSide: T1 (Sigma), pure sum
///
/// Buy or Sell side classification.
impl GroundsTo for OrderSide {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// ExchangeStatus: T1 (Sigma), pure sum
///
/// Order status: Filled, PartialFill, Rejected, etc.
impl GroundsTo for ExchangeStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// MarketOrder: T2-C (N + Sigma + kappa + partial), dominant N
///
/// An order with quantity, side, price, and limits.
impl GroundsTo for MarketOrder {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- quantity, price
            LexPrimitiva::Sum,        // Sigma -- side classification
            LexPrimitiva::Comparison, // kappa -- price comparison
            LexPrimitiva::Boundary,   // partial -- order limits
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// ExchangeResult: T2-P (N + Sigma + kappa), dominant N
///
/// Result of an exchange operation.
impl GroundsTo for ExchangeResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- fill quantity, price
            LexPrimitiva::Sum,        // Sigma -- status variant
            LexPrimitiva::Comparison, // kappa -- fill comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// SignalClassification: T2-P (kappa + Sigma + N), dominant kappa
///
/// Classification of a betting signal with confidence score.
impl GroundsTo for SignalClassification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- threshold comparison
            LexPrimitiva::Sum,        // Sigma -- signal type variant
            LexPrimitiva::Quantity,   // N -- confidence score
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ===========================================================================
// Crypto Module -- mu (Mapping) dominant
// ===========================================================================

/// CryptoEngine: T2-P (mu + partial), dominant mu
///
/// Cryptographic transformation engine.
impl GroundsTo for CryptoEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- plaintext -> ciphertext mapping
            LexPrimitiva::Boundary, // partial -- security boundary
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
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
    fn quiz_answer_is_t2p() {
        assert_eq!(QuizAnswer::tier(), Tier::T2Primitive);
        assert_eq!(QuizAnswer::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn quiz_is_t2c() {
        assert_eq!(Quiz::tier(), Tier::T2Composite);
        assert_eq!(Quiz::dominant_primitive(), Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn order_side_is_t1() {
        assert_eq!(OrderSide::tier(), Tier::T1Universal);
        assert!(OrderSide::is_pure_primitive());
    }

    #[test]
    fn market_order_is_t2c() {
        assert_eq!(MarketOrder::tier(), Tier::T2Composite);
    }

    #[test]
    fn signal_classification_comparison_dominant() {
        assert_eq!(
            SignalClassification::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            QuizAnswer::primitive_composition(),
            QuizConfig::primitive_composition(),
            StorageBackend::primitive_composition(),
            QuestionType::primitive_composition(),
            Quiz::primitive_composition(),
            QuizQuestion::primitive_composition(),
            PlayGame::primitive_composition(),
            GamePlayer::primitive_composition(),
            GameSession::primitive_composition(),
            GameResults::primitive_composition(),
            User::primitive_composition(),
            QuizError::primitive_composition(),
            OrderSide::primitive_composition(),
            ExchangeStatus::primitive_composition(),
            MarketOrder::primitive_composition(),
            ExchangeResult::primitive_composition(),
            SignalClassification::primitive_composition(),
            CryptoEngine::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
