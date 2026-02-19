//! NCBI E-utilities HTTP client.
//!
//! Provides `NcbiClient` for ESearch, EFetch, ELink, and ESummary against
//! NCBI's Entrez system.
//!
//! ## Primitives
//!
//! μ(Mapping): query → UIDs → FASTA → BioRecord
//! ∂(Boundary): timeouts, rate limits, error handling
//! σ(Sequence): search-then-fetch pipeline

use std::io::Cursor;
use std::time::Duration;

use tracing::debug;

use crate::bio::{BioParser, BioRecord};

use super::error::{NcbiError, Result};
use super::types::{
    DocSummary, EFetchParams, ELinkParams, ELinkResponse, ELinkResult, ESearchParams,
    ESearchResponse, ESearchResult, ESummaryParams, ESummaryResponse,
};

const ESEARCH_URL: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi";
const EFETCH_URL: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi";
const ELINK_URL: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/elink.fcgi";
const ESUMMARY_URL: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// NCBI E-utilities client.
///
/// Supports optional API key (3 req/sec without, 10 req/sec with).
/// NCBI requests that automated tools identify themselves via `tool` and `email`.
#[derive(Debug)]
pub struct NcbiClient {
    http: reqwest::Client,
    api_key: Option<String>,
    email: Option<String>,
    tool_name: String,
}

impl NcbiClient {
    /// Create a client without an API key (limited to 3 requests/sec).
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(NcbiError::Http)?;

        Ok(Self {
            http,
            api_key: None,
            email: None,
            tool_name: "nexcore-dna".to_string(),
        })
    }

    /// Create a client with an NCBI API key (up to 10 requests/sec).
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        let key = api_key.into();
        if key.is_empty() {
            return Err(NcbiError::InvalidParams("API key cannot be empty".into()));
        }

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(NcbiError::Http)?;

        Ok(Self {
            http,
            api_key: Some(key),
            email: None,
            tool_name: "nexcore-dna".to_string(),
        })
    }

    /// Create a client from the `NCBI_API_KEY` environment variable (if set).
    pub fn from_env() -> Result<Self> {
        match std::env::var("NCBI_API_KEY") {
            Ok(key) if !key.is_empty() => Self::with_api_key(key),
            _ => Self::new(),
        }
    }

    /// Set the contact email for NCBI identification.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the tool name for NCBI identification.
    pub fn with_tool_name(mut self, name: impl Into<String>) -> Self {
        self.tool_name = name.into();
        self
    }

    /// Append common query parameters (api_key, tool, email).
    fn append_common_params(&self, params: &mut Vec<(&str, String)>) {
        if let Some(ref key) = self.api_key {
            params.push(("api_key", key.clone()));
        }
        params.push(("tool", self.tool_name.clone()));
        if let Some(ref email) = self.email {
            params.push(("email", email.clone()));
        }
    }

    /// Send GET and handle common HTTP error statuses. Returns response body as text.
    async fn get_text(&self, url: &str, query: &[(&str, String)]) -> Result<String> {
        let response = self
            .http
            .get(url)
            .query(query)
            .send()
            .await
            .map_err(NcbiError::Http)?;

        let status = response.status().as_u16();
        if status == 429 {
            return Err(NcbiError::RateLimited {
                retry_after_secs: 1,
            });
        }

        let body = response.text().await.map_err(NcbiError::Http)?;

        if !(200..300).contains(&status) {
            return Err(NcbiError::Api {
                status,
                message: body,
            });
        }

        Ok(body)
    }

    /// Send GET, handle errors, parse JSON response.
    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        let body = self.get_text(url, query).await?;
        serde_json::from_str(&body).map_err(NcbiError::Json)
    }

    // -----------------------------------------------------------------------
    // ESearch
    // -----------------------------------------------------------------------

    /// Search an NCBI database. Returns matching UIDs and metadata.
    pub async fn esearch(&self, params: &ESearchParams) -> Result<ESearchResult> {
        if params.term.is_empty() {
            return Err(NcbiError::InvalidParams(
                "search term cannot be empty".into(),
            ));
        }

        let mut query: Vec<(&str, String)> = vec![
            ("db", params.db.as_str().to_string()),
            ("term", params.term.clone()),
            ("retmax", params.retmax.to_string()),
            ("retstart", params.retstart.to_string()),
            ("retmode", "json".to_string()),
        ];

        if let Some(ref dt) = params.datetype {
            query.push(("datetype", dt.clone()));
        }
        if let Some(ref min) = params.mindate {
            query.push(("mindate", min.clone()));
        }
        if let Some(ref max) = params.maxdate {
            query.push(("maxdate", max.clone()));
        }

        self.append_common_params(&mut query);

        debug!(db = %params.db, term = %params.term, retmax = params.retmax, "NCBI ESearch");

        let parsed: ESearchResponse = self.get_json(ESEARCH_URL, &query).await?;

        debug!(
            count = %parsed.esearchresult.count,
            returned = parsed.esearchresult.idlist.len(),
            "ESearch complete"
        );

        Ok(parsed.esearchresult)
    }

    // -----------------------------------------------------------------------
    // EFetch
    // -----------------------------------------------------------------------

    /// Fetch FASTA records by UID. Returns parsed `BioRecord`s.
    pub async fn efetch(&self, params: &EFetchParams) -> Result<Vec<BioRecord>> {
        let text = self.efetch_raw(params).await?;
        let records = BioParser::parse_fasta(Cursor::new(text))?;
        Ok(records)
    }

    /// Fetch raw text from EFetch (for non-FASTA rettypes like XML, GenBank, VCF).
    pub async fn efetch_raw(&self, params: &EFetchParams) -> Result<String> {
        if params.ids.is_empty() {
            return Err(NcbiError::InvalidParams("ID list cannot be empty".into()));
        }

        let id_str = params.ids.join(",");

        let mut query: Vec<(&str, String)> = vec![
            ("db", params.db.as_str().to_string()),
            ("id", id_str.clone()),
            ("rettype", params.rettype.clone()),
            ("retmode", params.retmode.clone()),
        ];
        self.append_common_params(&mut query);

        debug!(db = %params.db, ids = %id_str, rettype = %params.rettype, "NCBI EFetch");

        let text = self.get_text(EFETCH_URL, &query).await?;
        debug!(bytes = text.len(), "EFetch received");
        Ok(text)
    }

    // -----------------------------------------------------------------------
    // ELink
    // -----------------------------------------------------------------------

    /// Find linked records across NCBI databases.
    pub async fn elink(&self, params: &ELinkParams) -> Result<Vec<ELinkResult>> {
        if params.ids.is_empty() {
            return Err(NcbiError::InvalidParams("ID list cannot be empty".into()));
        }

        let id_str = params.ids.join(",");

        let mut query: Vec<(&str, String)> = vec![
            ("dbfrom", params.dbfrom.as_str().to_string()),
            ("db", params.db.as_str().to_string()),
            ("id", id_str.clone()),
            ("cmd", params.cmd.clone()),
            ("retmode", "json".to_string()),
        ];
        self.append_common_params(&mut query);

        debug!(
            dbfrom = %params.dbfrom, db = %params.db,
            ids = %id_str, cmd = %params.cmd,
            "NCBI ELink"
        );

        let parsed: ELinkResponse = self.get_json(ELINK_URL, &query).await?;

        let mut results = Vec::new();
        for linkset in &parsed.linksets {
            for lsdb in &linkset.linksetdbs {
                results.push(ELinkResult {
                    dbfrom: linkset.dbfrom.clone(),
                    dbto: lsdb.dbto.clone(),
                    linkname: lsdb.linkname.clone(),
                    source_ids: linkset.idlist.clone(),
                    linked_ids: lsdb.links.clone(),
                });
            }
        }

        debug!(linksets = results.len(), "ELink complete");
        Ok(results)
    }

    // -----------------------------------------------------------------------
    // ESummary
    // -----------------------------------------------------------------------

    /// Fetch document summaries (metadata without full records).
    pub async fn esummary(&self, params: &ESummaryParams) -> Result<Vec<DocSummary>> {
        if params.ids.is_empty() {
            return Err(NcbiError::InvalidParams("ID list cannot be empty".into()));
        }

        let id_str = params.ids.join(",");

        let mut query: Vec<(&str, String)> = vec![
            ("db", params.db.as_str().to_string()),
            ("id", id_str.clone()),
            ("retmode", "json".to_string()),
        ];
        self.append_common_params(&mut query);

        debug!(db = %params.db, ids = %id_str, "NCBI ESummary");

        let parsed: ESummaryResponse = self.get_json(ESUMMARY_URL, &query).await?;

        let summaries = match parsed.result {
            Some(result) => result.summaries(),
            None => Vec::new(),
        };

        debug!(count = summaries.len(), "ESummary complete");
        Ok(summaries)
    }

    // -----------------------------------------------------------------------
    // Convenience pipelines
    // -----------------------------------------------------------------------

    /// Search then fetch in one call.
    pub async fn search_and_fetch(&self, params: &ESearchParams) -> Result<Vec<BioRecord>> {
        let search_result = self.esearch(params).await?;

        if search_result.idlist.is_empty() {
            return Ok(Vec::new());
        }

        let fetch_params = EFetchParams::new(params.db, search_result.idlist);
        self.efetch(&fetch_params).await
    }

    /// Search then summarize in one call (faster than full fetch).
    pub async fn search_and_summarize(&self, params: &ESearchParams) -> Result<Vec<DocSummary>> {
        let search_result = self.esearch(params).await?;

        if search_result.idlist.is_empty() {
            return Ok(Vec::new());
        }

        let summary_params = ESummaryParams::new(params.db, search_result.idlist);
        self.esummary(&summary_params).await
    }

    /// Traverse: find UIDs in db_target linked to source UIDs in db_source.
    pub async fn linked_ids(
        &self,
        dbfrom: super::types::Database,
        dbto: super::types::Database,
        source_ids: Vec<String>,
    ) -> Result<Vec<String>> {
        let params = ELinkParams::new(dbfrom, dbto, source_ids);
        let results = self.elink(&params).await?;
        let mut ids: Vec<String> = results.into_iter().flat_map(|r| r.linked_ids).collect();
        ids.dedup();
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ncbi::types::Database;

    #[test]
    fn client_new_succeeds() {
        let client = NcbiClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn client_with_empty_key_fails() {
        let client = NcbiClient::with_api_key("");
        assert!(client.is_err());
        let err = client.unwrap_err();
        assert!(format!("{err}").contains("empty"));
    }

    #[test]
    fn client_with_valid_key_succeeds() {
        let client = NcbiClient::with_api_key("abc123");
        assert!(client.is_ok());
    }

    #[test]
    fn client_builder_methods() {
        let client = NcbiClient::new()
            .unwrap()
            .with_email("test@example.com")
            .with_tool_name("my-tool");
        assert_eq!(client.email, Some("test@example.com".to_string()));
        assert_eq!(client.tool_name, "my-tool");
    }

    #[tokio::test]
    async fn esearch_rejects_empty_term() {
        let client = NcbiClient::new().unwrap();
        let params = ESearchParams::new(Database::Nucleotide, "");
        let result = client.esearch(&params).await;
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("empty"));
    }

    #[tokio::test]
    async fn efetch_rejects_empty_ids() {
        let client = NcbiClient::new().unwrap();
        let params = EFetchParams::new(Database::Nucleotide, vec![]);
        let result = client.efetch(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn elink_rejects_empty_ids() {
        let client = NcbiClient::new().unwrap();
        let params = ELinkParams::new(Database::Gene, Database::Nucleotide, vec![]);
        let result = client.elink(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn esummary_rejects_empty_ids() {
        let client = NcbiClient::new().unwrap();
        let params = ESummaryParams::new(Database::Gene, vec![]);
        let result = client.esummary(&params).await;
        assert!(result.is_err());
    }

    // Live API tests — run with `cargo test --features ncbi -- --ignored`
    #[tokio::test]
    #[ignore]
    async fn live_esearch_brca1() {
        let client = NcbiClient::from_env().unwrap();
        let params = ESearchParams::new(Database::Nucleotide, "BRCA1 homo sapiens").with_retmax(5);
        let result = client.esearch(&params).await.unwrap();
        assert!(result.total_count() > 0);
        assert!(!result.idlist.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_search_and_fetch() {
        let client = NcbiClient::from_env().unwrap();
        let params =
            ESearchParams::new(Database::Nucleotide, "NM_001301717.2[accn]").with_retmax(1);
        let records = client.search_and_fetch(&params).await.unwrap();
        assert!(!records.is_empty());
        assert!(!records[0].sequence.bases.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_elink_gene_to_nucleotide() {
        let client = NcbiClient::from_env().unwrap();
        let params = ELinkParams::new(
            Database::Gene,
            Database::Nucleotide,
            vec!["7157".to_string()], // TP53
        );
        let results = client.elink(&params).await.unwrap();
        assert!(!results.is_empty());
        assert!(!results[0].linked_ids.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_esummary_gene() {
        let client = NcbiClient::from_env().unwrap();
        let params = ESummaryParams::new(Database::Gene, vec!["7157".to_string()]);
        let summaries = client.esummary(&params).await.unwrap();
        assert!(!summaries.is_empty());
        assert!(
            summaries[0]
                .title
                .to_lowercase()
                .contains("tumor protein p53")
        );
    }

    #[tokio::test]
    #[ignore]
    async fn live_pubmed_date_range() {
        let client = NcbiClient::from_env().unwrap();
        let params = ESearchParams::new(Database::PubMed, "BRCA1 adverse")
            .with_date_range("pdat", "2024/01/01", "2024/12/31")
            .with_retmax(5);
        let result = client.esearch(&params).await.unwrap();
        assert!(result.total_count() > 0);
    }
}
