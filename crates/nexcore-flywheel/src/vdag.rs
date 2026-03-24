//! VDAG integration — Reality Gradient + Learning Loops for the flywheel.
//!
//! Wraps the deterministic cascade in goal-aware evaluation: evidence grading,
//! cumulative Reality Gradient (0.0-1.0), and learning loop analysis that
//! recommends threshold adjustments.
//!
//! ## Design: Wrapper, Not Invasion
//!
//! Loop evaluators remain pure functions. This module adds observation and
//! learning *around* the cascade, never *inside* it.
//!
//! ## T1 Primitive Grounding: N (Quantity) + ∂ (Boundary) + ρ (Recursion) + κ (Comparison)

use crate::loops::{
    CascadeInput, CascadeResult, SystemState, cascade, elastic::ElasticState,
    friction::FrictionClassification, gyroscopic::GyroscopicState,
    momentum::MomentumClassification, rim_integrity::RimState,
};
use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

// ============================================================================
// Evidence Quality
// ============================================================================

/// Evidence quality level for a single loop evaluation.
/// Maps to VDAG evidence_quality: None=0.0, Weak=0.33, Moderate=0.66, Strong=1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQuality {
    /// No data or default/zero inputs.
    None,
    /// Single signal, borderline state.
    Weak,
    /// Clear signal, consistent state.
    Moderate,
    /// Definitive signal, extreme state (healthy or failing).
    Strong,
}

impl EvidenceQuality {
    /// Numeric score for Reality Gradient computation.
    pub fn score(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Weak => 0.33,
            Self::Moderate => 0.66,
            Self::Strong => 1.0,
        }
    }
}

impl std::fmt::Display for EvidenceQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Weak => write!(f, "weak"),
            Self::Moderate => write!(f, "moderate"),
            Self::Strong => write!(f, "strong"),
        }
    }
}

// ============================================================================
// Goal Types
// ============================================================================

/// A flywheel goal — what state we're aiming for.
/// Default: all loops healthy, equal weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelGoal {
    /// Human-readable goal label.
    pub description: String,
    /// Desired system state (e.g. `Thriving`).
    pub target_state: SystemState,
    /// Per-loop weights (rim, momentum, friction, gyroscopic, elastic), summing to 1.0.
    pub loop_weights: [f64; 5],
}

impl Default for FlywheelGoal {
    fn default() -> Self {
        Self {
            description: "All loops healthy".into(),
            target_state: SystemState::Thriving,
            loop_weights: [0.2; 5],
        }
    }
}

// ============================================================================
// Reality Gradient
// ============================================================================

/// VDAG Reality Gradient classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealityRating {
    /// < 0.20 — execution blocked.
    TestingTheater,
    /// 0.20 - 0.50
    SafetyValidated,
    /// 0.50 - 0.80
    EfficacyDemonstrated,
    /// 0.80 - 0.95
    ScaleConfirmed,
    /// > 0.95
    ProductionReady,
}

impl std::fmt::Display for RealityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TestingTheater => write!(f, "testing_theater"),
            Self::SafetyValidated => write!(f, "safety_validated"),
            Self::EfficacyDemonstrated => write!(f, "efficacy_demonstrated"),
            Self::ScaleConfirmed => write!(f, "scale_confirmed"),
            Self::ProductionReady => write!(f, "production_ready"),
        }
    }
}

/// Per-loop evidence breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopEvidence {
    /// Loop identifier (e.g. "rim", "momentum").
    pub loop_name: String,
    /// Evidence quality for this loop.
    pub quality: EvidenceQuality,
    /// Weight in the composite score.
    pub weight: f64,
    /// Whether this loop met the goal's target state.
    pub achieved_target: bool,
}

/// Cumulative Reality Gradient across all 5 loops.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityGradient {
    /// Overall score 0.0-1.0.
    pub score: f64,
    /// Classification.
    pub rating: RealityRating,
    /// Per-loop breakdown.
    pub per_loop: Vec<LoopEvidence>,
    /// False if score < 0.20 (testing theater).
    pub executable: bool,
}

// ============================================================================
// Graded Result
// ============================================================================

/// CascadeResult enriched with Reality Gradient and goal context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradedCascadeResult {
    /// Raw cascade evaluation result.
    pub cascade: CascadeResult,
    /// Reality Gradient derived from evidence grading.
    pub reality: RealityGradient,
    /// Goal context used for this evaluation.
    pub goal: FlywheelGoal,
}

// ============================================================================
// Learning Types
// ============================================================================

/// Which learning loop to activate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningLoopType {
    /// Fix execution errors.
    Single,
    /// Question thresholds (failure rate > 20%).
    Double,
    /// Question the model itself (every 5th analysis).
    Triple,
}

impl std::fmt::Display for LearningLoopType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single => write!(f, "single"),
            Self::Double => write!(f, "double"),
            Self::Triple => write!(f, "triple"),
        }
    }
}

/// A concrete threshold adjustment recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    /// Parameter name to adjust.
    pub parameter: String,
    /// Current threshold value.
    pub current_value: f64,
    /// Recommended new value.
    pub suggested_value: f64,
    /// 0.0-1.0 confidence in this recommendation.
    pub confidence: f64,
    /// Human-readable justification.
    pub reason: String,
}

/// A learning insight produced by history analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningInsight {
    /// Which learning loop produced this insight.
    pub loop_type: LearningLoopType,
    /// What was observed.
    pub observation: String,
    /// Concrete threshold adjustments, if any.
    pub suggested_adjustments: Vec<ThresholdAdjustment>,
}

/// A timestamped cascade record for history tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeRecord {
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Raw cascade result.
    pub cascade: CascadeResult,
    /// Graded reality assessment.
    pub reality: RealityGradient,
}

// ============================================================================
// Evidence Grading — Pure Functions
// ============================================================================

/// Grade Rim Integrity evidence from result.
fn grade_rim(result: &crate::loops::rim_integrity::RimIntegrityResult) -> EvidenceQuality {
    match result.state {
        RimState::Disintegrated => EvidenceQuality::Strong,
        RimState::Critical => EvidenceQuality::Weak,
        RimState::Thriving => {
            // Margin-based: high margin = strong, narrow = moderate
            if result.ratio > 2.0 {
                EvidenceQuality::Strong
            } else {
                EvidenceQuality::Moderate
            }
        }
    }
}

/// Grade Momentum evidence from result.
fn grade_momentum(result: &crate::loops::momentum::MomentumResult) -> EvidenceQuality {
    match result.classification {
        MomentumClassification::Stalled => EvidenceQuality::Strong,
        MomentumClassification::Low => EvidenceQuality::Weak,
        MomentumClassification::Normal => EvidenceQuality::Moderate,
        MomentumClassification::High => EvidenceQuality::Strong,
    }
}

/// Grade Friction evidence from result.
fn grade_friction(result: &crate::loops::friction::FrictionResult) -> EvidenceQuality {
    match result.classification {
        FrictionClassification::Critical => EvidenceQuality::Strong,
        FrictionClassification::Warning => EvidenceQuality::Moderate,
        FrictionClassification::Acceptable => {
            if result.net_drain.abs() < f64::EPSILON {
                // Zero friction with zero inputs = no data
                if result.total_drain.abs() < f64::EPSILON {
                    EvidenceQuality::None
                } else {
                    EvidenceQuality::Strong
                }
            } else {
                EvidenceQuality::Moderate
            }
        }
    }
}

/// Grade Gyroscopic evidence from result.
fn grade_gyroscopic(result: &crate::loops::gyroscopic::GyroscopicResult) -> EvidenceQuality {
    match result.state {
        GyroscopicState::GimbalLock => EvidenceQuality::Strong,
        GyroscopicState::NoStability => EvidenceQuality::Weak,
        GyroscopicState::Precessing => EvidenceQuality::Moderate,
        GyroscopicState::Stable => {
            if result.score > 0.9 {
                EvidenceQuality::Strong
            } else {
                EvidenceQuality::Moderate
            }
        }
    }
}

/// Grade Elastic evidence from result.
fn grade_elastic(result: &crate::loops::elastic::ElasticResult) -> EvidenceQuality {
    match result.state {
        ElasticState::FatigueFailureImminent => EvidenceQuality::Strong,
        ElasticState::YieldExceeded => EvidenceQuality::Moderate,
        ElasticState::Nominal => {
            if result.cycles_remaining > 500 {
                EvidenceQuality::Strong
            } else {
                EvidenceQuality::Moderate
            }
        }
    }
}

/// Classify Reality Gradient score into a rating.
fn classify_rating(score: f64) -> RealityRating {
    if score > 0.95 {
        RealityRating::ProductionReady
    } else if score > 0.80 {
        RealityRating::ScaleConfirmed
    } else if score > 0.50 {
        RealityRating::EfficacyDemonstrated
    } else if score >= 0.20 {
        RealityRating::SafetyValidated
    } else {
        RealityRating::TestingTheater
    }
}

/// Check if a loop achieved its target state.
fn loop_achieved_target(cascade: &CascadeResult, loop_index: usize, goal: &FlywheelGoal) -> bool {
    let target = goal.target_state;
    match loop_index {
        0 => match target {
            SystemState::Thriving => cascade.rim.state == RimState::Thriving,
            SystemState::Failed => cascade.rim.state == RimState::Disintegrated,
            _ => cascade.rim.state != RimState::Disintegrated,
        },
        1 => match target {
            SystemState::Thriving => matches!(
                cascade.momentum.classification,
                MomentumClassification::High | MomentumClassification::Normal
            ),
            SystemState::Failed => {
                cascade.momentum.classification == MomentumClassification::Stalled
            }
            _ => cascade.momentum.classification != MomentumClassification::Stalled,
        },
        2 => match target {
            SystemState::Thriving => {
                cascade.friction.classification == FrictionClassification::Acceptable
            }
            SystemState::Failed => {
                cascade.friction.classification == FrictionClassification::Critical
            }
            _ => cascade.friction.classification != FrictionClassification::Critical,
        },
        3 => match target {
            SystemState::Thriving => matches!(
                cascade.gyroscopic.state,
                GyroscopicState::Stable | GyroscopicState::Precessing
            ),
            SystemState::Failed => cascade.gyroscopic.state == GyroscopicState::GimbalLock,
            _ => cascade.gyroscopic.state != GyroscopicState::GimbalLock,
        },
        4 => match target {
            SystemState::Thriving => cascade.elastic.state == ElasticState::Nominal,
            SystemState::Failed => cascade.elastic.state == ElasticState::FatigueFailureImminent,
            _ => cascade.elastic.state != ElasticState::FatigueFailureImminent,
        },
        _ => false,
    }
}

// ============================================================================
// Core Public Functions
// ============================================================================

const LOOP_NAMES: [&str; 5] = ["rim", "momentum", "friction", "gyroscopic", "elastic"];

/// Grade a CascadeResult's evidence quality per loop and compute Reality Gradient.
///
/// Pure function — no side effects.
pub fn grade_cascade(result: &CascadeResult, goal: &FlywheelGoal) -> RealityGradient {
    let qualities = [
        grade_rim(&result.rim),
        grade_momentum(&result.momentum),
        grade_friction(&result.friction),
        grade_gyroscopic(&result.gyroscopic),
        grade_elastic(&result.elastic),
    ];

    let per_loop: Vec<LoopEvidence> = qualities
        .iter()
        .enumerate()
        .map(|(i, &quality)| LoopEvidence {
            loop_name: LOOP_NAMES[i].to_string(),
            quality,
            weight: goal.loop_weights[i],
            achieved_target: loop_achieved_target(result, i, goal),
        })
        .collect();

    let weighted_sum: f64 = per_loop.iter().map(|e| e.weight * e.quality.score()).sum();
    let max_possible: f64 = per_loop.iter().map(|e| e.weight).sum();
    let score = if max_possible.abs() < f64::EPSILON {
        0.0
    } else {
        (weighted_sum / max_possible).clamp(0.0, 1.0)
    };

    let rating = classify_rating(score);
    let executable = score >= 0.20;

    RealityGradient {
        score,
        rating,
        per_loop,
        executable,
    }
}

/// Run cascade + grade in one call. Convenience wrapper.
pub fn evaluate(
    input: &CascadeInput,
    thresholds: &FlywheelThresholds,
    goal: &FlywheelGoal,
) -> GradedCascadeResult {
    let cascade_result = cascade(input, thresholds);
    let reality = grade_cascade(&cascade_result, goal);
    GradedCascadeResult {
        cascade: cascade_result,
        reality,
        goal: goal.clone(),
    }
}

/// Analyze a history of cascade records to produce learning insights.
///
/// - Single loop: identify which loops failed most recently.
/// - Double loop: if failure_rate > 20%, question thresholds.
/// - Triple loop: every 5th analysis, question loop weights.
pub fn analyze_history(
    history: &[CascadeRecord],
    thresholds: &FlywheelThresholds,
) -> Vec<LearningInsight> {
    if history.is_empty() {
        return vec![];
    }

    let mut insights = Vec::new();
    let total = history.len();

    // Count failures (non-Thriving system states)
    let failure_count = history
        .iter()
        .filter(|r| r.cascade.system_state != SystemState::Thriving)
        .count();
    let failure_rate = failure_count as f64 / total as f64;

    // Triple loop: every 5th analysis, question the model
    if total.is_multiple_of(5) && total > 0 {
        // Find loops that never contributed strong evidence
        let mut weak_loops = Vec::new();
        for (i, name) in LOOP_NAMES.iter().enumerate() {
            let strong_count = history
                .iter()
                .filter(|r| {
                    r.reality
                        .per_loop
                        .get(i)
                        .is_some_and(|e| e.quality == EvidenceQuality::Strong)
                })
                .count();
            if strong_count == 0 {
                weak_loops.push(name.to_string());
            }
        }
        if !weak_loops.is_empty() {
            insights.push(LearningInsight {
                loop_type: LearningLoopType::Triple,
                observation: format!(
                    "Loops [{}] never produced strong evidence across {} evaluations — question whether they contribute meaningful signal",
                    weak_loops.join(", "),
                    total,
                ),
                suggested_adjustments: vec![],
            });
        }
    }

    // Double loop: failure rate > 20%, question thresholds
    if failure_rate > 0.20 {
        let adjustments = recommend_adjustments_from_history(history, thresholds);
        insights.push(LearningInsight {
            loop_type: LearningLoopType::Double,
            observation: format!(
                "Failure rate {:.0}% ({}/{}) exceeds 20% threshold — thresholds may be miscalibrated",
                failure_rate * 100.0,
                failure_count,
                total,
            ),
            suggested_adjustments: adjustments,
        });
    }

    // Single loop: identify most recent failure
    if let Some(last_failure) = history
        .iter()
        .rev()
        .find(|r| r.cascade.system_state != SystemState::Thriving)
    {
        let failing_loops: Vec<String> = last_failure
            .reality
            .per_loop
            .iter()
            .filter(|e| !e.achieved_target)
            .map(|e| e.loop_name.clone())
            .collect();
        if !failing_loops.is_empty() {
            insights.push(LearningInsight {
                loop_type: LearningLoopType::Single,
                observation: format!(
                    "Most recent failure: system_state={}, failing loops: [{}]",
                    last_failure.cascade.system_state,
                    failing_loops.join(", "),
                ),
                suggested_adjustments: vec![],
            });
        }
    }

    insights
}

/// Derive threshold adjustment recommendations from history patterns.
fn recommend_adjustments_from_history(
    history: &[CascadeRecord],
    thresholds: &FlywheelThresholds,
) -> Vec<ThresholdAdjustment> {
    let mut adjustments = Vec::new();
    let total = history.len();
    if total == 0 {
        return adjustments;
    }

    // Check per-loop failure rates to identify over-sensitive thresholds
    let friction_critical_count = history
        .iter()
        .filter(|r| r.cascade.friction.classification == FrictionClassification::Critical)
        .count();
    let friction_rate = friction_critical_count as f64 / total as f64;
    if friction_rate > 0.30 {
        adjustments.push(ThresholdAdjustment {
            parameter: "friction_warning_threshold".into(),
            current_value: thresholds.friction_warning_threshold,
            suggested_value: thresholds.friction_warning_threshold * 1.25,
            confidence: (friction_rate * 0.8).min(0.9),
            reason: format!(
                "Friction critical in {:.0}% of evaluations — threshold may be too sensitive",
                friction_rate * 100.0,
            ),
        });
    }

    let momentum_stall_count = history
        .iter()
        .filter(|r| r.cascade.momentum.classification == MomentumClassification::Stalled)
        .count();
    let momentum_rate = momentum_stall_count as f64 / total as f64;
    if momentum_rate > 0.30 {
        adjustments.push(ThresholdAdjustment {
            parameter: "min_momentum_for_stability".into(),
            current_value: thresholds.min_momentum_for_stability,
            suggested_value: thresholds.min_momentum_for_stability * 0.75,
            confidence: (momentum_rate * 0.8).min(0.9),
            reason: format!(
                "Momentum stalled in {:.0}% of evaluations — stability threshold may be too high",
                momentum_rate * 100.0,
            ),
        });
    }

    let rim_critical_count = history
        .iter()
        .filter(|r| {
            r.cascade.rim.state == RimState::Critical
                || r.cascade.rim.state == RimState::Disintegrated
        })
        .count();
    let rim_rate = rim_critical_count as f64 / total as f64;
    if rim_rate > 0.30 {
        adjustments.push(ThresholdAdjustment {
            parameter: "rim_critical_margin".into(),
            current_value: thresholds.rim_critical_margin,
            suggested_value: thresholds.rim_critical_margin * 1.5,
            confidence: (rim_rate * 0.7).min(0.9),
            reason: format!(
                "Rim critical/disintegrated in {:.0}% of evaluations — margin may be too tight",
                rim_rate * 100.0,
            ),
        });
    }

    adjustments
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loops::*;

    fn healthy_input() -> CascadeInput {
        CascadeInput {
            rim: RimInput {
                tensile_strength: 200.0,
                centrifugal_force: 50.0,
            },
            momentum: MomentumInput {
                inertia: 100.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            friction: FrictionInput {
                manual_processes: 1.0,
                human_touchpoints: 1.0,
                velocity: 1.0,
                automation_coverage: 0.9,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 500.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 20.0,
                yield_point: 100.0,
                fatigue_cycles: 50,
                fatigue_limit: 1000,
            },
        }
    }

    fn failing_input() -> CascadeInput {
        CascadeInput {
            rim: RimInput {
                tensile_strength: 10.0,
                centrifugal_force: 100.0,
            },
            momentum: MomentumInput {
                inertia: 1.0,
                omega: 1.0,
                friction_drain: 50.0,
            },
            friction: FrictionInput {
                manual_processes: 100.0,
                human_touchpoints: 100.0,
                velocity: 50.0,
                automation_coverage: 0.0,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 0.0,
                perturbation_torque: 100.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 200.0,
                yield_point: 100.0,
                fatigue_cycles: 2000,
                fatigue_limit: 1000,
            },
        }
    }

    fn default_goal() -> FlywheelGoal {
        FlywheelGoal::default()
    }

    fn default_thresholds() -> FlywheelThresholds {
        FlywheelThresholds::default()
    }

    // ── Evidence Quality ───────────────────────────────────────────────

    #[test]
    fn evidence_quality_scores() {
        assert!((EvidenceQuality::None.score() - 0.0).abs() < f64::EPSILON);
        assert!((EvidenceQuality::Weak.score() - 0.33).abs() < f64::EPSILON);
        assert!((EvidenceQuality::Moderate.score() - 0.66).abs() < f64::EPSILON);
        assert!((EvidenceQuality::Strong.score() - 1.0).abs() < f64::EPSILON);
    }

    // ── Grading ────────────────────────────────────────────────────────

    #[test]
    fn grade_healthy_cascade_strong() {
        let result = cascade(&healthy_input(), &default_thresholds());
        let gradient = grade_cascade(&result, &default_goal());
        // Healthy system should produce high reality score
        assert!(
            gradient.score > 0.60,
            "Expected > 0.60, got {}",
            gradient.score
        );
        assert!(gradient.executable);
        assert_ne!(gradient.rating, RealityRating::TestingTheater);
    }

    #[test]
    fn grade_failing_cascade() {
        let result = cascade(&failing_input(), &default_thresholds());
        let gradient = grade_cascade(&result, &default_goal());
        // Failing system still produces strong evidence (of failure)
        assert!(
            gradient.score > 0.20,
            "Strong failure evidence should be above theater"
        );
        assert!(gradient.executable);
    }

    #[test]
    fn per_loop_breakdown_has_five_entries() {
        let result = cascade(&healthy_input(), &default_thresholds());
        let gradient = grade_cascade(&result, &default_goal());
        assert_eq!(gradient.per_loop.len(), 5);
        assert_eq!(gradient.per_loop[0].loop_name, "rim");
        assert_eq!(gradient.per_loop[4].loop_name, "elastic");
    }

    #[test]
    fn healthy_loops_achieve_targets() {
        let result = cascade(&healthy_input(), &default_thresholds());
        let gradient = grade_cascade(&result, &default_goal());
        for evidence in &gradient.per_loop {
            assert!(
                evidence.achieved_target,
                "Loop {} should achieve target",
                evidence.loop_name
            );
        }
    }

    // ── Evaluate convenience ───────────────────────────────────────────

    #[test]
    fn evaluate_returns_graded_result() {
        let graded = evaluate(&healthy_input(), &default_thresholds(), &default_goal());
        assert_eq!(graded.cascade.system_state, SystemState::Thriving);
        assert!(graded.reality.score > 0.50);
        assert_eq!(graded.goal.description, "All loops healthy");
    }

    // ── Reality Rating classification ──────────────────────────────────

    #[test]
    fn rating_boundaries() {
        assert_eq!(classify_rating(0.0), RealityRating::TestingTheater);
        assert_eq!(classify_rating(0.19), RealityRating::TestingTheater);
        assert_eq!(classify_rating(0.20), RealityRating::SafetyValidated);
        assert_eq!(classify_rating(0.50), RealityRating::SafetyValidated);
        assert_eq!(classify_rating(0.51), RealityRating::EfficacyDemonstrated);
        assert_eq!(classify_rating(0.80), RealityRating::EfficacyDemonstrated);
        assert_eq!(classify_rating(0.81), RealityRating::ScaleConfirmed);
        assert_eq!(classify_rating(0.95), RealityRating::ScaleConfirmed);
        assert_eq!(classify_rating(0.96), RealityRating::ProductionReady);
    }

    // ── Learning Loop Analysis ─────────────────────────────────────────

    #[test]
    fn empty_history_no_insights() {
        let insights = analyze_history(&[], &default_thresholds());
        assert!(insights.is_empty());
    }

    #[test]
    fn single_loop_on_recent_failure() {
        let t = default_thresholds();
        let goal = default_goal();
        // 3 healthy + 1 failing = 25% failure rate
        let mut history = Vec::new();
        for _ in 0..3 {
            let result = cascade(&healthy_input(), &t);
            let reality = grade_cascade(&result, &goal);
            history.push(CascadeRecord {
                timestamp_ms: 0,
                cascade: result,
                reality,
            });
        }
        let result = cascade(&failing_input(), &t);
        let reality = grade_cascade(&result, &goal);
        history.push(CascadeRecord {
            timestamp_ms: 0,
            cascade: result,
            reality,
        });

        let insights = analyze_history(&history, &t);
        // Should have both single and double (25% > 20%)
        assert!(
            insights
                .iter()
                .any(|i| i.loop_type == LearningLoopType::Single)
        );
        assert!(
            insights
                .iter()
                .any(|i| i.loop_type == LearningLoopType::Double)
        );
    }

    #[test]
    fn triple_loop_on_fifth() {
        let t = default_thresholds();
        let goal = default_goal();
        let mut history = Vec::new();
        for _ in 0..5 {
            let result = cascade(&healthy_input(), &t);
            let reality = grade_cascade(&result, &goal);
            history.push(CascadeRecord {
                timestamp_ms: 0,
                cascade: result,
                reality,
            });
        }
        let insights = analyze_history(&history, &t);
        assert!(
            insights
                .iter()
                .any(|i| i.loop_type == LearningLoopType::Triple)
        );
    }

    #[test]
    fn no_double_below_threshold() {
        let t = default_thresholds();
        let goal = default_goal();
        // 9 healthy + 1 failing = 10% failure rate, below 20%
        let mut history = Vec::new();
        for i in 0..10 {
            let input = if i == 5 {
                failing_input()
            } else {
                healthy_input()
            };
            let result = cascade(&input, &t);
            let reality = grade_cascade(&result, &goal);
            history.push(CascadeRecord {
                timestamp_ms: 0,
                cascade: result,
                reality,
            });
        }
        let insights = analyze_history(&history, &t);
        assert!(
            !insights
                .iter()
                .any(|i| i.loop_type == LearningLoopType::Double)
        );
    }

    // ── Threshold Adjustments ──────────────────────────────────────────

    #[test]
    fn adjustments_from_high_friction() {
        let t = default_thresholds();
        let goal = default_goal();
        // 4 evaluations with critical friction
        let mut history = Vec::new();
        for _ in 0..4 {
            let input = CascadeInput {
                rim: RimInput {
                    tensile_strength: 200.0,
                    centrifugal_force: 50.0,
                },
                momentum: MomentumInput {
                    inertia: 100.0,
                    omega: 5.0,
                    friction_drain: 0.0,
                },
                friction: FrictionInput {
                    manual_processes: 100.0,
                    human_touchpoints: 100.0,
                    velocity: 50.0,
                    automation_coverage: 0.0,
                },
                gyroscopic: GyroscopicInput {
                    momentum_l: 500.0,
                    perturbation_torque: 10.0,
                    critical_momentum: 50.0,
                },
                elastic: ElasticInput {
                    stress: 20.0,
                    yield_point: 100.0,
                    fatigue_cycles: 50,
                    fatigue_limit: 1000,
                },
            };
            let result = cascade(&input, &t);
            let reality = grade_cascade(&result, &goal);
            history.push(CascadeRecord {
                timestamp_ms: 0,
                cascade: result,
                reality,
            });
        }
        let adjustments = recommend_adjustments_from_history(&history, &t);
        assert!(
            adjustments
                .iter()
                .any(|a| a.parameter == "friction_warning_threshold")
        );
    }

    // ── Default Goal ───────────────────────────────────────────────────

    #[test]
    fn default_goal_equal_weights() {
        let goal = FlywheelGoal::default();
        let sum: f64 = goal.loop_weights.iter().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    // ── Serialization ──────────────────────────────────────────────────

    #[test]
    fn graded_result_serializable() {
        let graded = evaluate(&healthy_input(), &default_thresholds(), &default_goal());
        let json = serde_json::to_string(&graded);
        assert!(json.is_ok());
        let parsed: Result<GradedCascadeResult, _> =
            serde_json::from_str(&json.unwrap_or_default());
        assert!(parsed.is_ok());
    }
}
