//! Export control screening for compound data.
//!
//! Screens compound exports against sanctioned country lists and
//! dual-use chemical indicators. This module implements a simplified
//! export control regime inspired by:
//! - US Export Administration Regulations (EAR)
//! - EU Dual-Use Regulation (EC 428/2009, updated 2021/821)
//! - Chemical Weapons Convention (CWC) schedules
//!
//! For production use, this should be supplemented with a full
//! restricted party screening service and up-to-date control lists.

use serde::{Deserialize, Serialize};

// ============================================================================
// Risk Classification
// ============================================================================

/// Export risk level for a compound or dataset.
///
/// Determines the level of review and licensing required before
/// data can be exported to a given destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ExportRisk {
    /// No export restrictions identified.
    None,
    /// Minor flags — export permitted with standard logging.
    Low,
    /// Moderate flags — requires compliance officer review.
    Medium,
    /// Significant flags — requires Export License Application (ELA).
    High,
    /// Export is prohibited (sanctioned destination or CWC Schedule 1).
    Prohibited,
}

// ============================================================================
// Screening Result
// ============================================================================

/// Result of screening a compound for export to a specific destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportScreeningResult {
    /// Identifier of the compound being screened.
    pub compound_id: String,
    /// Assessed risk level.
    pub risk_level: ExportRisk,
    /// Specific flags that were triggered during screening.
    pub flags: Vec<String>,
    /// ISO 3166-1 alpha-2 country code of the destination.
    pub destination_country: String,
    /// Whether an Export License Application is required.
    pub requires_license: bool,
}

// ============================================================================
// Sanctioned Countries
// ============================================================================

/// Comprehensive sanctions list (US OFAC + EU restrictive measures).
///
/// These country codes are subject to broad trade embargoes.
/// Data exports to these destinations are prohibited without
/// specific government authorization.
const SANCTIONED_COUNTRIES: &[&str] = &["CU", "IR", "KP", "SY", "RU"];

/// Check whether a country code is on the sanctions list.
///
/// Uses ISO 3166-1 alpha-2 country codes. Case-insensitive comparison.
#[must_use]
pub fn is_sanctioned_country(country_code: &str) -> bool {
    let upper = country_code.to_uppercase();
    SANCTIONED_COUNTRIES.iter().any(|&c| c == upper)
}

// ============================================================================
// Dual-Use Chemical Indicators
// ============================================================================

/// Keywords in compound data that indicate potential dual-use concern.
///
/// These are simplified indicators — production systems should use
/// full CAS number matching against CWC schedules and Commerce
/// Control List (CCL) ECCN classifications.
const DUAL_USE_INDICATORS: &[&str] = &[
    "nerve agent",
    "precursor",
    "schedule 1",
    "schedule 2",
    "cwc",
    "chemical weapon",
    "blister agent",
    "choking agent",
    "blood agent",
    "riot control",
    "incapacitating",
    "toxin",
    "biological agent",
];

/// Keywords indicating restricted compound classes that need review.
const RESTRICTED_CLASS_INDICATORS: &[&str] = &[
    "controlled substance",
    "schedule ii",
    "schedule iii",
    "schedule iv",
    "narcotic",
    "psychotropic",
    "anabolic steroid",
    "fentanyl",
    "opioid",
];

/// Check compound data for dual-use chemical indicators.
///
/// Scans the JSON value's string representation for known keywords.
/// Returns a list of matched indicator strings.
fn check_dual_use_indicators(compound_data: &serde_json::Value) -> Vec<String> {
    let data_str = compound_data.to_string().to_lowercase();
    let mut flags = Vec::new();

    for &indicator in DUAL_USE_INDICATORS {
        if data_str.contains(indicator) {
            flags.push(format!("dual_use_indicator: {indicator}"));
        }
    }

    flags
}

/// Check compound data for restricted substance class indicators.
fn check_restricted_class(compound_data: &serde_json::Value) -> Vec<String> {
    let data_str = compound_data.to_string().to_lowercase();
    let mut flags = Vec::new();

    for &indicator in RESTRICTED_CLASS_INDICATORS {
        if data_str.contains(indicator) {
            flags.push(format!("restricted_class: {indicator}"));
        }
    }

    flags
}

// ============================================================================
// Screening Function
// ============================================================================

/// Screen a compound for export to a specific destination country.
///
/// Evaluates the compound data against:
/// 1. Sanctioned country list (destination check)
/// 2. Dual-use chemical indicators (CWC schedule keywords)
/// 3. Restricted compound class indicators (controlled substances)
///
/// Risk assessment:
/// - **Prohibited**: Destination is sanctioned, or CWC Schedule 1 indicators found
/// - **High**: Multiple dual-use or restricted-class indicators
/// - **Medium**: Single dual-use or restricted-class indicator
/// - **Low**: Minor flags (e.g., general "precursor" mention)
/// - **None**: No flags triggered
#[must_use]
pub fn screen_compound_export(
    compound_data: &serde_json::Value,
    destination_country: &str,
) -> ExportScreeningResult {
    let compound_id = compound_data
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut flags = Vec::new();

    // Check 1: Sanctioned destination
    if is_sanctioned_country(destination_country) {
        flags.push(format!("sanctioned_destination: {destination_country}"));
    }

    // Check 2: Dual-use indicators
    let dual_use_flags = check_dual_use_indicators(compound_data);
    flags.extend(dual_use_flags);

    // Check 3: Restricted class indicators
    let restricted_flags = check_restricted_class(compound_data);
    flags.extend(restricted_flags);

    // Determine risk level
    let has_sanctioned_dest = is_sanctioned_country(destination_country);
    let has_schedule_1 = flags.iter().any(|f| f.contains("schedule 1"));
    let dual_use_count = flags
        .iter()
        .filter(|f| f.starts_with("dual_use_indicator"))
        .count();
    let restricted_count = flags
        .iter()
        .filter(|f| f.starts_with("restricted_class"))
        .count();
    let total_substance_flags = dual_use_count + restricted_count;

    let risk_level = if has_sanctioned_dest || has_schedule_1 {
        ExportRisk::Prohibited
    } else if total_substance_flags >= 3 {
        ExportRisk::High
    } else if total_substance_flags >= 1 {
        ExportRisk::Medium
    } else if !flags.is_empty() {
        ExportRisk::Low
    } else {
        ExportRisk::None
    };

    let requires_license = requires_ela(&risk_level);

    ExportScreeningResult {
        compound_id,
        risk_level,
        flags,
        destination_country: destination_country.to_string(),
        requires_license,
    }
}

/// Determine whether an Export License Application (ELA) is required.
///
/// An ELA is required for High risk and above. Prohibited exports
/// also require an ELA (though it is unlikely to be granted for
/// sanctioned destinations).
#[must_use]
pub fn requires_ela(risk_level: &ExportRisk) -> bool {
    matches!(risk_level, ExportRisk::High | ExportRisk::Prohibited)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sanctioned_countries_detected() {
        assert!(is_sanctioned_country("CU"));
        assert!(is_sanctioned_country("IR"));
        assert!(is_sanctioned_country("KP"));
        assert!(is_sanctioned_country("SY"));
        assert!(is_sanctioned_country("RU"));
    }

    #[test]
    fn non_sanctioned_countries_pass() {
        assert!(!is_sanctioned_country("US"));
        assert!(!is_sanctioned_country("GB"));
        assert!(!is_sanctioned_country("DE"));
        assert!(!is_sanctioned_country("JP"));
        assert!(!is_sanctioned_country("CA"));
    }

    #[test]
    fn sanctioned_check_is_case_insensitive() {
        assert!(is_sanctioned_country("cu"));
        assert!(is_sanctioned_country("Cu"));
        assert!(is_sanctioned_country("ir"));
    }

    #[test]
    fn clean_compound_to_safe_country_is_no_risk() {
        let compound = json!({
            "id": "cpd_001",
            "name": "Aspirin",
            "formula": "C9H8O4",
            "description": "Common analgesic"
        });

        let result = screen_compound_export(&compound, "US");
        assert_eq!(result.risk_level, ExportRisk::None);
        assert!(result.flags.is_empty());
        assert!(!result.requires_license);
    }

    #[test]
    fn clean_compound_to_sanctioned_country_is_prohibited() {
        let compound = json!({
            "id": "cpd_002",
            "name": "Ibuprofen",
            "formula": "C13H18O2"
        });

        let result = screen_compound_export(&compound, "KP");
        assert_eq!(result.risk_level, ExportRisk::Prohibited);
        assert!(
            result
                .flags
                .iter()
                .any(|f| f.contains("sanctioned_destination"))
        );
        assert!(result.requires_license);
    }

    #[test]
    fn dual_use_compound_flagged() {
        let compound = json!({
            "id": "cpd_003",
            "name": "Test Compound",
            "notes": "This is a known precursor compound"
        });

        let result = screen_compound_export(&compound, "GB");
        assert!(result.risk_level >= ExportRisk::Medium);
        assert!(
            result
                .flags
                .iter()
                .any(|f| f.contains("dual_use_indicator"))
        );
    }

    #[test]
    fn schedule_1_compound_is_prohibited() {
        let compound = json!({
            "id": "cpd_004",
            "name": "Restricted Agent",
            "classification": "Schedule 1 chemical weapon precursor"
        });

        let result = screen_compound_export(&compound, "DE");
        assert_eq!(result.risk_level, ExportRisk::Prohibited);
        assert!(result.requires_license);
    }

    #[test]
    fn multiple_flags_elevate_risk() {
        let compound = json!({
            "id": "cpd_005",
            "name": "Complex Compound",
            "notes": "Known precursor, potential toxin, related to blister agent research"
        });

        let result = screen_compound_export(&compound, "JP");
        // 3 dual-use flags: precursor, toxin, blister agent
        assert!(result.risk_level >= ExportRisk::High);
        assert!(result.flags.len() >= 3);
    }

    #[test]
    fn restricted_class_flagged() {
        let compound = json!({
            "id": "cpd_006",
            "name": "Controlled Drug",
            "class": "controlled substance, schedule ii narcotic"
        });

        let result = screen_compound_export(&compound, "CA");
        assert!(result.risk_level >= ExportRisk::Medium);
        assert!(result.flags.iter().any(|f| f.contains("restricted_class")));
    }

    #[test]
    fn requires_ela_for_high_and_prohibited() {
        assert!(!requires_ela(&ExportRisk::None));
        assert!(!requires_ela(&ExportRisk::Low));
        assert!(!requires_ela(&ExportRisk::Medium));
        assert!(requires_ela(&ExportRisk::High));
        assert!(requires_ela(&ExportRisk::Prohibited));
    }

    #[test]
    fn compound_id_extracted_from_data() {
        let compound = json!({
            "id": "cpd_custom_123",
            "name": "Test"
        });

        let result = screen_compound_export(&compound, "US");
        assert_eq!(result.compound_id, "cpd_custom_123");
    }

    #[test]
    fn missing_compound_id_defaults_to_unknown() {
        let compound = json!({
            "name": "No ID Compound"
        });

        let result = screen_compound_export(&compound, "US");
        assert_eq!(result.compound_id, "unknown");
    }

    #[test]
    fn destination_country_preserved_in_result() {
        let compound = json!({"id": "cpd_007"});
        let result = screen_compound_export(&compound, "FR");
        assert_eq!(result.destination_country, "FR");
    }

    #[test]
    fn screening_result_serialization_roundtrip() {
        let result = ExportScreeningResult {
            compound_id: "cpd_test".to_string(),
            risk_level: ExportRisk::Medium,
            flags: vec!["test_flag".to_string()],
            destination_country: "US".to_string(),
            requires_license: false,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ExportScreeningResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.risk_level, ExportRisk::Medium);
        assert_eq!(deserialized.compound_id, "cpd_test");
    }
}
