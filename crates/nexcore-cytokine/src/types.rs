//! Core cytokine types and enums.
//!
//! ## T1 Grounding
//!
//! - `CytokineFamily` → Σ (sum type, categorical)
//! - `Severity` → N (quantity, ordinal)
//! - `Scope` → λ (location, spatial extent)
//! - `Cytokine` → ς (state carrier)

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Cytokine families based on immunological classification.
///
/// Each family serves a distinct signaling purpose.
///
/// # Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: Σ (sum type)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CytokineFamily {
    /// IL-1: Alarm/Alert - Initial threat detection
    /// Triggers: SessionStart, first detection of antipattern
    Il1,

    /// IL-2: Growth/Proliferation - Spawn more responders
    /// Triggers: Subagent creation, skill instantiation
    Il2,

    /// IL-6: Acute phase - Immediate response coordination
    /// Triggers: PreToolUse blocking, urgent actions
    Il6,

    /// IL-10: Suppression - Dampen excessive response
    /// Triggers: Rate limiting, cooldown periods
    Il10,

    /// TNF-α: Destruction - Terminate threats
    /// Triggers: Block verdicts, process termination
    TnfAlpha,

    /// IFN-γ: Activation - Enhance response capability
    /// Triggers: Skill amplification, escalation
    IfnGamma,

    /// TGF-β: Regulation - Modulate behavior
    /// Triggers: Configuration changes, adaptation
    TgfBeta,

    /// CSF: Colony Stimulating - Create new agents
    /// Triggers: Subagent spawning, hook instantiation
    Csf,

    /// Custom family for extension
    Custom(u16),
}

impl fmt::Display for CytokineFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Il1 => write!(f, "IL-1"),
            Self::Il2 => write!(f, "IL-2"),
            Self::Il6 => write!(f, "IL-6"),
            Self::Il10 => write!(f, "IL-10"),
            Self::TnfAlpha => write!(f, "TNF-α"),
            Self::IfnGamma => write!(f, "IFN-γ"),
            Self::TgfBeta => write!(f, "TGF-β"),
            Self::Csf => write!(f, "CSF"),
            Self::Custom(id) => write!(f, "Custom-{id}"),
        }
    }
}

impl CytokineFamily {
    /// Get the T1 primitive this family primarily maps to
    pub fn primary_primitive(&self) -> &'static str {
        match self {
            Self::Il1 => "∃ (existence - threat detected)",
            Self::Il2 => "ρ (recursion - spawn more)",
            Self::Il6 => "→ (causality - immediate action)",
            Self::Il10 => "∂ (boundary - suppress/limit)",
            Self::TnfAlpha => "∅ (void - terminate)",
            Self::IfnGamma => "N (quantity - amplify)",
            Self::TgfBeta => "ς (state - transform)",
            Self::Csf => "μ (mapping - create new)",
            Self::Custom(_) => "Σ (sum - extension)",
        }
    }

    /// Is this a pro-inflammatory (activating) cytokine?
    pub fn is_activating(&self) -> bool {
        matches!(
            self,
            Self::Il1 | Self::Il2 | Self::Il6 | Self::TnfAlpha | Self::IfnGamma | Self::Csf
        )
    }

    /// Is this an anti-inflammatory (suppressing) cytokine?
    pub fn is_suppressing(&self) -> bool {
        matches!(self, Self::Il10 | Self::TgfBeta)
    }
}

/// Ordinal escalation scale for inter-crate signal threat detection.
///
/// # Tier: T2-P
/// Grounds to: N (quantity)
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ThreatLevel {
    /// Trace level - minimal signal
    Trace = 0,
    /// Low - background noise
    Low = 1,
    /// Medium - notable
    #[default]
    Medium = 2,
    /// High - requires attention
    High = 3,
    /// Critical - immediate action required
    Critical = 4,
}

/// Backward-compatible alias.
#[deprecated(note = "use ThreatLevel — F2 equivocation fix")]
pub type Severity = ThreatLevel;

impl fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace => write!(f, "trace"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Signal scope - how far does the signal travel?
///
/// # Tier: T2-P
/// Grounds to: λ (location)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Autocrine: Signal affects only the sender
    Autocrine,
    /// Paracrine: Signal affects nearby components (same file/module)
    #[default]
    Paracrine,
    /// Endocrine: Signal affects distant components (cross-module)
    Endocrine,
    /// Systemic: Signal affects entire system (session-wide)
    Systemic,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Autocrine => write!(f, "autocrine"),
            Self::Paracrine => write!(f, "paracrine"),
            Self::Endocrine => write!(f, "endocrine"),
            Self::Systemic => write!(f, "systemic"),
        }
    }
}

/// A cytokine signal with typed payload.
///
/// # Tier: T2-C (Composite)
/// Grounds to: ς (state) via payload, → (causality) via emission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cytokine {
    /// Unique signal ID
    pub id: String,

    /// Cytokine family (determines behavior)
    pub family: CytokineFamily,

    /// Signal name/type within family
    pub name: String,

    /// Signal severity/concentration
    pub severity: ThreatLevel,

    /// Signal scope (how far it travels)
    pub scope: Scope,

    /// JSON payload (flexible data)
    pub payload: serde_json::Value,

    /// Emission timestamp
    pub emitted_at: DateTime,

    /// Source component that emitted
    pub source: Option<String>,

    /// Target component (if directed)
    pub target: Option<String>,

    /// TTL in seconds (0 = no expiry)
    pub ttl_secs: u32,

    /// Can this signal trigger cascades?
    pub cascadable: bool,
}

impl Cytokine {
    /// Create a new cytokine signal
    pub fn new(family: CytokineFamily, name: impl Into<String>) -> Self {
        Self {
            id: format!("{}-{}", family, uuid_v4_simple()),
            family,
            name: name.into(),
            severity: ThreatLevel::default(),
            scope: Scope::default(),
            payload: serde_json::Value::Null,
            emitted_at: DateTime::now(),
            source: None,
            target: None,
            ttl_secs: 300, // 5 minute default
            cascadable: true,
        }
    }

    /// Set severity
    #[must_use]
    pub fn with_severity(mut self, severity: ThreatLevel) -> Self {
        self.severity = severity;
        self
    }

    /// Set scope
    #[must_use]
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Set payload
    #[must_use]
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Set source
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set target
    #[must_use]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set TTL
    #[must_use]
    pub fn with_ttl(mut self, secs: u32) -> Self {
        self.ttl_secs = secs;
        self
    }

    /// Disable cascade triggering
    #[must_use]
    pub fn no_cascade(mut self) -> Self {
        self.cascadable = false;
        self
    }

    /// Check if signal has expired
    pub fn is_expired(&self) -> bool {
        if self.ttl_secs == 0 {
            return false;
        }
        let elapsed = DateTime::now()
            .signed_duration_since(self.emitted_at)
            .num_seconds();
        elapsed > i64::from(self.ttl_secs)
    }

    /// Create an IL-1 alarm signal
    pub fn alarm(name: impl Into<String>) -> Self {
        Self::new(CytokineFamily::Il1, name)
            .with_severity(ThreatLevel::High)
            .with_scope(Scope::Systemic)
    }

    /// Create a TNF-α termination signal
    pub fn terminate(name: impl Into<String>) -> Self {
        Self::new(CytokineFamily::TnfAlpha, name)
            .with_severity(ThreatLevel::Critical)
            .with_scope(Scope::Endocrine)
    }

    /// Create an IL-10 suppression signal
    pub fn suppress(name: impl Into<String>) -> Self {
        Self::new(CytokineFamily::Il10, name)
            .with_severity(ThreatLevel::Medium)
            .with_scope(Scope::Paracrine)
    }

    /// Create an IFN-γ amplification signal
    pub fn amplify(name: impl Into<String>) -> Self {
        Self::new(CytokineFamily::IfnGamma, name)
            .with_severity(ThreatLevel::High)
            .with_scope(Scope::Endocrine)
    }
}

/// Generate a simple UUID v4-like string (no external dep)
fn uuid_v4_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{now:016x}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_cytokine_family_display() {
        assert_eq!(CytokineFamily::Il1.to_string(), "IL-1");
        assert_eq!(CytokineFamily::TnfAlpha.to_string(), "TNF-α");
        assert_eq!(CytokineFamily::Custom(42).to_string(), "Custom-42");
    }

    #[test]
    fn test_cytokine_family_classification() {
        assert!(CytokineFamily::Il1.is_activating());
        assert!(CytokineFamily::Il10.is_suppressing());
        assert!(!CytokineFamily::Il1.is_suppressing());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ThreatLevel::Critical > ThreatLevel::High);
        assert!(ThreatLevel::High > ThreatLevel::Medium);
        assert!(ThreatLevel::Medium > ThreatLevel::Low);
        assert!(ThreatLevel::Low > ThreatLevel::Trace);
    }

    #[test]
    fn test_cytokine_creation() {
        let signal = Cytokine::new(CytokineFamily::Il1, "threat_detected")
            .with_severity(ThreatLevel::High)
            .with_source("guardian");

        assert_eq!(signal.family, CytokineFamily::Il1);
        assert_eq!(signal.name, "threat_detected");
        assert_eq!(signal.severity, ThreatLevel::High);
        assert_eq!(signal.source, Some("guardian".to_string()));
    }

    #[test]
    fn test_cytokine_expiry() {
        let signal = Cytokine::new(CytokineFamily::Il1, "test").with_ttl(0); // No expiry

        assert!(!signal.is_expired());
    }

    #[test]
    fn test_convenience_constructors() {
        let alarm = Cytokine::alarm("intrusion");
        assert_eq!(alarm.family, CytokineFamily::Il1);
        assert_eq!(alarm.severity, ThreatLevel::High);
        assert_eq!(alarm.scope, Scope::Systemic);

        let term = Cytokine::terminate("bad_process");
        assert_eq!(term.family, CytokineFamily::TnfAlpha);
        assert_eq!(term.severity, ThreatLevel::Critical);
    }
}
