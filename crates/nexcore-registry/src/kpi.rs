//! KPI computation from live registry data.
//!
//! Computes ecosystem-level KPIs from `active_skills` and `active_agents`
//! and persists results into `skill_kpis`.

use nexcore_chrono::DateTime;
use rusqlite::Connection;

use crate::error::Result;
use crate::goals::{self, KpiRow};

/// All known KPI definitions.
const KPI_DEFS: &[KpiDef] = &[
    KpiDef {
        name: "total_skills",
        description: "Total skills in ecosystem",
        formula: "SELECT COUNT(*) FROM active_skills",
        target: None,
        unit: "count",
        direction: "higher_better",
    },
    KpiDef {
        name: "top_level_skills",
        description: "Top-level skills (not sub-skills)",
        formula: "SELECT COUNT(*) FROM active_skills WHERE parent_skill IS NULL",
        target: None,
        unit: "count",
        direction: "higher_better",
    },
    KpiDef {
        name: "agent_coverage",
        description: "Percentage of top-level skills with agent pairing",
        formula: "SELECT ROUND(100.0 * SUM(has_agent) / MAX(COUNT(*), 1), 1) FROM active_skills WHERE parent_skill IS NULL",
        target: Some(100.0),
        unit: "percent",
        direction: "higher_better",
    },
    KpiDef {
        name: "diamond_compliance",
        description: "Percentage of skills at Diamond compliance",
        formula: "SELECT ROUND(100.0 * SUM(CASE WHEN compliance = 'Diamond' THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 1) FROM active_skills WHERE parent_skill IS NULL",
        target: Some(80.0),
        unit: "percent",
        direction: "higher_better",
    },
    KpiDef {
        name: "avg_line_count",
        description: "Average SKILL.md line count",
        formula: "SELECT ROUND(AVG(line_count), 0) FROM active_skills",
        target: Some(350.0),
        unit: "lines",
        direction: "lower_better",
    },
    KpiDef {
        name: "over_limit_count",
        description: "Skills over 500-line limit",
        formula: "SELECT COUNT(*) FROM active_skills WHERE line_count > 500",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "user_invocable_count",
        description: "Skills with user-invocable: true",
        formula: "SELECT COUNT(*) FROM active_skills WHERE user_invocable = 1",
        target: None,
        unit: "count",
        direction: "higher_better",
    },
    // --- ToV Axiom 1: Decomposition ---
    KpiDef {
        name: "tov_decomposition_depth",
        description: "Avg sub-skill count for top-level skills with subs",
        formula: "SELECT ROUND(AVG(sub_skill_count), 1) FROM active_skills WHERE parent_skill IS NULL AND sub_skill_count > 0",
        target: Some(2.0),
        unit: "ratio",
        direction: "higher_better",
    },
    KpiDef {
        name: "tov_primitive_coverage",
        description: "Pct of skills with tags containing T1 primitive names",
        formula: "SELECT ROUND(100.0 * SUM(CASE WHEN tags IS NOT NULL THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 1) FROM active_skills WHERE parent_skill IS NULL",
        target: Some(50.0),
        unit: "percent",
        direction: "higher_better",
    },
    // --- ToV Axiom 2: Hierarchy ---
    KpiDef {
        name: "tov_layer_compliance",
        description: "Pct of skills with pipeline field populated",
        formula: "SELECT ROUND(100.0 * SUM(CASE WHEN pipeline IS NOT NULL THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 1) FROM active_skills WHERE parent_skill IS NULL",
        target: Some(80.0),
        unit: "percent",
        direction: "higher_better",
    },
    KpiDef {
        name: "tov_parent_child_ratio",
        description: "Ratio of sub-skills to top-level skills",
        formula: "SELECT ROUND(CAST(SUM(CASE WHEN parent_skill IS NOT NULL THEN 1 ELSE 0 END) AS REAL) / MAX(SUM(CASE WHEN parent_skill IS NULL THEN 1 ELSE 0 END), 1), 2) FROM active_skills",
        target: Some(2.0),
        unit: "ratio",
        direction: "target",
    },
    // --- ToV Axiom 3: Conservation ---
    KpiDef {
        name: "tov_audit_coverage",
        description: "Pct of skills with audit entry in last 7 days",
        formula: "SELECT ROUND(100.0 * COUNT(DISTINCT a.skill_name) / MAX((SELECT COUNT(*) FROM active_skills WHERE parent_skill IS NULL), 1), 1) FROM audit_log a WHERE a.created_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-7 days')",
        target: Some(90.0),
        unit: "percent",
        direction: "higher_better",
    },
    KpiDef {
        name: "tov_version_tracking",
        description: "Pct of skills with version field populated",
        formula: "SELECT ROUND(100.0 * SUM(CASE WHEN version IS NOT NULL THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 1) FROM active_skills",
        target: Some(100.0),
        unit: "percent",
        direction: "higher_better",
    },
    // --- ToV Axiom 4: Safety Manifold ---
    KpiDef {
        name: "tov_safety_distance",
        description: "d(s) = min(diamond%, budget%) / 100",
        formula: "SELECT ROUND(MIN(100.0 * SUM(CASE WHEN compliance = 'Diamond' THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 100.0 * SUM(CASE WHEN content_chars IS NULL OR content_chars <= 16384 THEN 1 ELSE 0 END) / MAX(COUNT(*), 1)) / 100.0, 3) FROM active_skills WHERE parent_skill IS NULL",
        target: Some(0.3),
        unit: "ratio",
        direction: "higher_better",
    },
    KpiDef {
        name: "tov_boundary_enforcement",
        description: "Pct of user-invocable skills with allowed_tools",
        formula: "SELECT ROUND(100.0 * SUM(CASE WHEN allowed_tools IS NOT NULL THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 1) FROM active_skills WHERE user_invocable = 1 AND parent_skill IS NULL",
        target: Some(80.0),
        unit: "percent",
        direction: "higher_better",
    },
    // --- ToV Axiom 5: Emergence ---
    KpiDef {
        name: "tov_chain_utilization",
        description: "Pct of skills with chain_position populated",
        formula: "SELECT ROUND(100.0 * SUM(CASE WHEN chain_position IS NOT NULL THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 1) FROM active_skills WHERE parent_skill IS NULL",
        target: Some(30.0),
        unit: "percent",
        direction: "higher_better",
    },
    KpiDef {
        name: "tov_feature_adoption",
        description: "Pct of skills using >= 3 Anthropic features",
        formula: "SELECT ROUND(100.0 * SUM(CASE WHEN (CASE WHEN description IS NOT NULL THEN 1 ELSE 0 END + CASE WHEN allowed_tools IS NOT NULL THEN 1 ELSE 0 END + CASE WHEN hooks IS NOT NULL THEN 1 ELSE 0 END + CASE WHEN model IS NOT NULL THEN 1 ELSE 0 END + CASE WHEN argument_hint IS NOT NULL THEN 1 ELSE 0 END + CASE WHEN context IS NOT NULL THEN 1 ELSE 0 END) >= 3 THEN 1 ELSE 0 END) / MAX(COUNT(*), 1), 1) FROM active_skills WHERE parent_skill IS NULL",
        target: Some(60.0),
        unit: "percent",
        direction: "higher_better",
    },
    // --- Harm Type Monitoring (A-H) ---
    KpiDef {
        name: "harm_a_unguarded",
        description: "Skills without allowed_tools (direct harm risk)",
        formula: "SELECT COUNT(*) FROM active_skills WHERE allowed_tools IS NULL AND parent_skill IS NULL",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "harm_b_no_disable",
        description: "Fork skills without disable_model_invocation",
        formula: "SELECT COUNT(*) FROM active_skills WHERE context = 'fork' AND disable_model_invocation = 0",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "harm_c_no_description",
        description: "Skills without description (opaque to users)",
        formula: "SELECT COUNT(*) FROM active_skills WHERE description IS NULL",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "harm_d_over_budget",
        description: "Skills exceeding 16KB context budget",
        formula: "SELECT COUNT(*) FROM active_skills WHERE content_chars > 16384",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "harm_e_stale",
        description: "Skills not scanned in > 30 days",
        formula: "SELECT COUNT(*) FROM active_skills WHERE scanned_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-30 days')",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "harm_f_broken_chains",
        description: "Skills with chain_position but no pipeline",
        formula: "SELECT COUNT(*) FROM active_skills WHERE chain_position IS NOT NULL AND pipeline IS NULL",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "harm_g_unrestricted_fork",
        description: "Fork skills without allowed_tools restriction",
        formula: "SELECT COUNT(*) FROM active_skills WHERE context = 'fork' AND allowed_tools IS NULL",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
    KpiDef {
        name: "harm_h_complexity",
        description: "Skills > 400 lines without sub-skill decomposition",
        formula: "SELECT COUNT(*) FROM active_skills WHERE line_count > 400 AND sub_skill_count = 0 AND parent_skill IS NULL",
        target: Some(0.0),
        unit: "count",
        direction: "lower_better",
    },
];

/// Internal KPI definition.
struct KpiDef {
    name: &'static str,
    description: &'static str,
    formula: &'static str,
    target: Option<f64>,
    unit: &'static str,
    direction: &'static str,
}

/// Compute all KPIs from live data and persist them.
///
/// # Errors
///
/// Returns an error on query or upsert failure.
pub fn compute_all_kpis(conn: &Connection) -> Result<Vec<KpiRow>> {
    let now = DateTime::now();
    let mut results = Vec::with_capacity(KPI_DEFS.len());

    for def in KPI_DEFS {
        let value: Option<f64> = conn.query_row(def.formula, [], |row| row.get(0)).ok();

        let kpi = KpiRow {
            name: def.name.to_string(),
            description: def.description.to_string(),
            formula: Some(def.formula.to_string()),
            current_value: value,
            target_value: def.target,
            unit: Some(def.unit.to_string()),
            direction: Some(def.direction.to_string()),
            updated_at: now,
        };

        goals::upsert_kpi(conn, &kpi)?;
        results.push(kpi);
    }

    Ok(results)
}

/// Compute a single KPI by name.
///
/// # Errors
///
/// Returns an error if the KPI name is unknown or query fails.
pub fn compute_kpi(conn: &Connection, name: &str) -> Result<KpiRow> {
    let def = KPI_DEFS
        .iter()
        .find(|d| d.name == name)
        .ok_or_else(|| crate::error::RegistryError::NotFound(format!("kpi definition {name}")))?;

    let now = DateTime::now();
    let value: Option<f64> = conn.query_row(def.formula, [], |row| row.get(0)).ok();

    let kpi = KpiRow {
        name: def.name.to_string(),
        description: def.description.to_string(),
        formula: Some(def.formula.to_string()),
        current_value: value,
        target_value: def.target,
        unit: Some(def.unit.to_string()),
        direction: Some(def.direction.to_string()),
        updated_at: now,
    };

    goals::upsert_kpi(conn, &kpi)?;
    Ok(kpi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RegistryPool;
    use crate::skills::{self, SkillRow};

    fn seed_skills(conn: &Connection) {
        let now = DateTime::now();
        for i in 0..5 {
            let row = SkillRow {
                name: format!("skill-{i}"),
                path: format!("/skills/skill-{i}/SKILL.md"),
                description: Some(format!("Skill {i}")),
                argument_hint: None,
                disable_model_invocation: false,
                user_invocable: i % 2 == 0,
                allowed_tools: None,
                model: None,
                context: None,
                agent: None,
                hooks: None,
                line_count: Some(100 + i * 150),
                has_agent: i == 0,
                sub_skill_count: 0,
                parent_skill: None,
                uses_arguments: i % 3 == 0,
                uses_dynamic_context: false,
                uses_session_id: false,
                content_chars: Some(500 + i * 100),
                smst_input: None,
                smst_output: None,
                smst_logic: None,
                smst_error_handling: None,
                smst_examples: None,
                smst_references: None,
                last_assessed_at: None,
                assessed_by: None,
                version: None,
                compliance: if i == 0 {
                    Some("Diamond".to_string())
                } else {
                    None
                },
                smst_v1: None,
                smst_v2: None,
                tags: None,
                chain_position: None,
                pipeline: None,
                scanned_at: now,
                updated_at: now,
            };
            skills::upsert(conn, &row).ok();
        }
    }

    #[test]
    fn test_compute_all_kpis() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            seed_skills(conn);
            let kpis = compute_all_kpis(conn)?;
            assert_eq!(kpis.len(), 25);

            // total_skills should be 5
            let total = kpis.iter().find(|k| k.name == "total_skills");
            assert!(total.is_some());
            assert!(
                (total.map(|t| t.current_value.unwrap_or(0.0)).unwrap_or(0.0) - 5.0).abs()
                    < f64::EPSILON
            );

            // user_invocable_count should be 3 (0, 2, 4)
            let ui = kpis.iter().find(|k| k.name == "user_invocable_count");
            assert!(ui.is_some());
            assert!(
                (ui.map(|t| t.current_value.unwrap_or(0.0)).unwrap_or(0.0) - 3.0).abs()
                    < f64::EPSILON
            );

            // over_limit_count: line_counts are 100,250,400,550,700 — 2 are > 500
            let over = kpis.iter().find(|k| k.name == "over_limit_count");
            assert!(over.is_some());
            assert!(
                (over.map(|t| t.current_value.unwrap_or(0.0)).unwrap_or(0.0) - 2.0).abs()
                    < f64::EPSILON
            );

            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_compute_single_kpi() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            seed_skills(conn);
            let kpi = compute_kpi(conn, "total_skills")?;
            assert!((kpi.current_value.unwrap_or(0.0) - 5.0).abs() < f64::EPSILON);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_compute_unknown_kpi() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let result = compute_kpi(conn, "nonexistent");
            assert!(result.is_err());
            Ok(())
        })
        .ok();
    }
}
