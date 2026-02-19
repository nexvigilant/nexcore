//! Cascade amplification for cytokine signals.
//!
//! ## T1 Grounding
//!
//! - `CascadeRule` → → (causality) - trigger causes response
//! - `Amplification` → N (quantity) - multiply signal count
//! - `Cascade chain` → ρ (recursion) - signals trigger more signals

use crate::{Cytokine, CytokineFamily, ReceptorFilter, Scope, ThreatLevel};

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
    pub fn with_severity(mut self, severity: ThreatLevel) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set scope
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set amplification factor
    pub fn amplified(mut self, factor: u8) -> Self {
        self.amplification = factor.max(1);
        self
    }

    /// Set delay
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
    pub fn with_response(mut self, response: CascadeResponse) -> Self {
        self.responses.push(response);
        self
    }

    /// Set max cascade depth
    pub fn with_max_depth(mut self, depth: u8) -> Self {
        self.max_depth = depth;
        self
    }

    /// Deactivate the rule
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

/// Pre-defined cascade patterns based on biological immune responses
pub mod patterns {
    use super::*;

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
}
