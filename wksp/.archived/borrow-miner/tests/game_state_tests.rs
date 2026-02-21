//! Unit tests for GameState
//!
//! Phase 0 Preclinical: State machine isolation tests

use borrow_miner::game::{Combo, Depth, Score, Points};

#[test]
fn combo_increment_increases_value() {
    let combo = Combo::ZERO;
    let next = combo.increment();
    assert_eq!(next.0, 1);
}

#[test]
fn combo_increment_caps_at_max() {
    let combo = Combo::MAX;
    let next = combo.increment();
    assert_eq!(next.0, Combo::MAX.0);
}

#[test]
fn combo_decrement_reduces_value() {
    let combo = Combo(5);
    let next = combo.decrement();
    assert_eq!(next.0, 4);
}

#[test]
fn combo_decrement_floors_at_zero() {
    let combo = Combo::ZERO;
    let next = combo.decrement();
    assert_eq!(next.0, 0);
}

#[test]
fn combo_reset_returns_zero() {
    let combo = Combo(7);
    let reset = combo.reset();
    assert_eq!(reset.0, 0);
}

#[test]
fn depth_increase_adds_delta() {
    let depth = Depth::default();
    let next = depth.increase(0.5);
    assert!((next.0 - 1.5).abs() < f64::EPSILON);
}

#[test]
fn depth_default_is_one() {
    let depth = Depth::default();
    assert!((depth.0 - 1.0).abs() < f64::EPSILON);
}

#[test]
fn score_add_points_increases_value() {
    let score = Score::ZERO;
    let points = Points(100);
    let new_score = score + points;
    assert_eq!(new_score.0, 100);
}

#[test]
fn score_add_assign_works() {
    let mut score = Score(50);
    score += Points(25);
    assert_eq!(score.0, 75);
}

#[test]
fn score_saturating_add_prevents_overflow() {
    let score = Score(u64::MAX - 10);
    let new_score = score.saturating_add(Points(100));
    assert_eq!(new_score.0, u64::MAX);
}

#[test]
fn points_from_ore_scales_with_combo() {
    use borrow_miner::game::OreType;

    let ore = OreType::Iron; // base value 10
    let depth = Depth::default(); // 1.0

    let points_no_combo = Points::from_ore(&ore, Combo::ZERO, depth);
    let points_with_combo = Points::from_ore(&ore, Combo(5), depth);

    // combo 5 = 1.0 + (5 * 0.1) = 1.5x multiplier
    assert!(points_with_combo.0 > points_no_combo.0);
}

#[test]
fn points_from_ore_scales_with_depth() {
    use borrow_miner::game::OreType;

    let ore = OreType::Iron;
    let combo = Combo::ZERO;

    let shallow = Points::from_ore(&ore, combo, Depth(1.0));
    let deep = Points::from_ore(&ore, combo, Depth(2.0));

    assert_eq!(deep.0, shallow.0 * 2);
}
