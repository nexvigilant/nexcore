//! Skill ecosystem measurement: information-theoretic analysis of SKILL.md files.
//!
//! ## Scoring Formula
//!
//! ```text
//! SkillHealth = (info_density_norm * 0.25 + structural_completeness_norm * 0.30
//!              + section_balance_norm * 0.20 + uniqueness_norm * 0.15
//!              + grounding_norm * 0.10) * 10
//! ```
//!
//! ## Five Measurable Effectiveness Principles
//!
//! 1. **Information Density** (Shannon): Unique tokens per line.
//! 2. **Structural Completeness** (Boolean coverage): Required sections present.
//! 3. **Section Balance** (Entropy): Even distribution across sections.
//! 4. **Uniqueness** (NCD): Kolmogorov distance from nearest skill.
//! 5. **Primitive Grounding** (Tier annotations): T1/T2/T3 references.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | section_lines → entropy, features → composite |
//! | T1: Boundary (δ) | score ∈ [0, 10], normalized ∈ [0, 1] |
//! | T1: Comparison (κ) | NCD distance, health thresholds |

use crate::entropy;
use crate::error::{MeasureError, MeasureResult};
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Weights (sum to 1.0)
// ---------------------------------------------------------------------------

const W_INFO_DENSITY: f64 = 0.25;
const W_STRUCTURAL: f64 = 0.30;
const W_SECTION_BALANCE: f64 = 0.20;
const W_UNIQUENESS: f64 = 0.15;
const W_GROUNDING: f64 = 0.10;

// ---------------------------------------------------------------------------
// Required structural sections in a Diamond v2 skill
// ---------------------------------------------------------------------------

const REQUIRED_SECTIONS: &[&str] = &[
    "description",
    "trigger",
    "instruction",
    "example",
    "constraint",
    "tool",
    "invocation",
];

const BONUS_SECTIONS: &[&str] = &[
    "primitive",
    "tier",
    "compliance",
    "anti-pattern",
    "output",
    "integration",
];

// ---------------------------------------------------------------------------
// T3: Skill measurement types
// ---------------------------------------------------------------------------

/// Tier: T3 — Per-skill measurement snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillMeasurement {
    /// Skill name (directory name).
    pub name: String,
    /// Total lines in SKILL.md.
    pub total_lines: usize,
    /// Total non-blank lines.
    pub content_lines: usize,
    /// Unique token count (lowercased, alphanumeric).
    pub unique_tokens: usize,
    /// Total token count.
    pub total_tokens: usize,
    /// Lines per detected section.
    pub section_distribution: Vec<usize>,
    /// Number of sections detected.
    pub section_count: usize,
    /// Names of detected sections.
    pub sections_found: Vec<String>,
    /// Shannon entropy of section distribution (bits).
    pub section_entropy: Entropy,
    /// How many required sections are present (0.0-1.0).
    pub structural_completeness: Probability,
    /// How many bonus sections are present (0.0-1.0).
    pub bonus_completeness: Probability,
    /// Unique tokens / content_lines ratio.
    pub info_density: f64,
    /// Count of T1/T2/T3/Tier references.
    pub grounding_refs: usize,
    /// Cross-references to other skills/tools.
    pub cross_refs: Vec<String>,
}

/// Tier: T3 — Composite skill health assessment.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillHealth {
    pub name: String,
    pub score: HealthScore,
    pub rating: HealthRating,
    pub components: SkillHealthComponents,
}

/// Normalized component scores for skill health.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillHealthComponents {
    /// Information density normalized — sigmoid centered at 5 tokens/line.
    pub info_density_norm: f64,
    /// Structural completeness — fraction of required sections present.
    pub structural_norm: f64,
    /// Section balance — entropy normalization (optimal at [0.6, 0.9]).
    pub section_balance_norm: f64,
    /// Uniqueness — NCD nearest-neighbor distance.
    pub uniqueness_norm: f64,
    /// Primitive grounding — sigmoid centered at 3 tier references.
    pub grounding_norm: f64,
}

/// Tier: T3 — Ecosystem-wide skill health.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillEcosystemHealth {
    pub timestamp: MeasureTimestamp,
    pub skill_count: usize,
    pub total_lines: usize,
    pub mean_score: HealthScore,
    pub mean_rating: HealthRating,
    pub rating_distribution: RatingDistribution,
    pub skill_healths: Vec<SkillHealth>,
    pub ecosystem_entropy: Entropy,
    pub mean_uniqueness: Probability,
}

// ---------------------------------------------------------------------------
// Tokenization primitive (σ: sequence → set)
// ---------------------------------------------------------------------------

/// Extract all tokens and unique token set from lines.
fn tokenize_lines(lines: &[&str]) -> (Vec<String>, HashSet<String>) {
    let mut all = Vec::new();
    let mut unique = HashSet::new();
    for line in lines {
        for word in line.split_whitespace() {
            let token: String = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !token.is_empty() {
                all.push(token.clone());
                unique.insert(token);
            }
        }
    }
    (all, unique)
}

/// Count completeness against a section list.
fn section_coverage(content: &str, sections: &[&str]) -> Probability {
    let found = sections.iter().filter(|s| content.contains(*s)).count();
    Probability::new(found as f64 / sections.len() as f64)
}

// ---------------------------------------------------------------------------
// Section detection primitive (μ: lines → section map)
// ---------------------------------------------------------------------------

/// Detect markdown sections and their line counts.
fn detect_sections(lines: &[&str]) -> HashMap<String, usize> {
    let mut sections = HashMap::new();
    let mut current = String::from("preamble");
    let mut count = 0_usize;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            if count > 0 {
                *sections.entry(current.clone()).or_insert(0) += count;
            }
            current = trimmed.trim_start_matches('#').trim().to_lowercase();
            count = 0;
        } else {
            count += 1;
        }
    }
    if count > 0 {
        *sections.entry(current).or_insert(0) += count;
    }
    sections
}

/// Count T1/T2/T3/Tier primitive grounding references.
fn count_grounding_refs(content: &str) -> usize {
    let patterns = ["t1:", "t1 ", "t2-p", "t2-c", "t2:", "t3:", "t3 ", "tier:"];
    patterns.iter().map(|p| content.matches(p).count()).sum()
}

/// Detect cross-references to other skills, agents, or MCP tools.
fn detect_cross_refs(content: &str) -> Vec<String> {
    let keywords = [
        "mcp__nexcore__",
        "mcp__ferrostack__",
        "/forge",
        "/commit",
        "rust-anatomy-expert",
        "primitive-extractor",
        "ctvp-validator",
        "skill-dev",
        "vigilance-dev",
    ];
    keywords
        .iter()
        .filter(|kw| content.contains(*kw))
        .map(|kw| kw.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Measurement (composed from primitives)
// ---------------------------------------------------------------------------

/// Measure a single SKILL.md file.
pub fn measure_skill(name: &str, content: &str) -> MeasureResult<SkillMeasurement> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let content_lines = lines.iter().filter(|l| !l.trim().is_empty()).count();
    let (all_tokens, unique_set) = tokenize_lines(&lines);

    let sections = detect_sections(&lines);
    let section_distribution: Vec<usize> = sections.values().copied().collect();
    let section_names: Vec<String> = sections.keys().cloned().collect();
    let section_entropy = compute_section_entropy(&section_distribution)?;

    let lower = content.to_lowercase();
    let info_density = if content_lines > 0 {
        unique_set.len() as f64 / content_lines as f64
    } else {
        0.0
    };

    Ok(SkillMeasurement {
        name: name.to_string(),
        total_lines,
        content_lines,
        unique_tokens: unique_set.len(),
        total_tokens: all_tokens.len(),
        section_distribution,
        section_count: section_names.len(),
        sections_found: section_names,
        section_entropy,
        structural_completeness: section_coverage(&lower, REQUIRED_SECTIONS),
        bonus_completeness: section_coverage(&lower, BONUS_SECTIONS),
        info_density,
        grounding_refs: count_grounding_refs(&lower),
        cross_refs: detect_cross_refs(&lower),
    })
}

/// Compute section entropy from distribution.
fn compute_section_entropy(dist: &[usize]) -> MeasureResult<Entropy> {
    if dist.is_empty() {
        return Ok(Entropy::new(0.0));
    }
    entropy::shannon_entropy(dist)
}

// ---------------------------------------------------------------------------
// Health scoring (composed from normalization primitives)
// ---------------------------------------------------------------------------

/// Compute health for a single skill.
pub fn skill_health(measurement: &SkillMeasurement, uniqueness: f64) -> SkillHealth {
    let components = SkillHealthComponents {
        info_density_norm: normalize_info_density(measurement.info_density),
        structural_norm: normalize_structural(measurement),
        section_balance_norm: normalize_section_balance(measurement),
        uniqueness_norm: uniqueness.clamp(0.0, 1.0),
        grounding_norm: normalize_grounding(measurement.grounding_refs),
    };
    let raw = components.info_density_norm * W_INFO_DENSITY
        + components.structural_norm * W_STRUCTURAL
        + components.section_balance_norm * W_SECTION_BALANCE
        + components.uniqueness_norm * W_UNIQUENESS
        + components.grounding_norm * W_GROUNDING;
    let score = HealthScore::new(raw * 10.0);
    SkillHealth {
        name: measurement.name.clone(),
        score,
        rating: score.rating(),
        components,
    }
}

/// Information density sigmoid: centered at 5.0 unique tokens/line.
fn normalize_info_density(density: f64) -> f64 {
    1.0 / (1.0 + (-0.5 * (density - 5.0)).exp())
}

/// Structural completeness: required (0.8) + bonus (0.2).
fn normalize_structural(m: &SkillMeasurement) -> f64 {
    (m.structural_completeness.value() * 0.8 + m.bonus_completeness.value() * 0.2).clamp(0.0, 1.0)
}

/// Section balance: entropy with optimal window [0.6, 0.9].
fn normalize_section_balance(m: &SkillMeasurement) -> f64 {
    if m.section_count <= 1 {
        return 0.0;
    }
    let h_max = entropy::max_entropy(m.section_count)
        .map(|e| e.value())
        .unwrap_or(1.0);
    if h_max < f64::EPSILON {
        return 0.0;
    }
    let ratio = m.section_entropy.value() / h_max;
    if ratio < 0.6 {
        ratio / 0.6
    } else if ratio <= 0.9 {
        1.0
    } else {
        1.0 - (ratio - 0.9) * 3.0
    }
    .clamp(0.0, 1.0)
}

/// Grounding sigmoid: centered at 3 tier references.
fn normalize_grounding(refs: usize) -> f64 {
    1.0 / (1.0 + (-1.0 * (refs as f64 - 3.0)).exp())
}

// ---------------------------------------------------------------------------
// Ecosystem measurement (composed from per-skill primitives)
// ---------------------------------------------------------------------------

/// Scan a skill directory and return (name, content) pairs.
pub fn scan_skill_directory(skills_dir: &Path) -> MeasureResult<Vec<(String, String)>> {
    if !skills_dir.is_dir() {
        return Err(MeasureError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("not a directory: {}", skills_dir.display()),
        )));
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(skills_dir)
        .map_err(MeasureError::Io)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();
    read_skill_files(&entries)
}

/// Read SKILL.md from each directory, skipping meta dirs.
fn read_skill_files(entries: &[PathBuf]) -> MeasureResult<Vec<(String, String)>> {
    let mut results = Vec::new();
    for entry in entries {
        let name = entry
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        if name.starts_with('_') || name == "templates" {
            continue;
        }
        let skill_md = entry.join("SKILL.md");
        if skill_md.exists() {
            let content = std::fs::read_to_string(&skill_md).map_err(MeasureError::Io)?;
            results.push((name, content));
        }
    }
    Ok(results)
}

/// Measure the entire skill ecosystem with NCD-based uniqueness.
pub fn measure_ecosystem(skills_dir: &Path) -> MeasureResult<SkillEcosystemHealth> {
    let skill_contents = scan_skill_directory(skills_dir)?;
    if skill_contents.is_empty() {
        return Ok(empty_ecosystem());
    }
    let measurements: Vec<SkillMeasurement> = skill_contents
        .iter()
        .filter_map(|(name, content)| measure_skill(name, content).ok())
        .collect();
    let uniqueness_scores = compute_uniqueness_scores(&skill_contents);
    aggregate_ecosystem(&measurements, &uniqueness_scores)
}

/// Build an empty ecosystem result.
fn empty_ecosystem() -> SkillEcosystemHealth {
    SkillEcosystemHealth {
        timestamp: MeasureTimestamp::now(),
        skill_count: 0,
        total_lines: 0,
        mean_score: HealthScore::new(0.0),
        mean_rating: HealthRating::Critical,
        rating_distribution: RatingDistribution::default(),
        skill_healths: vec![],
        ecosystem_entropy: Entropy::new(0.0),
        mean_uniqueness: Probability::new(0.0),
    }
}

/// Aggregate per-skill measurements into ecosystem health.
fn aggregate_ecosystem(
    measurements: &[SkillMeasurement],
    uniqueness: &[f64],
) -> MeasureResult<SkillEcosystemHealth> {
    let mut healths = Vec::new();
    let mut dist = RatingDistribution::default();
    let mut score_sum = 0.0_f64;
    let mut total_lines = 0_usize;
    let mut sizes = Vec::new();

    for (i, m) in measurements.iter().enumerate() {
        let uniq = uniqueness.get(i).copied().unwrap_or(0.5);
        let h = skill_health(m, uniq);
        score_sum += h.score.value();
        dist.add(h.rating);
        total_lines += m.total_lines;
        sizes.push(m.total_lines);
        healths.push(h);
    }

    let n = healths.len();
    let mean = if n > 0 { score_sum / n as f64 } else { 0.0 };
    let mean_score = HealthScore::new(mean);
    let eco_entropy = entropy::shannon_entropy(&sizes)?;
    let mean_uniq = if uniqueness.is_empty() {
        0.0
    } else {
        uniqueness.iter().sum::<f64>() / uniqueness.len() as f64
    };

    Ok(SkillEcosystemHealth {
        timestamp: MeasureTimestamp::now(),
        skill_count: n,
        total_lines,
        mean_score,
        mean_rating: mean_score.rating(),
        rating_distribution: dist,
        skill_healths: healths,
        ecosystem_entropy: eco_entropy,
        mean_uniqueness: Probability::new(mean_uniq),
    })
}

/// Per-skill uniqueness via NCD nearest-neighbor.
fn compute_uniqueness_scores(skills: &[(String, String)]) -> Vec<f64> {
    let n = skills.len();
    (0..n).map(|i| nearest_neighbor_ncd(i, skills)).collect()
}

/// Find minimum NCD from skill[i] to any other skill.
fn nearest_neighbor_ncd(i: usize, skills: &[(String, String)]) -> f64 {
    let mut min_ncd = 1.0_f64;
    for (j, _) in skills.iter().enumerate() {
        if i == j {
            continue;
        }
        let ncd_val = entropy::ncd(skills[i].1.as_bytes(), skills[j].1.as_bytes())
            .map(|p| p.value())
            .unwrap_or(1.0);
        if ncd_val < min_ncd {
            min_ncd = ncd_val;
        }
    }
    min_ncd.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skill() -> String {
        r#"---
description: Test skill for measurement
triggers: ["test", "measure"]
user-invocable: true
---

# Test Skill

## Description

This is a test skill that demonstrates measurement.
It has multiple sections and some content.

## Triggers

- `test` — run tests
- `measure` — run measurements

## Instructions

1. First do this thing
2. Then do that thing
3. Finally check results

## Examples

```
/test run
```

## Constraints

- Must not use Python
- Must follow Primitive Codex

## Tool Mappings

- `mcp__nexcore__foundation_levenshtein` — distance

## Tier Classification

| Type | Tier |
|------|------|
| Input | T1: Sequence |
| Output | T2-P: Score |
| Domain | T3: SkillHealth |
"#
        .to_string()
    }

    fn minimal_skill() -> String {
        "# Minimal\n\nA very short skill.\n".to_string()
    }

    fn default_measurement(name: &str) -> SkillMeasurement {
        SkillMeasurement {
            name: name.into(),
            total_lines: 0,
            content_lines: 0,
            unique_tokens: 0,
            total_tokens: 0,
            section_distribution: vec![],
            section_count: 0,
            sections_found: vec![],
            section_entropy: Entropy::new(0.0),
            structural_completeness: Probability::new(0.0),
            bonus_completeness: Probability::new(0.0),
            info_density: 0.0,
            grounding_refs: 0,
            cross_refs: vec![],
        }
    }

    #[test]
    fn measure_well_formed_skill() {
        let m = measure_skill("test-skill", &sample_skill())
            .unwrap_or_else(|_| default_measurement("test-skill"));
        assert!(m.total_lines > 30);
        assert!(m.content_lines > 20);
        assert!(m.unique_tokens > 10);
        assert!(m.section_count >= 5);
        assert!(m.structural_completeness.value() > 0.5);
        assert!(m.grounding_refs > 0);
    }

    #[test]
    fn measure_minimal_is_sparse() {
        let m =
            measure_skill("min", &minimal_skill()).unwrap_or_else(|_| default_measurement("min"));
        assert!(m.total_lines < 5);
        assert!(m.section_count <= 1);
        assert!(m.structural_completeness.value() < 0.5);
    }

    #[test]
    fn health_score_in_range() {
        let m = measure_skill("t", &sample_skill()).unwrap_or_else(|_| default_measurement("t"));
        let h = skill_health(&m, 0.7);
        assert!(h.score.value() >= 0.0 && h.score.value() <= 10.0);
    }

    #[test]
    fn well_formed_beats_minimal() {
        let good = measure_skill("g", &sample_skill()).unwrap_or_else(|_| default_measurement("g"));
        let bad = measure_skill("b", &minimal_skill()).unwrap_or_else(|_| default_measurement("b"));
        let h_g = skill_health(&good, 0.7);
        let h_b = skill_health(&bad, 0.7);
        assert!(h_g.score.value() > h_b.score.value());
    }

    #[test]
    fn info_density_sigmoid() {
        let low = normalize_info_density(1.0);
        let mid = normalize_info_density(5.0);
        let high = normalize_info_density(15.0);
        assert!(low < mid);
        assert!(mid < high);
        assert!((mid - 0.5).abs() < 0.01);
    }

    #[test]
    fn grounding_sigmoid() {
        let zero = normalize_grounding(0);
        let mid = normalize_grounding(3);
        let high = normalize_grounding(10);
        assert!(zero < mid);
        assert!(mid < high);
        assert!((mid - 0.5).abs() < 0.01);
    }

    #[test]
    fn section_detection_works() {
        let lines = vec![
            "# Title",
            "preamble",
            "## Description",
            "desc1",
            "desc2",
            "## Triggers",
            "trigger",
        ];
        let s = detect_sections(&lines);
        assert!(s.contains_key("description"));
        assert_eq!(s.get("description").copied().unwrap_or(0), 2);
    }

    #[test]
    fn grounding_ref_counting() {
        let c = "t1: seq and t2-p and t3: domain and tier: t2-c";
        assert!(count_grounding_refs(c) >= 4);
    }

    #[test]
    fn cross_ref_detection() {
        let c = "use mcp__nexcore__something and /forge";
        let refs = detect_cross_refs(c);
        assert!(!refs.is_empty());
    }

    #[test]
    fn uniqueness_identical_is_low() {
        let s = vec![
            (
                "a".into(),
                "hello world skill content for compression test".into(),
            ),
            (
                "b".into(),
                "hello world skill content for compression test".into(),
            ),
        ];
        let scores = compute_uniqueness_scores(&s);
        assert!(scores[0] < 0.3, "identical={}", scores[0]);
    }

    #[test]
    fn uniqueness_different_is_higher() {
        let a: String = (0..200).map(|i| format!("pharma signal {i} ")).collect();
        let b: String = (0..200).map(|i| format!("quantum qubit {i} ")).collect();
        let s = vec![("a".into(), a), ("b".into(), b)];
        let scores = compute_uniqueness_scores(&s);
        assert!(scores[0] > 0.3, "different={}", scores[0]);
    }

    #[test]
    fn structural_blend() {
        let mut m = default_measurement("t");
        m.structural_completeness = Probability::new(1.0);
        m.bonus_completeness = Probability::new(0.0);
        let norm = normalize_structural(&m);
        assert!((norm - 0.8).abs() < 0.01);
    }

    #[test]
    fn section_balance_single_is_zero() {
        let m = SkillMeasurement {
            section_count: 1,
            section_entropy: Entropy::new(0.0),
            ..default_measurement("s")
        };
        assert!((normalize_section_balance(&m)).abs() < f64::EPSILON);
    }
}
