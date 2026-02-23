//! Error types for the Jeopardy optimizer.

use nexcore_error::Error;

/// Errors that can occur in Jeopardy optimization.
#[derive(Debug, Error)]
pub enum JeopardyError {
    /// Attempted to select a clue that has already been answered.
    #[error("clue at row {row}, category {category} already answered")]
    ClueAlreadyAnswered {
        /// Row index (0-based).
        row: usize,
        /// Category index (0-based).
        category: usize,
    },

    /// Wager exceeds the allowed maximum for the current game state.
    #[error("wager {attempted} exceeds maximum allowed {maximum}")]
    WagerExceedsMaximum {
        /// The attempted wager amount.
        attempted: u64,
        /// The maximum allowed wager.
        maximum: u64,
    },

    /// Confidence value is out of the valid [0.0, 1.0] range.
    ///
    /// Deprecated: canonical `Confidence::new()` now clamps instead of rejecting.
    /// This variant is retained for backward compatibility.
    #[error("confidence {0} is out of range [0.0, 1.0]")]
    InvalidConfidence(f64),

    /// Board dimensions are invalid (zero rows or zero categories).
    #[error("invalid board dimensions: {rows} rows x {cols} categories")]
    InvalidBoardDimensions {
        /// Number of rows.
        rows: usize,
        /// Number of columns (categories).
        cols: usize,
    },

    /// Cannot advance to the next round from the current state.
    #[error("cannot advance from {current_round:?}: {reason}")]
    CannotAdvance {
        /// The current round.
        current_round: String,
        /// Reason advancement is blocked.
        reason: String,
    },

    /// No clues remaining on the board.
    #[error("no clues remaining on board")]
    EmptyBoard,
}

/// Convenience Result alias.
pub type Result<T> = std::result::Result<T, JeopardyError>;
