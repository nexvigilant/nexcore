//! # NexVigilant Core — Cardiovascular Data Transport
//!
//! Typed data transport system based on cardiovascular physiology.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol | Role |
//! |---------|-----------|--------|------|
//! | Heart pump | Causality | → | Pump causes circulation |
//! | Vessel walls | Boundary | ∂ | Channels constrain flow |
//! | Blood mapping | Mapping | μ | Payloads mapped to destinations |
//! | Pressure regulation | Comparison | κ | Current vs set point |
//! | Heartbeat | Frequency | ν | Pump cycle rate |
//! | Pathology | Irreversibility | ∝ | Damage is one-way |
//! | Flow rate | Quantity | N | Throughput measurement |
//! | Pump state | State | ς | Beat count, circuit |
//!
//! ## Key Laws
//!
//! - **Frank-Starling**: stroke_volume = f(venous_return) — throughput auto-scales with input
//! - **Blood Pressure**: BP = cardiac_output × peripheral_resistance
//! - **Ohm's Analog**: ΔP = flow × resistance (pressure drop across vessel)
//!
//! ## Modules
//!
//! - [`heart`]: Central pump with dual circulation (pulmonary + systemic)
//! - [`vessels`]: Vessel hierarchy (arteries → capillaries → veins)
//! - [`blood`]: Blood components (red cells, white cells, platelets, plasma)
//! - [`pressure`]: Baroreceptor regulation and monitoring
//! - [`pathology`]: Disease detection (atherosclerosis, infarction, hypertension)
//! - [`grounding`]: Lex Primitiva GroundsTo implementations (25 types)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod blood;
pub mod grounding;
pub mod heart;
pub mod pathology;
pub mod pressure;
pub mod types;
pub mod vessels;

pub use blood::*;
pub use heart::{CardiacVitals, Heart, HeartConfig, PumpResult};
pub use pathology::{Diagnosis, Severity, diagnose};
pub use pressure::{Baroreceptor, PressureClass, PressureConfig, PressureReading};
pub use types::*;
pub use vessels::{CapillaryExchange, VascularBed, Vessel, capillary_exchange};

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════
    // Heart tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn heart_default_creates_valid_heart() {
        let heart = Heart::new();
        assert_eq!(heart.beat_count, 0);
        assert!(heart.stroke_volume > 0.0);
        assert!(heart.rate.value() > 0.0);
    }

    #[test]
    fn frank_starling_scales_with_preload() {
        let heart = Heart::new();
        let sv_low = heart.frank_starling(50.0);
        let sv_high = heart.frank_starling(150.0);
        assert!(sv_high > sv_low, "Higher preload should produce higher SV");
    }

    #[test]
    fn frank_starling_caps_at_max() {
        let heart = Heart::new();
        let sv = heart.frank_starling(10000.0);
        assert!(
            sv <= heart.config.max_stroke_volume,
            "SV {} should not exceed max {}",
            sv,
            heart.config.max_stroke_volume
        );
    }

    #[test]
    fn frank_starling_handles_zero_preload() {
        let heart = Heart::new();
        let sv = heart.frank_starling(0.0);
        assert!((sv - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn frank_starling_handles_negative_preload() {
        let heart = Heart::new();
        let sv = heart.frank_starling(-10.0);
        assert!((sv - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cardiac_output_is_sv_times_hr() {
        let heart = Heart::new();
        let co = heart.cardiac_output();
        let expected = heart.stroke_volume * heart.rate.value() / 60.0;
        assert!((co.value() - expected).abs() < 0.01);
    }

    #[test]
    fn pump_increments_beat_count() {
        let mut heart = Heart::new();
        assert_eq!(heart.beat_count, 0);
        heart.pump(100.0);
        assert_eq!(heart.beat_count, 1);
        heart.pump(100.0);
        assert_eq!(heart.beat_count, 2);
    }

    #[test]
    fn pump_alternates_circuits() {
        let mut heart = Heart::new();
        let r1 = heart.pump(100.0);
        let r2 = heart.pump(100.0);
        assert_ne!(r1.circuit, r2.circuit);
    }

    #[test]
    fn pump_returns_positive_stroke_volume() {
        let mut heart = Heart::new();
        let result = heart.pump(100.0);
        assert!(result.stroke_volume > 0.0);
    }

    #[test]
    fn blood_pressure_is_co_times_resistance() {
        let heart = Heart::new();
        let r = Resistance::new(2.0);
        let bp = heart.blood_pressure(r);
        let co = heart.cardiac_output();
        let expected = co.value() * r.value();
        assert!((bp.value() - expected).abs() < 0.01);
    }

    #[test]
    fn regulate_rate_increases_with_demand() {
        let mut heart = Heart::new();
        let resting = heart.rate.value();
        heart.regulate_rate(0.8);
        assert!(heart.rate.value() > resting);
    }

    #[test]
    fn regulate_rate_clamps_demand() {
        let mut heart = Heart::new();
        heart.regulate_rate(5.0); // Over 1.0
        assert!(heart.rate.value() <= heart.config.max_rate.value());
    }

    #[test]
    fn vitals_snapshot_captures_state() {
        let heart = Heart::new();
        let vitals = heart.vitals(Resistance::default());
        assert_eq!(vitals.beat_count, 0);
        assert!(vitals.cardiac_output.value() > 0.0);
    }

    // ═══════════════════════════════════════════════════════════
    // Vessel tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn vessel_creation_by_type() {
        let artery = Vessel::new("test_artery", VesselType::Artery);
        let capillary = Vessel::new("test_cap", VesselType::Capillary);
        assert!(artery.capacity.value() > capillary.capacity.value());
    }

    #[test]
    fn vessel_receive_respects_capacity() {
        let mut vessel = Vessel::new("v1", VesselType::Capillary);
        let accepted = vessel.receive(10000.0);
        assert!(accepted < 10000.0, "Should not accept more than capacity");
    }

    #[test]
    fn vessel_release_respects_fill() {
        let mut vessel = Vessel::new("v1", VesselType::Artery);
        vessel.receive(10.0);
        let released = vessel.release(5.0);
        assert!((released - 5.0).abs() < 0.01);
    }

    #[test]
    fn vessel_release_caps_at_available() {
        let mut vessel = Vessel::new("v1", VesselType::Artery);
        vessel.receive(3.0);
        let released = vessel.release(100.0);
        assert!(released <= 3.01);
    }

    #[test]
    fn vessel_occlusion_increases_resistance() {
        let mut vessel = Vessel::new("v1", VesselType::Artery);
        let r_clear = vessel.effective_resistance().value();
        vessel.occlusion = 0.5;
        let r_occluded = vessel.effective_resistance().value();
        assert!(r_occluded > r_clear);
    }

    #[test]
    fn vessel_critical_occlusion_at_80_percent() {
        let mut vessel = Vessel::new("v1", VesselType::Artery);
        vessel.occlusion = 0.79;
        assert!(!vessel.is_critically_occluded());
        vessel.occlusion = 0.81;
        assert!(vessel.is_critically_occluded());
    }

    #[test]
    fn vessel_type_classification() {
        assert!(VesselType::Aorta.is_arterial());
        assert!(VesselType::Artery.is_arterial());
        assert!(VesselType::Arteriole.is_arterial());
        assert!(VesselType::Capillary.is_capillary());
        assert!(VesselType::Venule.is_venous());
        assert!(VesselType::Vein.is_venous());
        assert!(VesselType::VenaCava.is_venous());
    }

    #[test]
    fn vessel_pressure_drops_with_flow() {
        let vessel = Vessel::new("v1", VesselType::Artery);
        let drop = vessel.pressure_drop(FlowRate::new(5.0));
        assert!(drop.value() > 0.0);
    }

    #[test]
    fn vascular_bed_standard_organ() {
        let bed = VascularBed::standard_organ("kidney");
        assert_eq!(bed.vessels.len(), 5);
        assert!(bed.vessels[0].vessel_type.is_arterial());
        assert!(bed.vessels[2].vessel_type.is_capillary());
        assert!(bed.vessels[4].vessel_type.is_venous());
    }

    #[test]
    fn vascular_bed_total_resistance() {
        let bed = VascularBed::standard_organ("liver");
        let r = bed.total_resistance();
        assert!(r.value() > 0.0);
    }

    #[test]
    fn vascular_bed_finds_occlusions() {
        let mut bed = VascularBed::standard_organ("heart");
        bed.vessels[1].occlusion = 0.9;
        let occluded = bed.find_occlusions();
        assert_eq!(occluded.len(), 1);
    }

    #[test]
    fn capillary_exchange_computes() {
        let ex = capillary_exchange(Pressure::new(0.6), Pressure::new(0.1), 0.3);
        assert!(ex.delivered > 0.0);
        assert!(ex.collected > 0.0);
    }

    // ═══════════════════════════════════════════════════════════
    // Blood tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn red_cell_load_and_unload() {
        let mut cell = RedCell::new("payload1", "source_a", 100);
        let loaded = cell.load_payload(50);
        assert_eq!(loaded, 50);
        assert_eq!(cell.load, 50);
        let unloaded = cell.unload();
        assert_eq!(unloaded, 50);
        assert_eq!(cell.load, 0);
    }

    #[test]
    fn red_cell_caps_at_capacity() {
        let mut cell = RedCell::new("p1", "s1", 10);
        let loaded = cell.load_payload(100);
        assert_eq!(loaded, 10);
        assert!(cell.is_full());
    }

    #[test]
    fn red_cell_desaturates_on_unload() {
        let mut cell = RedCell::new("p1", "s1", 100);
        cell.load_payload(50);
        assert!((cell.saturation - 1.0).abs() < f64::EPSILON);
        cell.unload();
        assert!(cell.saturation < 1.0);
        assert!(cell.needs_reoxygenation() || cell.saturation >= 0.5);
    }

    #[test]
    fn red_cell_oxygenate_restores_saturation() {
        let mut cell = RedCell::new("p1", "s1", 100);
        cell.saturation = 0.3;
        cell.oxygenate();
        assert!((cell.saturation - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn white_cell_targets_matching_threats() {
        let mut wbc = WhiteCell::new("wbc1", vec!["error".to_string()]);
        assert!(wbc.can_target("error"));
        assert!(!wbc.can_target("anomaly"));
        assert!(wbc.neutralize("error"));
        assert_eq!(wbc.kills, 1);
    }

    #[test]
    fn white_cell_wildcard_targets_all() {
        let wbc = WhiteCell::new("wbc_gen", vec!["*".to_string()]);
        assert!(wbc.can_target("anything"));
    }

    #[test]
    fn platelet_activates_once() {
        let mut p = Platelet::new(RepairType::Seal);
        assert!(p.activate());
        assert!(!p.activate()); // Already used
    }

    #[test]
    fn plasma_pressure_is_ratio() {
        let plasma = Plasma::new(1000.0);
        let p = plasma.pressure();
        assert!(p.value() > 0.0);
        assert!(p.value() <= 1.0);
    }

    #[test]
    fn plasma_consume_reduces_available() {
        let mut plasma = Plasma::new(1000.0);
        let initial = plasma.available_volume.value();
        plasma.consume(100.0);
        assert!(plasma.available_volume.value() < initial);
    }

    #[test]
    fn plasma_release_increases_available() {
        let mut plasma = Plasma::new(1000.0);
        plasma.consume(200.0);
        let after_consume = plasma.available_volume.value();
        plasma.release(100.0);
        assert!(plasma.available_volume.value() > after_consume);
    }

    #[test]
    fn blood_health_assessment() {
        assert_eq!(
            assess_blood_health(500, 50, 200, Pressure::new(0.5)),
            BloodHealth::Healthy
        );
        assert_eq!(
            assess_blood_health(50, 50, 200, Pressure::new(0.5)),
            BloodHealth::Anemic
        );
        assert_eq!(
            assess_blood_health(500, 5, 200, Pressure::new(0.5)),
            BloodHealth::Leukopenic
        );
        assert_eq!(
            assess_blood_health(500, 50, 20, Pressure::new(0.5)),
            BloodHealth::Thrombocytopenic
        );
        assert_eq!(
            assess_blood_health(500, 50, 200, Pressure::new(0.1)),
            BloodHealth::Hypovolemic
        );
    }

    // ═══════════════════════════════════════════════════════════
    // Pressure tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn pressure_newtype_clamps_negative() {
        let p = Pressure::new(-5.0);
        assert!((p.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pressure_hypertension_detection() {
        let p = Pressure::new(0.9);
        assert!(p.is_hypertensive(0.8));
        assert!(!p.is_hypertensive(0.95));
    }

    #[test]
    fn pressure_hypotension_detection() {
        let p = Pressure::new(0.1);
        assert!(p.is_hypotensive(0.2));
        assert!(!p.is_hypotensive(0.05));
    }

    #[test]
    fn baroreceptor_classifies_normal() {
        let mut baro = Baroreceptor::new();
        let reading = baro.sense(Pressure::new(0.5));
        assert_eq!(reading.classification, PressureClass::Normal);
    }

    #[test]
    fn baroreceptor_classifies_hypertension() {
        let mut baro = Baroreceptor::new();
        let reading = baro.sense(Pressure::new(0.9));
        assert_eq!(reading.classification, PressureClass::Hypertensive);
    }

    #[test]
    fn baroreceptor_classifies_hypotension() {
        let mut baro = Baroreceptor::new();
        let reading = baro.sense(Pressure::new(0.1));
        assert_eq!(reading.classification, PressureClass::Hypotensive);
    }

    #[test]
    fn baroreceptor_sustained_hypertension_detects_pathology() {
        let mut baro = Baroreceptor::with_config(PressureConfig {
            pathology_threshold: 3,
            ..PressureConfig::default()
        });
        // Below threshold
        baro.sense(Pressure::new(0.9));
        baro.sense(Pressure::new(0.9));
        assert!(baro.detect_pathology().is_none());
        // Hit threshold
        baro.sense(Pressure::new(0.9));
        assert!(baro.detect_pathology().is_some());
    }

    #[test]
    fn baroreceptor_reflex_response_high_pressure() {
        let baro = Baroreceptor::new();
        let response = baro.reflex_response(Pressure::new(0.8));
        assert!(response < 1.0, "High pressure should reduce rate");
    }

    #[test]
    fn baroreceptor_reflex_response_low_pressure() {
        let baro = Baroreceptor::new();
        let response = baro.reflex_response(Pressure::new(0.2));
        assert!(response > 1.0, "Low pressure should increase rate");
    }

    #[test]
    fn baroreceptor_normal_resets_counters() {
        let mut baro = Baroreceptor::new();
        baro.sense(Pressure::new(0.9)); // hypertensive
        assert_eq!(baro.hypertension_count, 1);
        baro.sense(Pressure::new(0.5)); // normal
        assert_eq!(baro.hypertension_count, 0);
    }

    #[test]
    fn compute_bp_function() {
        let bp = pressure::compute_blood_pressure(5.0, 2.0);
        assert!((bp.value() - 10.0).abs() < 0.01);
    }

    // ═══════════════════════════════════════════════════════════
    // Pathology tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn diagnose_finds_atherosclerosis() {
        let mut bed = VascularBed::standard_organ("test");
        bed.vessels[0].occlusion = 0.6;
        let baro = Baroreceptor::new();
        let results = diagnose(&[bed], &baro, BloodHealth::Healthy);
        assert!(!results.is_empty());
        assert!(matches!(
            results[0].pathology,
            Pathology::Atherosclerosis { .. }
        ));
    }

    #[test]
    fn diagnose_finds_infarction_at_95_percent() {
        let mut bed = VascularBed::standard_organ("test");
        bed.vessels[0].occlusion = 0.96;
        let baro = Baroreceptor::new();
        let results = diagnose(&[bed], &baro, BloodHealth::Healthy);
        assert!(!results.is_empty());
        assert!(matches!(
            results[0].pathology,
            Pathology::Infarction { complete: true, .. }
        ));
        assert_eq!(results[0].severity, Severity::Critical);
    }

    #[test]
    fn diagnose_finds_anemia() {
        let bed = VascularBed::standard_organ("test");
        let baro = Baroreceptor::new();
        let results = diagnose(&[bed], &baro, BloodHealth::Anemic);
        assert!(
            results
                .iter()
                .any(|d| matches!(d.pathology, Pathology::Anemia { .. }))
        );
    }

    #[test]
    fn diagnose_clean_bill_of_health() {
        let bed = VascularBed::standard_organ("test");
        let baro = Baroreceptor::new();
        let results = diagnose(&[bed], &baro, BloodHealth::Healthy);
        assert!(results.is_empty());
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Mild < Severity::Moderate);
        assert!(Severity::Moderate < Severity::Severe);
        assert!(Severity::Severe < Severity::Critical);
    }

    // ═══════════════════════════════════════════════════════════
    // Type tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn flow_rate_clamps_negative() {
        let f = FlowRate::new(-5.0);
        assert!((f.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn volume_utilization() {
        let v = Volume::new(75.0);
        let util = v.utilization(100.0);
        assert!((util - 0.75).abs() < 0.01);
    }

    #[test]
    fn volume_utilization_zero_capacity() {
        let v = Volume::new(50.0);
        let util = v.utilization(0.0);
        assert!((util - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn heart_rate_bradycardia() {
        let hr = HeartRate::new(40.0);
        assert!(hr.is_bradycardic(60.0));
        assert!(!hr.is_tachycardic(100.0));
    }

    #[test]
    fn heart_rate_tachycardia() {
        let hr = HeartRate::new(120.0);
        assert!(!hr.is_bradycardic(60.0));
        assert!(hr.is_tachycardic(100.0));
    }

    #[test]
    fn circuit_display() {
        assert_eq!(format!("{}", Circuit::Pulmonary), "Pulmonary");
        assert_eq!(format!("{}", Circuit::Systemic), "Systemic");
        assert_eq!(format!("{}", Circuit::Portal), "Portal");
    }

    #[test]
    fn vessel_type_pressure_gradient() {
        let aorta_p = VesselType::Aorta.typical_pressure();
        let cap_p = VesselType::Capillary.typical_pressure();
        let vc_p = VesselType::VenaCava.typical_pressure();
        assert!(aorta_p > cap_p);
        assert!(cap_p > vc_p);
    }

    #[test]
    fn pathology_display() {
        let p = Pathology::Atherosclerosis {
            vessel: "coronary".to_string(),
            occlusion: 0.5,
        };
        let s = format!("{}", p);
        assert!(s.contains("Atherosclerosis"));
        assert!(s.contains("50%"));
    }

    #[test]
    fn blood_component_display() {
        assert_eq!(format!("{}", BloodComponent::RedCell), "Red Cell");
        assert_eq!(format!("{}", BloodComponent::Plasma), "Plasma");
    }

    #[test]
    fn component_role_descriptions() {
        let role = blood::component_role(BloodComponent::RedCell);
        assert!(role.contains("carrier"));
    }

    // ═══════════════════════════════════════════════════════════
    // Serialization round-trip tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn heart_serializes_round_trip() {
        let heart = Heart::new();
        let json = serde_json::to_string(&heart);
        assert!(json.is_ok());
        let parsed: Result<Heart, _> =
            serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
        assert!(parsed.is_ok());
    }

    #[test]
    fn vessel_serializes_round_trip() {
        let vessel = Vessel::new("aorta", VesselType::Aorta);
        let json = serde_json::to_string(&vessel);
        assert!(json.is_ok());
        let parsed: Result<Vessel, _> =
            serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
        assert!(parsed.is_ok());
    }

    #[test]
    fn pathology_serializes_round_trip() {
        let path = Pathology::Hypertension {
            pressure: Pressure::new(0.9),
            duration: 15,
        };
        let json = serde_json::to_string(&path);
        assert!(json.is_ok());
        let parsed: Result<Pathology, _> =
            serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
        assert!(parsed.is_ok());
    }

    #[test]
    fn diagnosis_serializes() {
        let d = Diagnosis {
            pathology: Pathology::Anemia {
                carrier_count: 50,
                required: 100,
            },
            severity: Severity::Moderate,
            recommendation: "Increase workers".to_string(),
        };
        let json = serde_json::to_string(&d);
        assert!(json.is_ok());
    }
}
