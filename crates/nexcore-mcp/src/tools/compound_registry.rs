//! Compound Registry MCP tools.
//!
//! Three-layer chemical compound resolution (Cache → PubChem → ChEMBL) with
//! local SQLite caching. Provides compound lookup, batch resolution, and
//! cache management.

use std::sync::OnceLock;

use nexcore_compound_registry::{CacheStore, CompoundRecord};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::compound_registry::{
    CompoundCacheCountParams, CompoundCacheGetParams, CompoundCacheSearchParams,
    CompoundResolveBatchParams, CompoundResolveParams,
};

// ── State ────────────────────────────────────────────────────────────────

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn internal_err(msg: String) -> McpError {
    McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: msg.into(),
        data: None,
    }
}

/// Create a CacheStore per call. rusqlite::Connection is !Sync, so we can't
/// put it in OnceLock. SQLite handles concurrent file access via its own locks.
fn open_cache() -> Result<CacheStore, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let db_path = std::path::PathBuf::from(home).join(".local/share/nexcore/compounds.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| internal_err(["Cache dir creation failed: ", &e.to_string()].concat()))?;
    }
    let path_str = db_path.to_string_lossy().to_string();
    CacheStore::new(&path_str)
        .map_err(|e| internal_err(["Compound cache open failed: ", &e.to_string()].concat()))
}

fn http() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("nexcore-mcp/1.0 (NexVigilant)")
            .build()
            .unwrap_or_default()
    })
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn record_to_json(r: &CompoundRecord) -> serde_json::Value {
    json!({
        "name": r.name,
        "smiles": r.smiles,
        "inchi": r.inchi,
        "inchi_key": r.inchi_key,
        "cas_number": r.cas_number,
        "pubchem_cid": r.pubchem_cid,
        "chembl_id": r.chembl_id,
        "synonyms": r.synonyms,
        "source": r.source,
        "resolved_at": r.resolved_at.to_rfc3339(),
    })
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Resolve a compound by name through the 3-layer pipeline (Cache → PubChem → ChEMBL).
/// Sync wrapper: uses block_in_place because CacheStore (rusqlite) is !Send.
pub fn compound_resolve(p: CompoundResolveParams) -> Result<CallToolResult, McpError> {
    let store = open_cache()?;
    let client = http();
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(nexcore_compound_registry::resolve(&p.name, &store, client))
    });
    match result {
        Ok(record) => ok_json(json!({
            "resolved": true,
            "compound": record_to_json(&record),
        })),
        Err(e) => err_result(&["Resolution failed for '", &p.name, "': ", &e.to_string()].concat()),
    }
}

/// Batch resolve multiple compounds by name.
/// Sync wrapper: uses block_in_place because CacheStore (rusqlite) is !Send.
pub fn compound_resolve_batch(p: CompoundResolveBatchParams) -> Result<CallToolResult, McpError> {
    let store = open_cache()?;
    let client = http();
    let name_refs: Vec<&str> = p.names.iter().map(|s| s.as_str()).collect();
    let results = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(nexcore_compound_registry::resolve_batch(
            &name_refs, &store, client,
        ))
    });

    let mut resolved_count = 0usize;
    let items: Vec<serde_json::Value> = results
        .iter()
        .map(|(name, result)| match result {
            Ok(record) => {
                resolved_count += 1;
                json!({
                    "name": name,
                    "resolved": true,
                    "compound": record_to_json(record),
                })
            }
            Err(e) => json!({
                "name": name,
                "resolved": false,
                "error": e.to_string(),
            }),
        })
        .collect();

    ok_json(json!({
        "total": items.len(),
        "resolved": resolved_count,
        "failed": items.len() - resolved_count,
        "results": items,
    }))
}

/// Search cached compounds by partial name match.
pub fn compound_cache_search(p: CompoundCacheSearchParams) -> Result<CallToolResult, McpError> {
    let store = open_cache()?;
    let limit = p.limit.unwrap_or(20);
    match store.search(&p.query, limit) {
        Ok(records) => ok_json(json!({
            "query": p.query,
            "returned": records.len(),
            "results": records.iter().map(record_to_json).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("Cache search error: {e}")),
    }
}

/// Get a specific compound from cache by exact name.
pub fn compound_cache_get(p: CompoundCacheGetParams) -> Result<CallToolResult, McpError> {
    let store = open_cache()?;
    match store.get(&p.name) {
        Ok(Some(record)) => ok_json(json!({
            "found": true,
            "compound": record_to_json(&record),
        })),
        Ok(None) => ok_json(json!({
            "found": false,
            "name": p.name,
        })),
        Err(e) => err_result(&format!("Cache get error: {e}")),
    }
}

/// Count total compounds in the local cache.
pub fn compound_cache_count(_p: CompoundCacheCountParams) -> Result<CallToolResult, McpError> {
    let store = open_cache()?;
    match store.count() {
        Ok(count) => ok_json(json!({
            "count": count,
        })),
        Err(e) => err_result(&format!("Cache count error: {e}")),
    }
}
