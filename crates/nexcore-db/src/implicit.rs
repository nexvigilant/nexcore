//! Implicit knowledge CRUD: preferences, patterns, corrections, beliefs,
//! trust accumulators, and belief implications.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{DbError, Result};

// ========== Preferences ==========

/// A preference row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceRow {
    /// Preference key
    pub key: String,
    /// JSON value
    pub value: String,
    /// Optional description
    pub description: Option<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Times reinforced
    pub reinforcement_count: u32,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Upsert a preference.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_preference(conn: &Connection, pref: &PreferenceRow) -> Result<()> {
    conn.execute(
        "INSERT INTO preferences (key, value, description, confidence, reinforcement_count, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            description = excluded.description,
            confidence = excluded.confidence,
            reinforcement_count = excluded.reinforcement_count,
            updated_at = excluded.updated_at",
        params![
            pref.key,
            pref.value,
            pref.description,
            pref.confidence,
            pref.reinforcement_count,
            pref.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get a preference by key.
///
/// # Errors
///
/// Returns `NotFound` if the key doesn't exist.
pub fn get_preference(conn: &Connection, key: &str) -> Result<PreferenceRow> {
    conn.query_row(
        "SELECT key, value, description, confidence, reinforcement_count, updated_at
         FROM preferences WHERE key = ?1",
        [key],
        |row| {
            Ok(PreferenceRow {
                key: row.get(0)?,
                value: row.get(1)?,
                description: row.get(2)?,
                confidence: row.get(3)?,
                reinforcement_count: row.get(4)?,
                updated_at: parse_dt(row.get::<_, String>(5)?),
            })
        },
    )
    .map_err(|_| DbError::NotFound(format!("preference {key}")))
}

/// List all preferences.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_preferences(conn: &Connection) -> Result<Vec<PreferenceRow>> {
    let mut stmt = conn.prepare(
        "SELECT key, value, description, confidence, reinforcement_count, updated_at
         FROM preferences ORDER BY key",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PreferenceRow {
                key: row.get(0)?,
                value: row.get(1)?,
                description: row.get(2)?,
                confidence: row.get(3)?,
                reinforcement_count: row.get(4)?,
                updated_at: parse_dt(row.get::<_, String>(5)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ========== Patterns ==========

/// A pattern row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRow {
    /// Pattern ID
    pub id: String,
    /// Pattern type (naming, structure, workflow, etc.)
    pub pattern_type: String,
    /// Description
    pub description: String,
    /// JSON array of example strings
    pub examples: String,
    /// When first detected
    pub detected_at: DateTime<Utc>,
    /// When last reinforced
    pub updated_at: DateTime<Utc>,
    /// Raw confidence before decay
    pub confidence: f64,
    /// Occurrences observed
    pub occurrence_count: u32,
    /// T1 primitive grounding (optional)
    pub t1_grounding: Option<String>,
}

/// Upsert a pattern.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_pattern(conn: &Connection, p: &PatternRow) -> Result<()> {
    conn.execute(
        "INSERT INTO patterns (id, pattern_type, description, examples, detected_at,
                               updated_at, confidence, occurrence_count, t1_grounding)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            description = excluded.description,
            examples = excluded.examples,
            updated_at = excluded.updated_at,
            confidence = excluded.confidence,
            occurrence_count = excluded.occurrence_count,
            t1_grounding = COALESCE(excluded.t1_grounding, patterns.t1_grounding)",
        params![
            p.id,
            p.pattern_type,
            p.description,
            p.examples,
            p.detected_at.to_rfc3339(),
            p.updated_at.to_rfc3339(),
            p.confidence,
            p.occurrence_count,
            p.t1_grounding,
        ],
    )?;
    Ok(())
}

/// List all patterns.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_patterns(conn: &Connection) -> Result<Vec<PatternRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, pattern_type, description, examples, detected_at,
                updated_at, confidence, occurrence_count, t1_grounding
         FROM patterns ORDER BY confidence DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PatternRow {
                id: row.get(0)?,
                pattern_type: row.get(1)?,
                description: row.get(2)?,
                examples: row.get(3)?,
                detected_at: parse_dt(row.get::<_, String>(4)?),
                updated_at: parse_dt(row.get::<_, String>(5)?),
                confidence: row.get(6)?,
                occurrence_count: row.get(7)?,
                t1_grounding: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ========== Corrections ==========

/// A correction row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionRow {
    /// Auto-increment ID (None for new)
    pub id: Option<i64>,
    /// What was wrong
    pub mistake: String,
    /// What should have been done
    pub correction: String,
    /// Context when this occurred
    pub context: Option<String>,
    /// When learned
    pub learned_at: DateTime<Utc>,
    /// Times applied
    pub application_count: u32,
}

/// Insert a correction.
///
/// # Errors
///
/// Returns an error if the insert fails.
pub fn insert_correction(conn: &Connection, c: &CorrectionRow) -> Result<()> {
    conn.execute(
        "INSERT INTO corrections (mistake, correction, context, learned_at, application_count)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            c.mistake,
            c.correction,
            c.context,
            c.learned_at.to_rfc3339(),
            c.application_count,
        ],
    )?;
    Ok(())
}

/// List all corrections.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_corrections(conn: &Connection) -> Result<Vec<CorrectionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, mistake, correction, context, learned_at, application_count
         FROM corrections ORDER BY learned_at DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CorrectionRow {
                id: Some(row.get(0)?),
                mistake: row.get(1)?,
                correction: row.get(2)?,
                context: row.get(3)?,
                learned_at: parse_dt(row.get::<_, String>(4)?),
                application_count: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ========== Beliefs (PROJECT GROUNDED) ==========

/// A belief row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefRow {
    /// Unique ID
    pub id: String,
    /// Proposition in natural language
    pub proposition: String,
    /// Category (capability, behavior, preference, constraint)
    pub category: String,
    /// Raw confidence before decay
    pub confidence: f64,
    /// JSON array of evidence references
    pub evidence: String,
    /// T1 primitive grounding
    pub t1_grounding: Option<String>,
    /// When formed
    pub formed_at: DateTime<Utc>,
    /// When last updated
    pub updated_at: DateTime<Utc>,
    /// Times validated
    pub validation_count: u32,
    /// User confirmed
    pub user_confirmed: bool,
}

/// Upsert a belief.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_belief(conn: &Connection, b: &BeliefRow) -> Result<()> {
    conn.execute(
        "INSERT INTO beliefs (id, proposition, category, confidence, evidence,
                              t1_grounding, formed_at, updated_at, validation_count, user_confirmed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            proposition = excluded.proposition,
            confidence = excluded.confidence,
            evidence = excluded.evidence,
            t1_grounding = COALESCE(excluded.t1_grounding, beliefs.t1_grounding),
            updated_at = excluded.updated_at,
            validation_count = excluded.validation_count,
            user_confirmed = excluded.user_confirmed",
        params![
            b.id,
            b.proposition,
            b.category,
            b.confidence,
            b.evidence,
            b.t1_grounding,
            b.formed_at.to_rfc3339(),
            b.updated_at.to_rfc3339(),
            b.validation_count,
            b.user_confirmed as i32,
        ],
    )?;
    Ok(())
}

/// List all beliefs.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_beliefs(conn: &Connection) -> Result<Vec<BeliefRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, proposition, category, confidence, evidence,
                t1_grounding, formed_at, updated_at, validation_count, user_confirmed
         FROM beliefs ORDER BY confidence DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BeliefRow {
                id: row.get(0)?,
                proposition: row.get(1)?,
                category: row.get(2)?,
                confidence: row.get(3)?,
                evidence: row.get(4)?,
                t1_grounding: row.get(5)?,
                formed_at: parse_dt(row.get::<_, String>(6)?),
                updated_at: parse_dt(row.get::<_, String>(7)?),
                validation_count: row.get(8)?,
                user_confirmed: row.get::<_, i32>(9)? != 0,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ========== Trust Accumulators ==========

/// A trust accumulator row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRow {
    /// Domain name
    pub domain: String,
    /// Success count
    pub demonstrations: u32,
    /// Failure count
    pub failures: u32,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last updated
    pub updated_at: DateTime<Utc>,
    /// T1 grounding
    pub t1_grounding: Option<String>,
}

/// Upsert a trust accumulator.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_trust(conn: &Connection, t: &TrustRow) -> Result<()> {
    conn.execute(
        "INSERT INTO trust_accumulators (domain, demonstrations, failures, created_at, updated_at, t1_grounding)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(domain) DO UPDATE SET
            demonstrations = excluded.demonstrations,
            failures = excluded.failures,
            updated_at = excluded.updated_at,
            t1_grounding = COALESCE(excluded.t1_grounding, trust_accumulators.t1_grounding)",
        params![
            t.domain,
            t.demonstrations,
            t.failures,
            t.created_at.to_rfc3339(),
            t.updated_at.to_rfc3339(),
            t.t1_grounding,
        ],
    )?;
    Ok(())
}

/// List all trust accumulators.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_trust(conn: &Connection) -> Result<Vec<TrustRow>> {
    let mut stmt = conn.prepare(
        "SELECT domain, demonstrations, failures, created_at, updated_at, t1_grounding
         FROM trust_accumulators ORDER BY domain",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TrustRow {
                domain: row.get(0)?,
                demonstrations: row.get(1)?,
                failures: row.get(2)?,
                created_at: parse_dt(row.get::<_, String>(3)?),
                updated_at: parse_dt(row.get::<_, String>(4)?),
                t1_grounding: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ========== Belief Implications ==========

/// A belief implication edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplicationRow {
    /// Source belief ID
    pub from_belief: String,
    /// Target belief ID
    pub to_belief: String,
    /// Strength: strong, moderate, weak
    pub strength: String,
    /// When established
    pub established_at: DateTime<Utc>,
}

/// Insert a belief implication.
///
/// # Errors
///
/// Returns an error if the insert fails.
pub fn insert_implication(conn: &Connection, imp: &ImplicationRow) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO belief_implications (from_belief, to_belief, strength, established_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            imp.from_belief,
            imp.to_belief,
            imp.strength,
            imp.established_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// List all implications.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_implications(conn: &Connection) -> Result<Vec<ImplicationRow>> {
    let mut stmt = conn.prepare(
        "SELECT from_belief, to_belief, strength, established_at
         FROM belief_implications ORDER BY established_at ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ImplicationRow {
                from_belief: row.get(0)?,
                to_belief: row.get(1)?,
                strength: row.get(2)?,
                established_at: parse_dt(row.get::<_, String>(3)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Parse an RFC3339 datetime string.
fn parse_dt(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    fn pool() -> DbPool {
        DbPool::open_in_memory().expect("open")
    }

    #[test]
    fn test_preference_crud() {
        let db = pool();
        db.with_conn(|conn| {
            let pref = PreferenceRow {
                key: "code_style".into(),
                value: r#""functional""#.into(),
                description: Some("Prefers functional style".into()),
                confidence: 0.7,
                reinforcement_count: 3,
                updated_at: Utc::now(),
            };
            upsert_preference(conn, &pref)?;

            let loaded = get_preference(conn, "code_style")?;
            assert_eq!(loaded.key, "code_style");
            assert!((loaded.confidence - 0.7).abs() < f64::EPSILON);

            let all = list_preferences(conn)?;
            assert_eq!(all.len(), 1);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_pattern_crud() {
        let db = pool();
        db.with_conn(|conn| {
            let p = PatternRow {
                id: "snake_case".into(),
                pattern_type: "naming".into(),
                description: "Uses snake_case".into(),
                examples: r#"["fn_name", "var_name"]"#.into(),
                detected_at: Utc::now(),
                updated_at: Utc::now(),
                confidence: 0.8,
                occurrence_count: 5,
                t1_grounding: Some("mapping".into()),
            };
            upsert_pattern(conn, &p)?;

            let all = list_patterns(conn)?;
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].t1_grounding.as_deref(), Some("mapping"));
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_correction_crud() {
        let db = pool();
        db.with_conn(|conn| {
            insert_correction(
                conn,
                &CorrectionRow {
                    id: None,
                    mistake: "used unwrap".into(),
                    correction: "use expect with message".into(),
                    context: Some("error handling".into()),
                    learned_at: Utc::now(),
                    application_count: 0,
                },
            )?;
            let all = list_corrections(conn)?;
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].mistake, "used unwrap");
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_belief_crud() {
        let db = pool();
        db.with_conn(|conn| {
            let b = BeliefRow {
                id: "rust-safe".into(),
                proposition: "Rust prevents memory bugs".into(),
                category: "capability".into(),
                confidence: 0.9,
                evidence: "[]".into(),
                t1_grounding: Some("boundary".into()),
                formed_at: Utc::now(),
                updated_at: Utc::now(),
                validation_count: 3,
                user_confirmed: true,
            };
            upsert_belief(conn, &b)?;

            let all = list_beliefs(conn)?;
            assert_eq!(all.len(), 1);
            assert!(all[0].user_confirmed);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_trust_crud() {
        let db = pool();
        db.with_conn(|conn| {
            upsert_trust(
                conn,
                &TrustRow {
                    domain: "rust_dev".into(),
                    demonstrations: 10,
                    failures: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    t1_grounding: None,
                },
            )?;
            let all = list_trust(conn)?;
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].demonstrations, 10);
            Ok(())
        })
        .expect("test");
    }
}
