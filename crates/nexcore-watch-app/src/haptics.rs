#![allow(dead_code)]
//! NDK vibration for P0-P5 haptic patterns.
//!
//! ## Primitive Grounding
//! - μ (Mapping): AlertLevel → vibration pattern
//! - → (Causality): detection event → haptic response
//! - ∂ (Boundary): P0-P2 wake screen, P3-P5 silent
//!
//! ## Tier: T2-C (μ + → + ∂)

use nexcore_watch_core::AlertLevel;

/// Haptic pattern durations in milliseconds.
///
/// ## Primitive: σ (Sequence) + N (Quantity)
/// ## Tier: T2-P
pub struct HapticPattern {
    /// On/off durations in ms — σ (Sequence)
    pub durations: &'static [u64],
    /// Amplitude (0-255) — N (Quantity)
    pub amplitude: u8,
}

/// Map alert level to haptic pattern.
///
/// ## Primitive: μ (Mapping)
/// ## Tier: T1
pub fn pattern_for_level(level: AlertLevel) -> HapticPattern {
    match level {
        // P0: PERSISTENT — long strong pulses, repeating
        AlertLevel::P0PatientSafety => HapticPattern {
            durations: &[0, 500, 200, 500, 200, 500],
            amplitude: 255,
        },
        // P1: STRONG — two firm pulses
        AlertLevel::P1SignalIntegrity => HapticPattern {
            durations: &[0, 400, 200, 400],
            amplitude: 200,
        },
        // P2: STANDARD — single pulse
        AlertLevel::P2Regulatory => HapticPattern {
            durations: &[0, 300],
            amplitude: 150,
        },
        // P3: GENTLE — light tap
        AlertLevel::P3DataQuality => HapticPattern {
            durations: &[0, 150],
            amplitude: 80,
        },
        // P4, P5: SILENT — no vibration
        AlertLevel::P4Operational | AlertLevel::P5Cost => HapticPattern {
            durations: &[],
            amplitude: 0,
        },
    }
}

/// Execute haptic pattern via NDK Vibrator.
///
/// ## Primitive: → (Causality) — pattern → physical vibration
/// ## Tier: T3
///
/// Currently a stub — requires NDK vibrator service.
/// Will be wired in Phase 3 via `ndk::hardware::vibrator`.
pub fn execute_haptic(_pattern: &HapticPattern) {
    // Phase 3: Wire to NDK vibrator
    // let vibrator = ndk::hardware::vibrator::Vibrator::new();
    // vibrator.vibrate(pattern.durations, pattern.amplitude);
}
