//! Tier promotion automation for skill compliance acceleration.
//!
//! Identifies skills eligible for promotion to the next compliance tier
//! and generates actionable fix plans to close gaps.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::assess::{self, ComplianceTier};
use crate::error::Result;
use crate::skills;

/// An action needed to promote a skill to a higher tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromotionAction {
    /// Add a missing frontmatter field
    AddFrontmatterField {
        /// Field key (e.g., "tags", "version")
        key: String,
        /// Suggested value
        suggested_value: String,
    },
    /// Reduce line count to meet tier threshold
    ReduceLineCount {
        /// Current line count
        current: i32,
        /// Target line count
        target: i32,
    },
    /// Reduce content chars to meet budget
    ReduceContentChars {
        /// Current chars
        current: i32,
        /// Target chars
        target: i32,
    },
    /// Add a paired agent file
    AddAgentPairing,
    /// Add hooks definition to frontmatter
    AddHooksDefinition,
    /// Add allowed-tools restriction
    AddAllowedTools {
        /// Suggested tool list
        suggested: Vec<String>,
    },
}

/// A promotion plan for a single skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionPlan {
    /// Skill name
    pub name: String,
    /// Current compliance tier
    pub current_tier: String,
    /// Target compliance tier
    pub target_tier: String,
    /// Actions needed to reach target
    pub actions: Vec<PromotionAction>,
}

/// Get skill names that are eligible (close to) promotion to the given tier.
///
/// A skill is "promotable" if it is one tier below `target` and has
/// few enough gaps that promotion is achievable.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn promotable_to(conn: &Connection, target: ComplianceTier) -> Result<Vec<String>> {
    let all = skills::list_all(conn)?;
    let mut names = Vec::new();

    let required_current = match target {
        ComplianceTier::Invalid => return Ok(Vec::new()),
        ComplianceTier::Bronze => ComplianceTier::Invalid,
        ComplianceTier::Silver => ComplianceTier::Bronze,
        ComplianceTier::Gold => ComplianceTier::Silver,
        ComplianceTier::Platinum => ComplianceTier::Gold,
        ComplianceTier::Diamond => ComplianceTier::Platinum,
    };

    for row in &all {
        let assessment = assess::assess_row(row);
        if assessment.compliance == required_current {
            // Count gaps specific to the target tier
            let tier_gaps: usize = assessment
                .gaps
                .iter()
                .filter(|g| g.target_tier == target.as_str())
                .count();
            // Promotable if 3 or fewer gaps to close
            if tier_gaps <= 3 {
                names.push(row.name.clone());
            }
        }
    }

    Ok(names)
}

/// Generate a promotion plan for a skill to reach the target tier.
///
/// # Errors
///
/// Returns an error if the skill is not found.
pub fn promotion_plan(
    conn: &Connection,
    name: &str,
    target: ComplianceTier,
) -> Result<PromotionPlan> {
    let row = skills::get(conn, name)?;
    let assessment = assess::assess_row(&row);

    let mut actions = Vec::new();

    // Walk through each tier requirement up to target
    if target >= ComplianceTier::Bronze
        && assessment.compliance < ComplianceTier::Bronze
        && row.description.is_none()
    {
        actions.push(PromotionAction::AddFrontmatterField {
            key: "description".to_string(),
            suggested_value: format!("Use when working with {name} tasks"),
        });
    }

    if target >= ComplianceTier::Silver && assessment.compliance < ComplianceTier::Silver {
        if row.tags.is_none() {
            let parts: Vec<&str> = name.split('/').next().unwrap_or(name).split('-').collect();
            let tags = parts.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
            actions.push(PromotionAction::AddFrontmatterField {
                key: "tags".to_string(),
                suggested_value: format!("[{tags}]"),
            });
        }
        if row.version.is_none() {
            actions.push(PromotionAction::AddFrontmatterField {
                key: "version".to_string(),
                suggested_value: "1.0.0".to_string(),
            });
        }
        if row.line_count.is_some_and(|lc| lc > 500) {
            actions.push(PromotionAction::ReduceLineCount {
                current: row.line_count.unwrap_or(0),
                target: 500,
            });
        }
    }

    if target >= ComplianceTier::Gold && assessment.compliance < ComplianceTier::Gold {
        if row.user_invocable && row.argument_hint.is_none() {
            actions.push(PromotionAction::AddFrontmatterField {
                key: "argument-hint".to_string(),
                suggested_value: "[args]".to_string(),
            });
        }
        if row.allowed_tools.is_none() {
            actions.push(PromotionAction::AddAllowedTools {
                suggested: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
            });
        }
        if row.content_chars.is_some_and(|c| c > 16384) {
            actions.push(PromotionAction::ReduceContentChars {
                current: row.content_chars.unwrap_or(0),
                target: 16384,
            });
        }
    }

    if target >= ComplianceTier::Platinum
        && assessment.compliance < ComplianceTier::Platinum
        && !row.has_agent
    {
        actions.push(PromotionAction::AddAgentPairing);
    }

    if target >= ComplianceTier::Diamond && assessment.compliance < ComplianceTier::Diamond {
        if row.hooks.is_none() {
            actions.push(PromotionAction::AddHooksDefinition);
        }
        if row.model.is_none() {
            actions.push(PromotionAction::AddFrontmatterField {
                key: "model".to_string(),
                suggested_value: "sonnet".to_string(),
            });
        }
    }

    Ok(PromotionPlan {
        name: name.to_string(),
        current_tier: assessment.compliance.as_str().to_string(),
        target_tier: target.as_str().to_string(),
        actions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::pool::RegistryPool;
    use crate::skills::SkillRow;

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
    fn test_promotable_to_silver() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            // Bronze skill (has description but no tags/version)
            skills::upsert(conn, &make_skill("bronze-skill"))?;
            let promotable = promotable_to(conn, ComplianceTier::Silver)?;
            assert!(promotable.contains(&"bronze-skill".to_string()));
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_promotion_plan() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            skills::upsert(conn, &make_skill("plan-test"))?;
            let plan = promotion_plan(conn, "plan-test", ComplianceTier::Silver)?;
            assert_eq!(plan.current_tier, "Bronze");
            assert_eq!(plan.target_tier, "Silver");
            assert!(!plan.actions.is_empty());
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_promotion_plan_to_diamond() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            skills::upsert(conn, &make_skill("diamond-path"))?;
            let plan = promotion_plan(conn, "diamond-path", ComplianceTier::Diamond)?;
            // Should have many actions to reach Diamond from Bronze
            assert!(plan.actions.len() >= 3);
            Ok(())
        })
        .ok();
    }
}
