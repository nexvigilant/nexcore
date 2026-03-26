//! PV Guidelines tools: ICH, CIOMS, and EMA GVP guideline search and lookup
//!
//! Provides programmatic access to pharmacovigilance guidelines with scoring-based search.

use crate::params::{GuidelinesGetParams, GuidelinesSearchParams, GuidelinesUrlParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Embedded guidelines index (loaded at compile time)
const GUIDELINES_INDEX: &str = include_str!("../../data/guidelines.json");

/// Guideline document
///
/// Tier: T3 (Domain-specific guideline record)
/// Grounds to T1 Concepts via String/Vec/Option fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineDoc {
    /// Unique identifier (e.g., "E2B", "CIOMS-I", "GVP-Module-VI")
    pub id: String,
    /// Document title
    pub title: String,
    /// Brief description of the guideline's scope
    pub description: String,
    /// URL to official PDF or webpage
    pub url: String,
    /// Keywords for search matching
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Document status (e.g., "Current", "Step 4", "Under revision")
    #[serde(default)]
    pub status: Option<String>,
    /// Publication or revision year
    #[serde(default)]
    pub year: Option<u32>,
    /// Source organization (ich, cioms, ema) - runtime populated
    #[serde(skip_deserializing)]
    pub source: String,
    /// Category within source (e.g., "efficacy", "gvp_modules") - runtime populated
    #[serde(skip_deserializing)]
    pub category: String,
}

/// Guidelines index structure
///
/// Tier: T3 (Domain-specific guidelines index)
/// Grounds to T1 Concepts via HashMap and nested records
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Deserialize)]
struct IndexData {
    documents: IndexDocuments,
}

/// Guidelines document collections by source
///
/// Tier: T3 (Domain-specific guidelines index)
/// Grounds to T1 Concepts via HashMap and Vec
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Deserialize)]
struct IndexDocuments {
    ich: HashMap<String, Vec<GuidelineDoc>>,
    cioms: HashMap<String, Vec<GuidelineDoc>>,
    ema: HashMap<String, Vec<GuidelineDoc>>,
}

#[inline]
fn load_index() -> Result<IndexData, nexcore_error::NexError> {
    // ALLOC: Error message only on parse failure (rare)
    serde_json::from_str(GUIDELINES_INDEX)
        .map_err(|e| nexcore_error::nexerror!("Failed to parse guidelines index: {e}"))
}

/// Score a single document against query (helper to avoid nested loops)
#[inline]
fn score_single(doc: &GuidelineDoc, query_lower: &str, query_terms: &[&str]) -> f64 {
    let mut score = 0.0;
    if query_lower == doc.id.to_lowercase() {
        score += 100.0;
    }
    if doc.title.to_lowercase().contains(query_lower) {
        score += 50.0;
    }
    let desc_lower = doc.description.to_lowercase();
    query_terms
        .iter()
        .filter(|t| desc_lower.contains(*t))
        .count();
    score += query_terms
        .iter()
        .filter(|t| desc_lower.contains(*t))
        .count() as f64
        * 2.0;
    score += doc
        .keywords
        .iter()
        .filter(|kw| kw.to_lowercase().contains(query_lower))
        .count() as f64
        * 20.0;
    score
}

/// Collect all documents from index with source/category metadata
fn collect_all_docs(index: &IndexData) -> Vec<GuidelineDoc> {
    let mut all = Vec::new();
    // Flatten all sources into single Vec using extend
    index.documents.ich.iter().for_each(|(cat, docs)| {
        all.extend(docs.iter().map(|d| {
            let mut doc = d.clone();
            doc.source = "ich".to_string();
            doc.category = cat.clone();
            doc
        }));
    });
    index.documents.cioms.iter().for_each(|(cat, docs)| {
        all.extend(docs.iter().map(|d| {
            let mut doc = d.clone();
            doc.source = "cioms".to_string();
            doc.category = cat.clone();
            doc
        }));
    });
    index.documents.ema.iter().for_each(|(cat, docs)| {
        all.extend(docs.iter().map(|d| {
            let mut doc = d.clone();
            doc.source = "ema".to_string();
            doc.category = cat.clone();
            doc
        }));
    });
    all
}

/// Search documents by query string
pub fn search(params: GuidelinesSearchParams) -> Result<CallToolResult, McpError> {
    let index = load_index().map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let query = params.query;
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
    let limit = params.limit.unwrap_or(10);

    let all_docs = collect_all_docs(&index);

    // Filter by source/category if specified, then score
    let mut results: Vec<(GuidelineDoc, f64)> = all_docs
        .into_iter()
        .filter(|doc| {
            params
                .source
                .as_ref()
                .is_none_or(|s| s.eq_ignore_ascii_case(&doc.source))
                && params
                    .category
                    .as_ref()
                    .is_none_or(|c| c.eq_ignore_ascii_case(&doc.category))
        })
        .map(|doc| {
            let score = score_single(&doc, &query_lower, &query_terms);
            (doc, score)
        })
        .filter(|(_, score)| *score > 0.0)
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    if results.is_empty() {
        // ALLOC: User-facing message (once per request)
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "No guidelines found matching '{query}'"
        ))]));
    }

    // ALLOC: Output string built once per request (not hot path)
    let mut output = format!(
        "Found {} guideline(s) matching '{query}':\n\n",
        results.len()
    );
    results.iter().for_each(|(doc, _)| {
        output.push_str(&format!("**{}** - {}\n", doc.id, doc.title));
        output.push_str(&format!(
            "  Source: {} ({})\n",
            doc.source.to_uppercase(),
            doc.category
        ));
        output.push_str(&format!("  {}\n", doc.description));
        output.push_str(&format!("  URL: {}\n\n", doc.url));
    });

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// Get a specific guideline by ID
pub fn get(params: GuidelinesGetParams) -> Result<CallToolResult, McpError> {
    let index = load_index().map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let doc_id_upper = params.id.to_uppercase();

    let all_docs = collect_all_docs(&index);
    let found = all_docs.into_iter().find(|doc| {
        let id_upper = doc.id.to_uppercase();
        id_upper == doc_id_upper || id_upper.starts_with(&doc_id_upper)
    });

    match found {
        Some(doc) => format_guideline_output(&doc),
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Guideline '{}' not found. Try guidelines_search to find available documents.",
            params.id
        ))])),
    }
}

#[inline]
fn format_guideline_output(doc: &GuidelineDoc) -> Result<CallToolResult, McpError> {
    // ALLOC: All format! calls build user-facing output (once per request)
    let mut output = format!("# {} - {}\n\n", doc.id, doc.title);
    output.push_str(&format!("**Source:** {}\n", doc.source.to_uppercase()));
    output.push_str(&format!("**Category:** {}\n", doc.category));
    // ALLOC: Optional status field
    if let Some(ref status) = doc.status {
        output.push_str(&format!("**Status:** {status}\n"));
    }
    // ALLOC: Optional year field
    if let Some(year) = doc.year {
        output.push_str(&format!("**Year:** {year}\n"));
    }
    // ALLOC: Description always present
    output.push_str(&format!("\n**Description:**\n{}\n", doc.description));
    // ALLOC: Keywords array join
    if !doc.keywords.is_empty() {
        output.push_str(&format!("\n**Keywords:** {}\n", doc.keywords.join(", ")));
    }
    // ALLOC: URL always present
    output.push_str(&format!("\n**URL:** {}\n", doc.url));
    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// List all guideline categories with counts
pub fn categories() -> Result<CallToolResult, McpError> {
    let index = load_index().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // ALLOC: Output string built once per request
    let mut output = String::from("# Available Guideline Categories\n\n");
    output.push_str("## ICH (International Council for Harmonisation)\n\n");

    // ICH
    [
        ("quality", "Q"),
        ("safety", "S"),
        ("efficacy", "E"),
        ("multidisciplinary", "M"),
    ]
    .iter()
    .for_each(|(cat, prefix)| {
        if let Some(docs) = index.documents.ich.get(*cat) {
            let ids: Vec<_> = docs.iter().map(|d| d.id.as_str()).collect();
            output.push_str(&format!(
                "### {} - {} ({} documents)\nDocuments: {}\n\n",
                prefix,
                capitalize(cat),
                docs.len(),
                ids.join(", ")
            ));
        }
    });

    output.push_str("## CIOMS\n\n");
    ["pharmacovigilance", "ethics"].iter().for_each(|cat| {
        if let Some(docs) = index.documents.cioms.get(*cat) {
            let ids: Vec<_> = docs.iter().map(|d| d.id.as_str()).collect();
            output.push_str(&format!(
                "### {} ({} documents)\nDocuments: {}\n\n",
                capitalize(cat),
                docs.len(),
                ids.join(", ")
            ));
        }
    });

    output.push_str("## EMA GVP\n\n");
    [
        ("gvp_modules", "Modules"),
        ("gvp_addenda", "Addenda"),
        ("gvp_product_specific", "Product-Specific"),
        ("gvp_annexes", "Annexes"),
    ]
    .iter()
    .for_each(|(cat, name)| {
        if let Some(docs) = index.documents.ema.get(*cat) {
            let ids: Vec<_> = docs.iter().map(|d| d.id.as_str()).collect();
            output.push_str(&format!(
                "### {} ({} documents)\nDocuments: {}\n\n",
                name,
                docs.len(),
                ids.join(", ")
            ));
        }
    });

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// Get all pharmacovigilance-specific guidelines
pub fn pv_all() -> Result<CallToolResult, McpError> {
    let index = load_index().map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let all_docs = collect_all_docs(&index);

    // Filter to PV-related guidelines
    let pv_docs: Vec<_> = all_docs
        .into_iter()
        .filter(|doc| {
            doc.id.starts_with("E2")
                || doc.category == "pharmacovigilance"
                || doc.category.starts_with("gvp")
                || doc
                    .keywords
                    .iter()
                    .any(|k| k.to_lowercase().contains("pharmacovigilance"))
        })
        .collect();

    // ALLOC: Output string built once per request
    let mut output = String::from("# Pharmacovigilance Guidelines\n\n");
    output.push_str(&format!(
        "Found {} PV-related guidelines:\n\n",
        pv_docs.len()
    ));

    // Group by source using filter
    output.push_str("## ICH\n\n");
    pv_docs
        .iter()
        .filter(|d| d.source == "ich")
        .for_each(|doc| {
            output.push_str(&format!(
                "- **{}**: {}\n  {}\n\n",
                doc.id, doc.title, doc.description
            ));
        });

    output.push_str("## CIOMS\n\n");
    pv_docs
        .iter()
        .filter(|d| d.source == "cioms")
        .for_each(|doc| {
            output.push_str(&format!(
                "- **{}**: {}\n  {}\n\n",
                doc.id, doc.title, doc.description
            ));
        });

    output.push_str("## EMA GVP\n\n");
    pv_docs
        .iter()
        .filter(|d| d.source == "ema")
        .for_each(|doc| {
            output.push_str(&format!(
                "- **{}**: {}\n  {}\n\n",
                doc.id, doc.title, doc.description
            ));
        });

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// Get URL for a guideline
pub fn url(params: GuidelinesUrlParams) -> Result<CallToolResult, McpError> {
    let index = load_index().map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let doc_id_upper = params.id.to_uppercase();

    let all_docs = collect_all_docs(&index);
    let found = all_docs.into_iter().find(|doc| {
        let id_upper = doc.id.to_uppercase();
        id_upper == doc_id_upper || id_upper.starts_with(&doc_id_upper)
    });

    // ALLOC: Output string built once per request
    match found {
        Some(doc) => Ok(CallToolResult::success(vec![Content::text(format!(
            "URL for {}:\n{}",
            doc.id, doc.url
        ))])),
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Guideline '{}' not found.",
            params.id
        ))])),
    }
}

#[inline]
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
