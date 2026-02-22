//! OHDSI OMOP CDM v5.4 — Vocabulary Table Types
//!
//! Implements the core vocabulary tables: Concept, ConceptRelationship,
//! ConceptAncestor, Vocabulary, and Domain.
//!
//! The vocabulary layer is the backbone of OMOP's semantic standardisation —
//! every clinical concept maps to a standard concept via these tables.
//!
//! Reference: <https://ohdsi.github.io/CommonDataModel/cdm54.html#vocabulary-tables>

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ─── Concept ─────────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 CONCEPT table.
///
/// The fundamental unit of the OMOP vocabulary — every clinical event maps
/// to a concept_id. Standard concepts are the target of ETL mappings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    /// Unique identifier for each concept.
    pub concept_id: i64,
    /// Full name of the concept (max 255 chars).
    pub concept_name: String,
    /// Domain this concept belongs to (e.g., "Drug", "Condition", "Measurement").
    pub domain_id: String,
    /// Vocabulary source (e.g., "RxNorm", "SNOMED", "LOINC").
    pub vocabulary_id: String,
    /// Class within the vocabulary (e.g., "Ingredient", "Clinical Finding").
    pub concept_class_id: String,
    /// Standard concept flag: "S" = standard, "C" = classification, None = non-standard.
    pub standard_concept: Option<String>,
    /// Source code within the vocabulary (e.g., "1234567" for RxNorm).
    pub concept_code: String,
    /// Date the concept became valid.
    pub valid_start_date: NaiveDate,
    /// Date the concept became invalid (9999-12-31 if still valid).
    pub valid_end_date: NaiveDate,
    /// Reason the concept is invalid: "D" = deleted, "U" = upgraded, None = valid.
    pub invalid_reason: Option<String>,
}

// ─── ConceptRelationship ─────────────────────────────────────────────────────

/// OMOP CDM v5.4 CONCEPT_RELATIONSHIP table.
///
/// Directed relationships between pairs of concepts (e.g., "Maps to",
/// "Is a", "Has ingredient"). Used for ETL mapping and hierarchy traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelationship {
    /// FK to Concept — the source concept.
    pub concept_id_1: i64,
    /// FK to Concept — the target concept.
    pub concept_id_2: i64,
    /// Type of relationship (e.g., "Maps to", "Is a", "RxNorm has ing").
    pub relationship_id: String,
    /// Date the relationship became valid.
    pub valid_start_date: NaiveDate,
    /// Date the relationship became invalid (9999-12-31 if still valid).
    pub valid_end_date: NaiveDate,
    /// Reason the relationship is invalid: "D" = deleted, "U" = upgraded, None = valid.
    pub invalid_reason: Option<String>,
}

// ─── ConceptAncestor ─────────────────────────────────────────────────────────

/// OMOP CDM v5.4 CONCEPT_ANCESTOR table.
///
/// Hierarchical relationships between concepts — every ancestor-descendant
/// pair is pre-computed with the min/max levels of separation.
/// Used for cohort definition and signal roll-up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptAncestor {
    /// FK to Concept — the ancestor (higher in hierarchy).
    pub ancestor_concept_id: i64,
    /// FK to Concept — the descendant (lower in hierarchy).
    pub descendant_concept_id: i64,
    /// Minimum path length between ancestor and descendant.
    pub min_levels_of_separation: i32,
    /// Maximum path length between ancestor and descendant.
    pub max_levels_of_separation: i32,
}

// ─── Vocabulary ───────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 VOCABULARY table.
///
/// Metadata about each source vocabulary included in the OMOP standardised
/// vocabulary release (e.g., RxNorm, SNOMED-CT, LOINC, MedDRA).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    /// Short identifier for the vocabulary (e.g., "RxNorm", "SNOMED").
    pub vocabulary_id: String,
    /// Full descriptive name of the vocabulary.
    pub vocabulary_name: String,
    /// Reference URL or citation for the vocabulary, nullable.
    pub vocabulary_reference: Option<String>,
    /// Version string of the vocabulary release, nullable.
    pub vocabulary_version: Option<String>,
    /// FK to Concept that represents this vocabulary.
    pub vocabulary_concept_id: i64,
}

// ─── Domain ───────────────────────────────────────────────────────────────────

/// OMOP CDM v5.4 DOMAIN table.
///
/// Defines the clinical domains used to categorise concepts and route them
/// to the appropriate CDM table (e.g., "Drug" → DRUG_EXPOSURE).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    /// Short identifier for the domain (e.g., "Drug", "Condition", "Measurement").
    pub domain_id: String,
    /// Full descriptive name of the domain.
    pub domain_name: String,
    /// FK to Concept that represents this domain.
    pub domain_concept_id: i64,
}
