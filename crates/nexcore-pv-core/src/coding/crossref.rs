//! # Terminology Cross-Reference Module
//!
//! Cross-reference types for mapping between MESH, MedDRA, SNOMED-CT, and ICH.
//!
//! ## Strategy
//!
//! 1. **Primary**: UMLS Metathesaurus CUI mappings (confidence 0.95)
//! 2. **Secondary**: BioOntology API (confidence 0.85)
//! 3. **Tertiary**: Fuzzy matching with Jaro similarity (confidence 0.70)
//!
//! ## Mapping Relationships
//!
//! Based on SKOS and UMLS relationship types:
//! - **Exact**: Terms are interchangeable
//! - **Broader**: Target is more general
//! - **Narrower**: Target is more specific
//! - **Related**: Associated but not hierarchical
//! - **CloseMatch**: Very similar but not identical

use nexcore_proof_of_meaning::element::{Atom, ElementClass};
use nexcore_proof_of_meaning::registry::AtomRegistry;
use nexcore_proof_of_meaning::titration::{self, EquivalenceVerdict, Titrator};
use serde::{Deserialize, Serialize};

/// Terminology system identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminologySystem {
    /// NLM Medical Subject Headings
    Mesh,
    /// Medical Dictionary for Regulatory Activities
    MedDRA,
    /// Systematized Nomenclature of Medicine - Clinical Terms
    Snomed,
    /// International Council for Harmonisation glossary
    Ich,
    /// NCI Thesaurus
    NciThesaurus,
    /// UMLS Metathesaurus
    Umls,
}

impl TerminologySystem {
    /// Get system abbreviation
    #[must_use]
    pub fn abbreviation(&self) -> &'static str {
        match self {
            TerminologySystem::Mesh => "MESH",
            TerminologySystem::MedDRA => "MedDRA",
            TerminologySystem::Snomed => "SNOMED-CT",
            TerminologySystem::Ich => "ICH",
            TerminologySystem::NciThesaurus => "NCIT",
            TerminologySystem::Umls => "UMLS",
        }
    }

    /// Get full system name
    #[must_use]
    pub fn full_name(&self) -> &'static str {
        match self {
            TerminologySystem::Mesh => "Medical Subject Headings",
            TerminologySystem::MedDRA => "Medical Dictionary for Regulatory Activities",
            TerminologySystem::Snomed => "Systematized Nomenclature of Medicine - Clinical Terms",
            TerminologySystem::Ich => "International Council for Harmonisation",
            TerminologySystem::NciThesaurus => "NCI Thesaurus",
            TerminologySystem::Umls => "Unified Medical Language System",
        }
    }

    /// Get base URL for term lookup
    #[must_use]
    pub fn base_url(&self) -> &'static str {
        match self {
            TerminologySystem::Mesh => "https://id.nlm.nih.gov/mesh/",
            TerminologySystem::MedDRA => "https://www.meddra.org/",
            TerminologySystem::Snomed => {
                "https://browser.ihtsdotools.org/?perspective=full&conceptId1="
            }
            TerminologySystem::Ich => "https://www.ich.org/",
            TerminologySystem::NciThesaurus => {
                "https://ncit.nci.nih.gov/ncitbrowser/ConceptReport.jsp?dictionary=NCI_Thesaurus&code="
            }
            TerminologySystem::Umls => "https://uts.nlm.nih.gov/uts/umls/concept/",
        }
    }
}

/// Mapping relationship type (SKOS-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MappingRelationship {
    /// Exact equivalence - terms are interchangeable
    Exact,
    /// Target is more general (broader term)
    Broader,
    /// Target is more specific (narrower term)
    Narrower,
    /// Related but not hierarchical
    Related,
    /// Very similar but not identical
    CloseMatch,
}

impl MappingRelationship {
    /// Get relationship confidence modifier (hardcoded constants).
    ///
    /// # Deprecated
    ///
    /// Use [`titrate_confidence`] for POM titration-based scoring instead.
    /// Hardcoded constants do not account for semantic similarity between
    /// source and target terms.
    #[deprecated(
        since = "1.1.0",
        note = "Use titrate_confidence() for POM titration-based equivalence scoring"
    )]
    #[must_use]
    pub fn confidence_modifier(&self) -> f64 {
        match self {
            MappingRelationship::Exact => 1.0,
            MappingRelationship::CloseMatch => 0.9,
            MappingRelationship::Broader | MappingRelationship::Narrower => 0.8,
            MappingRelationship::Related => 0.7,
        }
    }
}

/// Provenance of cross-reference mapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrossRefProvenance {
    /// From UMLS Metathesaurus (CUI mapping)
    Umls {
        /// Concept Unique Identifier
        cui: String,
    },
    /// From BioOntology API
    BioOntology,
    /// Computed via fuzzy matching
    Computed {
        /// Algorithm used
        algorithm: String,
        /// Match score (0.0-1.0)
        score: f64,
    },
    /// Manual curation
    Manual {
        /// Curator identifier
        curator: String,
    },
    /// Computed via POM titration-based semantic equivalence
    ///
    /// The confidence is derived from titrating both source and target terms
    /// against canonical PV atoms and measuring shared equivalence points.
    PomTitration {
        /// Titration equivalence score (0.0-1.0)
        equivalence_score: f64,
        /// Verdict: Equivalent (>0.90), PartialOverlap (0.60-0.90), Distinct (<0.60)
        verdict: String,
        /// Number of shared canonical atoms between source and target
        shared_atoms: usize,
    },
}

impl CrossRefProvenance {
    /// Get base confidence for this provenance type
    #[must_use]
    pub fn base_confidence(&self) -> f64 {
        match self {
            CrossRefProvenance::Umls { .. } => 0.95,
            CrossRefProvenance::BioOntology => 0.85,
            CrossRefProvenance::Computed { score, .. } => *score * 0.70,
            CrossRefProvenance::Manual { .. } => 0.90,
            CrossRefProvenance::PomTitration {
                equivalence_score, ..
            } => *equivalence_score,
        }
    }
}

/// Create a POM titrator seeded with all standard PV atoms.
///
/// Uses the full POM `AtomRegistry::seed_all()` (88 atoms across 8 element classes)
/// to provide comprehensive titrant coverage for terminology cross-referencing.
#[must_use]
pub fn pom_titrator() -> Titrator {
    let mut registry = AtomRegistry::new();
    registry.seed_all();

    let titrants: Vec<Atom> = ElementClass::all()
        .iter()
        .flat_map(|class| registry.by_class(class).into_iter().cloned())
        .map(|ra| ra.atom)
        .collect();

    Titrator::new(titrants)
}

/// Compute a titration-based confidence score for a cross-reference mapping.
///
/// Replaces [`MappingRelationship::confidence_modifier()`]'s hardcoded constants with
/// POM titration-based semantic equivalence measurement. The `MappingRelationship` is
/// used as a validation hint: if titration disagrees with the pre-assigned relationship,
/// the disagreement is recorded in the returned provenance.
///
/// Returns `(confidence, provenance)` where confidence is the titration equivalence score
/// and provenance captures the full titration metadata.
#[must_use]
pub fn titrate_confidence(
    titrator: &Titrator,
    source_term: &str,
    target_term: &str,
    relationship: MappingRelationship,
) -> (f64, CrossRefProvenance) {
    let proof = titration::prove_equivalence(titrator, source_term, target_term);
    let score = proof.equivalence_score.into_inner();

    let verdict_str = match &proof.verdict {
        EquivalenceVerdict::Equivalent => "Equivalent",
        EquivalenceVerdict::PartialOverlap => "PartialOverlap",
        EquivalenceVerdict::Distinct => "Distinct",
    };

    // Validate titration against pre-assigned relationship.
    // If MappingRelationship::Exact but titration scores < 0.8, the disagreement
    // is captured in the verdict string for downstream audit.
    let validation_note = match relationship {
        MappingRelationship::Exact if score < 0.8 => {
            format!(
                "{} (warning: relationship=Exact but titration={:.2}<0.80)",
                verdict_str, score
            )
        }
        MappingRelationship::CloseMatch if score < 0.6 => {
            format!(
                "{} (warning: relationship=CloseMatch but titration={:.2}<0.60)",
                verdict_str, score
            )
        }
        _ => verdict_str.to_string(),
    };

    let provenance = CrossRefProvenance::PomTitration {
        equivalence_score: score,
        verdict: validation_note,
        shared_atoms: proof.shared_atoms,
    };

    (score, provenance)
}

/// Reference to a term in a terminology system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermReference {
    /// Terminology system
    pub system: TerminologySystem,
    /// Term identifier (UI, code, or ID)
    pub identifier: String,
    /// Preferred term/name
    pub name: String,
    /// Definition if available
    pub definition: Option<String>,
}

impl TermReference {
    /// Create a new term reference
    #[must_use]
    pub fn new(
        system: TerminologySystem,
        identifier: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            system,
            identifier: identifier.into(),
            name: name.into(),
            definition: None,
        }
    }

    /// Get URL to view this term
    #[must_use]
    pub fn url(&self) -> String {
        format!("{}{}", self.system.base_url(), self.identifier)
    }
}

/// A single term mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermMapping {
    /// Target term reference
    pub target: TermReference,
    /// Relationship type
    pub relationship: MappingRelationship,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Provenance information
    pub provenance: CrossRefProvenance,
}

/// Complete cross-reference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminologyCrossRef {
    /// Source term being cross-referenced
    pub source: TermReference,
    /// Mappings to other systems
    pub mappings: Vec<TermMapping>,
    /// Overall confidence (highest mapping)
    pub confidence: f64,
    /// Provenance chain
    pub provenance: CrossRefProvenance,
    /// Timestamp of cross-reference
    pub timestamp: String,
}

impl TerminologyCrossRef {
    /// Create a new cross-reference result
    #[must_use]
    pub fn new(source: TermReference, provenance: CrossRefProvenance) -> Self {
        Self {
            source,
            mappings: Vec::new(),
            confidence: 0.0,
            provenance,
            timestamp: nexcore_chrono::DateTime::now().to_rfc3339(),
        }
    }

    /// Add a mapping and update overall confidence
    pub fn add_mapping(&mut self, mapping: TermMapping) {
        if mapping.confidence > self.confidence {
            self.confidence = mapping.confidence;
        }
        self.mappings.push(mapping);
    }

    /// Add a mapping with POM titration-based confidence scoring.
    ///
    /// Uses [`titrate_confidence`] to compute confidence from semantic
    /// equivalence rather than hardcoded constants. The `relationship` hint
    /// is validated against the titration result.
    pub fn add_titrated_mapping(
        &mut self,
        titrator: &Titrator,
        target: TermReference,
        relationship: MappingRelationship,
    ) {
        let (confidence, provenance) =
            titrate_confidence(titrator, &self.source.name, &target.name, relationship);

        let mapping = TermMapping {
            target,
            relationship,
            confidence,
            provenance,
        };

        if mapping.confidence > self.confidence {
            self.confidence = mapping.confidence;
        }
        self.mappings.push(mapping);
    }

    /// Get mappings for a specific target system
    #[must_use]
    pub fn mappings_for(&self, system: TerminologySystem) -> Vec<&TermMapping> {
        self.mappings
            .iter()
            .filter(|m| m.target.system == system)
            .collect()
    }

    /// Get best mapping for a target system
    #[must_use]
    pub fn best_mapping_for(&self, system: TerminologySystem) -> Option<&TermMapping> {
        self.mappings_for(system).into_iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Check if any exact mapping exists
    #[must_use]
    pub fn has_exact_mapping(&self) -> bool {
        self.mappings
            .iter()
            .any(|m| m.relationship == MappingRelationship::Exact)
    }
}

/// Consistency issue detected between terminologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    /// Terms involved in the inconsistency
    pub terms: Vec<TermReference>,
    /// Type of inconsistency
    pub issue_type: ConsistencyIssueType,
    /// Description of the issue
    pub description: String,
    /// Severity (0.0-1.0)
    pub severity: f64,
}

/// Type of consistency issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyIssueType {
    /// Definitions differ significantly
    DefinitionConflict,
    /// Hierarchy placement doesn't match
    HierarchyMismatch,
    /// Scope/context differs
    ScopeDifference,
    /// One term is more specific than equivalent
    GranularityMismatch,
    /// Missing expected mapping
    MissingMapping,
}

impl ConsistencyIssueType {
    /// Get base severity for this issue type
    #[must_use]
    pub fn base_severity(&self) -> f64 {
        match self {
            ConsistencyIssueType::DefinitionConflict => 0.8,
            ConsistencyIssueType::HierarchyMismatch => 0.6,
            ConsistencyIssueType::ScopeDifference => 0.5,
            ConsistencyIssueType::GranularityMismatch => 0.4,
            ConsistencyIssueType::MissingMapping => 0.3,
        }
    }
}

/// Result of consistency check across terminologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    /// Terms checked
    pub terms: Vec<String>,
    /// Corpora/systems checked
    pub corpora: Vec<TerminologySystem>,
    /// Issues found
    pub issues: Vec<ConsistencyIssue>,
    /// Overall consistency score (1.0 = fully consistent)
    pub consistency_score: f64,
}

impl ConsistencyCheckResult {
    /// Create a new result
    #[must_use]
    pub fn new(terms: Vec<String>, corpora: Vec<TerminologySystem>) -> Self {
        Self {
            terms,
            corpora,
            issues: Vec::new(),
            consistency_score: 1.0,
        }
    }

    /// Add an issue and update score
    pub fn add_issue(&mut self, issue: ConsistencyIssue) {
        self.consistency_score -= issue.severity * 0.2; // 5 severe issues = 0% consistency
        self.consistency_score = self.consistency_score.max(0.0);
        self.issues.push(issue);
    }

    /// Check if result is consistent (no major issues)
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.consistency_score >= 0.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminology_system() {
        assert_eq!(TerminologySystem::Mesh.abbreviation(), "MESH");
        assert!(TerminologySystem::Mesh.base_url().contains("nlm.nih.gov"));
    }

    #[test]
    #[allow(deprecated, reason = "Validating backward-compat of deprecated method")]
    fn test_mapping_relationship_confidence() {
        assert_eq!(MappingRelationship::Exact.confidence_modifier(), 1.0);
        assert!(MappingRelationship::Broader.confidence_modifier() < 1.0);
    }

    #[test]
    fn test_term_reference_url() {
        let term = TermReference::new(TerminologySystem::Mesh, "D001241", "Aspirin");
        assert!(term.url().contains("D001241"));
    }

    #[test]
    fn test_crossref_add_mapping() {
        let source = TermReference::new(TerminologySystem::Mesh, "D001241", "Aspirin");
        let mut crossref = TerminologyCrossRef::new(source, CrossRefProvenance::BioOntology);

        let target = TermReference::new(TerminologySystem::Snomed, "387458008", "Aspirin");
        let mapping = TermMapping {
            target,
            relationship: MappingRelationship::Exact,
            confidence: 0.95,
            provenance: CrossRefProvenance::Umls {
                cui: "C0004057".into(),
            },
        };

        crossref.add_mapping(mapping);
        assert_eq!(crossref.mappings.len(), 1);
        assert!((crossref.confidence - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_consistency_scoring() {
        let mut result = ConsistencyCheckResult::new(
            vec!["adverse event".into()],
            vec![TerminologySystem::Mesh, TerminologySystem::Ich],
        );

        assert_eq!(result.consistency_score, 1.0);

        result.add_issue(ConsistencyIssue {
            terms: vec![],
            issue_type: ConsistencyIssueType::ScopeDifference,
            description: "ICH scope is regulatory-focused".into(),
            severity: 0.5,
        });

        assert!(result.consistency_score < 1.0);
        assert!(result.is_consistent()); // Still above 0.8
    }

    #[test]
    fn test_pom_titrator_construction() {
        let titrator = pom_titrator();
        // Should have titrants from all 8 element classes (88 atoms total)
        assert!(!titrator.titrants.is_empty());
    }

    #[test]
    fn test_titrate_confidence_identical_terms() {
        let titrator = pom_titrator();
        let (score, provenance) = titrate_confidence(
            &titrator,
            "cardiac adverse event",
            "cardiac adverse event",
            MappingRelationship::Exact,
        );
        // Identical terms should titrate with high equivalence
        assert!(score > 0.5);
        assert!(matches!(
            provenance,
            CrossRefProvenance::PomTitration { .. }
        ));
    }

    #[test]
    fn test_titrate_confidence_distinct_terms() {
        let titrator = pom_titrator();
        let (score, _provenance) = titrate_confidence(
            &titrator,
            "cardiac disorder",
            "hepatic disorder",
            MappingRelationship::Related,
        );
        // Different organ systems — score should be lower than identical
        let (identical_score, _) = titrate_confidence(
            &titrator,
            "cardiac disorder",
            "cardiac disorder",
            MappingRelationship::Exact,
        );
        assert!(score <= identical_score);
    }

    #[test]
    fn test_titrate_confidence_validation_warning() {
        let titrator = pom_titrator();
        let (score, provenance) = titrate_confidence(
            &titrator,
            "cardiac disorder",
            "hepatic disorder",
            MappingRelationship::Exact, // Mismatch: claiming Exact for different organs
        );
        if score < 0.8 {
            // Should contain a validation warning
            if let CrossRefProvenance::PomTitration { verdict, .. } = &provenance {
                assert!(
                    verdict.contains("warning"),
                    "Expected validation warning for Exact mismatch"
                );
            }
        }
    }

    #[test]
    fn test_pom_provenance_base_confidence() {
        let provenance = CrossRefProvenance::PomTitration {
            equivalence_score: 0.85,
            verdict: "PartialOverlap".into(),
            shared_atoms: 2,
        };
        assert!((provenance.base_confidence() - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_add_titrated_mapping() {
        let titrator = pom_titrator();
        let source =
            TermReference::new(TerminologySystem::Mesh, "D002318", "cardiac adverse event");
        let mut crossref = TerminologyCrossRef::new(source, CrossRefProvenance::BioOntology);

        let target = TermReference::new(
            TerminologySystem::Snomed,
            "269814003",
            "cardiac adverse event",
        );
        crossref.add_titrated_mapping(&titrator, target, MappingRelationship::Exact);

        assert_eq!(crossref.mappings.len(), 1);
        assert!(matches!(
            crossref.mappings[0].provenance,
            CrossRefProvenance::PomTitration { .. }
        ));
        // Identical PV terms should produce non-zero confidence via titration
        assert!(crossref.confidence > 0.0);
    }
}
