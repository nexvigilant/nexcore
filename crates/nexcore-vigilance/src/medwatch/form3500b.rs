//! FDA Form 3500B - MedWatch Consumer/Patient Voluntary Reporting
//!
//! Form Approved: OMB No. 0910-0291
//! Expires: 09-30-2027
//!
//! Simpler consumer-friendly version of Form 3500 for VOLUNTARY reporting of:
//! - Adverse events
//! - Product problems
//! - Product use/medication errors

use serde::{Deserialize, Serialize};

use super::{
    Age, ConsumerOutcomes, ConsumerProductType, Duration, FdaDate, FormMetadata, LabTest,
    ProblemType, PurchaseInfo, Race, Sex, SubmissionInfo, ValidationResult, Weight,
};

/// Complete FDA Form 3500B (Consumer/Patient)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Form3500B {
    pub metadata: FormMetadata,
    pub section_a: AboutTheProblem,
    pub section_b: ProductAvailabilityConsumer,
    /// Required for drugs/biologics/cosmetics
    pub section_c: Option<AboutTheProducts>,
    /// Required for medical devices
    pub section_d: Option<AboutTheMedicalDevice>,
    pub section_e: AboutThePersonWithProblem,
    pub section_f: AboutTheReporter,
    pub submission: Option<SubmissionInfo>,
}

impl Form3500B {
    /// Check if Section C (products) is required
    #[must_use]
    pub const fn requires_section_c(&self) -> bool {
        // Section C required for drugs, biologics, cosmetics, cannabinoid products
        self.section_d.is_none()
    }

    /// Check if Section D (medical device) is required
    #[must_use]
    pub const fn requires_section_d(&self) -> bool {
        // Section D required for medical devices
        self.section_c.is_none()
    }
}

/// SECTION A: ABOUT THE PROBLEM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AboutTheProblem {
    /// A1: What kind of problem was it? (check all that apply)
    #[serde(default)]
    pub problem_types: Vec<ProblemType>,

    /// A2: Did any of the following happen? (check all that apply)
    #[serde(default)]
    pub outcomes: ConsumerOutcomes,

    /// A3: Date the problem occurred
    pub date_of_problem: Option<FdaDate>,

    /// A4: Tell us what happened, how it happened or why it happened
    /// Include as many details as possible (max 4,000 characters)
    #[serde(default)]
    pub description: String,

    /// A5: Relevant Tests/Laboratory Results
    #[serde(default)]
    pub lab_results: Vec<LabTest>,

    /// A5: Additional Comments (max 2,000 characters)
    pub additional_comments: Option<String>,
}

/// SECTION B: PRODUCT AVAILABILITY (Consumer version)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProductAvailabilityConsumer {
    /// B1: Do you still have the product?
    /// We will contact you directly if we need it
    /// DO NOT send the product to FDA
    pub still_have_product: Option<bool>,

    /// B2: Do you have a picture of the product?
    /// While not required, pictures of all sides help FDA review
    pub has_photograph: Option<bool>,
}

/// SECTION C: ABOUT THE PRODUCTS
/// For drugs, biologics, cosmetics, cannabinoid products
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AboutTheProducts {
    /// C1: Name(s) of the product as it appears on box/bottle/package
    #[serde(default)]
    pub product_names: Vec<String>,

    /// C1: Place and Date of Purchase
    pub purchase: Option<PurchaseInfo>,

    /// C2: Check if therapy/usage is on-going
    pub ongoing_therapy: Option<bool>,

    /// C3: Name(s) of the company that makes (or compounds) the product
    pub manufacturer: Option<String>,

    /// C4: Product Type (check all that apply)
    #[serde(default)]
    pub product_type: Vec<ConsumerProductType>,

    /// C5: Expiration date
    pub expiration_date: Option<FdaDate>,

    /// C6: Lot number
    pub lot_number: Option<String>,

    /// C7: NDC number
    pub ndc_number: Option<String>,

    /// C8: Strength (e.g., 800mg/160mg or 20mg)
    pub strength: Option<String>,

    /// C9: Quantity (e.g., 2 pills, 2 puffs, or 1 teaspoon)
    pub quantity: Option<String>,

    /// C10: Frequency (e.g., twice daily or at bedtime)
    pub frequency: Option<String>,

    /// C11: How was it taken or used?
    /// (e.g., by mouth, injection, or on the skin)
    pub how_used: Option<String>,

    /// C12a-c: Therapy/Usage Dates
    pub therapy_dates: Option<ConsumerTherapyDates>,

    /// C14: Why was the person using the product?
    /// (Such as, what condition was it supposed to treat)
    pub indication: Option<String>,

    /// C15: Did the problem stop after the person reduced dose or stopped using?
    pub problem_stopped_after_stopping: Option<bool>,

    /// C16: Did the problem return if the person started using again?
    pub problem_returned_after_restart: Option<ProblemReturnedStatus>,
}

/// Therapy dates for consumer form
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConsumerTherapyDates {
    pub started: Option<FdaDate>,
    pub stopped: Option<FdaDate>,
    pub duration: Option<Duration>,
}

/// Status for C16: Did the problem return after restart
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProblemReturnedStatus {
    Yes,
    No,
    DidntRestart,
}

/// SECTION D: ABOUT THE MEDICAL DEVICE
/// For medical devices only
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AboutTheMedicalDevice {
    /// D1: Name of medical device
    pub device_name: Option<String>,

    /// D2: Name of the company that makes the medical device
    pub manufacturer: Option<String>,

    /// D3: Model number
    pub model_number: Option<String>,

    /// D4: Catalog number
    pub catalog_number: Option<String>,

    /// D5: Lot number
    pub lot_number: Option<String>,

    /// D6: Serial number
    pub serial_number: Option<String>,

    /// D7: Unique Device Identifier (UDI) number (max 1,000 characters)
    pub udi_number: Option<String>,

    /// D8: Expiration date
    pub expiration_date: Option<FdaDate>,

    /// D9: Was someone operating the medical device when the problem occurred?
    pub operator_info: Option<DeviceOperatorInfo>,

    /// D10: For implanted medical devices ONLY
    pub implant: Option<ConsumerImplantInfo>,
}

/// Device operator info for consumer form
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeviceOperatorInfo {
    pub was_operating: bool,
    pub operator: Option<ConsumerDeviceOperator>,
    /// If "someone else"
    pub operator_description: Option<String>,
}

/// Device operator types for consumer form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsumerDeviceOperator {
    PersonWithProblem,
    HealthProfessional,
    SomeoneElse,
}

/// Implant info for consumer form
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConsumerImplantInfo {
    pub date_implanted: Option<FdaDate>,
    pub date_removed: Option<FdaDate>,
}

/// SECTION E: ABOUT THE PERSON WHO HAD THE PROBLEM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AboutThePersonWithProblem {
    /// E1: Person's Initials
    pub initials: Option<String>,

    /// E2: Sex
    pub sex: Option<Sex>,

    /// E3: Age (Specify unit of time)
    pub age: Option<Age>,

    /// E4: Date of Birth
    pub date_of_birth: Option<FdaDate>,

    /// E5: Weight (Specify lbs or kg)
    pub weight: Option<Weight>,

    /// E6: Race and/or Ethnicity (select all that apply)
    #[serde(default)]
    pub race: Vec<Race>,

    /// E7: List known medical conditions
    /// (Such as diabetes, high blood pressure, cancer, heart disease)
    /// Max 2,000 characters
    pub medical_conditions: Option<String>,

    /// E8: Please list all allergies
    /// (Such as to drugs, foods, pollen or others)
    pub allergies: Option<String>,

    /// E9: List any other important information about the person
    /// (Such as tobacco use, pregnancy, alcohol use, etc.)
    pub other_info: Option<String>,

    /// E10: List all OTC medications and vitamins, minerals, supplements, herbal remedies
    /// Max 2,000 characters
    pub otc_medications: Option<String>,

    /// E11: List all current prescription medications and medical devices being used
    /// Max 2,000 characters
    pub prescription_medications: Option<String>,
}

/// SECTION F: ABOUT THE PERSON FILLING OUT THIS FORM
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AboutTheReporter {
    /// F1: Last name
    pub last_name: Option<String>,

    /// F2: First name
    pub first_name: Option<String>,

    /// F3: Number/Street
    pub street: Option<String>,

    /// F4: City
    pub city: Option<String>,

    /// F4: State/Province
    pub state: Option<String>,

    /// F5: ZIP or Postal code
    pub zip_code: Option<String>,

    /// F6: Country
    pub country: Option<String>,

    /// F7: Telephone number
    pub phone: Option<String>,

    /// F8: Email address
    pub email: Option<String>,

    /// F9: Today's date
    pub todays_date: Option<FdaDate>,

    /// F10: Did you report this problem to the company that makes the product?
    pub reported_to_manufacturer: Option<bool>,

    /// F11: If you do NOT want your identity disclosed to the manufacturer
    pub withhold_identity_from_manufacturer: Option<bool>,
}

/// Form 3500B validation result with form reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Form3500BValidation {
    #[serde(flatten)]
    pub result: ValidationResult,
    pub form: Form3500B,
}
