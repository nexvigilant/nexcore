//! Typed structs for all openFDA drug endpoint responses.
//!
//! Covers:
//! - `/drug/event.json` — adverse event reports (FAERS)
//! - `/drug/label.json` — structured product labels (SPL)
//! - `/drug/enforcement.json` — drug recalls
//! - `/drug/ndc.json` — National Drug Code directory
//! - `/drug/drugsfda.json` — Drugs@FDA applications

use serde::{Deserialize, Serialize};

use super::common::OpenFdaEnrichment;

// =============================================================================
// Drug Event (/drug/event.json)
// =============================================================================

/// A single FDA adverse event report from the FAERS database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEvent {
    /// FAERS safety report identifier.
    #[serde(default)]
    pub safetyreportid: String,
    /// Date report received by FDA (YYYYMMDD).
    #[serde(default)]
    pub receiptdate: String,
    /// Report type (1=spontaneous, 2=report from study, 3=not applicable, 4=other, 5=not available).
    #[serde(default)]
    pub reporttype: String,
    /// Overall seriousness (1=serious, 2=not serious).
    #[serde(default)]
    pub serious: String,
    /// Seriousness — death outcome.
    #[serde(default)]
    pub seriousnessdeath: Option<String>,
    /// Seriousness — resulted in hospitalization.
    #[serde(default)]
    pub seriousnesshospitalization: Option<String>,
    /// Seriousness — caused a disability.
    #[serde(default)]
    pub seriousnessdisabling: Option<String>,
    /// Seriousness — congenital anomaly.
    #[serde(default)]
    pub seriousnesscongenitalanomali: Option<String>,
    /// Seriousness — life-threatening.
    #[serde(default)]
    pub seriousnesslifethreatening: Option<String>,
    /// Seriousness — other serious.
    #[serde(default)]
    pub seriousnessother: Option<String>,
    /// Information about the initial reporter.
    #[serde(default)]
    pub primarysource: Option<PrimarySource>,
    /// Patient demographics and drug/reaction information.
    #[serde(default)]
    pub patient: Option<Patient>,
}

/// Origin of the adverse event report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimarySource {
    /// Country where the report originated.
    #[serde(default)]
    pub reportercountry: Option<String>,
    /// Reporter qualification (1=Physician, 2=Pharmacist, 3=Other health professional,
    /// 4=Lawyer, 5=Consumer or non-health professional).
    #[serde(default)]
    pub qualification: Option<String>,
}

/// Patient demographic and clinical information within a drug event report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Patient {
    /// Patient age at time of event onset.
    #[serde(default)]
    pub patientonsetage: Option<String>,
    /// Unit for `patientonsetage` (800=decade, 801=year, 802=month, 803=week, 804=day, 805=hour).
    #[serde(default)]
    pub patientonsetageunit: Option<String>,
    /// Patient sex (0=unknown, 1=male, 2=female).
    #[serde(default)]
    pub patientsex: Option<String>,
    /// Patient weight in kilograms.
    #[serde(default)]
    pub patientweight: Option<String>,
    /// Drugs involved in the adverse event.
    #[serde(default)]
    pub drug: Vec<EventDrug>,
    /// Adverse reactions experienced.
    #[serde(default)]
    pub reaction: Vec<Reaction>,
}

/// A single drug entry within a patient's adverse event report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDrug {
    /// Drug characterization (1=suspect, 2=concomitant, 3=interacting).
    #[serde(default)]
    pub drugcharacterization: String,
    /// Drug name as reported.
    #[serde(default)]
    pub medicinalproduct: String,
    /// openFDA cross-reference annotation.
    #[serde(default)]
    pub openfda: Option<OpenFdaEnrichment>,
}

/// An adverse reaction (MedDRA coded) within a patient record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    /// MedDRA preferred term describing the reaction.
    #[serde(default)]
    pub reactionmeddrapt: String,
    /// Outcome of the reaction (1=recovered/resolved, 2=recovering/resolving,
    /// 3=not recovered/not resolved, 4=recovered/resolved with sequelae,
    /// 5=fatal, 6=unknown).
    #[serde(default)]
    pub reactionoutcome: Option<String>,
}

// =============================================================================
// Drug Label (/drug/label.json)
// =============================================================================

/// A structured product label (SPL) record for a drug.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugLabel {
    /// SPL document identifier.
    #[serde(default)]
    pub id: String,
    /// SPL set identifier grouping all versions of a label.
    #[serde(default)]
    pub set_id: String,
    /// Date the label became effective (YYYYMMDD).
    #[serde(default)]
    pub effective_time: String,
    /// Version number of this SPL.
    #[serde(default)]
    pub version: String,
    /// openFDA cross-reference annotation.
    #[serde(default)]
    pub openfda: OpenFdaEnrichment,
    /// Boxed warning text (black-box warning).
    #[serde(default)]
    pub boxed_warning: Vec<String>,
    /// Warnings section text.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Indications and usage section.
    #[serde(default)]
    pub indications_and_usage: Vec<String>,
    /// Dosage and administration section.
    #[serde(default)]
    pub dosage_and_administration: Vec<String>,
    /// Contraindications section.
    #[serde(default)]
    pub contraindications: Vec<String>,
    /// Adverse reactions section.
    #[serde(default)]
    pub adverse_reactions: Vec<String>,
    /// Clinical pharmacology section.
    #[serde(default)]
    pub clinical_pharmacology: Vec<String>,
    /// Drug interactions section.
    #[serde(default)]
    pub drug_interactions: Vec<String>,
    /// Use in specific populations section.
    #[serde(default)]
    pub use_in_specific_populations: Vec<String>,
    /// Overdosage section.
    #[serde(default)]
    pub overdosage: Vec<String>,
    /// How supplied section.
    #[serde(default)]
    pub how_supplied: Vec<String>,
    /// Mechanism of action section.
    #[serde(default)]
    pub mechanism_of_action: Vec<String>,
    /// Warnings and precautions section.
    #[serde(default)]
    pub warnings_and_cautions: Vec<String>,
}

// =============================================================================
// Drug Recall / Enforcement (/drug/enforcement.json)
// =============================================================================

/// A drug recall record from FDA enforcement actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugRecall {
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
    /// Event identifier linking related recall entries.
    #[serde(default)]
    pub event_id: Option<String>,
}

// =============================================================================
// Drug NDC (/drug/ndc.json)
// =============================================================================

/// An NDC directory product record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugNdc {
    /// Product identifier.
    #[serde(default)]
    pub product_id: String,
    /// 10-digit National Drug Code.
    #[serde(default)]
    pub product_ndc: String,
    /// Product type (e.g., "HUMAN PRESCRIPTION DRUG").
    #[serde(default)]
    pub product_type: String,
    /// Whether the product is a finished drug product.
    #[serde(default)]
    pub finished: bool,
    /// Proprietary (brand) name.
    #[serde(default)]
    pub brand_name: String,
    /// Base brand name (without dosage form suffix).
    #[serde(default)]
    pub brand_name_base: String,
    /// Non-proprietary (generic) name.
    #[serde(default)]
    pub generic_name: String,
    /// Dosage form.
    #[serde(default)]
    pub dosage_form: String,
    /// Routes of administration.
    #[serde(default)]
    pub route: Vec<String>,
    /// Marketing category (e.g., "NDA", "ANDA").
    #[serde(default)]
    pub marketing_category: String,
    /// NDA/ANDA application number.
    #[serde(default)]
    pub application_number: String,
    /// Name of the labeler/manufacturer.
    #[serde(default)]
    pub labeler_name: String,
    /// Active ingredients with strengths.
    #[serde(default)]
    pub active_ingredients: Vec<ActiveIngredient>,
    /// Packaging configurations.
    #[serde(default)]
    pub packaging: Vec<NdcPackaging>,
    /// openFDA cross-reference annotation.
    #[serde(default)]
    pub openfda: OpenFdaEnrichment,
    /// Date the product listing expires (YYYYMMDD).
    #[serde(default)]
    pub listing_expiration_date: Option<String>,
    /// Date marketing began (YYYYMMDD).
    #[serde(default)]
    pub marketing_start_date: Option<String>,
    /// Date marketing ended (YYYYMMDD), if applicable.
    #[serde(default)]
    pub marketing_end_date: Option<String>,
}

/// An active ingredient and its labeled strength.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActiveIngredient {
    /// Ingredient name.
    #[serde(default)]
    pub name: String,
    /// Labeled strength (e.g., "500 mg/1").
    #[serde(default)]
    pub strength: String,
}

/// A packaging configuration for an NDC product.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NdcPackaging {
    /// Package-level NDC code.
    #[serde(default)]
    pub package_ndc: String,
    /// Human-readable package description.
    #[serde(default)]
    pub description: String,
    /// Date this packaging began marketing (YYYYMMDD).
    #[serde(default)]
    pub marketing_start_date: Option<String>,
    /// Date this packaging ended marketing (YYYYMMDD), if applicable.
    #[serde(default)]
    pub marketing_end_date: Option<String>,
    /// Whether this is a sample package.
    #[serde(default)]
    pub sample: Option<bool>,
}

// =============================================================================
// Drugs@FDA (/drug/drugsfda.json)
// =============================================================================

/// An NDA/BLA/ANDA application from the Drugs@FDA database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugApplication {
    /// FDA application number (e.g., "NDA020500").
    #[serde(default)]
    pub application_number: String,
    /// Sponsor (applicant) name.
    #[serde(default)]
    pub sponsor_name: String,
    /// Approved drug products under this application.
    #[serde(default)]
    pub products: Vec<FdaProduct>,
    /// Regulatory submissions associated with this application.
    #[serde(default)]
    pub submissions: Vec<FdaSubmission>,
    /// openFDA cross-reference annotation.
    #[serde(default)]
    pub openfda: OpenFdaEnrichment,
}

/// A drug product approved under an NDA/BLA/ANDA application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FdaProduct {
    /// Brand name of the product.
    #[serde(default)]
    pub brand_name: String,
    /// Dosage form.
    #[serde(default)]
    pub dosage_form: String,
    /// Route of administration.
    #[serde(default)]
    pub route: String,
    /// Marketing status (Prescription, OTC, Discontinued, etc.).
    #[serde(default)]
    pub marketing_status: String,
    /// Active ingredients.
    #[serde(default)]
    pub active_ingredients: Vec<ActiveIngredient>,
    /// Whether this is the reference listed drug.
    #[serde(default)]
    pub reference_drug: String,
    /// Therapeutic equivalence code (if assigned).
    #[serde(default)]
    pub te_code: Option<String>,
    /// Product number within the application.
    #[serde(default)]
    pub product_number: Option<String>,
}

/// A regulatory submission associated with a Drugs@FDA application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FdaSubmission {
    /// Submission type (e.g., "ORIG", "SUPPL").
    #[serde(default)]
    pub submission_type: String,
    /// Submission number within the application.
    #[serde(default)]
    pub submission_number: String,
    /// Current submission status (e.g., "AP" = Approved).
    #[serde(default)]
    pub submission_status: String,
    /// Date the submission reached its current status (YYYYMMDD).
    #[serde(default)]
    pub submission_status_date: String,
    /// Review priority (STANDARD, PRIORITY).
    #[serde(default)]
    pub review_priority: Option<String>,
    /// Submission class code.
    #[serde(default)]
    pub submission_class_code: Option<String>,
    /// Application documents linked to this submission.
    #[serde(default)]
    pub application_docs: Vec<FdaApplicationDoc>,
}

/// A document attached to a Drugs@FDA submission.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FdaApplicationDoc {
    /// Document identifier.
    #[serde(default)]
    pub id: String,
    /// URL to the document.
    #[serde(default)]
    pub url: String,
    /// Document date (YYYYMMDD).
    #[serde(default)]
    pub date: Option<String>,
    /// Document type label.
    #[serde(rename = "type", default)]
    pub doc_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drug_event_deserialize_minimal() {
        let json = r#"{
            "safetyreportid": "12345",
            "serious": "1",
            "receiptdate": "20230101",
            "reporttype": "1"
        }"#;
        let event: DrugEvent = serde_json::from_str(json).expect("deserialize");
        assert_eq!(event.safetyreportid, "12345");
        assert_eq!(event.serious, "1");
        assert!(event.patient.is_none());
    }

    #[test]
    fn drug_event_seriousness_defaults() {
        let json = r#"{"safetyreportid":"1","serious":"1","receiptdate":"","reporttype":""}"#;
        let event: DrugEvent = serde_json::from_str(json).expect("deserialize");
        assert!(event.seriousnessdeath.is_none());
        assert!(event.seriousnesshospitalization.is_none());
    }

    #[test]
    fn drug_label_deserialize_minimal() {
        let json = r#"{"id":"abc","set_id":"xyz","effective_time":"20240101","version":"1"}"#;
        let label: DrugLabel = serde_json::from_str(json).expect("deserialize");
        assert_eq!(label.id, "abc");
        assert!(label.boxed_warning.is_empty());
    }

    #[test]
    fn drug_recall_deserialize() {
        let json = r#"{
            "recall_number": "D-0001-2024",
            "classification": "Class II",
            "status": "Ongoing"
        }"#;
        let recall: DrugRecall = serde_json::from_str(json).expect("deserialize");
        assert_eq!(recall.classification, "Class II");
        assert!(recall.termination_date.is_none());
    }

    #[test]
    fn drug_ndc_deserialize() {
        let json = r#"{
            "product_ndc": "12345-678",
            "brand_name": "ASPIRIN",
            "generic_name": "aspirin",
            "dosage_form": "TABLET"
        }"#;
        let ndc: DrugNdc = serde_json::from_str(json).expect("deserialize");
        assert_eq!(ndc.brand_name, "ASPIRIN");
        assert!(!ndc.finished); // default false
    }

    #[test]
    fn active_ingredient_default() {
        let ai = ActiveIngredient::default();
        assert!(ai.name.is_empty());
        assert!(ai.strength.is_empty());
    }
}
