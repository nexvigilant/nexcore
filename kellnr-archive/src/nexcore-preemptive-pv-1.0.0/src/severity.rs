//! Irreversibility-Weighted Severity (Omega).
//!
//! Tier: T2-P (maps to Irreversibility `proportional` + Quantity `N`)
//!
//! Computes the severity-weighted score that accounts for irreversibility:
//!
//! ```text
//! Omega(e) = S(e) * (1 + irreversibility_factor(e))
//! ```
//!
//! Pre-computed reference values (ICH E2A):
//! - Fatal:           S=5, irreversibility=1.0, Omega=10.0
//! - Life-threatening: S=4, irreversibility=0.8, Omega=7.2
//! - Disability:       S=3, irreversibility=0.7, Omega=5.1
//! - Hospitalization:  S=2, irreversibility=0.3, Omega=2.6
//! - Non-serious:      S=1, irreversibility=0.0, Omega=1.0

use crate::types::Seriousness;

/// Computes the irreversibility-weighted severity Omega for a seriousness category.
///
/// ```text
/// Omega = S * (1 + irreversibility_factor)
/// ```
///
/// This amplifies severity for outcomes that cannot be reversed,
/// making fatal and life-threatening events proportionally heavier.
#[must_use]
pub fn omega(seriousness: Seriousness) -> f64 {
    let s = seriousness.severity_score();
    let alpha = seriousness.irreversibility_factor();
    s * (1.0 + alpha)
}

/// Returns all seriousness categories with their omega values, ordered by omega descending.
#[must_use]
pub fn omega_table() -> Vec<(Seriousness, f64)> {
    let categories = [
        Seriousness::Fatal,
        Seriousness::LifeThreatening,
        Seriousness::Disability,
        Seriousness::Hospitalization,
        Seriousness::NonSerious,
    ];
    categories.iter().map(|&s| (s, omega(s))).collect()
}

/// Normalizes omega to [0, 1] range relative to the maximum possible omega (Fatal = 10.0).
#[must_use]
pub fn omega_normalized(seriousness: Seriousness) -> f64 {
    let max_omega = omega(Seriousness::Fatal);
    if max_omega == 0.0 {
        return 0.0;
    }
    omega(seriousness) / max_omega
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omega_fatal() {
        let result = omega(Seriousness::Fatal);
        // S=5, alpha=1.0, Omega = 5 * (1 + 1) = 10.0
        assert!((result - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn omega_life_threatening() {
        let result = omega(Seriousness::LifeThreatening);
        // S=4, alpha=0.8, Omega = 4 * (1 + 0.8) = 7.2
        assert!((result - 7.2).abs() < f64::EPSILON);
    }

    #[test]
    fn omega_disability() {
        let result = omega(Seriousness::Disability);
        // S=3, alpha=0.7, Omega = 3 * (1 + 0.7) = 5.1
        assert!((result - 5.1).abs() < f64::EPSILON);
    }

    #[test]
    fn omega_hospitalization() {
        let result = omega(Seriousness::Hospitalization);
        // S=2, alpha=0.3, Omega = 2 * (1 + 0.3) = 2.6
        assert!((result - 2.6).abs() < f64::EPSILON);
    }

    #[test]
    fn omega_non_serious() {
        let result = omega(Seriousness::NonSerious);
        // S=1, alpha=0.0, Omega = 1 * (1 + 0) = 1.0
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn omega_table_ordering() {
        let table = omega_table();
        assert_eq!(table.len(), 5);
        // Verify descending order
        for window in table.windows(2) {
            assert!(window[0].1 >= window[1].1);
        }
    }

    #[test]
    fn omega_normalized_fatal_is_one() {
        let result = omega_normalized(Seriousness::Fatal);
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn omega_normalized_non_serious() {
        let result = omega_normalized(Seriousness::NonSerious);
        // 1.0 / 10.0 = 0.1
        assert!((result - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn omega_monotonically_increases_with_severity() {
        let ns = omega(Seriousness::NonSerious);
        let hosp = omega(Seriousness::Hospitalization);
        let dis = omega(Seriousness::Disability);
        let lt = omega(Seriousness::LifeThreatening);
        let fatal = omega(Seriousness::Fatal);

        assert!(ns < hosp);
        assert!(hosp < dis);
        assert!(dis < lt);
        assert!(lt < fatal);
    }
}
