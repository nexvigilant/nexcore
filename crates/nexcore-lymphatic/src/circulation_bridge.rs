//! # Circulation Bridge
//!
//! Inter-crate pipeline: Circulatory → Lymphatic.
//!
//! Converts circulatory `Pulse` overflow into lymphatic `OverflowItem`s
//! for drainage, and maps `BloodPressure` to edema risk assessment.
//!
//! ```text
//! Circulatory::Pulse → OverflowItem → LymphDrainage/OverflowHandler
//! ```

use nexcore_circulatory::{BloodPressure, Pulse};

use crate::OverflowItem;

/// Convert circulatory pulse excess into lymphatic overflow items.
///
/// When distributed cells exceed collected cells, the surplus represents
/// interstitial fluid that the lymphatic system must drain.
///
/// **Biological mapping**: Capillary filtration — plasma leaks from
/// capillaries into interstitial space; lymph vessels collect the excess.
pub fn pulse_to_overflow(pulse: &Pulse) -> Vec<OverflowItem> {
    // Each distributed-but-not-enriched cell becomes an overflow item.
    // If distributed > enriched, the excess wasn't fully processed.
    let excess = pulse.distributed.saturating_sub(pulse.enriched);

    let healthy = pulse.pressure.is_healthy();
    let priority = if healthy { 3 } else { 1 };

    let mut items = Vec::with_capacity(excess);
    for i in 0..excess {
        items.push(OverflowItem::new(
            format!("pulse-overflow-{}-{i}", pulse.timestamp),
            "circulatory",
            priority,
        ));
    }

    // Also create an item for the pressure reading itself if unhealthy
    if !healthy {
        items.push(OverflowItem::new(
            format!("pressure-alert:ratio={:.2}", pulse.pressure.ratio()),
            "circulatory",
            1, // High priority
        ));
    }

    items
}

/// Assess edema risk from circulatory blood pressure.
///
/// Returns true if pressure ratio indicates fluid buildup risk.
///
/// **Biological mapping**: Edema detection — when capillary pressure
/// exceeds lymphatic drainage capacity, fluid accumulates in tissues.
pub fn pressure_to_edema_risk(pressure: &BloodPressure) -> bool {
    // Unhealthy pressure (ratio < 0.7 or > 1.0) indicates edema risk
    !pressure.is_healthy()
}

/// Extract a throughput value from a pulse for chain tracing.
///
/// Returns the distributed count as a scalar metric that can be
/// propagated through the lymphatic → urinary chain.
pub fn pulse_throughput(pulse: &Pulse) -> usize {
    pulse.distributed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pulse(collected: usize, enriched: usize, distributed: usize) -> Pulse {
        Pulse {
            collected,
            enriched,
            distributed,
            pressure: BloodPressure::new(100, 80),
            timestamp: "test-ts".to_string(),
        }
    }

    #[test]
    fn test_pulse_to_overflow_no_excess() {
        let pulse = make_pulse(10, 10, 10);
        let overflow = pulse_to_overflow(&pulse);
        // No excess, pressure is healthy (80/100 = 0.8) → empty
        assert!(overflow.is_empty());
    }

    #[test]
    fn test_pulse_to_overflow_with_excess() {
        let pulse = make_pulse(10, 5, 8);
        let overflow = pulse_to_overflow(&pulse);
        // excess = 8 - 5 = 3 items, pressure healthy → 3 items
        assert_eq!(overflow.len(), 3);
        assert!(overflow[0].source == "circulatory");
        assert_eq!(overflow[0].priority, 3); // healthy pressure = low priority
    }

    #[test]
    fn test_pulse_to_overflow_unhealthy_pressure() {
        let pulse = Pulse {
            collected: 10,
            enriched: 10,
            distributed: 10,
            pressure: BloodPressure::new(100, 50), // 0.5 ratio = unhealthy
            timestamp: "test".to_string(),
        };
        let overflow = pulse_to_overflow(&pulse);
        // No excess items, but 1 pressure alert
        assert_eq!(overflow.len(), 1);
        assert!(overflow[0].content.contains("pressure-alert"));
        assert_eq!(overflow[0].priority, 1); // high priority
    }

    #[test]
    fn test_pressure_to_edema_risk_healthy() {
        let pressure = BloodPressure::new(100, 80);
        assert!(!pressure_to_edema_risk(&pressure));
    }

    #[test]
    fn test_pressure_to_edema_risk_unhealthy() {
        let pressure = BloodPressure::new(100, 50);
        assert!(pressure_to_edema_risk(&pressure));
    }

    #[test]
    fn test_pulse_throughput() {
        let pulse = make_pulse(10, 8, 7);
        assert_eq!(pulse_throughput(&pulse), 7);
    }

    #[test]
    fn test_excess_items_have_correct_source() {
        let pulse = make_pulse(10, 3, 8);
        let items = pulse_to_overflow(&pulse);
        for item in &items {
            assert_eq!(item.source, "circulatory");
        }
    }
}
