//! # nexcore Guardian
//!
//! Healthcare compliance domain types for the Guardian platform.
//!
//! This crate provides core domain types for:
//! - **Alerts**: Security incidents and compliance alerts
//! - **Compliance**: FDA 21 CFR Part 11 electronic records and signatures
//! - **Disaster Recovery**: Healthcare-compliant backup and recovery
//! - **Equipment**: GMP equipment calibration tracking
//! - **FHIR**: HL7 FHIR R4 healthcare interoperability types
//! - **NIST CSF**: Cybersecurity Framework alignment and assessment
//! - **Privacy**: GDPR, CCPA, HIPAA privacy compliance
//! - **Threat Response**: NIST Zero Trust automated threat detection
//! - **Training**: GMP personnel training and competency
//! - **Users**: Healthcare user management
//! - **Vendor Risk**: HIPAA/HITRUST vendor risk assessment
//! - **Vulnerability**: Security vulnerability scanning
//! - **Wizard**: Setup wizard session management
//!
//! ## Example
//!
//! ```
//! use nexcore_vigilance::guardian_domain::{Alert, AlertSeverity, AlertStatus, GuardianResult};
//!
//! fn example() -> GuardianResult<()> {
//!     // Create a security alert
//!     let alert = Alert::new(AlertSeverity::Warning, "Unusual access pattern", "audit-system")?;
//!     assert_eq!(alert.status, AlertStatus::Active);
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]

pub mod alerts;
pub mod compliance;
pub mod disaster_recovery;
pub mod equipment;
pub mod error;
pub mod fhir;
pub mod nist_csf;
pub mod privacy;
pub mod threat;
pub mod training;
pub mod users;
pub mod vendor;
pub mod vulnerability;
pub mod wizard;

// Re-export main types at crate root for convenience
pub use alerts::{Alert, AlertSeverity, AlertStatus, AlertUpdate};
pub use compliance::{
    AuditAction, BreachAssessment, BreachLog, BreachNotification, BreachRiskLevel, BreachStatus,
    ComplianceAuditEntry, ComplianceMetrics, ElectronicSignature, HipaaAuditEntry,
    HipaaComplianceStatus, NotificationMethod, PhiAccessType, RecordType, SignatureMeaning,
    SignatureMethod, SystemValidationRecord,
};
pub use disaster_recovery::{
    BackupConfiguration, BackupMetadata, BackupType, DRComplianceFramework, DRTestResult,
    DataClassification, RecoveryOperation, RecoveryPlan, RecoveryStatus, SystemHealth,
};
pub use equipment::{
    CalibrationRecord, CalibrationSchedule, CalibrationStatus, Equipment, EquipmentStatus,
    EquipmentType,
};
pub use error::{GuardianError, GuardianResult};
pub use fhir::{
    FHIRAllergyIntolerance, FHIRCodeableConcept, FHIRCoding, FHIRCondition, FHIRDiagnosticReport,
    FHIRIdentifier, FHIRMedicationRequest, FHIRMeta, FHIRPractitioner, FHIRProcedure, FHIRQuantity,
    FHIRReference,
};
pub use nist_csf::{
    ControlAssessment, FunctionAssessment, GapAnalysis, ImplementationTier, MaturityLevel,
    NistComplianceFramework, NistFunction, NistRiskAssessment, NistRiskLevel, NistSubcategory,
    OrganizationalProfile as NistOrganizationalProfile,
};
pub use privacy::{
    ConsentRecord, ConsentStatus, DataCategory, DataInventoryItem, DataProcessingRecord,
    DataSubject, DataSubjectRight, PrivacyComplianceReport, PrivacyDataBreach, PrivacyRegulation,
    PrivacyRequest, ProcessingLawfulBasis, RequestStatus,
};
pub use threat::{
    AccountLockout, AccountLockoutStatus, IPBlock, IPBlockStatus, MFAStepUp, ResponseAction,
    SessionTermination, ThreatEvent, ThreatLevel, ThreatType,
};
pub use training::{
    CompetencyAssessment, CompetencyResult, QuizQuestion, TrainingCourse, TrainingCourseType,
    TrainingRecord, TrainingStatus,
};
pub use users::{User, UserRole, UserStatus};
pub use vendor::{
    ControlMapping, HIPAARequirement, HITRUSTDomain, Vendor, VendorAssessment, VendorCriticality,
    VendorRiskSummary, VendorStatus,
};
pub use vulnerability::{
    DependencyInfo, ScanConfiguration, ScanResult, ScanStatus, VulnerabilityDetails,
    VulnerabilityMatch, VulnerabilitySeverity, VulnerabilitySource,
};
pub use wizard::{
    ComplianceLevel, OrganizationProfile, OrganizationSize, OrganizationType, WizardSessionData,
};
