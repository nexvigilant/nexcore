//! Typed structs for openFDA food endpoint responses.
//!
//! Covers:
//! - `/food/enforcement.json` — food recalls
//! - `/food/event.json` — CAERS adverse event reports

use serde::{Deserialize, Serialize};

// =============================================================================
// Food Recall / Enforcement (/food/enforcement.json)
// =============================================================================

/// A food recall record from FDA enforcement actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodRecall {
    /// Unique recall number assigned by FDA.
    #[serde(default)]
    pub recall_number: String,
    /// Recall classification (Class I = most serious, Class III = least serious).
    #[serde(default)]
    pub classification: String,
    /// Date recall was initiated (YYYYMMDD).
    #[serde(default)]
    pub recall_initiation_date: String,
    /// Date FDA center classified the recall.
    #[serde(default)]
    pub center_classification_date: String,
    /// Name of the firm initiating the recall.
    #[serde(default)]
    pub recalling_firm: String,
    /// City of the recalling firm.
    #[serde(default)]
    pub city: String,
    /// State of the recalling firm.
    #[serde(default)]
    pub state: String,
    /// Country of the recalling firm.
    #[serde(default)]
    pub country: String,
    /// Reason the product is being recalled.
    #[serde(default)]
    pub reason_for_recall: String,
    /// Description of the recalled product.
    #[serde(default)]
    pub product_description: String,
    /// Geographic distribution of the recalled product.
    #[serde(default)]
    pub distribution_pattern: String,
    /// Current status of the recall (Ongoing, Completed, Terminated).
    #[serde(default)]
    pub status: String,
    /// Whether recall was voluntary or mandated.
    #[serde(default)]
    pub voluntary_mandated: String,
    /// Quantity of product subject to recall.
    #[serde(default)]
    pub product_quantity: String,
    /// Lot or batch identifiers.
    #[serde(default)]
    pub code_info: String,
    /// Date the recall was first published.
    #[serde(default)]
    pub report_date: String,
    /// Date the recall was terminated (if applicable).
    #[serde(default)]
    pub termination_date: Option<String>,
    /// Event identifier.
    #[serde(default)]
    pub event_id: Option<String>,
    /// FDA product type.
    #[serde(default)]
    pub product_type: String,
}

// =============================================================================
// Food Event / CAERS (/food/event.json)
// =============================================================================

/// A food adverse event report from the CAERS (CFSAN Adverse Event Reporting System).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodEvent {
    /// CAERS report number.
    #[serde(default)]
    pub report_number: String,
    /// Date the report was created (YYYYMMDD).
    #[serde(default)]
    pub date_created: String,
    /// Date the adverse event started (YYYYMMDD).
    #[serde(default)]
    pub date_started: String,
    /// Health outcomes associated with the event.
    #[serde(default)]
    pub outcomes: Vec<FoodOutcome>,
    /// Products implicated in the adverse event.
    #[serde(default)]
    pub products: Vec<FoodProduct>,
    /// Reactions reported.
    #[serde(default)]
    pub reactions: Vec<FoodReaction>,
    /// Consumer (patient) information.
    #[serde(default)]
    pub consumer: Option<FoodConsumer>,
}

/// A reported health outcome in a CAERS event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoodOutcome {
    /// Outcome description (e.g., "Hospitalization", "ER Visit", "Death").
    #[serde(default)]
    pub outcome: String,
}

/// A food/dietary supplement product implicated in a CAERS event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoodProduct {
    /// Brand name of the product.
    #[serde(default)]
    pub name_brand: String,
    /// FDA industry code for the product category.
    #[serde(default)]
    pub industry_code: String,
    /// Human-readable industry name.
    #[serde(default)]
    pub industry_name: String,
    /// Role of the product in the event (suspect, concomitant).
    #[serde(default)]
    pub role: Option<String>,
}

/// A reported reaction in a CAERS event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoodReaction {
    /// MedDRA-coded reaction term.
    #[serde(default)]
    pub reaction_coded: String,
}

/// Consumer (patient) demographic information in a CAERS event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoodConsumer {
    /// Consumer age (numeric).
    #[serde(default)]
    pub age: Option<f64>,
    /// Age unit (e.g., "years", "months").
    #[serde(default)]
    pub age_unit: Option<String>,
    /// Consumer gender (Male, Female).
    #[serde(default)]
    pub gender: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_recall_deserialize_minimal() {
        let json = r#"{
            "recall_number": "F-0001-2024",
            "classification": "Class I",
            "status": "Ongoing",
            "reason_for_recall": "Undeclared allergen"
        }"#;
        let recall: FoodRecall =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(recall.classification, "Class I");
        assert_eq!(recall.reason_for_recall, "Undeclared allergen");
        assert!(recall.termination_date.is_none());
    }

    #[test]
    fn food_event_deserialize_minimal() {
        let json = r#"{
            "report_number": "CAERS-2024-001",
            "date_created": "20240101"
        }"#;
        let event: FoodEvent =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(event.report_number, "CAERS-2024-001");
        assert!(event.reactions.is_empty());
        assert!(event.consumer.is_none());
    }

    #[test]
    fn food_event_with_consumer() {
        let json = r#"{
            "report_number": "1",
            "date_created": "20240101",
            "consumer": {"age": 42.0, "gender": "Female"}
        }"#;
        let event: FoodEvent =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        let consumer = event.consumer.as_ref().expect("consumer present");
        assert_eq!(consumer.gender.as_deref(), Some("Female"));
        assert!((consumer.age.unwrap_or(0.0) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn food_outcome_default() {
        let outcome = FoodOutcome::default();
        assert!(outcome.outcome.is_empty());
    }
}
