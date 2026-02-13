//! Form validation logic for FDA MedWatch forms
//!
//! Validation rules based on FDA Form 3500 and 3500B specifications.

use std::sync::LazyLock;

use regex::Regex;

use super::{Form3500, Form3500B, Form3500BValidation, Form3500Validation, ValidationResult};

// ============================================================================
// FDA Date Validation
// ============================================================================

/// FDA Date regex pattern: dd-mmm-yyyy (e.g., 16-Oct-2019)
static FDA_DATE_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"^\d{2}-(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)-\d{4}$").ok()
});

/// Validate FDA Date format: dd-mmm-yyyy
///
/// # Examples
///
/// ```
/// use nexcore_vigilance::medwatch::validate_fda_date;
///
/// assert!(validate_fda_date(Some("16-Oct-2019")));
/// assert!(validate_fda_date(Some("01-Jan-2025")));
/// assert!(!validate_fda_date(Some("2019-10-16"))); // Wrong format
/// assert!(validate_fda_date(None)); // Optional fields pass
/// ```
#[must_use]
pub fn validate_fda_date(date: Option<&str>) -> bool {
    date.is_none_or(|d| {
        FDA_DATE_PATTERN
            .as_ref()
            .is_some_and(|pattern| pattern.is_match(d))
    })
}

/// Check if date is not in the future
#[must_use]
pub fn is_not_future_date(date: Option<&str>) -> bool {
    let Some(date_str) = date else {
        return true;
    };

    // Parse dd-mmm-yyyy
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    let Ok(day) = parts[0].parse::<u32>() else {
        return false;
    };
    let Ok(year) = parts[2].parse::<i32>() else {
        return false;
    };

    let month = match parts[1] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return false,
    };

    // Compare with current date
    let now = chrono::Utc::now();
    let current_year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2026);
    let current_month = now.format("%m").to_string().parse::<u32>().unwrap_or(1);
    let current_day = now.format("%d").to_string().parse::<u32>().unwrap_or(1);

    if year > current_year {
        return false;
    }
    if year == current_year && month > current_month {
        return false;
    }
    if year == current_year && month == current_month && day > current_day {
        return false;
    }

    true
}

// ============================================================================
// Character Limit Validation
// ============================================================================

/// Validate character limit
#[must_use]
pub const fn validate_char_limit(text: Option<&str>, max_chars: usize) -> bool {
    match text {
        Some(t) => t.len() <= max_chars,
        None => true,
    }
}

// ============================================================================
// Form 3500 Validation
// ============================================================================

/// Validate Form 3500 (Health Professional)
///
/// Validates all required fields and business rules for FDA Form 3500.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_form3500(form: Form3500) -> Form3500Validation {
    let mut result = ValidationResult::valid();

    // Section A: Patient Information
    if form.section_a.patient_identifier.trim().is_empty() {
        result.add_error(
            "sectionA.patientIdentifier",
            "Patient identifier is required (initials, patient number, or \"none\")",
        );
    }

    // Warn if patient identifier might be a name or SSN
    if form.section_a.patient_identifier.contains(' ') {
        result.add_warning(
            "sectionA.patientIdentifier",
            "Patient identifier should not be a full name. Use initials or patient number.",
        );
    }

    // Section B: Adverse Event
    if form.section_b.report_types.is_empty() {
        result.add_error(
            "sectionB.reportTypes",
            "At least one report type must be selected",
        );
    }

    if form.section_b.description.trim().is_empty() {
        result.add_error(
            "sectionB.description",
            "Event description is required (Section B5)",
        );
    }

    if !validate_char_limit(Some(&form.section_b.description), 4000) {
        result.add_error(
            "sectionB.description",
            "Event description exceeds 4,000 character limit",
        );
    }

    if !validate_char_limit(form.section_b.lab_comments.as_deref(), 2000) {
        result.add_error(
            "sectionB.labComments",
            "Lab comments exceed 2,000 character limit",
        );
    }

    if !validate_char_limit(form.section_b.relevant_history.as_deref(), 2000) {
        result.add_error(
            "sectionB.relevantHistory",
            "Relevant history exceeds 2,000 character limit",
        );
    }

    if form.section_b.date_of_report.is_none() {
        result.add_error(
            "sectionB.dateOfReport",
            "Date of report is required (Section B4)",
        );
    }

    if let Some(ref date) = form.section_b.date_of_report {
        if !validate_fda_date(Some(date)) {
            result.add_error(
                "sectionB.dateOfReport",
                "Date of report must be in format dd-mmm-yyyy (e.g., 16-Oct-2019)",
            );
        }
    }

    if let Some(ref date) = form.section_b.date_of_event {
        if !validate_fda_date(Some(date)) {
            result.add_error(
                "sectionB.dateOfEvent",
                "Date of event must be in format dd-mmm-yyyy",
            );
        }
        if !is_not_future_date(Some(date)) {
            result.add_error(
                "sectionB.dateOfEvent",
                "Date of event cannot be in the future",
            );
        }
    }

    // Section D: Suspect Products
    if form.section_d.products.is_empty() {
        result.add_error(
            "sectionD.products",
            "At least one suspect product must be specified",
        );
    }

    for (index, product) in form.section_d.products.iter().enumerate() {
        if product.product.name.trim().is_empty() {
            result.add_error(
                format!("sectionD.products[{index}].product.name"),
                format!("Product #{} must have a name", index + 1),
            );
        }
    }

    // Section E: Medical Device (if applicable)
    if let Some(ref device) = form.section_e {
        if device.common_device_name.is_none() && device.brand_name.is_none() {
            result.add_warning(
                "sectionE",
                "Provide either brand name or common device name",
            );
        }
    }

    // Section G: Reporter Information
    let has_name =
        form.section_g.contact.first_name.is_some() || form.section_g.contact.last_name.is_some();
    if !has_name {
        result.add_error(
            "sectionG.contact",
            "Reporter name is required for follow-up",
        );
    }

    let has_contact =
        form.section_g.contact.phone.is_some() || form.section_g.contact.email.is_some();
    if !has_contact {
        result.add_warning(
            "sectionG.contact",
            "Provide phone or email for FDA follow-up",
        );
    }

    if form.section_g.is_health_professional && form.section_g.occupation.is_none() {
        result.add_warning(
            "sectionG.occupation",
            "Specify occupation if you are a health professional",
        );
    }

    Form3500Validation { result, form }
}

/// Quick validation - just check required fields
#[must_use]
pub fn quick_validate_form3500(form: &Form3500) -> bool {
    !form.section_a.patient_identifier.is_empty()
        && !form.section_b.report_types.is_empty()
        && !form.section_b.description.is_empty()
        && form.section_b.date_of_report.is_some()
        && !form.section_d.products.is_empty()
        && (form.section_g.contact.first_name.is_some()
            || form.section_g.contact.last_name.is_some())
}

// ============================================================================
// Form 3500B Validation
// ============================================================================

/// Validate Form 3500B (Consumer)
///
/// Validates all required fields and business rules for FDA Form 3500B.
#[must_use]
pub fn validate_form3500b(form: Form3500B) -> Form3500BValidation {
    let mut result = ValidationResult::valid();

    // Section A: About the Problem
    if form.section_a.problem_types.is_empty() {
        result.add_error(
            "sectionA.problemTypes",
            "Please select what kind of problem it was",
        );
    }

    if form.section_a.description.trim().is_empty() {
        result.add_error(
            "sectionA.description",
            "Please tell us what happened (Section A4)",
        );
    }

    if !validate_char_limit(Some(&form.section_a.description), 4000) {
        result.add_error(
            "sectionA.description",
            "Description exceeds 4,000 character limit",
        );
    }

    if !validate_char_limit(form.section_a.additional_comments.as_deref(), 2000) {
        result.add_error(
            "sectionA.additionalComments",
            "Additional comments exceed 2,000 character limit",
        );
    }

    // Section C: About the Products (if present)
    if let Some(ref products) = form.section_c {
        if products.product_names.is_empty() {
            result.add_error(
                "sectionC.productNames",
                "Product name is required (Section C1)",
            );
        }
    }

    // Section D: About the Medical Device (if present)
    if let Some(ref device) = form.section_d {
        if device
            .device_name
            .as_ref()
            .is_none_or(|n| n.trim().is_empty())
        {
            result.add_error(
                "sectionD.deviceName",
                "Device name is required (Section D1)",
            );
        }
    }

    // Must have either Section C or Section D
    if form.section_c.is_none() && form.section_d.is_none() {
        result.add_error(
            "form",
            "Please provide information about the product or device involved",
        );
    }

    // Section E: About the Person Who Had the Problem
    if form.section_e.initials.is_none() && form.section_e.date_of_birth.is_none() {
        result.add_warning("sectionE", "Provide patient initials or date of birth");
    }

    // Section F: About the Reporter
    if form.section_f.todays_date.is_none() {
        result.add_error(
            "sectionF.todaysDate",
            "Today's date is required (Section F9)",
        );
    }

    if let Some(ref date) = form.section_f.todays_date {
        if !validate_fda_date(Some(date)) {
            result.add_error(
                "sectionF.todaysDate",
                "Date must be in format dd-mmm-yyyy (e.g., 16-Oct-2019)",
            );
        }
    }

    let has_name = form.section_f.first_name.is_some() || form.section_f.last_name.is_some();
    if !has_name {
        result.add_warning(
            "sectionF.contact",
            "Provide your name in case FDA needs to contact you",
        );
    }

    let has_contact = form.section_f.phone.is_some() || form.section_f.email.is_some();
    if !has_contact {
        result.add_warning(
            "sectionF.contact",
            "Provide phone or email in case FDA needs more information",
        );
    }

    Form3500BValidation { result, form }
}

/// Quick validation - just check required fields
#[must_use]
pub fn quick_validate_form3500b(form: &Form3500B) -> bool {
    !form.section_a.problem_types.is_empty()
        && !form.section_a.description.is_empty()
        && (form.section_c.is_some() || form.section_d.is_some())
        && form.section_f.todays_date.is_some()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_fda_date_valid() {
        assert!(validate_fda_date(Some("16-Oct-2019")));
        assert!(validate_fda_date(Some("01-Jan-2025")));
        assert!(validate_fda_date(Some("31-Dec-2024")));
        assert!(validate_fda_date(None));
    }

    #[test]
    fn test_validate_fda_date_invalid() {
        assert!(!validate_fda_date(Some("2019-10-16"))); // ISO format
        assert!(!validate_fda_date(Some("10/16/2019"))); // US format
        assert!(!validate_fda_date(Some("16-October-2019"))); // Full month
        assert!(!validate_fda_date(Some("16-oct-2019"))); // Lowercase
        assert!(!validate_fda_date(Some("invalid")));
    }

    #[test]
    fn test_validate_char_limit() {
        assert!(validate_char_limit(Some("short"), 100));
        assert!(validate_char_limit(Some("exactly"), 7));
        assert!(!validate_char_limit(Some("too long"), 5));
        assert!(validate_char_limit(None, 100));
    }

    #[test]
    fn test_is_not_future_date() {
        // Past dates should pass
        assert!(is_not_future_date(Some("01-Jan-2020")));
        assert!(is_not_future_date(Some("16-Oct-2019")));
        assert!(is_not_future_date(None));

        // Far future dates should fail
        assert!(!is_not_future_date(Some("01-Jan-2099")));
    }
}
