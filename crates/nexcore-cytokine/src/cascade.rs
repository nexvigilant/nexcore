//! Cascade amplification for cytokine signals.
//!
//! ## T1 Grounding
//!
//! - `CascadeRule` → → (causality) - trigger causes response
//! - `Amplification` → N (quantity) - multiply signal count
//! - `Cascade chain` → ρ (recursion) - signals trigger more signals

use crate::{Cytokine, CytokineFamily, ReceptorFilter, Scope, ThreatLevel};
use serde::{Deserialize, Serialize};

/// Rule for triggering cascade reactions.
///
/// When a signal matches the trigger filter, emit the response signals.
///
/// # Tier: T2-C (Composite)
/// Grounds to: → (causality) + ρ (recursion)
#[derive(Debug, Clone)]
pub struct CascadeRule {
    /// Rule identifier
    pub id: String,

    /// Filter for triggering signals
    pub trigger: ReceptorFilter,

    /// Signals to emit when triggered
    pub responses: Vec<CascadeResponse>,

    /// Maximum cascade depth (prevents infinite loops)
    pub max_depth: u8,

    /// Is this rule active?
    pub active: bool,
}

/// A response signal template for cascade emission.
#[derive(Debug, Clone)]
pub struct CascadeResponse {
    /// Cytokine family for response
    pub family: CytokineFamily,

    /// Signal name
    pub name: String,

    /// Severity (or inherit from trigger)
    pub severity: Option<ThreatLevel>,

    /// Scope (or inherit from trigger)
    pub scope: Option<Scope>,

    /// Amplification factor (how many copies to emit)
    pub amplification: u8,

    /// Delay before emission (milliseconds)
    pub delay_ms: u32,
}

impl CascadeResponse {
    /// Create a new cascade response
    pub fn new(family: CytokineFamily, name: impl Into<String>) -> Self {
        Self {
            family,
            name: name.into(),
            severity: None,
            scope: None,
            amplification: 1,
            delay_ms: 0,
        }
    }

    /// Set severity
    #[must_use]
    pub fn with_severity(mut self, severity: ThreatLevel) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set scope
    #[must_use]
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set amplification factor
    #[must_use]
    pub fn amplified(mut self, factor: u8) -> Self {
        self.amplification = factor.max(1);
        self
    }

    /// Set delay
    #[must_use]
    pub fn delayed(mut self, ms: u32) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Generate response signals from a trigger
    pub fn generate(&self, trigger: &Cytokine, depth: u8) -> Vec<Cytokine> {
        let mut signals = Vec::with_capacity(self.amplification as usize);

        for i in 0..self.amplification {
            let signal = Cytokine::new(self.family, &self.name)
                .with_severity(self.severity.unwrap_or(trigger.severity))
                .with_scope(self.scope.unwrap_or(trigger.scope))
                .with_payload(serde_json::json!({
                    "cascade_depth": depth,
                    "cascade_index": i,
                    "trigger_id": trigger.id,
                    "trigger_name": trigger.name,
                }))
                .with_source(format!("cascade:{}", trigger.id));

            // Mark as non-cascadable if at max depth to prevent loops
            let signal = if depth >= 3 {
                signal.no_cascade()
            } else {
                signal
            };

            signals.push(signal);
        }

        signals
    }
}

impl CascadeRule {
    /// Create a new cascade rule
    pub fn new(id: impl Into<String>, trigger: ReceptorFilter) -> Self {
        Self {
            id: id.into(),
            trigger,
            responses: Vec::new(),
            max_depth: 3,
            active: true,
        }
    }

    /// Add a response to this rule
    #[must_use]
    pub fn with_response(mut self, response: CascadeResponse) -> Self {
        self.responses.push(response);
        self
    }

    /// Set max cascade depth
    #[must_use]
    pub fn with_max_depth(mut self, depth: u8) -> Self {
        self.max_depth = depth;
        self
    }

    /// Deactivate the rule
    #[must_use]
    pub fn deactivate(mut self) -> Self {
        self.active = false;
        self
    }

    /// Check if a signal should trigger this cascade
    pub fn matches(&self, signal: &Cytokine, current_depth: u8) -> bool {
        if !self.active {
            return false;
        }
        if current_depth >= self.max_depth {
            return false;
        }
        if !signal.cascadable {
            return false;
        }
        self.trigger.matches(signal)
    }

    /// Generate all response signals for a trigger
    pub fn execute(&self, trigger: &Cytokine, current_depth: u8) -> Vec<Cytokine> {
        if !self.matches(trigger, current_depth) {
            return Vec::new();
        }

        self.responses
            .iter()
            .flat_map(|r| r.generate(trigger, current_depth + 1))
            .collect()
    }
}

// ── Loop Gain Monitor ─────────────────────────────────────────────────────────

/// Monitors cumulative loop gain across cascade chain hops.
///
/// When the product of amplification factors exceeds the threshold, the cascade
/// is self-sustaining (positive feedback loop) and must be broken.
///
/// ## T1 Grounding
/// - `N` (Quantity) — tracks cumulative amplification product
/// - `∂` (Boundary) — enforces the loop-gain ceiling
/// - `→` (Causality) — breaks runaway causal chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopGainMonitor {
    /// Per-hop amplification records: `(source_id, amplification_factor)`.
    pub chain_amplifications: Vec<(String, f64)>,
    /// Trip threshold — expert range 5–8, default midpoint 6.0.
    pub loop_gain_threshold: f64,
    /// Whether the breaker is currently tripped.
    pub is_tripped: bool,
}

/// Emitted when loop gain exceeds the configured threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopGainViolation {
    /// Computed product of all per-hop amplification factors.
    pub total_gain: f64,
    /// The chain of `(source_id, amplification_factor)` hops that produced the violation.
    pub chain: Vec<(String, f64)>,
    /// The threshold that was exceeded.
    pub threshold: f64,
}

impl LoopGainMonitor {
    /// Create a new monitor with the given trip threshold.
    #[must_use]
    pub fn new(threshold: f64) -> Self {
        Self {
            chain_amplifications: Vec::new(),
            loop_gain_threshold: threshold,
            is_tripped: false,
        }
    }

    /// Create a monitor with the default threshold of 6.0.
    #[must_use]
    pub fn default_threshold() -> Self {
        Self::new(6.0)
    }

    /// Record one hop in the cascade chain.
    pub fn record_hop(&mut self, source: &str, amplification: f64) {
        self.chain_amplifications
            .push((source.to_owned(), amplification));
    }

    /// Check whether the cumulative loop gain exceeds the threshold.
    ///
    /// Returns `Some(LoopGainViolation)` when the cascade is self-sustaining,
    /// or `None` when gain is within safe bounds.
    #[must_use]
    pub fn check_loop_gain(&self) -> Option<LoopGainViolation> {
        let total_gain: f64 = self
            .chain_amplifications
            .iter()
            .map(|(_, amp)| amp)
            .product();
        if total_gain > self.loop_gain_threshold {
            Some(LoopGainViolation {
                total_gain,
                chain: self.chain_amplifications.clone(),
                threshold: self.loop_gain_threshold,
            })
        } else {
            None
        }
    }

    /// Reset the monitor, clearing all recorded hops and the tripped flag.
    pub fn reset(&mut self) {
        self.chain_amplifications.clear();
        self.is_tripped = false;
    }

    /// Compute the current cumulative loop gain (product of all hop amplifications).
    #[must_use]
    pub fn total_gain(&self) -> f64 {
        self.chain_amplifications
            .iter()
            .map(|(_, amp)| amp)
            .product()
    }
}

/// Pre-defined cascade patterns based on biological immune responses
pub mod patterns {
    use super::{CascadeResponse, CascadeRule, CytokineFamily, ReceptorFilter, Scope, ThreatLevel};

    /// Inflammatory cascade: IL-1 → IL-6 + TNF-α
    ///
    /// When alarm is raised, trigger acute response and potential termination.
    pub fn inflammatory() -> CascadeRule {
        CascadeRule::new("inflammatory", ReceptorFilter::family(CytokineFamily::Il1))
            .with_response(
                CascadeResponse::new(CytokineFamily::Il6, "acute_response")
                    .with_severity(ThreatLevel::High),
            )
            .with_response(
                CascadeResponse::new(CytokineFamily::TnfAlpha, "potential_termination")
                    .with_severity(ThreatLevel::Medium),
            )
    }

    /// Proliferation cascade: IL-2 → CSF (spawn more agents)
    pub fn proliferation() -> CascadeRule {
        CascadeRule::new("proliferation", ReceptorFilter::family(CytokineFamily::Il2))
            .with_response(CascadeResponse::new(CytokineFamily::Csf, "spawn_agent").amplified(2))
    }

    /// Suppression cascade: IL-10 → TGF-β (dampen response)
    pub fn suppression() -> CascadeRule {
        CascadeRule::new("suppression", ReceptorFilter::family(CytokineFamily::Il10)).with_response(
            CascadeResponse::new(CytokineFamily::TgfBeta, "regulate").with_scope(Scope::Paracrine),
        )
    }

    /// Activation cascade: IFN-γ → IL-2 (enhance and multiply)
    pub fn activation() -> CascadeRule {
        CascadeRule::new(
            "activation",
            ReceptorFilter::family(CytokineFamily::IfnGamma),
        )
        .with_response(
            CascadeResponse::new(CytokineFamily::Il2, "proliferate")
                .with_severity(ThreatLevel::High),
        )
    }

    /// Critical response: Critical severity → systemic alarm
    pub fn critical_response() -> CascadeRule {
        CascadeRule::new(
            "critical_response",
            ReceptorFilter::default().with_min_severity(ThreatLevel::Critical),
        )
        .with_response(
            CascadeResponse::new(CytokineFamily::Il1, "systemic_alarm")
                .with_scope(Scope::Systemic)
                .with_severity(ThreatLevel::Critical),
        )
        .with_max_depth(1) // Prevent alarm loops
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_cascade_response_generation() {
        let response = CascadeResponse::new(CytokineFamily::Il6, "acute").amplified(3);

        let trigger = Cytokine::new(CytokineFamily::Il1, "alarm").with_severity(ThreatLevel::High);

        let signals = response.generate(&trigger, 1);
        assert_eq!(signals.len(), 3);

        for signal in &signals {
            assert_eq!(signal.family, CytokineFamily::Il6);
            assert_eq!(signal.name, "acute");
            assert_eq!(signal.severity, ThreatLevel::High); // Inherited
        }
    }

    #[test]
    fn test_cascade_rule_matching() {
        let rule =
            CascadeRule::new("test", ReceptorFilter::family(CytokineFamily::Il1)).with_max_depth(2);

        let il1 = Cytokine::new(CytokineFamily::Il1, "test");
        let tnf = Cytokine::new(CytokineFamily::TnfAlpha, "test");

        assert!(rule.matches(&il1, 0));
        assert!(rule.matches(&il1, 1));
        assert!(!rule.matches(&il1, 2)); // Max depth reached
        assert!(!rule.matches(&tnf, 0)); // Wrong family
    }

    #[test]
    fn test_cascade_rule_execution() {
        let rule = CascadeRule::new("test", ReceptorFilter::family(CytokineFamily::Il1))
            .with_response(CascadeResponse::new(CytokineFamily::Il6, "response1"))
            .with_response(CascadeResponse::new(CytokineFamily::TnfAlpha, "response2"));

        let trigger = Cytokine::new(CytokineFamily::Il1, "alarm");
        let responses = rule.execute(&trigger, 0);

        assert_eq!(responses.len(), 2);
    }

    #[test]
    fn test_non_cascadable_signal() {
        let rule = CascadeRule::new("test", ReceptorFilter::default())
            .with_response(CascadeResponse::new(CytokineFamily::Il6, "response"));

        let trigger = Cytokine::new(CytokineFamily::Il1, "test").no_cascade();

        assert!(!rule.matches(&trigger, 0));
    }

    #[test]
    fn test_inflammatory_pattern() {
        let rule = patterns::inflammatory();
        let trigger = Cytokine::new(CytokineFamily::Il1, "alarm");

        let responses = rule.execute(&trigger, 0);
        assert_eq!(responses.len(), 2);

        let families: Vec<_> = responses.iter().map(|s| s.family).collect();
        assert!(families.contains(&CytokineFamily::Il6));
        assert!(families.contains(&CytokineFamily::TnfAlpha));
    }

    // ── LoopGainMonitor tests ──────────────────────────────────────────────────

    #[test]
    fn test_loop_gain_below_threshold_passes() {
        let mut monitor = LoopGainMonitor::default_threshold(); // threshold = 6.0
        monitor.record_hop("hop-a", 1.5);
        monitor.record_hop("hop-b", 2.0);
        // total gain = 3.0 — below 6.0
        assert!((monitor.total_gain() - 3.0).abs() < f64::EPSILON);
        assert!(monitor.check_loop_gain().is_none());
    }

    #[test]
    fn test_loop_gain_above_threshold_trips() {
        let mut monitor = LoopGainMonitor::default_threshold(); // threshold = 6.0
        monitor.record_hop("hop-a", 2.0);
        monitor.record_hop("hop-b", 2.0);
        monitor.record_hop("hop-c", 2.0);
        // total gain = 8.0 — exceeds 6.0
        assert!((monitor.total_gain() - 8.0).abs() < f64::EPSILON);
        let violation = monitor.check_loop_gain();
        assert!(violation.is_some());
        let v = violation.unwrap();
        assert!((v.total_gain - 8.0).abs() < f64::EPSILON);
        assert_eq!(v.chain.len(), 3);
        assert!((v.threshold - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_loop_gain_exact_threshold_does_not_trip() {
        let mut monitor = LoopGainMonitor::new(4.0);
        monitor.record_hop("hop-a", 2.0);
        monitor.record_hop("hop-b", 2.0);
        // total gain = 4.0 — NOT strictly greater than threshold
        assert!(monitor.check_loop_gain().is_none());
    }

    #[test]
    fn test_loop_gain_monitor_reset_clears_state() {
        let mut monitor = LoopGainMonitor::default_threshold();
        monitor.record_hop("hop-a", 3.0);
        monitor.record_hop("hop-b", 3.0);
        monitor.is_tripped = true;
        monitor.reset();
        assert!(monitor.chain_amplifications.is_empty());
        assert!(!monitor.is_tripped);
        assert!(monitor.check_loop_gain().is_none());
    }

    #[test]
    fn test_loop_gain_empty_chain_returns_one() {
        let monitor = LoopGainMonitor::default_threshold();
        // Product of empty iterator is 1.0 (multiplicative identity)
        assert!((monitor.total_gain() - 1.0).abs() < f64::EPSILON);
        assert!(monitor.check_loop_gain().is_none());
    }
}
