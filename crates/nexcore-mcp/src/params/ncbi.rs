use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

/// Parameters for NCBI ESearch tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NcbiSearchParams {
    /// Database to search (e.g., 'nucleotide', 'protein', 'gene', 'pubmed', 'clinvar', 'snp', 'taxonomy', 'omim').
    pub db: String,
    /// Entrez text query.
    pub term: String,
    /// Maximum number of IDs to return (default 20).
    pub retmax: Option<u32>,
    /// Start index of the first record to return (default 0).
    pub retstart: Option<u32>,
    /// Date type for filtering (e.g., 'pdat' for publication date, 'edat' for Entrez date).
    pub datetype: Option<String>,
    /// Minimum date for date-range filtering (YYYY/MM/DD format).
    pub mindate: Option<String>,
    /// Maximum date for date-range filtering (YYYY/MM/DD format).
    pub maxdate: Option<String>,
}

/// Parameters for NCBI ESummary tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NcbiSummaryParams {
    /// Database from which to retrieve summaries.
    pub db: String,
    /// Comma-separated list of UIDs for which summaries are requested.
    pub id: String,
}

/// Parameters for NCBI EFetch tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NcbiFetchParams {
    /// Database from which to retrieve records.
    pub db: String,
    /// Comma-separated list of UIDs for which records are requested.
    pub id: String,
    /// Format of the record (e.g., 'fasta', 'gb', 'xml', 'vcf', 'clinvarset'). Default: 'fasta'.
    pub rettype: Option<String>,
    /// Mode of retrieval (e.g., 'text', 'xml', 'json'). Default: 'text'.
    pub retmode: Option<String>,
}

/// Parameters for NCBI ELink tool (cross-database traversal).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NcbiLinkParams {
    /// Source database (e.g., 'gene').
    pub dbfrom: String,
    /// Target database (e.g., 'nucleotide', 'pubmed').
    pub db: String,
    /// Comma-separated list of source UIDs.
    pub id: String,
    /// Link command (default 'neighbor'). Options: 'neighbor', 'neighbor_score', 'acheck', 'llinks'.
    pub cmd: Option<String>,
}

/// Parameters for NCBI search-and-fetch pipeline tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NcbiSearchFetchParams {
    /// Database to search and fetch from (e.g., 'nucleotide', 'protein').
    pub db: String,
    /// Entrez text query.
    pub term: String,
    /// Maximum number of records to return (default 5).
    pub retmax: Option<u32>,
}

/// Parameters for NCBI search-and-summarize pipeline tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NcbiSearchSummarizeParams {
    /// Database to search and summarize (e.g., 'gene', 'pubmed').
    pub db: String,
    /// Entrez text query.
    pub term: String,
    /// Maximum number of records to return (default 10).
    pub retmax: Option<u32>,
    /// Date type for filtering (e.g., 'pdat' for publication date).
    pub datetype: Option<String>,
    /// Minimum date for date-range filtering (YYYY/MM/DD format).
    pub mindate: Option<String>,
    /// Maximum date for date-range filtering (YYYY/MM/DD format).
    pub maxdate: Option<String>,
}
