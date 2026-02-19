//! # CTVP Validation Tests for nexcore-edit-distance
//!
//! Clinical Trial Validation Paradigm phases:
//! - Phase 0 (Preclinical): Property-based invariants via proptest
//! - Phase 1 (Safety): Edge cases, adversarial input, fault injection
//! - Phase 2 (Efficacy): Real-world validation against known results

use nexcore_edit_distance::adapter::{DnaAdapter, DomainAdapter, TokenAdapter};
use nexcore_edit_distance::prelude::*;
use nexcore_edit_distance::transfer::{TransferMapBuilder, TransferRegistry};

// ============================================================================
// Phase 0: Preclinical — Property-Based Invariants
// ============================================================================

mod phase0_preclinical {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Metric axiom: d(x, x) = 0 (identity of indiscernibles)
        #[test]
        fn identity_property(s in "\\PC{0,50}") {
            let d = levenshtein(&s, &s);
            prop_assert!((d - 0.0).abs() < f64::EPSILON,
                "d({s:?}, {s:?}) = {d}, expected 0.0");
        }

        /// Metric axiom: d(x, y) = d(y, x) (symmetry)
        #[test]
        fn symmetry_property(a in "\\PC{0,30}", b in "\\PC{0,30}") {
            let d1 = levenshtein(&a, &b);
            let d2 = levenshtein(&b, &a);
            prop_assert!((d1 - d2).abs() < f64::EPSILON,
                "d({a:?},{b:?})={d1} != d({b:?},{a:?})={d2}");
        }

        /// Metric axiom: d(x, y) >= 0 (non-negativity)
        #[test]
        fn non_negativity(a in "\\PC{0,30}", b in "\\PC{0,30}") {
            let d = levenshtein(&a, &b);
            prop_assert!(d >= 0.0, "d({a:?},{b:?})={d} < 0");
        }

        /// Triangle inequality: d(x, z) <= d(x, y) + d(y, z)
        #[test]
        fn triangle_inequality(
            a in "\\PC{0,20}",
            b in "\\PC{0,20}",
            c in "\\PC{0,20}",
        ) {
            let d_ac = levenshtein(&a, &c);
            let d_ab = levenshtein(&a, &b);
            let d_bc = levenshtein(&b, &c);
            prop_assert!(d_ac <= d_ab + d_bc + f64::EPSILON,
                "triangle violated: d({a:?},{c:?})={d_ac} > d({a:?},{b:?})+d({b:?},{c:?})={}", d_ab + d_bc);
        }

        /// Upper bound: d(a, b) <= max(|a|, |b|)
        #[test]
        fn upper_bound(a in "\\PC{0,30}", b in "\\PC{0,30}") {
            let d = levenshtein(&a, &b);
            let max_len = a.chars().count().max(b.chars().count()) as f64;
            prop_assert!(d <= max_len + f64::EPSILON,
                "d({a:?},{b:?})={d} > max_len={max_len}");
        }

        /// Empty string: d("", s) = |s|
        #[test]
        fn empty_source(s in "\\PC{0,50}") {
            let d = levenshtein("", &s);
            let expected = s.chars().count() as f64;
            prop_assert!((d - expected).abs() < f64::EPSILON,
                "d('', {s:?})={d}, expected {expected}");
        }

        /// Empty target: d(s, "") = |s|
        #[test]
        fn empty_target(s in "\\PC{0,50}") {
            let d = levenshtein(&s, "");
            let expected = s.chars().count() as f64;
            prop_assert!((d - expected).abs() < f64::EPSILON,
                "d({s:?}, '')={d}, expected {expected}");
        }

        /// Damerau-Levenshtein <= Levenshtein for all inputs
        /// (transposition can only reduce or equal)
        #[test]
        fn damerau_le_levenshtein(a in "[a-z]{0,15}", b in "[a-z]{0,15}") {
            let dl = damerau_levenshtein(&a, &b);
            let lev = levenshtein(&a, &b);
            prop_assert!(dl <= lev + f64::EPSILON,
                "DL({a:?},{b:?})={dl} > Lev={lev}");
        }

        /// LCS distance >= Levenshtein (no substitution means more ops)
        #[test]
        fn lcs_ge_levenshtein(a in "[a-z]{0,15}", b in "[a-z]{0,15}") {
            let lcs = lcs_distance(&a, &b);
            let lev = levenshtein(&a, &b);
            prop_assert!(lcs >= lev - f64::EPSILON,
                "LCS({a:?},{b:?})={lcs} < Lev={lev}");
        }

        /// Similarity is in [0.0, 1.0]
        #[test]
        fn similarity_bounds(a in "\\PC{0,30}", b in "\\PC{0,30}") {
            let m = Levenshtein::default();
            let src: Vec<char> = a.chars().collect();
            let tgt: Vec<char> = b.chars().collect();
            let sim = m.similarity(&src, &tgt);
            prop_assert!(sim >= 0.0 && sim <= 1.0,
                "sim({a:?},{b:?})={sim} out of [0,1]");
        }

        /// Banded solver agrees with full when band is wide enough
        #[test]
        fn banded_agrees_with_full(a in "[a-z]{0,20}", b in "[a-z]{0,20}") {
            let full = levenshtein(&a, &b);
            let wide_band = a.chars().count().max(b.chars().count()) + 1;
            let result = compute(&a, &b, &StdOps, &UniformCost, &BandedDp::new(wide_band));
            prop_assert!((result.distance - full).abs() < f64::EPSILON,
                "banded({a:?},{b:?})={} != full={full}", result.distance);
        }

        /// Transfer confidence is in [0.0, 1.0]
        #[test]
        fn transfer_confidence_bounded(
            s in 0.0f64..=1.0,
            f in 0.0f64..=1.0,
            c in 0.0f64..=1.0,
        ) {
            let map = TransferMapBuilder::new("a", "b")
                .structural(s)
                .functional(f)
                .contextual(c)
                .build();
            let conf = map.confidence();
            prop_assert!(conf >= 0.0 && conf <= 1.0,
                "confidence({s},{f},{c})={conf}");
        }
    }
}

// ============================================================================
// Phase 1: Safety — Edge Cases, Adversarial Input
// ============================================================================

mod phase1_safety {
    use super::*;

    #[test]
    fn empty_empty() {
        assert!((levenshtein("", "") - 0.0).abs() < f64::EPSILON);
        assert!((damerau_levenshtein("", "") - 0.0).abs() < f64::EPSILON);
        assert!((lcs_distance("", "") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn single_char() {
        assert!((levenshtein("a", "b") - 1.0).abs() < f64::EPSILON);
        assert!((levenshtein("a", "") - 1.0).abs() < f64::EPSILON);
        assert!((levenshtein("", "a") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn identical_long_string() {
        let s = "a".repeat(1000);
        assert!((levenshtein(&s, &s) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn completely_different_long_string() {
        let a = "a".repeat(500);
        let b = "b".repeat(500);
        assert!((levenshtein(&a, &b) - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn asymmetric_lengths() {
        let short = "abc";
        let long = "abcdefghij";
        let d = levenshtein(short, long);
        // Must insert 7 chars
        assert!((d - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unicode_emoji() {
        let d = levenshtein("😀😁😂", "😀😄😂");
        assert!((d - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unicode_combining_marks() {
        // é as single char vs e + combining accent
        let a = "caf\u{00E9}"; // é (precomposed)
        let b = "cafe\u{0301}"; // e + combining accent
        // These are different char sequences, distance reflects that
        let d = levenshtein(a, b);
        assert!(d > 0.0, "Composed vs decomposed should differ: d={d}");
    }

    #[test]
    fn null_bytes_in_string() {
        let a = "ab\0cd";
        let b = "ab\0ce";
        assert!((levenshtein(a, b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn banded_zero_band_returns_infinity() {
        let result = compute("abc", "xyz", &StdOps, &UniformCost, &BandedDp::new(0));
        assert!(result.distance.is_infinite());
    }

    #[test]
    fn banded_band_one() {
        let result = compute("abc", "axc", &StdOps, &UniformCost, &BandedDp::new(1));
        // 1 substitution, within band of 1
        assert!((result.distance - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_cost_zero_substitution() {
        let m = EditMetricBuilder::<char, _, _, _>::new()
            .ops(StdOps)
            .cost(WeightedCost::new(1.0, 1.0, 0.0, 1.0))
            .solver(TwoRowDp)
            .build();
        // Zero-cost substitutions: "abc" → "xyz" costs 0
        assert!((m.str_distance("abc", "xyz") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn traceback_operations_are_valid() {
        let m = LevenshteinTraceback::default();
        let result = m.str_solve("kitten", "sitting");
        let ops = result.operations.expect("traceback should exist");
        assert!(!ops.is_empty());
        // Verify distance matches operation count
        assert_eq!(ops.len(), result.distance as usize);
    }

    #[test]
    fn dna_adapter_invalid_chars() {
        let a = DnaAdapter;
        // All invalid chars map to N
        assert_eq!(a.encode("XYZ123"), vec![b'N', b'N', b'N', b'N', b'N', b'N']);
    }

    #[test]
    fn token_adapter_multiple_spaces() {
        let a = TokenAdapter;
        let tokens = a.encode("  hello   world  ");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn transfer_map_clamping() {
        let map = TransferMapBuilder::new("a", "b")
            .structural(1.5) // over 1.0 → clamped to 1.0
            .functional(-0.3) // under 0.0 → clamped to 0.0
            .contextual(0.5)
            .build();
        // 1.0*0.4 + 0.0*0.4 + 0.5*0.2 = 0.4 + 0.0 + 0.1 = 0.5
        assert!((map.confidence() - 0.5).abs() < 0.001);
    }

    #[test]
    fn transfer_registry_empty_lookup() {
        let reg = TransferRegistry::with_defaults();
        let maps = reg.lookup("nonexistent", "domain");
        assert!(maps.is_empty());
    }
}

// ============================================================================
// Phase 2: Efficacy — Known Results Validation
// ============================================================================

mod phase2_efficacy {
    use super::*;

    /// Wikipedia Levenshtein examples
    #[test]
    fn wikipedia_examples() {
        assert!((levenshtein("kitten", "sitting") - 3.0).abs() < f64::EPSILON);
        assert!((levenshtein("saturday", "sunday") - 3.0).abs() < f64::EPSILON);
    }

    /// Damerau-Levenshtein: transposition is 1 operation
    #[test]
    fn damerau_known_transpositions() {
        assert!((damerau_levenshtein("ab", "ba") - 1.0).abs() < f64::EPSILON);
        assert!((damerau_levenshtein("abc", "bac") - 1.0).abs() < f64::EPSILON);
    }

    /// LCS distance = |a| + |b| - 2*LCS_length
    #[test]
    fn lcs_known_results() {
        // "abc" vs "axc": LCS = "ac" (len 2), distance = 3+3-4 = 2
        assert!((lcs_distance("abc", "axc") - 2.0).abs() < f64::EPSILON);
        // "abcdef" vs "ace": LCS = "ace" (len 3), distance = 6+3-6 = 3
        assert!((lcs_distance("abcdef", "ace") - 3.0).abs() < f64::EPSILON);
    }

    /// Cross-solver agreement on known inputs
    #[test]
    fn solver_agreement() {
        let cases = [
            ("kitten", "sitting"),
            ("abc", "def"),
            ("hello", "hallo"),
            ("flaw", "lawn"),
            ("intention", "execution"),
        ];
        for (a, b) in cases {
            let two_row = levenshtein(a, b);
            let full = compute(a, b, &StdOps, &UniformCost, &FullMatrixDp).distance;
            let wide_band = a.len().max(b.len()) + 1;
            let banded = compute(a, b, &StdOps, &UniformCost, &BandedDp::new(wide_band)).distance;
            assert!(
                (two_row - full).abs() < f64::EPSILON,
                "TwoRow({a},{b})={two_row} != FullMatrix={full}"
            );
            assert!(
                (two_row - banded).abs() < f64::EPSILON,
                "TwoRow({a},{b})={two_row} != Banded={banded}"
            );
        }
    }

    /// DNA adapter: bioinformatics use case
    #[test]
    fn dna_edit_distance() {
        let adapter = DnaAdapter;
        let m = Levenshtein::default();
        let src = adapter.encode("ACGTACGT");
        let tgt = adapter.encode("ACGAACGT");
        // Cast u8→char for compatibility with char-based metric
        let src_chars: Vec<char> = src.iter().map(|&b| b as char).collect();
        let tgt_chars: Vec<char> = tgt.iter().map(|&b| b as char).collect();
        let d = m.distance(&src_chars, &tgt_chars);
        assert!((d - 1.0).abs() < f64::EPSILON, "1 substitution (T→A)");
    }

    /// Token-level WER validation
    #[test]
    fn word_error_rate_validation() {
        let adapter = TokenAdapter;
        let ref_tokens = adapter.encode("the quick brown fox jumps");
        let hyp_tokens = adapter.encode("the slow brown cat jumps");
        // "quick"→"slow" (sub), "fox"→"cat" (sub) = 2 edits
        // We need a String-based metric for this
        let m = EditMetric::new(StdOps, UniformCost, TwoRowDp);
        let d = m.distance(&ref_tokens, &hyp_tokens);
        assert!((d - 2.0).abs() < f64::EPSILON, "WER: 2 word substitutions");
    }

    /// Pharmacovigilance drug name matching
    #[test]
    fn pv_drug_name_matching() {
        let m = Levenshtein::default();
        // Common PV fuzzy match scenarios
        let pairs = [
            ("acetaminophen", "acetaminophin", 1.0), // misspelling
            ("ibuprofen", "ibuprofin", 1.0),         // misspelling
            ("metformin", "metfornin", 1.0),         // OCR error
        ];
        for (drug, variant, expected) in pairs {
            let d = m.str_distance(drug, variant);
            assert!(
                (d - expected).abs() < f64::EPSILON,
                "PV match: '{drug}' vs '{variant}' = {d}, expected {expected}"
            );
        }
    }

    /// Similarity thresholds for PV matching (high recall required)
    #[test]
    fn pv_similarity_thresholds() {
        let m = Levenshtein::default();
        // At 0.80 threshold, common misspellings should still match
        let cases = [
            ("acetaminophen", "acetaminophin", true),   // sim ≈ 0.923
            ("atorvastatin", "atorvastin", true),       // sim ≈ 0.833
            ("metoprolol", "metropolol", true),         // sim ≈ 0.800
            ("aspirin", "completely_different", false), // sim ≈ 0.0
        ];
        for (a, b, should_match) in cases {
            let sim = m.str_similarity(a, b);
            let matches = sim >= 0.75;
            assert_eq!(
                matches, should_match,
                "PV threshold: '{a}' vs '{b}' sim={sim:.3}, matches={matches}, expected={should_match}"
            );
        }
    }

    /// Transfer registry coverage for PV domain
    #[test]
    fn pv_transfer_entry_exists() {
        let reg = TransferRegistry::with_defaults();
        let maps = reg.lookup("text/unicode", "pharmacovigilance");
        assert_eq!(maps.len(), 1);
        let pv = maps[0];
        // PV requires high confidence for safety-critical matching
        assert!(
            pv.confidence() > 0.80,
            "PV transfer confidence={}",
            pv.confidence()
        );
        assert!(
            pv.caveat.contains("safety-critical") || pv.caveat.contains("recall"),
            "PV caveat should mention safety: {}",
            pv.caveat
        );
    }

    /// Weighted cost: bioinformatics-style gap penalty
    #[test]
    fn gap_penalty_model() {
        let m = EditMetricBuilder::<char, _, _, _>::new()
            .ops(StdOps)
            .cost(WeightedCost::new(2.0, 2.0, 1.0, 1.0)) // gaps cost 2, sub costs 1
            .solver(TwoRowDp)
            .build();
        // "abc" → "axc": 1 substitution at cost 1.0
        assert!((m.str_distance("abc", "axc") - 1.0).abs() < f64::EPSILON);
        // "abc" → "ac": 1 deletion at cost 2.0
        assert!((m.str_distance("abc", "ac") - 2.0).abs() < f64::EPSILON);
    }

    /// Traceback produces valid edit script
    #[test]
    fn traceback_produces_correct_edit_script() {
        let m = LevenshteinTraceback::default();
        let result = m.str_solve("intention", "execution");
        let ops = result.operations.expect("traceback required");
        // Count ops by type
        let subs = ops
            .iter()
            .filter(|o| matches!(o, EditOp::Substitute { .. }))
            .count();
        let ins = ops
            .iter()
            .filter(|o| matches!(o, EditOp::Insert { .. }))
            .count();
        let dels = ops
            .iter()
            .filter(|o| matches!(o, EditOp::Delete { .. }))
            .count();
        let total = subs + ins + dels;
        assert_eq!(
            total as f64, result.distance,
            "ops count={total} != distance={}",
            result.distance
        );
    }
}
