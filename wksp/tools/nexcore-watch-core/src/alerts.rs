#![allow(dead_code)]
//! Alert severity classification for wearable notifications.
//!
//! ## Primitive Grounding
//! - ∂ (Boundary): P0-P5 priority thresholds
//! - → (Causality): priority → response time → haptic pattern
//! - ∝ (Irreversibility): P0 patient safety events cannot be undone
//! - κ (Comparison): priority ordering (P0 > P1 > ... > P5)
//! - μ (Mapping): priority → vibration, priority → screen wake
//! - ν (Frequency): ICH E2A response time deadlines
//!
//! ## Tier: T2-C (∂ + → + ∝ + κ + μ + ν)
//!
//! ## Grammar: Type-3 (regular)
//! Priority levels form a total order: P0 < P1 < P2 < P3 < P4 < P5
//! Each level maps deterministically to response parameters.

use serde::{Deserialize, Serialize};

/// P0-P5 priority levels from NexVigilant patient safety hierarchy.
///
/// ## Primitive Grounding
/// - ∂ (Boundary): each level is a threshold boundary
/// - κ (Comparison): PartialOrd/Ord ordering
/// - ∝ (Irreversibility): P0 events are irreversible harm
///
/// ## Tier: T2-P (∂ + κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    /// P0: Patient Safety — supreme directive. Vibrate + persistent notification.
    /// ∝ (Irreversibility): fatal/life-threatening harm
    P0PatientSafety = 0,
    /// P1: Signal Integrity — no signal lost or downgraded. Strong vibrate.
    /// → (Causality): signal loss causes downstream harm
    P1SignalIntegrity = 1,
    /// P2: Regulatory Compliance — ICH E2A timelines. Vibrate.
    /// ν (Frequency): time-bounded regulatory deadlines
    P2Regulatory = 2,
    /// P3: Data Quality. Gentle notification.
    /// κ (Comparison): data validation checks
    P3DataQuality = 3,
    /// P4: Operational Efficiency. Silent notification.
    /// N (Quantity): throughput metrics
    P4Operational = 4,
    /// P5: Cost Optimization. Badge only.
    /// ∅ (Void): lowest priority, no haptic response
    P5Cost = 5,
}

/// Alert payload for watch notification.
///
/// ## Primitive Grounding
/// - ∂ (Boundary): level determines response
/// - λ (Location): source identifies origin system
/// - ν (Frequency): timestamp_ms for ordering
/// - ∃ (Existence): action_url presence = actionable
///
/// ## Tier: T3 (∂ + λ + ν + ∃) — domain-specific alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Priority level — ∂ (Boundary)
    pub level: AlertLevel,
    /// Short title for notification — μ (Mapping)
    pub title: String,
    /// Detailed body text — σ (Sequence)
    pub body: String,
    /// Originating system — λ (Location)
    pub source: String,
    /// Creation timestamp (epoch ms) — ν (Frequency)
    pub timestamp_ms: u64,
    /// Optional action URL — ∃ (Existence): Some = actionable
    pub action_url: Option<String>,
}

impl AlertLevel {
    /// Maximum response time for this priority level (ICH E2A).
    ///
    /// ## Primitive: ν (Frequency) + ∂ (Boundary)
    /// ## Tier: T2-P
    ///
    /// - P0: 0h (IMMEDIATE) — Fatal/life-threatening
    /// - P1: 4h — Signal integrity
    /// - P2: 24h — Regulatory
    /// - P3: 72h — Data quality
    /// - P4: 720h (30d) — Operational
    /// - P5: None — No deadline
    pub fn max_response_hours(&self) -> Option<u32> {
        match self {
            Self::P0PatientSafety => Some(0),
            Self::P1SignalIntegrity => Some(4),
            Self::P2Regulatory => Some(24),
            Self::P3DataQuality => Some(72),
            Self::P4Operational => Some(720),
            Self::P5Cost => None, // ∅ (Void): no deadline
        }
    }

    /// Vibration pattern index for Galaxy Watch7 haptics.
    ///
    /// ## Primitive: μ (Mapping)
    /// ## Tier: T1
    ///
    /// 0=none, 1=gentle, 2=standard, 3=strong, 4=persistent
    pub fn vibration_pattern(&self) -> u8 {
        match self {
            Self::P0PatientSafety => 4,   // → persistent: P0 demands attention
            Self::P1SignalIntegrity => 3, // → strong
            Self::P2Regulatory => 2,      // → standard
            Self::P3DataQuality => 1,     // → gentle
            Self::P4Operational => 0,     // ∅: silent
            Self::P5Cost => 0,            // ∅: silent
        }
    }

    /// Whether this alert should wake the watch screen.
    ///
    /// ## Primitive: κ (Comparison) + ∂ (Boundary)
    /// ## Tier: T2-P
    ///
    /// Only P0-P2 wake the screen. P3-P5 are passive.
    pub fn wakes_screen(&self) -> bool {
        matches!(
            self,
            Self::P0PatientSafety | Self::P1SignalIntegrity | Self::P2Regulatory
        )
    }

    /// Whether this level represents patient harm risk.
    ///
    /// ## Primitive: ∝ (Irreversibility) + ∂ (Boundary)
    /// ## Tier: T2-P
    ///
    /// P0 and P1 involve direct or indirect harm potential.
    pub fn is_harm_related(&self) -> bool {
        matches!(self, Self::P0PatientSafety | Self::P1SignalIntegrity)
    }

    /// Priority code as integer.
    ///
    /// ## Primitive: μ (Mapping) — enum → u8
    /// ## Tier: T1
    pub fn code(&self) -> u8 {
        *self as u8
    }
}

impl Alert {
    /// Format for watch notification (short form for small display).
    ///
    /// ## Primitive: μ (Mapping) + σ (Sequence)
    /// ## Tier: T2-P — concatenated display string
    pub fn short_display(&self) -> String {
        format!("[P{}] {}", self.level as u8, self.title)
    }

    /// Whether this alert is actionable (has a URL).
    ///
    /// ## Primitive: ∃ (Existence)
    /// ## Tier: T1
    pub fn is_actionable(&self) -> bool {
        self.action_url.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════
    // Response Time Tests — ν (Frequency) + ∂ (Boundary)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn p0_immediate_response() {
        assert_eq!(AlertLevel::P0PatientSafety.max_response_hours(), Some(0));
    }

    #[test]
    fn p1_four_hour_response() {
        assert_eq!(AlertLevel::P1SignalIntegrity.max_response_hours(), Some(4));
    }

    #[test]
    fn p2_twenty_four_hour_response() {
        assert_eq!(AlertLevel::P2Regulatory.max_response_hours(), Some(24));
    }

    #[test]
    fn p3_seventy_two_hour_response() {
        assert_eq!(AlertLevel::P3DataQuality.max_response_hours(), Some(72));
    }

    #[test]
    fn p4_thirty_day_response() {
        assert_eq!(AlertLevel::P4Operational.max_response_hours(), Some(720));
    }

    #[test]
    fn p5_no_deadline() {
        // ∅ (Void): no response deadline
        assert_eq!(AlertLevel::P5Cost.max_response_hours(), None);
    }

    // ═══════════════════════════════════════════════════════════
    // Vibration Pattern Tests — μ (Mapping)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn vibration_monotonically_decreases() {
        // Higher priority → stronger vibration
        assert!(
            AlertLevel::P0PatientSafety.vibration_pattern()
                > AlertLevel::P1SignalIntegrity.vibration_pattern()
        );
        assert!(
            AlertLevel::P1SignalIntegrity.vibration_pattern()
                > AlertLevel::P2Regulatory.vibration_pattern()
        );
        assert!(
            AlertLevel::P2Regulatory.vibration_pattern()
                > AlertLevel::P3DataQuality.vibration_pattern()
        );
        assert!(
            AlertLevel::P3DataQuality.vibration_pattern()
                > AlertLevel::P4Operational.vibration_pattern()
        );
    }

    #[test]
    fn p0_persistent_vibration() {
        assert_eq!(AlertLevel::P0PatientSafety.vibration_pattern(), 4);
    }

    #[test]
    fn p5_no_vibration() {
        assert_eq!(AlertLevel::P5Cost.vibration_pattern(), 0);
    }

    // ═══════════════════════════════════════════════════════════
    // Screen Wake Tests — ∂ (Boundary)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn p0_p1_p2_wake_screen() {
        assert!(AlertLevel::P0PatientSafety.wakes_screen());
        assert!(AlertLevel::P1SignalIntegrity.wakes_screen());
        assert!(AlertLevel::P2Regulatory.wakes_screen());
    }

    #[test]
    fn p3_p4_p5_dont_wake_screen() {
        assert!(!AlertLevel::P3DataQuality.wakes_screen());
        assert!(!AlertLevel::P4Operational.wakes_screen());
        assert!(!AlertLevel::P5Cost.wakes_screen());
    }

    // ═══════════════════════════════════════════════════════════
    // Harm Tests — ∝ (Irreversibility)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn p0_p1_harm_related() {
        assert!(AlertLevel::P0PatientSafety.is_harm_related());
        assert!(AlertLevel::P1SignalIntegrity.is_harm_related());
    }

    #[test]
    fn p2_through_p5_not_harm_related() {
        assert!(!AlertLevel::P2Regulatory.is_harm_related());
        assert!(!AlertLevel::P3DataQuality.is_harm_related());
        assert!(!AlertLevel::P4Operational.is_harm_related());
        assert!(!AlertLevel::P5Cost.is_harm_related());
    }

    // ═══════════════════════════════════════════════════════════
    // Code Tests — μ (Mapping)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn codes_sequential() {
        assert_eq!(AlertLevel::P0PatientSafety.code(), 0);
        assert_eq!(AlertLevel::P1SignalIntegrity.code(), 1);
        assert_eq!(AlertLevel::P2Regulatory.code(), 2);
        assert_eq!(AlertLevel::P3DataQuality.code(), 3);
        assert_eq!(AlertLevel::P4Operational.code(), 4);
        assert_eq!(AlertLevel::P5Cost.code(), 5);
    }

    // ═══════════════════════════════════════════════════════════
    // Ordering Tests — κ (Comparison)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn priority_ordering() {
        // P0 < P1 < P2 < ... (lower = higher priority)
        assert!(AlertLevel::P0PatientSafety < AlertLevel::P1SignalIntegrity);
        assert!(AlertLevel::P1SignalIntegrity < AlertLevel::P2Regulatory);
        assert!(AlertLevel::P4Operational < AlertLevel::P5Cost);
    }

    // ═══════════════════════════════════════════════════════════
    // Alert Tests — T3 domain
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn short_display_format() {
        let alert = Alert {
            level: AlertLevel::P0PatientSafety,
            title: "Fatal AE".to_string(),
            body: "Details".to_string(),
            source: "Guardian".to_string(),
            timestamp_ms: 0,
            action_url: None,
        };
        assert_eq!(alert.short_display(), "[P0] Fatal AE");
    }

    #[test]
    fn actionable_with_url() {
        let alert = Alert {
            level: AlertLevel::P2Regulatory,
            title: "Report Due".to_string(),
            body: "".to_string(),
            source: "".to_string(),
            timestamp_ms: 0,
            action_url: Some("https://nexvigilant.com/report/123".to_string()),
        };
        assert!(alert.is_actionable());
    }

    #[test]
    fn not_actionable_without_url() {
        let alert = Alert {
            level: AlertLevel::P5Cost,
            title: "Budget update".to_string(),
            body: "".to_string(),
            source: "".to_string(),
            timestamp_ms: 0,
            action_url: None, // ∅ (Void)
        };
        assert!(!alert.is_actionable());
    }

    // ═══════════════════════════════════════════════════════════
    // Serialization Tests — μ (Mapping) + σ (Sequence)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn alert_json_roundtrip() {
        let original = Alert {
            level: AlertLevel::P1SignalIntegrity,
            title: "Signal Lost".to_string(),
            body: "PRR dropped below threshold".to_string(),
            source: "SignalEngine".to_string(),
            timestamp_ms: 1707300000000,
            action_url: Some("https://nexvigilant.com/signal/456".to_string()),
        };
        let json = serde_json::to_string(&original);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        let parsed: Result<Alert, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok());
        let restored = parsed.unwrap_or_else(|_| Alert {
            level: AlertLevel::P5Cost,
            title: String::new(),
            body: String::new(),
            source: String::new(),
            timestamp_ms: 0,
            action_url: None,
        });
        assert_eq!(restored.level, AlertLevel::P1SignalIntegrity);
        assert_eq!(restored.title, "Signal Lost");
        assert!(restored.is_actionable());
    }
}
