//! PV Case Processor — ICSR lifecycle as a processor pipeline.
//!
//! Maps the 4-stage ICSR workflow to the processor framework:
//!
//! ```text
//! intake(∂) → code(μ) → assess(μ+κ) → submit(∂+∝)
//! ```
//!
//! Each stage is a `Bounded<FnProcessor>` — boundary-guarded mapping.
//! The full pipeline is composed via `Pipeline::new(a, b).then(c).then(d)`.

use crate::boundary::PredicateBoundary;
use crate::error::ProcessorError;
use crate::pipeline::{Bounded, OpenBoundary, Pipeline};
use crate::processor::{FnProcessor, Processor};

/// Seriousness classification per ICH E2A.
#[derive(Debug, Clone, PartialEq)]
pub enum Seriousness {
    /// Results in death.
    Death,
    /// Life-threatening.
    LifeThreatening,
    /// Requires hospitalization or prolongs existing hospitalization.
    Hospitalization,
    /// Results in persistent or significant disability/incapacity.
    Disability,
    /// Congenital anomaly/birth defect.
    CongenitalAnomaly,
    /// Other medically important condition.
    MedicallyImportant,
    /// Non-serious.
    NonSerious,
}

/// Minimal ICSR data flowing through the pipeline.
#[derive(Debug, Clone)]
pub struct CaseReport {
    /// Identifiable patient (minimum: age or sex or initials).
    pub patient_id: Option<String>,
    /// Reporter information.
    pub reporter: Option<String>,
    /// Suspect drug name.
    pub drug: String,
    /// Adverse event description (free text).
    pub event: String,
    /// MedDRA preferred term (populated by coding stage).
    pub meddra_pt: Option<String>,
    /// Seriousness classification (populated by assessment stage).
    pub seriousness: Option<Seriousness>,
    /// Causality assessment result (populated by assessment stage).
    pub causality: Option<String>,
    /// Whether the case is valid for submission.
    pub valid_for_submission: bool,
}

impl CaseReport {
    /// Create a new case report from minimum required fields.
    pub fn new(drug: impl Into<String>, event: impl Into<String>) -> Self {
        Self {
            patient_id: None,
            reporter: None,
            drug: drug.into(),
            event: event.into(),
            meddra_pt: None,
            seriousness: None,
            causality: None,
            valid_for_submission: false,
        }
    }

    /// Add patient identifier.
    pub fn with_patient(mut self, id: impl Into<String>) -> Self {
        self.patient_id = Some(id.into());
        self
    }

    /// Add reporter.
    pub fn with_reporter(mut self, reporter: impl Into<String>) -> Self {
        self.reporter = Some(reporter.into());
        self
    }
}

/// Stage 1: INTAKE — validate minimum case criteria (ICH E2A four elements).
///
/// ∂ gate: requires identifiable patient, identifiable reporter,
/// suspect drug, and adverse event. Rejects incomplete reports.
fn intake_processor() -> Bounded<
    FnProcessor<CaseReport, CaseReport, impl Fn(CaseReport) -> Result<CaseReport, ProcessorError>>,
    PredicateBoundary<CaseReport, impl Fn(&CaseReport) -> bool>,
    OpenBoundary,
> {
    let validator = PredicateBoundary::new(
        "ICH E2A minimum criteria: patient + reporter + drug + event",
        |case: &CaseReport| {
            case.patient_id.is_some()
                && case.reporter.is_some()
                && !case.drug.is_empty()
                && !case.event.is_empty()
        },
    );

    let passthrough = FnProcessor::new(
        "intake",
        |case: CaseReport| -> Result<CaseReport, ProcessorError> { Ok(case) },
    );

    Bounded::new(passthrough, validator, OpenBoundary)
}

/// Stage 2: CODE — map free-text event to MedDRA preferred term.
///
/// μ: event description → MedDRA PT (simplified lookup).
fn coding_processor()
-> FnProcessor<CaseReport, CaseReport, impl Fn(CaseReport) -> Result<CaseReport, ProcessorError>> {
    FnProcessor::new(
        "code",
        |mut case: CaseReport| -> Result<CaseReport, ProcessorError> {
            // Simplified MedDRA coding — real implementation would use MedDRA dictionary
            let pt = match case.event.to_lowercase() {
                ref e if e.contains("headache") => "Headache",
                ref e if e.contains("nausea") => "Nausea",
                ref e if e.contains("rash") => "Rash",
                ref e if e.contains("liver") || e.contains("hepato") => "Hepatotoxicity",
                ref e if e.contains("lactic") => "Lactic acidosis",
                ref e if e.contains("death") || e.contains("died") => "Death",
                ref e if e.contains("heart") || e.contains("cardiac") => "Cardiac disorder",
                _ => "Other",
            };
            case.meddra_pt = Some(pt.to_string());
            Ok(case)
        },
    )
}

/// Stage 3: ASSESS — classify seriousness and assess causality.
///
/// μ+κ: coded event → seriousness classification + causality.
fn assessment_processor()
-> FnProcessor<CaseReport, CaseReport, impl Fn(CaseReport) -> Result<CaseReport, ProcessorError>> {
    FnProcessor::new(
        "assess",
        |mut case: CaseReport| -> Result<CaseReport, ProcessorError> {
            // Seriousness classification per ICH E2A
            let seriousness = match case.meddra_pt.as_deref() {
                Some("Death") => Seriousness::Death,
                Some("Hepatotoxicity") => Seriousness::Hospitalization,
                Some("Lactic acidosis") => Seriousness::LifeThreatening,
                Some("Cardiac disorder") => Seriousness::LifeThreatening,
                _ => Seriousness::NonSerious,
            };

            // Simplified causality — real implementation would use Naranjo or WHO-UMC
            let causality = if case.meddra_pt.is_some() {
                "Possible"
            } else {
                "Unassessable"
            };

            case.seriousness = Some(seriousness);
            case.causality = Some(causality.to_string());
            Ok(case)
        },
    )
}

/// Stage 4: SUBMIT — validate completeness and mark for submission.
///
/// ∂+∝: final boundary check, then irreversibly mark as submitted.
fn submission_processor() -> Bounded<
    FnProcessor<CaseReport, CaseReport, impl Fn(CaseReport) -> Result<CaseReport, ProcessorError>>,
    PredicateBoundary<CaseReport, impl Fn(&CaseReport) -> bool>,
    OpenBoundary,
> {
    let completeness_gate = PredicateBoundary::new(
        "submission requires MedDRA PT + seriousness + causality",
        |case: &CaseReport| {
            case.meddra_pt.is_some() && case.seriousness.is_some() && case.causality.is_some()
        },
    );

    let submit = FnProcessor::new(
        "submit",
        |mut case: CaseReport| -> Result<CaseReport, ProcessorError> {
            case.valid_for_submission = true;
            Ok(case)
        },
    );

    Bounded::new(submit, completeness_gate, OpenBoundary)
}

/// Build the full 4-stage ICSR pipeline.
///
/// ```text
/// intake(∂) → code(μ) → assess(μ+κ) → submit(∂+∝)
/// ```
///
/// Returns a `Processor<Input=CaseReport, Output=CaseReport>`.
pub fn icsr_pipeline() -> impl Processor<Input = CaseReport, Output = CaseReport> {
    let intake = intake_processor();
    let code = coding_processor();
    let assess = assessment_processor();
    let submit = submission_processor();

    Pipeline::new(intake, code).then(assess).then(submit)
}

/// Check whether a case requires expedited reporting (serious + unexpected).
///
/// Per ICH E2A: serious cases must be reported within 15 calendar days
/// (7 days for fatal/life-threatening).
pub fn is_expedited(case: &CaseReport) -> bool {
    matches!(
        case.seriousness,
        Some(Seriousness::Death)
            | Some(Seriousness::LifeThreatening)
            | Some(Seriousness::Hospitalization)
            | Some(Seriousness::Disability)
            | Some(Seriousness::CongenitalAnomaly)
            | Some(Seriousness::MedicallyImportant)
    )
}

/// Reporting deadline in calendar days per ICH E2A.
pub fn reporting_deadline_days(case: &CaseReport) -> Option<u32> {
    match case.seriousness.as_ref() {
        Some(Seriousness::Death) | Some(Seriousness::LifeThreatening) => Some(7),
        Some(Seriousness::Hospitalization)
        | Some(Seriousness::Disability)
        | Some(Seriousness::CongenitalAnomaly)
        | Some(Seriousness::MedicallyImportant) => Some(15),
        Some(Seriousness::NonSerious) => None, // periodic reporting only
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipeline_processes_valid_case() {
        let pipeline = icsr_pipeline();
        let case = CaseReport::new("metformin", "severe lactic acidosis")
            .with_patient("PT-001")
            .with_reporter("Dr. Smith");

        let result = pipeline.process(case);
        assert!(result.is_ok());

        let processed = result.ok();
        assert!(processed.is_some());
        let processed = processed.as_ref();
        assert_eq!(
            processed.map(|p| p.meddra_pt.as_deref()),
            Some(Some("Lactic acidosis"))
        );
        assert_eq!(
            processed.map(|p| &p.seriousness),
            Some(&Some(Seriousness::LifeThreatening))
        );
        assert_eq!(processed.map(|p| p.valid_for_submission), Some(true));
    }

    #[test]
    fn pipeline_rejects_incomplete_case() {
        let pipeline = icsr_pipeline();
        // Missing patient and reporter — fails intake ∂
        let case = CaseReport::new("aspirin", "headache");

        let result = pipeline.process(case);
        assert!(result.is_err());
    }

    #[test]
    fn expedited_reporting_for_serious_cases() {
        let mut case = CaseReport::new("drug", "event");
        case.seriousness = Some(Seriousness::Death);
        assert!(is_expedited(&case));
        assert_eq!(reporting_deadline_days(&case), Some(7));

        case.seriousness = Some(Seriousness::Hospitalization);
        assert!(is_expedited(&case));
        assert_eq!(reporting_deadline_days(&case), Some(15));

        case.seriousness = Some(Seriousness::NonSerious);
        assert!(!is_expedited(&case));
        assert_eq!(reporting_deadline_days(&case), None);
    }

    #[test]
    fn coding_maps_event_to_meddra() {
        let coder = coding_processor();
        let case = CaseReport::new("drug", "severe hepatotoxicity");
        let coded = coder.process(case);
        assert!(coded.is_ok());
        assert_eq!(
            coded.ok().and_then(|c| c.meddra_pt),
            Some("Hepatotoxicity".to_string())
        );
    }

    #[test]
    fn batch_processing_mixed_cases() {
        let pipeline = icsr_pipeline();

        let cases = vec![
            CaseReport::new("metformin", "lactic acidosis")
                .with_patient("PT-001")
                .with_reporter("Dr. A"),
            CaseReport::new("aspirin", "headache"), // incomplete — no patient/reporter
            CaseReport::new("warfarin", "death")
                .with_patient("PT-002")
                .with_reporter("Dr. B"),
        ];

        let result = crate::process_batch(&pipeline, cases);
        assert_eq!(result.success_count(), 2);
        assert_eq!(result.failure_count(), 1);
        // Failure at index 1 (the incomplete aspirin case)
        assert_eq!(result.failures[0].0, 1);
    }
}
