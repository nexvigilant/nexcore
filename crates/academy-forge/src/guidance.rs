//! Guidance-based scaffold generation for academy pathway authoring.
//!
//! Takes FDA guidance document metadata (or any regulatory document) and
//! generates a complete StaticPathway JSON template with stages mapped to
//! document sections, progressive Bloom levels, and quiz skeletons.
//!
//! ## Pipeline
//!
//! ```text
//! fda_guidance_search(query) → guidance metadata
//!     → forge_scaffold_from_guidance(guidance, sections) → StaticPathway JSON
//!         → Author fills TODOs (using guidance PDF as reference)
//!             → forge_validate(content) → pass/fail
//!                 → forge_compile(pathway_json, output_dir) → TypeScript
//! ```
//!
//! ## Difference from `scaffold.rs`
//!
//! `scaffold.rs` generates from Rust crate domain IR (axioms, theorems).
//! `guidance.rs` generates from regulatory document metadata (topics, sections).
//! Both produce identical StaticPathway JSON output format.

use serde::{Deserialize, Serialize};

/// Bloom's taxonomy levels in ascending order.
const BLOOM_LEVELS: &[&str] = &[
    "Remember",
    "Understand",
    "Apply",
    "Analyze",
    "Evaluate",
    "Create",
];

/// Input metadata for a guidance document.
///
/// Decoupled from `nexcore-fda-guidance` types — the MCP tool converts
/// FDA guidance data into this struct before calling `generate()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidanceInput {
    /// Document slug or identifier.
    pub slug: String,
    /// Document title.
    pub title: String,
    /// Topics/keywords from the document (e.g., "ICH-Quality", "Biosimilars").
    pub topics: Vec<String>,
    /// FDA centers (e.g., "CDER", "CBER").
    pub centers: Vec<String>,
    /// Document status: "Draft" or "Final".
    pub status: String,
    /// Document type (e.g., "Guidance Document").
    pub document_type: String,
}

/// Parameters for guidance scaffold generation.
#[derive(Debug, Clone)]
pub struct GuidanceScaffoldParams {
    /// Pathway ID (e.g., "fda-01").
    pub pathway_id: String,
    /// Pathway title.
    pub title: String,
    /// Domain name (e.g., "pharmacovigilance").
    pub domain: String,
    /// Author-provided section titles to structure stages around.
    /// If empty, stages are auto-generated from guidance topics.
    pub sections: Vec<String>,
}

/// Generate a complete pathway scaffold from guidance document metadata.
///
/// Produces a `serde_json::Value` matching the StaticPathway schema with:
/// - Stages derived from sections (author-provided) or topics (auto-generated)
/// - Bloom levels assigned progressively across stages
/// - Passing scores non-decreasing (70 → 85)
/// - Duration non-decreasing across stages
/// - Quiz skeletons with TODO markers referencing the guidance document
/// - Activity types: reading → interactive → case-study → quiz per stage
pub fn generate(guidance: &GuidanceInput, params: &GuidanceScaffoldParams) -> serde_json::Value {
    // Determine stage sources: explicit sections or auto-generated from topics
    let stage_sources: Vec<String> = if params.sections.is_empty() {
        auto_stages_from_topics(&guidance.topics, &guidance.title)
    } else {
        params.sections.clone()
    };

    let total_stages = stage_sources.len().max(1);
    let mut stages = Vec::new();

    for (i, section_title) in stage_sources.iter().enumerate() {
        let stage_num = (i + 1) as u32;
        let bloom = bloom_for_position(stage_num, total_stages as u32);
        let passing_score = base_score(stage_num);
        let duration = base_duration(stage_num);
        let stage_id = format!("{}-{:02}", params.pathway_id, stage_num);

        let stage = build_guidance_stage(
            &stage_id,
            stage_num,
            section_title,
            guidance,
            bloom,
            passing_score,
            duration,
        );
        stages.push(stage);
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

    let centers_text = if guidance.centers.is_empty() {
        String::new()
    } else {
        format!(" ({})", guidance.centers.join(", "))
    };

    serde_json::json!({
        "$schema": "./pathway.schema.json",
        "id": params.pathway_id,
        "title": params.title,
        "description": format!(
            "TODO: Write a 1-3 sentence description. Based on {} guidance: \"{}\"{}. Status: {}.",
            guidance.document_type, guidance.title, centers_text, guidance.status
        ),
        "domain": params.domain,
        "componentCount": component_count,
        "estimatedDuration": format!("{} hours", estimate_total_hours(total_stages)),
        "sourceGuidance": {
            "slug": guidance.slug,
            "title": guidance.title,
            "status": guidance.status,
            "centers": guidance.centers,
            "topics": guidance.topics,
            "documentType": guidance.document_type,
        },
        "stages": stages
    })
}

/// Build a single stage from a guidance section.
fn build_guidance_stage(
    stage_id: &str,
    stage_num: u32,
    section_title: &str,
    guidance: &GuidanceInput,
    bloom: &str,
    passing_score: u32,
    duration_minutes: u32,
) -> serde_json::Value {
    let guidance_ref = format!("Reference: \"{}\" ({})", guidance.title, guidance.status);

    let mut activities = Vec::new();
    let mut act_num = 0u32;

    // Activity 1: Reading — conceptual introduction
    act_num += 1;
    activities.push(activity_reading(
        stage_id,
        act_num,
        &format!("{section_title}: Key Concepts"),
        15,
        &format!(
            "TODO: Write reading content covering the key concepts from the guidance section on {}. {}.",
            section_title, guidance_ref
        ),
    ));

    // Activity 2: Interactive — applied understanding
    act_num += 1;
    activities.push(activity_interactive(
        stage_id,
        act_num,
        &format!("{section_title} in Practice"),
        15,
        &format!(
            "TODO: Design an interactive exercise applying {} concepts to a realistic scenario. {}.",
            section_title, guidance_ref
        ),
    ));

    // Activity 3: Case study (for Apply+ Bloom levels)
    if bloom_rank(bloom) >= 2 {
        act_num += 1;
        activities.push(activity_case_study(
            stage_id,
            act_num,
            &format!("{section_title}: Case Analysis"),
            20,
            &format!(
                "TODO: Write a case study applying {} with real-world regulatory context. {}.",
                section_title, guidance_ref
            ),
        ));
    }

    // Activity 4: Quiz assessment
    act_num += 1;
    let question_count = if bloom_rank(bloom) >= 3 { 4 } else { 3 };
    let questions: Vec<serde_json::Value> = (1..=question_count)
        .map(|qi| {
            if qi % 3 == 0 {
                quiz_tf(
                    stage_id,
                    qi,
                    &format!(
                        "TODO: Write a true/false question about {} based on the guidance. {}.",
                        section_title, guidance_ref
                    ),
                )
            } else {
                quiz_mc(
                    stage_id,
                    qi,
                    &format!(
                        "TODO: Write a multiple-choice question about {} based on the guidance. {}.",
                        section_title, guidance_ref
                    ),
                )
            }
        })
        .collect();

    activities.push(activity_quiz(
        stage_id,
        act_num,
        &format!("{section_title} Assessment"),
        10,
        &questions,
    ));

    serde_json::json!({
        "id": stage_id,
        "title": section_title,
        "description": format!(
            "TODO: Describe the learning objectives for {}. {}.",
            section_title, guidance_ref
        ),
        "bloomLevel": bloom,
        "passingScore": passing_score,
        "estimatedDuration": format!("{duration_minutes} minutes"),
        "activities": activities
    })
}

/// Auto-generate stage titles from guidance topics when no sections provided.
///
/// Strategy:
/// 1. If topics exist, use them as stage sources (max 8 stages)
/// 2. Always prepend an "Introduction & Regulatory Context" stage
/// 3. Always append an "Integration & Practical Application" stage
fn auto_stages_from_topics(topics: &[String], guidance_title: &str) -> Vec<String> {
    let mut stages = Vec::new();

    // Opening stage
    stages.push(format!("Introduction to {guidance_title}"));

    if topics.is_empty() {
        // Fallback: generic regulatory structure
        stages.push("Regulatory Context and Scope".to_string());
        stages.push("Key Definitions and Terminology".to_string());
        stages.push("Requirements and Compliance".to_string());
        stages.push("Implementation Considerations".to_string());
    } else {
        // Use topics as stage titles (cap at 6 to leave room for intro + capstone)
        for topic in topics.iter().take(6) {
            stages.push(topic.clone());
        }
    }

    // Capstone stage
    stages.push("Integration and Practical Application".to_string());

    stages
}

// ═══════════════════════════════════════════════════════════════════════════
// Activity builders
// ═══════════════════════════════════════════════════════════════════════════

fn activity_reading(
    stage_id: &str,
    num: u32,
    title: &str,
    minutes: u32,
    todo_note: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-a{num:02}"),
        "title": title,
        "type": "reading",
        "estimatedDuration": format!("{minutes} minutes"),
        "_authorNote": todo_note
    })
}

fn activity_interactive(
    stage_id: &str,
    num: u32,
    title: &str,
    minutes: u32,
    todo_note: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-a{num:02}"),
        "title": title,
        "type": "interactive",
        "estimatedDuration": format!("{minutes} minutes"),
        "_authorNote": todo_note
    })
}

fn activity_case_study(
    stage_id: &str,
    num: u32,
    title: &str,
    minutes: u32,
    todo_note: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": format!("{stage_id}-a{num:02}"),
        "title": title,
        "type": "case-study",
        "estimatedDuration": format!("{minutes} minutes"),
        "_authorNote": todo_note
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
fn bloom_for_position(current_stage: u32, total_stages: u32) -> &'static str {
    if total_stages <= 1 {
        return BLOOM_LEVELS[0];
    }
    let progress = (current_stage as f64 - 1.0) / (total_stages as f64 - 1.0);
    let idx = (progress * (BLOOM_LEVELS.len() - 1) as f64).round() as usize;
    BLOOM_LEVELS[idx.min(BLOOM_LEVELS.len() - 1)]
}

/// Get the ordinal rank of a Bloom level (0-5).
fn bloom_rank(bloom: &str) -> usize {
    BLOOM_LEVELS.iter().position(|&b| b == bloom).unwrap_or(0)
}

/// Non-decreasing passing score: 70 → 85 over stages.
fn base_score(stage_num: u32) -> u32 {
    (70 + (stage_num.saturating_sub(1)) * 2).min(85)
}

/// Non-decreasing duration: 45 → 90 minutes over stages.
fn base_duration(stage_num: u32) -> u32 {
    (45 + (stage_num.saturating_sub(1)) * 5).min(90)
}

/// Estimate total hours from stage count.
fn estimate_total_hours(stage_count: usize) -> u32 {
    ((stage_count as f32) * 1.2).ceil() as u32
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn test_guidance() -> GuidanceInput {
        GuidanceInput {
            slug: "test-guidance-slug".to_string(),
            title: "Safety Reporting Requirements for INDs and BA/BE Studies".to_string(),
            topics: vec![
                "Safety Reporting".to_string(),
                "IND Applications".to_string(),
                "Bioavailability".to_string(),
                "Bioequivalence".to_string(),
            ],
            centers: vec!["CDER".to_string()],
            status: "Final".to_string(),
            document_type: "Guidance Document".to_string(),
        }
    }

    #[test]
    fn scaffold_produces_valid_json() {
        let guidance = test_guidance();
        let params = GuidanceScaffoldParams {
            pathway_id: "fda-99".to_string(),
            title: "Safety Reporting Fundamentals".to_string(),
            domain: "pharmacovigilance".to_string(),
            sections: vec![],
        };

        let result = generate(&guidance, &params);
        assert_eq!(result["id"], "fda-99");
        assert_eq!(result["domain"], "pharmacovigilance");
        // Auto-generated: intro + 4 topics + capstone = 6 stages
        assert_eq!(result["stages"].as_array().unwrap().len(), 6);
        assert!(result["componentCount"].as_u64().unwrap() > 0);
    }

    #[test]
    fn scaffold_with_explicit_sections() {
        let guidance = test_guidance();
        let params = GuidanceScaffoldParams {
            pathway_id: "fda-99".to_string(),
            title: "Custom Sections".to_string(),
            domain: "pharmacovigilance".to_string(),
            sections: vec![
                "Background and Scope".to_string(),
                "Definitions".to_string(),
                "Reporting Requirements".to_string(),
            ],
        };

        let result = generate(&guidance, &params);
        assert_eq!(result["stages"].as_array().unwrap().len(), 3);
        assert_eq!(result["stages"][0]["title"], "Background and Scope");
        assert_eq!(result["stages"][1]["title"], "Definitions");
        assert_eq!(result["stages"][2]["title"], "Reporting Requirements");
    }

    #[test]
    fn bloom_levels_are_non_decreasing() {
        let guidance = test_guidance();
        let params = GuidanceScaffoldParams {
            pathway_id: "fda-99".to_string(),
            title: "Bloom Test".to_string(),
            domain: "pharmacovigilance".to_string(),
            sections: vec![],
        };

        let result = generate(&guidance, &params);
        let stages = result["stages"].as_array().unwrap();

        let mut prev_bloom = 0usize;
        for stage in stages {
            let bloom = stage["bloomLevel"].as_str().unwrap();
            let order = bloom_rank(bloom);
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
        let guidance = test_guidance();
        let params = GuidanceScaffoldParams {
            pathway_id: "fda-99".to_string(),
            title: "Score Test".to_string(),
            domain: "pharmacovigilance".to_string(),
            sections: vec![],
        };

        let result = generate(&guidance, &params);
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

    #[test]
    fn source_guidance_metadata_preserved() {
        let guidance = test_guidance();
        let params = GuidanceScaffoldParams {
            pathway_id: "fda-99".to_string(),
            title: "Metadata Test".to_string(),
            domain: "pharmacovigilance".to_string(),
            sections: vec![],
        };

        let result = generate(&guidance, &params);
        let source = &result["sourceGuidance"];
        assert_eq!(source["slug"], "test-guidance-slug");
        assert_eq!(source["status"], "Final");
        assert_eq!(source["centers"][0], "CDER");
    }

    #[test]
    fn empty_topics_produces_fallback_stages() {
        let guidance = GuidanceInput {
            slug: "empty-topics".to_string(),
            title: "Generic Guidance".to_string(),
            topics: vec![],
            centers: vec![],
            status: "Draft".to_string(),
            document_type: "Guidance Document".to_string(),
        };
        let params = GuidanceScaffoldParams {
            pathway_id: "fda-99".to_string(),
            title: "Fallback Test".to_string(),
            domain: "pharmacovigilance".to_string(),
            sections: vec![],
        };

        let result = generate(&guidance, &params);
        let stages = result["stages"].as_array().unwrap();
        // intro + 4 fallback + capstone = 6
        assert_eq!(stages.len(), 6);
        assert_eq!(stages[0]["title"], "Introduction to Generic Guidance");
        assert_eq!(stages[1]["title"], "Regulatory Context and Scope");
    }

    #[test]
    fn case_study_only_at_apply_and_above() {
        let guidance = test_guidance();
        let params = GuidanceScaffoldParams {
            pathway_id: "fda-99".to_string(),
            title: "Case Study Test".to_string(),
            domain: "pharmacovigilance".to_string(),
            sections: vec![],
        };

        let result = generate(&guidance, &params);
        let stages = result["stages"].as_array().unwrap();

        for stage in stages {
            let bloom = stage["bloomLevel"].as_str().unwrap();
            let activities = stage["activities"].as_array().unwrap();
            let has_case_study = activities
                .iter()
                .any(|a| a["type"].as_str() == Some("case-study"));

            if bloom_rank(bloom) >= 2 {
                assert!(
                    has_case_study,
                    "Stage {} (Bloom: {}) should have case study",
                    stage["id"], bloom
                );
            }
        }
    }
}
