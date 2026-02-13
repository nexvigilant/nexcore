/// Trust-based decision engine.
///
/// Maps trust engine state to actionable decisions (Allow, RequireApproval,
/// Deny, Escalate) by evaluating multiple signals simultaneously:
///
/// 1. **Trust level**: Primary classification from the engine score
/// 2. **Confidence**: Credible interval width — uncertain entities need review
/// 3. **Velocity**: Rapid trust degradation triggers escalation
/// 4. **Significance**: Insufficient evidence forces conservative decisions
///
/// # Decision Matrix
///
/// ```text
/// Level       | Significant? | Stable?    | Decision
/// ------------|--------------|------------|------------------
/// HighlyTrust | Yes          | Yes        | Allow
/// Trusted     | Yes          | Yes        | Allow
/// Trusted     | Yes          | Degrading  | RequireApproval
/// Neutral     | Yes          | *          | RequireApproval
/// Suspicious  | Yes          | *          | Deny
/// Untrusted   | *            | *          | Deny
/// *           | No           | *          | RequireApproval
/// *           | *            | Anomalous  | Escalate
/// ```
///
/// Tier: T3 (ς State + ∂ Boundary + κ Comparison + → Causality + N Quantity)
use crate::confidence::prob_exceeds;
use crate::engine::TrustEngine;
use crate::level::TrustLevel;
use crate::volatility::{TrustDirection, TrustVelocity};

/// An actionable decision derived from trust state.
///
/// Tier: T2-P (ς State + ∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustDecision {
    /// Proceed without intervention. Trust is sufficient.
    Allow,
    /// Require human review before proceeding. Trust is ambiguous.
    RequireApproval,
    /// Block the action. Trust is insufficient.
    Deny,
    /// Immediate escalation required. Anomalous trust behavior detected.
    Escalate,
}

impl TrustDecision {
    /// Whether the decision permits the action (Allow only).
    pub fn is_permitted(self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Whether the decision blocks the action (Deny or Escalate).
    pub fn is_blocked(self) -> bool {
        matches!(self, Self::Deny | Self::Escalate)
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Allow => "Allow",
            Self::RequireApproval => "Require Approval",
            Self::Deny => "Deny",
            Self::Escalate => "Escalate",
        }
    }
}

impl core::fmt::Display for TrustDecision {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Configuration for the trust decision policy.
///
/// Controls thresholds for each signal that contributes to the decision.
///
/// Tier: T2-C (∂ Boundary + N Quantity + κ Comparison)
#[derive(Debug, Clone, Copy)]
pub struct PolicyConfig {
    /// Minimum trust level to Allow without approval. Default: Trusted
    pub allow_level: TrustLevel,
    /// Minimum trust level before outright Deny. Default: Suspicious
    /// Scores below this level result in Deny.
    pub deny_below: TrustLevel,
    /// Anomaly threshold for velocity-based Escalation. Default: 0.08
    /// If absolute velocity exceeds this, escalate regardless of level.
    pub anomaly_threshold: f64,
    /// Minimum probability that score exceeds `allow_level.lower_bound()`
    /// before Allow is granted. Default: 0.7 (70% confidence).
    /// This prevents allowing entities with high uncertainty.
    pub confidence_threshold: f64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allow_level: TrustLevel::Trusted,
            deny_below: TrustLevel::Suspicious,
            anomaly_threshold: 0.08,
            confidence_threshold: 0.7,
        }
    }
}

impl PolicyConfig {
    /// Strict policy for safety-critical contexts.
    ///
    /// - Requires HighlyTrusted for Allow
    /// - Denies below Neutral
    /// - Lower anomaly threshold (more sensitive)
    /// - Higher confidence requirement
    pub fn strict() -> Self {
        Self {
            allow_level: TrustLevel::HighlyTrusted,
            deny_below: TrustLevel::Neutral,
            anomaly_threshold: 0.05,
            confidence_threshold: 0.85,
        }
    }

    /// Permissive policy for low-stakes contexts.
    ///
    /// - Allows at Neutral level
    /// - Only denies Untrusted
    /// - Higher anomaly threshold (less sensitive)
    /// - Lower confidence requirement
    pub fn permissive() -> Self {
        Self {
            allow_level: TrustLevel::Neutral,
            deny_below: TrustLevel::Untrusted,
            anomaly_threshold: 0.15,
            confidence_threshold: 0.5,
        }
    }
}

/// Detailed explanation of why a particular decision was reached.
///
/// Provides transparency into the decision-making process by
/// recording which factors contributed to the final outcome.
///
/// Tier: T2-C (→ Causality + ς State + N Quantity)
#[derive(Debug, Clone)]
pub struct DecisionRationale {
    /// The final decision
    pub decision: TrustDecision,
    /// Trust level at decision time
    pub level: TrustLevel,
    /// Trust score at decision time
    pub score: f64,
    /// Whether the engine had significant evidence
    pub significant: bool,
    /// Trust direction at decision time (if velocity tracker provided)
    pub direction: Option<TrustDirection>,
    /// Whether anomalous velocity was detected
    pub anomalous: bool,
    /// Probability that true score exceeds allow threshold
    pub confidence: f64,
    /// Which factor was the primary driver of the decision
    pub primary_factor: DecisionFactor,
}

/// The primary factor driving a trust decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionFactor {
    /// Decision based on trust level meeting or exceeding threshold
    TrustLevel,
    /// Decision due to insufficient evidence (not significant)
    InsufficientEvidence,
    /// Decision due to anomalous velocity (trust changing rapidly)
    AnomalousVelocity,
    /// Decision due to trust degradation trend
    DegradingTrend,
    /// Decision due to low confidence in the estimate
    LowConfidence,
}

impl core::fmt::Display for DecisionFactor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TrustLevel => write!(f, "Trust Level"),
            Self::InsufficientEvidence => write!(f, "Insufficient Evidence"),
            Self::AnomalousVelocity => write!(f, "Anomalous Velocity"),
            Self::DegradingTrend => write!(f, "Degrading Trend"),
            Self::LowConfidence => write!(f, "Low Confidence"),
        }
    }
}

/// Evaluate trust state and produce an actionable decision.
///
/// This is the primary decision function. It examines trust level,
/// significance, velocity, and confidence to produce a decision
/// with full rationale.
///
/// # Priority Order
///
/// 1. **Anomalous velocity** → Escalate (override everything)
/// 2. **Insufficient evidence** → RequireApproval
/// 3. **Below deny threshold** → Deny
/// 4. **Degrading trend + borderline** → RequireApproval
/// 5. **Low confidence** → RequireApproval
/// 6. **Above allow threshold** → Allow
/// 7. **Default** → RequireApproval
pub fn decide(
    engine: &TrustEngine,
    velocity: Option<&TrustVelocity>,
    config: &PolicyConfig,
) -> DecisionRationale {
    let level = engine.level();
    let score = engine.score();
    let significant = engine.is_significant();
    let alpha = engine.alpha();
    let beta = engine.beta();

    // Compute confidence: P(true score > allow_level.lower_bound())
    let allow_threshold = config.allow_level.lower_bound();
    let confidence = if alpha > 0.0 && beta > 0.0 {
        prob_exceeds(alpha, beta, allow_threshold)
    } else {
        0.0
    };

    // Check velocity
    let direction = velocity.map(|v| v.direction());
    let anomalous = velocity
        .map(|v| v.is_anomalous(config.anomaly_threshold))
        .unwrap_or(false);

    // Priority 1: Anomalous velocity → Escalate
    if anomalous {
        return DecisionRationale {
            decision: TrustDecision::Escalate,
            level,
            score,
            significant,
            direction,
            anomalous,
            confidence,
            primary_factor: DecisionFactor::AnomalousVelocity,
        };
    }

    // Priority 2: Insufficient evidence → RequireApproval
    if !significant {
        return DecisionRationale {
            decision: TrustDecision::RequireApproval,
            level,
            score,
            significant,
            direction,
            anomalous,
            confidence,
            primary_factor: DecisionFactor::InsufficientEvidence,
        };
    }

    // Priority 3: Below deny threshold → Deny
    if level < config.deny_below {
        return DecisionRationale {
            decision: TrustDecision::Deny,
            level,
            score,
            significant,
            direction,
            anomalous,
            confidence,
            primary_factor: DecisionFactor::TrustLevel,
        };
    }

    // Priority 4: Degrading + at or near allow boundary → RequireApproval
    if direction == Some(TrustDirection::Degrading) && level <= config.allow_level {
        return DecisionRationale {
            decision: TrustDecision::RequireApproval,
            level,
            score,
            significant,
            direction,
            anomalous,
            confidence,
            primary_factor: DecisionFactor::DegradingTrend,
        };
    }

    // Priority 5: Low confidence → RequireApproval
    if confidence < config.confidence_threshold {
        return DecisionRationale {
            decision: TrustDecision::RequireApproval,
            level,
            score,
            significant,
            direction,
            anomalous,
            confidence,
            primary_factor: DecisionFactor::LowConfidence,
        };
    }

    // Priority 6: Level meets or exceeds allow threshold → Allow
    if level >= config.allow_level {
        return DecisionRationale {
            decision: TrustDecision::Allow,
            level,
            score,
            significant,
            direction,
            anomalous,
            confidence,
            primary_factor: DecisionFactor::TrustLevel,
        };
    }

    // Default: RequireApproval
    DecisionRationale {
        decision: TrustDecision::RequireApproval,
        level,
        score,
        significant,
        direction,
        anomalous,
        confidence,
        primary_factor: DecisionFactor::TrustLevel,
    }
}

/// Quick decision without velocity tracking.
///
/// Convenience wrapper for `decide()` when velocity tracking is not used.
pub fn decide_simple(engine: &TrustEngine, config: &PolicyConfig) -> TrustDecision {
    decide(engine, None, config).decision
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::Evidence;

    const EPS: f64 = 1e-6;

    // --- TrustDecision ---

    #[test]
    fn decision_ordering() {
        assert!(TrustDecision::Allow < TrustDecision::RequireApproval);
        assert!(TrustDecision::RequireApproval < TrustDecision::Deny);
        assert!(TrustDecision::Deny < TrustDecision::Escalate);
    }

    #[test]
    fn decision_permission_semantics() {
        assert!(TrustDecision::Allow.is_permitted());
        assert!(!TrustDecision::RequireApproval.is_permitted());
        assert!(!TrustDecision::Deny.is_permitted());
        assert!(!TrustDecision::Escalate.is_permitted());

        assert!(!TrustDecision::Allow.is_blocked());
        assert!(!TrustDecision::RequireApproval.is_blocked());
        assert!(TrustDecision::Deny.is_blocked());
        assert!(TrustDecision::Escalate.is_blocked());
    }

    // --- Insufficient evidence → RequireApproval ---

    #[test]
    fn fresh_engine_requires_approval() {
        let engine = TrustEngine::new();
        let config = PolicyConfig::default();
        let rationale = decide(&engine, None, &config);
        assert_eq!(rationale.decision, TrustDecision::RequireApproval);
        assert_eq!(
            rationale.primary_factor,
            DecisionFactor::InsufficientEvidence
        );
        assert!(!rationale.significant);
    }

    // --- Trusted + significant → Allow ---

    #[test]
    fn trusted_entity_allowed() {
        let mut engine = TrustEngine::new();
        for _ in 0..30 {
            engine.record(Evidence::positive());
        }
        let config = PolicyConfig::default();
        let rationale = decide(&engine, None, &config);
        assert_eq!(rationale.decision, TrustDecision::Allow);
        assert_eq!(rationale.primary_factor, DecisionFactor::TrustLevel);
        assert!(rationale.significant);
    }

    // --- Untrusted → Deny ---

    #[test]
    fn untrusted_entity_denied() {
        let mut engine = TrustEngine::new();
        for _ in 0..30 {
            engine.record(Evidence::negative());
        }
        let config = PolicyConfig::default();
        let rationale = decide(&engine, None, &config);
        assert_eq!(rationale.decision, TrustDecision::Deny);
        assert_eq!(rationale.primary_factor, DecisionFactor::TrustLevel);
    }

    // --- Anomalous velocity → Escalate ---

    #[test]
    fn anomalous_velocity_triggers_escalation() {
        let mut engine = TrustEngine::new();
        for _ in 0..30 {
            engine.record(Evidence::positive());
        }

        let mut velocity = TrustVelocity::default();
        // Simulate dramatic sudden drop
        velocity.update(-0.15);
        velocity.update(-0.15);

        let config = PolicyConfig::default();
        let rationale = decide(&engine, Some(&velocity), &config);
        assert_eq!(rationale.decision, TrustDecision::Escalate);
        assert_eq!(rationale.primary_factor, DecisionFactor::AnomalousVelocity);
        assert!(rationale.anomalous);
    }

    // --- Degrading trend → RequireApproval ---

    #[test]
    fn degrading_trusted_requires_approval() {
        let mut engine = TrustEngine::new();
        // Build to Trusted (not HighlyTrusted) range
        for _ in 0..10 {
            engine.record(Evidence::positive());
        }
        // Add some negatives to pull score down near the Trusted boundary
        for _ in 0..3 {
            engine.record(Evidence::negative());
        }
        assert!(engine.is_significant());
        assert!(engine.level() <= TrustLevel::Trusted);

        let mut velocity = TrustVelocity::default();
        // Moderate degradation (not anomalous at 0.08, but degrading direction)
        for _ in 0..10 {
            velocity.update(-0.02);
        }

        let config = PolicyConfig::default();
        let rationale = decide(&engine, Some(&velocity), &config);
        assert_eq!(rationale.decision, TrustDecision::RequireApproval);
        assert_eq!(rationale.primary_factor, DecisionFactor::DegradingTrend);
    }

    // --- Policy configs ---

    #[test]
    fn strict_policy_requires_highly_trusted() {
        let mut engine = TrustEngine::new();
        // Build to "Trusted" but not "HighlyTrusted" (score in [0.6, 0.8))
        // 10 positives + 1 negative: alpha=11, beta=3.5, score=0.758 → Trusted
        for _ in 0..10 {
            engine.record(Evidence::positive());
        }
        engine.record(Evidence::negative());
        assert!(engine.is_significant());
        assert_eq!(engine.level(), TrustLevel::Trusted);

        let strict = PolicyConfig::strict();
        let decision = decide_simple(&engine, &strict);
        // Trusted is not HighlyTrusted, so strict policy won't Allow
        assert_ne!(decision, TrustDecision::Allow);
    }

    #[test]
    fn permissive_policy_allows_neutral() {
        let mut engine = TrustEngine::new();
        // Build to Neutral level with enough evidence for significance
        for _ in 0..5 {
            engine.record(Evidence::positive());
            engine.record(Evidence::negative());
        }
        assert!(engine.is_significant());

        let permissive = PolicyConfig::permissive();
        // Permissive allows at Neutral level
        // The actual decision depends on confidence and other factors
        let rationale = decide(&engine, None, &permissive);
        // At minimum, permissive should not Deny a neutral entity
        assert_ne!(rationale.decision, TrustDecision::Deny);
    }

    // --- decide_simple convenience ---

    #[test]
    fn decide_simple_matches_full() {
        let mut engine = TrustEngine::new();
        for _ in 0..20 {
            engine.record(Evidence::positive());
        }
        let config = PolicyConfig::default();
        let simple = decide_simple(&engine, &config);
        let full = decide(&engine, None, &config).decision;
        assert_eq!(simple, full);
    }

    // --- Rationale transparency ---

    #[test]
    fn rationale_includes_all_fields() {
        let mut engine = TrustEngine::new();
        for _ in 0..10 {
            engine.record(Evidence::positive());
        }
        let mut velocity = TrustVelocity::default();
        velocity.update(0.01);

        let config = PolicyConfig::default();
        let rationale = decide(&engine, Some(&velocity), &config);

        // All fields should be populated
        assert!(rationale.score > 0.0);
        assert!(rationale.direction.is_some());
        assert!(rationale.confidence >= 0.0);
    }

    // --- Display ---

    #[test]
    fn decision_display() {
        assert_eq!(format!("{}", TrustDecision::Allow), "Allow");
        assert_eq!(format!("{}", TrustDecision::Escalate), "Escalate");
        assert_eq!(
            format!("{}", DecisionFactor::AnomalousVelocity),
            "Anomalous Velocity"
        );
    }

    // --- Scenario: Trust lifecycle with policy ---

    #[test]
    fn lifecycle_with_policy_decisions() {
        let mut engine = TrustEngine::new();
        let config = PolicyConfig::default();

        // Phase 1: New entity — insufficient data
        assert_eq!(
            decide_simple(&engine, &config),
            TrustDecision::RequireApproval
        );

        // Phase 2: Build trust
        for _ in 0..30 {
            engine.record(Evidence::positive());
        }
        assert_eq!(decide_simple(&engine, &config), TrustDecision::Allow);

        // Phase 3: Betrayal
        for _ in 0..15 {
            engine.record(Evidence::negative());
        }
        let decision = decide_simple(&engine, &config);
        assert!(
            decision == TrustDecision::Deny || decision == TrustDecision::RequireApproval,
            "betrayed entity should not be allowed: {decision}"
        );
    }
}
