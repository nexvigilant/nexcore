//! # Claude Code Infrastructure Mapping — Integumentary System
//!
//! Maps biological integumentary concepts to Claude Code's permission and settings
//! infrastructure per Biological Alignment v2.0 §2.
//!
//! ```text
//! Epidermis  = Permission Rules (deny → ask → allow cascade, first match wins)
//! Dermis     = Settings Precedence Stack (managed → cli → local → project → user)
//! Hypodermis = Sandboxing (Docker isolation, network restrictions)
//! Scarring   = Adaptive deny rules added after security incidents
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Permission Rules — Epidermis (§2: outer boundary, stateless, disposable)
// ============================================================================

/// A single permission rule in the epidermis cascade.
/// Biology: dead skin cells (deny), sensory receptors (ask), pores (allow).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRule {
    /// Pattern to match (e.g., "Bash(cargo test *)", "Read(.env)")
    pub pattern: String,
    /// Decision level for this rule
    pub decision: PermissionDecision,
    /// Origin: manual configuration or adaptive scarring
    pub origin: RuleOrigin,
}

/// Permission decision levels — maps to biological skin barrier response.
/// Evaluation order: Deny → Ask → Allow (first match wins).
/// Biology: Kill → Investigate → Permit (same logic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionDecision {
    /// Block without deliberation — dead skin cells shedding threats
    Deny,
    /// Pause to evaluate — sensory receptors detecting stimulus
    Ask,
    /// Intentional opening — pores allowing known-good flows
    Allow,
}

/// How a rule was created — distinguishes innate from adaptive immunity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleOrigin {
    /// Manually configured by user
    Manual,
    /// Auto-generated after security incident (scarring mechanism)
    Adaptive,
    /// Inherited from managed/org settings
    Managed,
}

/// The complete permission cascade (epidermis).
/// Evaluates rules in order: Deny → Ask → Allow.
/// First matching rule wins — just like biological immune checkpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCascade {
    /// All permission rules ordered by precedence
    pub rules: Vec<PermissionRule>,
}

impl Default for PermissionCascade {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl PermissionCascade {
    /// Evaluate a tool request against the permission cascade.
    /// Returns the first matching decision, or None if no rule matches.
    ///
    /// Biology: the skin evaluates incoming stimuli against layers of barrier.
    pub fn evaluate(&self, tool_request: &str) -> Option<PermissionDecision> {
        // Deny rules checked first (kill), then Ask (investigate), then Allow (permit)
        for decision_level in &[
            PermissionDecision::Deny,
            PermissionDecision::Ask,
            PermissionDecision::Allow,
        ] {
            for rule in &self.rules {
                if rule.decision == *decision_level && tool_request.contains(&rule.pattern) {
                    return Some(*decision_level);
                }
            }
        }
        None
    }

    /// Add a permission rule.
    pub fn add_rule(&mut self, rule: PermissionRule) {
        self.rules.push(rule);
    }

    /// Count rules by origin — useful for measuring scarring rate.
    pub fn adaptive_rule_count(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| r.origin == RuleOrigin::Adaptive)
            .count()
    }

    /// Selective permeability check: is Allow(*) absent?
    /// A wide-open boundary (Allow-all) is biologically inviable.
    pub fn is_selectively_permeable(&self) -> bool {
        !self
            .rules
            .iter()
            .any(|r| r.decision == PermissionDecision::Allow && r.pattern == "*")
    }
}

// ============================================================================
// Settings Precedence — Dermis (§2: deep inspection layer)
// ============================================================================

/// Settings precedence scope — dermis layers ordered from deepest to surface.
/// Each layer overrides the ones below it.
///
/// Biology: skin regenerates from bottom up; settings evaluate from top down.
/// Managed policy (deepest, most protected) produces rules that surface at boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsScope {
    /// Subcutaneous sensors — org-wide, cannot override
    Managed,
    /// Pressure receptors — session-specific overrides
    CliArgs,
    /// Temperature sensors — personal environment tuning
    Local,
    /// Pain receptors — project-specific constraints
    Project,
    /// Deep tissue — baseline personal preferences
    User,
}

/// A settings value at a specific precedence scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedSetting {
    /// Setting key
    pub key: String,
    /// Setting value (JSON)
    pub value: serde_json::Value,
    /// Which dermis layer this lives in
    pub scope: SettingsScope,
}

/// The complete settings precedence stack (dermis).
/// Evaluates from highest precedence (Managed) to lowest (User).
///
/// Biology: each layer senses different stimuli.
/// Managed = hypothalamic directive (org-wide, non-negotiable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPrecedence {
    /// All settings ordered by scope precedence
    pub settings: Vec<ScopedSetting>,
}

impl Default for SettingsPrecedence {
    fn default() -> Self {
        Self {
            settings: Vec::new(),
        }
    }
}

impl SettingsPrecedence {
    /// Resolve a setting key through the precedence stack.
    /// Higher-precedence scopes override lower ones.
    ///
    /// Evaluation order: Managed → CliArgs → Local → Project → User
    pub fn resolve(&self, key: &str) -> Option<&serde_json::Value> {
        let precedence = [
            SettingsScope::Managed,
            SettingsScope::CliArgs,
            SettingsScope::Local,
            SettingsScope::Project,
            SettingsScope::User,
        ];

        for scope in &precedence {
            if let Some(setting) = self
                .settings
                .iter()
                .find(|s| s.key == key && s.scope == *scope)
            {
                return Some(&setting.value);
            }
        }
        None
    }

    /// Add a scoped setting.
    pub fn set(&mut self, key: String, value: serde_json::Value, scope: SettingsScope) {
        self.settings.push(ScopedSetting { key, value, scope });
    }

    /// Count layers that define a given key — measures how deeply contested a setting is.
    pub fn layer_count(&self, key: &str) -> usize {
        self.settings
            .iter()
            .filter(|s| s.key == key)
            .map(|s| s.scope)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

// ============================================================================
// Sandboxing — Hypodermis (§2: buffer between outside and vital organs)
// ============================================================================

/// Sandbox isolation layer — the fat layer insulating core from external damage.
///
/// Biology: hypodermis stores energy and insulates. Fascia controls flow between layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxLayer {
    /// Docker sandbox active (fat layer insulation)
    pub docker_active: bool,
    /// Network restrictions active (fascia flow control)
    pub network_restricted: bool,
    /// Allowed network targets (selective permeability through fascia)
    pub allowed_networks: Vec<String>,
}

impl Default for SandboxLayer {
    fn default() -> Self {
        Self {
            docker_active: false,
            network_restricted: true,
            allowed_networks: Vec::new(),
        }
    }
}

impl SandboxLayer {
    /// Check if the sandbox provides adequate insulation.
    /// Biology: thin hypodermis = poor insulation = thermal vulnerability.
    pub fn is_insulated(&self) -> bool {
        self.docker_active || self.network_restricted
    }
}

// ============================================================================
// Scarring — Adaptive Permission Rules (§2: skin reinforcement under stress)
// ============================================================================

/// A scar — a permanent reinforcement at the site of a security incident.
///
/// Biology: repeated stress causes tissue reinforcement (stronger at break point).
/// Claude Code: after a security incident, new deny rules are added permanently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scar {
    /// What caused the scarring (incident description)
    pub incident: String,
    /// The deny rule created in response
    pub deny_rule: PermissionRule,
    /// When the scarring occurred
    pub scarred_at: String,
    /// Risk level that triggered scarring (HIGH = 0.7% of decisions in brain.db)
    pub risk_level: RiskLevel,
}

/// Risk level classification from brain.db decision audit.
/// LOW=97.9%, MEDIUM=1.3%, HIGH=0.7% — scarring happens at the 0.7%.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Routine operation — no scarring
    Low,
    /// Elevated caution — potential scarring trigger
    Medium,
    /// Scarring event — produces adaptive deny rule
    High,
}

/// The scarring mechanism — accumulates adaptive deny rules over the organism's lifetime.
///
/// Biology: each scar makes that exact point more resistant to future damage.
/// Evidence: 14,252 decisions, 98 HIGH risk events → 98 potential scars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScarringMechanism {
    /// Accumulated scars (adaptive deny rules)
    pub scars: Vec<Scar>,
    /// Reference to permission cascade where scars are applied
    pub total_decisions: u64,
}

impl Default for ScarringMechanism {
    fn default() -> Self {
        Self {
            scars: Vec::new(),
            total_decisions: 0,
        }
    }
}

impl ScarringMechanism {
    /// Record a new scar from a security incident.
    /// The system becomes more resistant at this exact point.
    pub fn scar(&mut self, incident: &str, pattern: &str, risk_level: RiskLevel) -> &Scar {
        let deny_rule = PermissionRule {
            pattern: pattern.to_string(),
            decision: PermissionDecision::Deny,
            origin: RuleOrigin::Adaptive,
        };

        self.scars.push(Scar {
            incident: incident.to_string(),
            deny_rule,
            scarred_at: nexcore_chrono::DateTime::now().to_rfc3339(),
            risk_level,
        });

        self.total_decisions += 1;

        // Return reference to last element
        // Safety: we just pushed, so last() is guaranteed Some
        self.scars.last().unwrap_or_else(|| {
            // This branch is unreachable but satisfies deny(unwrap_used)
            static FALLBACK: std::sync::LazyLock<Scar> = std::sync::LazyLock::new(|| Scar {
                incident: String::new(),
                deny_rule: PermissionRule {
                    pattern: String::new(),
                    decision: PermissionDecision::Deny,
                    origin: RuleOrigin::Adaptive,
                },
                scarred_at: String::new(),
                risk_level: RiskLevel::Low,
            });
            &FALLBACK
        })
    }

    /// Scarring rate: percentage of decisions that produced scars.
    /// Healthy: <1% (matches brain.db HIGH risk at 0.7%).
    pub fn scarring_rate(&self) -> f64 {
        if self.total_decisions == 0 {
            return 0.0;
        }
        self.scars.len() as f64 / self.total_decisions as f64 * 100.0
    }

    /// Extract all adaptive deny rules for injection into a PermissionCascade.
    pub fn deny_rules(&self) -> Vec<&PermissionRule> {
        self.scars.iter().map(|s| &s.deny_rule).collect()
    }
}

// ============================================================================
// Integumentary Health — Design Diagnostic (§2 table)
// ============================================================================

/// Health check for the integumentary system per alignment doc §2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegumentaryHealth {
    /// Is the outermost layer disposable? (Can rotate permissions without restart)
    pub layer_disposable: bool,
    /// Does repeated stress cause reinforcement? (Adaptive deny rules after incidents)
    pub stress_reinforcement: bool,
    /// Is the boundary selectively permeable? (No Allow(*) rules)
    pub selectively_permeable: bool,
    /// Can you sense the environment? (Denied/asked requests logged for analysis)
    pub environment_sensing: bool,
}

impl IntegumentaryHealth {
    /// Diagnose from current system state.
    pub fn diagnose(cascade: &PermissionCascade, scars: &ScarringMechanism) -> Self {
        Self {
            layer_disposable: true, // Permission rules are always rotatable
            stress_reinforcement: !scars.scars.is_empty(),
            selectively_permeable: cascade.is_selectively_permeable(),
            environment_sensing: true, // brain.db logs all decisions
        }
    }

    /// Is the integumentary system healthy?
    pub fn is_healthy(&self) -> bool {
        self.layer_disposable
            && self.stress_reinforcement
            && self.selectively_permeable
            && self.environment_sensing
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_cascade_deny_first() {
        let mut cascade = PermissionCascade::default();
        cascade.add_rule(PermissionRule {
            pattern: ".env".to_string(),
            decision: PermissionDecision::Deny,
            origin: RuleOrigin::Manual,
        });
        cascade.add_rule(PermissionRule {
            pattern: ".env".to_string(),
            decision: PermissionDecision::Allow,
            origin: RuleOrigin::Manual,
        });

        // Deny should win over Allow for same pattern
        assert_eq!(
            cascade.evaluate("Read(.env)"),
            Some(PermissionDecision::Deny)
        );
    }

    #[test]
    fn permission_cascade_no_match() {
        let cascade = PermissionCascade::default();
        assert_eq!(cascade.evaluate("Read(src/main.rs)"), None);
    }

    #[test]
    fn selective_permeability_detects_allow_all() {
        let mut cascade = PermissionCascade::default();
        cascade.add_rule(PermissionRule {
            pattern: "*".to_string(),
            decision: PermissionDecision::Allow,
            origin: RuleOrigin::Manual,
        });
        assert!(!cascade.is_selectively_permeable());
    }

    #[test]
    fn selective_permeability_normal() {
        let mut cascade = PermissionCascade::default();
        cascade.add_rule(PermissionRule {
            pattern: "src/**".to_string(),
            decision: PermissionDecision::Allow,
            origin: RuleOrigin::Manual,
        });
        assert!(cascade.is_selectively_permeable());
    }

    #[test]
    fn settings_precedence_managed_wins() {
        let mut stack = SettingsPrecedence::default();
        stack.set(
            "max_tokens".to_string(),
            serde_json::json!(1000),
            SettingsScope::User,
        );
        stack.set(
            "max_tokens".to_string(),
            serde_json::json!(500),
            SettingsScope::Managed,
        );

        let resolved = stack.resolve("max_tokens");
        assert_eq!(resolved, Some(&serde_json::json!(500)));
    }

    #[test]
    fn settings_precedence_missing_key() {
        let stack = SettingsPrecedence::default();
        assert!(stack.resolve("nonexistent").is_none());
    }

    #[test]
    fn sandbox_insulation() {
        let sandbox = SandboxLayer::default();
        assert!(sandbox.is_insulated()); // network_restricted=true by default
    }

    #[test]
    fn scarring_creates_deny_rule() {
        let mut mechanism = ScarringMechanism::default();
        mechanism.total_decisions = 100;
        let scar = mechanism.scar("unsafe code detected", "unsafe", RiskLevel::High);
        assert_eq!(scar.deny_rule.decision, PermissionDecision::Deny);
        assert_eq!(scar.deny_rule.origin, RuleOrigin::Adaptive);
    }

    #[test]
    fn scarring_rate_calculation() {
        let mut mechanism = ScarringMechanism::default();
        mechanism.total_decisions = 1000;
        let _ = mechanism.scar("incident 1", "pattern1", RiskLevel::High);
        let _ = mechanism.scar("incident 2", "pattern2", RiskLevel::High);
        // 2 scars / 1002 decisions (scar() increments) ≈ 0.2%
        assert!(mechanism.scarring_rate() < 1.0);
    }

    #[test]
    fn adaptive_rule_count() {
        let mut cascade = PermissionCascade::default();
        cascade.add_rule(PermissionRule {
            pattern: "manual".to_string(),
            decision: PermissionDecision::Deny,
            origin: RuleOrigin::Manual,
        });
        cascade.add_rule(PermissionRule {
            pattern: "adaptive".to_string(),
            decision: PermissionDecision::Deny,
            origin: RuleOrigin::Adaptive,
        });
        assert_eq!(cascade.adaptive_rule_count(), 1);
    }

    #[test]
    fn health_diagnosis() {
        let mut cascade = PermissionCascade::default();
        cascade.add_rule(PermissionRule {
            pattern: "src/**".to_string(),
            decision: PermissionDecision::Allow,
            origin: RuleOrigin::Manual,
        });

        let mut scars = ScarringMechanism::default();
        scars.total_decisions = 100;
        let _ = scars.scar("test", "test_pattern", RiskLevel::High);

        let health = IntegumentaryHealth::diagnose(&cascade, &scars);
        assert!(health.is_healthy());
    }
}
