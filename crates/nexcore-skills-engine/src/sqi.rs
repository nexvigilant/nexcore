//! # Skill Quality Index (SQI) v2
//!
//! Chemistry-derived scoring of SKILL.md files across 6 per-skill dimensions
//! plus a 7th ecosystem-level dimension (DistributionEntropy).
//!
//! ## v2 Changes
//! - Equal weights (1/6 per-skill, 1/7 ecosystem) replace arbitrary 0.15/0.20
//! - DistributionEntropy (Shannon) captures tool concentration risk
//! - Occupancy uses direct coverage ratio + competitive penalty (not naive Langmuir)
//! - Sensitivity analysis for weight perturbation

use crate::foundation::skill_metadata::{SkillMetadata, parse_frontmatter};
use nexcore_primitives::chemistry::{CompetitiveLangmuir, hill_response};
use serde::{Deserialize, Serialize};

// ============================================================================
// Weight constants
// ============================================================================

/// Per-skill scoring: 6 dimensions, equal weight.
const WEIGHT_PER_SKILL_DIM: f64 = 1.0 / 6.0;

/// Ecosystem scoring: 7 dimensions, equal weight.
const WEIGHT_PER_ECO_DIM: f64 = 1.0 / 7.0;

/// v1 weights preserved for backward comparison.
#[allow(dead_code)]
const V1_WEIGHTS: [f64; 6] = [0.15, 0.20, 0.15, 0.20, 0.15, 0.15];

// ============================================================================
// Error types
// ============================================================================

/// Errors during SQI computation.
#[derive(Debug, nexcore_error::Error)]
pub enum SqiError {
    /// Missing frontmatter delimiter
    #[error("missing frontmatter: {0}")]
    MissingFrontmatter(String),
    /// Content too short to analyze
    #[error("content too short ({0} bytes)")]
    ContentTooShort(usize),
    /// Entropy computation failed
    #[error("entropy error: {0}")]
    EntropyError(String),
}

// ============================================================================
// Grade
// ============================================================================

/// Grade derived from composite SQI score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SqiGrade {
    Draft,
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl SqiGrade {
    /// Classify from a 0.0-1.0 SQI score.
    #[must_use]
    pub fn from_score(score: f64) -> Self {
        if score >= 0.85 {
            Self::Platinum
        } else if score >= 0.70 {
            Self::Gold
        } else if score >= 0.55 {
            Self::Silver
        } else if score >= 0.40 {
            Self::Bronze
        } else {
            Self::Draft
        }
    }
}

impl std::fmt::Display for SqiGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Bronze => write!(f, "bronze"),
            Self::Silver => write!(f, "silver"),
            Self::Gold => write!(f, "gold"),
            Self::Platinum => write!(f, "platinum"),
        }
    }
}

// ============================================================================
// Dimensions
// ============================================================================

/// Which chemistry-derived dimension is being scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SqiDimension {
    /// Activation energy - how many triggers activate this skill
    Activation,
    /// Saturation - content richness (sections, code, tables, decision trees)
    Saturation,
    /// Cooperativity - network connectivity (see-also, upstream, downstream)
    Cooperativity,
    /// Stability - infrastructure buffering (MCP tools, model, flags)
    Stability,
    /// Decay - estimated longevity based on domain and specificity
    Decay,
    /// Occupancy - section coverage completeness (v2: direct ratio + competitive penalty)
    Occupancy,
    /// Distribution entropy - Shannon entropy of tool/skill allocation (ecosystem only)
    DistributionEntropy,
}

impl std::fmt::Display for SqiDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Activation => write!(f, "Ea (Activation)"),
            Self::Saturation => write!(f, "Vsat (Saturation)"),
            Self::Cooperativity => write!(f, "Hill (Cooperativity)"),
            Self::Stability => write!(f, "Buf (Stability)"),
            Self::Decay => write!(f, "t\u{00bd} (Decay)"),
            Self::Occupancy => write!(f, "Occ (Occupancy)"),
            Self::DistributionEntropy => write!(f, "H (Distribution Entropy)"),
        }
    }
}

// ============================================================================
// Score structs
// ============================================================================

/// Score for a single dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub dimension: SqiDimension,
    pub score: f64,
    pub weight: f64,
    pub weighted: f64,
    pub rationale: String,
}

/// Final SQI result for a single skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqiResult {
    pub sqi: f64,
    pub grade: SqiGrade,
    pub dimensions: Vec<DimensionScore>,
    pub limiting_dimension: SqiDimension,
    pub recommendations: Vec<String>,
}

/// Ecosystem-level SQI result aggregating multiple skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemSqiResult {
    /// Arithmetic mean of per-skill SQI scores (unweighted by tool count).
    pub mean_sqi_unweighted: f64,
    /// Tool-count-weighted mean of per-skill SQI scores.
    pub mean_sqi_weighted: f64,
    /// Normalized Shannon entropy H/H_max of tool distribution [0,1].
    pub distribution_entropy: f64,
    /// Concentration risk = 1 - H_normalized.
    pub concentration_risk: f64,
    /// Individual skill results.
    pub skill_results: Vec<SqiResult>,
    /// Overall ecosystem grade (from weighted mean).
    pub grade: SqiGrade,
}

// ============================================================================
// Content analysis
// ============================================================================

/// Internal content analysis counts.
struct ContentAnalysis {
    h2_count: usize,
    decision_tree_count: usize,
    code_block_count: usize,
    table_count: usize,
    word_count: usize,
    has_examples: bool,
    has_anti_patterns: bool,
    has_integration: bool,
}

/// Classify a single line for structural markers.
fn classify_line(line: &str) -> (bool, bool, bool, bool, bool, bool) {
    let trimmed = line.trim();
    let is_h2 = trimmed.starts_with("## ");
    let is_code_fence = trimmed.starts_with("```");
    let is_table_row = trimmed.starts_with('|') && trimmed.ends_with('|');
    let is_decision =
        trimmed.contains("- if ") || trimmed.contains("\u{2192}") || trimmed.contains("decision");

    let heading_lower = trimmed.to_lowercase();
    let is_example_heading = is_h2 && heading_lower.contains("example");
    let is_antipattern_heading =
        is_h2 && (heading_lower.contains("anti-pattern") || heading_lower.contains("antipattern"));

    (
        is_h2,
        is_code_fence,
        is_table_row,
        is_decision,
        is_example_heading,
        is_antipattern_heading,
    )
}

/// Count table separator rows for correction.
fn count_table_separators(content: &str) -> usize {
    let lower = content.to_lowercase();
    lower.matches("|---").count() + lower.matches("| ---").count()
}

impl ContentAnalysis {
    fn from_content(content: &str) -> Self {
        let mut h2 = 0;
        let mut decision = 0;
        let mut code_fence = 0;
        let mut table_row: usize = 0;
        let mut has_examples = false;
        let mut has_anti_patterns = false;

        for line in content.lines() {
            let (is_h2, is_cf, is_tr, is_dec, is_ex, is_ap) = classify_line(line);
            if is_h2 {
                h2 += 1;
            }
            if is_cf {
                code_fence += 1;
            }
            if is_tr {
                table_row += 1;
            }
            if is_dec {
                decision += 1;
            }
            if is_ex {
                has_examples = true;
            }
            if is_ap {
                has_anti_patterns = true;
            }
        }

        // Code blocks come in pairs (open + close)
        let code_block_count = code_fence / 2;

        // Table rows: subtract separator rows
        let table_count = table_row.saturating_sub(count_table_separators(content));

        let content_lower = content.to_lowercase();
        if !has_examples && content_lower.contains("example") {
            has_examples = true;
        }
        let has_integration =
            content_lower.contains("## integration") || content_lower.contains("## mcp");

        Self {
            h2_count: h2,
            decision_tree_count: decision,
            code_block_count,
            table_count,
            word_count: content.split_whitespace().count(),
            has_examples,
            has_anti_patterns,
            has_integration,
        }
    }
}

// ============================================================================
// Dimension scorers (Phase 2: equal weights)
// ============================================================================

/// Ea: Activation - based on trigger count.
/// Sweet spot is 6-8 triggers. Too few = hard to activate. Too many = unfocused.
fn score_activation(trigger_count: usize) -> DimensionScore {
    let score = match trigger_count {
        0 => 0.10,
        1 => 0.30,
        2..=3 => 0.50,
        4..=5 => 0.65,
        6..=8 => 0.80,
        9..=12 => 0.70,
        13..=15 => 0.55,
        _ => 0.40,
    };

    DimensionScore {
        dimension: SqiDimension::Activation,
        score,
        weight: WEIGHT_PER_SKILL_DIM,
        weighted: score * WEIGHT_PER_SKILL_DIM,
        rationale: format!("{trigger_count} triggers \u{2192} {score:.2}"),
    }
}

/// Vsat: Saturation - content richness from structural elements.
fn score_saturation(analysis: &ContentAnalysis) -> DimensionScore {
    #[allow(clippy::cast_precision_loss)]
    let raw = f64::min(1.0, analysis.h2_count as f64 / 8.0) * 0.30
        + f64::min(1.0, analysis.code_block_count as f64 / 5.0) * 0.25
        + f64::min(1.0, analysis.table_count as f64 / 15.0) * 0.15
        + f64::min(1.0, analysis.decision_tree_count as f64 / 6.0) * 0.15
        + f64::min(1.0, analysis.word_count as f64 / 1500.0) * 0.15;

    let bonus = if analysis.has_examples { 0.05 } else { 0.0 }
        + if analysis.has_anti_patterns {
            0.05
        } else {
            0.0
        };

    let score = f64::min(1.0, raw + bonus);

    DimensionScore {
        dimension: SqiDimension::Saturation,
        score,
        weight: WEIGHT_PER_SKILL_DIM,
        weighted: score * WEIGHT_PER_SKILL_DIM,
        rationale: format!(
            "{}H2 {}code {}tbl {}dec {}w \u{2192} {score:.2}",
            analysis.h2_count,
            analysis.code_block_count,
            analysis.table_count,
            analysis.decision_tree_count,
            analysis.word_count,
        ),
    }
}

/// Hill: Cooperativity - network connectivity via Hill equation.
fn score_cooperativity(
    see_also: usize,
    upstream: usize,
    downstream: usize,
    has_pipeline: bool,
) -> DimensionScore {
    let connection_count = see_also + upstream + downstream;
    let pipeline_bonus: f64 = if has_pipeline { 1.0 } else { 0.0 };

    #[allow(clippy::cast_precision_loss)]
    let hill_input = connection_count as f64 + pipeline_bonus;
    let hill_score = hill_response(hill_input, 4.0, 2.0);

    let flow_bonus = if upstream > 0 && downstream > 0 {
        0.10
    } else {
        0.0
    };
    let score = f64::min(1.0, hill_score + flow_bonus);

    DimensionScore {
        dimension: SqiDimension::Cooperativity,
        score,
        weight: WEIGHT_PER_SKILL_DIM,
        weighted: score * WEIGHT_PER_SKILL_DIM,
        rationale: format!(
            "{connection_count} connections (hill={hill_score:.2}) + flow={flow_bonus:.2} \u{2192} {score:.2}"
        ),
    }
}

/// Count buffered infrastructure factors for stability scoring.
fn count_buffered_factors(
    mcp_tools: usize,
    has_model: bool,
    has_integration: bool,
    content: &str,
) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let mut factors = f64::min(2.0, mcp_tools as f64);
    if has_model {
        factors += 1.0;
    }
    if has_integration {
        factors += 1.0;
    }

    let lower = content.to_lowercase();
    let has_hardcode =
        lower.contains("/usr/local/bin") || lower.contains("/home/") || lower.contains("c:\\");
    if !has_hardcode {
        factors += 1.0;
    }

    factors
}

/// Buf: Stability - infrastructure buffering.
fn score_stability(
    mcp_tools: usize,
    has_model: bool,
    has_integration: bool,
    content: &str,
) -> DimensionScore {
    let factors = count_buffered_factors(mcp_tools, has_model, has_integration, content);
    let score = f64::min(1.0, factors / 5.0);

    DimensionScore {
        dimension: SqiDimension::Stability,
        score,
        weight: WEIGHT_PER_SKILL_DIM,
        weighted: score * WEIGHT_PER_SKILL_DIM,
        rationale: format!(
            "{factors:.0}/5 factors (mcp={mcp_tools}, model={has_model}, integration={has_integration}) \u{2192} {score:.2}"
        ),
    }
}

/// Estimate half-life in days from domain string.
fn base_half_life_from_domain(domain: Option<&str>) -> f64 {
    match domain {
        Some(d) => {
            let d_lower = d.to_lowercase();
            if d_lower.contains("methodology") || d_lower.contains("foundation") {
                730.0
            } else if d_lower.contains("pharmacovigilance")
                || d_lower.contains("pv")
                || d_lower.contains("safety")
            {
                548.0
            } else if d_lower.contains("rust") || d_lower.contains("code") {
                365.0
            } else {
                300.0
            }
        }
        None => 250.0,
    }
}

/// t1/2: Decay - estimated longevity.
fn score_decay(domain: Option<&str>, tags: &[String], intent: Option<&str>) -> DimensionScore {
    let mut half_life = base_half_life_from_domain(domain);

    let tag_set: Vec<&str> = tags.iter().map(String::as_str).collect();
    if tag_set
        .iter()
        .any(|t| *t == "evergreen" || *t == "foundational")
    {
        half_life *= 1.3;
    }
    if tag_set.iter().any(|t| *t == "experimental" || *t == "beta") {
        half_life *= 0.5;
    }
    if let Some(i) = intent {
        if i.len() > 200 {
            half_life *= 0.9;
        }
    }

    let score = f64::min(1.0, half_life / 365.0);

    DimensionScore {
        dimension: SqiDimension::Decay,
        score,
        weight: WEIGHT_PER_SKILL_DIM,
        weighted: score * WEIGHT_PER_SKILL_DIM,
        rationale: format!("estimated t\u{00bd}={half_life:.0}d \u{2192} {score:.2}"),
    }
}

/// Occ v2: Direct coverage ratio + tag richness + competitive penalty.
///
/// Primary (70%): sections_covered / expected_sections (capped at 1.0)
/// Secondary (30%): tag richness factor
/// Penalty: if overlap_skills > 1, apply CompetitiveLangmuir fractional penalty
fn score_occupancy(
    analysis: &ContentAnalysis,
    tag_count: usize,
    overlap_skills: usize,
) -> DimensionScore {
    const EXPECTED_SECTIONS: f64 = 8.0;

    // Primary: direct coverage ratio (no Langmuir model assumptions)
    #[allow(clippy::cast_precision_loss)]
    let coverage = f64::min(1.0, analysis.h2_count as f64 / EXPECTED_SECTIONS);

    // Secondary: tag richness
    #[allow(clippy::cast_precision_loss)]
    let tag_richness = f64::min(1.0, tag_count as f64 / 5.0);

    let base_score = coverage * 0.70 + tag_richness * 0.30;

    // Competitive penalty: when multiple skills overlap the same niche,
    // each gets a fractional share via CompetitiveLangmuir
    let score = if overlap_skills > 1 {
        // Model: each overlapping skill competes for the same "niche capacity"
        if let Ok(mut comp) = CompetitiveLangmuir::new(1.0) {
            for i in 0..overlap_skills {
                // Equal affinity (K=1.0) and equal concentration (1.0) for all competitors
                let _ = comp.add_component(&format!("skill_{i}"), 1.0, 1.0);
            }
            // Our share of the niche
            let our_share = comp.total_coverage() / overlap_skills.max(1) as f64;
            // Penalty scales with competition: 1 skill = no penalty, N skills = reduced
            let penalty_factor = our_share * overlap_skills as f64;
            f64::min(1.0, base_score * penalty_factor)
        } else {
            base_score
        }
    } else {
        base_score
    };

    let score = f64::min(1.0, score);

    DimensionScore {
        dimension: SqiDimension::Occupancy,
        score,
        weight: WEIGHT_PER_SKILL_DIM,
        weighted: score * WEIGHT_PER_SKILL_DIM,
        rationale: format!(
            "{}/{EXPECTED_SECTIONS:.0} sections (cov={coverage:.2}) + {tag_count} tags, {overlap_skills} overlap \u{2192} {score:.2}",
            analysis.h2_count,
        ),
    }
}

/// H: Distribution Entropy - Shannon entropy of tool/skill allocation.
///
/// Uses `nexcore_measure::entropy::shannon_entropy` for normalized H/H_max.
/// Score is the normalized entropy [0,1]. Higher = more evenly distributed.
fn score_distribution_entropy(tool_counts: &[usize]) -> DimensionScore {
    let n = tool_counts.iter().filter(|&&c| c > 0).count();

    if n <= 1 {
        return DimensionScore {
            dimension: SqiDimension::DistributionEntropy,
            score: 0.0,
            weight: WEIGHT_PER_ECO_DIM,
            weighted: 0.0,
            rationale: format!("{n} active sources \u{2192} 0.00 (no distribution)"),
        };
    }

    let h = nexcore_measure::entropy::shannon_entropy(tool_counts)
        .map(|e| e.value())
        .unwrap_or(0.0);
    let h_max = nexcore_measure::entropy::max_entropy(n)
        .map(|e| e.value())
        .unwrap_or(1.0);

    let h_normalized = if h_max > f64::EPSILON { h / h_max } else { 0.0 };
    let score = h_normalized.clamp(0.0, 1.0);

    DimensionScore {
        dimension: SqiDimension::DistributionEntropy,
        score,
        weight: WEIGHT_PER_ECO_DIM,
        weighted: score * WEIGHT_PER_ECO_DIM,
        rationale: format!(
            "H={h:.3} / H_max={h_max:.3} = {h_normalized:.3} across {n} sources \u{2192} {score:.2}"
        ),
    }
}

// ============================================================================
// Main entry points
// ============================================================================

/// Build all 6 dimension scores from metadata and content analysis.
fn build_dimensions(
    meta: &SkillMetadata,
    analysis: &ContentAnalysis,
    content: &str,
) -> Vec<DimensionScore> {
    vec![
        score_activation(meta.triggers.len()),
        score_saturation(analysis),
        score_cooperativity(
            meta.see_also.len(),
            meta.upstream.len(),
            meta.downstream.len(),
            meta.pipeline.is_some(),
        ),
        score_stability(
            meta.mcp_tools.len(),
            meta.model.is_some(),
            analysis.has_integration,
            content,
        ),
        score_decay(meta.domain.as_deref(), &meta.tags, meta.intent.as_deref()),
        score_occupancy(analysis, meta.tags.len(), 1), // per-skill: no overlap info
    ]
}

/// Generate improvement recommendations for weak dimensions.
fn recommend(dimensions: &[DimensionScore]) -> Vec<String> {
    dimensions
        .iter()
        .filter(|d| d.score < 0.40)
        .map(|d| {
            match d.dimension {
                SqiDimension::Activation => "Add trigger phrases to frontmatter (target: 6-8)",
                SqiDimension::Saturation => {
                    "Add more structured content: code blocks, tables, decision trees"
                }
                SqiDimension::Cooperativity => {
                    "Add see-also, upstream, and downstream skill references"
                }
                SqiDimension::Stability => "Add mcp-tools and model fields to frontmatter",
                SqiDimension::Decay => "Specify a domain and add 'evergreen'/'foundational' tags",
                SqiDimension::Occupancy => "Add more H2 sections (aim for ~8) and tags",
                SqiDimension::DistributionEntropy => {
                    "Redistribute tools more evenly across skills/servers"
                }
            }
            .to_string()
        })
        .collect()
}

/// Find the dimension with the lowest raw score.
fn find_limiting(dimensions: &[DimensionScore]) -> SqiDimension {
    dimensions
        .iter()
        .min_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|d| d.dimension)
        .unwrap_or(SqiDimension::Activation)
}

/// Compute the Skill Quality Index for a SKILL.md content string.
///
/// Scores across 6 dimensions with equal weight (1/6 each).
///
/// # Errors
///
/// Returns `SqiError` if frontmatter is missing or content is too short.
pub fn compute_sqi(content: &str) -> Result<SqiResult, SqiError> {
    if content.len() < 20 {
        return Err(SqiError::ContentTooShort(content.len()));
    }

    let meta = parse_frontmatter(content).map_err(SqiError::MissingFrontmatter)?;
    let analysis = ContentAnalysis::from_content(content);
    let dimensions = build_dimensions(&meta, &analysis, content);

    let sqi: f64 = dimensions.iter().map(|d| d.weighted).sum();

    Ok(SqiResult {
        sqi,
        grade: SqiGrade::from_score(sqi),
        limiting_dimension: find_limiting(&dimensions),
        recommendations: recommend(&dimensions),
        dimensions,
    })
}

/// Compute ecosystem-level SQI across multiple skills.
///
/// Adds the 7th dimension (DistributionEntropy) and provides both
/// weighted and unweighted aggregate means.
///
/// # Arguments
/// * `skills` - Per-skill SQI results
/// * `tool_counts` - Number of tools each skill/server provides
/// * `weights` - Optional per-skill weighting (defaults to tool_counts)
pub fn compute_ecosystem_sqi(skills: &[SqiResult], tool_counts: &[usize]) -> EcosystemSqiResult {
    let n = skills.len();

    // Unweighted mean
    let mean_unweighted = if n > 0 {
        skills.iter().map(|s| s.sqi).sum::<f64>() / n as f64
    } else {
        0.0
    };

    // Tool-count-weighted mean
    let total_tools: usize = tool_counts.iter().sum();
    let mean_weighted = if total_tools > 0 {
        skills
            .iter()
            .zip(tool_counts.iter())
            .map(|(s, &tc)| s.sqi * tc as f64)
            .sum::<f64>()
            / total_tools as f64
    } else {
        mean_unweighted
    };

    // 7th dimension: distribution entropy
    let entropy_dim = score_distribution_entropy(tool_counts);
    let h_normalized = entropy_dim.score;
    let concentration_risk = 1.0 - h_normalized;

    // Ecosystem grade from weighted mean (incorporating entropy penalty)
    let eco_score = mean_weighted * (6.0 / 7.0) + entropy_dim.weighted;
    let grade = SqiGrade::from_score(eco_score);

    EcosystemSqiResult {
        mean_sqi_unweighted: mean_unweighted,
        mean_sqi_weighted: mean_weighted,
        distribution_entropy: h_normalized,
        concentration_risk,
        skill_results: skills.to_vec(),
        grade,
    }
}

/// Sensitivity analysis: perturb each dimension weight by +/- delta and report SQI change.
///
/// Returns (dimension, delta_sqi) pairs showing how sensitive the result is
/// to each dimension. Equal-weight systems produce uniform sensitivity.
#[must_use]
pub fn sensitivity_analysis(result: &SqiResult, delta: f64) -> Vec<(SqiDimension, f64)> {
    result
        .dimensions
        .iter()
        .map(|d| {
            // How much does SQI change if this dimension's weight increases by delta?
            // Under equal weights: sensitivity = d.score * delta (linear)
            let delta_sqi = d.score * delta;
            (d.dimension, delta_sqi)
        })
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_SKILL: &str = r#"---
name: minimal
intent: A minimal skill
tags:
  - test
---
# Minimal Skill
"#;

    const RICH_SKILL: &str = r#"---
name: rich-skill
intent: A fully-featured skill for testing
compliance: gold
domain: methodology
model: sonnet
chain-position: head
pipeline: quality-pipeline
triggers:
  - analyze quality
  - check skill
  - assess skill
  - evaluate skill
  - grade skill
  - score skill
  - rate skill
see-also:
  - skill-chemometrics
  - ctvp-validator
upstream:
  - data-transformer
downstream:
  - alert-dispatcher
mcp-tools:
  - skill_validate
  - skill_scan
  - pv_signal_complete
tags:
  - methodology
  - quality
  - analysis
  - foundational
  - scoring
---
# Rich Skill

## Overview

This is a comprehensive skill with many sections.

## Decision Tree

- if score > 0.85 → Platinum
- if score > 0.70 → Gold
- if score > 0.55 → Silver

## Examples

```rust
let result = compute_sqi(content)?;
println!("{}", result.grade);
```

## Anti-Patterns

Do not skip frontmatter fields.

## Integration

Uses MCP tools for validation.

| Dimension | Weight | Description |
|-----------|--------|-------------|
| Activation | 1/6 | Trigger count |
| Saturation | 1/6 | Content depth |
| Cooperativity | 1/6 | Network links |
| Stability | 1/6 | Infrastructure |
| Decay | 1/6 | Longevity |
| Occupancy | 1/6 | Coverage |

## Configuration

Set up via frontmatter fields.

## Verification

Run tests to validate.
"#;

    #[test]
    fn test_grade_boundaries() {
        assert_eq!(SqiGrade::from_score(0.90), SqiGrade::Platinum);
        assert_eq!(SqiGrade::from_score(0.85), SqiGrade::Platinum);
        assert_eq!(SqiGrade::from_score(0.84), SqiGrade::Gold);
        assert_eq!(SqiGrade::from_score(0.70), SqiGrade::Gold);
        assert_eq!(SqiGrade::from_score(0.69), SqiGrade::Silver);
        assert_eq!(SqiGrade::from_score(0.55), SqiGrade::Silver);
        assert_eq!(SqiGrade::from_score(0.54), SqiGrade::Bronze);
        assert_eq!(SqiGrade::from_score(0.40), SqiGrade::Bronze);
        assert_eq!(SqiGrade::from_score(0.39), SqiGrade::Draft);
        assert_eq!(SqiGrade::from_score(0.0), SqiGrade::Draft);
    }

    #[test]
    fn test_minimal_skill_is_draft_or_bronze() {
        let result =
            compute_sqi(MINIMAL_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        assert!(
            result.grade == SqiGrade::Draft || result.grade == SqiGrade::Bronze,
            "Minimal skill should be Draft or Bronze, got {:?} (sqi={:.2})",
            result.grade,
            result.sqi,
        );
        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_rich_skill_scores_higher() {
        let rich =
            compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("rich compute_sqi failed: {e}"));
        let minimal = compute_sqi(MINIMAL_SKILL)
            .unwrap_or_else(|e| panic!("minimal compute_sqi failed: {e}"));
        assert!(
            rich.sqi > minimal.sqi,
            "Rich ({:.2}) should beat minimal ({:.2})",
            rich.sqi,
            minimal.sqi,
        );
    }

    #[test]
    fn test_rich_skill_is_silver_or_above() {
        let result = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        assert!(
            result.sqi >= 0.55,
            "Rich skill should be Silver+ (sqi={:.2}, grade={:?})",
            result.sqi,
            result.grade,
        );
    }

    #[test]
    fn test_all_six_dimensions_present() {
        let result = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        assert_eq!(result.dimensions.len(), 6);
        let dims: Vec<SqiDimension> = result.dimensions.iter().map(|d| d.dimension).collect();
        assert!(dims.contains(&SqiDimension::Activation));
        assert!(dims.contains(&SqiDimension::Saturation));
        assert!(dims.contains(&SqiDimension::Cooperativity));
        assert!(dims.contains(&SqiDimension::Stability));
        assert!(dims.contains(&SqiDimension::Decay));
        assert!(dims.contains(&SqiDimension::Occupancy));
    }

    #[test]
    fn test_weights_sum_to_one() {
        let result = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        let total: f64 = result.dimensions.iter().map(|d| d.weight).sum();
        assert!(
            (total - 1.0).abs() < 1e-10,
            "Weights should sum to 1.0, got {total}"
        );
    }

    #[test]
    fn test_sqi_equals_weighted_sum() {
        let result = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        let computed: f64 = result.dimensions.iter().map(|d| d.weighted).sum();
        assert!((result.sqi - computed).abs() < 1e-10);
    }

    #[test]
    fn test_activation_sweet_spot() {
        let low = score_activation(0);
        let sweet = score_activation(7);
        let high = score_activation(20);
        assert!(sweet.score > low.score);
        assert!(sweet.score > high.score);
    }

    #[test]
    fn test_content_analysis() {
        let analysis = ContentAnalysis::from_content(RICH_SKILL);
        assert!(
            analysis.h2_count >= 6,
            "Expected 6+ H2, got {}",
            analysis.h2_count
        );
        assert!(analysis.code_block_count >= 1);
        assert!(analysis.has_examples);
        assert!(analysis.has_anti_patterns);
        assert!(analysis.has_integration);
    }

    #[test]
    fn test_cooperativity_uses_hill() {
        let none = score_cooperativity(0, 0, 0, false);
        let many = score_cooperativity(3, 2, 2, true);
        assert!(many.score > none.score);
        assert!(
            many.score > 0.5,
            "7 connections + pipeline should score >0.5"
        );
    }

    #[test]
    fn test_occupancy_v2_direct_coverage() {
        let sparse = ContentAnalysis::from_content("---\nname: x\n---\n# X\n");
        let rich = ContentAnalysis::from_content(RICH_SKILL);
        assert!(score_occupancy(&rich, 5, 1).score > score_occupancy(&sparse, 0, 1).score);
    }

    #[test]
    fn test_occupancy_competitive_penalty() {
        let analysis = ContentAnalysis::from_content(RICH_SKILL);
        let no_overlap = score_occupancy(&analysis, 5, 1);
        let with_overlap = score_occupancy(&analysis, 5, 5);
        assert!(
            no_overlap.score >= with_overlap.score,
            "Overlap should not increase score: no_overlap={:.3}, with_overlap={:.3}",
            no_overlap.score,
            with_overlap.score,
        );
    }

    #[test]
    fn test_decay_foundational_lasts_longer() {
        let foundation = score_decay(Some("methodology"), &["foundational".to_string()], None);
        let experimental = score_decay(Some("misc"), &["experimental".to_string()], None);
        assert!(foundation.score > experimental.score);
    }

    #[test]
    fn test_stability_with_mcp_tools() {
        assert!(
            score_stability(3, true, true, "").score > score_stability(0, false, false, "").score
        );
    }

    #[test]
    fn test_error_too_short() {
        assert!(matches!(
            compute_sqi("hi"),
            Err(SqiError::ContentTooShort(_))
        ));
    }

    #[test]
    fn test_error_no_frontmatter() {
        assert!(matches!(
            compute_sqi("# Just a heading with no frontmatter at all"),
            Err(SqiError::MissingFrontmatter(_)),
        ));
    }

    #[test]
    fn test_limiting_dimension_is_lowest() {
        let result =
            compute_sqi(MINIMAL_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        let min_score = result
            .dimensions
            .iter()
            .map(|d| d.score)
            .fold(f64::INFINITY, f64::min);
        let limiting = result
            .dimensions
            .iter()
            .find(|d| d.dimension == result.limiting_dimension)
            .unwrap_or_else(|| panic!("limiting dimension not found in results"));
        assert!((limiting.score - min_score).abs() < 1e-10);
    }

    #[test]
    fn test_grade_display() {
        assert_eq!(SqiGrade::Platinum.to_string(), "platinum");
        assert_eq!(SqiGrade::Draft.to_string(), "draft");
    }

    // === v2 tests ===

    #[test]
    fn test_v2_equal_weights() {
        let result = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        for dim in &result.dimensions {
            assert!(
                (dim.weight - WEIGHT_PER_SKILL_DIM).abs() < 1e-10,
                "Dimension {:?} weight={}, expected {}",
                dim.dimension,
                dim.weight,
                WEIGHT_PER_SKILL_DIM,
            );
        }
    }

    #[test]
    fn test_distribution_entropy_uniform() {
        // Perfectly uniform distribution: 10 tools each across 5 servers
        let counts = vec![10, 10, 10, 10, 10];
        let dim = score_distribution_entropy(&counts);
        assert!(
            (dim.score - 1.0).abs() < 1e-10,
            "Uniform should give H_norm=1.0, got {}",
            dim.score,
        );
    }

    #[test]
    fn test_distribution_entropy_concentrated() {
        // Highly concentrated: one server has almost everything
        let counts = vec![260, 1, 1, 1, 1];
        let dim = score_distribution_entropy(&counts);
        assert!(
            dim.score < 0.30,
            "Concentrated should give low entropy, got {}",
            dim.score,
        );
    }

    #[test]
    fn test_distribution_entropy_single() {
        // Single server: no distribution
        let counts = vec![100];
        let dim = score_distribution_entropy(&counts);
        assert!(
            dim.score < f64::EPSILON,
            "Single source should give 0 entropy, got {}",
            dim.score,
        );
    }

    #[test]
    fn test_ecosystem_sqi_basic() {
        let r1 = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("failed: {e}"));
        let r2 = compute_sqi(MINIMAL_SKILL).unwrap_or_else(|e| panic!("failed: {e}"));
        let tool_counts = vec![50, 10];
        let eco = compute_ecosystem_sqi(&[r1.clone(), r2.clone()], &tool_counts);

        // Weighted mean should lean toward r1 (50 tools vs 10)
        let expected_unweighted = (r1.sqi + r2.sqi) / 2.0;
        assert!(
            (eco.mean_sqi_unweighted - expected_unweighted).abs() < 1e-10,
            "unweighted mean: got {}, expected {}",
            eco.mean_sqi_unweighted,
            expected_unweighted,
        );
        assert!(eco.distribution_entropy > 0.0);
        assert!(eco.concentration_risk < 1.0);
    }

    #[test]
    fn test_ecosystem_concentration_risk() {
        let r1 = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("failed: {e}"));
        // Heavy concentration on first
        let counts_concentrated = vec![260, 5, 5];
        let eco1 =
            compute_ecosystem_sqi(&[r1.clone(), r1.clone(), r1.clone()], &counts_concentrated);

        // Even distribution
        let counts_even = vec![90, 90, 90];
        let eco2 = compute_ecosystem_sqi(&[r1.clone(), r1.clone(), r1.clone()], &counts_even);

        assert!(
            eco1.concentration_risk > eco2.concentration_risk,
            "Concentrated ({:.3}) should have higher risk than even ({:.3})",
            eco1.concentration_risk,
            eco2.concentration_risk,
        );
    }

    #[test]
    fn test_sensitivity_uniform() {
        let result = compute_sqi(RICH_SKILL).unwrap_or_else(|e| panic!("compute_sqi failed: {e}"));
        let sens = sensitivity_analysis(&result, 0.01);

        // Under equal weights, sensitivity is proportional to raw score
        assert_eq!(sens.len(), 6);
        for (dim, delta) in &sens {
            assert!(
                *delta >= 0.0,
                "Sensitivity for {:?} should be non-negative: {}",
                dim,
                delta,
            );
        }
    }

    #[test]
    fn test_dimension_display_all() {
        // Ensure all 7 variants have Display
        let dims = [
            SqiDimension::Activation,
            SqiDimension::Saturation,
            SqiDimension::Cooperativity,
            SqiDimension::Stability,
            SqiDimension::Decay,
            SqiDimension::Occupancy,
            SqiDimension::DistributionEntropy,
        ];
        for d in &dims {
            let s = d.to_string();
            assert!(!s.is_empty(), "Display for {:?} should not be empty", d);
        }
    }
}
