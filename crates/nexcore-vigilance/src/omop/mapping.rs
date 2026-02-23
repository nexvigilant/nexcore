//! FAERS → OMOP CDM Mapping Layer
//!
//! Provides the `OmopMappable` trait and a `FaersToOmopMapper` that translates
//! raw FAERS adverse-event data into OMOP standard concept identifiers.
//!
//! ## Design
//!
//! Mapping is intentionally stub-based: the trait defines the contract and the
//! mapper holds the lookup tables. Production implementations would back these
//! lookups with the full OMOP vocabulary tables loaded from a database.
//!
//! All mapping functions return `Option<i64>` — unmapped values produce `None`
//! rather than panicking, preserving conservation (ToV Axiom 3).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── OmopMappable trait ───────────────────────────────────────────────────────

/// Trait for types that can resolve to an OMOP standard concept identifier.
///
/// Implement this on any domain type (drug name, MedDRA PT code, etc.) that
/// needs to be expressed in the OMOP vocabulary.
///
/// Returns `None` when no mapping is available — callers must handle unmapped
/// values explicitly (no silent data loss).
pub trait OmopMappable {
    /// Resolve this value to an OMOP standard concept_id, or `None` if unknown.
    fn to_omop_concept_id(&self) -> Option<i64>;
}

// ─── FaersToOmopMapper ────────────────────────────────────────────────────────

/// Maps FAERS source data to OMOP standard concept identifiers.
///
/// Holds two lookup tables:
/// - `drug_map`: FAERS drug name → OMOP RxNorm concept_id
/// - `event_map`: FAERS MedDRA PT name → OMOP SNOMED condition concept_id
///
/// Populate these maps from the OMOP vocabulary database at startup. The zero
/// state (empty maps) is valid — all lookups will return `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaersToOmopMapper {
    /// FAERS drug name (case-insensitive key) → RxNorm Ingredient concept_id.
    drug_map: HashMap<String, i64>,
    /// FAERS adverse event MedDRA PT name → SNOMED Condition concept_id.
    event_map: HashMap<String, i64>,
}

impl FaersToOmopMapper {
    /// Create an empty mapper. Populate with `add_drug_mapping` and
    /// `add_event_mapping` or by building from the OMOP vocabulary tables.
    pub fn new() -> Self {
        Self {
            drug_map: HashMap::new(),
            event_map: HashMap::new(),
        }
    }

    /// Create a mapper pre-populated from existing lookup tables.
    pub fn from_maps(drug_map: HashMap<String, i64>, event_map: HashMap<String, i64>) -> Self {
        Self {
            drug_map,
            event_map,
        }
    }

    /// Register a drug name → OMOP concept_id mapping.
    ///
    /// Keys are normalised to lowercase for case-insensitive lookup.
    pub fn add_drug_mapping(&mut self, faers_drug_name: impl Into<String>, concept_id: i64) {
        self.drug_map
            .insert(faers_drug_name.into().to_lowercase(), concept_id);
    }

    /// Register an adverse event name → OMOP concept_id mapping.
    ///
    /// Keys are normalised to lowercase for case-insensitive lookup.
    pub fn add_event_mapping(&mut self, faers_event_name: impl Into<String>, concept_id: i64) {
        self.event_map
            .insert(faers_event_name.into().to_lowercase(), concept_id);
    }

    /// Map a FAERS drug name to an OMOP RxNorm Ingredient concept_id.
    ///
    /// Lookup is case-insensitive. Returns `None` for unmapped names.
    ///
    /// # Example
    /// ```
    /// use nexcore_vigilance::omop::mapping::FaersToOmopMapper;
    ///
    /// let mut mapper = FaersToOmopMapper::new();
    /// mapper.add_drug_mapping("ASPIRIN", 1112807);
    /// assert_eq!(mapper.map_drug("aspirin"), Some(1112807));
    /// assert_eq!(mapper.map_drug("ASPIRIN"), Some(1112807));
    /// assert_eq!(mapper.map_drug("UNKNOWN"), None);
    /// ```
    pub fn map_drug(&self, faers_drug_name: &str) -> Option<i64> {
        self.drug_map.get(&faers_drug_name.to_lowercase()).copied()
    }

    /// Map a FAERS MedDRA PT adverse event name to an OMOP SNOMED concept_id.
    ///
    /// Lookup is case-insensitive. Returns `None` for unmapped names.
    ///
    /// # Example
    /// ```
    /// use nexcore_vigilance::omop::mapping::FaersToOmopMapper;
    ///
    /// let mut mapper = FaersToOmopMapper::new();
    /// mapper.add_event_mapping("Myocardial infarction", 4329847);
    /// assert_eq!(mapper.map_event("myocardial infarction"), Some(4329847));
    /// assert_eq!(mapper.map_event("UNKNOWN EVENT"), None);
    /// ```
    pub fn map_event(&self, faers_event_name: &str) -> Option<i64> {
        self.event_map
            .get(&faers_event_name.to_lowercase())
            .copied()
    }

    /// Total number of registered drug mappings.
    pub fn drug_map_len(&self) -> usize {
        self.drug_map.len()
    }

    /// Total number of registered event mappings.
    pub fn event_map_len(&self) -> usize {
        self.event_map.len()
    }
}

impl Default for FaersToOmopMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Newtype wrappers implementing OmopMappable ───────────────────────────────

/// A FAERS drug name that can be resolved to an OMOP concept.
///
/// Wraps a `(name, mapper)` pair so it satisfies the `OmopMappable` trait.
pub struct FaersDrugName<'a> {
    pub name: &'a str,
    pub mapper: &'a FaersToOmopMapper,
}

impl OmopMappable for FaersDrugName<'_> {
    fn to_omop_concept_id(&self) -> Option<i64> {
        self.mapper.map_drug(self.name)
    }
}

/// A FAERS adverse event name that can be resolved to an OMOP concept.
pub struct FaersEventName<'a> {
    pub name: &'a str,
    pub mapper: &'a FaersToOmopMapper,
}

impl OmopMappable for FaersEventName<'_> {
    fn to_omop_concept_id(&self) -> Option<i64> {
        self.mapper.map_event(self.name)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper_drug_round_trip() {
        let mut mapper = FaersToOmopMapper::new();
        mapper.add_drug_mapping("ASPIRIN", 1_112_807_i64);

        assert_eq!(mapper.map_drug("ASPIRIN"), Some(1_112_807));
        assert_eq!(mapper.map_drug("aspirin"), Some(1_112_807));
        assert_eq!(mapper.map_drug("WARFARIN"), None);
    }

    #[test]
    fn mapper_event_round_trip() {
        let mut mapper = FaersToOmopMapper::new();
        mapper.add_event_mapping("Myocardial infarction", 4_329_847_i64);

        assert_eq!(mapper.map_event("Myocardial infarction"), Some(4_329_847));
        assert_eq!(mapper.map_event("MYOCARDIAL INFARCTION"), Some(4_329_847));
        assert_eq!(mapper.map_event("Headache"), None);
    }

    #[test]
    fn omop_mappable_trait_drug() {
        let mut mapper = FaersToOmopMapper::new();
        mapper.add_drug_mapping("metformin", 1_503_297_i64);

        let drug = FaersDrugName {
            name: "METFORMIN",
            mapper: &mapper,
        };
        assert_eq!(drug.to_omop_concept_id(), Some(1_503_297));
    }

    #[test]
    fn omop_mappable_trait_event() {
        let mut mapper = FaersToOmopMapper::new();
        mapper.add_event_mapping("Nausea", 27_674_i64);

        let event = FaersEventName {
            name: "nausea",
            mapper: &mapper,
        };
        assert_eq!(event.to_omop_concept_id(), Some(27_674));
    }

    #[test]
    fn empty_mapper_returns_none() {
        let mapper = FaersToOmopMapper::default();
        assert_eq!(mapper.map_drug("anything"), None);
        assert_eq!(mapper.map_event("anything"), None);
    }

    #[test]
    fn from_maps_constructor() {
        let mut drugs = std::collections::HashMap::new();
        drugs.insert("lisinopril".to_string(), 1_308_216_i64);
        let mapper = FaersToOmopMapper::from_maps(drugs, std::collections::HashMap::new());
        assert_eq!(mapper.map_drug("LISINOPRIL"), Some(1_308_216));
        assert_eq!(mapper.drug_map_len(), 1);
        assert_eq!(mapper.event_map_len(), 0);
    }
}
