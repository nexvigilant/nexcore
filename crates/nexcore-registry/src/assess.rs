//! Automated skill assessment engine.
//!
//! Computes compliance tier and SMST v2 score from database fields.
//! Pure Rust, deterministic, no LLM — assessment is a function of metadata.

use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::skills::{self, SkillRow};

/// Compliance tier (ordered from lowest to highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplianceTier {
    /// Missing description or path
    Invalid,
    /// Has description + path + scanned
    Bronze,
    /// Bronze + tags + version + line_count <= 500
    Silver,
    /// Silver + argument_hint (if user_invocable) + allowed_tools + content <= 16384
    Gold,
    /// Gold + SMST v2 >= 70 + has_agent pairing
    Platinum,
    /// Platinum + SMST v2 >= 85 + hooks + all Anthropic fields populated
    Diamond,
}

impl ComplianceTier {
    /// Convert to the string stored in the database.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Invalid => "Invalid",
            Self::Bronze => "Bronze",
            Self::Silver => "Silver",
            Self::Gold => "Gold",
            Self::Platinum => "Platinum",
            Self::Diamond => "Diamond",
        }
    }

    /// Parse from database string.
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "Invalid" => Some(Self::Invalid),
            "Bronze" => Some(Self::Bronze),
            "Silver" => Some(Self::Silver),
            "Gold" => Some(Self::Gold),
            "Platinum" => Some(Self::Platinum),
            "Diamond" => Some(Self::Diamond),
            _ => None,
        }
    }
}

/// Per-component SMST v2 score breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmstComponents {
    /// 0-15: has argument_hint, uses_arguments, description quality
    pub input: i32,
    /// 0-15: has description of output behavior
    pub output: i32,
    /// 0-25: line_count reasonable, no over-budget
    pub logic: i32,
    /// 0-20: has boundary conditions documented
    pub error_handling: i32,
    /// 0-15: has usage examples in content
    pub examples: i32,
    /// 0-10: has tags, version, pipeline
    pub references: i32,
}

impl SmstComponents {
    /// Total SMST v2 score (0-100).
    #[must_use]
    pub fn total(&self) -> i32 {
        self.input + self.output + self.logic + self.error_handling + self.examples + self.references
    }
}

/// What's missing for the next compliance tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceGap {
    /// Which tier this gap prevents reaching
    pub target_tier: String,
    /// What's missing
    pub field: String,
    /// Human-readable description
    pub reason: String,
}

/// Full assessment result for a single skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    /// Skill name
    pub name: String,
    /// Computed compliance tier
    pub compliance: ComplianceTier,
    /// SMST v2 total score (0-100)
    pub smst_v2: i32,
    /// Per-component breakdown
    pub components: SmstComponents,
    /// Gaps preventing higher tiers
    pub gaps: Vec<ComplianceGap>,
}

/// Assess a single skill from its database row.
#[must_use]
pub fn assess_row(row: &SkillRow) -> Assessment {
    let components = compute_smst(row);
    let smst_v2 = components.total();
    let gaps = compute_gaps(row, smst_v2);
    let compliance = derive_tier(row, smst_v2);

    Assessment {
        name: row.name.clone(),
        compliance,
        smst_v2,
        components,
        gaps,
    }
}

/// Assess a single skill by name from the database.
///
/// # Errors
///
/// Returns an error if the skill is not found.
pub fn assess_skill(conn: &Connection, name: &str) -> Result<Assessment> {
    let row = skills::get(conn, name)?;
    Ok(assess_row(&row))
}

/// Assess all skills in the database.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn assess_all(conn: &Connection) -> Result<Vec<Assessment>> {
    let rows = skills::list_all(conn)?;
    Ok(rows.iter().map(assess_row).collect())
}

/// Write assessment results back to the database.
///
/// Updates `compliance`, `smst_v2`, SMST component columns,
/// `last_assessed_at`, and `assessed_by` for each skill.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn apply_assessments(conn: &Connection, assessments: &[Assessment]) -> Result<usize> {
    let now = Utc::now().to_rfc3339();
    let mut applied = 0usize;

    for a in assessments {
        let changed = conn.execute(
            "UPDATE active_skills SET
                compliance = ?1, smst_v2 = ?2,
                smst_input = ?3, smst_output = ?4, smst_logic = ?5,
                smst_error_handling = ?6, smst_examples = ?7, smst_references = ?8,
                last_assessed_at = ?9, assessed_by = ?10,
                updated_at = ?9
             WHERE name = ?11",
            params![
                a.compliance.as_str(),
                a.smst_v2,
                a.components.input,
                a.components.output,
                a.components.logic,
                a.components.error_handling,
                a.components.examples,
                a.components.references,
                now,
                "assess_engine",
                a.name,
            ],
        )?;
        if changed > 0 {
            applied += 1;
        }
    }

    Ok(applied)
}

// --- Internal scoring logic ---

/// Compute SMST v2 component scores from a skill row.
fn compute_smst(row: &SkillRow) -> SmstComponents {
    SmstComponents {
        input: score_input(row),
        output: score_output(row),
        logic: score_logic(row),
        error_handling: score_error_handling(row),
        examples: score_examples(row),
        references: score_references(row),
    }
}

/// Input quality (0-15): argument_hint, uses_arguments, description presence.
fn score_input(row: &SkillRow) -> i32 {
    let mut score = 0;
    if row.description.is_some() {
        score += 5;
    }
    if row.argument_hint.is_some() {
        score += 5;
    }
    if row.uses_arguments {
        score += 5;
    }
    score
}

/// Output quality (0-15): description length, context specification.
fn score_output(row: &SkillRow) -> i32 {
    let mut score = 0;
    if let Some(ref desc) = row.description {
        if desc.len() >= 20 {
            score += 5;
        }
        if desc.len() >= 60 {
            score += 5;
        }
    }
    if row.context.is_some() || row.agent.is_some() {
        score += 5;
    }
    score
}

/// Logic quality (0-25): line count sweet spot, budget compliance.
fn score_logic(row: &SkillRow) -> i32 {
    let mut score = 0;
    if let Some(lc) = row.line_count {
        // Reward reasonable size (20-500 lines)
        if lc >= 20 {
            score += 5;
        }
        if lc <= 500 {
            score += 10;
        } else if lc <= 800 {
            score += 5;
        }
    }
    if let Some(chars) = row.content_chars {
        // Budget compliance: <= 16384 chars
        if chars <= 16384 {
            score += 10;
        } else if chars <= 24000 {
            score += 5;
        }
    }
    score
}

/// Error handling quality (0-20): keywords in content (heuristic).
fn score_error_handling(row: &SkillRow) -> i32 {
    let mut score = 0;
    if let Some(ref desc) = row.description {
        let lower = desc.to_lowercase();
        if lower.contains("error") || lower.contains("fail") || lower.contains("invalid") {
            score += 10;
        }
    }
    // allowed_tools implies boundary awareness
    if row.allowed_tools.is_some() {
        score += 10;
    }
    score
}

/// Examples quality (0-15): presence of code blocks or usage patterns.
fn score_examples(row: &SkillRow) -> i32 {
    let mut score = 0;
    // argument_hint serves as a usage example
    if row.argument_hint.is_some() {
        score += 5;
    }
    // uses_dynamic_context implies documented invocation pattern
    if row.uses_dynamic_context {
        score += 5;
    }
    // Sub-skills imply structured decomposition
    if row.sub_skill_count > 0 {
        score += 5;
    }
    score
}

/// References quality (0-10): tags, version, pipeline presence.
fn score_references(row: &SkillRow) -> i32 {
    let mut score = 0;
    if row.tags.is_some() {
        score += 4;
    }
    if row.version.is_some() {
        score += 3;
    }
    if row.pipeline.is_some() {
        score += 3;
    }
    score
}

/// Derive compliance tier from row fields and SMST score.
fn derive_tier(row: &SkillRow, smst_v2: i32) -> ComplianceTier {
    // Invalid: missing description or path
    if row.description.is_none() || row.path.is_empty() {
        return ComplianceTier::Invalid;
    }

    // Bronze: has description + path + scanned (always true if we got here)
    // Silver requires: tags + version + line_count <= 500
    let has_tags = row.tags.is_some();
    let has_version = row.version.is_some();
    let line_ok = row.line_count.is_some_and(|lc| lc <= 500);
    if !has_tags || !has_version || !line_ok {
        return ComplianceTier::Bronze;
    }

    // Gold requires: argument_hint (if user_invocable) + allowed_tools + content <= 16384
    let hint_ok = !row.user_invocable || row.argument_hint.is_some();
    let has_tools = row.allowed_tools.is_some();
    let budget_ok = row.content_chars.is_some_and(|c| c <= 16384);
    if !hint_ok || !has_tools || !budget_ok {
        return ComplianceTier::Silver;
    }

    // Platinum requires: SMST v2 >= 70 + has_agent pairing
    if smst_v2 < 70 || !row.has_agent {
        return ComplianceTier::Gold;
    }

    // Diamond requires: SMST v2 >= 85 + hooks + all Anthropic fields populated
    let has_hooks = row.hooks.is_some();
    let all_anthropic = row.description.is_some()
        && row.allowed_tools.is_some()
        && row.model.is_some();
    if smst_v2 < 85 || !has_hooks || !all_anthropic {
        return ComplianceTier::Platinum;
    }

    ComplianceTier::Diamond
}

/// Compute gaps preventing higher compliance tiers.
fn compute_gaps(row: &SkillRow, smst_v2: i32) -> Vec<ComplianceGap> {
    let mut gaps = Vec::new();

    // Bronze gaps
    if row.description.is_none() {
        gaps.push(ComplianceGap {
            target_tier: "Bronze".to_string(),
            field: "description".to_string(),
            reason: "Missing description frontmatter field".to_string(),
        });
    }

    // Silver gaps
    if row.tags.is_none() {
        gaps.push(ComplianceGap {
            target_tier: "Silver".to_string(),
            field: "tags".to_string(),
            reason: "Missing tags field".to_string(),
        });
    }
    if row.version.is_none() {
        gaps.push(ComplianceGap {
            target_tier: "Silver".to_string(),
            field: "version".to_string(),
            reason: "Missing version field".to_string(),
        });
    }
    if row.line_count.is_none_or(|lc| lc > 500) {
        gaps.push(ComplianceGap {
            target_tier: "Silver".to_string(),
            field: "line_count".to_string(),
            reason: "Exceeds 500-line limit".to_string(),
        });
    }

    // Gold gaps
    if row.user_invocable && row.argument_hint.is_none() {
        gaps.push(ComplianceGap {
            target_tier: "Gold".to_string(),
            field: "argument_hint".to_string(),
            reason: "User-invocable skill missing argument-hint".to_string(),
        });
    }
    if row.allowed_tools.is_none() {
        gaps.push(ComplianceGap {
            target_tier: "Gold".to_string(),
            field: "allowed_tools".to_string(),
            reason: "Missing allowed-tools restriction".to_string(),
        });
    }
    if row.content_chars.is_none_or(|c| c > 16384) {
        gaps.push(ComplianceGap {
            target_tier: "Gold".to_string(),
            field: "content_chars".to_string(),
            reason: "Content exceeds 16KB budget".to_string(),
        });
    }

    // Platinum gaps
    if smst_v2 < 70 {
        gaps.push(ComplianceGap {
            target_tier: "Platinum".to_string(),
            field: "smst_v2".to_string(),
            reason: format!("SMST v2 score {smst_v2} < 70 threshold"),
        });
    }
    if !row.has_agent {
        gaps.push(ComplianceGap {
            target_tier: "Platinum".to_string(),
            field: "has_agent".to_string(),
            reason: "No paired agent file".to_string(),
        });
    }

    // Diamond gaps
    if smst_v2 < 85 {
        gaps.push(ComplianceGap {
            target_tier: "Diamond".to_string(),
            field: "smst_v2".to_string(),
            reason: format!("SMST v2 score {smst_v2} < 85 threshold"),
        });
    }
    if row.hooks.is_none() {
        gaps.push(ComplianceGap {
            target_tier: "Diamond".to_string(),
            field: "hooks".to_string(),
            reason: "Missing hooks definition".to_string(),
        });
    }
    if row.model.is_none() {
        gaps.push(ComplianceGap {
            target_tier: "Diamond".to_string(),
            field: "model".to_string(),
            reason: "Missing model field".to_string(),
        });
    }

    gaps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RegistryPool;

    fn make_skill(name: &str) -> SkillRow {
        let now = Utc::now();
        SkillRow {
            name: name.to_string(),
            path: format!("/skills/{name}/SKILL.md"),
            description: Some("Test skill for assessment".to_string()),
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
    fn test_assess_invalid_no_description() {
        let mut row = make_skill("no-desc");
        row.description = None;
        let a = assess_row(&row);
        assert_eq!(a.compliance, ComplianceTier::Invalid);
    }

    #[test]
    fn test_assess_bronze_minimal() {
        let row = make_skill("minimal");
        let a = assess_row(&row);
        assert_eq!(a.compliance, ComplianceTier::Bronze);
        assert!(a.smst_v2 > 0);
    }

    #[test]
    fn test_assess_silver() {
        let mut row = make_skill("silver");
        row.tags = Some("[\"test\"]".to_string());
        row.version = Some("1.0.0".to_string());
        row.line_count = Some(200);
        let a = assess_row(&row);
        assert_eq!(a.compliance, ComplianceTier::Silver);
    }

    #[test]
    fn test_assess_gold() {
        let mut row = make_skill("gold");
        row.tags = Some("[\"test\"]".to_string());
        row.version = Some("1.0.0".to_string());
        row.line_count = Some(200);
        row.argument_hint = Some("[args]".to_string());
        row.allowed_tools = Some("[\"Read\"]".to_string());
        row.content_chars = Some(5000);
        let a = assess_row(&row);
        assert_eq!(a.compliance, ComplianceTier::Gold);
    }

    #[test]
    fn test_smst_components_sum() {
        let row = make_skill("components");
        let c = compute_smst(&row);
        assert_eq!(c.total(), c.input + c.output + c.logic + c.error_handling + c.examples + c.references);
    }

    #[test]
    fn test_apply_assessments() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let row = make_skill("apply-test");
            skills::upsert(conn, &row)?;
            let assessments = assess_all(conn)?;
            assert_eq!(assessments.len(), 1);
            let applied = apply_assessments(conn, &assessments)?;
            assert_eq!(applied, 1);
            // Verify it was written
            let updated = skills::get(conn, "apply-test")?;
            assert!(updated.compliance.is_some());
            assert!(updated.smst_v2.is_some());
            assert!(updated.last_assessed_at.is_some());
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_gaps_include_missing_fields() {
        let row = make_skill("gappy");
        let a = assess_row(&row);
        // Should have gaps for tags, version, allowed_tools, argument_hint, etc.
        assert!(!a.gaps.is_empty());
        let gap_fields: Vec<&str> = a.gaps.iter().map(|g| g.field.as_str()).collect();
        assert!(gap_fields.contains(&"tags"));
        assert!(gap_fields.contains(&"allowed_tools"));
    }
}
