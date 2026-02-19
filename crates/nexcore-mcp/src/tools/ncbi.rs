use crate::params::ncbi::*;
use nexcore_dna::ncbi::{
    Database, EFetchParams, ELinkParams, ESearchParams, ESummaryParams, NcbiClient,
};
use nexcore_dna::ops::gc_content;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

/// Parse a database string into a Database enum, with error handling.
fn parse_db(db: &str) -> Result<Database, McpError> {
    Database::from_str_loose(db).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown database '{}'. Valid: nucleotide, protein, gene, pubmed, clinvar, snp, taxonomy, omim",
                db
            ),
            None,
        )
    })
}

/// Create an NcbiClient from environment (uses NCBI_API_KEY if set).
fn make_client() -> Result<NcbiClient, McpError> {
    NcbiClient::from_env()
        .map_err(|e| McpError::internal_error(format!("Failed to create NCBI client: {}", e), None))
}

/// Search NCBI databases for UIDs matching a query.
pub async fn esearch(params: NcbiSearchParams) -> Result<CallToolResult, McpError> {
    let db = parse_db(&params.db)?;
    let client = make_client()?;

    let mut search = ESearchParams::new(db, &params.term);
    if let Some(max) = params.retmax {
        search = search.with_retmax(max);
    }
    if let Some(start) = params.retstart {
        search = search.with_retstart(start);
    }
    if let (Some(dt), Some(min), Some(max)) = (&params.datetype, &params.mindate, &params.maxdate) {
        search = search.with_date_range(dt.as_str(), min.as_str(), max.as_str());
    }

    let result = client
        .esearch(&search)
        .await
        .map_err(|e| McpError::internal_error(format!("ESearch failed: {}", e), None))?;

    let json = serde_json::json!({
        "count": result.total_count(),
        "retmax": result.retmax,
        "retstart": result.retstart,
        "idlist": result.idlist,
        "querytranslation": result.querytranslation,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", result)),
    )]))
}

/// Retrieve summaries for a list of NCBI UIDs.
pub async fn esummary(params: NcbiSummaryParams) -> Result<CallToolResult, McpError> {
    let db = parse_db(&params.db)?;
    let client = make_client()?;
    let ids: Vec<String> = params.id.split(',').map(|s| s.trim().to_string()).collect();

    let summary_params = ESummaryParams::new(db, ids);
    let summaries = client
        .esummary(&summary_params)
        .await
        .map_err(|e| McpError::internal_error(format!("ESummary failed: {}", e), None))?;

    let json: Vec<serde_json::Value> = summaries
        .iter()
        .map(|s| {
            serde_json::json!({
                "uid": s.uid,
                "title": s.title,
                "extra": s.extra,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", summaries)),
    )]))
}

/// Retrieve full records for a list of NCBI UIDs in various formats.
pub async fn efetch(params: NcbiFetchParams) -> Result<CallToolResult, McpError> {
    let db = parse_db(&params.db)?;
    let client = make_client()?;
    let ids: Vec<String> = params.id.split(',').map(|s| s.trim().to_string()).collect();

    let mut fetch = EFetchParams::new(db, ids);
    if let Some(ref rt) = params.rettype {
        fetch = fetch.with_rettype(rt.as_str());
    }
    if let Some(ref rm) = params.retmode {
        fetch = fetch.with_retmode(rm.as_str());
    }

    let text = client
        .efetch_raw(&fetch)
        .await
        .map_err(|e| McpError::internal_error(format!("EFetch failed: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Find linked records across NCBI databases.
pub async fn elink(params: NcbiLinkParams) -> Result<CallToolResult, McpError> {
    let dbfrom = parse_db(&params.dbfrom)?;
    let db = parse_db(&params.db)?;
    let client = make_client()?;
    let ids: Vec<String> = params.id.split(',').map(|s| s.trim().to_string()).collect();

    let mut link = ELinkParams::new(dbfrom, db, ids);
    if let Some(ref cmd) = params.cmd {
        link = link.with_cmd(cmd.as_str());
    }

    let results = client
        .elink(&link)
        .await
        .map_err(|e| McpError::internal_error(format!("ELink failed: {}", e), None))?;

    let json: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "dbfrom": r.dbfrom,
                "dbto": r.dbto,
                "linkname": r.linkname,
                "source_ids": r.source_ids,
                "linked_ids_count": r.linked_ids.len(),
                "linked_ids": if r.linked_ids.len() <= 20 {
                    r.linked_ids.clone()
                } else {
                    let mut preview = r.linked_ids[..20].to_vec();
                    preview.push(format!("... and {} more", r.linked_ids.len() - 20));
                    preview
                },
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", results)),
    )]))
}

/// Search then fetch FASTA records in one call.
pub async fn search_and_fetch(params: NcbiSearchFetchParams) -> Result<CallToolResult, McpError> {
    let db = parse_db(&params.db)?;
    let client = make_client()?;

    let mut search = ESearchParams::new(db, &params.term);
    search = search.with_retmax(params.retmax.unwrap_or(5));

    let records = client
        .search_and_fetch(&search)
        .await
        .map_err(|e| McpError::internal_error(format!("Search-and-fetch failed: {}", e), None))?;

    let json: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "description": r.description,
                "length": r.sequence.bases.len(),
                "gc_content": format!("{:.2}%", gc_content(&r.sequence) * 100.0),
                "first_60_bases": r.sequence.bases.iter().take(60)
                    .map(|n| n.as_char())
                    .collect::<String>(),
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", records)),
    )]))
}

/// Search then summarize in one call (faster than full fetch).
pub async fn search_and_summarize(
    params: NcbiSearchSummarizeParams,
) -> Result<CallToolResult, McpError> {
    let db = parse_db(&params.db)?;
    let client = make_client()?;

    let mut search = ESearchParams::new(db, &params.term);
    search = search.with_retmax(params.retmax.unwrap_or(10));
    if let (Some(dt), Some(min), Some(max)) = (&params.datetype, &params.mindate, &params.maxdate) {
        search = search.with_date_range(dt.as_str(), min.as_str(), max.as_str());
    }

    let summaries = client.search_and_summarize(&search).await.map_err(|e| {
        McpError::internal_error(format!("Search-and-summarize failed: {}", e), None)
    })?;

    let json: Vec<serde_json::Value> = summaries
        .iter()
        .map(|s| {
            serde_json::json!({
                "uid": s.uid,
                "title": s.title,
                "extra": s.extra,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", summaries)),
    )]))
}
