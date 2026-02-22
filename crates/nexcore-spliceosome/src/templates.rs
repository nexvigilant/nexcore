// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Default EJC templates per task category.
//!
//! ## Primitive Grounding: sigma(Sequence) + mapping(mu)
//!
//! Templates map (mu) task categories to ordered sequences (sigma)
//! of EJC markers representing expected execution structure.
//!
//! These are initial templates derived from empirical observation.
//! The adaptive learning loop (Phase 7) refines them over time.

use std::collections::HashMap;

use crate::types::{EjcMarker, TaskCategory};

/// Build default EJC templates for all task categories.
///
/// Templates encode the expected phase structure for each category.
/// Derived from Phase 0 analysis of 151K tool calls across 17 sessions.
#[must_use]
pub fn default_templates() -> HashMap<TaskCategory, Vec<EjcMarker>> {
    let mut templates = HashMap::new();

    templates.insert(TaskCategory::Explore, vec![
        EjcMarker {
            phase_id: "investigate".into(),
            expected_tool_categories: vec![TaskCategory::Explore],
            grounding_confidence_threshold: 0.0,
            max_calls_before_checkpoint: 10,
            expected_confidence_range: (0.3, 0.7),
            skippable: false,
        },
        EjcMarker {
            phase_id: "synthesize".into(),
            expected_tool_categories: vec![TaskCategory::Explore],
            grounding_confidence_threshold: 0.2,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.5, 0.9),
            skippable: true,
        },
    ]);

    templates.insert(TaskCategory::Mutate, vec![
        EjcMarker {
            phase_id: "investigate".into(),
            expected_tool_categories: vec![TaskCategory::Explore],
            grounding_confidence_threshold: 0.1,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.3, 0.7),
            skippable: false,
        },
        EjcMarker {
            phase_id: "implement".into(),
            expected_tool_categories: vec![TaskCategory::Mutate],
            grounding_confidence_threshold: 0.3,
            max_calls_before_checkpoint: 8,
            expected_confidence_range: (0.5, 0.9),
            skippable: false,
        },
        EjcMarker {
            phase_id: "verify".into(),
            expected_tool_categories: vec![TaskCategory::Verify],
            grounding_confidence_threshold: 0.5,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.7, 1.0),
            skippable: false,
        },
    ]);

    templates.insert(TaskCategory::Orchestrate, vec![
        EjcMarker {
            phase_id: "plan".into(),
            expected_tool_categories: vec![TaskCategory::Explore],
            grounding_confidence_threshold: 0.2,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.4, 0.8),
            skippable: false,
        },
        EjcMarker {
            phase_id: "delegate".into(),
            expected_tool_categories: vec![TaskCategory::Orchestrate],
            grounding_confidence_threshold: 0.3,
            max_calls_before_checkpoint: 10,
            expected_confidence_range: (0.5, 0.9),
            skippable: false,
        },
        EjcMarker {
            phase_id: "converge".into(),
            expected_tool_categories: vec![TaskCategory::Orchestrate, TaskCategory::Verify],
            grounding_confidence_threshold: 0.5,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.7, 1.0),
            skippable: false,
        },
    ]);

    templates.insert(TaskCategory::Compute, vec![
        EjcMarker {
            phase_id: "gather".into(),
            expected_tool_categories: vec![TaskCategory::Explore, TaskCategory::Compute],
            grounding_confidence_threshold: 0.5,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.4, 0.8),
            skippable: false,
        },
        EjcMarker {
            phase_id: "compute".into(),
            expected_tool_categories: vec![TaskCategory::Compute],
            grounding_confidence_threshold: 0.7,
            max_calls_before_checkpoint: 8,
            expected_confidence_range: (0.6, 0.95),
            skippable: false,
        },
        EjcMarker {
            phase_id: "validate".into(),
            expected_tool_categories: vec![TaskCategory::Verify, TaskCategory::Compute],
            grounding_confidence_threshold: 0.8,
            max_calls_before_checkpoint: 3,
            expected_confidence_range: (0.8, 1.0),
            skippable: false,
        },
    ]);

    templates.insert(TaskCategory::Verify, vec![
        EjcMarker {
            phase_id: "setup".into(),
            expected_tool_categories: vec![TaskCategory::Explore, TaskCategory::Mutate],
            grounding_confidence_threshold: 0.1,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.3, 0.7),
            skippable: true,
        },
        EjcMarker {
            phase_id: "execute".into(),
            expected_tool_categories: vec![TaskCategory::Verify],
            grounding_confidence_threshold: 0.5,
            max_calls_before_checkpoint: 10,
            expected_confidence_range: (0.5, 1.0),
            skippable: false,
        },
    ]);

    templates.insert(TaskCategory::Browse, vec![
        EjcMarker {
            phase_id: "navigate".into(),
            expected_tool_categories: vec![TaskCategory::Browse],
            grounding_confidence_threshold: 0.1,
            max_calls_before_checkpoint: 8,
            expected_confidence_range: (0.3, 0.8),
            skippable: false,
        },
        EjcMarker {
            phase_id: "interact".into(),
            expected_tool_categories: vec![TaskCategory::Browse],
            grounding_confidence_threshold: 0.2,
            max_calls_before_checkpoint: 15,
            expected_confidence_range: (0.4, 0.9),
            skippable: true,
        },
    ]);

    templates.insert(TaskCategory::Mixed, vec![
        EjcMarker {
            phase_id: "investigate".into(),
            expected_tool_categories: vec![TaskCategory::Explore],
            grounding_confidence_threshold: 0.2,
            max_calls_before_checkpoint: 8,
            expected_confidence_range: (0.3, 0.7),
            skippable: false,
        },
        EjcMarker {
            phase_id: "execute".into(),
            expected_tool_categories: vec![
                TaskCategory::Mutate,
                TaskCategory::Compute,
                TaskCategory::Orchestrate,
            ],
            grounding_confidence_threshold: 0.4,
            max_calls_before_checkpoint: 10,
            expected_confidence_range: (0.5, 0.9),
            skippable: false,
        },
        EjcMarker {
            phase_id: "integrate".into(),
            expected_tool_categories: vec![TaskCategory::Verify],
            grounding_confidence_threshold: 0.5,
            max_calls_before_checkpoint: 5,
            expected_confidence_range: (0.6, 1.0),
            skippable: true,
        },
    ]);

    templates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_categories_have_templates() {
        let templates = default_templates();
        for cat in TaskCategory::ALL {
            assert!(
                templates.contains_key(&cat),
                "Missing template for {cat:?}"
            );
            assert!(
                !templates[&cat].is_empty(),
                "Empty template for {cat:?}"
            );
        }
    }

    #[test]
    fn test_mutate_has_verify_phase() {
        let templates = default_templates();
        let mutate = &templates[&TaskCategory::Mutate];
        let has_verify = mutate.iter().any(|m| {
            m.expected_tool_categories.contains(&TaskCategory::Verify)
        });
        assert!(has_verify, "Mutate template must include a verify phase");
    }

    #[test]
    fn test_grounding_thresholds_in_range() {
        let templates = default_templates();
        for (cat, markers) in &templates {
            for m in markers {
                assert!(
                    (0.0..=1.0).contains(&m.grounding_confidence_threshold),
                    "Grounding threshold out of range for {cat:?}/{}", m.phase_id
                );
            }
        }
    }
}
