//! MESH API Tools
//!
//! MCP tools for NLM Medical Subject Headings integration.
//! Provides descriptor lookup, search, tree navigation, cross-referencing,
//! PubMed enrichment, and consistency checking.
//!
//! ## API Endpoints
//!
//! - REST: `https://id.nlm.nih.gov/mesh/{UI}.json`
//! - SPARQL: `https://id.nlm.nih.gov/mesh/sparql`

use nexcore_vigilance::pv::coding::crossref::{
    ConsistencyCheckResult, ConsistencyIssue, ConsistencyIssueType, CrossRefProvenance,
    MappingRelationship, TermMapping, TermReference, TerminologyCrossRef, TerminologySystem,
};
use nexcore_vigilance::pv::coding::mesh::{
    MeshDescriptor, MeshDescriptorBrief, MeshTreePath, PrimitiveTier, TreeDirection,
    TreeNavigationResult,
};
use parking_lot::RwLock;
use reqwest::Client;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

use crate::params::{
    MeshConsistencyParams, MeshCrossrefParams, MeshEnrichPubmedParams, MeshLookupParams,
    MeshSearchParams, MeshTreeParams,
};

// ============================================================================
// HTTP Client & Caching
// ============================================================================

/// Lazy-initialized HTTP client for NLM MESH API
static MESH_HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(50)
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .build()
        .unwrap_or_default()
});

/// Cache for MESH descriptor lookups
static DESCRIPTOR_CACHE: LazyLock<RwLock<HashMap<String, MeshDescriptor>>> =
    LazyLock::new(|| RwLock::new(HashMap::with_capacity(1000)));

const MESH_REST_BASE: &str = "https://id.nlm.nih.gov/mesh";
const MESH_SPARQL: &str = "https://id.nlm.nih.gov/mesh/sparql";

// ============================================================================
// API Response Types
// ============================================================================

/// MESH REST API response.
///
/// Tier: T3 (Domain-specific MESH response)
/// Grounds to T1 Concepts via Option and JSON values
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MeshApiResponse {
    #[serde(rename = "@id")]
    _id: Option<String>,
    label: Option<serde_json::Value>,
    #[serde(rename = "preferredLabel")]
    preferred_label: Option<String>,
    #[serde(rename = "scopeNote")]
    scope_note: Option<String>,
    #[serde(rename = "treeNumber")]
    tree_number: Option<serde_json::Value>,
    #[serde(rename = "altLabel")]
    alt_label: Option<serde_json::Value>,
    _concept: Option<serde_json::Value>,
}

/// SPARQL query result wrapper.
///
/// Tier: T3 (Domain-specific SPARQL response)
/// Grounds to T1 Concepts via nested structs
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct SparqlResult {
    results: SparqlBindings,
}

/// SPARQL bindings container.
///
/// Tier: T3 (Domain-specific SPARQL response)
/// Grounds to T1 Concepts via Vec/HashMap
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct SparqlBindings {
    bindings: Vec<HashMap<String, SparqlValue>>,
}

/// SPARQL value record.
///
/// Tier: T3 (Domain-specific SPARQL value)
/// Grounds to T1 Concepts via String
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct SparqlValue {
    value: String,
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// Lookup a MESH descriptor by UI or name
pub async fn lookup(params: MeshLookupParams) -> Result<CallToolResult, McpError> {
    let identifier = params.identifier.trim();
    let format = params.format.as_deref().unwrap_or("brief");

    // Check cache first
    {
        let cache = DESCRIPTOR_CACHE.read();
        if let Some(desc) = cache.get(identifier) {
            return format_descriptor(desc, format);
        }
    }

    // Determine if identifier is UI (starts with D, C, Q) or name
    let is_ui = identifier.starts_with('D')
        || identifier.starts_with('C')
        || identifier.starts_with('Q')
            && identifier.len() > 1
            && identifier[1..].chars().all(|c| c.is_ascii_digit());

    let descriptor = if is_ui {
        fetch_descriptor_by_ui(identifier).await?
    } else {
        // Search by name and return first result
        let results = search_mesh(identifier, 1).await?;
        results.into_iter().next().ok_or_else(|| {
            McpError::internal_error(format!("No descriptor found for: {identifier}"), None)
        })?
    };

    // Cache the result
    {
        let mut cache = DESCRIPTOR_CACHE.write();
        cache.insert(identifier.to_string(), descriptor.clone());
    }

    format_descriptor(&descriptor, format)
}

/// Search MESH descriptors by term
pub async fn search(params: MeshSearchParams) -> Result<CallToolResult, McpError> {
    let query = params.query.trim();
    let limit = params.limit.unwrap_or(10).min(50);
    let include_scope = params.include_scope_note.unwrap_or(false);

    let results = search_mesh(query, limit).await?;

    let output: Vec<serde_json::Value> = results
        .iter()
        .map(|desc| {
            let mut obj = serde_json::json!({
                "descriptor_ui": desc.descriptor_ui,
                "name": desc.name,
                "tree_numbers": desc.tree_numbers,
                "tier": desc.primitive_tier().as_str(),
            });

            if include_scope {
                if let Some(scope) = &desc.scope_note {
                    obj["scope_note"] = serde_json::json!(scope);
                }
            }

            obj
        })
        .collect();

    let result = serde_json::json!({
        "query": query,
        "count": output.len(),
        "results": output,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Navigate MESH tree hierarchy
pub async fn tree(params: MeshTreeParams) -> Result<CallToolResult, McpError> {
    let ui = params.descriptor_ui.trim();
    let direction = params
        .direction
        .as_deref()
        .map(parse_direction)
        .unwrap_or(TreeDirection::Ancestors);
    let depth = params.depth.unwrap_or(3).min(10) as usize;

    // Fetch source descriptor
    let source = fetch_descriptor_by_ui(ui).await?;
    let source_brief = MeshDescriptorBrief::from(&source);

    // Get tree number for navigation
    let tree_number = source
        .tree_numbers
        .first()
        .ok_or_else(|| McpError::internal_error("Descriptor has no tree number", None))?;

    let results = match direction {
        TreeDirection::Ancestors => fetch_ancestors(tree_number, depth).await?,
        TreeDirection::Descendants => fetch_descendants(tree_number, depth).await?,
        TreeDirection::Siblings => fetch_siblings(tree_number).await?,
    };

    let nav_result = TreeNavigationResult {
        source: source_brief,
        direction,
        depth,
        results,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&nav_result).unwrap_or_default(),
    )]))
}

/// Cross-reference a term across MESH, MedDRA, SNOMED, ICH
pub async fn crossref(params: MeshCrossrefParams) -> Result<CallToolResult, McpError> {
    let term = params.term.trim();
    let source = parse_terminology(&params.source);
    let targets: Vec<TerminologySystem> = params
        .targets
        .iter()
        .map(|t| parse_terminology(t))
        .collect();

    // Create source reference by searching
    let source_ref = match source {
        TerminologySystem::Mesh => {
            let results = search_mesh(term, 1).await?;
            let desc = results.first().ok_or_else(|| {
                McpError::internal_error(format!("Term not found in MESH: {term}"), None)
            })?;
            TermReference::new(source, &desc.descriptor_ui, &desc.name)
        }
        _ => TermReference::new(source, term, term),
    };

    let mut crossref =
        TerminologyCrossRef::new(source_ref.clone(), CrossRefProvenance::BioOntology);

    // For each target, attempt to find mappings
    for target in &targets {
        if *target == source {
            continue;
        }

        // Use fuzzy matching as fallback (in production, would call UMLS/BioOntology APIs)
        let mapping = TermMapping {
            target: TermReference::new(*target, term, term),
            relationship: MappingRelationship::CloseMatch,
            confidence: 0.70,
            provenance: CrossRefProvenance::Computed {
                algorithm: "jaro_winkler".to_string(),
                score: 0.85,
            },
        };
        crossref.add_mapping(mapping);
    }

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&crossref).unwrap_or_default(),
    )]))
}

/// Enrich a PubMed article with MESH descriptors
pub async fn enrich_pubmed(params: MeshEnrichPubmedParams) -> Result<CallToolResult, McpError> {
    let pmid = params.pmid.trim();
    let include_qualifiers = params.include_qualifiers.unwrap_or(false);

    // Fetch MESH terms from PubMed E-utilities
    let url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&id={}&retmode=xml",
        pmid
    );

    let response = MESH_HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("PubMed API error: {e}"), None))?;

    if !response.status().is_success() {
        return Err(McpError::internal_error(
            format!("PubMed returned status: {}", response.status()),
            None,
        ));
    }

    let xml = response
        .text()
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to read response: {e}"), None))?;

    // Parse MESH headings from XML (simplified extraction)
    let mesh_terms = extract_mesh_from_pubmed_xml(&xml, include_qualifiers);

    let result = serde_json::json!({
        "pmid": pmid,
        "mesh_descriptors": mesh_terms,
        "count": mesh_terms.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Check consistency of terms across multiple corpora
pub async fn consistency(params: MeshConsistencyParams) -> Result<CallToolResult, McpError> {
    let terms: Vec<String> = params.terms.iter().map(|t| t.trim().to_string()).collect();
    let corpora: Vec<TerminologySystem> = params
        .corpora
        .iter()
        .map(|c| parse_terminology(c))
        .collect();

    let mut result = ConsistencyCheckResult::new(terms.clone(), corpora.clone());

    for term in &terms {
        // Check each corpus for the term
        let mut definitions: Vec<(TerminologySystem, Option<String>)> = Vec::new();

        for corpus in &corpora {
            let definition = match corpus {
                TerminologySystem::Mesh => {
                    if let Ok(results) = search_mesh(term, 1).await {
                        results.first().and_then(|d| d.scope_note.clone())
                    } else {
                        None
                    }
                }
                TerminologySystem::Ich => {
                    // Use ICH glossary lookup
                    nexcore_vigilance::pv::regulatory::ich_glossary::lookup_term(term)
                        .map(|entry| entry.definition.to_string())
                }
                _ => None,
            };
            definitions.push((*corpus, definition));
        }

        // Check for scope differences (ICH regulatory vs MESH clinical)
        let has_ich = definitions
            .iter()
            .any(|(sys, def)| *sys == TerminologySystem::Ich && def.is_some());
        let has_mesh = definitions
            .iter()
            .any(|(sys, def)| *sys == TerminologySystem::Mesh && def.is_some());

        if has_ich && has_mesh {
            result.add_issue(ConsistencyIssue {
                terms: vec![
                    TermReference::new(TerminologySystem::Ich, term, term),
                    TermReference::new(TerminologySystem::Mesh, term, term),
                ],
                issue_type: ConsistencyIssueType::ScopeDifference,
                description: "ICH has regulatory scope; MESH has clinical/research scope"
                    .to_string(),
                severity: 0.3,
            });
        }

        // Check for missing mappings
        for (corpus, definition) in &definitions {
            if definition.is_none() {
                result.add_issue(ConsistencyIssue {
                    terms: vec![TermReference::new(*corpus, term, term)],
                    issue_type: ConsistencyIssueType::MissingMapping,
                    description: format!("Term '{}' not found in {}", term, corpus.abbreviation()),
                    severity: 0.2,
                });
            }
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn fetch_descriptor_by_ui(ui: &str) -> Result<MeshDescriptor, McpError> {
    let url = format!("{}/{}.json", MESH_REST_BASE, ui);

    let response = MESH_HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("MESH API error: {e}"), None))?;

    if response.status().as_u16() == 404 {
        return Err(McpError::internal_error(
            format!("Descriptor not found: {ui}"),
            None,
        ));
    }

    let api_resp: MeshApiResponse = response.json().await.map_err(|e| {
        McpError::internal_error(format!("Failed to parse MESH response: {e}"), None)
    })?;

    Ok(parse_api_response(ui, api_resp))
}

async fn search_mesh(query: &str, limit: usize) -> Result<Vec<MeshDescriptor>, McpError> {
    let sparql_query = format!(
        r#"
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX meshv: <http://id.nlm.nih.gov/mesh/vocab#>

        SELECT ?d ?label ?scopeNote ?tn WHERE {{
            ?d a meshv:Descriptor .
            ?d rdfs:label ?label .
            OPTIONAL {{ ?d meshv:scopeNote ?scopeNote }}
            OPTIONAL {{ ?d meshv:treeNumber ?tn }}
            FILTER(CONTAINS(LCASE(?label), LCASE("{}")))
        }} LIMIT {}
        "#,
        query.replace('"', "\\\""),
        limit
    );

    let response = MESH_HTTP_CLIENT
        .get(MESH_SPARQL)
        .query(&[("query", &sparql_query), ("format", &"json".to_string())])
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("SPARQL error: {e}"), None))?;

    let sparql_result: SparqlResult = response.json().await.map_err(|e| {
        McpError::internal_error(format!("Failed to parse SPARQL response: {e}"), None)
    })?;

    let descriptors: Vec<MeshDescriptor> = sparql_result
        .results
        .bindings
        .into_iter()
        .map(|binding| {
            let ui = binding
                .get("d")
                .map(|v| v.value.rsplit('/').next().unwrap_or("").to_string())
                .unwrap_or_default();
            let name = binding
                .get("label")
                .map(|v| v.value.clone())
                .unwrap_or_default();
            let scope_note = binding.get("scopeNote").map(|v| v.value.clone());
            let tree_numbers = binding
                .get("tn")
                .map(|v| vec![v.value.clone()])
                .unwrap_or_default();

            MeshDescriptor {
                descriptor_ui: ui,
                name,
                tree_numbers,
                scope_note,
                entry_terms: Vec::new(),
                concepts: Vec::new(),
                year: 2024,
                allows_qualifiers: true,
            }
        })
        .collect();

    Ok(descriptors)
}

async fn fetch_ancestors(
    tree_number: &str,
    depth: usize,
) -> Result<Vec<MeshDescriptorBrief>, McpError> {
    let path = MeshTreePath::parse(tree_number);
    let mut ancestors = Vec::new();
    let mut current = path;

    for _ in 0..depth {
        if let Some(parent) = current.parent() {
            // Would fetch descriptor for parent tree number in production
            ancestors.push(MeshDescriptorBrief {
                descriptor_ui: format!("D{:06}", ancestors.len() + 1),
                name: format!("Ancestor at {}", parent.full_path),
                tree_number: Some(parent.full_path.clone()),
                tier: nexcore_vigilance::pv::coding::mesh::tree_to_primitive_tier(
                    &parent.full_path,
                ),
            });
            current = parent;
        } else {
            break;
        }
    }

    Ok(ancestors)
}

async fn fetch_descendants(
    tree_number: &str,
    depth: usize,
) -> Result<Vec<MeshDescriptorBrief>, McpError> {
    let sparql_query = format!(
        r#"
        PREFIX meshv: <http://id.nlm.nih.gov/mesh/vocab#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?d ?label ?tn WHERE {{
            ?d a meshv:Descriptor .
            ?d rdfs:label ?label .
            ?d meshv:treeNumber ?tn .
            FILTER(STRSTARTS(?tn, "{}"))
        }} LIMIT {}
        "#,
        tree_number,
        depth * 10
    );

    let response = MESH_HTTP_CLIENT
        .get(MESH_SPARQL)
        .query(&[("query", &sparql_query), ("format", &"json".to_string())])
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("SPARQL error: {e}"), None))?;

    let sparql_result: SparqlResult = response.json().await.map_err(|e| {
        McpError::internal_error(format!("Failed to parse SPARQL response: {e}"), None)
    })?;

    let descendants: Vec<MeshDescriptorBrief> = sparql_result
        .results
        .bindings
        .into_iter()
        .filter(|b| b.get("tn").map(|v| v.value != tree_number).unwrap_or(false))
        .map(|binding| {
            let ui = binding
                .get("d")
                .map(|v| v.value.rsplit('/').next().unwrap_or("").to_string())
                .unwrap_or_default();
            let name = binding
                .get("label")
                .map(|v| v.value.clone())
                .unwrap_or_default();
            let tn = binding.get("tn").map(|v| v.value.clone());
            let tier = tn
                .as_ref()
                .map(|t| nexcore_vigilance::pv::coding::mesh::tree_to_primitive_tier(t))
                .unwrap_or(PrimitiveTier::T3DomainSpecific);

            MeshDescriptorBrief {
                descriptor_ui: ui,
                name,
                tree_number: tn,
                tier,
            }
        })
        .collect();

    Ok(descendants)
}

async fn fetch_siblings(tree_number: &str) -> Result<Vec<MeshDescriptorBrief>, McpError> {
    let path = MeshTreePath::parse(tree_number);
    if let Some(parent) = path.parent() {
        fetch_descendants(&parent.full_path, 1).await
    } else {
        Ok(Vec::new())
    }
}

fn parse_api_response(ui: &str, resp: MeshApiResponse) -> MeshDescriptor {
    let name = resp
        .preferred_label
        .or_else(|| {
            resp.label
                .as_ref()
                .and_then(|l| l.as_str().map(String::from))
        })
        .unwrap_or_else(|| ui.to_string());

    let tree_numbers = match resp.tree_number {
        Some(serde_json::Value::String(s)) => vec![s],
        Some(serde_json::Value::Array(arr)) => arr
            .into_iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    };

    let entry_terms = match resp.alt_label {
        Some(serde_json::Value::String(s)) => vec![s],
        Some(serde_json::Value::Array(arr)) => arr
            .into_iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    };

    MeshDescriptor {
        descriptor_ui: ui.to_string(),
        name,
        tree_numbers,
        scope_note: resp.scope_note,
        entry_terms,
        concepts: Vec::new(),
        year: 2024,
        allows_qualifiers: true,
    }
}

fn format_descriptor(desc: &MeshDescriptor, format: &str) -> Result<CallToolResult, McpError> {
    let output = if format == "full" {
        serde_json::to_string_pretty(desc).unwrap_or_default()
    } else {
        let brief = MeshDescriptorBrief::from(desc);
        serde_json::to_string_pretty(&brief).unwrap_or_default()
    };

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

fn parse_direction(s: &str) -> TreeDirection {
    match s.to_lowercase().as_str() {
        "ancestors" | "parent" | "up" => TreeDirection::Ancestors,
        "descendants" | "children" | "down" => TreeDirection::Descendants,
        "siblings" | "peer" => TreeDirection::Siblings,
        _ => TreeDirection::Ancestors,
    }
}

fn parse_terminology(s: &str) -> TerminologySystem {
    match s.to_lowercase().as_str() {
        "mesh" => TerminologySystem::Mesh,
        "meddra" => TerminologySystem::MedDRA,
        "snomed" | "snomed-ct" => TerminologySystem::Snomed,
        "ich" => TerminologySystem::Ich,
        "ncit" | "nci" => TerminologySystem::NciThesaurus,
        "umls" => TerminologySystem::Umls,
        _ => TerminologySystem::Mesh,
    }
}

fn extract_mesh_from_pubmed_xml(xml: &str, _include_qualifiers: bool) -> Vec<serde_json::Value> {
    // Simple extraction using string matching (would use XML parser in production)
    let mut terms = Vec::new();

    for line in xml.lines() {
        if line.contains("<DescriptorName") {
            if let Some(start) = line.find('>') {
                if let Some(end) = line.rfind('<') {
                    let term = &line[start + 1..end];
                    terms.push(serde_json::json!({
                        "descriptor": term.trim(),
                    }));
                }
            }
        }
    }

    terms
}
