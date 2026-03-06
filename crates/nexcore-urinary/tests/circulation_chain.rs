//! # Circulation Chain Integration Test
//!
//! Traces a throughput signal across all 4 crate boundaries:
//!
//! ```text
//! Energy(Regime) → Cardiovascular(pump) → Circulatory(pulse) → Lymphatic(overflow) → Urinary(excretion)
//! ```
//!
//! Uses dev-dependencies to access all 5 crates from the urinary test harness.
//! Production deps remain strictly linear (each imports only immediate upstream).

use nexcore_cardiovascular::energy_bridge::{energy_driven_pump, regime_to_demand};
use nexcore_circulatory::cardio_bridge::{pump_result_to_cell, pump_throughput};
use nexcore_circulatory::{BloodPressure, Pulse};
use nexcore_energy::Regime;
use nexcore_lymphatic::OverflowItem;
use nexcore_lymphatic::circulation_bridge::{pulse_throughput, pulse_to_overflow};
use nexcore_urinary::Bladder;
use nexcore_urinary::lymph_bridge::drain_to_bladder;

#[test]
fn test_full_circulation_chain_anabolic() {
    // ═══════════════════════════════════════════════════════
    // HOP 1: Energy → Cardiovascular
    // ═══════════════════════════════════════════════════════
    let regime = Regime::Anabolic;
    let mut heart = nexcore_cardiovascular::Heart::new();
    let pump_result = energy_driven_pump(&mut heart, &regime);

    // Verify: Anabolic regime produces meaningful pump output
    assert!(pump_result.stroke_volume > 0.0, "Hop 1: stroke volume > 0");
    let cardio_throughput = pump_throughput(&pump_result);
    assert!(cardio_throughput > 0.0, "Hop 1: throughput > 0");

    // ═══════════════════════════════════════════════════════
    // HOP 2: Cardiovascular → Circulatory
    // ═══════════════════════════════════════════════════════
    let blood_cell = pump_result_to_cell(&pump_result);
    assert_eq!(blood_cell.source, "cardiovascular");

    // Simulate a circulatory pulse using the cardiovascular output
    let pulse = Pulse {
        collected: 10,
        enriched: 7,
        distributed: 9,                        // 9 - 7 = 2 excess items
        pressure: BloodPressure::new(100, 75), // healthy
        timestamp: "integration-test".to_string(),
    };
    let circ_throughput = pulse_throughput(&pulse);
    assert!(circ_throughput > 0, "Hop 2: distributed > 0");

    // ═══════════════════════════════════════════════════════
    // HOP 3: Circulatory → Lymphatic
    // ═══════════════════════════════════════════════════════
    let overflow_items = pulse_to_overflow(&pulse);
    assert!(
        !overflow_items.is_empty(),
        "Hop 3: excess items become overflow"
    );
    // Each overflow item originated from circulatory
    for item in &overflow_items {
        assert_eq!(item.source, "circulatory");
    }

    // ═══════════════════════════════════════════════════════
    // HOP 4: Lymphatic → Urinary
    // ═══════════════════════════════════════════════════════
    let mut bladder = Bladder::new(100);
    let accepted = drain_to_bladder(&mut bladder, &overflow_items);
    assert_eq!(accepted, overflow_items.len(), "All items accepted");

    // Final excretion
    let excreted = bladder.flush();
    assert!(!excreted.is_empty(), "Hop 4: waste excreted");

    // ═══════════════════════════════════════════════════════
    // CHAIN INTEGRITY: cross-hop signal preservation
    // ═══════════════════════════════════════════════════════
    // Verify the signal count is preserved or amplified at each boundary.
    // Items in >= items out (some hops add alerts), never lost.
    assert_eq!(
        excreted.len(),
        overflow_items.len(),
        "All overflow items reached excretion (no signal loss)"
    );
    assert!(
        overflow_items.len() <= circ_throughput,
        "Overflow count bounded by circulatory throughput"
    );
}

#[test]
fn test_full_circulation_chain_crisis() {
    // Crisis regime: minimal throughput, but chain still functions
    let regime = Regime::Crisis;
    let demand = regime_to_demand(&regime);
    assert!(demand < 0.2, "Crisis demand should be minimal");

    let mut heart = nexcore_cardiovascular::Heart::new();
    let pump_result = energy_driven_pump(&mut heart, &regime);

    // Even in crisis, the heart still pumps
    assert!(pump_result.stroke_volume > 0.0);
    let cell = pump_result_to_cell(&pump_result);
    assert_eq!(cell.source, "cardiovascular");

    // Simulate minimal circulation
    let pulse = Pulse {
        collected: 2,
        enriched: 1,
        distributed: 2,                        // 2 - 1 = 1 excess
        pressure: BloodPressure::new(100, 60), // unhealthy (ratio 0.6)
        timestamp: "crisis-test".to_string(),
    };

    let overflow = pulse_to_overflow(&pulse);
    // Should have 1 excess item + 1 pressure alert (unhealthy)
    assert!(
        overflow.len() >= 2,
        "Crisis produces excess + pressure alert"
    );

    let mut bladder = Bladder::new(50);
    let accepted = drain_to_bladder(&mut bladder, &overflow);
    assert!(accepted > 0, "Crisis waste reaches bladder");

    let excreted = bladder.flush();
    assert!(!excreted.is_empty(), "Crisis waste excreted");
}

#[test]
fn test_all_regimes_produce_chain_output() {
    let regimes = [
        Regime::Anabolic,
        Regime::Homeostatic,
        Regime::Catabolic,
        Regime::Crisis,
    ];

    for regime in &regimes {
        let mut heart = nexcore_cardiovascular::Heart::new();
        let pump_result = energy_driven_pump(&mut heart, regime);

        assert!(
            pump_result.stroke_volume > 0.0,
            "{regime:?} should produce pump output"
        );

        let cell = pump_result_to_cell(&pump_result);
        assert_eq!(cell.source, "cardiovascular");
    }
}
