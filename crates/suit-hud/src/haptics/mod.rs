//! Haptics — 7.4
//!
//! Tactile feedback subsystem: wrist/palm ERM motors for directional cues,
//! torso vibrotactile belt for spatial awareness, and force feedback
//! through exoskeleton joints.

use serde::{Deserialize, Serialize};

/// Body zone for haptic output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HapticZone {
    /// 7.4.1 Left wrist.
    WristLeft,
    /// 7.4.1 Right wrist.
    WristRight,
    /// 7.4.1 Left palm.
    PalmLeft,
    /// 7.4.1 Right palm.
    PalmRight,
    /// 7.4.2 Torso front.
    TorsoFront,
    /// 7.4.2 Torso back.
    TorsoBack,
    /// 7.4.2 Torso left.
    TorsoLeft,
    /// 7.4.2 Torso right.
    TorsoRight,
    /// 7.4.3 Left shoulder joint.
    ShoulderLeft,
    /// 7.4.3 Right shoulder joint.
    ShoulderRight,
    /// 7.4.3 Left elbow joint.
    ElbowLeft,
    /// 7.4.3 Right elbow joint.
    ElbowRight,
    /// 7.4.3 Left knee joint.
    KneeLeft,
    /// 7.4.3 Right knee joint.
    KneeRight,
}

impl HapticZone {
    /// Whether this zone uses ERM motors (7.4.1).
    #[must_use]
    pub fn is_erm(self) -> bool {
        matches!(
            self,
            Self::WristLeft | Self::WristRight | Self::PalmLeft | Self::PalmRight
        )
    }

    /// Whether this zone is part of the vibrotactile belt (7.4.2).
    #[must_use]
    pub fn is_belt(self) -> bool {
        matches!(
            self,
            Self::TorsoFront | Self::TorsoBack | Self::TorsoLeft | Self::TorsoRight
        )
    }

    /// Whether this zone has force feedback via exo joints (7.4.3).
    #[must_use]
    pub fn is_force_feedback(self) -> bool {
        matches!(
            self,
            Self::ShoulderLeft
                | Self::ShoulderRight
                | Self::ElbowLeft
                | Self::ElbowRight
                | Self::KneeLeft
                | Self::KneeRight
        )
    }
}

/// Tactile pattern for haptic output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TactilePattern {
    /// Single pulse at given intensity (0.0..1.0) and duration (ms).
    Pulse {
        /// Intensity 0.0..1.0.
        intensity: f32,
        /// Duration in milliseconds.
        duration_ms: u32,
    },
    /// Repeating buzz.
    Buzz {
        /// Intensity 0.0..1.0.
        intensity: f32,
        /// On time in ms.
        on_ms: u32,
        /// Off time in ms.
        off_ms: u32,
        /// Number of cycles (0 = continuous until cancelled).
        cycles: u32,
    },
    /// Escalating intensity ramp (alert pattern).
    Ramp {
        /// Start intensity.
        start: f32,
        /// End intensity.
        end: f32,
        /// Duration in ms.
        duration_ms: u32,
    },
    /// Directional sweep across belt zones (navigation cue).
    Sweep {
        /// Start zone.
        from: HapticZone,
        /// End zone.
        to: HapticZone,
        /// Duration in ms.
        duration_ms: u32,
        /// Intensity.
        intensity: f32,
    },
    /// 7.4.3 Force resistance at joint (Newtons).
    ForceResist {
        /// Resistance force in Newtons.
        force_n: f32,
        /// Duration in ms (0 = hold until released).
        duration_ms: u32,
    },
}

/// A haptic cue — what to play, where, and when.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticCue {
    /// Target zone(s).
    pub zones: Vec<HapticZone>,
    /// Pattern to play.
    pub pattern: TactilePattern,
    /// Priority (higher = preempts lower).
    pub priority: u8,
    /// Semantic label (e.g., "threat_left", "nav_turn_right", "impact_warning").
    pub label: String,
}

impl HapticCue {
    /// Create a simple pulse cue on one zone.
    #[must_use]
    pub fn pulse(zone: HapticZone, intensity: f32, duration_ms: u32, label: &str) -> Self {
        Self {
            zones: vec![zone],
            pattern: TactilePattern::Pulse {
                intensity,
                duration_ms,
            },
            priority: 1,
            label: label.to_string(),
        }
    }

    /// Create a threat alert — escalating ramp on torso belt.
    #[must_use]
    pub fn threat_alert(direction: HapticZone, severity: f32) -> Self {
        Self {
            zones: vec![direction],
            pattern: TactilePattern::Ramp {
                start: 0.2,
                end: severity.clamp(0.3, 1.0),
                duration_ms: 500,
            },
            priority: 8,
            label: "threat_alert".to_string(),
        }
    }

    /// Create a navigation cue — pulse on the side to turn toward.
    #[must_use]
    pub fn nav_cue(turn_right: bool) -> Self {
        let zone = if turn_right {
            HapticZone::WristRight
        } else {
            HapticZone::WristLeft
        };
        Self {
            zones: vec![zone],
            pattern: TactilePattern::Pulse {
                intensity: 0.5,
                duration_ms: 200,
            },
            priority: 3,
            label: if turn_right { "nav_right" } else { "nav_left" }.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_classification() {
        assert!(HapticZone::WristLeft.is_erm());
        assert!(!HapticZone::WristLeft.is_belt());
        assert!(!HapticZone::WristLeft.is_force_feedback());

        assert!(HapticZone::TorsoFront.is_belt());
        assert!(!HapticZone::TorsoFront.is_erm());

        assert!(HapticZone::ElbowLeft.is_force_feedback());
        assert!(!HapticZone::ElbowLeft.is_belt());
    }

    #[test]
    fn pulse_cue() {
        let cue = HapticCue::pulse(HapticZone::PalmRight, 0.8, 150, "confirmation");
        assert_eq!(cue.zones.len(), 1);
        assert_eq!(cue.label, "confirmation");
        match cue.pattern {
            TactilePattern::Pulse {
                intensity,
                duration_ms,
            } => {
                assert!((intensity - 0.8).abs() < 0.01);
                assert_eq!(duration_ms, 150);
            }
            _ => panic!("expected Pulse"),
        }
    }

    #[test]
    fn threat_alert_clamped() {
        let cue = HapticCue::threat_alert(HapticZone::TorsoLeft, 1.5);
        match cue.pattern {
            TactilePattern::Ramp { end, .. } => {
                assert!(end <= 1.0, "severity should be clamped");
            }
            _ => panic!("expected Ramp"),
        }
        assert_eq!(cue.priority, 8);
    }

    #[test]
    fn nav_cue_direction() {
        let right = HapticCue::nav_cue(true);
        assert_eq!(right.zones[0], HapticZone::WristRight);
        assert_eq!(right.label, "nav_right");

        let left = HapticCue::nav_cue(false);
        assert_eq!(left.zones[0], HapticZone::WristLeft);
    }

    #[test]
    fn haptic_cue_serializes() {
        let cue = HapticCue::threat_alert(HapticZone::TorsoBack, 0.9);
        let json = serde_json::to_string(&cue);
        assert!(json.is_ok());
    }

    #[test]
    fn all_zones_classified() {
        let zones = [
            HapticZone::WristLeft,
            HapticZone::WristRight,
            HapticZone::PalmLeft,
            HapticZone::PalmRight,
            HapticZone::TorsoFront,
            HapticZone::TorsoBack,
            HapticZone::TorsoLeft,
            HapticZone::TorsoRight,
            HapticZone::ShoulderLeft,
            HapticZone::ShoulderRight,
            HapticZone::ElbowLeft,
            HapticZone::ElbowRight,
            HapticZone::KneeLeft,
            HapticZone::KneeRight,
        ];
        for z in &zones {
            let classified = z.is_erm() || z.is_belt() || z.is_force_feedback();
            assert!(classified, "{z:?} has no classification");
        }
    }
}
