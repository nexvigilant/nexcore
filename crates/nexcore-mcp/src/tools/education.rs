//! Education Machine MCP tools — Bayesian mastery, 5-phase FSM, spaced repetition.
//!
//! # T1 Grounding
//! - σ (sequence): Curriculum ordering, phase progression
//! - μ (mapping): Concepts → lessons, domain → primitives
//! - ρ (recursion): Spaced repetition review cycles
//! - ς (state): Learner state, enrollment tracking
//! - N (quantity): Mastery probability, difficulty, grades
//! - κ (comparison): Assessment, verdict thresholds

use nexcore_education_machine::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    EduAssessParams, EduBayesianUpdateParams, EduEnrollParams, EduLearnerCreateParams,
    EduLessonAddStepParams, EduLessonCreateParams, EduMasteryParams, EduPhaseInfoParams,
    EduPhaseTransitionParams, EduPrimitiveMapParams, EduReviewCreateParams,
    EduReviewScheduleParams, EduReviewStatusParams, EduSubjectCreateParams,
};

/// Create a new subject.
pub fn subject_create(params: EduSubjectCreateParams) -> Result<CallToolResult, McpError> {
    let id = params
        .name
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let mut subject = Subject::new(&id, &params.name, &params.description);
    for tag in &params.tags {
        subject.add_tag(tag);
    }

    let response = serde_json::json!({
        "id": subject.id,
        "name": subject.name,
        "description": subject.description,
        "tags": subject.tags,
        "lesson_count": subject.lesson_count(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// List available demo subjects.
pub fn subject_list() -> Result<CallToolResult, McpError> {
    let subjects = vec![
        serde_json::json!({
            "id": "pv-signal-detection",
            "name": "PV Signal Detection",
            "description": "Bayesian and frequentist methods for pharmacovigilance signal detection (PRR, ROR, IC, EBGM)",
            "tags": ["pharmacovigilance", "signal-detection", "bayesian"],
            "lesson_count": 5,
        }),
        serde_json::json!({
            "id": "lex-primitiva",
            "name": "Lex Primitiva",
            "description": "The 16 T1 primitive symbols that ground all NexCore types",
            "tags": ["primitives", "type-theory", "foundations"],
            "lesson_count": 4,
        }),
        serde_json::json!({
            "id": "bayesian-methods",
            "name": "Bayesian Methods",
            "description": "Beta distributions, prior/posterior updates, and mastery estimation",
            "tags": ["statistics", "bayesian", "mathematics"],
            "lesson_count": 3,
        }),
        serde_json::json!({
            "id": "rust-ownership",
            "name": "Rust Ownership & Borrowing",
            "description": "Move semantics, borrowing rules, and lifetime annotations in Rust",
            "tags": ["rust", "programming", "memory-safety"],
            "lesson_count": 6,
        }),
    ];

    let response = serde_json::json!({
        "subjects": subjects,
        "total": subjects.len(),
        "note": "Demo catalog — create custom subjects with edu_subject_create",
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Create a new lesson.
pub fn lesson_create(params: EduLessonCreateParams) -> Result<CallToolResult, McpError> {
    let difficulty = Difficulty::new(params.difficulty)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let id = format!(
        "{}-{}",
        params.subject_id,
        params
            .title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
    );

    let mut lesson = Lesson::new(&id, &params.title, &params.subject_id, difficulty);
    if let Some(ref desc) = params.description {
        lesson.description = desc.clone();
    }

    let response = serde_json::json!({
        "id": lesson.id,
        "title": lesson.title,
        "subject_id": lesson.subject_id,
        "difficulty": lesson.difficulty.value(),
        "step_count": lesson.step_count(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Add a step to a lesson.
pub fn lesson_add_step(params: EduLessonAddStepParams) -> Result<CallToolResult, McpError> {
    // We create a temp lesson to demonstrate step construction
    let mut lesson = Lesson::new(&params.lesson_id, "temp", "temp", Difficulty::MEDIUM);

    match params.step_type.to_lowercase().as_str() {
        "text" => {
            let body = params.body.as_deref().unwrap_or("");
            lesson.add_text_step(&params.title, body);
        }
        "exercise" => {
            let prompt = params.prompt.as_deref().unwrap_or("");
            let solution = params.solution.as_deref().unwrap_or("");
            lesson.add_exercise_step(&params.title, prompt, solution);
        }
        "decomposition" => {
            let concept = params.concept.as_deref().unwrap_or("unknown");
            let mapping = PrimitiveMapping {
                concept: concept.to_string(),
                tier: "T2-P".to_string(),
                primitives: vec!["σ".to_string()],
                dominant: "σ".to_string(),
            };
            lesson.add_decomposition_step(&params.title, concept, mapping);
        }
        other => {
            return Err(McpError::invalid_params(
                format!("Unknown step type: {other}. Valid: text, exercise, decomposition"),
                None,
            ));
        }
    }

    let step = &lesson.steps[0];
    let response = serde_json::json!({
        "lesson_id": params.lesson_id,
        "step_type": params.step_type,
        "title": step.title,
        "order": step.order,
        "message": format!("Added {} step to lesson {}", params.step_type, params.lesson_id),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Create a new learner.
pub fn learner_create(params: EduLearnerCreateParams) -> Result<CallToolResult, McpError> {
    let learner = Learner::new(&params.learner_id, &params.name, 0.0);

    let response = serde_json::json!({
        "id": learner.id,
        "name": learner.name,
        "enrollment_count": learner.enrollment_count(),
        "average_mastery": learner.average_mastery(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Enroll a learner in a subject.
pub fn enroll(params: EduEnrollParams) -> Result<CallToolResult, McpError> {
    let mut learner = Learner::new(&params.learner_id, "learner", 0.0);
    learner.enroll(&params.subject_id, 0.0);

    let enrollment = learner.enrollment(&params.subject_id);
    let response = if let Some(e) = enrollment {
        serde_json::json!({
            "learner_id": params.learner_id,
            "subject_id": params.subject_id,
            "phase": e.phase,
            "mastery_probability": e.mastery_probability(),
            "verdict": e.current_verdict().to_string(),
            "competency": e.competency().to_string(),
            "message": format!("Enrolled {} in {}", params.learner_id, params.subject_id),
        })
    } else {
        serde_json::json!({
            "error": "Failed to enroll",
        })
    };
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Run a Bayesian assessment.
pub fn assess(params: EduAssessParams) -> Result<CallToolResult, McpError> {
    let alpha = params.alpha.unwrap_or(1.0);
    let beta = params.beta.unwrap_or(1.0);
    let mut prior = BayesianPrior::new(alpha, beta);

    let question_results: Vec<QuestionResult> = params
        .results
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let difficulty =
                Difficulty::new(item.difficulty.clamp(0.0, 1.0)).unwrap_or(Difficulty::MEDIUM);
            QuestionResult {
                question_id: format!("q{i}"),
                correct: item.correct,
                given_answer: if item.correct {
                    "correct".to_string()
                } else {
                    "incorrect".to_string()
                },
                difficulty,
            }
        })
        .collect();

    let result = evaluate_assessment(&params.subject_id, &mut prior, &question_results)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let response = serde_json::json!({
        "subject_id": result.subject_id,
        "mastery_probability": result.mastery.value(),
        "verdict": result.verdict.to_string(),
        "correct_count": result.correct_count,
        "total_count": result.total_count,
        "accuracy": if result.total_count > 0 {
            result.correct_count as f64 / result.total_count as f64
        } else {
            0.0
        },
        "prior": {
            "alpha": prior.alpha,
            "beta": prior.beta,
            "confidence": prior.confidence(),
        },
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Query mastery verdict from a probability value.
pub fn mastery(params: EduMasteryParams) -> Result<CallToolResult, McpError> {
    let level = MasteryLevel::new(params.mastery_value)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let verdict = level.verdict();
    let competency = CompetencyLevel::from_mastery(params.mastery_value);

    let response = serde_json::json!({
        "mastery_value": params.mastery_value,
        "mastery_percent": format!("{:.1}%", params.mastery_value * 100.0),
        "verdict": verdict.to_string(),
        "competency": competency.to_string(),
        "thresholds": {
            "mastered": MASTERY_THRESHOLD,
            "developing": DEVELOPING_THRESHOLD,
        },
        "color_class": verdict.color_class(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Execute a learning phase transition.
pub fn phase_transition(params: EduPhaseTransitionParams) -> Result<CallToolResult, McpError> {
    let from = parse_learning_phase(&params.from)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let to = parse_learning_phase(&params.to)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let can = can_transition(from, to);

    if !can {
        let suggestion = suggest_next_phase(from);
        let response = serde_json::json!({
            "valid": false,
            "from": from.to_string(),
            "to": to.to_string(),
            "error": format!("Transition {} -> {} is not allowed", from, to),
            "suggestion": suggestion.map(|p| p.to_string()),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]));
    }

    let reason = params.reason.as_deref().unwrap_or("requested");
    let transition = execute_transition(from, to, reason, 0.0)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let response = serde_json::json!({
        "valid": true,
        "from": transition.from.to_string(),
        "to": transition.to.to_string(),
        "reason": transition.reason,
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Query phase state information.
pub fn phase_info(params: EduPhaseInfoParams) -> Result<CallToolResult, McpError> {
    let phase = parse_learning_phase(&params.phase)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let next = suggest_next_phase(phase);
    let remaining = phases_remaining(phase);
    let completion = completion_percentage(phase);

    let all_phases: Vec<serde_json::Value> = LearningPhase::all()
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.to_string(),
                "ordinal": p.ordinal(),
                "current": *p == phase,
            })
        })
        .collect();

    let response = serde_json::json!({
        "current_phase": phase,
        "ordinal": phase.ordinal(),
        "next_phase": next.map(|p| p.to_string()),
        "phases_remaining": remaining,
        "completion_percent": completion,
        "all_phases": all_phases,
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Create a new spaced repetition review item.
pub fn review_create(params: EduReviewCreateParams) -> Result<CallToolResult, McpError> {
    let now = params.current_time.unwrap_or(0.0);
    let state = ReviewState::new(&params.item_id, now);

    let response = serde_json::json!({
        "item_id": state.item_id,
        "stability_hours": state.stability,
        "interval_hours": state.interval_hours,
        "review_count": state.review_count,
        "next_review": state.next_review,
        "last_review": state.last_review,
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Grade a review item and reschedule.
pub fn review_schedule(params: EduReviewScheduleParams) -> Result<CallToolResult, McpError> {
    let grade =
        parse_grade(&params.grade).map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    // Reconstruct review state from params
    let mut state = ReviewState::new(&params.item_id, params.last_review.unwrap_or(0.0));
    if let Some(stability) = params.stability {
        state.stability = stability;
    }
    if let Some(interval) = params.interval_hours {
        state.interval_hours = interval;
    }
    if let Some(count) = params.review_count {
        state.review_count = count;
    }

    state.schedule_review(grade, params.current_time);

    let response = serde_json::json!({
        "item_id": state.item_id,
        "grade": params.grade,
        "stability_hours": state.stability,
        "interval_hours": state.interval_hours,
        "review_count": state.review_count,
        "next_review": state.next_review,
        "last_review": state.last_review,
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Check review status (retrievability, due state).
pub fn review_status(params: EduReviewStatusParams) -> Result<CallToolResult, McpError> {
    // Reconstruct state from params
    let mut state = ReviewState::new(&params.item_id, params.last_review);
    state.stability = params.stability;
    state.interval_hours = params.interval_hours;

    let retrievability = state.retrievability(params.current_time);
    let due = state.due_for_review(params.current_time);
    let needs_reinforcement = state.needs_reinforcement(params.current_time);
    let hours_until = state.hours_until_review(params.current_time);

    let response = serde_json::json!({
        "item_id": params.item_id,
        "retrievability": retrievability,
        "retrievability_percent": format!("{:.1}%", retrievability * 100.0),
        "due_for_review": due,
        "needs_reinforcement": needs_reinforcement,
        "hours_until_review": hours_until,
        "stability_hours": params.stability,
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Update a Bayesian prior with a single observation.
pub fn bayesian_update(params: EduBayesianUpdateParams) -> Result<CallToolResult, McpError> {
    let difficulty = Difficulty::new(params.difficulty)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let mut prior = BayesianPrior::new(params.alpha, params.beta);
    let before_prob = prior.mastery_probability();

    if params.correct {
        prior.update_correct(difficulty);
    } else {
        prior.update_incorrect(difficulty);
    }

    let after_prob = prior.mastery_probability();
    let verdict = MasteryVerdict::from_level(after_prob);

    let response = serde_json::json!({
        "alpha": prior.alpha,
        "beta": prior.beta,
        "mastery_probability": after_prob,
        "previous_probability": before_prob,
        "delta": after_prob - before_prob,
        "verdict": verdict.to_string(),
        "confidence": prior.confidence(),
        "total_evidence": prior.total_evidence(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Map a domain concept to its T1/T2/T3 primitives.
pub fn primitive_map(params: EduPrimitiveMapParams) -> Result<CallToolResult, McpError> {
    let mapping = PrimitiveMapping {
        concept: params.concept.clone(),
        tier: params.tier.clone(),
        primitives: params.primitives.clone(),
        dominant: params.dominant.clone(),
    };

    let primitive_count = mapping.primitives.len();
    let computed_tier = match primitive_count {
        0 => "T0 (no primitives)",
        1 => "T1 (single primitive)",
        2..=3 => "T2-P (cross-domain primitive)",
        4..=5 => "T2-C (cross-domain composite)",
        _ => "T3 (domain-specific)",
    };

    let response = serde_json::json!({
        "concept": mapping.concept,
        "tier": mapping.tier,
        "computed_tier": computed_tier,
        "primitives": mapping.primitives,
        "dominant": mapping.dominant,
        "primitive_count": primitive_count,
        "tier_consistent": mapping.tier == computed_tier.split(' ').next().unwrap_or(""),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ── Parsers ─────────────────────────────────────────────────────────────────

fn parse_learning_phase(s: &str) -> Result<LearningPhase, nexcore_error::NexError> {
    match s.to_lowercase().as_str() {
        "discover" => Ok(LearningPhase::Discover),
        "extract" => Ok(LearningPhase::Extract),
        "practice" => Ok(LearningPhase::Practice),
        "assess" => Ok(LearningPhase::Assess),
        "master" => Ok(LearningPhase::Master),
        _ => Err(nexcore_error::nexerror!(
            "Unknown phase: {s}. Valid: discover, extract, practice, assess, master"
        )),
    }
}

fn parse_grade(s: &str) -> Result<Grade, nexcore_error::NexError> {
    match s.to_lowercase().as_str() {
        "again" => Ok(Grade::Again),
        "hard" => Ok(Grade::Hard),
        "good" => Ok(Grade::Good),
        "easy" => Ok(Grade::Easy),
        _ => Err(nexcore_error::nexerror!(
            "Unknown grade: {s}. Valid: again, hard, good, easy"
        )),
    }
}
