//! PV Axioms Database MCP tools
//!
//! Read-only access to the `pv-axioms.db` SQLite database containing
//! 1,462 KSBs, 341 regulations, 79 axioms, 21 EPAs, 16 primitives,
//! and full regulatory traceability chains.

use crate::params::{
    PvAxiomsDomainDashboardParams, PvAxiomsKsbLookupParams, PvAxiomsQueryParams,
    PvAxiomsRegulationSearchParams, PvAxiomsTraceabilityParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use rusqlite::{Connection, OpenFlags};
use serde_json::{Value, json};

/// Open pv-axioms.db in read-only mode.
fn open_db() -> Result<Connection, McpError> {
    let root = std::env::var("NEXCORE_ROOT").unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|h| h.join("nexcore").to_string_lossy().to_string())
            .unwrap_or_else(|| "/home/matthew/nexcore".to_string())
    });
    let path = std::path::Path::new(&root).join("data/pv-axioms.db");
    Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| McpError::internal_error(format!("Failed to open pv-axioms.db: {e}"), None))
}

/// Execute a query and return results as a JSON array of objects.
fn query_to_json(
    conn: &Connection,
    sql: &str,
    param_values: &[&dyn rusqlite::types::ToSql],
    max_rows: usize,
) -> Result<Vec<Value>, McpError> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| McpError::internal_error(format!("SQL prepare error: {e}"), None))?;

    let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    let rows = stmt
        .query_map(param_values, |row| {
            let mut obj = serde_json::Map::new();
            for (i, name) in col_names.iter().enumerate() {
                let val: rusqlite::types::Value = row.get(i)?;
                let json_val = match val {
                    rusqlite::types::Value::Null => Value::Null,
                    rusqlite::types::Value::Integer(n) => json!(n),
                    rusqlite::types::Value::Real(f) => json!(f),
                    rusqlite::types::Value::Text(s) => json!(s),
                    rusqlite::types::Value::Blob(b) => json!(format!("<blob {} bytes>", b.len())),
                };
                obj.insert(name.clone(), json_val);
            }
            Ok(Value::Object(obj))
        })
        .map_err(|e| McpError::internal_error(format!("SQL query error: {e}"), None))?;

    let mut results = Vec::new();
    for row in rows {
        if results.len() >= max_rows {
            break;
        }
        results.push(row.map_err(|e| McpError::internal_error(format!("Row error: {e}"), None))?);
    }
    Ok(results)
}

/// Look up KSBs by ID, domain, type, or keyword.
pub fn ksb_lookup(params: PvAxiomsKsbLookupParams) -> Result<CallToolResult, McpError> {
    let conn = open_db()?;
    let limit = params.limit.unwrap_or(50).min(200);

    let mut conditions = Vec::new();
    let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref id) = params.ksb_id {
        conditions.push("k.ksb_id = ?");
        bind_values.push(Box::new(id.clone()));
    }
    if let Some(ref domain) = params.domain_id {
        conditions.push("k.domain_id = ?");
        bind_values.push(Box::new(domain.clone()));
    }
    if let Some(ref ktype) = params.ksb_type {
        conditions.push("k.ksb_type = ?");
        bind_values.push(Box::new(ktype.clone()));
    }
    if let Some(ref kw) = params.keyword {
        conditions.push("(k.item_name LIKE ? OR k.description LIKE ?)");
        let pattern = format!("%{kw}%");
        bind_values.push(Box::new(pattern.clone()));
        bind_values.push(Box::new(pattern));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT k.ksb_id, k.domain_id, k.ksb_type, k.major_section, k.section, \
         k.item_name, k.description, k.proficiency_level, k.bloom_level, \
         k.keywords, k.regulatory_refs, k.epa_ids, k.cpa_ids, k.status \
         FROM ksbs k {where_clause} ORDER BY k.ksb_id LIMIT {limit}"
    );

    let refs: Vec<&dyn rusqlite::types::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
    let results = query_to_json(&conn, &sql, &refs, limit)?;

    let output = json!({
        "count": results.len(),
        "ksbs": results,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Search regulations by text, jurisdiction, or domain.
pub fn regulation_search(
    params: PvAxiomsRegulationSearchParams,
) -> Result<CallToolResult, McpError> {
    let conn = open_db()?;
    let limit = params.limit.unwrap_or(50).min(200);

    let mut conditions = Vec::new();
    let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut use_domain_join = false;

    if let Some(ref q) = params.query {
        conditions.push("(r.title LIKE ? OR r.summary_description LIKE ?)");
        let pattern = format!("%{q}%");
        bind_values.push(Box::new(pattern.clone()));
        bind_values.push(Box::new(pattern));
    }
    if let Some(ref j) = params.jurisdiction {
        conditions.push("r.jurisdiction = ?");
        bind_values.push(Box::new(j.clone()));
    }
    if let Some(ref d) = params.domain_id {
        use_domain_join = true;
        conditions.push("rd.domain_id = ?");
        bind_values.push(Box::new(d.clone()));
    }

    let join = if use_domain_join {
        "JOIN regulation_domains rd ON rd.reg_id = r.reg_id"
    } else {
        ""
    };

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT DISTINCT r.reg_id, r.official_identifier, r.title, r.jurisdiction, \
         r.document_type, r.status, r.pv_activity_category, r.summary_description, \
         r.key_requirements, r.compliance_risk_level, r.harmonization_status, \
         r.effective_date \
         FROM regulations r {join} {where_clause} ORDER BY r.reg_id LIMIT {limit}"
    );

    let refs: Vec<&dyn rusqlite::types::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
    let results = query_to_json(&conn, &sql, &refs, limit)?;

    let output = json!({
        "count": results.len(),
        "regulations": results,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Trace axiom → parameter → pipeline stage → Rust coverage.
pub fn traceability_chain(params: PvAxiomsTraceabilityParams) -> Result<CallToolResult, McpError> {
    let conn = open_db()?;

    let mut conditions = Vec::new();
    let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref id) = params.axiom_id {
        conditions.push("axiom_id = ?");
        bind_values.push(Box::new(id.clone()));
    }
    if let Some(ref sg) = params.source_guideline {
        conditions.push("source_guideline LIKE ?");
        bind_values.push(Box::new(format!("%{sg}%")));
    }
    if let Some(ref p) = params.primitive {
        conditions.push("primitive = ?");
        bind_values.push(Box::new(p.clone()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT source_guideline, axiom_id, axiom_definition, binding_level, \
         parameter_name, primitive, pipeline_stage, rust_type, rust_crate, \
         implementation_status \
         FROM v_traceability {where_clause} ORDER BY axiom_id"
    );

    let refs: Vec<&dyn rusqlite::types::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
    let results = query_to_json(&conn, &sql, &refs, 500)?;

    let output = json!({
        "count": results.len(),
        "traceability": results,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Domain dashboard with KSB counts, regulation counts, and coverage.
pub fn domain_dashboard(params: PvAxiomsDomainDashboardParams) -> Result<CallToolResult, McpError> {
    let conn = open_db()?;

    // Domain overview
    let (domain_sql, domain_binds): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
        if let Some(ref d) = params.domain_id {
            (
                "SELECT * FROM domains WHERE domain_id = ?".to_string(),
                vec![Box::new(d.clone())],
            )
        } else {
            (
                "SELECT * FROM domains ORDER BY domain_id".to_string(),
                Vec::new(),
            )
        };
    let drefs: Vec<&dyn rusqlite::types::ToSql> = domain_binds.iter().map(|b| b.as_ref()).collect();
    let domains = query_to_json(&conn, &domain_sql, &drefs, 15)?;

    // KSB summary by domain
    let (ksb_sql, ksb_binds): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
        if let Some(ref d) = params.domain_id {
            (
                "SELECT * FROM v_ksb_by_domain WHERE domain_id = ?".to_string(),
                vec![Box::new(d.clone())],
            )
        } else {
            (
                "SELECT * FROM v_ksb_by_domain ORDER BY domain_id".to_string(),
                Vec::new(),
            )
        };
    let krefs: Vec<&dyn rusqlite::types::ToSql> = ksb_binds.iter().map(|b| b.as_ref()).collect();
    let ksb_summary = query_to_json(&conn, &ksb_sql, &krefs, 15)?;

    // System dashboard (single-row summary)
    let dashboard = query_to_json(&conn, "SELECT * FROM v_system_dashboard", &[], 1)?;

    let output = json!({
        "domains": domains,
        "ksb_summary": ksb_summary,
        "system_dashboard": dashboard.first().cloned().unwrap_or(Value::Null),
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Raw SQL query (read-only, max 100 rows).
pub fn query(params: PvAxiomsQueryParams) -> Result<CallToolResult, McpError> {
    // Safety: reject write operations
    let upper = params.sql.to_uppercase();
    let forbidden = [
        "INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "CREATE", "ATTACH", "DETACH",
    ];
    for kw in &forbidden {
        if upper.contains(kw) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Read-only database: '{kw}' operations are not allowed"
            ))]));
        }
    }

    let conn = open_db()?;
    let results = query_to_json(&conn, &params.sql, &[], 100)?;

    let output = json!({
        "count": results.len(),
        "rows": results,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
