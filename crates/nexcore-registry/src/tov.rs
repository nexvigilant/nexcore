//! Theory of Vigilance monitoring for the skill ecosystem.
//!
//! Maps the 5 ToV axioms and 8 harm types (A-H) to measurable
//! skill ecosystem health metrics. Provides the safety distance d(s)
//! computation required by Axiom 4 (Safety Manifold).

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Harm type indicator counts (A-H).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmReport {
    /// A: Direct — skills without allowed_tools that can execute arbitrary tools
    pub harm_a_count: i64,
    /// B: Facilitated — skills that should restrict model invocation but don't
    pub harm_b_count: i64,
    /// C: Informational — skills without description (opaque to users)
    pub harm_c_count: i64,
    /// D: Systemic — skills exceeding 16KB budget (context pollution)
    pub harm_d_count: i64,
    /// E: Delayed — skills not scanned in > 30 days
    pub harm_e_count: i64,
    /// F: Cascading — skills with chain_position but broken pipeline links
    pub harm_f_count: i64,
    /// G: Amplified — skills with context:fork but no tool restrictions
    pub harm_g_count: i64,
    /// H: Emergent — skills with line_count > 400 AND no sub-skill decomposition
    pub harm_h_count: i64,
    /// Weighted composite harm score
    pub total_harm_score: f64,
}

/// Compute the ToV safety distance d(s) for the ecosystem.
///
/// d(s) = min(diamond_compliance%, budget_compliance%) / 100
///
/// Axiom 4 requires d(s) > 0 always.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn safety_distance(conn: &Connection) -> Result<f64> {
    let total: f64 = conn.query_row(
        "SELECT CAST(COUNT(*) AS REAL) FROM active_skills WHERE parent_skill IS NULL",
        [],
        |r| r.get(0),
    )?;

    if total < 1.0 {
        return Ok(0.0);
    }

    let diamond_count: f64 = conn
        .query_row(
            "SELECT CAST(COUNT(*) AS REAL) FROM active_skills WHERE compliance = 'Diamond' AND parent_skill IS NULL",
            [],
            |r| r.get(0),
        )?;

    let budget_ok_count: f64 = conn
        .query_row(
            "SELECT CAST(COUNT(*) AS REAL) FROM active_skills WHERE (content_chars IS NULL OR content_chars <= 16384) AND parent_skill IS NULL",
            [],
            |r| r.get(0),
        )?;

    let diamond_pct = diamond_count / total * 100.0;
    let budget_pct = budget_ok_count / total * 100.0;

    // d(s) = min of the two percentages, scaled to 0-1
    let ds = if diamond_pct < budget_pct {
        diamond_pct
    } else {
        budget_pct
    } / 100.0;

    Ok(ds)
}

/// Compute all 8 harm type indicators.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn harm_indicators(conn: &Connection) -> Result<HarmReport> {
    // A: Unguarded — no allowed_tools
    let harm_a: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE allowed_tools IS NULL AND parent_skill IS NULL",
        [],
        |r| r.get(0),
    )?;

    // B: No disable — context:fork skills without disable_model_invocation
    let harm_b: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE context = 'fork' AND disable_model_invocation = 0",
        [],
        |r| r.get(0),
    )?;

    // C: No description
    let harm_c: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE description IS NULL",
        [],
        |r| r.get(0),
    )?;

    // D: Over budget (> 16384 chars)
    let harm_d: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE content_chars > 16384",
        [],
        |r| r.get(0),
    )?;

    // E: Stale (> 30 days since scan)
    let harm_e: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE scanned_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-30 days')",
        [],
        |r| r.get(0),
    )?;

    // F: Broken chains — has chain_position but no pipeline
    let harm_f: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE chain_position IS NOT NULL AND pipeline IS NULL",
        [],
        |r| r.get(0),
    )?;

    // G: Unrestricted fork — context:fork but no allowed_tools
    let harm_g: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE context = 'fork' AND allowed_tools IS NULL",
        [],
        |r| r.get(0),
    )?;

    // H: Excess complexity — line_count > 400 AND no sub-skills
    let harm_h: i64 = conn.query_row(
        "SELECT COUNT(*) FROM active_skills WHERE line_count > 400 AND sub_skill_count = 0 AND parent_skill IS NULL",
        [],
        |r| r.get(0),
    )?;

    // Weighted composite: higher weights for more severe harm types
    let total_harm_score = f64::from(harm_a as i32)           // A: weight 1.0
        + f64::from(harm_b as i32).mul_add(2.0, 0.0)          // B: weight 2.0
        + f64::from(harm_c as i32).mul_add(1.5, 0.0)          // C: weight 1.5
        + f64::from(harm_d as i32).mul_add(2.0, 0.0)          // D: weight 2.0
        + f64::from(harm_e as i32).mul_add(0.5, 0.0)          // E: weight 0.5
        + f64::from(harm_f as i32).mul_add(1.5, 0.0)          // F: weight 1.5
        + f64::from(harm_g as i32).mul_add(3.0, 0.0)          // G: weight 3.0
        + f64::from(harm_h as i32); // H: weight 1.0

    Ok(HarmReport {
        harm_a_count: harm_a,
        harm_b_count: harm_b,
        harm_c_count: harm_c,
        harm_d_count: harm_d,
        harm_e_count: harm_e,
        harm_f_count: harm_f,
        harm_g_count: harm_g,
        harm_h_count: harm_h,
        total_harm_score,
    })
}

/// Check if the ecosystem is in the safe manifold (d(s) > threshold).
///
/// # Errors
///
/// Returns an error on query failure.
pub fn is_safe(conn: &Connection, threshold: f64) -> Result<bool> {
    let ds = safety_distance(conn)?;
    Ok(ds > threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::pool::RegistryPool;
    use crate::skills::{self, SkillRow};

    fn make_skill(name: &str) -> SkillRow {
        let now = Utc::now();
        SkillRow {
            name: name.to_string(),
            path: format!("/skills/{name}/SKILL.md"),
            description: Some("Test skill".to_string()),
            argument_hint: None,
            disable_model_invocation: false,
            user_invocable: true,
            allowed_tools: None,
            model: None,
            context: None,
            agent: None,
            hooks: None,
            line_count: Some(100),
            has_agent: false,
            sub_skill_count: 0,
            parent_skill: None,
            uses_arguments: false,
            uses_dynamic_context: false,
            uses_session_id: false,
            content_chars: Some(500),
            smst_input: None,
            smst_output: None,
            smst_logic: None,
            smst_error_handling: None,
            smst_examples: None,
            smst_references: None,
            last_assessed_at: None,
            assessed_by: None,
            version: None,
            compliance: None,
            smst_v1: None,
            smst_v2: None,
            tags: None,
            chain_position: None,
            pipeline: None,
            scanned_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_safety_distance_empty() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let ds = safety_distance(conn)?;
            assert!((ds - 0.0).abs() < f64::EPSILON);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_safety_distance_with_skills() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            skills::upsert(conn, &make_skill("s1"))?;
            let ds = safety_distance(conn)?;
            // No Diamond skills, so d(s) = min(0%, 100%) / 100 = 0.0
            assert!((ds - 0.0).abs() < f64::EPSILON);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_harm_indicators() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            // Skill with no allowed_tools, no description
            let mut s = make_skill("harmful");
            s.description = None;
            s.content_chars = Some(20000); // over budget
            skills::upsert(conn, &s)?;

            let report = harm_indicators(conn)?;
            assert_eq!(report.harm_a_count, 1); // no allowed_tools
            assert_eq!(report.harm_c_count, 1); // no description
            assert_eq!(report.harm_d_count, 1); // over budget
            assert!(report.total_harm_score > 0.0);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_is_safe() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            // Empty DB: d(s) = 0, threshold 0.3 → not safe
            let safe = is_safe(conn, 0.3)?;
            assert!(!safe);
            // Threshold 0 → safe (d(s) = 0 is not > 0)
            let safe_zero = is_safe(conn, -0.1)?;
            assert!(safe_zero);
            Ok(())
        })
        .ok();
    }
}
