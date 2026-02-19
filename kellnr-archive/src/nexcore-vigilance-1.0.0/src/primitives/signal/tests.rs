//! Tests for signal primitives.

use super::*;

mod atom_tests {
    use super::*;

    #[test]
    fn count_arithmetic() {
        let a = Count::new(10);
        let b = Count::new(5);

        assert_eq!((a + b).value(), 15);
        assert_eq!((a - b).value(), 5);

        // Saturating subtraction
        assert_eq!((b - a).value(), 0);
    }

    #[test]
    fn frequency_from_count() {
        let num = Count::new(15);
        let denom = Count::new(115);

        let freq = Frequency::from_count(num, denom).unwrap();
        assert!((freq.value() - 0.130434).abs() < 0.0001);

        // Division by zero
        assert!(Frequency::from_count(num, Count::ZERO).is_none());
    }

    #[test]
    fn frequency_validation() {
        assert!(Frequency::new(0.5).is_some());
        assert!(Frequency::new(0.0).is_some());
        assert!(Frequency::new(-0.1).is_none());
        assert!(Frequency::new(f64::NAN).is_none());
        assert!(Frequency::new(f64::INFINITY).is_none());
    }

    #[test]
    fn ratio_from_frequencies() {
        let f1 = Frequency::new(0.2).unwrap();
        let f2 = Frequency::new(0.1).unwrap();

        let ratio = Ratio::from_frequencies(f1, f2).unwrap();
        assert!((ratio.value() - 2.0).abs() < f64::EPSILON);

        // Division by zero
        assert!(Ratio::from_frequencies(f1, Frequency::ZERO).is_none());
    }

    #[test]
    fn ratio_properties() {
        let elevated = Ratio::new(2.5).unwrap();
        let unity = Ratio::UNITY;
        let depressed = Ratio::new(0.5).unwrap();

        assert!(elevated.is_elevated());
        assert!(!unity.is_elevated());
        assert!(unity.is_unity());
        assert!(!depressed.is_elevated());
    }

    #[test]
    fn threshold_comparison() {
        let ratio = Ratio::new(2.5).unwrap();

        assert!(exceeds_threshold(ratio, Threshold::STANDARD).is_signal());
        assert!(!exceeds_threshold(ratio, Threshold::STRICT).is_signal());
        assert!(exceeds_threshold(ratio, Threshold::SENSITIVE).is_signal());
    }

    #[test]
    fn detected_semantics() {
        let signal = Detected::YES;
        let noise = Detected::NO;

        assert!(signal.is_signal());
        assert!(!signal.is_noise());
        assert!(noise.is_noise());
        assert!(!noise.is_signal());
    }

    #[test]
    fn source_variants() {
        let known = Source::known("FAERS");
        let unknown = Source::unknown("mystery");

        assert!(known.is_known());
        assert!(!unknown.is_known());
        assert_eq!(known.name(), "FAERS");
        assert_eq!(unknown.name(), "mystery");
    }

    #[test]
    fn timestamp_ordering() {
        let t1 = Timestamp::new(1000);
        let t2 = Timestamp::new(2000);

        assert!(t1.before(t2));
        assert!(t2.after(t1));
        assert_eq!(t1.duration_to(t2), 1000);
    }

    #[test]
    fn timestamp_from_secs() {
        let ts = Timestamp::from_secs(60);
        assert_eq!(ts.millis(), 60_000);
        assert_eq!(ts.secs(), 60);
    }

    #[test]
    fn association_basic() {
        let assoc = Association::new("aspirin", "bleeding");

        assert_eq!(assoc.exposure(), "aspirin");
        assert_eq!(assoc.outcome(), "bleeding");
        assert_eq!(format!("{assoc}"), "aspirin → bleeding");

        let reversed = assoc.reverse();
        assert_eq!(reversed.exposure(), "bleeding");
        assert_eq!(reversed.outcome(), "aspirin");
    }

    #[test]
    fn method_properties() {
        assert_eq!(Method::PRR.abbrev(), "PRR");
        assert_eq!(Method::ROR.abbrev(), "ROR");
        assert_eq!(Method::IC.abbrev(), "IC");
        assert_eq!(Method::EBGM.abbrev(), "EBGM");
        assert_eq!(Method::ChiSquare.abbrev(), "χ²");

        assert!((Method::PRR.default_threshold().value() - 2.0).abs() < f64::EPSILON);
        assert!((Method::IC.default_threshold().value() - 0.0).abs() < f64::EPSILON);
    }
}

mod composite_tests {
    use super::*;

    fn test_table() -> Table {
        // Classic aspirin-bleeding example
        Table::from_raw(15, 100, 20, 10_000)
    }

    #[test]
    fn table_marginals() {
        let t = test_table();

        assert_eq!(t.total().value(), 10_135);
        assert_eq!(t.exposed_total().value(), 115);
        assert_eq!(t.unexposed_total().value(), 10_020);
        assert_eq!(t.outcome_total().value(), 35);
        assert_eq!(t.no_outcome_total().value(), 10_100);
    }

    #[test]
    fn table_frequencies() {
        let t = test_table();

        let f_exp = t.freq_exposed().unwrap();
        let f_unexp = t.freq_unexposed().unwrap();

        // 15/115 = 0.1304...
        assert!((f_exp.value() - 0.1304).abs() < 0.001);
        // 20/10020 = 0.001996...
        assert!((f_unexp.value() - 0.001996).abs() < 0.001);
    }

    #[test]
    fn table_prr() {
        let t = test_table();
        let prr = t.prr().unwrap();

        // PRR = 0.1304 / 0.001996 = 65.3
        assert!(prr.value() > 2.0);
        assert!(prr.is_elevated());
    }

    #[test]
    fn table_ror() {
        let t = test_table();
        let ror = t.ror().unwrap();

        // ROR = (15*10000) / (100*20) = 75.0
        assert!((ror.value() - 75.0).abs() < 0.1);
    }

    #[test]
    fn table_chi_square() {
        let t = test_table();
        let chi = t.chi_square().unwrap();

        // Should be highly significant
        assert!(chi > Threshold::CHI_SQUARE_CRITICAL.value());
    }

    #[test]
    fn table_information_component() {
        let t = test_table();
        let ic = t.information_component().unwrap();

        // IC should be positive for elevated signal
        assert!(ic > 0.0);
    }

    #[test]
    fn table_from_u32() {
        let t = Table::from_u32(15, 100, 20, 10_000);
        assert_eq!(t.a.value(), 15);
        assert_eq!(t.b.value(), 100);
        assert_eq!(t.c.value(), 20);
        assert_eq!(t.d.value(), 10_000);
        assert_eq!(t.total().value(), 10_135);
    }

    #[test]
    fn table_from_u32_matches_from_raw() {
        let from_u32 = Table::from_u32(15, 100, 20, 10_000);
        let from_raw = Table::from_raw(15, 100, 20, 10_000);
        assert_eq!(from_u32, from_raw);
    }

    #[test]
    fn table_empty() {
        let empty = Table::default();

        assert!(!empty.is_valid());
        assert!(empty.prr().is_none());
        assert!(empty.ror().is_none());
        assert!(empty.chi_square().is_none());
    }

    #[test]
    fn table_ebgm_strong_signal() {
        let t = test_table(); // 15, 100, 20, 10_000
        let ebgm = t.ebgm();
        assert!(ebgm.is_some());

        let ebgm = ebgm.unwrap_or(Ratio::UNITY);
        // EBGM should be elevated for this strong signal
        assert!(ebgm.is_elevated(), "EBGM {} should be >= 1.0", ebgm.value());
    }

    #[test]
    fn table_ebgm_shrinkage() {
        // EBGM uses Bayesian shrinkage — it should be less extreme than raw ratio
        let t = Table::from_raw(3, 97, 300, 9600);
        let ebgm = t.ebgm();
        assert!(ebgm.is_some());

        let ebgm_val = ebgm.unwrap_or(Ratio::UNITY).value();
        let raw_ratio = t.a.as_f64() / t.expected().unwrap_or(1.0);

        // EBGM should show shrinkage (be closer to 1.0 than raw ratio)
        assert!(
            ebgm_val < raw_ratio,
            "EBGM {ebgm_val:.3} should be < raw ratio {raw_ratio:.3} (shrinkage)"
        );
    }

    #[test]
    fn table_ebgm_with_interval() {
        let t = test_table();
        let result = t.ebgm_with_interval();
        assert!(result.is_some());

        let (ebgm, interval) = result.unwrap_or_else(|| {
            (
                Ratio::UNITY,
                Interval::new(0.5, 2.0, 0.90).unwrap_or_else(|| Interval::ci95(0.5, 2.0).unwrap()),
            )
        });

        // EB05 < EBGM < EB95
        assert!(interval.lower() <= ebgm.value());
        assert!(ebgm.value() <= interval.upper());

        // 90% credibility interval
        assert!((interval.level() - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn table_ebgm_empty() {
        let empty = Table::default();
        assert!(empty.ebgm().is_none());
        assert!(empty.ebgm_with_interval().is_none());
    }

    #[test]
    fn table_ebgm_zero_cases() {
        let t = Table::from_raw(0, 100, 100, 9800);
        // Should still compute (prior-dominated), but not be a signal
        let result = t.ebgm();
        // With 0 cases, the expected value calculation might still work
        // but the EBGM should be very low
        if let Some(ebgm) = result {
            assert!(
                ebgm.value() < 2.0,
                "Zero cases should not produce signal-level EBGM"
            );
        }
    }

    #[test]
    fn interval_properties() {
        let ci = Interval::ci95(1.5, 3.5).unwrap();

        assert!((ci.lower() - 1.5).abs() < f64::EPSILON);
        assert!((ci.upper() - 3.5).abs() < f64::EPSILON);
        assert!((ci.level() - 0.95).abs() < f64::EPSILON);
        assert!((ci.width() - 2.0).abs() < f64::EPSILON);
        assert!((ci.midpoint() - 2.5).abs() < f64::EPSILON);

        // Excludes null (1.0)
        assert!(ci.excludes_null_ratio());
        assert!(ci.excludes(0.5));
        assert!(!ci.excludes(2.0));
    }

    #[test]
    fn interval_invalid() {
        // Lower > upper
        assert!(Interval::new(3.0, 1.0, 0.95).is_none());
        // Level out of range
        assert!(Interval::new(1.0, 3.0, 1.5).is_none());
    }

    #[test]
    fn signal_from_table() {
        let t = test_table();
        let signal = Signal::from_table_evans(t, Source::known("FAERS")).unwrap();

        assert!(signal.is_signal());
        assert_eq!(signal.case_count().value(), 15);
        assert!(signal.ratio.is_elevated());
        assert_eq!(signal.method(), Method::PRR);
    }

    #[test]
    fn prr_with_confidence_interval() {
        let t = test_table();
        let (prr, ci) = t.prr_with_ci().unwrap();

        // PRR should be elevated
        assert!(prr.is_elevated());

        // CI should exclude null (1.0)
        assert!(ci.excludes_null_ratio());
        assert!(ci.lower() > 1.0);
    }

    #[test]
    fn ror_with_confidence_interval() {
        let t = test_table();
        let (ror, ci) = t.ror_with_ci().unwrap();

        // ROR = 75.0 approximately
        assert!((ror.value() - 75.0).abs() < 1.0);

        // CI should exclude null
        assert!(ci.excludes_null_ratio());
    }

    #[test]
    fn signal_with_association() {
        let t = test_table();
        let assoc = Association::new("aspirin", "GI_bleeding");
        let signal = Signal::from_table_with_association(t, Source::known("FAERS"), assoc).unwrap();

        assert!(signal.is_signal());
        let a = signal.association().unwrap();
        assert_eq!(a.exposure(), "aspirin");
        assert_eq!(a.outcome(), "GI_bleeding");
    }

    #[test]
    fn signal_from_table_ebgm() {
        let t = Table::from_raw(20, 80, 100, 9800);
        let signal = Signal::from_table_ebgm(t, Source::known("FAERS"));
        assert!(signal.is_some());

        let signal =
            signal.unwrap_or_else(|| Signal::from_table_evans(t, Source::known("FAERS")).unwrap());
        assert_eq!(signal.method(), Method::EBGM);
        // Chi-square should be None for EBGM (Bayesian method)
        assert!(signal.chi_square.is_none());
        // Should have an interval (EB05/EB95)
        assert!(signal.interval.is_some());
    }

    #[test]
    fn signal_builder_pattern() {
        let t = test_table();
        let signal = Signal::from_table_evans(t, Source::known("FAERS"))
            .unwrap()
            .with_method(Method::ROR)
            .with_timestamp(Timestamp::new(1234567890))
            .with_association(Association::new("drug", "event"));

        assert_eq!(signal.method(), Method::ROR);
        assert_eq!(signal.timestamp().millis(), 1234567890);
        assert!(signal.association().is_some());
    }

    #[test]
    fn signal_strength_classification() {
        let none = SignalStrength::from_ratio(Ratio::new(1.0).unwrap());
        let weak = SignalStrength::from_ratio(Ratio::new(1.8).unwrap());
        let moderate = SignalStrength::from_ratio(Ratio::new(2.5).unwrap());
        let strong = SignalStrength::from_ratio(Ratio::new(4.0).unwrap());
        let critical = SignalStrength::from_ratio(Ratio::new(10.0).unwrap());

        assert_eq!(none, SignalStrength::None);
        assert_eq!(weak, SignalStrength::Weak);
        assert_eq!(moderate, SignalStrength::Moderate);
        assert_eq!(strong, SignalStrength::Strong);
        assert_eq!(critical, SignalStrength::Critical);

        // Action warranted
        assert!(!none.warrants_action());
        assert!(!weak.warrants_action());
        assert!(!moderate.warrants_action());
        assert!(strong.warrants_action());
        assert!(critical.warrants_action());

        // Investigation warranted
        assert!(moderate.warrants_investigation());
        assert!(strong.warrants_investigation());
    }

    #[test]
    fn signal_lifecycle_transitions() {
        let new = SignalLifecycle::New;
        let review = SignalLifecycle::UnderReview;
        let confirmed = SignalLifecycle::Confirmed;
        let closed = SignalLifecycle::Closed;

        // Valid transitions
        assert!(new.can_transition_to(SignalLifecycle::UnderReview));
        assert!(review.can_transition_to(SignalLifecycle::Confirmed));
        assert!(review.can_transition_to(SignalLifecycle::Escalated));
        assert!(confirmed.can_transition_to(SignalLifecycle::Closed));

        // Invalid transitions
        assert!(!new.can_transition_to(SignalLifecycle::Confirmed));
        assert!(!closed.can_transition_to(SignalLifecycle::New)); // Terminal — irreversible (∝)

        // Lifecycle properties
        assert!(closed.is_terminal());
        assert!(confirmed.is_actionable());
        assert!(new.is_pending());
    }
}

mod integration_tests {
    use super::*;

    /// End-to-end signal detection using primitives.
    #[test]
    fn full_detection_pipeline() {
        // T1: Raw counts (observed data)
        let a = Count::new(15); // Drug + Event
        let b = Count::new(100); // Drug + No Event
        let c = Count::new(20); // No Drug + Event
        let d = Count::new(10_000); // No Drug + No Event

        // T2-C: Build contingency table
        let table = Table::new(a, b, c, d);
        assert!(table.is_valid());

        // T2-P: Compute frequencies
        let f_exp = table.freq_exposed().unwrap();
        let f_unexp = table.freq_unexposed().unwrap();

        // T2-P: Compute ratio (PRR)
        let prr = compute_ratio(f_exp, f_unexp).unwrap();

        // T2-P: Apply threshold
        let detected = exceeds_threshold(prr, Threshold::STANDARD);

        // T2-C: Classify strength
        let strength = SignalStrength::from_ratio(prr);

        // Assertions
        assert!(prr.is_elevated());
        assert!(detected.is_signal());
        assert!(strength.warrants_action());

        // T2-C: Build full signal
        let signal = Signal::from_table_evans(table, Source::known("TestData")).unwrap();
        assert!(signal.is_signal());
    }

    /// Test transfer to different domain semantics.
    #[test]
    fn cross_domain_transfer() {
        // Same underlying statistics work for any domain
        let table = Table::from_raw(50, 200, 30, 5000);

        // Pharmacovigilance interpretation
        // a=50: patients with drug AND adverse event
        let pv_prr = table.prr().unwrap();
        assert!(pv_prr.is_elevated());

        // Finance interpretation (same math)
        // a=50: trades in condition X AND price drop
        let finance_ratio = table.prr().unwrap();
        assert_eq!(pv_prr, finance_ratio);

        // Epidemiology interpretation (same math)
        // a=50: people with exposure AND disease
        let epi_rr = table.prr().unwrap();
        assert_eq!(pv_prr, epi_rr);

        // The T1/T2-P primitives are truly universal
    }
}
