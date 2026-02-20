//! Typed structs for all openFDA device endpoint responses.
//!
//! Covers:
//! - `/device/event.json` — MDR adverse event reports
//! - `/device/enforcement.json` — device recalls
//! - `/device/510k.json` — 510(k) premarket notifications
//! - `/device/pma.json` — premarket approvals
//! - `/device/classification.json` — device classification
//! - `/device/udi.json` — unique device identifiers

use serde::{Deserialize, Serialize};

// =============================================================================
// Device Event / MDR (/device/event.json)
// =============================================================================

/// A single Medical Device Report (MDR) adverse event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    /// MDR report key — primary identifier.
    #[serde(default)]
    pub mdr_report_key: String,
    /// Type of event (Malfunction, Injury, Death, Other).
    #[serde(default)]
    pub event_type: String,
    /// Date the report was received by FDA (YYYYMMDD).
    #[serde(default)]
    pub date_received: String,
    /// Date the adverse event occurred (YYYYMMDD).
    #[serde(default)]
    pub date_of_event: String,
    /// Type of report (E=initial, C=correction/amendment).
    #[serde(default)]
    pub report_source_code: Option<String>,
    /// Whether the event was life-threatening (Y/N).
    #[serde(default)]
    pub is_adverse_event: Option<String>,
    /// Devices involved in the event.
    #[serde(default)]
    pub device: Vec<MdrDevice>,
    /// Patient outcomes.
    #[serde(default)]
    pub patient: Vec<MdrPatient>,
    /// Narrative text descriptions of the event.
    #[serde(default)]
    pub mdr_text: Vec<MdrText>,
    /// Manufacturer name.
    #[serde(default)]
    pub manufacturer_name: Option<String>,
    /// Manufacturer city.
    #[serde(default)]
    pub manufacturer_city: Option<String>,
    /// Manufacturer country.
    #[serde(default)]
    pub manufacturer_country: Option<String>,
    /// Manufacturer ZIP code.
    #[serde(default)]
    pub manufacturer_zip_code: Option<String>,
}

/// A device entry within an MDR report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MdrDevice {
    /// Proprietary (brand) name of the device.
    #[serde(default)]
    pub brand_name: String,
    /// Generic/common device name.
    #[serde(default)]
    pub generic_name: String,
    /// Device manufacturer name.
    #[serde(default)]
    pub manufacturer_d_name: String,
    /// FDA product code classifying the device.
    #[serde(default)]
    pub product_code: String,
    /// Device model number.
    #[serde(default)]
    pub model_number: Option<String>,
    /// Device catalog number.
    #[serde(default)]
    pub catalog_number: Option<String>,
    /// Device lot number.
    #[serde(default)]
    pub lot_number: Option<String>,
    /// UDI-DI if present.
    #[serde(default)]
    pub udi_di: Option<String>,
    /// Whether the device was implanted.
    #[serde(default)]
    pub implant_flag: Option<String>,
    /// Date the device was returned to manufacturer.
    #[serde(default)]
    pub date_returned_to_manufacturer: Option<String>,
    /// Device availability code.
    #[serde(default)]
    pub device_availability: Option<String>,
    /// Whether this is a single-use device.
    #[serde(default)]
    pub device_evaluated_by_manufacturer: Option<String>,
}

/// Patient outcome information from an MDR report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MdrPatient {
    /// Sequence number for this patient.
    #[serde(default)]
    pub sequence_number_outcome: Vec<String>,
    /// Date of treatment.
    #[serde(default)]
    pub date_received: Option<String>,
    /// Sequence number of the treatment.
    #[serde(default)]
    pub sequence_number_treatment: Vec<String>,
}

/// A narrative text block within an MDR report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MdrText {
    /// Narrative text content.
    #[serde(default)]
    pub text: String,
    /// Type code for the text (e.g., "Description of Event or Problem").
    #[serde(default)]
    pub text_type_code: String,
    /// Patient sequence number this text refers to.
    #[serde(default)]
    pub patient_sequence_number: Option<String>,
}

// =============================================================================
// Device Recall / Enforcement (/device/enforcement.json)
// =============================================================================

/// A device recall record from FDA enforcement actions.
///
/// Same field structure as drug enforcement records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecall {
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
    /// Current status of the recall.
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
// Device 510(k) (/device/510k.json)
// =============================================================================

/// A 510(k) premarket notification clearance record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device510k {
    /// FDA 510(k) number (e.g., "K230001").
    #[serde(default)]
    pub k_number: String,
    /// Applicant company name.
    #[serde(default)]
    pub applicant: String,
    /// Contact person at the applicant.
    #[serde(default)]
    pub contact: String,
    /// Device name as submitted.
    #[serde(default)]
    pub device_name: String,
    /// FDA product code classifying the device.
    #[serde(default)]
    pub product_code: String,
    /// Date 510(k) was received by FDA.
    #[serde(default)]
    pub date_received: String,
    /// Date of decision.
    #[serde(default)]
    pub decision_date: String,
    /// Decision code (SESE=Substantially Equivalent, etc.).
    #[serde(default)]
    pub decision_code: String,
    /// Human-readable decision description.
    #[serde(default)]
    pub decision_description: String,
    /// Type of clearance pathway.
    #[serde(default)]
    pub clearance_type: String,
    /// Advisory committee name.
    #[serde(default)]
    pub advisory_committee: String,
    /// Advisory committee description.
    #[serde(default)]
    pub advisory_committee_description: String,
    /// Whether expedited review was granted.
    #[serde(default)]
    pub expedited_review_flag: Option<String>,
    /// Whether reviewed by a third party.
    #[serde(default)]
    pub third_party_flag: Option<String>,
    /// Summary or statement document reference.
    #[serde(default)]
    pub statement_or_summary: Option<String>,
    /// Applicant street address.
    #[serde(default)]
    pub address_1: String,
    /// Applicant city.
    #[serde(default)]
    pub city: String,
    /// Applicant state.
    #[serde(default)]
    pub state: String,
    /// Applicant ZIP code.
    #[serde(default)]
    pub zip_code: String,
    /// Applicant country code (ISO 3166-1 alpha-2).
    #[serde(default)]
    pub country_code: String,
    /// Applicant ZIP code extension.
    #[serde(default)]
    pub zip_ext: Option<String>,
    /// Review advisory committee.
    #[serde(default)]
    pub review_advisory_committee: Option<String>,
}

// =============================================================================
// Device PMA (/device/pma.json)
// =============================================================================

/// A Premarket Approval (PMA) record for Class III devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePma {
    /// FDA PMA number (e.g., "P230001").
    #[serde(default)]
    pub pma_number: String,
    /// Applicant company name.
    #[serde(default)]
    pub applicant: String,
    /// Date of PMA decision.
    #[serde(default)]
    pub decision_date: String,
    /// Decision code (e.g., "AP" = Approved).
    #[serde(default)]
    pub decision_code: String,
    /// Advisory committee code.
    #[serde(default)]
    pub advisory_committee: String,
    /// Advisory committee description.
    #[serde(default)]
    pub advisory_committee_description: String,
    /// FDA product code.
    #[serde(default)]
    pub product_code: String,
    /// Generic device name.
    #[serde(default)]
    pub generic_name: String,
    /// Trade (brand) name.
    #[serde(default)]
    pub trade_name: String,
    /// Supplement number (e.g., "S001").
    #[serde(default)]
    pub supplement_number: String,
    /// Supplement type.
    #[serde(default)]
    pub supplement_type: String,
    /// Reason for supplement.
    #[serde(default)]
    pub supplement_reason: String,
    /// Applicant address.
    #[serde(default)]
    pub address_1: String,
    /// Applicant city.
    #[serde(default)]
    pub city: String,
    /// Applicant state.
    #[serde(default)]
    pub state: String,
    /// Applicant ZIP code.
    #[serde(default)]
    pub zip: String,
    /// Applicant country.
    #[serde(default)]
    pub country_code: String,
    /// FDA review division.
    #[serde(default)]
    pub docket_number: Option<String>,
    /// AO date.
    #[serde(default)]
    pub ao_statement: Option<String>,
}

// =============================================================================
// Device Classification (/device/classification.json)
// =============================================================================

/// An FDA device classification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceClass {
    /// FDA product code (3 letters, e.g., "FPA").
    #[serde(default)]
    pub product_code: String,
    /// Device name for this product code.
    #[serde(default)]
    pub device_name: String,
    /// Device class (1=Class I, 2=Class II, 3=Class III, U=Unclassified, F=HDE, N=Not classified).
    #[serde(default)]
    pub device_class: String,
    /// Medical specialty code.
    #[serde(default)]
    pub medical_specialty: String,
    /// Medical specialty description.
    #[serde(default)]
    pub medical_specialty_description: String,
    /// CFR regulation number.
    #[serde(default)]
    pub regulation_number: String,
    /// Primary submission type identifier.
    #[serde(default)]
    pub submission_type_id: String,
    /// Definition of the device type.
    #[serde(default)]
    pub definition: String,
    /// Reason for unclassified status (if applicable).
    #[serde(default)]
    pub unclassified_reason: Option<String>,
    /// Implant flag.
    #[serde(default)]
    pub implant_flag: Option<String>,
    /// Life-sustain/support flag.
    #[serde(default)]
    pub life_sustain_support_flag: Option<String>,
    /// GMP (Good Manufacturing Practices) exempt flag.
    #[serde(default)]
    pub gmp_exempt_flag: Option<String>,
    /// Third-party review eligible flag.
    #[serde(default)]
    pub third_party_flag: Option<String>,
    /// Review panel code.
    #[serde(default)]
    pub review_panel: Option<String>,
}

// =============================================================================
// Device UDI (/device/udi.json)
// =============================================================================

/// A Unique Device Identifier (UDI) record from the Global UDI Database (GUDID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceUdi {
    /// GUDID record key.
    #[serde(default)]
    pub record_key: String,
    /// Primary DI (Device Identifier) from the UDI.
    #[serde(default)]
    pub device_id: String,
    /// Brand name of the device.
    #[serde(default)]
    pub brand_name: String,
    /// Company (labeler) name.
    #[serde(default)]
    pub company_name: String,
    /// Catalog number.
    #[serde(default)]
    pub catalog_number: String,
    /// Version or model number.
    #[serde(default)]
    pub version_or_model_number: String,
    /// Human-readable device description.
    #[serde(default)]
    pub device_description: String,
    /// Commercial distribution status (In Commercial Distribution, Not in Commercial Distribution).
    #[serde(default)]
    pub commercial_distribution_status: String,
    /// MRI safety status (Labeling does not contain MRI Safety Information, MR Conditional, etc.).
    #[serde(default)]
    pub mri_safety: String,
    /// Labeler DUNS number.
    #[serde(default)]
    pub labeler_duns_number: String,
    /// UDI identifiers (DI, PI, etc.).
    #[serde(default)]
    pub identifiers: Vec<UdiIdentifier>,
    /// Device sizes and dimensions.
    #[serde(default)]
    pub device_sizes: Vec<UdiDeviceSize>,
    /// Sterilization information.
    #[serde(default)]
    pub sterilization: Vec<UdiSterilization>,
    /// Whether device contains latex.
    #[serde(default)]
    pub has_latex: Option<bool>,
    /// Whether device requires prescription use.
    #[serde(default)]
    pub is_rx: Option<bool>,
    /// Whether device is for over-the-counter use.
    #[serde(default)]
    pub is_otc: Option<bool>,
    /// Whether device is implantable.
    #[serde(default)]
    pub is_implantable: Option<bool>,
    /// Whether device is single-use.
    #[serde(default)]
    pub is_single_use: Option<bool>,
}

/// A UDI identifier entry (DI, PI, or combination).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UdiIdentifier {
    /// Identifier type (Primary DI Number, Package DI Number, etc.).
    #[serde(rename = "type", default)]
    pub id_type: String,
    /// The identifier value.
    #[serde(default)]
    pub id: String,
    /// Issuing agency (GS1, HIBCC, ICCBBA, etc.).
    #[serde(default)]
    pub issuing_agency: String,
    /// Package type (if this is a package DI).
    #[serde(default)]
    pub package_type: Option<String>,
    /// Quantity per unit-of-use (if applicable).
    #[serde(default)]
    pub quantity_per_package: Option<u32>,
    /// Whether this identifier is the device ID.
    #[serde(default)]
    pub unit_of_use_id: Option<String>,
}

/// Size/dimension information for a UDI device.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UdiDeviceSize {
    /// Size type (e.g., "Height", "Width", "Depth").
    #[serde(rename = "sizeType", default)]
    pub size_type: String,
    /// Numeric size value.
    #[serde(default)]
    pub value: Option<f64>,
    /// Unit of measurement.
    #[serde(default)]
    pub unit: String,
    /// Text description of size.
    #[serde(default)]
    pub text: Option<String>,
}

/// Sterilization information for a UDI device.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UdiSterilization {
    /// Whether the device is provided sterile.
    #[serde(default)]
    pub is_sterile: Option<bool>,
    /// Whether the device requires sterilization before use.
    #[serde(default)]
    pub is_sterilization_prior_use: Option<bool>,
    /// Sterilization methods used.
    #[serde(default)]
    pub sterilization_methods: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_event_deserialize_minimal() {
        let json = r#"{
            "mdr_report_key": "9999999",
            "event_type": "Malfunction",
            "date_received": "20230601"
        }"#;
        let event: DeviceEvent =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(event.mdr_report_key, "9999999");
        assert_eq!(event.event_type, "Malfunction");
        assert!(event.device.is_empty());
    }

    #[test]
    fn device_510k_deserialize_minimal() {
        let json = r#"{
            "k_number": "K230001",
            "device_name": "TEST DEVICE",
            "decision_code": "SESE"
        }"#;
        let k: Device510k =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(k.k_number, "K230001");
        assert_eq!(k.decision_code, "SESE");
    }

    #[test]
    fn device_class_deserialize_minimal() {
        let json = r#"{
            "product_code": "FPA",
            "device_name": "PACEMAKER",
            "device_class": "3"
        }"#;
        let dc: DeviceClass =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(dc.device_class, "3");
        assert!(dc.unclassified_reason.is_none());
    }

    #[test]
    fn device_udi_identifiers_default_empty() {
        let udi = DeviceUdi {
            record_key: "R1".to_string(),
            device_id: "D1".to_string(),
            brand_name: String::new(),
            company_name: String::new(),
            catalog_number: String::new(),
            version_or_model_number: String::new(),
            device_description: String::new(),
            commercial_distribution_status: String::new(),
            mri_safety: String::new(),
            labeler_duns_number: String::new(),
            identifiers: Vec::new(),
            device_sizes: Vec::new(),
            sterilization: Vec::new(),
            has_latex: None,
            is_rx: None,
            is_otc: None,
            is_implantable: None,
            is_single_use: None,
        };
        assert!(udi.identifiers.is_empty());
    }
}
