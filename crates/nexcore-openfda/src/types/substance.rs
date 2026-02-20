//! Typed structs for the openFDA substance endpoint.
//!
//! Covers:
//! - `/other/substance.json` — FDA substance registry (UNII)

use serde::{Deserialize, Serialize};

// =============================================================================
// Substance (/other/substance.json)
// =============================================================================

/// A substance record from the FDA Substance Registration System.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Substance {
    /// Preferred substance name.
    #[serde(default)]
    pub substance_name: String,
    /// FDA Unique Ingredient Identifier (UNII).
    #[serde(default)]
    pub unii: String,
    /// Molecular formula (e.g., "C9H8O4").
    #[serde(default)]
    pub molecular_formula: String,
    /// Average molecular weight.
    #[serde(default)]
    pub molecular_weight: String,
    /// Chemical Abstracts Service registry number.
    #[serde(default)]
    pub cas: String,
    /// NDA/BLA approval identifier.
    #[serde(default)]
    pub approval_id: String,
    /// Substance codes from other nomenclature systems.
    #[serde(default)]
    pub codes: Vec<SubstanceCode>,
    /// Related names and synonyms.
    #[serde(default)]
    pub names: Vec<SubstanceName>,
    /// Substance class/type (e.g., "Chemical", "Polymer", "Protein", "Structurally Diverse").
    #[serde(default)]
    pub substance_class: String,
    /// Structure representation (if available).
    #[serde(default)]
    pub structure: Option<SubstanceStructure>,
    /// InChIKey for chemical structure lookup.
    #[serde(default)]
    pub inchikey: Option<String>,
    /// SMILES notation.
    #[serde(default)]
    pub smiles: Option<String>,
}

/// A code/identifier from an external nomenclature system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubstanceCode {
    /// Code value.
    #[serde(default)]
    pub code: String,
    /// Code system (e.g., "CAS", "INN", "USAN", "WHO-INN", "FDA SPL Indexing Data").
    #[serde(default)]
    pub code_system: String,
    /// Code type (e.g., "PRIMARY", "SECONDARY").
    #[serde(default)]
    pub code_type: Option<String>,
    /// Comments.
    #[serde(default)]
    pub comments: Option<String>,
}

/// A name/synonym for a substance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubstanceName {
    /// The name string.
    #[serde(default)]
    pub name: String,
    /// Name type (e.g., "cn" = common name, "bn" = brand name, "sys" = systematic).
    #[serde(rename = "type", default)]
    pub name_type: String,
    /// Preferred flag.
    #[serde(default)]
    pub preferred: Option<bool>,
    /// Language (ISO 639-1 code).
    #[serde(default)]
    pub language: Option<String>,
}

/// Chemical structure representation for a substance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubstanceStructure {
    /// Number of stereocenters.
    #[serde(default)]
    pub stereocenters: Option<u32>,
    /// Defined stereocenters.
    #[serde(default)]
    pub defined_stereocenters: Option<u32>,
    /// Molecular weight.
    #[serde(default)]
    pub molecular_weight: Option<f64>,
    /// InChI string.
    #[serde(default)]
    pub inchi: Option<String>,
    /// SMILES notation.
    #[serde(default)]
    pub smiles: Option<String>,
    /// Stereochemistry description.
    #[serde(default)]
    pub stereochemistry: Option<String>,
    /// Optical activity.
    #[serde(default)]
    pub optical_activity: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substance_deserialize_minimal() {
        let json = r#"{
            "substance_name": "ASPIRIN",
            "unii": "R16CO5Y76E",
            "molecular_formula": "C9H8O4",
            "cas": "50-78-2"
        }"#;
        let sub: Substance =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(sub.substance_name, "ASPIRIN");
        assert_eq!(sub.unii, "R16CO5Y76E");
        assert_eq!(sub.cas, "50-78-2");
    }

    #[test]
    fn substance_defaults_empty() {
        let json = r#"{"substance_name": "WATER"}"#;
        let sub: Substance =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert!(sub.codes.is_empty());
        assert!(sub.names.is_empty());
        assert!(sub.structure.is_none());
    }

    #[test]
    fn substance_code_default() {
        let code = SubstanceCode::default();
        assert!(code.code.is_empty());
        assert!(code.code_system.is_empty());
    }

    #[test]
    fn substance_name_deserialize() {
        let json = r#"{"name": "acetylsalicylic acid", "type": "cn"}"#;
        let name: SubstanceName =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(name.name, "acetylsalicylic acid");
        assert_eq!(name.name_type, "cn");
    }
}
