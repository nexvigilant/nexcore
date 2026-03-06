//! # Synapse Bridge
//!
//! Inter-crate pipeline: Synapse → Nervous.
//!
//! Converts synaptic amplitude and consolidation data into nervous system
//! impulses and myelination patterns.
//!
//! ```text
//! Synapse::Synapse → Impulse (consolidated patterns become nerve signals)
//! Synapse::Amplitude → Myelination (strong synapses create hot-path caches)
//! Synapse::ConsolidationStatus → SignalSpeed (consolidated = myelinated)
//! ```
//!
//! **Biological mapping**: Synaptic transmission — when a synapse fires
//! (amplitude exceeds threshold), it generates an action potential (Impulse)
//! in the post-synaptic neuron. Repeatedly activated synapses trigger
//! myelination of the associated nerve fiber, increasing signal speed.

use nexcore_synapse::{Amplitude, ConsolidationStatus, Synapse};

use crate::{Impulse, Myelination, SignalSpeed};

/// Convert a consolidated synapse into a nervous system impulse.
///
/// Only consolidated synapses generate impulses — sub-threshold
/// synapses are not strong enough to trigger an action potential.
///
/// **Biological mapping**: Action potential generation — the synapse
/// must reach threshold (consolidation) before the post-synaptic
/// neuron fires. The impulse priority scales with amplitude strength.
pub fn synapse_to_impulse(synapse: &Synapse) -> Option<Impulse> {
    if synapse.status() != ConsolidationStatus::Consolidated {
        return None;
    }

    let amplitude = synapse.current_amplitude();
    let priority = amplitude_to_priority(amplitude);

    Some(Impulse {
        source: format!("synapse:{}", synapse.id),
        target: "cortex".to_string(),
        payload: format!(
            "consolidated:amplitude={:.4},observations={}",
            amplitude.value(),
            synapse.observation_count()
        ),
        priority,
        timestamp: nexcore_chrono::DateTime::now(),
    })
}

/// Map a synapse to a myelination cache entry.
///
/// Frequently activated synapses (high observation count) trigger
/// myelination, which caches the learned pattern for faster retrieval.
///
/// **Biological mapping**: Activity-dependent myelination — oligodendrocytes
/// wrap frequently-firing axons in myelin sheaths, dramatically increasing
/// conduction velocity from ~1 m/s to ~100 m/s.
pub fn synapse_to_myelination(synapse: &Synapse) -> Myelination {
    Myelination::new(
        format!("synapse:{}", synapse.id),
        format!(
            "amplitude={:.4},status={}",
            synapse.current_amplitude().value(),
            synapse.status()
        ),
    )
}

/// Determine signal speed based on synapse consolidation status.
///
/// **Biological mapping**: Nerve conduction velocity —
/// - Consolidated (myelinated): fastest transmission
/// - Accumulating (unmyelinated): slow transmission
/// - Decayed (demyelinated): reflex-level fallback to prevent signal loss
pub fn consolidation_to_speed(status: ConsolidationStatus) -> SignalSpeed {
    match status {
        ConsolidationStatus::Consolidated => SignalSpeed::Myelinated,
        ConsolidationStatus::Accumulating => SignalSpeed::Unmyelinated,
        ConsolidationStatus::Decayed => SignalSpeed::Reflex,
    }
}

/// Compute the throughput metric for a synapse.
///
/// Returns the current amplitude value as a scalar (0.0–1.0) representing
/// the strength of synaptic transmission into the nervous system.
///
/// **Biological mapping**: Synaptic efficacy — stronger synapses transmit
/// more neurotransmitter, producing larger post-synaptic potentials.
pub fn synapse_throughput(synapse: &Synapse) -> f64 {
    synapse.current_amplitude().value()
}

/// Map amplitude to impulse priority (0-255, higher = more urgent).
///
/// **Biological mapping**: Post-synaptic potential magnitude — larger
/// amplitudes produce stronger depolarization, which is routed with
/// higher priority in the nervous system.
fn amplitude_to_priority(amplitude: Amplitude) -> u8 {
    // amplitude is 0.0–1.0; invert and scale to 0–255
    // Higher amplitude → lower number → higher priority
    let inverted = 1.0 - amplitude.value();
    (inverted * 200.0) as u8 + 1 // Range: 1 (strongest) to 201 (weakest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_synapse::{AmplitudeConfig, LearningSignal};

    fn make_consolidated_synapse() -> Synapse {
        let config = AmplitudeConfig {
            consolidation_threshold: 0.3,
            learning_rate: 0.5,
            ..AmplitudeConfig::FAST
        };
        let mut synapse = Synapse::new("test-pattern", config);
        // Add enough signals to consolidate
        for _ in 0..10 {
            synapse.observe(LearningSignal::new(1.0, 1.0));
        }
        synapse
    }

    fn make_accumulating_synapse() -> Synapse {
        Synapse::new("fresh-pattern", AmplitudeConfig::default())
    }

    #[test]
    fn test_synapse_to_impulse_consolidated() {
        let synapse = make_consolidated_synapse();
        assert_eq!(synapse.status(), ConsolidationStatus::Consolidated);

        let impulse = synapse_to_impulse(&synapse);
        assert!(impulse.is_some());

        let impulse = impulse.unwrap_or_else(|| Impulse {
            source: String::new(),
            target: String::new(),
            payload: String::new(),
            priority: 0,
            timestamp: nexcore_chrono::DateTime::now(),
        });
        assert!(impulse.source.contains("test-pattern"));
        assert_eq!(impulse.target, "cortex");
        assert!(impulse.payload.contains("consolidated"));
    }

    #[test]
    fn test_synapse_to_impulse_not_consolidated() {
        let synapse = make_accumulating_synapse();
        assert_eq!(synapse.status(), ConsolidationStatus::Accumulating);

        let impulse = synapse_to_impulse(&synapse);
        assert!(impulse.is_none());
    }

    #[test]
    fn test_synapse_to_myelination() {
        let synapse = make_consolidated_synapse();
        let myelin = synapse_to_myelination(&synapse);

        assert!(myelin.pattern.contains("test-pattern"));
        assert!(myelin.cached_response.contains("amplitude="));
        assert_eq!(myelin.hit_count, 0);
        assert_eq!(myelin.miss_count, 0);
    }

    #[test]
    fn test_consolidation_to_speed_consolidated() {
        assert_eq!(
            consolidation_to_speed(ConsolidationStatus::Consolidated),
            SignalSpeed::Myelinated
        );
    }

    #[test]
    fn test_consolidation_to_speed_accumulating() {
        assert_eq!(
            consolidation_to_speed(ConsolidationStatus::Accumulating),
            SignalSpeed::Unmyelinated
        );
    }

    #[test]
    fn test_consolidation_to_speed_decayed() {
        assert_eq!(
            consolidation_to_speed(ConsolidationStatus::Decayed),
            SignalSpeed::Reflex
        );
    }

    #[test]
    fn test_synapse_throughput_zero() {
        let synapse = make_accumulating_synapse();
        let throughput = synapse_throughput(&synapse);
        assert!((throughput - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_synapse_throughput_consolidated() {
        let synapse = make_consolidated_synapse();
        let throughput = synapse_throughput(&synapse);
        assert!(throughput > 0.0, "Consolidated synapse should have positive throughput");
    }

    #[test]
    fn test_amplitude_to_priority_strong() {
        // Strong amplitude → low priority number (high urgency)
        let priority = amplitude_to_priority(Amplitude::new(0.9));
        assert!(priority < 50, "Strong amplitude should have low priority number: got {priority}");
    }

    #[test]
    fn test_amplitude_to_priority_weak() {
        // Weak amplitude → high priority number (low urgency)
        let priority = amplitude_to_priority(Amplitude::new(0.1));
        assert!(priority > 100, "Weak amplitude should have high priority number: got {priority}");
    }
}
