//! FDA Form 3500 - MedWatch Health Professionals Voluntary Reporting
//!
//! Form Approved: OMB No. 0910-0291
//! Expires: 09-30-2027
//!
//! For use by health professionals for VOLUNTARY reporting of:
//! - Adverse events
//! - Product problems
//! - Product use/medication errors

use serde::{Deserialize, Serialize};

use super::{
    Age, AlsoReportedTo, ContactInfo, DeviceOperator, DosingInformation, FdaDate, FormMetadata,
    ImplantInfo, LabTest, MedicalDeviceInfo, ProductAvailability, ProductInformation, PurchaseInfo,
    Race, ReportType, SeriousOutcomes, Sex, SubmissionInfo, TherapyDates, ValidationResult, Weight,
};

/// Complete FDA Form 3500 (Health Professionals)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Form3500 {
    pub metadata: FormMetadata,
    pub section_a: PatientInformation,
    pub section_b: AdverseEventProductProblem,
    pub section_c: ProductAvailabilitySection,
    pub section_d: SuspectProducts,
    /// Optional - only for device reports
    pub section_e: Option<SuspectMedicalDevice>,
    pub section_f: ConcomitantMedicalProducts,
    pub section_g: ReporterInformation,
    pub submission: Option<SubmissionInfo>,
}

/// SECTION A: PATIENT INFORMATION
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PatientInformation {
    /// A1: Patient Identifier (In confidence)
    /// Use initials, patient number, or identifier
    /// NEVER use name or SSN
    pub patient_identifier: String,

    /// A2: Age
    pub age: Option<Age>,

    /// A2: Date of Birth (alternative to age)
    pub date_of_birth: Option<FdaDate>,

    /// A3: Sex
    pub sex: Option<Sex>,

    /// A4: Weight
    pub weight: Option<Weight>,

    /// A5-A6: Race and/or Ethnicity
    /// Select all that apply. Do NOT make a best guess.
    #[serde(default)]
    pub race: Vec<Race>,
}

/// SECTION B: ADVERSE EVENT, PRODUCT PROBLEM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AdverseEventProductProblem {
    /// B1: Type of Report (check all that apply)
    #[serde(default)]
    pub report_types: Vec<ReportType>,

    /// B2: Outcome Attributed to Adverse Event (check all that apply)
    #[serde(default)]
    pub outcomes: SeriousOutcomes,

    /// B3: Date of Event
    pub date_of_event: Option<FdaDate>,

    /// B4: Date of this Report
    pub date_of_report: Option<FdaDate>,

    /// B5: Describe Event, Problem or Product Use/Medication Error
    /// Max 4,000 characters
    #[serde(default)]
    pub description: String,

    /// B6: Relevant Tests/Laboratory Data (include dates if known)
    #[serde(default)]
    pub lab_data: Vec<LabTest>,

    /// B6: Additional Lab Comments (max 2,000 characters)
    pub lab_comments: Option<String>,

    /// B7: Other Relevant History, Including Preexisting Medical Conditions
    /// (e.g., allergies, pregnancy, tobacco use, liver/kidney problems)
    /// Max 2,000 characters
    pub relevant_history: Option<String>,
}

/// SECTION C: PRODUCT AVAILABILITY
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProductAvailabilitySection {
    /// C1: Product Available for Evaluation?
    /// DO NOT send product to FDA
    #[serde(default)]
    pub availability: ProductAvailability,

    /// C2: Do you have a picture of the product?
    pub has_photograph: Option<bool>,
}

/// SECTION D: SUSPECT PRODUCTS
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SuspectProducts {
    #[serde(default)]
    pub products: Vec<SuspectProduct>,
}

/// Individual suspect product entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SuspectProduct {
    /// D1: Name, Strength, Manufacturer/Compounder
    #[serde(default)]
    pub product: ProductInformation,

    /// D1: Place and Date of Purchase
    pub purchase: Option<PurchaseInfo>,

    /// D2: Dose or Amount
    pub dosing: Option<DosingInformation>,

    /// D3: Treatment/Therapy/Usage Dates
    pub therapy_dates: Option<TherapyDates>,

    /// D4: Diagnosis for use (Indication)
    pub indication: Option<String>,

    /// D5: Product Type (check all that apply)
    #[serde(default)]
    pub product_types: Vec<String>,

    /// D6: Expiration Date
    pub expiration_date: Option<FdaDate>,

    /// D7: Event Abated after use Stopped or Dose Reduced?
    pub event_abated: Option<EventAbatedStatus>,

    /// D8: Event Reappeared after Reintroduction?
    pub event_reappeared: Option<EventReappearedStatus>,
}

/// Status for D7: Event Abated after use Stopped or Dose Reduced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventAbatedStatus {
    Yes,
    No,
    DoesNotApply,
}

/// Status for D8: Event Reappeared after Reintroduction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventReappearedStatus {
    Yes,
    No,
    DoesNotApply,
}

/// SECTION E: SUSPECT MEDICAL DEVICE
/// Only for medical device reports
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SuspectMedicalDevice {
    /// E1: Brand Name
    pub brand_name: Option<String>,

    /// E2a: Procode (3-letter FDA classification)
    pub procode: Option<String>,

    /// E2b: Common Device Name
    pub common_device_name: Option<String>,

    /// E3: Manufacturer Name, City and State
    pub manufacturer: Option<DeviceManufacturerInfo>,

    /// E4: Model #, Lot #, Catalog #, Expiration Date, Serial #, UDI #
    #[serde(default)]
    pub device_info: MedicalDeviceInfo,

    /// E5: Operator of device
    pub operator: Option<DeviceOperator>,

    /// E6: Implant information
    pub implant: Option<ImplantInfo>,

    /// E7a: Is this a single-use device that was reprocessed and reused?
    pub reprocessed_single_use: Option<bool>,

    /// E7b: If Yes to E7a, Enter Name, Address of Reprocessor
    pub reprocessor_info: Option<ContactInfo>,

    /// E8: Was this device ever serviced by a third-party servicer?
    pub third_party_serviced: Option<ThirdPartyServicedStatus>,
}

/// Device manufacturer info for Section E
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeviceManufacturerInfo {
    pub name: String,
    pub city: Option<String>,
    pub state: Option<String>,
}

/// Third-party serviced status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThirdPartyServicedStatus {
    Yes,
    No,
    Unknown,
}

/// SECTION F: OTHER (CONCOMITANT) MEDICAL PRODUCTS
/// Products used concurrently but not suspected in the event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConcomitantMedicalProducts {
    #[serde(default)]
    pub products: Vec<ConcomitantProduct>,
}

/// Concomitant product entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConcomitantProduct {
    pub product_name: String,
    pub therapy_start_date: Option<FdaDate>,
    pub therapy_end_date: Option<FdaDate>,
}

/// SECTION G: REPORTER
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReporterInformation {
    /// G1: Name and Address (required for follow-up)
    #[serde(default)]
    pub contact: ContactInfo,

    /// G2: Health Professional?
    #[serde(default)]
    pub is_health_professional: bool,

    /// G3: Occupation
    pub occupation: Option<String>,

    /// G3: Specialty (if applicable)
    pub specialty: Option<String>,

    /// G4: Also Reported to
    pub also_reported_to: Option<AlsoReportedTo>,

    /// G5: If you do NOT want your identity disclosed to the manufacturer
    /// Note: Reporter identity may be shared with manufacturer unless blocked
    /// Patient identity is ALWAYS confidential
    pub withhold_identity_from_manufacturer: Option<bool>,
}

/// Form 3500 validation result with form reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Form3500Validation {
    #[serde(flatten)]
    pub result: ValidationResult,
    pub form: Form3500,
}
