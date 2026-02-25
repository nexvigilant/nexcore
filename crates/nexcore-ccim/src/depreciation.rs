//! Depreciation computation for CCIM.
//!
//! Computes W(d) — the cumulative capability withdrawal from depreciation.
//! Grounding: Σ(Sum) + N(Quantity) + ∝(Irreversibility).

use crate::types::DepreciationEntry;

/// Compute the weighted average depreciation rate (delta_avg) across all entries.
///
/// Each entry contributes its category rate weighted by its capability at risk.
/// If entries are empty, returns 0.0 (no depreciation).
#[must_use]
pub fn compute_delta_avg(entries: &[DepreciationEntry]) -> f64 {
    if entries.is_empty() {
        return 0.0;
    }

    let total_cu: f64 = entries.iter().map(|e| e.capability_at_risk).sum();
    if total_cu == 0.0 {
        return 0.0;
    }

    let weighted_sum: f64 = entries
        .iter()
        .map(|e| e.category.rate() * e.capability_at_risk)
        .sum();

    weighted_sum / total_cu
}

/// Compute total withdrawal W(d) = sum(capability_at_risk * rate * periods).
///
/// This is the cumulative capability lost to depreciation across all entries.
#[must_use]
pub fn compute_withdrawal(entries: &[DepreciationEntry]) -> f64 {
    entries
        .iter()
        .map(|e| e.capability_at_risk * e.category.rate() * f64::from(e.periods_unmaintained))
        .sum()
}

/// Identify high-priority depreciation alerts: entries where cumulative loss > 50%.
#[must_use]
pub fn depreciation_alerts(entries: &[DepreciationEntry]) -> Vec<&DepreciationEntry> {
    entries
        .iter()
        .filter(|e| e.category.rate() * f64::from(e.periods_unmaintained) > 0.50)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DepreciationCategory;

    #[test]
    fn test_compute_delta_avg_single_category() {
        let entries = vec![DepreciationEntry {
            description: "old code".to_string(),
            category: DepreciationCategory::UnmaintainedCode,
            capability_at_risk: 100.0,
            periods_unmaintained: 3,
        }];
        let avg = compute_delta_avg(&entries);
        assert!((avg - 0.02).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_delta_avg_mixed_categories() {
        let entries = vec![
            DepreciationEntry {
                description: "old code".to_string(),
                category: DepreciationCategory::UnmaintainedCode,
                capability_at_risk: 100.0,
                periods_unmaintained: 1,
            },
            DepreciationEntry {
                description: "failing tests".to_string(),
                category: DepreciationCategory::FailingTests,
                capability_at_risk: 100.0,
                periods_unmaintained: 1,
            },
        ];
        let avg = compute_delta_avg(&entries);
        // (0.02 * 100 + 0.10 * 100) / 200 = 0.06
        assert!((avg - 0.06).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_withdrawal_accumulates() {
        let entries = vec![
            DepreciationEntry {
                description: "old code".to_string(),
                category: DepreciationCategory::UnmaintainedCode,
                capability_at_risk: 100.0,
                periods_unmaintained: 5,
            },
            DepreciationEntry {
                description: "stale docs".to_string(),
                category: DepreciationCategory::StaleDocs,
                capability_at_risk: 50.0,
                periods_unmaintained: 3,
            },
        ];
        let w = compute_withdrawal(&entries);
        // 100 * 0.02 * 5 + 50 * 0.01 * 3 = 10.0 + 1.5 = 11.5
        assert!((w - 11.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_withdrawal_empty() {
        let entries: Vec<DepreciationEntry> = vec![];
        assert!((compute_withdrawal(&entries) - 0.0).abs() < f64::EPSILON);
    }
}
