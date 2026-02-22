//! Property-based fuzz tests for PV signal detection algorithms.
//!
//! Uses proptest to verify safety and correctness invariants across the full
//! input space for the core disproportionality algorithms.
//!
//! ## Properties Tested
//!
//! 1. **Construction Safety** — `ContingencyTable::new` always succeeds
//! 2. **PRR No-Panic** — `calculate_prr` never panics on any input
//! 3. **PRR Result Invariants** — Output satisfies mathematical constraints
//! 4. **ROR No-Panic** — `calculate_ror` never panics on any input
//! 5. **ROR Result Invariants** — Output satisfies mathematical constraints
//! 6. **Complete Data** — All methods return finite, positive estimates for well-formed tables
//! 7. **Min-Cases Gate** — Signal detection respects the minimum case count threshold
//!
//! ## Run
//!
//! ```bash
//! cargo test -p nexcore-vigilance -- pv_fuzz
//! ```

use crate::pv::signals::{ContingencyTable, SignalCriteria, calculate_prr, calculate_ror};
use proptest::prelude::*;

/// Maximum cell value that keeps totals well within u64::MAX.
/// 4 cells × 10_000_000 = 40_000_000, far below u64::MAX (~1.8 × 10^19).
const MAX_CELL: u64 = 10_000_000;

// ============================================================================
// Property 1: ContingencyTable construction always succeeds
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_contingency_table_fields_preserved(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        prop_assert_eq!(table.a, a, "field 'a' not preserved");
        prop_assert_eq!(table.b, b, "field 'b' not preserved");
        prop_assert_eq!(table.c, c, "field 'c' not preserved");
        prop_assert_eq!(table.d, d, "field 'd' not preserved");
        prop_assert_eq!(table.total(), a + b + c + d, "total mismatch");
    }
}

// ============================================================================
// Property 2: calculate_prr never panics
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_prr_never_panics(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();
        // Must not panic — Result return type absorbs all error paths
        let _result = calculate_prr(&table, &criteria);
    }
}

// ============================================================================
// Property 3: calculate_prr result invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_prr_ok_point_estimate_nonnegative(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_prr(&table, &criteria) {
            prop_assert!(
                result.point_estimate >= 0.0,
                "PRR point_estimate must be >= 0, got {}",
                result.point_estimate
            );
        }
        // Err path is acceptable — empty table or zero non-drug events
    }

    #[test]
    fn prop_prr_case_count_matches_table_a(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_prr(&table, &criteria) {
            prop_assert_eq!(
                result.case_count, a,
                "case_count must equal table.a ({})", a
            );
            prop_assert_eq!(
                result.total_reports, table.total(),
                "total_reports must equal table.total()"
            );
        }
    }

    #[test]
    fn prop_prr_zero_cases_not_a_signal(
        b in 0u64..MAX_CELL,
        c in 1u64..MAX_CELL, // c > 0 so table is non-empty
        d in 0u64..MAX_CELL,
    ) {
        // a == 0: null result, never a signal
        let table = ContingencyTable::new(0, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_prr(&table, &criteria) {
            prop_assert!(
                !result.is_signal,
                "PRR signal with zero cases is impossible"
            );
            prop_assert_eq!(
                result.point_estimate, 0.0,
                "PRR with a=0 must have point_estimate == 0"
            );
        }
    }

    #[test]
    fn prop_prr_below_min_cases_not_a_signal(
        // a in 1..3 (below Evans min_cases=3)
        a in 1u64..3u64,
        b in 1u64..MAX_CELL,
        c in 1u64..MAX_CELL,
        d in 1u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_prr(&table, &criteria) {
            prop_assert!(
                !result.is_signal,
                "PRR signal with a={} < min_cases={} is impossible",
                a,
                criteria.min_cases
            );
        }
    }

    #[test]
    fn prop_prr_err_only_on_valid_conditions(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        if calculate_prr(&table, &criteria).is_err() {
            // Errors are only valid when:
            // 1. Table is empty (total == 0), OR
            // 2. Non-drug event rate is zero (c == 0) with drug+event cases (a > 0)
            let is_empty = table.total() == 0;
            let is_zero_non_drug_events = c == 0 && a > 0;
            prop_assert!(
                is_empty || is_zero_non_drug_events,
                "Unexpected PRR error: a={}, b={}, c={}, d={}",
                a, b, c, d
            );
        }
    }
}

// ============================================================================
// Property 4: calculate_ror never panics
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_ror_never_panics(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();
        let _result = calculate_ror(&table, &criteria);
    }
}

// ============================================================================
// Property 5: calculate_ror result invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_ror_ok_point_estimate_nonnegative(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_ror(&table, &criteria) {
            prop_assert!(
                result.point_estimate >= 0.0,
                "ROR point_estimate must be >= 0, got {}",
                result.point_estimate
            );
        }
    }

    #[test]
    fn prop_ror_case_count_matches_table_a(
        a in 0u64..MAX_CELL,
        b in 0u64..MAX_CELL,
        c in 0u64..MAX_CELL,
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_ror(&table, &criteria) {
            prop_assert_eq!(
                result.case_count, a,
                "case_count must equal table.a ({})", a
            );
        }
    }

    #[test]
    fn prop_ror_zero_cases_not_a_signal(
        b in 0u64..MAX_CELL,
        c in 1u64..MAX_CELL, // non-empty table
        d in 0u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(0, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_ror(&table, &criteria) {
            prop_assert!(
                !result.is_signal,
                "ROR signal with zero cases is impossible"
            );
        }
    }

    #[test]
    fn prop_ror_below_min_cases_not_a_signal(
        a in 1u64..3u64,
        b in 1u64..MAX_CELL,
        c in 1u64..MAX_CELL,
        d in 1u64..MAX_CELL,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        if let Ok(result) = calculate_ror(&table, &criteria) {
            prop_assert!(
                !result.is_signal,
                "ROR signal with a={} < min_cases={} is impossible",
                a,
                criteria.min_cases
            );
        }
    }
}

// ============================================================================
// Property 6: Complete data (all cells >= 1) — finite, positive estimates
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_prr_finite_positive_for_complete_table(
        a in 1u64..10_000u64,
        b in 1u64..10_000u64,
        c in 1u64..10_000u64,
        d in 1u64..10_000u64,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        match calculate_prr(&table, &criteria) {
            Ok(result) => {
                prop_assert!(
                    result.point_estimate.is_finite(),
                    "PRR must be finite for complete table, got {}",
                    result.point_estimate
                );
                prop_assert!(
                    result.point_estimate > 0.0,
                    "PRR must be positive for complete table (a>0), got {}",
                    result.point_estimate
                );
                // Confidence intervals must be finite when computed from complete data
                prop_assert!(
                    result.lower_ci.is_finite(),
                    "PRR lower_ci must be finite, got {}",
                    result.lower_ci
                );
                prop_assert!(
                    result.upper_ci.is_finite(),
                    "PRR upper_ci must be finite, got {}",
                    result.upper_ci
                );
                // Lower CI must be <= point estimate <= upper CI
                prop_assert!(
                    result.lower_ci <= result.point_estimate,
                    "lower_ci ({}) > point_estimate ({})",
                    result.lower_ci,
                    result.point_estimate
                );
                prop_assert!(
                    result.point_estimate <= result.upper_ci,
                    "point_estimate ({}) > upper_ci ({})",
                    result.point_estimate,
                    result.upper_ci
                );
            }
            Err(e) => {
                prop_assert!(
                    false,
                    "PRR must succeed for complete table (all cells >= 1): {:?}",
                    e
                );
            }
        }
    }

    #[test]
    fn prop_ror_finite_positive_for_complete_table(
        a in 1u64..10_000u64,
        b in 1u64..10_000u64,
        c in 1u64..10_000u64,
        d in 1u64..10_000u64,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        match calculate_ror(&table, &criteria) {
            Ok(result) => {
                prop_assert!(
                    result.point_estimate.is_finite(),
                    "ROR must be finite for complete table, got {}",
                    result.point_estimate
                );
                prop_assert!(
                    result.point_estimate > 0.0,
                    "ROR must be positive for complete table (a>0), got {}",
                    result.point_estimate
                );
                prop_assert!(
                    result.lower_ci.is_finite(),
                    "ROR lower_ci must be finite, got {}",
                    result.lower_ci
                );
                prop_assert!(
                    result.upper_ci.is_finite(),
                    "ROR upper_ci must be finite, got {}",
                    result.upper_ci
                );
                prop_assert!(
                    result.lower_ci <= result.point_estimate,
                    "lower_ci ({}) > point_estimate ({})",
                    result.lower_ci,
                    result.point_estimate
                );
                prop_assert!(
                    result.point_estimate <= result.upper_ci,
                    "point_estimate ({}) > upper_ci ({})",
                    result.point_estimate,
                    result.upper_ci
                );
            }
            Err(e) => {
                prop_assert!(
                    false,
                    "ROR must succeed for complete table (all cells >= 1): {:?}",
                    e
                );
            }
        }
    }
}

// ============================================================================
// Property 7: PRR and ROR agree on signal direction for extreme cases
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// When PRR is extremely high (a >> b and c is small relative to d),
    /// both PRR and ROR should be > 1, indicating association.
    #[test]
    fn prop_prr_ror_both_above_one_for_strong_signal_data(
        // a large (strong drug+event association), b small
        a in 100u64..1000u64,
        b in 1u64..10u64,
        // c small (rare event in non-drug group), d large
        c in 1u64..10u64,
        d in 1000u64..10_000u64,
    ) {
        let table = ContingencyTable::new(a, b, c, d);
        let criteria = SignalCriteria::evans();

        let prr_result = calculate_prr(&table, &criteria);
        let ror_result = calculate_ror(&table, &criteria);

        if let (Ok(prr), Ok(ror)) = (prr_result, ror_result) {
            // Both should show elevation > 1 for strong signal data
            prop_assert!(
                prr.point_estimate > 1.0,
                "PRR should be > 1 for strong signal data: {}",
                prr.point_estimate
            );
            prop_assert!(
                ror.point_estimate > 1.0,
                "ROR should be > 1 for strong signal data: {}",
                ror.point_estimate
            );
        }
    }
}
