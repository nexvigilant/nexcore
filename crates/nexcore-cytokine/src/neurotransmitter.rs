//! # Neurotransmitter Model for Hook Outcomes
//!
//! Maps Claude Code hook exit codes to biological neurotransmitter types
//! per Biological Alignment v2.0 §5 (Nervous System).
//!
//! ## Biological Mapping
//!
//! | Hook Outcome | Exit Code | Neurotransmitter | Effect |
//! |-------------|-----------|------------------|--------|
//! | Pass/Allow  | 0         | ACh (Acetylcholine) | Excitatory — action proceeds |
//! | Warn/Modify | 1         | Dopamine | Modulatory — action proceeds with adjustment |
//! | Block/Deny  | 2         | GABA | Inhibitory — action prevented |
//!
//! ## Key Property: Synaptic Gap
//!
//! Just like biological neurons communicate across a synaptic gap (never
//! direct contact), hooks communicate via exit codes and stdout — never
//! by sharing memory. This is the fundamental isolation mechanism.

use serde::{Deserialize, Serialize};

// ============================================================================
// Neurotransmitter — The signal molecule type
// ============================================================================

/// Neurotransmitter type modeling hook outcomes.
///
/// Each hook outcome maps to a biological neurotransmitter:
/// - ACh (Acetylcholine): Excitatory — allows the action to proceed
/// - Dopamine: Modulatory — adjusts parameters, adds warnings
/// - GABA: Inhibitory — blocks the action entirely
///
/// ## Tier: T2-P (Σ sum + → causality), dominant Σ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Neurotransmitter {
    /// Acetylcholine — excitatory. Hook exit 0 (pass).
    /// Action proceeds unmodified.
    Acetylcholine,
    /// Dopamine — modulatory. Hook exit 1 (warn).
    /// Action proceeds but with modifications/warnings.
    Dopamine,
    /// GABA (gamma-aminobutyric acid) — inhibitory. Hook exit 2 (block).
    /// Action is prevented entirely.
    Gaba,
}

impl Neurotransmitter {
    /// Create from hook exit code.
    ///
    /// - 0 → ACh (excitatory, pass)
    /// - 1 → Dopamine (modulatory, warn)
    /// - 2+ → GABA (inhibitory, block)
    #[must_use]
    pub fn from_exit_code(code: i32) -> Self {
        match code {
            0 => Self::Acetylcholine,
            1 => Self::Dopamine,
            _ => Self::Gaba,
        }
    }

    /// Whether this neurotransmitter allows the action to proceed.
    #[must_use]
    pub fn allows_action(&self) -> bool {
        matches!(self, Self::Acetylcholine | Self::Dopamine)
    }

    /// Whether this neurotransmitter blocks the action.
    #[must_use]
    pub fn blocks_action(&self) -> bool {
        matches!(self, Self::Gaba)
    }

    /// Whether this neurotransmitter modifies the action.
    #[must_use]
    pub fn modifies_action(&self) -> bool {
        matches!(self, Self::Dopamine)
    }

    /// The biological effect classification.
    #[must_use]
    pub fn effect(&self) -> NeuralEffect {
        match self {
            Self::Acetylcholine => NeuralEffect::Excitatory,
            Self::Dopamine => NeuralEffect::Modulatory,
            Self::Gaba => NeuralEffect::Inhibitory,
        }
    }

    /// The hook exit code this maps to.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Acetylcholine => 0,
            Self::Dopamine => 1,
            Self::Gaba => 2,
        }
    }
}

impl core::fmt::Display for Neurotransmitter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Acetylcholine => write!(f, "ACh"),
            Self::Dopamine => write!(f, "DA"),
            Self::Gaba => write!(f, "GABA"),
        }
    }
}

// ============================================================================
// NeuralEffect — Classification of neurotransmitter effect
// ============================================================================

/// Classification of neural signal effect.
///
/// ## Tier: T2-P (Σ sum + κ comparison), dominant Σ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeuralEffect {
    /// Increases likelihood of action (ACh, glutamate).
    Excitatory,
    /// Decreases likelihood of action (GABA, glycine).
    Inhibitory,
    /// Adjusts action parameters without binary allow/deny (dopamine, serotonin).
    Modulatory,
}

impl core::fmt::Display for NeuralEffect {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Excitatory => write!(f, "excitatory"),
            Self::Inhibitory => write!(f, "inhibitory"),
            Self::Modulatory => write!(f, "modulatory"),
        }
    }
}

// ============================================================================
// HookNeuralProfile — Per-hook neurotransmitter statistics
// ============================================================================

/// Neural profile of a hook: tracks its neurotransmitter output distribution.
///
/// Like measuring neurotransmitter concentrations at a synapse.
/// A healthy hook should have a dominant neurotransmitter profile that
/// matches its intended function.
///
/// ## Tier: T2-C (ν frequency + N quantity + κ comparison), dominant ν
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookNeuralProfile {
    /// Hook name or ID.
    pub hook_name: String,
    /// Hook event type (e.g., "PreToolUse", "PostToolUse").
    pub hook_event: String,
    /// Number of ACh emissions (pass/allow).
    pub ach_count: u64,
    /// Number of Dopamine emissions (warn/modify).
    pub dopamine_count: u64,
    /// Number of GABA emissions (block/deny).
    pub gaba_count: u64,
}

impl HookNeuralProfile {
    /// Create a new neural profile for a hook.
    #[must_use]
    pub fn new(hook_name: impl Into<String>, hook_event: impl Into<String>) -> Self {
        Self {
            hook_name: hook_name.into(),
            hook_event: hook_event.into(),
            ach_count: 0,
            dopamine_count: 0,
            gaba_count: 0,
        }
    }

    /// Record a neurotransmitter emission from this hook.
    pub fn record(&mut self, nt: Neurotransmitter) {
        match nt {
            Neurotransmitter::Acetylcholine => self.ach_count += 1,
            Neurotransmitter::Dopamine => self.dopamine_count += 1,
            Neurotransmitter::Gaba => self.gaba_count += 1,
        }
    }

    /// Record from a hook exit code.
    pub fn record_exit_code(&mut self, code: i32) {
        self.record(Neurotransmitter::from_exit_code(code));
    }

    /// Total emissions from this hook.
    #[must_use]
    pub fn total_emissions(&self) -> u64 {
        self.ach_count + self.dopamine_count + self.gaba_count
    }

    /// The dominant neurotransmitter (most frequently emitted).
    ///
    /// Returns None if no emissions have occurred.
    #[must_use]
    pub fn dominant(&self) -> Option<Neurotransmitter> {
        if self.total_emissions() == 0 {
            return None;
        }
        if self.gaba_count >= self.dopamine_count && self.gaba_count >= self.ach_count {
            Some(Neurotransmitter::Gaba)
        } else if self.dopamine_count >= self.ach_count {
            Some(Neurotransmitter::Dopamine)
        } else {
            Some(Neurotransmitter::Acetylcholine)
        }
    }

    /// Inhibition ratio: GABA / total.
    ///
    /// A high inhibition ratio (>30%) may indicate the hook is too aggressive.
    #[must_use]
    pub fn inhibition_ratio(&self) -> f64 {
        let total = self.total_emissions();
        if total == 0 {
            return 0.0;
        }
        self.gaba_count as f64 / total as f64
    }

    /// Excitation ratio: ACh / total.
    #[must_use]
    pub fn excitation_ratio(&self) -> f64 {
        let total = self.total_emissions();
        if total == 0 {
            return 0.0;
        }
        self.ach_count as f64 / total as f64
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_exit_code() {
        assert_eq!(
            Neurotransmitter::from_exit_code(0),
            Neurotransmitter::Acetylcholine
        );
        assert_eq!(
            Neurotransmitter::from_exit_code(1),
            Neurotransmitter::Dopamine
        );
        assert_eq!(Neurotransmitter::from_exit_code(2), Neurotransmitter::Gaba);
        assert_eq!(
            Neurotransmitter::from_exit_code(127),
            Neurotransmitter::Gaba
        );
    }

    #[test]
    fn test_allows_action() {
        assert!(Neurotransmitter::Acetylcholine.allows_action());
        assert!(Neurotransmitter::Dopamine.allows_action());
        assert!(!Neurotransmitter::Gaba.allows_action());
    }

    #[test]
    fn test_blocks_action() {
        assert!(!Neurotransmitter::Acetylcholine.blocks_action());
        assert!(!Neurotransmitter::Dopamine.blocks_action());
        assert!(Neurotransmitter::Gaba.blocks_action());
    }

    #[test]
    fn test_modifies_action() {
        assert!(!Neurotransmitter::Acetylcholine.modifies_action());
        assert!(Neurotransmitter::Dopamine.modifies_action());
        assert!(!Neurotransmitter::Gaba.modifies_action());
    }

    #[test]
    fn test_effect_classification() {
        assert_eq!(
            Neurotransmitter::Acetylcholine.effect(),
            NeuralEffect::Excitatory
        );
        assert_eq!(
            Neurotransmitter::Dopamine.effect(),
            NeuralEffect::Modulatory
        );
        assert_eq!(Neurotransmitter::Gaba.effect(), NeuralEffect::Inhibitory);
    }

    #[test]
    fn test_exit_code_roundtrip() {
        for code in [0, 1, 2] {
            let nt = Neurotransmitter::from_exit_code(code);
            assert_eq!(nt.exit_code(), code);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(Neurotransmitter::Acetylcholine.to_string(), "ACh");
        assert_eq!(Neurotransmitter::Dopamine.to_string(), "DA");
        assert_eq!(Neurotransmitter::Gaba.to_string(), "GABA");
    }

    #[test]
    fn test_neural_effect_display() {
        assert_eq!(NeuralEffect::Excitatory.to_string(), "excitatory");
        assert_eq!(NeuralEffect::Inhibitory.to_string(), "inhibitory");
        assert_eq!(NeuralEffect::Modulatory.to_string(), "modulatory");
    }

    #[test]
    fn test_hook_profile_new() {
        let profile = HookNeuralProfile::new("pretool-dispatcher", "PreToolUse");
        assert_eq!(profile.hook_name, "pretool-dispatcher");
        assert_eq!(profile.total_emissions(), 0);
        assert!(profile.dominant().is_none());
    }

    #[test]
    fn test_hook_profile_record() {
        let mut profile = HookNeuralProfile::new("test-hook", "PreToolUse");
        profile.record(Neurotransmitter::Acetylcholine);
        profile.record(Neurotransmitter::Acetylcholine);
        profile.record(Neurotransmitter::Gaba);

        assert_eq!(profile.total_emissions(), 3);
        assert_eq!(profile.ach_count, 2);
        assert_eq!(profile.gaba_count, 1);
        assert_eq!(profile.dominant(), Some(Neurotransmitter::Acetylcholine));
    }

    #[test]
    fn test_hook_profile_record_exit_code() {
        let mut profile = HookNeuralProfile::new("test-hook", "PreToolUse");
        profile.record_exit_code(0); // ACh
        profile.record_exit_code(0); // ACh
        profile.record_exit_code(1); // Dopamine
        profile.record_exit_code(2); // GABA

        assert_eq!(profile.ach_count, 2);
        assert_eq!(profile.dopamine_count, 1);
        assert_eq!(profile.gaba_count, 1);
    }

    #[test]
    fn test_hook_profile_inhibition_ratio() {
        let mut profile = HookNeuralProfile::new("test-hook", "PreToolUse");
        profile.ach_count = 70;
        profile.dopamine_count = 10;
        profile.gaba_count = 20;

        assert!((profile.inhibition_ratio() - 0.2).abs() < f64::EPSILON);
        assert!((profile.excitation_ratio() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hook_profile_inhibition_ratio_empty() {
        let profile = HookNeuralProfile::new("test-hook", "PreToolUse");
        assert!((profile.inhibition_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hook_profile_dominant_gaba() {
        let mut profile = HookNeuralProfile::new("python-blocker", "PreToolUse");
        profile.ach_count = 5;
        profile.gaba_count = 95;
        assert_eq!(profile.dominant(), Some(Neurotransmitter::Gaba));
        assert!(profile.inhibition_ratio() > 0.90);
    }

    #[test]
    fn test_serde_roundtrip() {
        let nt = Neurotransmitter::Dopamine;
        let json = serde_json::to_string(&nt).unwrap_or_default();
        let back: Neurotransmitter = serde_json::from_str(&json).unwrap_or(Neurotransmitter::Gaba);
        assert_eq!(back, nt);
    }
}
