#![allow(dead_code)]
//! Guardian status for wearable display.
//!
//! ## Primitive Grounding
//! - ς (State): Guardian homeostasis loop state (Nominal|Elevated|Alert|Critical)
//! - ∂ (Boundary): state transitions at risk thresholds
//! - → (Causality): state → color → label → haptic response
//! - μ (Mapping): state → color hex, state → label
//! - κ (Comparison): risk level ordering (Low < Medium < High < Critical)
//!
//! ## Tier: T3 (ς + ∂ + → + μ + κ) — domain-specific Guardian display
//!
//! ## Grammar: Type-3 (regular)
//! States form a finite automaton: Nominal ↔ Elevated ↔ Alert ↔ Critical
//! Transitions driven by sensor input. No recursion needed.

use serde::{Deserialize, Serialize};

/// Guardian homeostasis loop state for watch display.
///
/// ## Primitive Grounding
/// - ς (State): loop state
/// - N (Quantity): iteration count, sensor/actuator counts
/// - ν (Frequency): last_tick_ms (timing)
/// - κ (Comparison): risk_level ordering
///
/// ## Tier: T3 (ς + N + ν + κ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianStatus {
    /// Current homeostasis state — ς (State)
    pub state: GuardianState,
    /// Loop iteration counter — N (Quantity)
    pub iteration: u64,
    /// Number of active sensors — N (Quantity)
    pub active_sensors: u32,
    /// Number of active actuators — N (Quantity)
    pub active_actuators: u32,
    /// Current risk assessment — κ (Comparison)
    pub risk_level: RiskLevel,
    /// Last tick timestamp (ms) — ν (Frequency)
    pub last_tick_ms: u64,
}

/// Guardian state — the core ς (State) primitive.
///
/// ## Tier: T1 (ς)
///
/// Four-state finite automaton:
/// ```text
/// Nominal ↔ Elevated ↔ Alert ↔ Critical
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardianState {
    /// ∂ (Boundary): below all thresholds
    Nominal,
    /// ∂ (Boundary): first threshold crossed
    Elevated,
    /// ∂ (Boundary): second threshold crossed
    Alert,
    /// ∂ (Boundary): all thresholds exceeded — P0 response
    Critical,
}

/// Risk level — κ (Comparison) ordering.
///
/// ## Tier: T2-P (ς + κ)
///
/// Ordinal scale: Low < Medium < High < Critical
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl GuardianStatus {
    /// Create a nominal status for display.
    ///
    /// ## Primitive: ∅ (Void) → ς (State)
    /// Default/zero state — all counters at 0, state Nominal.
    pub fn nominal() -> Self {
        Self {
            state: GuardianState::Nominal,
            iteration: 0,
            active_sensors: 0,
            active_actuators: 0,
            risk_level: RiskLevel::Low,
            last_tick_ms: 0,
        }
    }

    /// Color code for watch face complication.
    ///
    /// ## Primitive: μ (Mapping)
    /// ## Tier: T1 — pure state → string mapping
    ///
    /// Colors match Android resource `colors.xml` exactly:
    /// - Nominal  → #4CAF50 (Green)
    /// - Elevated → #FF9800 (Orange)
    /// - Alert    → #FF5722 (Red-Orange)
    /// - Critical → #F44336 (Red)
    pub fn color_hex(&self) -> &'static str {
        match self.state {
            GuardianState::Nominal => "#4CAF50",
            GuardianState::Elevated => "#FF9800",
            GuardianState::Alert => "#FF5722",
            GuardianState::Critical => "#F44336",
        }
    }

    /// Short label for watch tile.
    ///
    /// ## Primitive: μ (Mapping)
    /// ## Tier: T1 — pure state → string mapping
    pub fn label(&self) -> &'static str {
        match self.state {
            GuardianState::Nominal => "NOMINAL",
            GuardianState::Elevated => "ELEVATED",
            GuardianState::Alert => "ALERT",
            GuardianState::Critical => "CRITICAL",
        }
    }

    /// JSON serialization for JNI bridge.
    ///
    /// ## Primitive: μ (Mapping) + σ (Sequence)
    /// ## Tier: T2-P — struct → JSON string transform
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl GuardianState {
    /// Ordinal value for complication RANGED_VALUE.
    ///
    /// ## Primitive: μ (Mapping) — state → [0, 3] ordinal
    /// ## Tier: T1
    pub fn ordinal(&self) -> u8 {
        match self {
            Self::Nominal => 0,
            Self::Elevated => 1,
            Self::Alert => 2,
            Self::Critical => 3,
        }
    }

    /// Whether this state requires immediate attention.
    ///
    /// ## Primitive: ∂ (Boundary) + κ (Comparison)
    /// ## Tier: T2-P
    ///
    /// Alert and Critical require P0/P1 response.
    pub fn requires_attention(&self) -> bool {
        matches!(self, Self::Alert | Self::Critical)
    }
}

impl RiskLevel {
    /// Risk as normalized float [0.0, 1.0] for complication gauge.
    ///
    /// ## Primitive: μ (Mapping) — ordinal → float
    /// ## Tier: T1
    pub fn as_fraction(&self) -> f32 {
        match self {
            Self::Low => 0.0,
            Self::Medium => 0.33,
            Self::High => 0.66,
            Self::Critical => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════
    // GuardianStatus Tests — ς (State)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn nominal_default_values() {
        let status = GuardianStatus::nominal();
        assert_eq!(status.state, GuardianState::Nominal);
        assert_eq!(status.iteration, 0);
        assert_eq!(status.active_sensors, 0);
        assert_eq!(status.active_actuators, 0);
        assert_eq!(status.risk_level, RiskLevel::Low);
        assert_eq!(status.last_tick_ms, 0);
    }

    // ═══════════════════════════════════════════════════════════
    // Color Mapping Tests — μ (Mapping)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn color_hex_nominal_green() {
        let status = GuardianStatus {
            state: GuardianState::Nominal,
            ..GuardianStatus::nominal()
        };
        assert_eq!(status.color_hex(), "#4CAF50");
    }

    #[test]
    fn color_hex_elevated_orange() {
        let status = GuardianStatus {
            state: GuardianState::Elevated,
            ..GuardianStatus::nominal()
        };
        assert_eq!(status.color_hex(), "#FF9800");
    }

    #[test]
    fn color_hex_alert_red_orange() {
        let status = GuardianStatus {
            state: GuardianState::Alert,
            ..GuardianStatus::nominal()
        };
        assert_eq!(status.color_hex(), "#FF5722");
    }

    #[test]
    fn color_hex_critical_red() {
        let status = GuardianStatus {
            state: GuardianState::Critical,
            ..GuardianStatus::nominal()
        };
        assert_eq!(status.color_hex(), "#F44336");
    }

    // ═══════════════════════════════════════════════════════════
    // Label Mapping Tests — μ (Mapping)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn label_all_states() {
        assert_eq!(
            GuardianStatus {
                state: GuardianState::Nominal,
                ..GuardianStatus::nominal()
            }
            .label(),
            "NOMINAL"
        );
        assert_eq!(
            GuardianStatus {
                state: GuardianState::Elevated,
                ..GuardianStatus::nominal()
            }
            .label(),
            "ELEVATED"
        );
        assert_eq!(
            GuardianStatus {
                state: GuardianState::Alert,
                ..GuardianStatus::nominal()
            }
            .label(),
            "ALERT"
        );
        assert_eq!(
            GuardianStatus {
                state: GuardianState::Critical,
                ..GuardianStatus::nominal()
            }
            .label(),
            "CRITICAL"
        );
    }

    // ═══════════════════════════════════════════════════════════
    // State Ordinal Tests — μ (Mapping)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn state_ordinals_monotonic() {
        assert_eq!(GuardianState::Nominal.ordinal(), 0);
        assert_eq!(GuardianState::Elevated.ordinal(), 1);
        assert_eq!(GuardianState::Alert.ordinal(), 2);
        assert_eq!(GuardianState::Critical.ordinal(), 3);
    }

    // ═══════════════════════════════════════════════════════════
    // Attention Tests — ∂ (Boundary)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn requires_attention_only_alert_and_critical() {
        assert!(!GuardianState::Nominal.requires_attention());
        assert!(!GuardianState::Elevated.requires_attention());
        assert!(GuardianState::Alert.requires_attention());
        assert!(GuardianState::Critical.requires_attention());
    }

    // ═══════════════════════════════════════════════════════════
    // RiskLevel Tests — κ (Comparison)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn risk_fractions_bounded() {
        assert!((RiskLevel::Low.as_fraction() - 0.0).abs() < f32::EPSILON);
        assert!((RiskLevel::Medium.as_fraction() - 0.33).abs() < 0.01);
        assert!((RiskLevel::High.as_fraction() - 0.66).abs() < 0.01);
        assert!((RiskLevel::Critical.as_fraction() - 1.0).abs() < f32::EPSILON);
    }

    // ═══════════════════════════════════════════════════════════
    // JSON Serialization Tests — μ (Mapping) + σ (Sequence)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn to_json_contains_state() {
        let status = GuardianStatus::nominal();
        let json = status.to_json();
        assert!(
            json.contains("\"state\":\"Nominal\""),
            "JSON should contain state: {json}"
        );
    }

    #[test]
    fn json_roundtrip() {
        let original = GuardianStatus {
            state: GuardianState::Alert,
            iteration: 42,
            active_sensors: 3,
            active_actuators: 2,
            risk_level: RiskLevel::High,
            last_tick_ms: 1234567890,
        };
        let json = original.to_json();
        let parsed: Result<GuardianStatus, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "JSON roundtrip should succeed");
        let restored = parsed.unwrap_or_else(|_| GuardianStatus::nominal());
        assert_eq!(restored.state, GuardianState::Alert);
        assert_eq!(restored.iteration, 42);
        assert_eq!(restored.active_sensors, 3);
        assert_eq!(restored.risk_level, RiskLevel::High);
    }
}
