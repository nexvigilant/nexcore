//! Atomize: decompose StaticPathway stages into Atomic Learning Objects.
//!
//! Takes a validated `StaticPathway` JSON and produces an `AtomizedPathway`
//! with ALOs and intra-pathway dependency edges.
//!
//! ## Pipeline Position
//!
//! ```text
//! forge_extract → forge_scaffold → [author] → forge_validate
//!     → forge_atomize → forge_validate(R28-R36) → forge_compile
//! ```

pub mod splitter;

use std::collections::HashMap;

use crate::error::{ForgeError, ForgeResult};
use crate::ir::{
    AloAssessment, AloEdge, AloEdgeType, AloType, AtomicLearningObject, AtomizedPathway, BloomLevel,
};
use splitter::{estimate_minutes, split_by_concept};

/// Atomize a validated StaticPathway JSON into ALOs with dependency edges.
///
/// Each stage is decomposed into Hook + Concept + Activity + Reflection ALOs.
/// Intra-pathway Prereq/Assesses edges are generated automatically.
pub fn atomize(pathway_json: &serde_json::Value) -> ForgeResult<AtomizedPathway> {
    let pathway_id = pathway_json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ForgeError::AtomizeError {
            message: "Missing pathway 'id' field".to_string(),
        })?;

    let title = pathway_json
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");

    let stages = pathway_json
        .get("stages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ForgeError::AtomizeError {
            message: "Missing or invalid 'stages' array".to_string(),
        })?;

    let mut all_alos: Vec<AtomicLearningObject> = Vec::new();
    let mut all_edges: Vec<AloEdge> = Vec::new();

    // Track last reflection per stage for inter-stage linking
    let mut last_reflection_ids: Vec<String> = Vec::new();

    for (stage_idx, stage) in stages.iter().enumerate() {
        let stage_num = stage_idx + 1;
        let stage_id = stage
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let stage_title = stage
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Stage");
        let stage_description = stage
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let bloom = stage
            .get("bloomLevel")
            .and_then(|v| v.as_str())
            .and_then(BloomLevel::from_str_loose)
            .unwrap_or(BloomLevel::Remember);

        let passing_score = stage
            .get("passingScore")
            .and_then(|v| v.as_u64())
            .unwrap_or(70) as u8;

        let activities = stage
            .get("activities")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Counters per type within this stage
        let mut type_counters: HashMap<AloType, u16> = HashMap::new();
        let mut stage_concept_ids: Vec<String> = Vec::new();
        let mut stage_activity_ids: Vec<String> = Vec::new();
        let mut stage_reflection_id: Option<String> = None;

        // ── Step 1: Hook ALO from stage description ──
        let hook_seq = next_seq(&mut type_counters, AloType::Hook);
        let hook_id = format!("{pathway_id}-{stage_num:02}-h{hook_seq:02}");
        let hook = AtomicLearningObject {
            id: hook_id.clone(),
            title: format!("Why {stage_title} Matters"),
            alo_type: AloType::Hook,
            learning_objective: format!("Recognize the importance of {stage_title}"),
            estimated_duration: 2,
            bloom_level: BloomLevel::Remember,
            content: format!("## Why This Matters\n\n{stage_description}\n"),
            ksb_refs: Vec::new(),
            source_stage_id: stage_id.to_string(),
            source_activity_id: None,
            assessment: None,
        };
        all_alos.push(hook);

        // Inter-stage sequencing: link previous stage's last reflection → this hook.
        // Uses Extends (not Prereq) to preserve R34: hooks have zero inbound Prereqs.
        // Extends means "unlocked after completing previous stage" without hard gating.
        if let Some(prev_ref_id) = last_reflection_ids.last() {
            all_edges.push(AloEdge {
                from: prev_ref_id.clone(),
                to: hook_id.clone(),
                edge_type: AloEdgeType::Extends,
                strength: 1.0,
            });
        }

        let mut prev_id = hook_id;

        // ── Step 2: Reading activities → Concept ALOs ──
        for activity in &activities {
            let act_type = activity.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let act_title = activity
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Reading");
            let act_id = activity.get("id").and_then(|v| v.as_str()).unwrap_or("");

            if act_type == "reading" {
                // Extract content or generate placeholder
                let content = activity
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let fragments = if content.len() > 100 {
                    split_by_concept(content, act_title)
                } else {
                    vec![splitter::ContentFragment {
                        title: act_title.to_string(),
                        content: content.to_string(),
                        estimated_minutes: estimate_minutes(content).max(5),
                    }]
                };

                for fragment in fragments {
                    let seq = next_seq(&mut type_counters, AloType::Concept);
                    let concept_id = format!("{pathway_id}-{stage_num:02}-c{seq:02}");

                    let duration = fragment.estimated_minutes.clamp(
                        AloType::Concept.min_duration(),
                        AloType::Concept.max_duration(),
                    );

                    let concept_alo = AtomicLearningObject {
                        id: concept_id.clone(),
                        title: fragment.title,
                        alo_type: AloType::Concept,
                        learning_objective: format!("Explain the key principles of this concept"),
                        estimated_duration: duration,
                        bloom_level: bloom_for_concept(bloom),
                        content: fragment.content,
                        ksb_refs: Vec::new(),
                        source_stage_id: stage_id.to_string(),
                        source_activity_id: Some(act_id.to_string()),
                        assessment: None,
                    };

                    // Prereq from previous ALO in chain
                    all_edges.push(AloEdge {
                        from: prev_id.clone(),
                        to: concept_id.clone(),
                        edge_type: AloEdgeType::Prereq,
                        strength: 1.0,
                    });

                    prev_id = concept_id.clone();
                    stage_concept_ids.push(concept_id.clone());
                    all_alos.push(concept_alo);
                }
            }
        }

        // ── Step 3: Interactive/case-study activities → Activity ALOs ──
        for activity in &activities {
            let act_type = activity.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let act_title = activity
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Exercise");
            let act_id = activity.get("id").and_then(|v| v.as_str()).unwrap_or("");

            if act_type == "interactive" || act_type == "case-study" {
                let seq = next_seq(&mut type_counters, AloType::Activity);
                let activity_alo_id = format!("{pathway_id}-{stage_num:02}-a{seq:02}");

                let content = activity
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let duration = parse_duration_str(
                    activity
                        .get("estimatedDuration")
                        .and_then(|v| v.as_str())
                        .unwrap_or("10 minutes"),
                )
                .clamp(
                    AloType::Activity.min_duration(),
                    AloType::Activity.max_duration(),
                );

                let activity_alo = AtomicLearningObject {
                    id: activity_alo_id.clone(),
                    title: act_title.to_string(),
                    alo_type: AloType::Activity,
                    learning_objective: format!("Apply {stage_title} concepts in practice"),
                    estimated_duration: duration,
                    bloom_level: bloom_for_activity(bloom),
                    content,
                    ksb_refs: Vec::new(),
                    source_stage_id: stage_id.to_string(),
                    source_activity_id: Some(act_id.to_string()),
                    assessment: None,
                };

                // Prereq from all concept ALOs in this stage
                for concept_id in &stage_concept_ids {
                    all_edges.push(AloEdge {
                        from: concept_id.clone(),
                        to: activity_alo_id.clone(),
                        edge_type: AloEdgeType::Prereq,
                        strength: 1.0,
                    });
                }

                stage_activity_ids.push(activity_alo_id.clone());
                all_alos.push(activity_alo);
            }
        }

        // ── Step 4: Quiz activities → Reflection ALOs ──
        for activity in &activities {
            let act_type = activity.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let act_title = activity
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Assessment");
            let act_id = activity.get("id").and_then(|v| v.as_str()).unwrap_or("");

            if act_type == "quiz" {
                let seq = next_seq(&mut type_counters, AloType::Reflection);
                let reflection_id = format!("{pathway_id}-{stage_num:02}-r{seq:02}");

                let questions = activity
                    .get("quiz")
                    .and_then(|q| q.get("questions"))
                    .and_then(|q| q.as_array())
                    .cloned()
                    .unwrap_or_default();

                // Limit to 4 questions per ALO; split if more
                let question_chunks = chunk_questions(&questions, 4);

                for (chunk_idx, chunk) in question_chunks.iter().enumerate() {
                    let rid = if question_chunks.len() > 1 {
                        let extra_seq = next_seq(&mut type_counters, AloType::Reflection);
                        if chunk_idx == 0 {
                            reflection_id.clone()
                        } else {
                            format!("{pathway_id}-{stage_num:02}-r{extra_seq:02}")
                        }
                    } else {
                        reflection_id.clone()
                    };

                    let reflection_alo = AtomicLearningObject {
                        id: rid.clone(),
                        title: if question_chunks.len() > 1 {
                            format!("{act_title} (Part {})", chunk_idx + 1)
                        } else {
                            act_title.to_string()
                        },
                        alo_type: AloType::Reflection,
                        learning_objective: format!("Evaluate your understanding of {stage_title}"),
                        estimated_duration: AloType::Reflection
                            .min_duration()
                            .max((chunk.len() as u16).min(AloType::Reflection.max_duration())),
                        bloom_level: bloom_for_reflection(bloom),
                        content: String::new(),
                        ksb_refs: Vec::new(),
                        source_stage_id: stage_id.to_string(),
                        source_activity_id: Some(act_id.to_string()),
                        assessment: Some(AloAssessment {
                            passing_score,
                            questions: chunk.clone(),
                        }),
                    };

                    // Prereq from activity ALOs (must practice before reflect)
                    for aid in &stage_activity_ids {
                        all_edges.push(AloEdge {
                            from: aid.clone(),
                            to: rid.clone(),
                            edge_type: AloEdgeType::Prereq,
                            strength: 1.0,
                        });
                    }

                    // If no activity ALOs, prereq from last concept
                    if stage_activity_ids.is_empty() {
                        if let Some(last_concept) = stage_concept_ids.last() {
                            all_edges.push(AloEdge {
                                from: last_concept.clone(),
                                to: rid.clone(),
                                edge_type: AloEdgeType::Prereq,
                                strength: 1.0,
                            });
                        }
                    }

                    // Assesses edges from all concepts
                    for cid in &stage_concept_ids {
                        all_edges.push(AloEdge {
                            from: rid.clone(),
                            to: cid.clone(),
                            edge_type: AloEdgeType::Assesses,
                            strength: 1.0,
                        });
                    }

                    stage_reflection_id = Some(rid.clone());
                    all_alos.push(reflection_alo);
                }
            }
        }

        // Track last reflection for inter-stage linking
        if let Some(rid) = stage_reflection_id {
            last_reflection_ids.push(rid);
        } else if let Some(last_concept) = stage_concept_ids.last() {
            // No quiz in this stage — use last concept as chain point
            last_reflection_ids.push(last_concept.clone());
        }
    }

    Ok(AtomizedPathway {
        id: pathway_id.to_string(),
        title: title.to_string(),
        source_pathway_id: pathway_id.to_string(),
        alos: all_alos,
        edges: all_edges,
    })
}

// ── Helpers ──

fn next_seq(counters: &mut HashMap<AloType, u16>, alo_type: AloType) -> u16 {
    let counter = counters.entry(alo_type).or_insert(0);
    *counter += 1;
    *counter
}

/// Parse "45 minutes" or "1 hour" into minutes.
fn parse_duration_str(s: &str) -> u16 {
    let lower = s.to_lowercase();
    if lower.contains("hour") {
        let num: u16 = lower
            .split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(1);
        num * 60
    } else {
        lower
            .split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(10)
    }
}

/// Map stage Bloom to appropriate Concept ALO Bloom (same or one below).
fn bloom_for_concept(stage_bloom: BloomLevel) -> BloomLevel {
    match stage_bloom {
        BloomLevel::Remember => BloomLevel::Remember,
        BloomLevel::Understand => BloomLevel::Remember,
        _ => BloomLevel::Understand,
    }
}

/// Map stage Bloom to Activity ALO Bloom (Apply minimum).
fn bloom_for_activity(stage_bloom: BloomLevel) -> BloomLevel {
    if stage_bloom < BloomLevel::Apply {
        BloomLevel::Apply
    } else {
        stage_bloom
    }
}

/// Map stage Bloom to Reflection ALO Bloom (Evaluate minimum).
fn bloom_for_reflection(stage_bloom: BloomLevel) -> BloomLevel {
    if stage_bloom < BloomLevel::Evaluate {
        BloomLevel::Evaluate
    } else {
        stage_bloom
    }
}

/// Chunk questions into groups of `max_per_chunk`.
fn chunk_questions(
    questions: &[serde_json::Value],
    max_per_chunk: usize,
) -> Vec<Vec<serde_json::Value>> {
    if questions.is_empty() {
        return vec![Vec::new()];
    }
    questions
        .chunks(max_per_chunk)
        .map(|c| c.to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pathway() -> serde_json::Value {
        serde_json::json!({
            "id": "test-01",
            "title": "Test Pathway",
            "description": "A test pathway for atomization.",
            "stages": [
                {
                    "id": "test-01-01",
                    "title": "First Concepts",
                    "description": "Introduction to the first concepts.",
                    "bloomLevel": "Remember",
                    "passingScore": 70,
                    "estimatedDuration": "45 minutes",
                    "activities": [
                        {
                            "id": "test-01-01-01",
                            "title": "Reading: Core Ideas",
                            "type": "reading",
                            "estimatedDuration": "15 minutes",
                            "content": "## Idea One\n\nFirst concept content here with enough words to be meaningful.\n\n## Idea Two\n\nSecond concept content here with enough words to be meaningful."
                        },
                        {
                            "id": "test-01-01-02",
                            "title": "Practice Exercise",
                            "type": "interactive",
                            "estimatedDuration": "15 minutes"
                        },
                        {
                            "id": "test-01-01-03",
                            "title": "Knowledge Check",
                            "type": "quiz",
                            "estimatedDuration": "10 minutes",
                            "quiz": {
                                "questions": [
                                    {
                                        "id": "q1",
                                        "type": "multiple-choice",
                                        "question": "What is concept one?",
                                        "options": ["A", "B", "C"],
                                        "correctAnswer": 0,
                                        "explanation": "A is correct."
                                    }
                                ]
                            }
                        }
                    ]
                },
                {
                    "id": "test-01-02",
                    "title": "Applied Concepts",
                    "description": "Applying what was learned.",
                    "bloomLevel": "Apply",
                    "passingScore": 75,
                    "estimatedDuration": "60 minutes",
                    "activities": [
                        {
                            "id": "test-01-02-01",
                            "title": "Deep Reading",
                            "type": "reading",
                            "estimatedDuration": "20 minutes",
                            "content": "Applied content without headings."
                        },
                        {
                            "id": "test-01-02-02",
                            "title": "Case Study",
                            "type": "case-study",
                            "estimatedDuration": "15 minutes"
                        }
                    ]
                }
            ]
        })
    }

    #[test]
    fn test_atomize_produces_alos() {
        let pathway = sample_pathway();
        let atomized = atomize(&pathway).unwrap();

        // 2 stages → each produces Hook + Concept + Activity + Reflection = 4 ALOs each = 8 total
        assert!(
            atomized.alos.len() >= 4,
            "Expected at least 4 ALOs, got {}",
            atomized.alos.len()
        );

        // Verify each ALO type is present at least once
        let has_hook = atomized.alos.iter().any(|a| a.alo_type == AloType::Hook);
        let has_concept = atomized.alos.iter().any(|a| a.alo_type == AloType::Concept);
        let has_activity = atomized
            .alos
            .iter()
            .any(|a| a.alo_type == AloType::Activity);
        let has_reflection = atomized
            .alos
            .iter()
            .any(|a| a.alo_type == AloType::Reflection);
        assert!(has_hook, "Expected at least one Hook ALO");
        assert!(has_concept, "Expected at least one Concept ALO");
        assert!(has_activity, "Expected at least one Activity ALO");
        assert!(has_reflection, "Expected at least one Reflection ALO");

        // Every ALO must have a non-empty id and a source_stage_id
        for alo in &atomized.alos {
            assert!(!alo.id.is_empty(), "ALO id should not be empty");
            assert!(
                !alo.source_stage_id.is_empty(),
                "ALO source_stage_id should not be empty for {}",
                alo.id
            );
        }
    }

    #[test]
    fn test_atomize_generates_edges() {
        let pathway = sample_pathway();
        let atomized = atomize(&pathway).unwrap();

        assert!(
            !atomized.edges.is_empty(),
            "Expected at least one edge, got 0"
        );

        // All edge endpoints must reference valid ALO ids
        let alo_ids: std::collections::HashSet<&str> =
            atomized.alos.iter().map(|a| a.id.as_str()).collect();
        for edge in &atomized.edges {
            assert!(
                alo_ids.contains(edge.from.as_str()),
                "Edge 'from' id '{}' does not reference a known ALO",
                edge.from
            );
            assert!(
                alo_ids.contains(edge.to.as_str()),
                "Edge 'to' id '{}' does not reference a known ALO",
                edge.to
            );
        }
    }

    #[test]
    fn test_atomize_hook_has_no_inbound_prereq() {
        let pathway = sample_pathway();
        let atomized = atomize(&pathway);
        assert!(atomized.is_ok());
        let atomized = atomized.ok();
        assert!(atomized.is_some());
        let atomized = atomized.as_ref();

        if let Some(a) = atomized {
            let hooks: Vec<&AtomicLearningObject> = a
                .alos
                .iter()
                .filter(|alo| alo.alo_type == AloType::Hook)
                .collect();

            // First hook should have no inbound prereqs
            let first_hook = &hooks[0];
            let inbound_prereqs = a
                .edges
                .iter()
                .filter(|e| e.to == first_hook.id && e.edge_type == AloEdgeType::Prereq)
                .count();
            assert_eq!(
                inbound_prereqs, 0,
                "First hook should have no inbound prereqs"
            );
        }
    }

    #[test]
    fn test_atomize_missing_stages() {
        let bad = serde_json::json!({"id": "bad-01", "title": "Bad"});
        let result = atomize(&bad);
        assert!(result.is_err());
    }

    #[test]
    fn test_inter_stage_linking() {
        let pathway = sample_pathway();
        let atomized = atomize(&pathway);
        assert!(atomized.is_ok());
        let atomized = atomized.ok();
        assert!(atomized.is_some());

        if let Some(a) = &atomized {
            // Find stage 2 hook
            let stage2_hook = a
                .alos
                .iter()
                .find(|alo| alo.alo_type == AloType::Hook && alo.source_stage_id == "test-01-02");
            assert!(stage2_hook.is_some(), "Should have a hook for stage 2");

            // Stage 2 hook should have an inbound Extends edge from stage 1
            if let Some(hook) = stage2_hook {
                let has_extends = a
                    .edges
                    .iter()
                    .any(|e| e.to == hook.id && e.edge_type == AloEdgeType::Extends);
                assert!(
                    has_extends,
                    "Stage 2 hook should have Extends edge from stage 1"
                );
            }
        }
    }
}
