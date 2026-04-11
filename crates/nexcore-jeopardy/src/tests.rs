//! Comprehensive tests for nexcore-jeopardy.
//!
//! Answer-First TDD: tests define expected outcomes, then validate implementation.

use crate::board::{Board, Cell};
use crate::compound::{GameResult, compound_velocity};
use crate::state::{AttemptRecord, GameState, Player};
use crate::strategy::{
    BuzzDecision, board_control_value, can_advance, optimal_daily_double_wager,
    optimal_final_wager, optimal_selection_order, score_selections, should_buzz,
};
use crate::types::{Category, Clue, CluePosition, ClueValue, Confidence, Round};

// =========================================================================
// Types Tests
// =========================================================================

mod type_tests {
    use super::*;

    #[test]
    fn confidence_valid_range() {
        let c = Confidence::new(0.5);
        assert!((c.value() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_zero_is_valid() {
        assert!((Confidence::new(0.0).value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_one_is_valid() {
        assert!((Confidence::new(1.0).value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_negative_clamps_to_zero() {
        // Canonical Confidence uses clamping, not validation.
        assert!((Confidence::new(-0.1).value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_over_one_clamps_to_one() {
        // Canonical Confidence uses clamping, not validation.
        assert!((Confidence::new(1.01).value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn category_all_returns_six() {
        assert_eq!(Category::all().len(), 6);
    }

    #[test]
    fn t2p_categories_compound_faster() {
        // T2-P categories should compound at 1.29x (29% faster)
        let pe = Category::PrimitiveExtraction.compound_multiplier();
        let sd = Category::SignalDetection.compound_multiplier();
        let cd = Category::CrossDomainTransfer.compound_multiplier();

        assert!((pe - 1.29).abs() < f64::EPSILON);
        assert!((sd - 1.29).abs() < f64::EPSILON);
        assert!((cd - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn round_progression() {
        assert_eq!(Round::Jeopardy.next(), Some(Round::DoubleJeopardy));
        assert_eq!(Round::DoubleJeopardy.next(), Some(Round::FinalJeopardy));
        assert_eq!(Round::FinalJeopardy.next(), None);
    }

    #[test]
    fn clue_values_for_jeopardy_round() {
        let values = ClueValue::for_round(Round::Jeopardy);
        assert_eq!(values[0].0, 200);
        assert_eq!(values[4].0, 1000);
    }

    #[test]
    fn clue_values_for_double_jeopardy() {
        let values = ClueValue::for_round(Round::DoubleJeopardy);
        assert_eq!(values[0].0, 400);
        assert_eq!(values[4].0, 2000);
    }
}

// =========================================================================
// Board Tests
// =========================================================================

mod board_tests {
    use super::*;

    #[test]
    fn new_board_has_30_clues() {
        // 5 rows * 6 categories = 30 clues
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            assert_eq!(b.remaining_count(), 30);
            assert!(!b.is_empty());
        }
    }

    #[test]
    fn board_dimensions_correct() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            assert_eq!(b.num_rows(), 5);
            assert_eq!(b.num_cols(), 6);
        }
    }

    #[test]
    fn answering_clue_reduces_count() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(mut b) = board {
            let result = b.answer(CluePosition::new(0, 0));
            assert!(result.is_ok());
            assert_eq!(b.remaining_count(), 29);
        }
    }

    #[test]
    fn answering_same_clue_twice_is_error() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(mut b) = board {
            let first = b.answer(CluePosition::new(0, 0));
            assert!(first.is_ok());
            let second = b.answer(CluePosition::new(0, 0));
            assert!(second.is_err());
        }
    }

    #[test]
    fn daily_double_placement() {
        let dd_pos = CluePosition::new(3, 2);
        let board = Board::new(Round::Jeopardy, &[dd_pos]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            let cell = b.get(dd_pos);
            assert!(cell.is_some());
            if let Some(Cell::Available(clue)) = cell {
                assert!(clue.is_daily_double);
            }
        }
    }

    #[test]
    fn remaining_daily_doubles_tracks_correctly() {
        let dd_pos = CluePosition::new(4, 1);
        let board = Board::new(Round::DoubleJeopardy, &[dd_pos]);
        assert!(board.is_ok());
        if let Ok(mut b) = board {
            let dds = b.remaining_daily_doubles();
            assert_eq!(dds.len(), 1);

            // Answer the daily double
            let _ = b.answer(dd_pos);
            let dds_after = b.remaining_daily_doubles();
            assert_eq!(dds_after.len(), 0);
        }
    }

    #[test]
    fn remaining_value_sums_correctly() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            // Sum: (200+400+600+800+1000) * 6 = 3000 * 6 = 18000
            assert_eq!(b.remaining_value(), 18000);
        }
    }

    #[test]
    fn difficulty_scales_with_row() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            let low = b.get(CluePosition::new(0, 0));
            let high = b.get(CluePosition::new(4, 0));
            if let (Some(Cell::Available(lo)), Some(Cell::Available(hi))) = (low, high) {
                assert!(lo.difficulty() < hi.difficulty());
                assert!((lo.difficulty() - 0.2).abs() < f64::EPSILON);
                assert!((hi.difficulty() - 1.0).abs() < f64::EPSILON);
            }
        }
    }
}

// =========================================================================
// State Tests
// =========================================================================

mod state_tests {
    use super::*;

    fn make_state() -> GameState {
        let board = Board::new(Round::Jeopardy, &[]);
        match board {
            Ok(b) => GameState::new(&["Alice", "Bob", "Carol"], b),
            Err(_) => {
                // Should never happen, but we cannot unwrap
                let fallback = Board::new(Round::Jeopardy, &[]).ok();
                assert!(fallback.is_some());
                GameState::new(
                    &["Alice", "Bob", "Carol"],
                    fallback.unwrap_or_else(|| {
                        // Double safety net
                        Board::new(Round::Jeopardy, &[]).ok().unwrap_or_else(|| {
                            // This is only in tests; the board constructor is infallible
                            // for valid dimensions, so this path is unreachable
                            unreachable!()
                        })
                    }),
                )
            }
        }
    }

    #[test]
    fn new_state_has_three_players() {
        let state = make_state();
        assert_eq!(state.players.len(), 3);
    }

    #[test]
    fn initial_scores_are_zero() {
        let state = make_state();
        for player in &state.players {
            assert_eq!(player.score(), 0);
        }
    }

    #[test]
    fn first_player_has_control() {
        let state = make_state();
        assert_eq!(state.active_player_index(), 0);
        if let Some(p) = state.active_player() {
            assert!(p.has_control());
        }
    }

    #[test]
    fn record_correct_increases_score() {
        let mut state = make_state();
        let clue = Clue::new(Category::SignalDetection, ClueValue(400), 0.4, false);
        let pos = CluePosition::new(1, 0);
        state.record_correct(400, Confidence::new(0.8), &clue, pos, None);
        assert_eq!(state.active_score(), 400);
    }

    #[test]
    fn record_incorrect_decreases_score() {
        let mut state = make_state();
        let clue = Clue::new(Category::SignalDetection, ClueValue(600), 0.6, false);
        let pos = CluePosition::new(2, 0);
        state.record_incorrect(600, Confidence::new(0.4), &clue, pos, None);
        assert_eq!(state.active_score(), -600);
    }

    #[test]
    fn daily_double_wager_affects_score() {
        let mut state = make_state();
        // Give the player some score first
        let clue1 = Clue::new(Category::SignalDetection, ClueValue(1000), 1.0, false);
        state.record_correct(
            1000,
            Confidence::new(0.9),
            &clue1,
            CluePosition::new(4, 0),
            None,
        );

        // Now a Daily Double with a wager of 2000
        let dd_clue = Clue::new(Category::PrimitiveExtraction, ClueValue(800), 0.8, true);
        state.record_correct(
            800,
            Confidence::new(0.9),
            &dd_clue,
            CluePosition::new(3, 1),
            Some(2000),
        );
        assert_eq!(state.active_score(), 3000); // 1000 + 2000 wager
    }

    #[test]
    fn transfer_control_updates_active_player() {
        let mut state = make_state();
        state.transfer_control(2);
        assert_eq!(state.active_player_index(), 2);
        if let Some(p) = state.active_player() {
            assert!(p.has_control());
            assert_eq!(p.name(), "Carol");
        }
    }

    #[test]
    fn player_accuracy() {
        let mut player = Player::new("Test");
        assert!((player.accuracy() - 0.0).abs() < f64::EPSILON);

        player.set_correct(3);
        player.set_incorrect(1);
        assert!((player.accuracy() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn gap_to_leader_when_trailing() {
        let mut state = make_state();
        // Give Bob (player 1) 5000 points
        state.transfer_control(1);
        let clue = Clue::new(Category::CompoundGrowth, ClueValue(1000), 1.0, false);
        state.record_correct(
            1000,
            Confidence::new(1.0),
            &clue,
            CluePosition::new(4, 4),
            Some(5000),
        );

        // Switch back to Alice (player 0)
        state.transfer_control(0);
        // Alice has 0, Bob has 5000 → gap = -5000
        assert_eq!(state.gap_to_leader(), -5000);
        assert!(!state.is_leading());
    }

    #[test]
    fn max_daily_double_wager_minimum_board_max() {
        let state = make_state();
        // Score is 0, so max wager should be the board max (1000 for Jeopardy round)
        assert_eq!(state.max_daily_double_wager(), 1000);
    }
}

// =========================================================================
// Strategy Tests — Holzhauer Board Traversal
// =========================================================================

mod holzhauer_tests {
    use super::*;

    #[test]
    fn optimal_order_prefers_high_value() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            let order = optimal_selection_order(&b);
            assert_eq!(order.len(), 30);

            // First selection should be from the highest row
            let first = order[0];
            assert_eq!(first.row, 4); // Row 4 = $1000 clues
        }
    }

    #[test]
    fn optimal_order_prefers_t2p_categories() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            let order = optimal_selection_order(&b);

            // Among the first row-4 selections, T2-P categories should come first
            let row4_selections: Vec<&CluePosition> = order.iter().filter(|p| p.row == 4).collect();

            // First among row 4 should be T2-P (SignalDetection or PrimitiveExtraction)
            if let Some(first_row4) = row4_selections.first() {
                let cell = b.get(**first_row4);
                if let Some(Cell::Available(clue)) = cell {
                    let is_t2p = matches!(
                        clue.category,
                        Category::SignalDetection | Category::PrimitiveExtraction
                    );
                    assert!(is_t2p, "First row-4 selection should be T2-P category");
                }
            }
        }
    }

    #[test]
    fn daily_double_gets_highest_priority() {
        // Place a DD at a low-value position; it should still rank high
        let dd_pos = CluePosition::new(0, 3); // $200 slot
        let board = Board::new(Round::Jeopardy, &[dd_pos]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            let order = optimal_selection_order(&b);
            // DD at $200 gets +5000 bonus, so it should be first
            assert_eq!(order[0], dd_pos);
        }
    }

    #[test]
    fn empty_board_returns_empty_order() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(mut b) = board {
            // Answer all clues
            for row in 0..5 {
                for col in 0..6 {
                    let _ = b.answer(CluePosition::new(row, col));
                }
            }
            let order = optimal_selection_order(&b);
            assert!(order.is_empty());
        }
    }

    #[test]
    fn score_selections_provides_reasons() {
        let board = Board::new(Round::Jeopardy, &[]);
        assert!(board.is_ok());
        if let Ok(b) = board {
            let scores = score_selections(&b);
            assert_eq!(scores.len(), 30);
            for s in &scores {
                assert!(!s.reason.is_empty());
            }
        }
    }
}

// =========================================================================
// Strategy Tests — Wagering
// =========================================================================

mod wager_tests {
    use super::*;

    fn leading_state() -> GameState {
        let board_result = Board::new(Round::DoubleJeopardy, &[CluePosition::new(4, 0)]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Leader", "Trailer", "Mid"], board);

        // Leader: 15000, Trailer: 5000, Mid: 8000
        state.players[0].set_score(15000);
        state.players[1].set_score(5000);
        state.players[2].set_score(8000);
        state.set_active_player_index(0);
        state.players[0].set_has_control(true);
        state
    }

    fn trailing_state() -> GameState {
        let board_result = Board::new(Round::DoubleJeopardy, &[CluePosition::new(4, 0)]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Trailer", "Leader", "Mid"], board);

        // Trailer: 3000, Leader: 12000, Mid: 7000
        state.players[0].set_score(3000);
        state.players[1].set_score(12000);
        state.players[2].set_score(7000);
        state.set_active_player_index(0);
        state.players[0].set_has_control(true);
        state
    }

    #[test]
    fn leading_player_wagers_for_lock() {
        let state = leading_state();
        let conf = Confidence::new(0.8);
        let wager = optimal_daily_double_wager(&state, conf);
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            // Leader (15000) vs 2nd (8000): lock target = 8000*2+1-15000 = 1001
            // With confidence 0.8, should wager at least 1001
            assert!(
                w.amount >= 1001,
                "Leading player should wager for lock: got {}",
                w.amount
            );
        }
    }

    #[test]
    fn trailing_player_wagers_aggressively() {
        let state = trailing_state();
        let conf = Confidence::new(0.7);
        let wager = optimal_daily_double_wager(&state, conf);
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            // Trailing by 9000, should wager at least gap+1 = 9001 if possible
            // Max wager = max(3000, 2000) = 3000
            // So wager should be capped at 3000
            assert!(
                w.amount <= 3000,
                "Wager should not exceed max: got {}",
                w.amount
            );
            // With 0.7 confidence and 0.5 threshold, should be aggressive
            assert!(w.amount > 0, "Trailing player should wager something");
        }
    }

    #[test]
    fn final_jeopardy_defensive_wager() {
        let board_result = Board::new(Round::DoubleJeopardy, &[]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Leader", "Second"], board);
        state.players[0].set_score(20000);
        state.players[1].set_score(12000);
        state.set_active_player_index(0);

        // Answer all clues to enable advancement
        for row in 0..5 {
            for col in 0..6 {
                let _ = state.board.answer(CluePosition::new(row, col));
            }
        }

        let conf = Confidence::new(0.6);
        let wager = optimal_final_wager(&state, conf);
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            // Defensive: (12000 * 2) - 20000 + 1 = 4001
            // This ensures if 2nd goes all-in and gets it right (24000),
            // leader with correct answer has 20000 + 4001 = 24001 > 24000
            assert_eq!(w.amount, 4001, "Defensive wager should be (2nd*2)-score+1");
        }
    }

    #[test]
    fn final_jeopardy_runaway_wagers_small() {
        let board_result = Board::new(Round::DoubleJeopardy, &[]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Runaway", "Distant"], board);
        state.players[0].set_score(30000);
        state.players[1].set_score(5000);
        state.set_active_player_index(0);

        for row in 0..5 {
            for col in 0..6 {
                let _ = state.board.answer(CluePosition::new(row, col));
            }
        }

        let conf = Confidence::new(0.5);
        let wager = optimal_final_wager(&state, conf);
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            // Runaway: (5000*2)-30000+1 = -19999, so defensive < 0
            // With low confidence (0.5), should wager 0
            assert_eq!(w.amount, 0, "Runaway with low confidence should wager 0");
        }
    }

    #[test]
    fn trailing_final_jeopardy_wagers_to_overtake() {
        let board_result = Board::new(Round::DoubleJeopardy, &[]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Trailer", "Leader"], board);
        state.players[0].set_score(10000);
        state.players[1].set_score(15000);
        state.set_active_player_index(0);

        for row in 0..5 {
            for col in 0..6 {
                let _ = state.board.answer(CluePosition::new(row, col));
            }
        }

        let conf = Confidence::new(0.8);
        let wager = optimal_final_wager(&state, conf);
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            // Trailing by 5000, need gap+1 = 5001
            // Capped at score (10000), so should wager 5001
            assert_eq!(w.amount, 5001, "Trailing player should wager gap+1");
        }
    }

    #[test]
    fn zero_score_final_wager_is_zero() {
        let board_result = Board::new(Round::DoubleJeopardy, &[]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Broke"], board);
        state.players[0].set_score(-500);
        state.set_active_player_index(0);

        let conf = Confidence::new(1.0);
        let wager = optimal_final_wager(&state, conf);
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            assert_eq!(w.amount, 0, "Negative score should wager 0 in Final");
        }
    }
}

// =========================================================================
// Strategy Tests — Buzz Decision
// =========================================================================

mod buzz_tests {
    use super::*;

    fn base_state() -> GameState {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        match board_result {
            Ok(b) => GameState::new(&["Player"], b),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn high_confidence_easy_clue_buzzes() {
        let state = base_state();
        let clue = Clue::new(Category::SignalDetection, ClueValue(200), 0.2, false);
        let conf = Confidence::new(0.9);
        let decision = should_buzz(&clue, conf, &state);
        assert_eq!(decision, BuzzDecision::Buzz);
    }

    #[test]
    fn low_confidence_hard_clue_passes() {
        let state = base_state();
        let clue = Clue::new(Category::CrossDomainTransfer, ClueValue(1000), 1.0, false);
        let conf = Confidence::new(0.3);
        let decision = should_buzz(&clue, conf, &state);
        assert_eq!(decision, BuzzDecision::Pass);
    }

    #[test]
    fn medium_confidence_medium_difficulty_threshold() {
        let state = base_state();
        let clue = Clue::new(Category::ValidationPhasing, ClueValue(600), 0.6, false);
        // Threshold: 0.50 + 0.6*0.15 + (600/1000)*0.10 - 0 + 0
        //          = 0.50 + 0.09 + 0.06 = 0.65
        let just_above = Confidence::new(0.66);
        assert_eq!(should_buzz(&clue, just_above, &state), BuzzDecision::Buzz);

        let just_below = Confidence::new(0.64);
        assert_eq!(should_buzz(&clue, just_below, &state), BuzzDecision::Pass);
    }

    #[test]
    fn trailing_player_buzzes_more_aggressively() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Trailer", "Leader"], board);
        state.players[0].set_score(0);
        state.players[1].set_score(5000);
        state.set_active_player_index(0);

        let clue = Clue::new(Category::CompoundGrowth, ClueValue(800), 0.8, false);

        // Normal threshold would be ~0.70, but urgency bonus lowers it
        let conf = Confidence::new(0.65);
        let decision = should_buzz(&clue, conf, &state);
        // Trailing by 5000: urgency = min((5000/1000)*0.15, 0.15) = 0.15
        // Threshold: 0.50 + 0.12 + 0.08 + 0 - 0.15 = 0.55
        // 0.65 >= 0.55 → Buzz
        assert_eq!(decision, BuzzDecision::Buzz);
    }

    #[test]
    fn negative_score_raises_threshold() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        let board = match board_result {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };
        let mut state = GameState::new(&["Negative"], board);
        state.players[0].set_score(-200);
        state.set_active_player_index(0);

        let clue = Clue::new(Category::PipelineOrchestration, ClueValue(200), 0.2, false);

        // With single player at -200, gap_to_leader = -200 - 0 = -200
        // urgency = min((200/1000)*0.15, 0.15) = 0.03
        // Threshold: 0.50 + 0.03 + 0.02 + 0.05 - 0.03 = 0.57
        // Use 0.56 (below threshold) to verify Pass
        let conf_below = Confidence::new(0.56);
        assert_eq!(should_buzz(&clue, conf_below, &state), BuzzDecision::Pass);

        // And 0.58 (above threshold) should Buzz
        let conf_above = Confidence::new(0.58);
        assert_eq!(should_buzz(&clue, conf_above, &state), BuzzDecision::Buzz);
    }
}

// =========================================================================
// Board Control Value Tests
// =========================================================================

mod control_tests {
    use super::*;

    #[test]
    fn full_board_has_positive_control_value() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(b) = board_result {
            let state = GameState::new(&["Player"], b.clone());
            let value = board_control_value(&b, &state);
            assert!(value > 0.0, "Full board should have positive control value");
        }
    }

    #[test]
    fn empty_board_has_zero_control_value() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(mut b) = board_result {
            for row in 0..5 {
                for col in 0..6 {
                    let _ = b.answer(CluePosition::new(row, col));
                }
            }
            let state = GameState::new(&["Player"], b.clone());
            let value = board_control_value(&b, &state);
            assert!((value - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn higher_accuracy_means_higher_control_value() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(b) = board_result {
            let mut state_high = GameState::new(&["Accurate"], b.clone());
            state_high.players[0].set_correct(9);
            state_high.players[0].set_incorrect(1);

            let mut state_low = GameState::new(&["Inaccurate"], b.clone());
            state_low.players[0].set_correct(3);
            state_low.players[0].set_incorrect(7);

            let value_high = board_control_value(&b, &state_high);
            let value_low = board_control_value(&b, &state_low);

            assert!(
                value_high > value_low,
                "Higher accuracy ({}) should produce higher control value ({}) than low accuracy ({}) value ({})",
                state_high.players[0].accuracy(),
                value_high,
                state_low.players[0].accuracy(),
                value_low,
            );
        }
    }
}

// =========================================================================
// Round Transition Tests
// =========================================================================

mod transition_tests {
    use super::*;

    #[test]
    fn cannot_advance_with_clues_remaining() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(b) = board_result {
            let state = GameState::new(&["Player"], b);
            assert!(!can_advance(&state));
        }
    }

    #[test]
    fn can_advance_from_jeopardy_with_zero_score() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(mut b) = board_result {
            for row in 0..5 {
                for col in 0..6 {
                    let _ = b.answer(CluePosition::new(row, col));
                }
            }
            let state = GameState::new(&["Player"], b);
            // Zero score >= 0, so can advance from Jeopardy
            assert!(can_advance(&state));
        }
    }

    #[test]
    fn cannot_advance_from_jeopardy_with_negative_score() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(mut b) = board_result {
            for row in 0..5 {
                for col in 0..6 {
                    let _ = b.answer(CluePosition::new(row, col));
                }
            }
            let mut state = GameState::new(&["Player"], b);
            state.players[0].set_score(-100);
            assert!(!can_advance(&state));
        }
    }

    #[test]
    fn cannot_advance_from_double_jeopardy_with_zero_score() {
        let board_result = Board::new(Round::DoubleJeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(mut b) = board_result {
            for row in 0..5 {
                for col in 0..6 {
                    let _ = b.answer(CluePosition::new(row, col));
                }
            }
            let state = GameState::new(&["Player"], b);
            // Need positive score (> 0) for Final Jeopardy
            assert!(!can_advance(&state));
        }
    }

    #[test]
    fn can_advance_from_double_jeopardy_with_positive_score() {
        let board_result = Board::new(Round::DoubleJeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(mut b) = board_result {
            for row in 0..5 {
                for col in 0..6 {
                    let _ = b.answer(CluePosition::new(row, col));
                }
            }
            let mut state = GameState::new(&["Player"], b);
            state.players[0].set_score(100);
            assert!(can_advance(&state));
        }
    }

    #[test]
    fn cannot_advance_from_final_jeopardy() {
        let board_result = Board::new(Round::FinalJeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(mut b) = board_result {
            for row in 0..5 {
                for col in 0..6 {
                    let _ = b.answer(CluePosition::new(row, col));
                }
            }
            let mut state = GameState::new(&["Player"], b);
            // round is already FinalJeopardy from board.round()
            state.players[0].set_score(50000);
            assert!(!can_advance(&state));
        }
    }
}

// =========================================================================
// Compound Growth Tests
// =========================================================================

mod compound_tests {
    use super::*;

    #[test]
    fn empty_history_returns_zero_velocity() {
        let metrics = compound_velocity(&[]);
        assert_eq!(metrics.games_played, 0);
        assert!((metrics.velocity - 0.0).abs() < f64::EPSILON);
        assert!((metrics.basis - 0.0).abs() < f64::EPSILON);
        assert!((metrics.transfer_cost - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn single_game_produces_initial_velocity() {
        let game = GameResult {
            final_score: 5000,
            categories_correct: vec![Category::SignalDetection, Category::PrimitiveExtraction],
            accuracy: 0.8,
            correct_count: 4,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(0.9),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(1, 1),
                    category: Category::PrimitiveExtraction,
                    value: ClueValue(400),
                    correct: true,
                    confidence: Confidence::new(0.8),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(2, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(600),
                    correct: true,
                    confidence: Confidence::new(0.7),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(3, 1),
                    category: Category::PrimitiveExtraction,
                    value: ClueValue(800),
                    correct: true,
                    confidence: Confidence::new(0.85),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };

        let metrics = compound_velocity(&[game]);
        assert_eq!(metrics.games_played, 1);
        assert!(metrics.basis > 0.0, "Basis should be positive");
        assert!(
            metrics.efficiency.value() > 0.0,
            "Efficiency should be positive"
        );
        assert!(metrics.reuse.value() > 0.0, "Reuse should be positive");
        assert!(
            metrics.velocity > 0.0,
            "Velocity should be positive: V = B * eta * r"
        );
    }

    #[test]
    fn more_categories_increase_reuse() {
        let game_narrow = GameResult {
            final_score: 2000,
            categories_correct: vec![Category::SignalDetection],
            accuracy: 1.0,
            correct_count: 2,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(1, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(400),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };

        let game_broad = GameResult {
            final_score: 2000,
            categories_correct: vec![
                Category::SignalDetection,
                Category::PrimitiveExtraction,
                Category::CompoundGrowth,
            ],
            accuracy: 1.0,
            correct_count: 3,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(0, 1),
                    category: Category::PrimitiveExtraction,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(0, 4),
                    category: Category::CompoundGrowth,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };

        let narrow = compound_velocity(&[game_narrow]);
        let broad = compound_velocity(&[game_broad]);

        assert!(
            broad.reuse > narrow.reuse,
            "Broader category coverage ({}) should have higher reuse than narrow ({})",
            broad.reuse,
            narrow.reuse,
        );
    }

    #[test]
    fn t2p_categories_compound_faster_in_basis() {
        // Two games: one focused on T2-P categories, one on T2-C
        let game_t2p = GameResult {
            final_score: 1000,
            categories_correct: vec![Category::PrimitiveExtraction, Category::PrimitiveExtraction],
            accuracy: 1.0,
            correct_count: 2,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 1),
                    category: Category::PrimitiveExtraction,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(1, 1),
                    category: Category::PrimitiveExtraction,
                    value: ClueValue(400),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };

        let game_t2c = GameResult {
            final_score: 1000,
            categories_correct: vec![Category::CompoundGrowth, Category::CompoundGrowth],
            accuracy: 1.0,
            correct_count: 2,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 4),
                    category: Category::CompoundGrowth,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(1, 4),
                    category: Category::CompoundGrowth,
                    value: ClueValue(400),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };

        let t2p_metrics = compound_velocity(&[game_t2p]);
        let t2c_metrics = compound_velocity(&[game_t2c]);

        assert!(
            t2p_metrics.basis > t2c_metrics.basis,
            "T2-P basis ({}) should exceed T2-C basis ({})",
            t2p_metrics.basis,
            t2c_metrics.basis,
        );
    }

    #[test]
    fn transfer_cost_decreases_with_domains() {
        // 0 domains covered: cost = 1.0
        let m0 = compound_velocity(&[]);
        assert!((m0.transfer_cost - 1.0).abs() < f64::EPSILON);

        // 2 domains covered
        let game = GameResult {
            final_score: 1000,
            categories_correct: vec![Category::SignalDetection, Category::CompoundGrowth],
            accuracy: 1.0,
            correct_count: 2,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(0, 4),
                    category: Category::CompoundGrowth,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(1.0),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };
        let m2 = compound_velocity(&[game]);
        // T(1) = 1, cost = 1.0 - 1*0.1 = 0.9
        assert!(
            m2.transfer_cost < m0.transfer_cost,
            "Transfer cost should decrease with more domains"
        );
        assert!((m2.transfer_cost - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn velocity_formula_holds() {
        let game = GameResult {
            final_score: 3000,
            categories_correct: vec![
                Category::SignalDetection,
                Category::PrimitiveExtraction,
                Category::CrossDomainTransfer,
            ],
            accuracy: 0.75,
            correct_count: 3,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(0.9),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(0, 1),
                    category: Category::PrimitiveExtraction,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(0.8),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(0, 2),
                    category: Category::CrossDomainTransfer,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(0.7),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(1, 3),
                    category: Category::ValidationPhasing,
                    value: ClueValue(400),
                    correct: false,
                    confidence: Confidence::new(0.5),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };

        let metrics = compound_velocity(&[game]);
        let expected_velocity = metrics.basis * metrics.efficiency.value() * metrics.reuse.value();
        assert!(
            (metrics.velocity - expected_velocity).abs() < 1e-10,
            "V(t) = B*eta*r: {} != {} * {} * {}",
            metrics.velocity,
            metrics.basis,
            metrics.efficiency.value(),
            metrics.reuse.value(),
        );
    }
}

// =========================================================================
// Serialization Round-Trip Tests
// =========================================================================

mod serde_tests {
    use super::*;

    #[test]
    fn category_round_trips_through_json() {
        let cat = Category::PrimitiveExtraction;
        let json = serde_json::to_string(&cat);
        assert!(json.is_ok());
        if let Ok(s) = json {
            let back: std::result::Result<Category, _> = serde_json::from_str(&s);
            assert!(back.is_ok());
            if let Ok(c) = back {
                assert_eq!(c, cat);
            }
        }
    }

    #[test]
    fn clue_round_trips_through_json() {
        let clue = Clue::new(Category::CompoundGrowth, ClueValue(800), 0.8, true);
        let json = serde_json::to_string(&clue);
        assert!(json.is_ok());
        if let Ok(s) = json {
            let back: std::result::Result<Clue, _> = serde_json::from_str(&s);
            assert!(back.is_ok());
        }
    }

    #[test]
    fn wager_round_trips_through_json() {
        let wager = crate::strategy::Wager {
            amount: 5000,
            confidence: Confidence::new(0.85),
            category: Category::SignalDetection,
        };
        let json = serde_json::to_string(&wager);
        assert!(json.is_ok());
        if let Ok(s) = json {
            let back: std::result::Result<crate::strategy::Wager, _> = serde_json::from_str(&s);
            assert!(back.is_ok());
        }
    }

    #[test]
    fn game_state_round_trips_through_json() {
        let board_result = Board::new(Round::Jeopardy, &[]);
        assert!(board_result.is_ok());
        if let Ok(b) = board_result {
            let state = GameState::new(&["Test"], b);
            let json = serde_json::to_string(&state);
            assert!(json.is_ok());
            if let Ok(s) = json {
                let back: std::result::Result<GameState, _> = serde_json::from_str(&s);
                assert!(back.is_ok());
            }
        }
    }
}

// =========================================================================
// Confidence Integration Tests (canonical Confidence from nexcore-constants)
// =========================================================================

mod confidence_integration_tests {
    use super::*;

    #[test]
    fn clamped_confidence_flows_through_buzz_decision() {
        // Confidence::new(1.5) clamps to 1.0 — should always buzz
        let board = Board::new(Round::Jeopardy, &[]).expect("Failed to initialize Jeopardy board");
        let state = GameState::new(&["Alice", "Bob", "Carol"], board);
        let clue = Clue::new(Category::SignalDetection, ClueValue(200), 0.5, false);
        let clamped = Confidence::new(1.5); // clamps to 1.0
        assert!((clamped.value() - 1.0).abs() < f64::EPSILON);
        assert_eq!(should_buzz(&clue, clamped, &state), BuzzDecision::Buzz);
    }

    #[test]
    fn clamped_negative_confidence_passes_buzz() {
        // Confidence::new(-0.5) clamps to 0.0 — should always pass
        let board = Board::new(Round::Jeopardy, &[]).expect("Failed to initialize Jeopardy board");
        let state = GameState::new(&["Alice", "Bob", "Carol"], board);
        let clue = Clue::new(Category::SignalDetection, ClueValue(200), 0.5, false);
        let clamped = Confidence::new(-0.5); // clamps to 0.0
        assert!((clamped.value() - 0.0).abs() < f64::EPSILON);
        assert_eq!(should_buzz(&clue, clamped, &state), BuzzDecision::Pass);
    }

    #[test]
    fn zero_confidence_daily_double_wager_is_zero() {
        let board = Board::new(Round::Jeopardy, &[CluePosition::new(4, 0)])
            .expect("Failed to initialize Jeopardy board");
        let state = GameState::new(&["Alice", "Bob", "Carol"], board);
        let wager = optimal_daily_double_wager(&state, Confidence::new(0.0));
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            assert_eq!(w.amount, 0, "Zero confidence should produce zero wager");
        }
    }

    #[test]
    fn max_confidence_daily_double_wager_is_max() {
        let board = Board::new(Round::Jeopardy, &[CluePosition::new(4, 0)])
            .expect("Failed to initialize Jeopardy board");
        let state = GameState::new(&["Alice", "Bob", "Carol"], board);
        let wager = optimal_daily_double_wager(&state, Confidence::new(1.0));
        assert!(wager.is_ok());
        if let Ok(w) = wager {
            assert!(w.amount > 0, "Max confidence should produce positive wager");
        }
    }

    #[test]
    fn confidence_round_trips_through_compound_metrics() {
        let game = GameResult {
            final_score: 2000,
            categories_correct: vec![Category::SignalDetection, Category::PrimitiveExtraction],
            accuracy: 0.8,
            correct_count: 2,
            attempts: vec![
                AttemptRecord {
                    position: CluePosition::new(0, 0),
                    category: Category::SignalDetection,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(0.9),
                    was_daily_double: false,
                    wager: None,
                },
                AttemptRecord {
                    position: CluePosition::new(0, 1),
                    category: Category::PrimitiveExtraction,
                    value: ClueValue(200),
                    correct: true,
                    confidence: Confidence::new(0.7),
                    was_daily_double: false,
                    wager: None,
                },
            ],
        };

        let metrics = compound_velocity(&[game]);
        // efficiency and reuse are Confidence types — verify they're in [0, 1]
        assert!(metrics.efficiency.value() >= 0.0 && metrics.efficiency.value() <= 1.0);
        assert!(metrics.reuse.value() >= 0.0 && metrics.reuse.value() <= 1.0);
    }

    #[test]
    fn confidence_value_preserved_in_attempt_record() {
        let conf = Confidence::new(0.73);
        let record = AttemptRecord {
            position: CluePosition::new(2, 3),
            category: Category::ValidationPhasing,
            value: ClueValue(600),
            correct: true,
            confidence: conf,
            was_daily_double: false,
            wager: None,
        };
        assert!((record.confidence.value() - 0.73).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_serializes_as_f64() {
        let conf = Confidence::new(0.85);
        let json = serde_json::to_string(&conf);
        assert!(json.is_ok());
        if let Ok(s) = json {
            // Should serialize as a plain f64
            let val: std::result::Result<f64, _> = serde_json::from_str(&s);
            assert!(val.is_ok());
            if let Ok(v) = val {
                assert!((v - 0.85).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn confidence_round_trips_through_serde() {
        // Derived Deserialize preserves exact value; clamping is only via new()
        let original = Confidence::new(0.42);
        let json = serde_json::to_string(&original);
        assert!(json.is_ok());
        if let Ok(s) = json {
            let back: std::result::Result<Confidence, _> = serde_json::from_str(&s);
            assert!(back.is_ok());
            if let Ok(c) = back {
                assert!((c.value() - 0.42).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn compound_empty_history_uses_zero_confidence() {
        let metrics = compound_velocity(&[]);
        assert!((metrics.efficiency.value() - 0.0).abs() < f64::EPSILON);
        assert!((metrics.reuse.value() - 0.0).abs() < f64::EPSILON);
        assert!((metrics.velocity - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn score_selections_returns_all_available() {
        let board = Board::new(Round::Jeopardy, &[]).expect("Failed to initialize Jeopardy board");
        let expected_count = board.remaining_count();
        let scores = score_selections(&board);
        assert_eq!(
            scores.len(),
            expected_count,
            "score_selections should return one entry per available clue"
        );
        // All scores should be positive
        for s in &scores {
            assert!(s.score > 0.0, "selection score should be positive");
        }
    }
}
