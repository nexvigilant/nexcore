//! # Domain Calibration Profiles
//!
//! Baseline feature expectations per PV domain (D02-D12).
//! Domain-specific calibration accounts for differences in writing style:
//! regulatory writing (D04) is more structured than case narratives (D08).
//!
//! ## Primitive Grounding
//! - μ Mapping: domain ID → profile parameters
//! - N Quantity: baseline numeric expectations
//! - ∂ Boundary: domain-specific threshold adjustments

use crate::error::IntegrityError;
use serde::Serialize;

/// Tier: T2-C (domain composite)
///
/// Domain-specific calibration profile for feature baselines.
#[derive(Debug, Clone, Serialize)]
pub struct CalibrationProfile {
    /// Domain identifier (e.g., "D02", "D08")
    pub domain_id: &'static str,
    /// Domain name
    pub domain_name: &'static str,
    /// Expected Zipf alpha for domain (human baseline)
    pub zipf_alpha_baseline: f64,
    /// Expected entropy std for domain (human baseline)
    pub entropy_std_baseline: f64,
    /// Expected burstiness for domain (human baseline)
    pub burstiness_baseline: f64,
    /// Expected perplexity variance for domain (human baseline)
    pub perplexity_var_baseline: f64,
    /// Expected TTR for domain (human baseline)
    pub ttr_baseline: f64,
}

/// Static domain profiles.
static PROFILES: &[CalibrationProfile] = &[
    CalibrationProfile {
        domain_id: "D02",
        domain_name: "PV Legislation & Guidelines",
        zipf_alpha_baseline: 0.95,
        entropy_std_baseline: 0.7,
        burstiness_baseline: 0.20,
        perplexity_var_baseline: 0.35,
        ttr_baseline: 0.60,
    },
    CalibrationProfile {
        domain_id: "D03",
        domain_name: "PV Systems & Quality",
        zipf_alpha_baseline: 0.90,
        entropy_std_baseline: 0.8,
        burstiness_baseline: 0.25,
        perplexity_var_baseline: 0.40,
        ttr_baseline: 0.65,
    },
    CalibrationProfile {
        domain_id: "D04",
        domain_name: "ICSR Collection & Processing",
        zipf_alpha_baseline: 0.88,
        entropy_std_baseline: 0.75,
        burstiness_baseline: 0.22,
        perplexity_var_baseline: 0.38,
        ttr_baseline: 0.62,
    },
    CalibrationProfile {
        domain_id: "D08",
        domain_name: "Signal Detection & Evaluation",
        zipf_alpha_baseline: 1.05,
        entropy_std_baseline: 1.0,
        burstiness_baseline: 0.30,
        perplexity_var_baseline: 0.50,
        ttr_baseline: 0.70,
    },
    CalibrationProfile {
        domain_id: "D10",
        domain_name: "Benefit-Risk Assessment",
        zipf_alpha_baseline: 1.00,
        entropy_std_baseline: 0.9,
        burstiness_baseline: 0.28,
        perplexity_var_baseline: 0.45,
        ttr_baseline: 0.68,
    },
    CalibrationProfile {
        domain_id: "D12",
        domain_name: "PV Auditing & Inspection",
        zipf_alpha_baseline: 0.92,
        entropy_std_baseline: 0.85,
        burstiness_baseline: 0.24,
        perplexity_var_baseline: 0.42,
        ttr_baseline: 0.63,
    },
];

/// Look up a calibration profile by domain ID.
///
/// # Errors
/// Returns `InvalidDomainId` if domain is not in the profile set.
pub fn get_profile(domain_id: &str) -> Result<&'static CalibrationProfile, IntegrityError> {
    PROFILES
        .iter()
        .find(|p| p.domain_id == domain_id)
        .ok_or_else(|| IntegrityError::InvalidDomainId {
            id: domain_id.to_string(),
        })
}

/// List all available domain profiles.
#[must_use]
pub fn list_profiles() -> &'static [CalibrationProfile] {
    PROFILES
}

/// Default profile (D08 Signal Detection — the most "general" PV domain).
#[must_use]
pub fn default_profile() -> &'static CalibrationProfile {
    // D08 is index 3 — safe because PROFILES is static and non-empty
    &PROFILES[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_valid_profiles() {
        for id in &["D02", "D03", "D04", "D08", "D10", "D12"] {
            let result = get_profile(id);
            assert!(result.is_ok(), "Profile {id} should exist");
        }
    }

    #[test]
    fn test_get_invalid_profile() {
        assert!(get_profile("D01").is_err());
        assert!(get_profile("D99").is_err());
        assert!(get_profile("").is_err());
    }

    #[test]
    fn test_default_is_d08() {
        let profile = default_profile();
        assert_eq!(profile.domain_id, "D08");
    }

    #[test]
    fn test_list_profiles_count() {
        let profiles = list_profiles();
        assert_eq!(profiles.len(), 6);
    }

    #[test]
    fn test_profile_baselines_reasonable() {
        for profile in list_profiles() {
            assert!(
                profile.zipf_alpha_baseline > 0.5 && profile.zipf_alpha_baseline < 1.5,
                "{}: zipf_alpha={}",
                profile.domain_id,
                profile.zipf_alpha_baseline
            );
            assert!(
                profile.ttr_baseline > 0.3 && profile.ttr_baseline < 0.9,
                "{}: ttr={}",
                profile.domain_id,
                profile.ttr_baseline
            );
        }
    }
}
