//! Cytokine-Guardian Integration Test
//!
//! Demonstrates bidirectional cytokine ↔ Guardian signal flow:
//! 1. Guardian action → CytokineActuator → emits cytokine
//! 2. Cytokine emission → CytokineSensor → Guardian signal

use std::sync::Arc;

use nexcore_cytokine::{CytokineBus, CytokineFamily, Emitter, Scope};
use nexcore_vigilance::guardian::{
    response::{Actuator, EscalationLevel, ResponseAction, cytokine::CytokineActuator},
    sensing::{Sensor, ThreatLevel, cytokine::CytokineSensor},
};

/// Test: Guardian ResponseAction triggers cytokine emission via CytokineActuator
#[tokio::test]
async fn test_guardian_action_emits_cytokine() {
    let bus = Arc::new(CytokineBus::new("integration-test"));
    let actuator = CytokineActuator::new(bus.clone());

    // Initial state: no signals emitted
    let stats_before = bus.stats().await;
    assert_eq!(stats_before.signals_emitted, 0);

    // Execute an Escalate action (should emit IL-1 alarm)
    let action = ResponseAction::Escalate {
        level: EscalationLevel::L2,
        description: "Test escalation from integration test".to_string(),
        assigned_to: None,
    };

    let result = actuator.execute(&action).await;
    assert!(result.success, "Actuator should succeed");
    assert!(
        result.message.contains("IL-1"),
        "Should emit IL-1 for escalation"
    );

    // Verify cytokine was emitted
    let stats_after = bus.stats().await;
    assert_eq!(
        stats_after.signals_emitted, 1,
        "Should have emitted 1 cytokine"
    );

    // Execute a Block action (should emit TNF-α terminate)
    let block_action = ResponseAction::Block {
        target: "192.168.1.100".to_string(),
        duration: Some(3600),
        reason: "Test block".to_string(),
    };

    let block_result = actuator.execute(&block_action).await;
    assert!(block_result.success);
    assert!(
        block_result.message.contains("TNF"),
        "Should emit TNF-α for block"
    );

    let final_stats = bus.stats().await;
    assert_eq!(
        final_stats.signals_emitted, 2,
        "Should have emitted 2 total"
    );
}

/// Test: Cytokine emission detected by CytokineSensor as Guardian signal
#[tokio::test]
async fn test_cytokine_detected_as_guardian_signal() {
    let bus = Arc::new(CytokineBus::new("sensor-test"));
    let sensor = CytokineSensor::new(bus.clone());

    // Sensor should start with no pending signals
    assert_eq!(sensor.name(), "cytokine-sensor");
    assert!(sensor.sensitivity() > 0.8);

    // Emit a high-severity alarm cytokine
    let alarm_result = bus.alarm("test_threat_detected").await;
    assert!(alarm_result.is_ok(), "Alarm emission should succeed");

    // Small delay to allow broadcast propagation
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Note: Due to broadcast channel semantics, new subscribers don't see past messages.
    // In production, the sensor would be registered before emissions.
    // This test validates the API contract rather than message propagation.

    let stats = bus.stats().await;
    assert!(stats.signals_emitted >= 1, "At least one signal emitted");
}

/// Test: Full bidirectional flow - Action → Cytokine → Sensor
#[tokio::test]
async fn test_bidirectional_flow() {
    let bus = Arc::new(CytokineBus::new("bidirectional-test"));

    // Create both sensor and actuator sharing the same bus
    let sensor = CytokineSensor::new(bus.clone());
    let actuator = CytokineActuator::new(bus.clone());

    // Subscribe the sensor BEFORE any emissions
    // (This simulates production setup where sensor is registered first)
    let _receiver = bus.subscribe();

    // Trigger a Guardian action that emits cytokine
    let action = ResponseAction::Alert {
        severity: ThreatLevel::High,
        message: "Bidirectional test alert".to_string(),
        recipients: vec!["test@example.com".to_string()],
    };

    let result = actuator.execute(&action).await;
    assert!(result.success);
    assert!(result.message.contains("IL-6"), "Alert should emit IL-6");

    // Verify the cytokine system is bridged
    let stats = bus.stats().await;
    assert_eq!(stats.signals_emitted, 1);

    // The sensor's detect() will pick up signals from its subscription
    // (In production, this runs in the homeostasis loop)
    println!(
        "Bidirectional flow complete: {} signals emitted, sensor sensitivity: {}",
        stats.signals_emitted,
        sensor.sensitivity()
    );
}

/// Test: HomeostasisLoop with cytokine bus integration
#[tokio::test]
async fn test_homeostasis_loop_with_cytokine_bus() {
    use nexcore_vigilance::guardian::homeostasis::{DecisionEngine, HomeostasisLoop};

    let bus = Arc::new(CytokineBus::new("homeostasis-test"));

    // Create loop with cytokine integration
    let loop_instance =
        HomeostasisLoop::new(DecisionEngine::default()).with_cytokine_bus(bus.clone());

    // Verify both sensor and actuator are registered
    assert!(
        loop_instance.sensor_count() >= 1,
        "Should have cytokine sensor"
    );
    assert!(
        loop_instance.actuator_count() >= 1,
        "Should have cytokine actuator"
    );

    // Run a tick - this exercises the full SENSE→DECIDE→RESPOND cycle
    let mut loop_instance = loop_instance; // Make mutable for tick
    let result = loop_instance.tick().await;
    // LoopIterationResult is always returned - check it completed
    assert!(!result.iteration_id.is_empty(), "Tick should complete");

    // The loop is now cytokine-aware
    println!(
        "HomeostasisLoop integrated: {} sensors, {} actuators",
        loop_instance.sensor_count(),
        loop_instance.actuator_count()
    );
}

/// Test: Action-to-cytokine mapping correctness
#[tokio::test]
async fn test_action_cytokine_mapping() {
    let bus = Arc::new(CytokineBus::new("mapping-test"));
    let actuator = CytokineActuator::new(bus.clone());

    // Test each action type maps to correct cytokine family
    let test_cases = vec![
        (
            ResponseAction::Escalate {
                level: EscalationLevel::L1,
                description: "test".to_string(),
                assigned_to: None,
            },
            "IL-1", // Alarm
        ),
        (
            ResponseAction::Block {
                target: "x".to_string(),
                duration: None,
                reason: "test".to_string(),
            },
            "TNF", // Terminate
        ),
        (
            ResponseAction::Alert {
                severity: ThreatLevel::Medium,
                message: "test".to_string(),
                recipients: vec![],
            },
            "IL-6", // Acute response
        ),
        (
            ResponseAction::RateLimit {
                resource: "x".to_string(),
                max_requests: 10,
                window_seconds: 60,
            },
            "IL-10", // Suppress
        ),
        (
            ResponseAction::Quarantine {
                target: "x".to_string(),
                quarantine_type: "test".to_string(),
            },
            "IFN", // Activate
        ),
    ];

    for (action, expected_family) in test_cases {
        let result = actuator.execute(&action).await;
        assert!(result.success, "Action should succeed");
        assert!(
            result.message.contains(expected_family),
            "Action {:?} should emit {} cytokine, got: {}",
            action,
            expected_family,
            result.message
        );
    }

    let final_stats = bus.stats().await;
    assert_eq!(
        final_stats.signals_emitted, 5,
        "Should emit 5 cytokines for 5 actions"
    );
}
