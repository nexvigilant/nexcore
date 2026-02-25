//! OHDSI OMOP CDM v5.4 — Core Clinical Table Types
//!
//! Implements the 10 core clinical tables from the OMOP Common Data Model v5.4.
//! All IDs use `i64` (SQL BIGINT), dates use `chrono::Date`.
//!
//! Reference: <https://ohdsi.github.io/CommonDataModel/cdm54.html>

use nexcore_chrono::{Date, DateTime};
use serde::{Deserialize, Serialize};

// ─── Person ──────────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 PERSON table.
///
/// The Person domain contains records that uniquely identify each patient.
/// A person can have multiple visits, conditions, drug exposures, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    /// Unique identifier for each person record.
    pub person_id: i64,
    /// FK to Concept → Gender domain.
    pub gender_concept_id: i64,
    /// Year of birth (4-digit).
    pub year_of_birth: i32,
    /// Month of birth (1–12), nullable.
    pub month_of_birth: Option<i32>,
    /// Day of birth (1–31), nullable.
    pub day_of_birth: Option<i32>,
    /// Full birth datetime, nullable.
    pub birth_datetime: Option<DateTime>,
    /// FK to Concept → Race domain.
    pub race_concept_id: i64,
    /// FK to Concept → Ethnicity domain.
    pub ethnicity_concept_id: i64,
    /// FK to Location table.
    pub location_id: Option<i64>,
    /// FK to Provider table.
    pub provider_id: Option<i64>,
    /// FK to Care Site table.
    pub care_site_id: Option<i64>,
    /// Source value for person identifier from the native data.
    pub person_source_value: Option<String>,
    /// Source value for gender from native data.
    pub gender_source_value: Option<String>,
    /// FK to Concept for the source gender value.
    pub gender_source_concept_id: Option<i64>,
    /// Source value for race from native data.
    pub race_source_value: Option<String>,
    /// FK to Concept for the source race value.
    pub race_source_concept_id: Option<i64>,
    /// Source value for ethnicity from native data.
    pub ethnicity_source_value: Option<String>,
    /// FK to Concept for the source ethnicity value.
    pub ethnicity_source_concept_id: Option<i64>,
}

// ─── ObservationPeriod ────────────────────────────────────────────────────────

/// OMOP CDM v5.4 OBSERVATION_PERIOD table.
///
/// Spans of time for which a person's clinical data is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationPeriod {
    /// Unique identifier for each observation period.
    pub observation_period_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// Start of the observation period.
    pub observation_period_start_date: Date,
    /// End of the observation period.
    pub observation_period_end_date: Date,
    /// FK to Concept → Type of observation period (e.g., insurance enrollment).
    pub period_type_concept_id: i64,
}

// ─── VisitOccurrence ─────────────────────────────────────────────────────────

/// OMOP CDM v5.4 VISIT_OCCURRENCE table.
///
/// Encounters with the healthcare system. May be inpatient, outpatient, ER, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitOccurrence {
    /// Unique identifier for each visit.
    pub visit_occurrence_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// FK to Concept → Visit domain (e.g., inpatient, outpatient).
    pub visit_concept_id: i64,
    /// Start date of the visit.
    pub visit_start_date: Date,
    /// Start datetime of the visit, nullable.
    pub visit_start_datetime: Option<DateTime>,
    /// End date of the visit.
    pub visit_end_date: Date,
    /// End datetime of the visit, nullable.
    pub visit_end_datetime: Option<DateTime>,
    /// FK to Concept → Type of visit record.
    pub visit_type_concept_id: i64,
    /// FK to Provider.
    pub provider_id: Option<i64>,
    /// FK to Care Site.
    pub care_site_id: Option<i64>,
    /// Source value for visit type from native data.
    pub visit_source_value: Option<String>,
    /// FK to Concept for the source visit value.
    pub visit_source_concept_id: Option<i64>,
    /// FK to Concept → where the patient was admitted from.
    pub admitted_from_concept_id: Option<i64>,
    /// Source value for admitted-from location.
    pub admitted_from_source_value: Option<String>,
    /// FK to Concept → where the patient was discharged to.
    pub discharged_to_concept_id: Option<i64>,
    /// Source value for discharged-to location.
    pub discharged_to_source_value: Option<String>,
    /// FK to previous visit occurrence.
    pub preceding_visit_occurrence_id: Option<i64>,
}

// ─── ConditionOccurrence ─────────────────────────────────────────────────────

/// OMOP CDM v5.4 CONDITION_OCCURRENCE table.
///
/// Records of clinical conditions or diagnoses (ICD, SNOMED-CT, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionOccurrence {
    /// Unique identifier for each condition record.
    pub condition_occurrence_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// FK to Concept → Condition domain (SNOMED-CT standard).
    pub condition_concept_id: i64,
    /// Start date of the condition.
    pub condition_start_date: Date,
    /// Start datetime, nullable.
    pub condition_start_datetime: Option<DateTime>,
    /// End date of the condition, nullable.
    pub condition_end_date: Option<Date>,
    /// End datetime, nullable.
    pub condition_end_datetime: Option<DateTime>,
    /// FK to Concept → how the condition was recorded (EHR diagnosis, etc.).
    pub condition_type_concept_id: i64,
    /// FK to Concept → clinical status of the condition.
    pub condition_status_concept_id: Option<i64>,
    /// Reason the medication was stopped (free text), nullable.
    pub stop_reason: Option<String>,
    /// FK to Provider who recorded the condition.
    pub provider_id: Option<i64>,
    /// FK to Visit Occurrence.
    pub visit_occurrence_id: Option<i64>,
    /// FK to Visit Detail.
    pub visit_detail_id: Option<i64>,
    /// Source value for the condition from native data.
    pub condition_source_value: Option<String>,
    /// FK to Concept for the source condition code.
    pub condition_source_concept_id: Option<i64>,
    /// Source value for condition status.
    pub condition_status_source_value: Option<String>,
}

// ─── DrugExposure ────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 DRUG_EXPOSURE table.
///
/// Records of drug administration or prescription (RxNorm standard concepts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugExposure {
    /// Unique identifier for each drug exposure record.
    pub drug_exposure_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// FK to Concept → Drug domain (RxNorm standard).
    pub drug_concept_id: i64,
    /// Start date of drug exposure.
    pub drug_exposure_start_date: Date,
    /// Start datetime, nullable.
    pub drug_exposure_start_datetime: Option<DateTime>,
    /// End date of drug exposure.
    pub drug_exposure_end_date: Date,
    /// End datetime, nullable.
    pub drug_exposure_end_datetime: Option<DateTime>,
    /// Verbatim end date from the source, nullable.
    pub verbatim_end_date: Option<Date>,
    /// FK to Concept → how the drug record was constructed (prescription, admin, etc.).
    pub drug_type_concept_id: i64,
    /// Reason the drug was stopped, nullable.
    pub stop_reason: Option<String>,
    /// Number of refills after the initial prescription.
    pub refills: Option<i32>,
    /// Numeric quantity of the drug as recorded.
    pub quantity: Option<f64>,
    /// Number of days of drug supply.
    pub days_supply: Option<i32>,
    /// Drug prescription directions (sig text), nullable.
    pub sig: Option<String>,
    /// FK to Concept → route of administration.
    pub route_concept_id: Option<i64>,
    /// Lot number of the drug product, nullable.
    pub lot_number: Option<String>,
    /// FK to Provider who prescribed/administered.
    pub provider_id: Option<i64>,
    /// FK to Visit Occurrence.
    pub visit_occurrence_id: Option<i64>,
    /// FK to Visit Detail.
    pub visit_detail_id: Option<i64>,
    /// Source value for the drug from native data.
    pub drug_source_value: Option<String>,
    /// FK to Concept for the source drug code.
    pub drug_source_concept_id: Option<i64>,
    /// Source value for route of administration.
    pub route_source_value: Option<String>,
    /// Source value for dose unit.
    pub dose_unit_source_value: Option<String>,
}

// ─── ProcedureOccurrence ─────────────────────────────────────────────────────

/// OMOP CDM v5.4 PROCEDURE_OCCURRENCE table.
///
/// Records of procedures performed on or for a person (CPT-4, ICD procedures, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureOccurrence {
    /// Unique identifier for each procedure record.
    pub procedure_occurrence_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// FK to Concept → Procedure domain.
    pub procedure_concept_id: i64,
    /// Date of the procedure.
    pub procedure_date: Date,
    /// Datetime of the procedure, nullable.
    pub procedure_datetime: Option<DateTime>,
    /// End date of the procedure, nullable.
    pub procedure_end_date: Option<Date>,
    /// End datetime of the procedure, nullable.
    pub procedure_end_datetime: Option<DateTime>,
    /// FK to Concept → how the procedure was recorded.
    pub procedure_type_concept_id: i64,
    /// FK to Concept → procedure modifier (e.g., laterality).
    pub modifier_concept_id: Option<i64>,
    /// Quantity of the procedure, nullable.
    pub quantity: Option<i32>,
    /// FK to Provider who performed the procedure.
    pub provider_id: Option<i64>,
    /// FK to Visit Occurrence.
    pub visit_occurrence_id: Option<i64>,
    /// FK to Visit Detail.
    pub visit_detail_id: Option<i64>,
    /// Source value for the procedure from native data.
    pub procedure_source_value: Option<String>,
    /// FK to Concept for the source procedure code.
    pub procedure_source_concept_id: Option<i64>,
    /// Source value for the procedure modifier.
    pub modifier_source_value: Option<String>,
}

// ─── Measurement ─────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 MEASUREMENT table.
///
/// Structured values obtained through systematic clinical assessments (labs, vitals, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Unique identifier for each measurement record.
    pub measurement_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// FK to Concept → Measurement domain (LOINC standard).
    pub measurement_concept_id: i64,
    /// Date of the measurement.
    pub measurement_date: Date,
    /// Datetime of the measurement, nullable.
    pub measurement_datetime: Option<DateTime>,
    /// Time of the measurement as a string (HH:MM:SS), nullable.
    pub measurement_time: Option<String>,
    /// FK to Concept → how the measurement was obtained.
    pub measurement_type_concept_id: i64,
    /// FK to Concept → comparison operator (=, <, >=, etc.).
    pub operator_concept_id: Option<i64>,
    /// Numeric measurement result, nullable.
    pub value_as_number: Option<f64>,
    /// FK to Concept → result expressed as a concept.
    pub value_as_concept_id: Option<i64>,
    /// FK to Concept → unit of the measurement.
    pub unit_concept_id: Option<i64>,
    /// Lower limit of the normal range, nullable.
    pub range_low: Option<f64>,
    /// Upper limit of the normal range, nullable.
    pub range_high: Option<f64>,
    /// FK to Provider who ordered/performed the measurement.
    pub provider_id: Option<i64>,
    /// FK to Visit Occurrence.
    pub visit_occurrence_id: Option<i64>,
    /// FK to Visit Detail.
    pub visit_detail_id: Option<i64>,
    /// Source value for the measurement from native data.
    pub measurement_source_value: Option<String>,
    /// FK to Concept for the source measurement code.
    pub measurement_source_concept_id: Option<i64>,
    /// Source value for the measurement unit.
    pub unit_source_value: Option<String>,
    /// FK to Concept for the source unit.
    pub unit_source_concept_id: Option<i64>,
    /// Source value for the measurement result.
    pub value_source_value: Option<String>,
    /// FK to Concept → field in the event record to which this measurement is linked.
    pub meas_event_field_concept_id: Option<i64>,
    /// FK to the event record to which this measurement is linked.
    pub measurement_event_id: Option<i64>,
}

// ─── Observation ─────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 OBSERVATION table.
///
/// Clinical facts about a person obtained in the context of examination, questioning,
/// or a procedure — not captured by more specific domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier for each observation record.
    pub observation_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// FK to Concept → Observation domain.
    pub observation_concept_id: i64,
    /// Date of the observation.
    pub observation_date: Date,
    /// Datetime of the observation, nullable.
    pub observation_datetime: Option<DateTime>,
    /// FK to Concept → how the observation was recorded.
    pub observation_type_concept_id: i64,
    /// Numeric result of the observation, nullable.
    pub value_as_number: Option<f64>,
    /// String result of the observation, nullable.
    pub value_as_string: Option<String>,
    /// FK to Concept → result expressed as a concept.
    pub value_as_concept_id: Option<i64>,
    /// FK to Concept → qualifier of the observation (e.g., severity).
    pub qualifier_concept_id: Option<i64>,
    /// FK to Concept → unit of the observation.
    pub unit_concept_id: Option<i64>,
    /// FK to Provider who recorded the observation.
    pub provider_id: Option<i64>,
    /// FK to Visit Occurrence.
    pub visit_occurrence_id: Option<i64>,
    /// FK to Visit Detail.
    pub visit_detail_id: Option<i64>,
    /// Source value for the observation from native data.
    pub observation_source_value: Option<String>,
    /// FK to Concept for the source observation code.
    pub observation_source_concept_id: Option<i64>,
    /// Source value for the unit.
    pub unit_source_value: Option<String>,
    /// Source value for the qualifier.
    pub qualifier_source_value: Option<String>,
    /// Source value for the observation result.
    pub value_source_value: Option<String>,
    /// FK to Concept → field in the event record to which this observation is linked.
    pub obs_event_field_concept_id: Option<i64>,
    /// FK to the event record to which this observation is linked.
    pub observation_event_id: Option<i64>,
}

// ─── Death ───────────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 DEATH table.
///
/// Records of the cause and time of death for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Death {
    /// FK to Person (one death record per person).
    pub person_id: i64,
    /// Date of death.
    pub death_date: Date,
    /// Datetime of death, nullable.
    pub death_datetime: Option<DateTime>,
    /// FK to Concept → how the death was recorded.
    pub death_type_concept_id: Option<i64>,
    /// FK to Concept → cause of death (ICD concept).
    pub cause_concept_id: Option<i64>,
    /// Source value for the cause of death from native data.
    pub cause_source_value: Option<String>,
    /// FK to Concept for the source cause-of-death code.
    pub cause_source_concept_id: Option<i64>,
}

// ─── DeviceExposure ──────────────────────────────────────────────────────────

/// OMOP CDM v5.4 DEVICE_EXPOSURE table.
///
/// Records of use of a medical device — implants, catheters, vascular access, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceExposure {
    /// Unique identifier for each device exposure record.
    pub device_exposure_id: i64,
    /// FK to Person.
    pub person_id: i64,
    /// FK to Concept → Device domain.
    pub device_concept_id: i64,
    /// Start date of the device exposure.
    pub device_exposure_start_date: Date,
    /// Start datetime, nullable.
    pub device_exposure_start_datetime: Option<DateTime>,
    /// End date of the device exposure, nullable.
    pub device_exposure_end_date: Option<Date>,
    /// End datetime, nullable.
    pub device_exposure_end_datetime: Option<DateTime>,
    /// FK to Concept → how the device record was constructed.
    pub device_type_concept_id: i64,
    /// UDI (Unique Device Identifier), nullable.
    pub unique_device_id: Option<String>,
    /// Production identifier for the device, nullable.
    pub production_id: Option<String>,
    /// Quantity of devices, nullable.
    pub quantity: Option<i32>,
    /// FK to Provider who prescribed/used the device.
    pub provider_id: Option<i64>,
    /// FK to Visit Occurrence.
    pub visit_occurrence_id: Option<i64>,
    /// FK to Visit Detail.
    pub visit_detail_id: Option<i64>,
    /// Source value for the device from native data.
    pub device_source_value: Option<String>,
    /// FK to Concept for the source device code.
    pub device_source_concept_id: Option<i64>,
    /// FK to Concept → unit associated with the device.
    pub unit_concept_id: Option<i64>,
    /// Source value for the unit.
    pub unit_source_value: Option<String>,
    /// FK to Concept for the source unit.
    pub unit_source_concept_id: Option<i64>,
}
