//! # PII Scrubber Pipeline Stage
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | σ Sequence | Pipeline: input fields → scrub → output + audit |
//! | μ Mapping | Field name → ScrubAction mapping |
//! | ∂ Boundary | PII detection patterns |
//! | ∅ Void | Redacted fields become absent |
//!
//! ## Tier: T2-P (ScrubAction), T3 (PiiScrubber)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::audit::{RedactionAudit, RedactionEntry};
use crate::config::{CategoryPolicy, DataCategory, GhostConfig};
use crate::mode::GhostMode;
use crate::pseudonymize::{HmacPseudonymizer, Pseudonymizer};

/// Action to take on a PII field.
///
/// ## Tier: T2-P (μ Mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScrubAction {
    /// Replace with HMAC pseudonym (reversible via key holder).
    Pseudonymize,
    /// Replace with generalized value (e.g., age 34 → "30-39").
    Generalize,
    /// Replace with "[REDACTED]" marker.
    Redact,
    /// Keep the original value unchanged.
    Retain,
    /// Remove the field entirely.
    Suppress,
}

impl fmt::Display for ScrubAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pseudonymize => write!(f, "pseudonymize"),
            Self::Generalize => write!(f, "generalize"),
            Self::Redact => write!(f, "redact"),
            Self::Retain => write!(f, "retain"),
            Self::Suppress => write!(f, "suppress"),
        }
    }
}

/// Known PII field patterns for auto-detection.
///
/// ## Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PiiFieldPattern {
    /// Fields containing "name" (patient_name, reporter_name, etc.).
    Name,
    /// Fields containing "email".
    Email,
    /// Fields containing "phone" or "tel".
    Phone,
    /// Fields containing "address", "street", "city", "zip", "postal".
    Address,
    /// Fields containing "dob", "birth_date", "date_of_birth".
    DateOfBirth,
    /// Fields containing "ssn", "national_id", "passport".
    NationalId,
    /// Fields containing "ip_address", "ip".
    IpAddress,
    /// Fields containing "lat", "lon", "gps", "coordinates".
    GeoLocation,
}

impl PiiFieldPattern {
    /// Check if a field name matches this pattern.
    #[must_use]
    pub fn matches(&self, field_name: &str) -> bool {
        let lower = field_name.to_lowercase();
        match self {
            Self::Name => {
                // Match person-name fields but not drug_name, event_name, etc.
                lower.contains("patient_name")
                    || lower.contains("reporter_name")
                    || lower.contains("first_name")
                    || lower.contains("last_name")
                    || lower.contains("full_name")
                    || lower.contains("person_name")
                    || lower.contains("sender_name")
                    || lower.contains("contact_name")
                    || lower == "name"
            }
            Self::Email => lower.contains("email"),
            Self::Phone => lower.contains("phone") || lower.contains("tel"),
            Self::Address => {
                lower.contains("address")
                    || lower.contains("street")
                    || lower.contains("city")
                    || lower.contains("zip")
                    || lower.contains("postal")
            }
            Self::DateOfBirth => {
                lower.contains("dob")
                    || lower.contains("birth_date")
                    || lower.contains("date_of_birth")
            }
            Self::NationalId => {
                lower.contains("ssn") || lower.contains("national_id") || lower.contains("passport")
            }
            Self::IpAddress => lower.contains("ip_address") || lower == "ip",
            Self::GeoLocation => {
                lower.contains("latitude")
                    || lower.contains("longitude")
                    || lower.contains("gps")
                    || lower.contains("coordinates")
            }
        }
    }

    /// All patterns for iteration.
    pub const ALL: [PiiFieldPattern; 8] = [
        Self::Name,
        Self::Email,
        Self::Phone,
        Self::Address,
        Self::DateOfBirth,
        Self::NationalId,
        Self::IpAddress,
        Self::GeoLocation,
    ];

    /// Map to the most likely DataCategory.
    #[must_use]
    pub const fn likely_category(&self) -> DataCategory {
        match self {
            Self::Name | Self::DateOfBirth | Self::NationalId => DataCategory::BasicIdentity,
            Self::Email | Self::Phone => DataCategory::CommunicationData,
            Self::Address | Self::GeoLocation => DataCategory::LocationData,
            Self::IpAddress => DataCategory::DeviceData,
        }
    }
}

/// Result of scrubbing a set of fields.
///
/// ## Tier: T2-C (σ + μ + π)
#[derive(Debug, Clone)]
pub struct ScrubResult {
    /// Scrubbed fields (field_name → scrubbed_value).
    pub fields: HashMap<String, String>,
    /// Audit trail of all actions taken.
    pub audit: RedactionAudit,
    /// Count of fields that were modified.
    pub modified_count: usize,
    /// Count of fields that were suppressed (removed).
    pub suppressed_count: usize,
}

/// Pipeline stage that scrubs PII from field maps.
///
/// ## Tier: T3 (σ + μ + ∅ + ∂ + ς + N)
pub struct PiiScrubber {
    config: GhostConfig,
    pseudonymizer: Option<HmacPseudonymizer>,
}

impl PiiScrubber {
    /// Create a scrubber from config.
    ///
    /// If `hmac_key` is provided, pseudonymization is enabled.
    /// If `None`, pseudonymize actions fall back to redaction.
    #[must_use]
    pub fn new(config: GhostConfig, hmac_key: Option<&[u8]>) -> Self {
        let pseudonymizer = hmac_key.and_then(|k| HmacPseudonymizer::new(k).ok());
        Self {
            config,
            pseudonymizer,
        }
    }

    /// Determine the scrub action for a field based on config and PII detection.
    #[must_use]
    pub fn action_for_field(&self, field_name: &str) -> ScrubAction {
        if !self.config.mode.is_active() {
            return ScrubAction::Retain;
        }

        // Auto-detect PII pattern
        for pattern in &PiiFieldPattern::ALL {
            if pattern.matches(field_name) {
                let category = pattern.likely_category();
                let policy = self.config.policy_for(category);
                return Self::action_from_policy(&policy, &self.config.mode);
            }
        }

        // No PII pattern detected — retain
        ScrubAction::Retain
    }

    /// Derive action from a category policy and mode.
    fn action_from_policy(policy: &CategoryPolicy, mode: &GhostMode) -> ScrubAction {
        if policy.redact {
            ScrubAction::Redact
        } else if policy.pseudonymize {
            ScrubAction::Pseudonymize
        } else {
            match mode {
                GhostMode::Maximum => ScrubAction::Redact,
                _ => ScrubAction::Retain,
            }
        }
    }

    /// Scrub a map of fields. Returns scrubbed fields + audit trail.
    pub fn scrub(&self, fields: &HashMap<String, String>) -> ScrubResult {
        let mut output = HashMap::new();
        let mut audit = RedactionAudit::new();
        let mut modified = 0;
        let mut suppressed = 0;

        for (name, value) in fields {
            let action = self.action_for_field(name);

            match action {
                ScrubAction::Retain => {
                    output.insert(name.clone(), value.clone());
                }
                ScrubAction::Pseudonymize => {
                    let scrubbed = if let Some(ref p) = self.pseudonymizer {
                        match p.pseudonymize(name, value) {
                            Ok(handle) => handle.token,
                            Err(_) => "[PSEUDONYMIZATION_FAILED]".to_string(),
                        }
                    } else {
                        "[REDACTED]".to_string()
                    };
                    output.insert(name.clone(), scrubbed);
                    audit.append(RedactionEntry::new(
                        name,
                        "pseudonymize",
                        "PII pattern match",
                        self.category_for_field(name),
                    ));
                    modified += 1;
                }
                ScrubAction::Redact => {
                    output.insert(name.clone(), "[REDACTED]".to_string());
                    audit.append(RedactionEntry::new(
                        name,
                        "redact",
                        "PII pattern match",
                        self.category_for_field(name),
                    ));
                    modified += 1;
                }
                ScrubAction::Generalize => {
                    output.insert(name.clone(), "[GENERALIZED]".to_string());
                    audit.append(RedactionEntry::new(
                        name,
                        "generalize",
                        "PII pattern match",
                        self.category_for_field(name),
                    ));
                    modified += 1;
                }
                ScrubAction::Suppress => {
                    // Field not included in output
                    audit.append(RedactionEntry::new(
                        name,
                        "suppress",
                        "PII pattern match",
                        self.category_for_field(name),
                    ));
                    suppressed += 1;
                }
            }
        }

        ScrubResult {
            fields: output,
            audit,
            modified_count: modified,
            suppressed_count: suppressed,
        }
    }

    /// Get category label for a field.
    fn category_for_field(&self, field_name: &str) -> String {
        for pattern in &PiiFieldPattern::ALL {
            if pattern.matches(field_name) {
                return pattern.likely_category().to_string();
            }
        }
        "unknown".to_string()
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> GhostConfig {
        GhostConfig::default()
    }

    fn test_key() -> Vec<u8> {
        vec![42u8; 32]
    }

    fn sample_fields() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("patient_name".to_string(), "John Doe".to_string());
        m.insert("email".to_string(), "john@example.com".to_string());
        m.insert("drug_name".to_string(), "Aspirin".to_string());
        m.insert("event_term".to_string(), "Headache".to_string());
        m
    }

    #[test]
    fn retain_when_off() {
        let mut cfg = test_config();
        cfg.mode = GhostMode::Off;
        let scrubber = PiiScrubber::new(cfg, Some(&test_key()));
        assert_eq!(
            scrubber.action_for_field("patient_name"),
            ScrubAction::Retain
        );
    }

    #[test]
    fn pseudonymize_name_in_standard() {
        let scrubber = PiiScrubber::new(test_config(), Some(&test_key()));
        assert_eq!(
            scrubber.action_for_field("patient_name"),
            ScrubAction::Pseudonymize
        );
    }

    #[test]
    fn non_pii_field_retained() {
        let scrubber = PiiScrubber::new(test_config(), Some(&test_key()));
        assert_eq!(scrubber.action_for_field("drug_name"), ScrubAction::Retain);
    }

    #[test]
    fn scrub_modifies_pii_fields() {
        let scrubber = PiiScrubber::new(test_config(), Some(&test_key()));
        let result = scrubber.scrub(&sample_fields());
        // PII fields should be modified
        assert_ne!(
            result.fields.get("patient_name").map(|s| s.as_str()),
            Some("John Doe")
        );
        // Non-PII fields should be unchanged
        assert_eq!(
            result.fields.get("drug_name").map(|s| s.as_str()),
            Some("Aspirin")
        );
    }

    #[test]
    fn scrub_produces_audit_trail() {
        let scrubber = PiiScrubber::new(test_config(), Some(&test_key()));
        let result = scrubber.scrub(&sample_fields());
        assert!(!result.audit.is_empty());
        assert!(result.modified_count > 0);
    }

    #[test]
    fn scrub_without_key_falls_back_to_redact() {
        let scrubber = PiiScrubber::new(test_config(), None);
        let result = scrubber.scrub(&sample_fields());
        assert_eq!(
            result.fields.get("patient_name").map(|s| s.as_str()),
            Some("[REDACTED]")
        );
    }

    #[test]
    fn email_field_detected() {
        assert!(PiiFieldPattern::Email.matches("email"));
        assert!(PiiFieldPattern::Email.matches("reporter_email"));
        assert!(PiiFieldPattern::Email.matches("EMAIL_ADDRESS"));
    }

    #[test]
    fn phone_field_detected() {
        assert!(PiiFieldPattern::Phone.matches("phone_number"));
        assert!(PiiFieldPattern::Phone.matches("telephone"));
    }

    #[test]
    fn address_field_detected() {
        assert!(PiiFieldPattern::Address.matches("street_address"));
        assert!(PiiFieldPattern::Address.matches("city"));
        assert!(PiiFieldPattern::Address.matches("zip_code"));
    }

    #[test]
    fn dob_field_detected() {
        assert!(PiiFieldPattern::DateOfBirth.matches("date_of_birth"));
        assert!(PiiFieldPattern::DateOfBirth.matches("dob"));
    }

    #[test]
    fn national_id_detected() {
        assert!(PiiFieldPattern::NationalId.matches("ssn"));
        assert!(PiiFieldPattern::NationalId.matches("passport_number"));
    }

    #[test]
    fn geo_detected() {
        assert!(PiiFieldPattern::GeoLocation.matches("latitude"));
        assert!(PiiFieldPattern::GeoLocation.matches("gps_coordinates"));
    }

    #[test]
    fn likely_category_mapping() {
        assert_eq!(
            PiiFieldPattern::Name.likely_category(),
            DataCategory::BasicIdentity
        );
        assert_eq!(
            PiiFieldPattern::Email.likely_category(),
            DataCategory::CommunicationData
        );
        assert_eq!(
            PiiFieldPattern::GeoLocation.likely_category(),
            DataCategory::LocationData
        );
    }

    #[test]
    fn scrub_action_display() {
        assert_eq!(format!("{}", ScrubAction::Pseudonymize), "pseudonymize");
        assert_eq!(format!("{}", ScrubAction::Redact), "redact");
        assert_eq!(format!("{}", ScrubAction::Suppress), "suppress");
    }

    #[test]
    fn scrub_empty_fields() {
        let scrubber = PiiScrubber::new(test_config(), Some(&test_key()));
        let result = scrubber.scrub(&HashMap::new());
        assert_eq!(result.modified_count, 0);
        assert_eq!(result.suppressed_count, 0);
        assert!(result.audit.is_empty());
    }

    #[test]
    fn event_term_not_scrubbed() {
        let scrubber = PiiScrubber::new(test_config(), Some(&test_key()));
        let result = scrubber.scrub(&sample_fields());
        assert_eq!(
            result.fields.get("event_term").map(|s| s.as_str()),
            Some("Headache")
        );
    }

    #[test]
    fn strict_mode_redacts_when_policy_says_redact() {
        let mut cfg = test_config();
        cfg.mode = GhostMode::Strict;
        // Manually set BasicIdentity policy to redact
        cfg.policies.insert(
            DataCategory::BasicIdentity,
            CategoryPolicy {
                category: DataCategory::BasicIdentity,
                pseudonymize: false,
                redact: true,
                retention_days: 30,
                reversal_permitted: false,
            },
        );
        let scrubber = PiiScrubber::new(cfg, Some(&test_key()));
        let action = scrubber.action_for_field("patient_name");
        assert_eq!(action, ScrubAction::Redact);
    }

    #[test]
    fn maximum_mode_no_pseudonymizer_needed() {
        let mut cfg = test_config();
        cfg.mode = GhostMode::Maximum;
        for cat in DataCategory::ALL {
            cfg.policies.insert(cat, CategoryPolicy::strict_for(cat));
        }
        let scrubber = PiiScrubber::new(cfg, None);
        let result = scrubber.scrub(&sample_fields());
        // All PII fields should be redacted
        for (key, val) in &result.fields {
            if key == "drug_name" || key == "event_term" {
                assert_ne!(val, "[REDACTED]");
            }
        }
    }

    #[test]
    fn ip_address_detected() {
        assert!(PiiFieldPattern::IpAddress.matches("ip_address"));
        assert!(PiiFieldPattern::IpAddress.matches("ip"));
        assert!(!PiiFieldPattern::IpAddress.matches("tip"));
    }
}
