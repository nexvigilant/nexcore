//! FAERS field parsing utilities.
//!
//! Low-level parsing functions for FAERS ASCII file fields.

use crate::faers::{AgeUnit, DrugRole};

/// Split a FAERS line by the `$` delimiter.
///
/// FAERS ASCII files use `$` as the field separator.
#[must_use]
pub fn split_faers_line(line: &str) -> Vec<&str> {
    line.trim_end_matches(['\r', '\n']).split('$').collect()
}

/// Parse an integer field, returning None for empty/invalid values.
#[must_use]
pub fn parse_int_field(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse().ok()
}

/// Parse an i64 field, returning None for empty/invalid values.
#[must_use]
pub fn parse_i64_field(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse().ok()
}

/// Parse a float field, returning None for empty/invalid values.
#[must_use]
pub fn parse_float_field(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse().ok()
}

/// Parse a string field, returning None for empty values.
#[must_use]
pub fn parse_str_field(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse a FAERS date field.
///
/// FAERS dates are typically YYYYMMDD format but may have variations.
/// Returns the cleaned numeric string if valid.
#[must_use]
pub fn parse_faers_date(date_str: &str) -> Option<String> {
    let cleaned: String = date_str.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() >= 4 {
        Some(cleaned)
    } else {
        None
    }
}

/// Normalize a drug name to uppercase for consistent matching.
#[must_use]
pub fn normalize_drug_name(drugname: &str) -> String {
    drugname.trim().to_uppercase()
}

/// Parse a role code string to DrugRole enum.
///
/// Defaults to Concomitant for unrecognized codes.
#[must_use]
pub fn parse_role_code(role_cod: &str) -> DrugRole {
    DrugRole::from_code(role_cod).unwrap_or(DrugRole::Concomitant)
}

/// Check if a role code indicates a suspect drug.
#[must_use]
pub fn is_suspect_drug(role_cod: &str) -> bool {
    let upper = role_cod.to_uppercase();
    upper == "PS" || upper == "SS"
}

/// Normalize age value to years using the age unit code.
#[must_use]
pub fn normalize_age_to_years(age: Option<f64>, age_cod: Option<&str>) -> Option<f64> {
    match (age, age_cod) {
        (Some(a), Some(cod)) => {
            let factor = AgeUnit::from_code(cod)
                .map(|u| u.to_years_factor())
                .unwrap_or(1.0);
            Some(a * factor)
        }
        (Some(a), None) => Some(a),
        _ => None,
    }
}

/// Get a field from a slice by index, returning empty string if out of bounds.
#[inline]
#[must_use]
pub fn get_field<'a>(fields: &[&'a str], idx: usize) -> &'a str {
    fields.get(idx).copied().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_faers_line() {
        let fields = split_faers_line("101$202$303\n");
        assert_eq!(fields, vec!["101", "202", "303"]);
    }

    #[test]
    fn test_split_faers_line_with_empty_fields() {
        let fields = split_faers_line("101$$303$\r\n");
        assert_eq!(fields, vec!["101", "", "303", ""]);
    }

    #[test]
    fn test_parse_int_field() {
        assert_eq!(parse_int_field("123"), Some(123));
        assert_eq!(parse_int_field("  456  "), Some(456));
        assert_eq!(parse_int_field(""), None);
        assert_eq!(parse_int_field("abc"), None);
    }

    #[test]
    fn test_parse_float_field() {
        assert_eq!(parse_float_field("12.5"), Some(12.5));
        assert_eq!(parse_float_field("  3.14  "), Some(3.14));
        assert_eq!(parse_float_field(""), None);
        assert_eq!(parse_float_field("abc"), None);
    }

    #[test]
    fn test_parse_str_field() {
        assert_eq!(parse_str_field("test"), Some("test".to_string()));
        assert_eq!(parse_str_field("  trim  "), Some("trim".to_string()));
        assert_eq!(parse_str_field(""), None);
        assert_eq!(parse_str_field("   "), None);
    }

    #[test]
    fn test_parse_faers_date() {
        assert_eq!(parse_faers_date("20230115"), Some("20230115".to_string()));
        assert_eq!(parse_faers_date("2023-01-15"), Some("20230115".to_string()));
        assert_eq!(parse_faers_date("2023"), Some("2023".to_string()));
        assert_eq!(parse_faers_date(""), None);
        assert_eq!(parse_faers_date("abc"), None);
    }

    #[test]
    fn test_normalize_drug_name() {
        assert_eq!(normalize_drug_name("aspirin"), "ASPIRIN");
        assert_eq!(normalize_drug_name("  Ibuprofen  "), "IBUPROFEN");
    }

    #[test]
    fn test_parse_role_code() {
        assert_eq!(parse_role_code("PS"), DrugRole::PrimarySuspect);
        assert_eq!(parse_role_code("ss"), DrugRole::SecondarySuspect);
        assert_eq!(parse_role_code("UNKNOWN"), DrugRole::Concomitant);
    }

    #[test]
    fn test_is_suspect_drug() {
        assert!(is_suspect_drug("PS"));
        assert!(is_suspect_drug("ss"));
        assert!(!is_suspect_drug("C"));
        assert!(!is_suspect_drug("I"));
    }

    #[test]
    fn test_normalize_age_to_years() {
        assert_eq!(normalize_age_to_years(Some(12.0), Some("MON")), Some(1.0));
        assert_eq!(normalize_age_to_years(Some(1.0), Some("YR")), Some(1.0));
        assert_eq!(normalize_age_to_years(Some(2.0), Some("DEC")), Some(20.0));
        assert_eq!(normalize_age_to_years(Some(5.0), None), Some(5.0));
        assert_eq!(normalize_age_to_years(None, Some("YR")), None);
    }

    #[test]
    fn test_get_field() {
        let fields = vec!["a", "b", "c"];
        assert_eq!(get_field(&fields, 0), "a");
        assert_eq!(get_field(&fields, 2), "c");
        assert_eq!(get_field(&fields, 10), "");
    }
}
