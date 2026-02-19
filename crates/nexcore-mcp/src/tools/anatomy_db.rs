//! Anatomy DB — persistent storage for all biological organ subsystems.
//!
//! # T1 Grounding
//! - π (persistence): Durable SQLite storage for organ state
//! - σ (sequence): Time-series data for hormones, energy, guardian ticks
//! - ν (frequency): Event counts, rates, trends
//! - ∂ (boundary): Schema validation, query limits

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use rusqlite::Connection;
use serde_json::json;
use std::sync::Mutex;

use crate::params::anatomy_db::{
    AnatomyQueryParams, AnatomyRecordCytokineParams, AnatomyRecordEnergyParams,
    AnatomyRecordGuardianTickParams, AnatomyRecordHormonesParams, AnatomyRecordImmunityEventParams,
    AnatomyRecordOrganSignalParams, AnatomyRecordPhenotypeParams, AnatomyRecordRibosomeParams,
    AnatomyRecordSynapseParams, AnatomyRecordTranscriptaseParams, AnatomyStatusParams,
};

static DB: Mutex<Option<Connection>> = Mutex::new(None);

fn db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    format!("{home}/.claude/anatomy/anatomy.db")
}

fn get_conn() -> Result<std::sync::MutexGuard<'static, Option<Connection>>, McpError> {
    let mut guard = DB
        .lock()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if guard.is_none() {
        let path = db_path();
        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path).map_err(|e| {
            McpError::internal_error(format!("Failed to open anatomy.db: {e}"), None)
        })?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        init_schema(&conn)?;
        *guard = Some(conn);
    }

    Ok(guard)
}

fn init_schema(conn: &Connection) -> Result<(), McpError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS organs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'unknown',
            last_seen TEXT,
            activation_count INTEGER DEFAULT 0,
            error_count INTEGER DEFAULT 0,
            total_latency_ms INTEGER DEFAULT 0,
            call_count INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS cytokine_signals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            signal_id TEXT NOT NULL,
            family TEXT NOT NULL,
            name TEXT NOT NULL,
            severity TEXT,
            scope TEXT,
            payload TEXT,
            emitted_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS hormone_samples (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cortisol REAL, dopamine REAL, serotonin REAL,
            adrenaline REAL, oxytocin REAL, melatonin REAL,
            mood_score REAL, risk_tolerance REAL,
            sampled_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS guardian_ticks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            iteration_id TEXT NOT NULL,
            signals_detected INTEGER,
            actions_taken INTEGER,
            max_threat_level TEXT,
            duration_ms INTEGER,
            ticked_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS immunity_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            antibody_id TEXT,
            threat_type TEXT,
            confidence REAL,
            action_taken TEXT,
            content_snippet TEXT,
            detected_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS synapse_observations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            synapse_id TEXT NOT NULL,
            amplitude REAL NOT NULL,
            observation_count INTEGER,
            peak_amplitude REAL,
            status TEXT,
            observed_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS energy_samples (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            atp REAL, adp REAL, amp REAL,
            charge REAL,
            regime TEXT,
            sampled_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS transcriptase_schemas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT,
            field_count INTEGER,
            observation_count INTEGER,
            schema_json TEXT NOT NULL,
            inferred_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS ribosome_contracts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contract_id TEXT NOT NULL,
            schema_json TEXT NOT NULL,
            observation_count INTEGER DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS phenotype_mutations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            primitives TEXT NOT NULL,
            uses_unsafe INTEGER DEFAULT 0,
            is_lethal INTEGER,
            verdict TEXT,
            checked_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS organ_signals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_organ TEXT NOT NULL,
            target_organ TEXT NOT NULL,
            signal_type TEXT DEFAULT 'data',
            payload TEXT,
            latency_ms INTEGER,
            signaled_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS organ_edges (
            source TEXT NOT NULL,
            target TEXT NOT NULL,
            weight REAL DEFAULT 1.0,
            edge_type TEXT DEFAULT 'signal',
            is_active INTEGER DEFAULT 1,
            PRIMARY KEY (source, target)
        );

        -- Seed the organ registry
        INSERT OR IGNORE INTO organs (id, name) VALUES
            ('brain', 'Brain'),
            ('guardian', 'Guardian'),
            ('cytokine', 'Cytokine Bus'),
            ('hormones', 'Endocrine System'),
            ('immunity', 'Immune System'),
            ('synapse', 'Synapse Network'),
            ('energy', 'Energy Metabolism'),
            ('transcriptase', 'Reverse Transcriptase'),
            ('ribosome', 'Ribosome'),
            ('reproductive', 'Reproductive/Phenotype'),
            ('vigil', 'Vigil Orchestrator'),
            ('implicit', 'Implicit Knowledge'),
            ('hooks', 'Hook System'),
            ('skills', 'Skill Registry'),
            ('compound_patterns', 'Compound Patterns');

        -- Seed the organ edges (DAG from graph analysis)
        INSERT OR IGNORE INTO organ_edges (source, target, edge_type) VALUES
            ('hooks', 'brain', 'data'),
            ('hooks', 'compound_patterns', 'data'),
            ('hooks', 'guardian', 'signal'),
            ('brain', 'implicit', 'data'),
            ('brain', 'guardian', 'signal'),
            ('guardian', 'cytokine', 'signal'),
            ('cytokine', 'immunity', 'signal'),
            ('cytokine', 'hormones', 'signal'),
            ('synapse', 'brain', 'data'),
            ('implicit', 'skills', 'data'),
            ('transcriptase', 'ribosome', 'data'),
            ('vigil', 'cytokine', 'signal'),
            ('vigil', 'guardian', 'signal'),
            ('phenotype', 'immunity', 'signal'),
            ('immunity', 'guardian', 'feedback'),
            ('hormones', 'energy', 'feedback'),
            ('energy', 'guardian', 'feedback');
        ",
    )
    .map_err(|e| McpError::internal_error(format!("Schema init failed: {e}"), None))?;

    Ok(())
}

// ─── Query Tool ────────────────────────────────────────────────

pub fn anatomy_query(params: AnatomyQueryParams) -> Result<CallToolResult, McpError> {
    let sql = params.sql.trim();

    // Read-only enforcement
    let upper = sql.to_uppercase();
    if !upper.starts_with("SELECT")
        && !upper.starts_with("WITH")
        && !upper.starts_with("EXPLAIN")
        && !upper.starts_with("PRAGMA")
    {
        return Err(McpError::invalid_params(
            "Only SELECT, WITH, EXPLAIN, PRAGMA queries allowed",
            None,
        ));
    }

    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let limit = params.limit.unwrap_or(500).min(1000) as usize;

    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let mut obj = serde_json::Map::new();
            for (i, name) in col_names.iter().enumerate() {
                let val: rusqlite::types::Value = row.get(i)?;
                let json_val = match val {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(n) => json!(n),
                    rusqlite::types::Value::Real(f) => json!(f),
                    rusqlite::types::Value::Text(s) => json!(s),
                    rusqlite::types::Value::Blob(b) => json!(format!("<blob:{} bytes>", b.len())),
                };
                obj.insert(name.clone(), json_val);
            }
            Ok(serde_json::Value::Object(obj))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?
        .take(limit)
        .filter_map(|r| r.ok())
        .collect();

    let result = json!({
        "columns": col_names,
        "rows": rows,
        "count": rows.len(),
        "truncated": rows.len() >= limit,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

// ─── Status Tool ───────────────────────────────────────────────

pub fn anatomy_status(_params: AnatomyStatusParams) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    let organs: Vec<serde_json::Value> = conn
        .prepare("SELECT id, name, status, last_seen, activation_count, error_count, call_count FROM organs ORDER BY id")
        .map_err(|e| McpError::internal_error(e.to_string(), None))?
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "status": row.get::<_, String>(2)?,
                "last_seen": row.get::<_, Option<String>>(3)?,
                "activations": row.get::<_, i64>(4)?,
                "errors": row.get::<_, i64>(5)?,
                "calls": row.get::<_, i64>(6)?,
            }))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?
        .filter_map(|r| r.ok())
        .collect();

    // Table row counts
    let tables = [
        "cytokine_signals",
        "hormone_samples",
        "guardian_ticks",
        "immunity_events",
        "synapse_observations",
        "energy_samples",
        "transcriptase_schemas",
        "ribosome_contracts",
        "phenotype_mutations",
        "organ_signals",
    ];

    let mut table_counts = serde_json::Map::new();
    for t in tables {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |row| row.get(0))
            .unwrap_or(0);
        table_counts.insert(t.to_string(), json!(count));
    }

    let edge_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM organ_edges", [], |row| row.get(0))
        .unwrap_or(0);

    let result = json!({
        "organs": organs,
        "table_row_counts": table_counts,
        "edge_count": edge_count,
        "db_path": db_path(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

// ─── Record Tools (one per organ) ──────────────────────────────

fn update_organ(conn: &Connection, organ_id: &str, status: &str) -> Result<(), McpError> {
    conn.execute(
        "UPDATE organs SET status = ?1, last_seen = datetime('now'), activation_count = activation_count + 1, call_count = call_count + 1 WHERE id = ?2",
        rusqlite::params![status, organ_id],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(())
}

pub fn record_cytokine(params: AnatomyRecordCytokineParams) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO cytokine_signals (signal_id, family, name, severity, scope, payload) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![params.signal_id, params.family, params.name, params.severity, params.scope, params.payload],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "cytokine", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "cytokine_signals"}).to_string(),
    )]))
}

pub fn record_hormones(params: AnatomyRecordHormonesParams) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO hormone_samples (cortisol, dopamine, serotonin, adrenaline, oxytocin, melatonin, mood_score, risk_tolerance) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![params.cortisol, params.dopamine, params.serotonin, params.adrenaline, params.oxytocin, params.melatonin, params.mood_score, params.risk_tolerance],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "hormones", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "hormone_samples"}).to_string(),
    )]))
}

pub fn record_guardian_tick(
    params: AnatomyRecordGuardianTickParams,
) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO guardian_ticks (iteration_id, signals_detected, actions_taken, max_threat_level, duration_ms) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![params.iteration_id, params.signals_detected, params.actions_taken, params.max_threat_level, params.duration_ms],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "guardian", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "guardian_ticks"}).to_string(),
    )]))
}

pub fn record_immunity_event(
    params: AnatomyRecordImmunityEventParams,
) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO immunity_events (antibody_id, threat_type, confidence, action_taken, content_snippet) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![params.antibody_id, params.threat_type, params.confidence, params.action_taken, params.content_snippet],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "immunity", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "immunity_events"}).to_string(),
    )]))
}

pub fn record_synapse(params: AnatomyRecordSynapseParams) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO synapse_observations (synapse_id, amplitude, observation_count, peak_amplitude, status) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![params.synapse_id, params.amplitude, params.observation_count, params.peak_amplitude, params.status],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "synapse", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "synapse_observations"}).to_string(),
    )]))
}

pub fn record_energy(params: AnatomyRecordEnergyParams) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO energy_samples (atp, adp, amp, charge, regime) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![
            params.atp,
            params.adp,
            params.amp,
            params.charge,
            params.regime
        ],
    )
    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "energy", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "energy_samples"}).to_string(),
    )]))
}

pub fn record_transcriptase(
    params: AnatomyRecordTranscriptaseParams,
) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO transcriptase_schemas (source, field_count, observation_count, schema_json) VALUES (?1,?2,?3,?4)",
        rusqlite::params![params.source, params.field_count, params.observation_count, params.schema_json],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "transcriptase", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "transcriptase_schemas"}).to_string(),
    )]))
}

pub fn record_ribosome(params: AnatomyRecordRibosomeParams) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT OR REPLACE INTO ribosome_contracts (contract_id, schema_json, observation_count, created_at, updated_at) VALUES (?1, ?2, COALESCE((SELECT observation_count + 1 FROM ribosome_contracts WHERE contract_id = ?1), 1), COALESCE((SELECT created_at FROM ribosome_contracts WHERE contract_id = ?1), datetime('now')), datetime('now'))",
        rusqlite::params![params.contract_id, params.schema_json],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "ribosome", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "ribosome_contracts"}).to_string(),
    )]))
}

pub fn record_phenotype(params: AnatomyRecordPhenotypeParams) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO phenotype_mutations (primitives, uses_unsafe, is_lethal, verdict) VALUES (?1,?2,?3,?4)",
        rusqlite::params![params.primitives, params.uses_unsafe as i32, params.is_lethal as i32, params.verdict],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    update_organ(conn, "reproductive", "healthy")?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "phenotype_mutations"}).to_string(),
    )]))
}

pub fn record_organ_signal(
    params: AnatomyRecordOrganSignalParams,
) -> Result<CallToolResult, McpError> {
    let guard = get_conn()?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No connection", None))?;

    conn.execute(
        "INSERT INTO organ_signals (source_organ, target_organ, signal_type, payload, latency_ms) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![params.source_organ, params.target_organ, params.signal_type, params.payload, params.latency_ms],
    ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({"recorded": true, "table": "organ_signals"}).to_string(),
    )]))
}
