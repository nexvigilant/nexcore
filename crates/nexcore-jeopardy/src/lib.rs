//! # NexVigilant Core — jeopardy
//!
//! Jeopardy!-inspired algorithm R&D investment optimizer using game theory
//! primitives transferred from the TV game show domain.
//!
//! ## Primitive Coverage
//!
//! | Primitive | Module | Manifestation |
//! |-----------|--------|---------------|
//! | sigma (Sequence) | board, strategy | Board traversal order, round progression |
//! | kappa (Compare) | strategy | Risk/reward evaluation, threshold comparison |
//! | partial (Branch) | strategy | Buzz/don't-buzz, wager/don't-wager decisions |
//! | Sigma (Match) | types, strategy | Category routing, round dispatch |
//! | rho (State) | state, board | Score tracking, board state, game phase |
//! | N (Quantity) | types | Clue values, wager amounts |
//!
//! ## Tier: T2-C (composed from sigma + kappa + partial + rho)
//!
//! ## Key Algorithms
//!
//! 1. **Holzhauer Strategy** — Bottom-up board traversal optimizing for
//!    Daily Double discovery and T2-P category prioritization.
//! 2. **Wagering Optimizer** — Game-position-aware investment allocation
//!    for Daily Doubles and Final Jeopardy.
//! 3. **Buzz Decision** — Signal threshold gate mapping confidence to
//!    buzz/pass decisions with value-weighted risk assessment.
//! 4. **Board Control Value** — Expected value of making optimal selections
//!    from the current board state.
//! 5. **Compound Velocity** — Cross-game primitive accumulation following
//!    V(t) = B * eta * r.
//!
//! ## Example
//!
//! ```
//! use nexcore_jeopardy::prelude::*;
//!
//! // Create a board for the first round
//! let board = Board::new(Round::Jeopardy, &[]).unwrap_or_else(|_| {
//!     panic!("board creation failed")  // doc example only
//! });
//!
//! // Get optimal selection order (Holzhauer strategy)
//! let order = optimal_selection_order(&board);
//! assert!(!order.is_empty());
//!
//! // Clone the clue data before moving the board into GameState
//! let clue = board.get(order[0]).and_then(|c| c.clue()).cloned().unwrap();
//! let state = GameState::new(&["Alice", "Bob", "Carol"], board);
//! let confidence = Confidence::new(0.8).unwrap();
//! let decision = should_buzz(&clue, confidence, &state);
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(missing_docs)]

pub mod board;
pub mod compound;
pub mod error;
pub mod grounding;
pub mod state;
pub mod strategy;
pub mod types;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::board::{Board, Cell};
    pub use crate::compound::{CompoundMetrics, GameResult, compound_velocity};
    pub use crate::error::{JeopardyError, Result};
    pub use crate::state::{AttemptRecord, GameState, Player};
    pub use crate::strategy::{
        BuzzDecision, SelectionScore, Wager, board_control_value, can_advance,
        optimal_daily_double_wager, optimal_final_wager, optimal_selection_order, score_selections,
        should_buzz,
    };
    pub use crate::types::{Category, Clue, CluePosition, ClueValue, Confidence, Round};
}

#[cfg(test)]
mod tests;
