//! # Tests for nexcore-relay
//!
//! Covers: RelayOutcome, FidelityBound/FidelityMetrics, Relay<I,O>,
//! RelayChain (two-hop composition), RelayStrategy.

#[cfg(test)]
mod outcome_tests {
    use nexcore_error::NexError;

    use crate::outcome::RelayOutcome;

    #[test]
    fn forwarded_is_forwarded() {
        let o: RelayOutcome<i32> = RelayOutcome::Forwarded(42);
        assert!(o.is_forwarded());
        assert!(!o.is_filtered());
        assert!(!o.is_failed());
    }

    #[test]
    fn filtered_is_filtered() {
        let o: RelayOutcome<i32> = RelayOutcome::Filtered;
        assert!(o.is_filtered());
        assert!(!o.is_forwarded());
        assert!(!o.is_failed());
    }

    #[test]
    fn failed_is_failed() {
        let o: RelayOutcome<i32> = RelayOutcome::Failed(NexError::new("boom"));
        assert!(o.is_failed());
        assert!(!o.is_forwarded());
        assert!(!o.is_filtered());
    }

    #[test]
    fn into_forwarded_extracts_value() {
        let o: RelayOutcome<u32> = RelayOutcome::Forwarded(7);
        assert_eq!(o.into_forwarded(), Some(7));
    }

    #[test]
    fn into_forwarded_returns_none_for_filtered() {
        let o: RelayOutcome<u32> = RelayOutcome::Filtered;
        assert_eq!(o.into_forwarded(), None);
    }

    #[test]
    fn into_forwarded_returns_none_for_failed() {
        let o: RelayOutcome<u32> = RelayOutcome::Failed(NexError::new("err"));
        assert_eq!(o.into_forwarded(), None);
    }

    #[test]
    fn map_transforms_forwarded_value() {
        let o: RelayOutcome<i32> = RelayOutcome::Forwarded(3);
        let mapped = o.map(|v| v * 2);
        assert_eq!(mapped.into_forwarded(), Some(6));
    }

    #[test]
    fn map_preserves_filtered() {
        let o: RelayOutcome<i32> = RelayOutcome::Filtered;
        let mapped = o.map(|v| v * 2);
        assert!(mapped.is_filtered());
    }

    #[test]
    fn map_preserves_failed() {
        let o: RelayOutcome<i32> = RelayOutcome::Failed(NexError::new("err"));
        let mapped = o.map(|v| v * 2);
        assert!(mapped.is_failed());
    }

    #[test]
    fn into_result_forwarded() {
        let o: RelayOutcome<i32> = RelayOutcome::Forwarded(99);
        let r = o.into_result();
        assert!(r.is_ok());
        assert_eq!(r.ok().flatten(), Some(99));
    }

    #[test]
    fn into_result_filtered() {
        let o: RelayOutcome<i32> = RelayOutcome::Filtered;
        let r = o.into_result();
        assert!(r.is_ok());
        assert_eq!(r.ok().flatten(), None);
    }

    #[test]
    fn into_result_failed() {
        let o: RelayOutcome<i32> = RelayOutcome::Failed(NexError::new("oops"));
        let r = o.into_result();
        assert!(r.is_err());
    }
}

#[cfg(test)]
mod fidelity_tests {
    use nexcore_primitives::relay::Fidelity;

    use crate::fidelity::{FidelityBound, FidelityMetrics};

    #[test]
    fn active_metrics_records_fidelity() {
        let m = FidelityMetrics::active("detect", 0.93);
        assert!(m.activated);
        assert!((m.fidelity.value() - 0.93).abs() < f64::EPSILON);
        assert_eq!(m.stage, "detect");
    }

    #[test]
    fn inactive_metrics_has_zero_fidelity() {
        let m = FidelityMetrics::inactive("blocked");
        assert!(!m.activated);
        assert!((m.fidelity.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn passes_a3_above_threshold() {
        let m = FidelityMetrics::active("hop", 0.85);
        assert!(m.passes_a3()); // 0.85 >= 0.80
    }

    #[test]
    fn fails_a3_below_threshold() {
        let m = FidelityMetrics::new("hop", 0.70, 0.80, true);
        assert!(!m.passes_a3()); // 0.70 < 0.80
    }

    #[test]
    fn dead_relay_detected_at_zero() {
        let m = FidelityMetrics::inactive("dead");
        assert!(m.is_dead_relay());
    }

    #[test]
    fn dead_relay_detected_at_one() {
        let m = FidelityMetrics::new("identity", 1.0, 0.80, true);
        assert!(m.is_dead_relay());
    }

    #[test]
    fn non_dead_relay_at_mid_fidelity() {
        let m = FidelityMetrics::active("signal", 0.92);
        assert!(!m.is_dead_relay());
    }

    #[test]
    fn signal_loss_is_complement() {
        let m = FidelityMetrics::active("hop", 0.85);
        assert!((m.signal_loss() - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn fidelity_bound_trait_via_metrics() {
        let m = FidelityMetrics::active("hop", 0.90);
        // FidelityMetrics implements FidelityBound
        assert!((m.fidelity().value() - 0.90).abs() < f64::EPSILON);
        assert!((m.min_fidelity() - 0.80).abs() < f64::EPSILON);
        assert!(m.meets_minimum());
    }

    #[test]
    fn fidelity_clamps_out_of_range() {
        let m = FidelityMetrics::new("hop", 1.5, 0.80, true);
        assert!((m.fidelity.value() - 1.0).abs() < f64::EPSILON);
        let m2 = FidelityMetrics::new("hop", -0.1, 0.80, true);
        assert!((m2.fidelity.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn fidelity_metrics_display_contains_stage() {
        let m = FidelityMetrics::active("faers_ingest", 0.98);
        let s = format!("{m}");
        assert!(s.contains("faers_ingest"));
        assert!(s.contains("A3=pass"));
    }

    // Struct literal test: verify Fidelity from nexcore-primitives is reachable
    #[test]
    fn fidelity_primitive_compose_via_metrics() {
        let a = FidelityMetrics::active("r1", 0.95);
        let b = FidelityMetrics::active("r2", 0.90);
        let composed = a.fidelity().compose(b.fidelity());
        assert!((composed.value() - 0.855).abs() < f64::EPSILON);
    }
}

#[cfg(test)]
mod relay_trait_tests {
    use nexcore_error::NexError;

    use crate::{outcome::RelayOutcome, relay::Relay};

    // --- Minimal Relay implementations for testing ---

    struct DoubleRelay;
    impl Relay<i32, i32> for DoubleRelay {
        fn process(&self, input: i32) -> RelayOutcome<i32> {
            RelayOutcome::Forwarded(input * 2)
        }
        fn stage_name(&self) -> &str {
            "double"
        }
    }

    struct ThresholdRelay {
        threshold: i32,
    }
    impl Relay<i32, i32> for ThresholdRelay {
        fn process(&self, input: i32) -> RelayOutcome<i32> {
            if input < self.threshold {
                RelayOutcome::Filtered
            } else {
                RelayOutcome::Forwarded(input)
            }
        }
        fn stage_name(&self) -> &str {
            "threshold"
        }
    }

    struct FailingRelay;
    impl Relay<i32, i32> for FailingRelay {
        fn process(&self, _input: i32) -> RelayOutcome<i32> {
            RelayOutcome::Failed(NexError::new("relay error"))
        }
    }

    struct StringifyRelay;
    impl Relay<i32, String> for StringifyRelay {
        fn process(&self, input: i32) -> RelayOutcome<String> {
            RelayOutcome::Forwarded(format!("value:{input}"))
        }
        fn stage_name(&self) -> &str {
            "stringify"
        }
    }

    #[test]
    fn relay_forwards_output() {
        let r = DoubleRelay;
        let out = r.process(5);
        assert_eq!(out.into_forwarded(), Some(10));
    }

    #[test]
    fn relay_filters_below_threshold() {
        let r = ThresholdRelay { threshold: 10 };
        let out = r.process(3);
        assert!(out.is_filtered());
    }

    #[test]
    fn relay_forwards_above_threshold() {
        let r = ThresholdRelay { threshold: 10 };
        let out = r.process(15);
        assert_eq!(out.into_forwarded(), Some(15));
    }

    #[test]
    fn relay_can_fail() {
        let r = FailingRelay;
        let out = r.process(1);
        assert!(out.is_failed());
    }

    #[test]
    fn relay_min_fidelity_default() {
        let r = DoubleRelay;
        assert!((r.min_fidelity() - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn relay_stage_name_returns_name() {
        let r = DoubleRelay;
        assert_eq!(r.stage_name(), "double");
    }

    #[test]
    fn relay_type_boundary_i32_to_string() {
        let r = StringifyRelay;
        let out = r.process(42);
        assert_eq!(out.into_forwarded().as_deref(), Some("value:42"));
    }
}

#[cfg(test)]
mod chain_tests {
    use nexcore_error::NexError;

    use crate::{chain::RelayChain, outcome::RelayOutcome, relay::Relay};

    // --- Relay stubs for chain composition ---

    struct AddOneRelay;
    impl Relay<i32, i32> for AddOneRelay {
        fn process(&self, input: i32) -> RelayOutcome<i32> {
            RelayOutcome::Forwarded(input + 1)
        }
        fn min_fidelity(&self) -> f64 {
            0.95
        }
        fn stage_name(&self) -> &str {
            "add_one"
        }
    }

    struct MultiplyRelay(i32);
    impl Relay<i32, i32> for MultiplyRelay {
        fn process(&self, input: i32) -> RelayOutcome<i32> {
            RelayOutcome::Forwarded(input * self.0)
        }
        fn min_fidelity(&self) -> f64 {
            0.90
        }
        fn stage_name(&self) -> &str {
            "multiply"
        }
    }

    struct FilterNegativeRelay;
    impl Relay<i32, i32> for FilterNegativeRelay {
        fn process(&self, input: i32) -> RelayOutcome<i32> {
            if input < 0 {
                RelayOutcome::Filtered
            } else {
                RelayOutcome::Forwarded(input)
            }
        }
    }

    struct AlwaysFailRelay;
    impl Relay<i32, i32> for AlwaysFailRelay {
        fn process(&self, _: i32) -> RelayOutcome<i32> {
            RelayOutcome::Failed(NexError::new("stage failed"))
        }
    }

    struct StringifyRelay;
    impl Relay<i32, String> for StringifyRelay {
        fn process(&self, input: i32) -> RelayOutcome<String> {
            RelayOutcome::Forwarded(format!("{input}"))
        }
        fn stage_name(&self) -> &str {
            "stringify"
        }
    }

    #[test]
    fn chain_composes_two_hops() {
        let chain = RelayChain::safety_critical(AddOneRelay, MultiplyRelay(3));
        let out = chain.process(4); // (4+1)*3 = 15
        assert_eq!(out.into_forwarded(), Some(15));
    }

    #[test]
    fn chain_short_circuits_on_filter_at_r1() {
        let chain = RelayChain::safety_critical(FilterNegativeRelay, MultiplyRelay(10));
        let out = chain.process(-5);
        assert!(out.is_filtered());
    }

    #[test]
    fn chain_short_circuits_on_fail_at_r1() {
        let chain = RelayChain::safety_critical(AlwaysFailRelay, MultiplyRelay(10));
        let out = chain.process(5);
        assert!(out.is_failed());
    }

    #[test]
    fn chain_r2_can_filter_after_r1_forwards() {
        let chain = RelayChain::safety_critical(AddOneRelay, FilterNegativeRelay);
        // AddOne(5) = 6, FilterNegative(6) = Forwarded(6)
        let out = chain.process(5);
        assert_eq!(out.into_forwarded(), Some(6));
    }

    #[test]
    fn chain_total_fidelity_is_multiplicative() {
        let chain = RelayChain::safety_critical(AddOneRelay, MultiplyRelay(3));
        chain.process(1);
        // AddOneRelay.min_fidelity = 0.95, MultiplyRelay.min_fidelity = 0.90
        let expected = 0.95 * 0.90;
        assert!((chain.total_fidelity().value() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn chain_verify_preservation_passes_when_total_above_fmin() {
        // f_min=0.80, total=0.95*0.90=0.855 → passes
        let chain = RelayChain::new(AddOneRelay, MultiplyRelay(3), 0.80);
        chain.process(1);
        assert!(chain.verify_preservation());
    }

    #[test]
    fn chain_verify_preservation_fails_when_total_below_fmin() {
        // f_min=0.90, total=0.95*0.90=0.855 → fails
        let chain = RelayChain::new(AddOneRelay, MultiplyRelay(3), 0.90);
        chain.process(1);
        assert!(!chain.verify_preservation());
    }

    #[test]
    fn chain_metrics_reflect_stage_names() {
        let chain = RelayChain::safety_critical(AddOneRelay, MultiplyRelay(2));
        chain.process(3);
        assert_eq!(chain.r1_metrics().stage, "add_one");
        assert_eq!(chain.r2_metrics().stage, "multiply");
    }

    #[test]
    fn chain_implements_relay_trait() {
        // RelayChain itself implements Relay<I,O>
        let chain = RelayChain::safety_critical(AddOneRelay, MultiplyRelay(2));
        assert!((chain.min_fidelity() - 0.80).abs() < f64::EPSILON);
        assert_eq!(chain.stage_name(), "RelayChain");
    }

    #[test]
    fn chain_heterogeneous_types_i32_to_string() {
        let chain = RelayChain::safety_critical(AddOneRelay, StringifyRelay);
        let out = chain.process(9); // 9+1=10, stringify → "10"
        assert_eq!(out.into_forwarded().as_deref(), Some("10"));
    }
}

#[cfg(test)]
mod strategy_tests {
    use crate::strategy::RelayStrategy;

    #[test]
    fn df_is_safety_critical() {
        assert!(RelayStrategy::DecodeAndForward.is_safety_critical());
    }

    #[test]
    fn cf_is_not_safety_critical() {
        assert!(!RelayStrategy::CompressAndForward.is_safety_critical());
    }

    #[test]
    fn af_is_not_safety_critical() {
        assert!(!RelayStrategy::AmplifyAndForward.is_safety_critical());
    }

    #[test]
    fn typical_min_fidelity_ordering() {
        // DF > CF > AF
        assert!(
            RelayStrategy::DecodeAndForward.typical_min_fidelity()
                > RelayStrategy::CompressAndForward.typical_min_fidelity()
        );
        assert!(
            RelayStrategy::CompressAndForward.typical_min_fidelity()
                > RelayStrategy::AmplifyAndForward.typical_min_fidelity()
        );
    }

    #[test]
    fn abbreviations_correct() {
        assert_eq!(RelayStrategy::DecodeAndForward.abbreviation(), "DF");
        assert_eq!(RelayStrategy::CompressAndForward.abbreviation(), "CF");
        assert_eq!(RelayStrategy::AmplifyAndForward.abbreviation(), "AF");
    }

    #[test]
    fn af_may_be_dead_relay() {
        assert!(RelayStrategy::AmplifyAndForward.may_be_dead_relay());
    }

    #[test]
    fn df_cf_not_dead_relay_candidates() {
        assert!(!RelayStrategy::DecodeAndForward.may_be_dead_relay());
        assert!(!RelayStrategy::CompressAndForward.may_be_dead_relay());
    }

    #[test]
    fn display_contains_abbreviation() {
        let s = format!("{}", RelayStrategy::DecodeAndForward);
        assert!(s.contains("DF"));
        let s2 = format!("{}", RelayStrategy::CompressAndForward);
        assert!(s2.contains("CF"));
    }

    #[test]
    fn serde_roundtrip() {
        let s = RelayStrategy::DecodeAndForward;
        let json = serde_json::to_string(&s);
        assert!(json.is_ok());
        if let Ok(j) = json {
            let back: Result<RelayStrategy, _> = serde_json::from_str(&j);
            assert!(back.is_ok());
            assert_eq!(back.ok(), Some(RelayStrategy::DecodeAndForward));
        }
    }
}
