//! Common response wrapper and enrichment types shared across all openFDA endpoints.

use serde::{Deserialize, Serialize};

/// Generic wrapper for every openFDA JSON response.
///
/// # Example
///
/// ```rust,ignore
/// use nexcore_openfda::types::common::{OpenFdaResponse, ResultsMeta};
/// use nexcore_openfda::types::drug::DrugEvent;
///
/// let resp: OpenFdaResponse<DrugEvent> = client.fetch("/drug/event.json", &params).await?;
/// println!("total: {}", resp.meta.results.total);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFdaResponse<T> {
    /// Top-level metadata (disclaimer, terms, result counts).
    pub meta: OpenFdaMeta,
    /// Result records — empty vec if the endpoint returns no matches.
    #[serde(default = "Vec::new")]
    pub results: Vec<T>,
}

/// Top-level metadata returned with every openFDA response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenFdaMeta {
    /// FDA disclaimer text.
    #[serde(default)]
    pub disclaimer: String,
    /// Terms of use URL.
    #[serde(default)]
    pub terms: String,
    /// License information.
    #[serde(default)]
    pub license: String,
    /// Dataset last-updated timestamp (YYYY-MM-DD).
    #[serde(default)]
    pub last_updated: String,
    /// Pagination and total count.
    #[serde(default)]
    pub results: ResultsMeta,
}

/// Pagination counts nested inside `OpenFdaMeta.results`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultsMeta {
    /// Number of records skipped (pagination offset).
    #[serde(default)]
    pub skip: u32,
    /// Number of records returned in this response.
    #[serde(default)]
    pub limit: u32,
    /// Total matching records available (may exceed `limit`).
    #[serde(default)]
    pub total: u64,
}

/// openFDA annotation block present in many endpoint responses.
///
/// Contains cross-referenced names, codes, and identifiers pulled from
/// FDA databases and linked to the reported drug/device/food.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenFdaEnrichment {
    /// Proprietary (brand) names.
    #[serde(default)]
    pub brand_name: Vec<String>,
    /// Non-proprietary (generic) names.
    #[serde(default)]
    pub generic_name: Vec<String>,
    /// Manufacturer names.
    #[serde(default)]
    pub manufacturer_name: Vec<String>,
    /// Active substance (ingredient) names.
    #[serde(default)]
    pub substance_name: Vec<String>,
    /// Routes of administration.
    #[serde(default)]
    pub route: Vec<String>,
    /// National Drug Code identifiers.
    #[serde(default)]
    pub product_ndc: Vec<String>,
    /// Product type (e.g., "HUMAN PRESCRIPTION DRUG").
    #[serde(default)]
    pub product_type: Vec<String>,
    /// NCI thesaurus unique identifiers.
    #[serde(default)]
    pub nui: Vec<String>,
    /// Structured Product Label identifiers.
    #[serde(default)]
    pub spl_id: Vec<String>,
    /// Structured Product Label set identifiers.
    #[serde(default)]
    pub spl_set_id: Vec<String>,
    /// Package NDC codes.
    #[serde(default)]
    pub package_ndc: Vec<String>,
    /// RxNorm concept unique identifiers.
    #[serde(default)]
    pub rxcui: Vec<String>,
    /// Unique Ingredient Identifiers.
    #[serde(default)]
    pub unii: Vec<String>,
    /// NDA/BLA application numbers.
    #[serde(default)]
    pub application_number: Vec<String>,
    /// Pharm class — established pharmacological class.
    #[serde(default)]
    pub pharm_class_epc: Vec<String>,
    /// Pharm class — mechanism of action.
    #[serde(default)]
    pub pharm_class_moa: Vec<String>,
    /// Pharm class — chemical structure.
    #[serde(default)]
    pub pharm_class_cs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_meta_default() {
        let meta = ResultsMeta::default();
        assert_eq!(meta.total, 0);
        assert_eq!(meta.skip, 0);
        assert_eq!(meta.limit, 0);
    }

    #[test]
    fn openfda_meta_deserialize_partial() {
        let json = r#"{"disclaimer":"test","terms":"","license":"","last_updated":"2024-01-01","results":{"skip":0,"limit":10,"total":100}}"#;
        let meta: OpenFdaMeta = serde_json::from_str(json).expect("deserialize");
        assert_eq!(meta.disclaimer, "test");
        assert_eq!(meta.results.total, 100);
    }

    #[test]
    fn enrichment_default_empty() {
        let e = OpenFdaEnrichment::default();
        assert!(e.brand_name.is_empty());
        assert!(e.generic_name.is_empty());
    }
}
