//! NCBI E-utilities request/response types.
//!
//! Serde structs for ESearch, ESummary, and ELink JSON responses.
//! EFetch returns raw FASTA/XML text, parsed downstream.

use serde::Deserialize;
use std::collections::HashMap;

/// NCBI database identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Database {
    Nucleotide,
    Protein,
    Gene,
    PubMed,
    ClinVar,
    Snp,
    Taxonomy,
    Omim,
}

impl Database {
    /// Returns the NCBI database string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nucleotide => "nucleotide",
            Self::Protein => "protein",
            Self::Gene => "gene",
            Self::PubMed => "pubmed",
            Self::ClinVar => "clinvar",
            Self::Snp => "snp",
            Self::Taxonomy => "taxonomy",
            Self::Omim => "omim",
        }
    }

    /// Parse a database name from string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nucleotide" | "nuc" | "nuccore" => Some(Self::Nucleotide),
            "protein" | "prot" => Some(Self::Protein),
            "gene" => Some(Self::Gene),
            "pubmed" => Some(Self::PubMed),
            "clinvar" => Some(Self::ClinVar),
            "snp" | "dbsnp" => Some(Self::Snp),
            "taxonomy" | "tax" => Some(Self::Taxonomy),
            "omim" => Some(Self::Omim),
            _ => None,
        }
    }
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// ESearch
// ---------------------------------------------------------------------------

/// Parameters for an ESearch request.
#[derive(Debug, Clone)]
pub struct ESearchParams {
    /// NCBI database to search.
    pub db: Database,
    /// Search query term.
    pub term: String,
    /// Maximum number of UIDs to return (default: 20).
    pub retmax: u32,
    /// Offset into result set (default: 0).
    pub retstart: u32,
    /// Minimum date for date-range filtering (YYYY/MM/DD).
    pub mindate: Option<String>,
    /// Maximum date for date-range filtering (YYYY/MM/DD).
    pub maxdate: Option<String>,
    /// Date field to filter on (e.g., "pdat" for publication date, "edat" for Entrez date).
    pub datetype: Option<String>,
}

impl ESearchParams {
    /// Create search params with defaults (retmax=20, retstart=0).
    pub fn new(db: Database, term: impl Into<String>) -> Self {
        Self {
            db,
            term: term.into(),
            retmax: 20,
            retstart: 0,
            mindate: None,
            maxdate: None,
            datetype: None,
        }
    }

    /// Set maximum results to return.
    pub fn with_retmax(mut self, retmax: u32) -> Self {
        self.retmax = retmax;
        self
    }

    /// Set result offset.
    pub fn with_retstart(mut self, retstart: u32) -> Self {
        self.retstart = retstart;
        self
    }

    /// Set date range filter (YYYY/MM/DD format).
    pub fn with_date_range(
        mut self,
        datetype: impl Into<String>,
        mindate: impl Into<String>,
        maxdate: impl Into<String>,
    ) -> Self {
        self.datetype = Some(datetype.into());
        self.mindate = Some(mindate.into());
        self.maxdate = Some(maxdate.into());
        self
    }
}

// ---------------------------------------------------------------------------
// EFetch
// ---------------------------------------------------------------------------

/// Parameters for an EFetch request.
#[derive(Debug, Clone)]
pub struct EFetchParams {
    /// NCBI database to fetch from.
    pub db: Database,
    /// List of UIDs to fetch.
    pub ids: Vec<String>,
    /// Return type (e.g., "fasta", "gb", "xml", "vcf"). Default: "fasta".
    pub rettype: String,
    /// Return mode (e.g., "text", "xml", "json"). Default: "text".
    pub retmode: String,
}

impl EFetchParams {
    /// Create fetch params with FASTA defaults.
    pub fn new(db: Database, ids: Vec<String>) -> Self {
        Self {
            db,
            ids,
            rettype: "fasta".to_string(),
            retmode: "text".to_string(),
        }
    }

    /// Set return type (e.g., "fasta", "gb", "xml", "vcf", "docsum").
    pub fn with_rettype(mut self, rettype: impl Into<String>) -> Self {
        self.rettype = rettype.into();
        self
    }

    /// Set return mode (e.g., "text", "xml", "json").
    pub fn with_retmode(mut self, retmode: impl Into<String>) -> Self {
        self.retmode = retmode.into();
        self
    }

    /// Convenience: ClinVar XML fetch.
    pub fn clinvar_xml(ids: Vec<String>) -> Self {
        Self {
            db: Database::ClinVar,
            ids,
            rettype: "clinvarset".to_string(),
            retmode: "xml".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// ELink
// ---------------------------------------------------------------------------

/// Parameters for an ELink request (cross-database traversal).
#[derive(Debug, Clone)]
pub struct ELinkParams {
    /// Source database.
    pub dbfrom: Database,
    /// Target database.
    pub db: Database,
    /// Source UIDs to find links for.
    pub ids: Vec<String>,
    /// Link command (default: "neighbor").
    pub cmd: String,
}

impl ELinkParams {
    pub fn new(dbfrom: Database, db: Database, ids: Vec<String>) -> Self {
        Self {
            dbfrom,
            db,
            ids,
            cmd: "neighbor".to_string(),
        }
    }

    /// Set link command (e.g., "neighbor", "neighbor_score", "acheck", "llinks").
    pub fn with_cmd(mut self, cmd: impl Into<String>) -> Self {
        self.cmd = cmd.into();
        self
    }
}

// ---------------------------------------------------------------------------
// ESummary
// ---------------------------------------------------------------------------

/// Parameters for an ESummary request (document metadata).
#[derive(Debug, Clone)]
pub struct ESummaryParams {
    pub db: Database,
    pub ids: Vec<String>,
}

impl ESummaryParams {
    pub fn new(db: Database, ids: Vec<String>) -> Self {
        Self { db, ids }
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Top-level ESearch JSON response wrapper.
#[derive(Debug, Deserialize)]
pub struct ESearchResponse {
    pub esearchresult: ESearchResult,
}

/// The actual ESearch result payload.
#[derive(Debug, Deserialize)]
pub struct ESearchResult {
    /// Total number of matching records.
    pub count: String,
    /// Number of UIDs returned in this response.
    pub retmax: String,
    /// Offset of the first UID returned.
    pub retstart: String,
    /// List of matching UIDs.
    pub idlist: Vec<String>,
    /// Query translation details.
    #[serde(default)]
    pub translationset: Vec<Translation>,
    /// Full translated query string.
    #[serde(default)]
    pub querytranslation: String,
}

impl ESearchResult {
    /// Parse the count field as a number.
    pub fn total_count(&self) -> u64 {
        self.count.parse().unwrap_or(0)
    }
}

/// A single query translation entry.
#[derive(Debug, Deserialize)]
pub struct Translation {
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
}

// ---------------------------------------------------------------------------
// ELink response
// ---------------------------------------------------------------------------

/// Top-level ELink JSON response.
#[derive(Debug, Deserialize)]
pub struct ELinkResponse {
    pub linksets: Vec<LinkSet>,
}

/// A single link set from ELink.
#[derive(Debug, Deserialize)]
pub struct LinkSet {
    #[serde(default)]
    pub dbfrom: String,
    #[serde(default)]
    pub idlist: Vec<String>,
    #[serde(default)]
    pub linksetdbs: Vec<LinkSetDb>,
}

/// Links to a specific target database.
#[derive(Debug, Deserialize)]
pub struct LinkSetDb {
    #[serde(default)]
    pub dbto: String,
    #[serde(default)]
    pub linkname: String,
    #[serde(default)]
    pub links: Vec<String>,
}

/// Flattened ELink result for easy consumption.
#[derive(Debug, Clone)]
pub struct ELinkResult {
    pub dbfrom: String,
    pub dbto: String,
    pub linkname: String,
    pub source_ids: Vec<String>,
    pub linked_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// ESummary response
// ---------------------------------------------------------------------------

/// Top-level ESummary JSON response.
#[derive(Debug, Deserialize)]
pub struct ESummaryResponse {
    pub result: Option<ESummaryResult>,
}

/// ESummary result (uid list + per-uid document summaries).
#[derive(Debug, Deserialize)]
pub struct ESummaryResult {
    #[serde(default)]
    pub uids: Vec<String>,
    /// Remaining keys are per-UID document summaries (dynamic).
    #[serde(flatten)]
    pub docs: HashMap<String, serde_json::Value>,
}

/// A parsed document summary with common fields.
#[derive(Debug, Clone)]
pub struct DocSummary {
    pub uid: String,
    pub title: String,
    pub extra: HashMap<String, serde_json::Value>,
}

impl ESummaryResult {
    /// Extract document summaries from the raw response.
    pub fn summaries(&self) -> Vec<DocSummary> {
        let mut out = Vec::new();
        for uid in &self.uids {
            if let Some(val) = self.docs.get(uid) {
                let title = val
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let extra = match val.as_object() {
                    Some(map) => map
                        .iter()
                        .filter(|(k, _)| k.as_str() != "title" && k.as_str() != "uid")
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                    None => HashMap::new(),
                };
                out.push(DocSummary {
                    uid: uid.clone(),
                    title,
                    extra,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_as_str() {
        assert_eq!(Database::Nucleotide.as_str(), "nucleotide");
        assert_eq!(Database::Protein.as_str(), "protein");
        assert_eq!(Database::Gene.as_str(), "gene");
        assert_eq!(Database::PubMed.as_str(), "pubmed");
        assert_eq!(Database::ClinVar.as_str(), "clinvar");
        assert_eq!(Database::Snp.as_str(), "snp");
    }

    #[test]
    fn database_from_str_loose() {
        assert_eq!(Database::from_str_loose("nuc"), Some(Database::Nucleotide));
        assert_eq!(Database::from_str_loose("prot"), Some(Database::Protein));
        assert_eq!(Database::from_str_loose("clinvar"), Some(Database::ClinVar));
        assert_eq!(Database::from_str_loose("dbsnp"), Some(Database::Snp));
        assert_eq!(Database::from_str_loose("unknown"), None);
    }

    #[test]
    fn esearch_params_defaults() {
        let p = ESearchParams::new(Database::Nucleotide, "BRCA1");
        assert_eq!(p.retmax, 20);
        assert_eq!(p.retstart, 0);
        assert_eq!(p.term, "BRCA1");
        assert!(p.mindate.is_none());
    }

    #[test]
    fn esearch_params_date_range() {
        let p = ESearchParams::new(Database::PubMed, "cancer").with_date_range(
            "pdat",
            "2024/01/01",
            "2025/12/31",
        );
        assert_eq!(p.datetype, Some("pdat".to_string()));
        assert_eq!(p.mindate, Some("2024/01/01".to_string()));
        assert_eq!(p.maxdate, Some("2025/12/31".to_string()));
    }

    #[test]
    fn esearch_params_builder() {
        let p = ESearchParams::new(Database::Protein, "insulin")
            .with_retmax(50)
            .with_retstart(10);
        assert_eq!(p.retmax, 50);
        assert_eq!(p.retstart, 10);
    }

    #[test]
    fn efetch_params_clinvar_xml() {
        let p = EFetchParams::clinvar_xml(vec!["12345".into()]);
        assert_eq!(p.db, Database::ClinVar);
        assert_eq!(p.rettype, "clinvarset");
        assert_eq!(p.retmode, "xml");
    }

    #[test]
    fn efetch_params_custom_rettype() {
        let p = EFetchParams::new(Database::Nucleotide, vec!["123".into()])
            .with_rettype("gb")
            .with_retmode("text");
        assert_eq!(p.rettype, "gb");
        assert_eq!(p.retmode, "text");
    }

    #[test]
    fn elink_params_defaults() {
        let p = ELinkParams::new(Database::Gene, Database::Nucleotide, vec!["7157".into()]);
        assert_eq!(p.cmd, "neighbor");
        assert_eq!(p.dbfrom, Database::Gene);
        assert_eq!(p.db, Database::Nucleotide);
    }

    #[test]
    fn parse_esearch_response() {
        let json = r#"{
            "esearchresult": {
                "count": "42",
                "retmax": "20",
                "retstart": "0",
                "idlist": ["12345", "67890"],
                "querytranslation": "BRCA1[All Fields]"
            }
        }"#;
        let resp: ESearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.esearchresult.total_count(), 42);
        assert_eq!(resp.esearchresult.idlist.len(), 2);
        assert_eq!(resp.esearchresult.idlist[0], "12345");
    }

    #[test]
    fn parse_esearch_empty_response() {
        let json = r#"{
            "esearchresult": {
                "count": "0",
                "retmax": "20",
                "retstart": "0",
                "idlist": []
            }
        }"#;
        let resp: ESearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.esearchresult.total_count(), 0);
        assert!(resp.esearchresult.idlist.is_empty());
    }

    #[test]
    fn parse_elink_response() {
        let json = r#"{
            "linksets": [{
                "dbfrom": "gene",
                "idlist": ["7157"],
                "linksetdbs": [{
                    "dbto": "nucleotide",
                    "linkname": "gene_nuccore_refseqrna",
                    "links": ["123", "456"]
                }]
            }]
        }"#;
        let resp: ELinkResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.linksets.len(), 1);
        assert_eq!(resp.linksets[0].linksetdbs[0].links.len(), 2);
        assert_eq!(
            resp.linksets[0].linksetdbs[0].linkname,
            "gene_nuccore_refseqrna"
        );
        assert_eq!(resp.linksets[0].linksetdbs[0].links[0], "123");
    }

    #[test]
    fn parse_esummary_response() {
        let json = r#"{
            "result": {
                "uids": ["7157"],
                "7157": {
                    "uid": "7157",
                    "title": "tumor protein p53",
                    "organism": {"scientificname": "Homo sapiens"}
                }
            }
        }"#;
        let resp: ESummaryResponse = serde_json::from_str(json).unwrap();
        let result = resp.result.unwrap();
        let summaries = result.summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "tumor protein p53");
        assert_eq!(summaries[0].uid, "7157");
    }
}
