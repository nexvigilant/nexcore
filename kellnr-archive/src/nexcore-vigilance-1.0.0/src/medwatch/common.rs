//! Common types shared across FDA Form 3500 and Form 3500B
//!
//! These types represent the standard FDA data elements used in MedWatch reporting.

use serde::{Deserialize, Serialize};

// ============================================================================
// Date Types
// ============================================================================

/// FDA Date format: dd-mmm-yyyy (e.g., "16-Oct-2019")
///
/// This is the standard date format used in MedWatch forms.
pub type FdaDate = String;

// ============================================================================
// Demographics
// ============================================================================

/// Patient/Person Sex
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    #[default]
    Male,
    Female,
}

/// Race/Ethnicity options (FDA standard categories)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Race {
    AmericanIndianAlaskaNative,
    Asian,
    BlackAfricanAmerican,
    HispanicLatino,
    MiddleEasternNorthAfrican,
    NativeHawaiianPacificIslander,
    White,
}

/// Age unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AgeUnit {
    #[default]
    Years,
    Months,
    Weeks,
    Days,
}

/// Age specification with unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Age {
    pub value: f64,
    #[serde(default)]
    pub unit: AgeUnit,
}

/// Weight unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WeightUnit {
    Lb,
    #[default]
    Kg,
}

/// Weight specification with unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Weight {
    pub value: f64,
    #[serde(default)]
    pub unit: WeightUnit,
}

// ============================================================================
// Report Classification
// ============================================================================

/// Report type classification (Form 3500)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportType {
    AdverseEvent,
    ProductUseError,
    ProductProblem,
    ManufacturerProblem,
}

/// Problem type (Form 3500B - consumer language)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProblemType {
    /// Were hurt or had a bad side effect
    HurtOrBadSideEffect,
    /// Used product incorrectly
    UsedIncorrectly,
    /// Noticed problem with quality
    QualityProblem,
    /// Had problems after switching manufacturers
    ManufacturerProblem,
}

// ============================================================================
// Outcomes
// ============================================================================

/// Death outcome details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeathOutcome {
    pub occurred: bool,
    pub date_of_death: Option<FdaDate>,
}

/// Hospitalization outcome details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HospitalizationOutcome {
    pub occurred: bool,
    pub initial: Option<bool>,
    pub prolonged: Option<bool>,
}

/// Serious outcomes (FDA criteria) - Form 3500
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SeriousOutcomes {
    pub death: Option<DeathOutcome>,
    pub life_threatening: Option<bool>,
    pub hospitalization: Option<HospitalizationOutcome>,
    pub disability: Option<bool>,
    pub congenital_anomaly: Option<bool>,
    pub required_intervention: Option<bool>,
    pub other_serious_medical_event: Option<bool>,
}

/// Consumer outcomes (Form 3500B - consumer-friendly language)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConsumerOutcomes {
    /// Hospitalization - admitted or stayed longer
    pub hospitalization: Option<bool>,
    /// Required help to prevent permanent harm
    pub required_intervention: Option<bool>,
    /// Disability or health problem
    pub disability: Option<bool>,
    /// Birth defect
    pub birth_defect: Option<bool>,
    /// Life-threatening
    pub life_threatening: Option<bool>,
    /// Death
    pub death: Option<DeathOutcome>,
    /// Other serious/important medical event
    pub other_serious: Option<bool>,
}

// ============================================================================
// Product Types
// ============================================================================

/// Product type categories (Form 3500)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductType {
    DrugBiologic,
    Brand,
    Generic,
    Biosimilar,
    OverTheCounter,
    Compounded,
    CosmeticProfessional,
    CosmeticRetail,
    CannabinoidHemp,
    Other,
}

/// Consumer product types (Form 3500B - simpler)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsumerProductType {
    Brand,
    GenericOrBiosimilar,
    OverTheCounter,
    Compounded,
    CosmeticProfessional,
    CosmeticRetail,
    CannabinoidHemp,
    Other,
}

// ============================================================================
// Administration
// ============================================================================

/// Route of administration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RouteOfAdministration {
    #[default]
    Oral,
    Injection,
    Topical,
    Inhalation,
    Rectal,
    Transdermal,
    Nasal,
    Ophthalmic,
    Otic,
    Vaginal,
    Other,
}

/// Frequency of administration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Frequency {
    #[default]
    OnceDaily,
    TwiceDaily,
    ThreeTimesDaily,
    FourTimesDaily,
    EveryOtherDay,
    Weekly,
    AsNeeded,
    Other,
}

/// Duration unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DurationUnit {
    #[default]
    Days,
    Weeks,
    Months,
    Years,
}

/// Duration specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Duration {
    pub value: u32,
    #[serde(default)]
    pub unit: DurationUnit,
}

// ============================================================================
// Product Information
// ============================================================================

/// Product information (drugs/biologics)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProductInformation {
    pub name: String,
    pub strength: Option<String>,
    pub manufacturer: Option<String>,
    /// National Drug Code
    pub ndc_number: Option<String>,
    pub lot_number: Option<String>,
    pub expiration_date: Option<FdaDate>,
    #[serde(default)]
    pub product_type: Vec<ProductType>,
}

/// Medication/Product dosing information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DosingInformation {
    /// e.g., "500 mg"
    pub dose: Option<String>,
    pub frequency: Option<Frequency>,
    pub other_frequency: Option<String>,
    pub route: Option<RouteOfAdministration>,
    pub other_route: Option<String>,
}

/// Therapy dates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TherapyDates {
    pub started: Option<FdaDate>,
    pub stopped: Option<FdaDate>,
    pub dose_reduced: Option<FdaDate>,
    pub duration: Option<Duration>,
    pub ongoing: Option<bool>,
}

// ============================================================================
// Device Information
// ============================================================================

/// Medical device manufacturer info
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeviceManufacturer {
    pub name: String,
    pub city: Option<String>,
    pub state: Option<String>,
}

/// Medical device information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MedicalDeviceInfo {
    pub brand_name: Option<String>,
    pub common_device_name: Option<String>,
    /// 3-letter FDA classification
    pub procode: Option<String>,
    pub manufacturer: Option<DeviceManufacturer>,
    pub model_number: Option<String>,
    pub lot_number: Option<String>,
    pub catalog_number: Option<String>,
    pub expiration_date: Option<FdaDate>,
    pub serial_number: Option<String>,
    /// Unique Device Identifier
    pub udi_number: Option<String>,
}

/// Device operator type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceOperator {
    #[default]
    HealthProfessional,
    PatientConsumer,
    Other,
}

/// Implant information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ImplantInfo {
    pub implanted: Option<bool>,
    pub implant_date: Option<FdaDate>,
    pub explant_date: Option<FdaDate>,
}

// ============================================================================
// Contact & Reporter
// ============================================================================

/// Contact information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContactInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub fax: Option<String>,
}

/// Reporter occupation/profession
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Occupation {
    Physician,
    Pharmacist,
    Nurse,
    Dentist,
    OtherHealthProfessional,
    Consumer,
    Patient,
    Attorney,
    #[default]
    Other,
}

/// Also reported to (for reporters)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AlsoReportedTo {
    pub manufacturer: Option<bool>,
    pub user_facility: Option<bool>,
    pub distributor: Option<bool>,
    pub packer: Option<bool>,
}

// ============================================================================
// Purchase & Availability
// ============================================================================

/// Product availability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProductAvailability {
    pub product_available: Option<bool>,
    pub returned_to_manufacturer: Option<bool>,
    pub return_date: Option<FdaDate>,
    pub has_photograph: Option<bool>,
}

/// Place and date of purchase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PurchaseInfo {
    pub place_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub purchase_date: Option<FdaDate>,
}

/// Lab test result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabTest {
    pub test_name: String,
    pub low_range: Option<String>,
    pub high_range: Option<String>,
    pub test_date: Option<FdaDate>,
}

// ============================================================================
// Form Metadata
// ============================================================================

/// Form type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormType {
    #[serde(rename = "FDA-3500")]
    Fda3500,
    #[serde(rename = "FDA-3500B")]
    Fda3500B,
}

/// Form status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FormStatus {
    #[default]
    Draft,
    InProgress,
    Completed,
    Submitted,
    Acknowledged,
}

/// Form metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormMetadata {
    pub form_id: String,
    pub form_version: String,
    pub form_type: FormType,
    /// Unix timestamp
    pub created_at: i64,
    pub updated_at: i64,
    /// User ID
    pub created_by: String,
    #[serde(default)]
    pub status: FormStatus,
}

/// Submission information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SubmissionInfo {
    pub submitted_at: Option<i64>,
    pub submitted_to: Option<SubmissionTarget>,
    pub confirmation_number: Option<String>,
    pub submission_method: Option<SubmissionMethod>,
    pub acknowledgment_received: Option<bool>,
    pub acknowledgment_date: Option<FdaDate>,
}

/// Submission target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionTarget {
    Fda,
    Manufacturer,
    Both,
}

/// Submission method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionMethod {
    Online,
    Email,
    Fax,
    Mail,
}

// ============================================================================
// Validation Types
// ============================================================================

/// Validation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// Validation error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

impl ValidationError {
    /// Create an error-level validation issue
    #[must_use]
    pub fn error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// Create a warning-level validation issue
    #[must_use]
    pub fn warning(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: ValidationSeverity::Warning,
        }
    }
}

/// Validation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<ValidationError>,
    #[serde(default)]
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a valid result with no issues
    #[must_use]
    pub const fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if there are any errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Add an error
    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError::error(field, message));
        self.valid = false;
    }

    /// Add a warning
    pub fn add_warning(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(ValidationError::warning(field, message));
    }
}
