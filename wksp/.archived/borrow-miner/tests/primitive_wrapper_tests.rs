//! Unit tests for T2-P primitive wrappers
//!
//! Phase 0 Preclinical: Type safety mechanism tests

use borrow_miner::game::{Combo, Depth, Lifetime, ParticleId, Points, Score};

// ═══════════════════════════════════════════════════════════════════════════
// Score Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn score_starts_at_zero() {
    assert_eq!(Score::ZERO.0, 0);
    assert_eq!(Score::default().0, 0);
}

#[test]
fn score_saturating_add_prevents_overflow() {
    let max_score = Score(u64::MAX);
    let points = Points(100);
    let result = max_score.saturating_add(points);
    assert_eq!(result.0, u64::MAX, "Score should saturate at max");
}

#[test]
fn score_add_assigns_correctly() {
    let mut score = Score(100);
    score += Points(50);
    assert_eq!(score.0, 150);
}

// ═══════════════════════════════════════════════════════════════════════════
// Combo Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn combo_starts_at_zero() {
    assert_eq!(Combo::ZERO.0, 0);
    assert_eq!(Combo::default().0, 0);
}

#[test]
fn combo_increment_respects_max() {
    let mut combo = Combo::ZERO;
    for _ in 0..20 {
        combo = combo.increment();
    }
    assert_eq!(combo.0, Combo::MAX.0, "Combo should cap at MAX");
}

#[test]
fn combo_decrement_floors_at_zero() {
    let combo = Combo::ZERO;
    let decremented = combo.decrement();
    assert_eq!(decremented.0, 0, "Combo should not go below 0");
}

#[test]
fn combo_reset_returns_zero() {
    let combo = Combo(5);
    assert_eq!(combo.reset().0, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Depth Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn depth_defaults_to_one() {
    assert_eq!(Depth::default().0, 1.0);
}

#[test]
fn depth_increase_is_additive() {
    let depth = Depth::default();
    let increased = depth.increase(0.5);
    assert!((increased.0 - 1.5).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════
// Lifetime Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifetime_defaults_to_one() {
    assert_eq!(Lifetime::default().0, 1.0);
}

#[test]
fn lifetime_decay_reduces_value() {
    let life = Lifetime::default();
    let decayed = life.decay(0.3);
    assert!((decayed.0 - 0.7).abs() < f64::EPSILON);
}

#[test]
fn lifetime_decay_floors_at_zero() {
    let life = Lifetime(0.1);
    let decayed = life.decay(0.5);
    assert_eq!(decayed.0, 0.0, "Lifetime should not go negative");
}

#[test]
fn lifetime_expired_when_zero_or_less() {
    assert!(Lifetime(0.0).is_expired());
    assert!(!Lifetime(0.001).is_expired());
}

// ═══════════════════════════════════════════════════════════════════════════
// ParticleId Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn particle_id_starts_at_zero() {
    assert_eq!(ParticleId::default().0, 0);
}

#[test]
fn particle_id_next_batch_increments() {
    let id = ParticleId(100);
    let next = id.next_batch(10);
    assert_eq!(next.0, 110);
}

#[test]
fn particle_id_wraps_on_overflow() {
    let id = ParticleId(usize::MAX - 5);
    let next = id.next_batch(10);
    // Should wrap around
    assert!(next.0 < id.0, "ParticleId should wrap on overflow");
}

// ═══════════════════════════════════════════════════════════════════════════
// Points Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn points_zero_constant() {
    assert_eq!(Points::ZERO.0, 0);
}
