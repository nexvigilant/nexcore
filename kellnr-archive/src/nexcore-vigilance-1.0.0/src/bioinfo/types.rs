//! Core domain types for KEGG bioinformatics.
//!
//! This module provides:
//! - Entity type enums for biological entities (genes, drugs, compounds)
//! - ID source enums for cross-database conversions
//! - Entry structs for parsed KEGG data

use serde::{Deserialize, Serialize};

// =============================================================================
// Response Format
// =============================================================================

/// Output format for tool responses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// Human-readable Markdown format.
    #[default]
    Markdown,
    /// Machine-readable JSON format.
    Json,
}

// =============================================================================
// Entity Types
// =============================================================================

/// Entity types for convergence analysis.
///
/// Represents the different biological entity types that can be analyzed
/// for pathway convergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    /// KEGG gene ID (e.g., hsa:5594) or gene symbol.
    Gene,
    /// KEGG compound ID (e.g., C00001).
    Compound,
    /// KEGG drug ID (e.g., D00001).
    Drug,
    /// UniProt accession number.
    Uniprot,
    /// PubChem compound ID.
    Pubchem,
}

impl EntityType {
    /// Returns a human-readable description of the entity type.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Gene => "KEGG gene ID or symbol",
            Self::Compound => "KEGG compound",
            Self::Drug => "KEGG drug",
            Self::Uniprot => "UniProt accession",
            Self::Pubchem => "PubChem CID",
        }
    }
}

// =============================================================================
// ID Source Types
// =============================================================================

/// Supported ID source databases for conversion.
///
/// Used when converting identifiers between different biological databases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdSource {
    /// KEGG database identifiers.
    Kegg,
    /// UniProt protein identifiers.
    Uniprot,
    /// NCBI Gene ID.
    NcbiGeneid,
    /// NCBI Protein ID.
    NcbiProteinid,
    /// PubChem compound identifiers.
    Pubchem,
    /// ChEBI compound identifiers.
    Chebi,
}

impl IdSource {
    /// Returns the KEGG API database code for this source.
    #[must_use]
    pub const fn kegg_db_code(&self) -> &'static str {
        match self {
            Self::Kegg => "hsa",
            Self::Uniprot => "uniprot",
            Self::NcbiGeneid => "ncbi-geneid",
            Self::NcbiProteinid => "ncbi-proteinid",
            Self::Pubchem => "pubchem",
            Self::Chebi => "chebi",
        }
    }
}

// =============================================================================
// Entry Types (parsed KEGG data)
// =============================================================================

/// Parsed KEGG pathway entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathwayEntry {
    /// KEGG pathway ID (e.g., "hsa04110").
    pub pathway_id: String,
    /// Pathway name.
    pub name: String,
    /// Organism code (e.g., "hsa" for human).
    #[serde(default)]
    pub organism: String,
}

impl PathwayEntry {
    /// Creates a new pathway entry.
    #[must_use]
    pub fn new(pathway_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            pathway_id: pathway_id.into(),
            name: name.into(),
            organism: String::new(),
        }
    }

    /// Creates a new pathway entry with organism.
    #[must_use]
    pub fn with_organism(
        pathway_id: impl Into<String>,
        name: impl Into<String>,
        organism: impl Into<String>,
    ) -> Self {
        Self {
            pathway_id: pathway_id.into(),
            name: name.into(),
            organism: organism.into(),
        }
    }

    /// Returns the KEGG pathway URL.
    #[must_use]
    pub fn url(&self) -> String {
        format!("https://www.kegg.jp/pathway/{}", self.pathway_id)
    }
}

/// Parsed KEGG gene entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneEntry {
    /// KEGG gene ID (e.g., "hsa:5594").
    pub kegg_id: String,
    /// Gene symbol (e.g., "MAPK1").
    pub symbol: String,
    /// Full gene name.
    #[serde(default)]
    pub name: String,
}

impl GeneEntry {
    /// Creates a new gene entry.
    #[must_use]
    pub fn new(kegg_id: impl Into<String>, symbol: impl Into<String>) -> Self {
        Self {
            kegg_id: kegg_id.into(),
            symbol: symbol.into(),
            name: String::new(),
        }
    }

    /// Creates a new gene entry with full name.
    #[must_use]
    pub fn with_name(
        kegg_id: impl Into<String>,
        symbol: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            kegg_id: kegg_id.into(),
            symbol: symbol.into(),
            name: name.into(),
        }
    }
}

/// Parsed KEGG compound entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompoundEntry {
    /// KEGG compound ID (e.g., "C00001").
    pub compound_id: String,
    /// Compound name.
    pub name: String,
    /// Molecular formula.
    #[serde(default)]
    pub formula: String,
}

impl CompoundEntry {
    /// Creates a new compound entry.
    #[must_use]
    pub fn new(compound_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            compound_id: compound_id.into(),
            name: name.into(),
            formula: String::new(),
        }
    }

    /// Creates a new compound entry with formula.
    #[must_use]
    pub fn with_formula(
        compound_id: impl Into<String>,
        name: impl Into<String>,
        formula: impl Into<String>,
    ) -> Self {
        Self {
            compound_id: compound_id.into(),
            name: name.into(),
            formula: formula.into(),
        }
    }
}

/// Parsed KEGG drug entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrugEntry {
    /// KEGG drug ID (e.g., "D00139").
    pub drug_id: String,
    /// Drug name.
    pub name: String,
    /// Molecular formula.
    #[serde(default)]
    pub formula: String,
    /// Exact mass.
    #[serde(default)]
    pub mass: String,
}

impl DrugEntry {
    /// Creates a new drug entry.
    #[must_use]
    pub fn new(drug_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            drug_id: drug_id.into(),
            name: name.into(),
            formula: String::new(),
            mass: String::new(),
        }
    }
}

/// Parsed KEGG disease entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiseaseEntry {
    /// KEGG disease ID (e.g., "H00001").
    pub disease_id: String,
    /// Disease name.
    pub name: String,
}

impl DiseaseEntry {
    /// Creates a new disease entry.
    #[must_use]
    pub fn new(disease_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            disease_id: disease_id.into(),
            name: name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_format_default() {
        let format = ResponseFormat::default();
        assert_eq!(format, ResponseFormat::Markdown);
    }

    #[test]
    fn test_entity_type_description() {
        assert_eq!(EntityType::Gene.description(), "KEGG gene ID or symbol");
        assert_eq!(EntityType::Drug.description(), "KEGG drug");
    }

    #[test]
    fn test_id_source_kegg_code() {
        assert_eq!(IdSource::Kegg.kegg_db_code(), "hsa");
        assert_eq!(IdSource::Uniprot.kegg_db_code(), "uniprot");
        assert_eq!(IdSource::Pubchem.kegg_db_code(), "pubchem");
    }

    #[test]
    fn test_pathway_entry_url() {
        let entry = PathwayEntry::new("hsa04110", "Cell cycle");
        assert_eq!(entry.url(), "https://www.kegg.jp/pathway/hsa04110");
    }

    #[test]
    fn test_gene_entry_with_name() {
        let entry = GeneEntry::with_name("hsa:5594", "MAPK1", "mitogen-activated protein kinase 1");
        assert_eq!(entry.kegg_id, "hsa:5594");
        assert_eq!(entry.symbol, "MAPK1");
        assert_eq!(entry.name, "mitogen-activated protein kinase 1");
    }

    #[test]
    fn test_compound_entry_with_formula() {
        let entry = CompoundEntry::with_formula("C00001", "Water", "H2O");
        assert_eq!(entry.compound_id, "C00001");
        assert_eq!(entry.name, "Water");
        assert_eq!(entry.formula, "H2O");
    }

    #[test]
    fn test_serde_entity_type() {
        // Test serialization
        let json = serde_json::to_string(&EntityType::Gene);
        assert!(json.is_ok());
        if let Ok(ref s) = json {
            assert_eq!(s, "\"gene\"");
        }

        // Test deserialization
        let parsed: Result<EntityType, _> = serde_json::from_str("\"drug\"");
        assert!(parsed.is_ok());
        if let Ok(entity) = parsed {
            assert_eq!(entity, EntityType::Drug);
        }
    }

    #[test]
    fn test_serde_id_source() {
        let json = serde_json::to_string(&IdSource::NcbiGeneid);
        assert!(json.is_ok());
        if let Ok(ref s) = json {
            assert_eq!(s, "\"ncbi-geneid\"");
        }
    }
}
