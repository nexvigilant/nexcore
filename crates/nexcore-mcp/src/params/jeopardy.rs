//! Parameter types for Jeopardy MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

// ── Shared sub-structures ────────────────────────────────────────────────

/// A player description for game state construction.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyPlayerInput {
    /// Player name.
    pub name: String,
    /// Current score (can be negative).
    pub score: i64,
    /// Number of correct answers.
    pub correct: u32,
    /// Number of incorrect answers.
    pub incorrect: u32,
}

/// A board position (row, col).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyPositionInput {
    /// Row index (0 = lowest value, 4 = highest).
    pub row: usize,
    /// Column index (0–5).
    pub col: usize,
}

/// Full game state description for strategy tools.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyGameStateInput {
    /// Round: "jeopardy", "double_jeopardy", or "final_jeopardy".
    pub round: String,
    /// Daily Double positions on the board.
    pub daily_double_positions: Vec<JeopardyPositionInput>,
    /// Already-answered positions (will be marked as answered on the board).
    pub answered_positions: Vec<JeopardyPositionInput>,
    /// Players in the game (typically 3).
    pub players: Vec<JeopardyPlayerInput>,
    /// Index of the active player (0-based).
    pub active_player: usize,
}

// ── Simple tools ─────────────────────────────────────────────────────────

/// Get clue dollar values for a given round.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyClueValuesParams {
    /// Round: "jeopardy", "double_jeopardy", or "final_jeopardy".
    pub round: String,
}

/// List all Jeopardy categories with compound multipliers.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyCategoriesParams {}

// ── Board-only tools ─────────────────────────────────────────────────────

/// Score available clue selections by strategic value (Holzhauer strategy).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyScoreBoardParams {
    /// Round: "jeopardy" or "double_jeopardy".
    pub round: String,
    /// Daily Double positions on the board.
    pub daily_double_positions: Vec<JeopardyPositionInput>,
    /// Already-answered positions.
    pub answered_positions: Vec<JeopardyPositionInput>,
}

// ── Game state tools ─────────────────────────────────────────────────────

/// Determine whether to buzz in on a specific clue.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyShouldBuzzParams {
    /// Full game state.
    pub state: JeopardyGameStateInput,
    /// Position of the clue to evaluate.
    pub position: JeopardyPositionInput,
    /// Confidence in knowing the answer (0.0–1.0).
    pub confidence: f64,
}

/// Compute optimal Daily Double wager.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyDdWagerParams {
    /// Full game state (must have a Daily Double remaining).
    pub state: JeopardyGameStateInput,
    /// Confidence in the Daily Double category (0.0–1.0).
    pub confidence: f64,
}

/// Compute optimal Final Jeopardy wager.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyFinalWagerParams {
    /// Full game state (round should be final_jeopardy).
    pub state: JeopardyGameStateInput,
    /// Confidence in the Final Jeopardy category (0.0–1.0).
    pub confidence: f64,
}

/// Compute the strategic value of board control.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyBoardControlParams {
    /// Full game state.
    pub state: JeopardyGameStateInput,
}

// ── Compound velocity ────────────────────────────────────────────────────

/// A simplified game result for compound velocity computation.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyGameInput {
    /// Final score achieved.
    pub final_score: i64,
    /// Categories with correct answers: "signal_detection", "primitive_extraction",
    /// "cross_domain_transfer", "validation_phasing", "compound_growth", "pipeline_orchestration".
    pub categories_correct: Vec<String>,
    /// Overall accuracy (0.0–1.0).
    pub accuracy: f64,
    /// Number of correct answers.
    pub correct_count: u32,
    /// Total number of attempts (correct + incorrect).
    pub total_attempts: u32,
}

/// Compute compound velocity from game history.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JeopardyCompoundVelocityParams {
    /// Game history (ordered chronologically).
    pub games: Vec<JeopardyGameInput>,
}
