//! Jeopardy MCP tools — game theory strategy engine.
//!
//! Pure-function wrappers: buzz decisions, optimal wagers, board control
//! valuation, selection scoring (Holzhauer strategy), and compound velocity.

use nexcore_jeopardy::board::Board;
use nexcore_jeopardy::prelude::Confidence;
use nexcore_jeopardy::state::GameState;
use nexcore_jeopardy::types::{Category, CluePosition, ClueValue, Round};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::jeopardy::{
    JeopardyBoardControlParams, JeopardyCategoriesParams, JeopardyClueValuesParams,
    JeopardyCompoundVelocityParams, JeopardyDdWagerParams, JeopardyFinalWagerParams,
    JeopardyGameStateInput, JeopardyScoreBoardParams, JeopardyShouldBuzzParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_round(s: &str) -> Option<Round> {
    match s.to_lowercase().trim() {
        "jeopardy" => Some(Round::Jeopardy),
        "double_jeopardy" | "double jeopardy" => Some(Round::DoubleJeopardy),
        "final_jeopardy" | "final jeopardy" => Some(Round::FinalJeopardy),
        _ => None,
    }
}

fn parse_category(s: &str) -> Option<Category> {
    match s.to_lowercase().replace(' ', "_").trim() {
        "signal_detection" | "signaldetection" => Some(Category::SignalDetection),
        "primitive_extraction" | "primitiveextraction" => Some(Category::PrimitiveExtraction),
        "cross_domain_transfer" | "crossdomaintransfer" => Some(Category::CrossDomainTransfer),
        "validation_phasing" | "validationphasing" => Some(Category::ValidationPhasing),
        "compound_growth" | "compoundgrowth" => Some(Category::CompoundGrowth),
        "pipeline_orchestration" | "pipelineorchestration" => Some(Category::PipelineOrchestration),
        _ => None,
    }
}

/// Build a Board from round + daily double positions, then answer specified positions.
fn build_board(
    round: Round,
    dd_positions: &[CluePosition],
    answered: &[CluePosition],
) -> Result<Board, String> {
    let mut board = Board::new(round, dd_positions).map_err(|e| format!("{e}"))?;
    for pos in answered {
        board.answer(*pos).map_err(|e| format!("{e}"))?;
    }
    Ok(board)
}

/// Build a full GameState from the shared input structure.
fn build_game_state(input: &JeopardyGameStateInput) -> Result<GameState, String> {
    let round = parse_round(&input.round)
        .ok_or_else(|| "round must be 'jeopardy', 'double_jeopardy', or 'final_jeopardy'")?;

    let dd_pos: Vec<CluePosition> = input
        .daily_double_positions
        .iter()
        .map(|p| CluePosition::new(p.row, p.col))
        .collect();

    let answered: Vec<CluePosition> = input
        .answered_positions
        .iter()
        .map(|p| CluePosition::new(p.row, p.col))
        .collect();

    let board = build_board(round, &dd_pos, &answered)?;

    let player_names: Vec<&str> = input.players.iter().map(|p| p.name.as_str()).collect();
    let mut state = GameState::new(&player_names, board);

    // Apply player state overrides
    for (i, p) in input.players.iter().enumerate() {
        if let Some(player) = state.players.get_mut(i) {
            player.score = p.score;
            player.correct = p.correct;
            player.incorrect = p.incorrect;
        }
    }

    if input.active_player < state.players.len() {
        state.transfer_control(input.active_player);
    }

    Ok(state)
}

// ── Tools ────────────────────────────────────────────────────────────────

/// List clue dollar values for a given round.
pub fn jeopardy_clue_values(p: JeopardyClueValuesParams) -> Result<CallToolResult, McpError> {
    let round = match parse_round(&p.round) {
        Some(r) => r,
        None => {
            return err_result("round must be 'jeopardy', 'double_jeopardy', or 'final_jeopardy'");
        }
    };

    let values = ClueValue::for_round(round);
    let vals: Vec<u64> = values.iter().map(|v| v.0).collect();
    ok_json(json!({
        "round": p.round,
        "values": vals,
        "count": vals.len(),
    }))
}

/// List all Jeopardy categories with their compound multipliers.
pub fn jeopardy_categories(_p: JeopardyCategoriesParams) -> Result<CallToolResult, McpError> {
    let cats: Vec<serde_json::Value> = Category::all()
        .iter()
        .map(|c| {
            json!({
                "name": c.name(),
                "compound_multiplier": c.compound_multiplier(),
            })
        })
        .collect();

    ok_json(json!({ "categories": cats }))
}

/// Score available board positions by strategic value (Holzhauer strategy).
pub fn jeopardy_score_board(p: JeopardyScoreBoardParams) -> Result<CallToolResult, McpError> {
    let round = match parse_round(&p.round) {
        Some(r) => r,
        None => return err_result("round must be 'jeopardy' or 'double_jeopardy'"),
    };

    let dd_pos: Vec<CluePosition> = p
        .daily_double_positions
        .iter()
        .map(|pos| CluePosition::new(pos.row, pos.col))
        .collect();
    let answered: Vec<CluePosition> = p
        .answered_positions
        .iter()
        .map(|pos| CluePosition::new(pos.row, pos.col))
        .collect();

    let board = match build_board(round, &dd_pos, &answered) {
        Ok(b) => b,
        Err(e) => return err_result(&e),
    };

    let scores = nexcore_jeopardy::strategy::score_selections(&board);
    ok_json(serde_json::to_value(&scores).unwrap_or_default())
}

/// Determine whether to buzz in on a specific clue.
pub fn jeopardy_should_buzz(p: JeopardyShouldBuzzParams) -> Result<CallToolResult, McpError> {
    let state = match build_game_state(&p.state) {
        Ok(s) => s,
        Err(e) => return err_result(&e),
    };

    let pos = CluePosition::new(p.position.row, p.position.col);
    let clue = match state.board.get(pos).and_then(|c| c.clue()) {
        Some(c) => c.clone(),
        None => return err_result("no available clue at the specified position"),
    };

    let confidence = Confidence::new(p.confidence);
    let decision = nexcore_jeopardy::strategy::should_buzz(&clue, confidence, &state);
    ok_json(serde_json::to_value(&decision).unwrap_or_default())
}

/// Compute optimal Daily Double wager.
pub fn jeopardy_optimal_dd_wager(p: JeopardyDdWagerParams) -> Result<CallToolResult, McpError> {
    let state = match build_game_state(&p.state) {
        Ok(s) => s,
        Err(e) => return err_result(&e),
    };

    let confidence = Confidence::new(p.confidence);
    match nexcore_jeopardy::strategy::optimal_daily_double_wager(&state, confidence) {
        Ok(wager) => ok_json(serde_json::to_value(&wager).unwrap_or_default()),
        Err(e) => err_result(&format!("{e}")),
    }
}

/// Compute optimal Final Jeopardy wager.
pub fn jeopardy_optimal_final_wager(
    p: JeopardyFinalWagerParams,
) -> Result<CallToolResult, McpError> {
    let state = match build_game_state(&p.state) {
        Ok(s) => s,
        Err(e) => return err_result(&e),
    };

    let confidence = Confidence::new(p.confidence);
    match nexcore_jeopardy::strategy::optimal_final_wager(&state, confidence) {
        Ok(wager) => ok_json(serde_json::to_value(&wager).unwrap_or_default()),
        Err(e) => err_result(&format!("{e}")),
    }
}

/// Compute the strategic value of board control.
pub fn jeopardy_board_control_value(
    p: JeopardyBoardControlParams,
) -> Result<CallToolResult, McpError> {
    let state = match build_game_state(&p.state) {
        Ok(s) => s,
        Err(e) => return err_result(&e),
    };

    let value = nexcore_jeopardy::strategy::board_control_value(&state.board, &state);
    ok_json(json!({
        "board_control_value": value,
        "active_score": state.active_score(),
        "leader_score": state.leader_score(),
        "remaining_clues": state.board.remaining_count(),
        "remaining_value": state.board.remaining_value(),
    }))
}

/// Compute compound velocity from game history.
pub fn jeopardy_compound_velocity(
    p: JeopardyCompoundVelocityParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_jeopardy::compound::GameResult;
    use nexcore_jeopardy::state::AttemptRecord;

    let mut history = Vec::with_capacity(p.games.len());

    for game in &p.games {
        let mut categories_correct = Vec::with_capacity(game.categories_correct.len());
        for cat_str in &game.categories_correct {
            match parse_category(cat_str) {
                Some(c) => categories_correct.push(c),
                None => {
                    return err_result(&format!(
                        "invalid category '{}': must be signal_detection, primitive_extraction, \
                         cross_domain_transfer, validation_phasing, compound_growth, or pipeline_orchestration",
                        cat_str
                    ));
                }
            }
        }

        // Construct minimal AttemptRecord entries to satisfy attempts.len()
        let attempts: Vec<AttemptRecord> = (0..game.total_attempts)
            .map(|_| AttemptRecord {
                position: CluePosition::new(0, 0),
                category: Category::SignalDetection,
                value: ClueValue(0),
                correct: false,
                confidence: 0.0,
                was_daily_double: false,
                wager: None,
            })
            .collect();

        history.push(GameResult {
            final_score: game.final_score,
            categories_correct,
            accuracy: game.accuracy,
            correct_count: game.correct_count,
            attempts,
        });
    }

    let metrics = nexcore_jeopardy::compound::compound_velocity(&history);
    ok_json(serde_json::to_value(&metrics).unwrap_or_default())
}
