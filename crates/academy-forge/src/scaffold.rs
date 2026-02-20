//! Scaffold generation for academy pathway authoring.
//!
//! Takes a `DomainAnalysis` IR and generates a complete StaticPathway JSON
//! template with pre-filled axiom references, quiz skeletons, and TODO markers
//! for narrative content. The output is designed to pass R1-R8 (schema) and
//! R20-R23 (progression) rules out of the box.
//!
//! ## Pipeline
//!
//! ```text
//! forge_extract(crate, domain) → DomainAnalysis
//!     → forge_scaffold(domain_ir, params) → StaticPathway JSON template
//!         → Author fills TODOs
//!             → forge_validate(content) → pass/fail
//! ```

use crate::ir::{AxiomIR, DomainAnalysis, HarmTypeIR};

/// Parameters for scaffold generation.
#[derive(Debug, Clone)]
pub struct ScaffoldParams {
    /// Pathway ID prefix (e.g., "tov-01").
    pub pathway_id: String,
    /// Pathway title.
    pub title: String,
    /// Domain name (e.g., "vigilance").
    pub domain: String,
}

/// Bloom's taxonomy levels in ascending order.
const BLOOM_LEVELS: &[&str] = &[
    "Remember",
    "Understand",
    "Apply",
    "Analyze",
    "Evaluate",
    "Create",
];

/// Generate a complete pathway scaffold from domain IR.
///
/// Produces a `serde_json::Value` matching the StaticPathway schema with:
/// - Stages auto-generated from axioms, harm types, conservation laws, theorems
/// - Bloom levels assigned by concept dependency depth
/// - Passing scores non-decreasing (70 → 85)
/// - Duration non-decreasing across stages
/// - Quiz skeletons with TODO markers
/// - Activity types pre-filled (reading, interactive, quiz per stage minimum)
pub fn generate(domain: &DomainAnalysis, params: &ScaffoldParams) -> serde_json::Value {
    let mut stages = Vec::new();
    let mut stage_num = 0u32;

    // Compute total stage count upfront for Bloom level assignment
    let total_stages = domain.axioms.len()
        + usize::from(!domain.harm_types.is_empty())
        + usize::from(!domain.conservation_laws.is_empty())
        + usize::from(!domain.theorems.is_empty());

    // Phase 1: Axiom stages (ordered by DAG depth, then ID)
    let mut axioms_sorted: Vec<&AxiomIR> = domain.axioms.iter().collect();
    axioms_sorted.sort_by(|a, b| a.depth.cmp(&b.depth).then(a.id.cmp(&b.id)));

    for axiom in &axioms_sorted {
        stage_num += 1;
        let bloom = bloom_for_position(stage_num, total_stages as u32);
        stages.push(axiom_stage(
            &params.pathway_id,
            stage_num,
            axiom,
            bloom,
            base_score(stage_num),
            base_duration(stage_num),
        ));
    }

    // Phase 2: Harm types stage (aggregated — 8 types in one stage)
    if !domain.harm_types.is_empty() {
        stage_num += 1;
        let bloom = bloom_for_position(stage_num, total_stages as u32);
        stages.push(harm_types_stage(
            &params.pathway_id,
            stage_num,
            &domain.harm_types,
            bloom,
            base_score(stage_num),
            base_duration(stage_num),
        ));
    }

    // Phase 3: Conservation laws stage (aggregated — 11 laws in one stage)
    if !domain.conservation_laws.is_empty() {
        stage_num += 1;
        let bloom = bloom_for_position(stage_num, total_stages as u32);
        let law_names: Vec<&str> = domain
            .conservation_laws
            .iter()
            .map(|l| l.name.as_str())
            .collect();
        stages.push(aggregated_stage(
            &params.pathway_id,
            stage_num,
            "Conservation Laws in Practice",
            &format!(
                "TODO: Describe how the {} conservation laws govern system behavior. Laws: {}.",
                domain.conservation_laws.len(),
                law_names.join(", ")
            ),
            bloom,
            base_score(stage_num),
            base_duration(stage_num),
            &law_names,
        ));
    }

    // Phase 4: Theorems & Integration stage
    if !domain.theorems.is_empty() {
        stage_num += 1;
        let bloom = bloom_for_position(stage_num, total_stages as u32);
        let theorem_names: Vec<&str> = domain.theorems.iter().map(|t| t.name.as_str()).collect();
        let theorem_descs: Vec<String> = domain
            .theorems
            .iter()
            .map(|t| format!("the {} (requires {})", t.name, t.required_axioms.join(", ")))
            .collect();
        stages.push(aggregated_stage(
            &params.pathway_id,
            stage_num,
            "Theorems and Integration",
            &format!(
                "TODO: Synthesize all axioms through the principal theorems: {}. Create integrated assessments using the complete framework.",
                theorem_descs.join("; ")
            ),
            bloom,
            base_score(stage_num),
            base_duration(stage_num),
            &theorem_names,
        ));
    }

    // Count total activities
    let component_count: usize = stages
        .iter()
        .filter_map(|s| {
            s.get("activities")
                .and_then(|a| a.as_array())
                .map(|a| a.len())
        })
        .sum();

    serde_json::json!({
        "$schema": "./pathway.schema.json",
        "id": params.pathway_id,
        "title": params.title,
        "description": format!("TODO: Write a 1-3 sentence description of the {} pathway.", params.title),
        "domain": params.domain,
        "componentCount": component_count,
        "estimatedDuration": format!("{} hours", (stage_num as f32 * 1.2).ceil() as u32),
        "stages": stages
    })
}

/// Generate a stage for a single axiom.
fn axiom_stage(
    pathway_id: &str,
    num: u32,
    axiom: &AxiomIR,
    bloom: &str,
    passing_score: u32,
    duration_minutes: u32,
) -> serde_json::Value {
    let stage_id = format!("{pathway_id}-{num:02}");
    let deps_text = if axiom.dependencies.is_empty() {
        "no dependencies (root axiom)".to_string()
    } else {
        format!("depends on {}", axiom.dependencies.join(", "))
    };

    let activities = vec![
        activity_reading(&stage_id, 1, &format!("{} Concepts", axiom.name), 15),
        activity_interactive(&stage_id, 2, &format!("{} in Practice", axiom.name), 15),
        activity_quiz(
            &stage_id,
            3,
            &format!("{} Assessment", axiom.name),
            15,
            &[
                quiz_mc(
                    &stage_id,
                    1,
                    &format!(
                        "TODO: Write a question about Axiom {} ({}) — {}.",
                        axiom.id, axiom.name, axiom.core_assertion
                    ),
                ),
                quiz_tf(
                    &stage_id,
                    2,
                    &format!(
                        "TODO: Write a true/false about {} — {}.",
                        axiom.name, deps_text
                    ),
                ),
            ],
        ),
    ];

    serde_json::json!({
        "id": stage_id,
        "title": axiom.name,
        "description": format!(
            "Axiom {} — {}. TODO: Expand with learning objectives. Dependencies: {}.",
            axiom.id, axiom.core_assertion, deps_text
        ),
        "bloomLevel": bloom,
        "passingScore": passing_score,
        "estimatedDuration": format!("{duration_minutes} minutes"),
        "activities": activities
    })
}

/// Generate a stage for harm types (aggregated).
fn harm_types_stage(
    pathway_id: &str,
    num: u32,
    harm_types: &[HarmTypeIR],
    bloom: &str,
    passing_score: u32,
    duration_minutes: u32,
) -> serde_json::Value {
    let stage_id = format!("{pathway_id}-{num:02}");
    let type_list: Vec<String> = harm_types
        .iter()
        .map(|h| format!("{} ({})", h.name, h.letter))
        .collect();

    let mut activities: Vec<serde_json::Value> = Vec::new();
    let mut act_num = 0u32;

    // Reading overview
    act_num += 1;
    activities.push(activity_reading(
        &stage_id,
        act_num,
        "Harm Type Classification Framework",
        15,
    ));

    // Interactive per pair of harm types
    for chunk in harm_types.chunks(2) {
        act_num += 1;
        let names: Vec<&str> = chunk.iter().map(|h| h.name.as_str()).collect();
        activities.push(activity_interactive(
            &stage_id,
            act_num,
            &format!("{} Harm", names.join(" and ")),
            10,
        ));
    }

    // Case study
    act_num += 1;
    activities.push(activity_case_study(
        &stage_id,
        act_num,
        "Harm Type Case Studies",
        15,
    ));

    // Quiz
    act_num += 1;
    let questions: Vec<serde_json::Value> = harm_types
        .iter()
        .enumerate()
        .take(3)
        .map(|(qi, h)| {
            let law_text = match h.conservation_law {
                Some(n) => format!("governed by Conservation Law {n}"),
                None => "has no associated conservation law".to_string(),
            };
            quiz_mc(
                &stage_id,
                qi as u32 + 1,
                &format!(
                    "TODO: Write a question about {} harm (Type {}) — affects levels {:?}, {}.",
                    h.name, h.letter, h.hierarchy_levels, law_text
                ),
            )
        })
        .collect();
    activities.push(activity_quiz(
        &stage_id,
        act_num,
        "Harm Types Assessment",
        10,
        &questions,
    ));

    serde_json::json!({
        "id": stage_id,
        "title": "Harm Types",
        "description": format!(
            "TODO: Describe the {} harm type classifications: {}. Map each to its conservation law and hierarchy levels.",
            harm_types.len(),
            type_list.join(", ")
        ),
        "bloomLevel": bloom,
        "passingScore": passing_score,
        "estimatedDuration": format!("{duration_minutes} minutes"),
        "activities": activities
    })
}

/// Generate a stage for aggregated concepts (conservation laws, theorems).
#[allow(clippy::too_many_arguments)]
fn aggregated_stage(
    pathway_id: &str,
    num: u32,
    title: &str,
    description: &str,
    bloom: &str,
    passing_score: u32,
    duration_minutes: u32,
    concept_names: &[&str],
) -> serde_json::Value {
    let stage_id = format!("{pathway_id}-{num:02}");

    let mut activities = vec![
        activity_reading(&stage_id, 1, &format!("{title} Overview"), 15),
        activity_interactive(&stage_id, 2, &format!("{title} Analysis"), 15),
        activity_case_study(&stage_id, 3, &format!("{title} Scenarios"), 15),
    ];

    // Quiz with one question per concept (capped at 4)
    let questions: Vec<serde_json::Value> = concept_names
        .iter()
        .enumerate()
        .take(4)
        .map(|(qi, name)| {
            quiz_mc(
                &stage_id,
                qi as u32 + 1,
                &format!("TODO: Write a question about {name}."),
            )
        })
        .collect();
    activities.push(activity_quiz(
        &stage_id,
        4,
        &format!("{title} Assessment"),
        15,
        &questions,
    ));

    serde_json::json!({
        "id": stage_id,
        "title": title,
        "description": description,
        "bloomLevel": bloom,
        "passingScore": passing_score,
        "estimatedDuration": format!("{duration_minutes} minutes"),
        "activities": activities
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Activity builders
// ═══════════════════════════════════════════════════════════════════════════

fn activity_reading(stage_id: &str, num: u32, title: &str, minutes: u32) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-a{num:02}"),
        "title": title,
        "type": "reading",
        "estimatedDuration": format!("{minutes} minutes")
    })
}

fn activity_interactive(stage_id: &str, num: u32, title: &str, minutes: u32) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-a{num:02}"),
        "title": title,
        "type": "interactive",
        "estimatedDuration": format!("{minutes} minutes")
    })
}

fn activity_case_study(stage_id: &str, num: u32, title: &str, minutes: u32) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-a{num:02}"),
        "title": title,
        "type": "case-study",
        "estimatedDuration": format!("{minutes} minutes")
    })
}

fn activity_quiz(
    stage_id: &str,
    num: u32,
    title: &str,
    minutes: u32,
    questions: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-a{num:02}"),
        "title": title,
        "type": "quiz",
        "estimatedDuration": format!("{minutes} minutes"),
        "quiz": {
            "questions": questions
        }
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Quiz question builders
// ═══════════════════════════════════════════════════════════════════════════

fn quiz_mc(stage_id: &str, num: u32, question_text: &str) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-q{num:02}"),
        "type": "multiple-choice",
        "question": question_text,
        "options": [
            "TODO: Correct answer",
            "TODO: Distractor 1",
            "TODO: Distractor 2",
            "TODO: Distractor 3"
        ],
        "correctAnswer": 0,
        "points": 2,
        "explanation": "TODO: Explain why the correct answer is correct."
    })
}

fn quiz_tf(stage_id: &str, num: u32, question_text: &str) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-q{num:02}"),
        "type": "true-false",
        "question": question_text,
        "correctAnswer": true,
        "points": 1,
        "explanation": "TODO: Explain the answer."
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Map global stage position to Bloom level, ensuring monotonic non-decreasing.
/// Stage 1/N → Remember, Stage N/N → Create.
fn bloom_for_position(current_stage: u32, total_stages: u32) -> &'static str {
    if total_stages <= 1 {
        return BLOOM_LEVELS[0];
    }
    let progress = (current_stage as f64 - 1.0) / (total_stages as f64 - 1.0);
    let idx = (progress * (BLOOM_LEVELS.len() - 1) as f64).round() as usize;
    BLOOM_LEVELS[idx.min(BLOOM_LEVELS.len() - 1)]
}

/// Non-decreasing passing score: 70 → 85 over stages.
fn base_score(stage_num: u32) -> u32 {
    (70 + (stage_num.saturating_sub(1)) * 2).min(85)
}

/// Non-decreasing duration: 45 → 90 minutes over stages.
fn base_duration(stage_num: u32) -> u32 {
    (45 + (stage_num.saturating_sub(1)) * 5).min(90)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn test_domain() -> DomainAnalysis {
        let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
        crate::domain::extract_domain("vigilance", &workspace).unwrap()
    }

    #[test]
    fn scaffold_produces_valid_json() {
        let domain = test_domain();
        let params = ScaffoldParams {
            pathway_id: "tov-99".to_string(),
            title: "Test Scaffold".to_string(),
            domain: "vigilance".to_string(),
        };

        let result = generate(&domain, &params);
        assert_eq!(result["id"], "tov-99");
        assert!(result["stages"].as_array().unwrap().len() >= 7);
        assert!(result["componentCount"].as_u64().unwrap() > 0);
    }

    #[test]
    fn scaffold_passes_schema_validation() {
        let domain = test_domain();
        let params = ScaffoldParams {
            pathway_id: "tov-99".to_string(),
            title: "Scaffold Validation Test".to_string(),
            domain: "vigilance".to_string(),
        };

        let scaffold = generate(&domain, &params);
        let report = crate::validate(&scaffold, Some(&domain));

        let errors: Vec<_> = report
            .findings
            .iter()
            .filter(|f| matches!(f.severity, crate::validate::Severity::Error))
            .collect();
        assert!(
            errors.is_empty(),
            "scaffold should have zero errors: {errors:?}"
        );
    }

    #[test]
    fn bloom_levels_are_non_decreasing() {
        let domain = test_domain();
        let params = ScaffoldParams {
            pathway_id: "tov-99".to_string(),
            title: "Bloom Test".to_string(),
            domain: "vigilance".to_string(),
        };

        let result = generate(&domain, &params);
        let stages = result["stages"].as_array().unwrap();

        let bloom_order =
            |b: &str| -> usize { BLOOM_LEVELS.iter().position(|&x| x == b).unwrap_or(0) };

        let mut prev_bloom = 0usize;
        for stage in stages {
            let bloom = stage["bloomLevel"].as_str().unwrap();
            let order = bloom_order(bloom);
            assert!(
                order >= prev_bloom,
                "Bloom regression: {} < {} at stage {}",
                bloom,
                BLOOM_LEVELS[prev_bloom],
                stage["id"]
            );
            prev_bloom = order;
        }
    }

    #[test]
    fn passing_scores_are_non_decreasing() {
        let domain = test_domain();
        let params = ScaffoldParams {
            pathway_id: "tov-99".to_string(),
            title: "Score Test".to_string(),
            domain: "vigilance".to_string(),
        };

        let result = generate(&domain, &params);
        let stages = result["stages"].as_array().unwrap();

        let mut prev_score = 0u64;
        for stage in stages {
            let score = stage["passingScore"].as_u64().unwrap();
            assert!(
                score >= prev_score,
                "Score regression at stage {}",
                stage["id"]
            );
            prev_score = score;
        }
    }
}
