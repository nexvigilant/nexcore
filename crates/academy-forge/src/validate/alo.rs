//! ALO-specific validation rules (R28-R36).
//!
//! Validates atomized pathways for:
//! - Duration ranges per ALO type (R28)
//! - Single learning objective with Bloom verb (R29)
//! - Valid ALO type (R30)
//! - Edge target validity (R31)
//! - DAG acyclicity (R32)
//! - Concept coverage by assessments (R33)
//! - Hook entry invariant (R34)
//! - Activity grounding (R35)
//! - KSB reference validity (R36)

use std::collections::{HashMap, HashSet, VecDeque};

use crate::ir::{AloEdgeType, AloType, AtomizedPathway, BloomLevel};

use super::{Severity, ValidationFinding};

/// Bloom verbs by level for R29 validation.
const BLOOM_VERBS: &[(&str, &[&str])] = &[
    (
        "Remember",
        &[
            "define",
            "list",
            "name",
            "recall",
            "identify",
            "state",
            "recognize",
            "describe",
            "label",
            "match",
        ],
    ),
    (
        "Understand",
        &[
            "explain",
            "describe",
            "summarize",
            "interpret",
            "classify",
            "compare",
            "discuss",
            "distinguish",
            "paraphrase",
        ],
    ),
    (
        "Apply",
        &[
            "apply",
            "calculate",
            "demonstrate",
            "solve",
            "use",
            "implement",
            "execute",
            "practice",
            "compute",
        ],
    ),
    (
        "Analyze",
        &[
            "analyze",
            "differentiate",
            "examine",
            "distinguish",
            "investigate",
            "compare",
            "contrast",
            "deconstruct",
        ],
    ),
    (
        "Evaluate",
        &[
            "evaluate",
            "assess",
            "justify",
            "critique",
            "judge",
            "defend",
            "determine",
            "appraise",
        ],
    ),
    (
        "Create",
        &[
            "create",
            "design",
            "develop",
            "construct",
            "propose",
            "formulate",
            "produce",
            "compose",
        ],
    ),
];

/// Validate an atomized pathway against ALO rules R28-R36.
pub fn validate_alo(pathway: &AtomizedPathway) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();

    // Build lookup structures
    let alo_ids: HashSet<&str> = pathway.alos.iter().map(|a| a.id.as_str()).collect();

    // R28: Duration range per ALO type
    for alo in &pathway.alos {
        let min = alo.alo_type.min_duration();
        let max = alo.alo_type.max_duration();
        if alo.estimated_duration < min || alo.estimated_duration > max {
            findings.push(ValidationFinding {
                rule: "R28".to_string(),
                severity: Severity::Error,
                message: format!(
                    "ALO '{}' ({:?}) duration {} min outside range [{}-{}]",
                    alo.id, alo.alo_type, alo.estimated_duration, min, max
                ),
                field_path: Some(format!("alos[id={}].estimated_duration", alo.id)),
            });
        }
    }

    // R29: Single learning objective with Bloom verb
    for alo in &pathway.alos {
        if alo.learning_objective.is_empty() {
            findings.push(ValidationFinding {
                rule: "R29".to_string(),
                severity: Severity::Error,
                message: format!("ALO '{}' has empty learning_objective", alo.id),
                field_path: Some(format!("alos[id={}].learning_objective", alo.id)),
            });
        } else {
            let first_word = alo
                .learning_objective
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();

            let valid_verb = BLOOM_VERBS
                .iter()
                .any(|(_, verbs)| verbs.contains(&first_word.as_str()));

            if !valid_verb {
                findings.push(ValidationFinding {
                    rule: "R29".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "ALO '{}' learning_objective does not start with a recognized Bloom verb (got '{}')",
                        alo.id, first_word
                    ),
                    field_path: Some(format!("alos[id={}].learning_objective", alo.id)),
                });
            }
        }
    }

    // R30: Valid ALO type (enforced by Rust enum, but check assessments match)
    for alo in &pathway.alos {
        if alo.alo_type == AloType::Reflection && alo.assessment.is_none() {
            findings.push(ValidationFinding {
                rule: "R30".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "Reflection ALO '{}' has no assessment data",
                    alo.id
                ),
                field_path: Some(format!("alos[id={}].assessment", alo.id)),
            });
        }
    }

    // R31: Edge targets reference valid ALO IDs
    for edge in &pathway.edges {
        if !alo_ids.contains(edge.from.as_str()) {
            findings.push(ValidationFinding {
                rule: "R31".to_string(),
                severity: Severity::Error,
                message: format!(
                    "Edge from '{}' references nonexistent ALO",
                    edge.from
                ),
                field_path: Some(format!("edges[from={}]", edge.from)),
            });
        }
        if !alo_ids.contains(edge.to.as_str()) {
            findings.push(ValidationFinding {
                rule: "R31".to_string(),
                severity: Severity::Error,
                message: format!(
                    "Edge to '{}' references nonexistent ALO",
                    edge.to
                ),
                field_path: Some(format!("edges[to={}]", edge.to)),
            });
        }
    }

    // R32: DAG acyclicity (Prereq subgraph)
    if has_cycle(pathway) {
        findings.push(ValidationFinding {
            rule: "R32".to_string(),
            severity: Severity::Error,
            message: "Cycle detected in Prereq edge subgraph".to_string(),
            field_path: None,
        });
    }

    // R33: Concept coverage — every Concept ALO should be assessed
    let assessed_ids: HashSet<&str> = pathway
        .edges
        .iter()
        .filter(|e| e.edge_type == AloEdgeType::Assesses)
        .map(|e| e.to.as_str())
        .collect();

    for alo in &pathway.alos {
        if alo.alo_type == AloType::Concept && !assessed_ids.contains(alo.id.as_str()) {
            findings.push(ValidationFinding {
                rule: "R33".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "Concept ALO '{}' has no Reflection ALO that assesses it",
                    alo.id
                ),
                field_path: Some(format!("alos[id={}]", alo.id)),
            });
        }
    }

    // R34: Hook ALOs have zero inbound Prereq edges
    let inbound_prereqs: HashMap<&str, usize> = {
        let mut map: HashMap<&str, usize> = HashMap::new();
        for edge in &pathway.edges {
            if edge.edge_type == AloEdgeType::Prereq {
                *map.entry(edge.to.as_str()).or_insert(0) += 1;
            }
        }
        map
    };

    for alo in &pathway.alos {
        if alo.alo_type == AloType::Hook {
            let count = inbound_prereqs.get(alo.id.as_str()).copied().unwrap_or(0);
            if count > 0 {
                findings.push(ValidationFinding {
                    rule: "R34".to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "Hook ALO '{}' has {} inbound Prereq edges (should have 0)",
                        alo.id, count
                    ),
                    field_path: Some(format!("alos[id={}]", alo.id)),
                });
            }
        }
    }

    // R35: Activity ALOs have at least 1 Concept prereq
    let prereq_sources: HashMap<&str, Vec<&str>> = {
        let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &pathway.edges {
            if edge.edge_type == AloEdgeType::Prereq {
                map.entry(edge.to.as_str())
                    .or_default()
                    .push(edge.from.as_str());
            }
        }
        map
    };

    let concept_ids: HashSet<&str> = pathway
        .alos
        .iter()
        .filter(|a| a.alo_type == AloType::Concept)
        .map(|a| a.id.as_str())
        .collect();

    for alo in &pathway.alos {
        if alo.alo_type == AloType::Activity {
            let sources = prereq_sources.get(alo.id.as_str());
            let has_concept_prereq = sources.map_or(false, |srcs| {
                srcs.iter().any(|s| concept_ids.contains(s))
            });

            if !has_concept_prereq {
                findings.push(ValidationFinding {
                    rule: "R35".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "Activity ALO '{}' has no Concept ALO as prerequisite",
                        alo.id
                    ),
                    field_path: Some(format!("alos[id={}]", alo.id)),
                });
            }
        }
    }

    // R36: KSB refs validity (advisory — can't check against library here)
    for alo in &pathway.alos {
        if alo.ksb_refs.len() > 5 {
            findings.push(ValidationFinding {
                rule: "R36".to_string(),
                severity: Severity::Advisory,
                message: format!(
                    "ALO '{}' has {} KSB refs (recommended max 5)",
                    alo.id,
                    alo.ksb_refs.len()
                ),
                field_path: Some(format!("alos[id={}].ksb_refs", alo.id)),
            });
        }
    }

    findings
}

/// Check for cycles in the Prereq subgraph using Kahn's algorithm.
fn has_cycle(pathway: &AtomizedPathway) -> bool {
    let node_ids: HashSet<&str> = pathway.alos.iter().map(|a| a.id.as_str()).collect();
    let prereq_edges: Vec<_> = pathway
        .edges
        .iter()
        .filter(|e| e.edge_type == AloEdgeType::Prereq)
        .collect();

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for &id in &node_ids {
        in_degree.insert(id, 0);
        adj.insert(id, Vec::new());
    }

    for edge in &prereq_edges {
        if node_ids.contains(edge.from.as_str()) && node_ids.contains(edge.to.as_str()) {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
            adj.entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }
    }

    let mut queue: VecDeque<&str> = VecDeque::new();
    for (&node, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(node);
        }
    }

    let mut visited = 0usize;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    visited < node_ids.len()
}

/// Validate an atomized pathway and return a full report.
pub fn validate_alo_report(
    pathway: &AtomizedPathway,
) -> super::ValidationReport {
    let findings = validate_alo(pathway);

    let error_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Error))
        .count();
    let warning_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Warning))
        .count();
    let advisory_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Advisory))
        .count();

    super::ValidationReport {
        passed: error_count == 0,
        total_findings: findings.len(),
        error_count,
        warning_count,
        advisory_count,
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn valid_atomized() -> AtomizedPathway {
        AtomizedPathway {
            id: "test-01".to_string(),
            title: "Test".to_string(),
            source_pathway_id: "test-01".to_string(),
            alos: vec![
                AtomicLearningObject {
                    id: "test-01-01-h01".to_string(),
                    title: "Hook".to_string(),
                    alo_type: AloType::Hook,
                    learning_objective: "Recognize the importance of testing".to_string(),
                    estimated_duration: 2,
                    bloom_level: BloomLevel::Remember,
                    content: "Why testing matters.".to_string(),
                    ksb_refs: Vec::new(),
                    source_stage_id: "test-01-01".to_string(),
                    source_activity_id: None,
                    assessment: None,
                },
                AtomicLearningObject {
                    id: "test-01-01-c01".to_string(),
                    title: "Concept".to_string(),
                    alo_type: AloType::Concept,
                    learning_objective: "Explain the fundamentals of testing".to_string(),
                    estimated_duration: 7,
                    bloom_level: BloomLevel::Understand,
                    content: "Testing fundamentals.".to_string(),
                    ksb_refs: vec!["KSB-001".to_string()],
                    source_stage_id: "test-01-01".to_string(),
                    source_activity_id: None,
                    assessment: None,
                },
                AtomicLearningObject {
                    id: "test-01-01-a01".to_string(),
                    title: "Activity".to_string(),
                    alo_type: AloType::Activity,
                    learning_objective: "Apply testing techniques".to_string(),
                    estimated_duration: 10,
                    bloom_level: BloomLevel::Apply,
                    content: "Practice exercise.".to_string(),
                    ksb_refs: Vec::new(),
                    source_stage_id: "test-01-01".to_string(),
                    source_activity_id: None,
                    assessment: None,
                },
                AtomicLearningObject {
                    id: "test-01-01-r01".to_string(),
                    title: "Reflection".to_string(),
                    alo_type: AloType::Reflection,
                    learning_objective: "Evaluate your understanding".to_string(),
                    estimated_duration: 4,
                    bloom_level: BloomLevel::Evaluate,
                    content: String::new(),
                    ksb_refs: Vec::new(),
                    source_stage_id: "test-01-01".to_string(),
                    source_activity_id: None,
                    assessment: Some(AloAssessment {
                        passing_score: 70,
                        questions: vec![serde_json::json!({"id": "q1"})],
                    }),
                },
            ],
            edges: vec![
                AloEdge {
                    from: "test-01-01-h01".to_string(),
                    to: "test-01-01-c01".to_string(),
                    edge_type: AloEdgeType::Prereq,
                    strength: 1.0,
                },
                AloEdge {
                    from: "test-01-01-c01".to_string(),
                    to: "test-01-01-a01".to_string(),
                    edge_type: AloEdgeType::Prereq,
                    strength: 1.0,
                },
                AloEdge {
                    from: "test-01-01-a01".to_string(),
                    to: "test-01-01-r01".to_string(),
                    edge_type: AloEdgeType::Prereq,
                    strength: 1.0,
                },
                AloEdge {
                    from: "test-01-01-r01".to_string(),
                    to: "test-01-01-c01".to_string(),
                    edge_type: AloEdgeType::Assesses,
                    strength: 1.0,
                },
            ],
        }
    }

    #[test]
    fn test_valid_pathway_passes() {
        let pathway = valid_atomized();
        let report = validate_alo_report(&pathway);
        assert!(
            report.passed,
            "Valid pathway should pass. Errors: {:?}",
            report
                .findings
                .iter()
                .filter(|f| matches!(f.severity, Severity::Error))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_r28_duration_out_of_range() {
        let mut pathway = valid_atomized();
        pathway.alos[0].estimated_duration = 10; // Hook max is 3
        let findings = validate_alo(&pathway);
        assert!(findings.iter().any(|f| f.rule == "R28"));
    }

    #[test]
    fn test_r32_cycle_detected() {
        let mut pathway = valid_atomized();
        // Add a cycle: r01 → h01 (Prereq)
        pathway.edges.push(AloEdge {
            from: "test-01-01-r01".to_string(),
            to: "test-01-01-h01".to_string(),
            edge_type: AloEdgeType::Prereq,
            strength: 1.0,
        });
        let findings = validate_alo(&pathway);
        assert!(findings.iter().any(|f| f.rule == "R32"));
    }

    #[test]
    fn test_r34_hook_with_inbound_prereq() {
        let mut pathway = valid_atomized();
        // First hook should normally have 0 inbound prereqs, but let's leave R34 alone
        // since our valid pathway already has the hook without inbound prereqs.
        // Instead, test that the valid one passes R34:
        let findings = validate_alo(&pathway);
        let r34_errors: Vec<_> = findings.iter().filter(|f| f.rule == "R34").collect();
        assert!(r34_errors.is_empty(), "Hook should have no R34 errors");

        // Now add a bad prereq edge TO the hook
        pathway.edges.push(AloEdge {
            from: "test-01-01-c01".to_string(),
            to: "test-01-01-h01".to_string(),
            edge_type: AloEdgeType::Prereq,
            strength: 1.0,
        });
        let findings = validate_alo(&pathway);
        assert!(findings.iter().any(|f| f.rule == "R34"));
    }
}
