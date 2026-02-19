//! # Ghost Configuration
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | μ Mapping | DataCategory maps PII types to policies |
//! | ς State | GhostConfig is the runtime privacy state |
//!
//! ## Tier: T3 (GhostConfig), T2-P (DataCategory), T2-C (CategoryPolicy)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::mode::GhostMode;

/// Categories of personally identifiable information.
///
/// Mirrors `guardian_domain::privacy::DataCategory` without creating
/// a reverse dependency. Bridge via `From` at the consumer layer.
///
/// ## Tier: T2-P (μ Mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataCategory {
    /// Name, address, email, phone.
    BasicIdentity,
    /// Medical records, health information, diagnoses.
    HealthData,
    /// Payment information, insurance details.
    FinancialData,
    /// Fingerprints, facial recognition data.
    BiometricData,
    /// Website usage, preferences, behavioral patterns.
    BehavioralData,
    /// GPS coordinates, IP addresses, location history.
    LocationData,
    /// Device IDs, browser fingerprints.
    DeviceData,
    /// Emails, messages, call logs.
    CommunicationData,
}

impl DataCategory {
    /// All category variants for iteration.
    pub const ALL: [DataCategory; 8] = [
        Self::BasicIdentity,
        Self::HealthData,
        Self::FinancialData,
        Self::BiometricData,
        Self::BehavioralData,
        Self::LocationData,
        Self::DeviceData,
        Self::CommunicationData,
    ];

    /// Whether this category is considered sensitive (health, biometric, financial).
    #[must_use]
    pub const fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Self::HealthData | Self::BiometricData | Self::FinancialData
        )
    }

    /// Short label for audit logs.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BasicIdentity => "basic_identity",
            Self::HealthData => "health_data",
            Self::FinancialData => "financial_data",
            Self::BiometricData => "biometric_data",
            Self::BehavioralData => "behavioral_data",
            Self::LocationData => "location_data",
            Self::DeviceData => "device_data",
            Self::CommunicationData => "communication_data",
        }
    }
}

impl fmt::Display for DataCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Per-category scrubbing policy.
///
/// ## Tier: T2-C (μ Mapping + ς State)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryPolicy {
    /// The data category this policy applies to.
    pub category: DataCategory,
    /// Whether pseudonymization is required.
    pub pseudonymize: bool,
    /// Whether the field should be fully redacted instead of pseudonymized.
    pub redact: bool,
    /// Retention period in days (0 = no retention, delete immediately).
    pub retention_days: u32,
    /// Whether reversal is permitted for this category.
    pub reversal_permitted: bool,
}

impl CategoryPolicy {
    /// Default policy for a category based on sensitivity.
    #[must_use]
    pub fn default_for(category: DataCategory) -> Self {
        if category.is_sensitive() {
            Self {
                category,
                pseudonymize: true,
                redact: false,
                retention_days: 90,
                reversal_permitted: true,
            }
        } else {
            Self {
                category,
                pseudonymize: true,
                redact: false,
                retention_days: 365,
                reversal_permitted: true,
            }
        }
    }

    /// Strict policy: shorter retention, pseudonymization enforced.
    #[must_use]
    pub fn strict_for(category: DataCategory) -> Self {
        Self {
            category,
            pseudonymize: true,
            redact: category.is_sensitive(),
            retention_days: 30,
            reversal_permitted: false,
        }
    }
}

/// Runtime privacy configuration.
///
/// ## Tier: T3 (ς State + μ Mapping + N Quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GhostConfig {
    /// Active enforcement level.
    pub mode: GhostMode,
    /// Per-category policies.
    pub policies: HashMap<DataCategory, CategoryPolicy>,
    /// HMAC key identifier (NOT the key itself — keys live in Vault).
    pub hmac_key_id: String,
    /// k-anonymity target (overrides mode default if set).
    pub k_anonymity_target: Option<u32>,
    /// l-diversity target for sensitive attributes.
    pub l_diversity_target: Option<u32>,
}

impl GhostConfig {
    /// Effective k-anonymity threshold (explicit override or mode default).
    #[must_use]
    pub fn effective_k(&self) -> u32 {
        self.k_anonymity_target
            .unwrap_or_else(|| self.mode.min_k_anonymity())
    }

    /// Look up policy for a category. Falls back to default_for if not configured.
    #[must_use]
    pub fn policy_for(&self, category: DataCategory) -> CategoryPolicy {
        self.policies
            .get(&category)
            .cloned()
            .unwrap_or_else(|| CategoryPolicy::default_for(category))
    }
}

impl Default for GhostConfig {
    fn default() -> Self {
        let mut policies = HashMap::new();
        for cat in DataCategory::ALL {
            policies.insert(cat, CategoryPolicy::default_for(cat));
        }
        Self {
            mode: GhostMode::default(),
            policies,
            hmac_key_id: "default".to_string(),
            k_anonymity_target: None,
            l_diversity_target: None,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_standard_mode() {
        let cfg = GhostConfig::default();
        assert_eq!(cfg.mode, GhostMode::Standard);
    }

    #[test]
    fn default_has_all_categories() {
        let cfg = GhostConfig::default();
        assert_eq!(cfg.policies.len(), 8);
    }

    #[test]
    fn health_data_is_sensitive() {
        assert!(DataCategory::HealthData.is_sensitive());
        assert!(DataCategory::BiometricData.is_sensitive());
        assert!(DataCategory::FinancialData.is_sensitive());
    }

    #[test]
    fn behavioral_not_sensitive() {
        assert!(!DataCategory::BehavioralData.is_sensitive());
        assert!(!DataCategory::DeviceData.is_sensitive());
    }

    #[test]
    fn sensitive_default_has_shorter_retention() {
        let health = CategoryPolicy::default_for(DataCategory::HealthData);
        let device = CategoryPolicy::default_for(DataCategory::DeviceData);
        assert!(health.retention_days < device.retention_days);
    }

    #[test]
    fn strict_policy_denies_reversal() {
        let policy = CategoryPolicy::strict_for(DataCategory::HealthData);
        assert!(!policy.reversal_permitted);
        assert!(policy.redact); // sensitive + strict = redact
    }

    #[test]
    fn effective_k_uses_override() {
        let mut cfg = GhostConfig::default();
        cfg.k_anonymity_target = Some(20);
        assert_eq!(cfg.effective_k(), 20);
    }

    #[test]
    fn effective_k_falls_back_to_mode() {
        let cfg = GhostConfig::default();
        assert_eq!(cfg.effective_k(), GhostMode::Standard.min_k_anonymity());
    }

    #[test]
    fn policy_lookup_falls_back() {
        let cfg = GhostConfig {
            mode: GhostMode::Standard,
            policies: HashMap::new(), // empty
            hmac_key_id: "test".into(),
            k_anonymity_target: None,
            l_diversity_target: None,
        };
        let policy = cfg.policy_for(DataCategory::HealthData);
        assert!(policy.pseudonymize);
    }

    #[test]
    fn serde_roundtrip() {
        let cfg = GhostConfig::default();
        let json = serde_json::to_string(&cfg).unwrap_or_default();
        let back: Result<GhostConfig, _> = serde_json::from_str(&json);
        assert!(back.is_ok());
    }
}
