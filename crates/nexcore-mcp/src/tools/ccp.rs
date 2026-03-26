//! Claude Care Process (CCP) MCP tools — pharmacokinetic engine for AI support.
//!
//! # T1 Grounding
//! - σ (sequence): Episode lifecycle flows through 5 phases
//! - ∝ (proportionality): PK math governs dose-response
//! - κ (comparison): Quality scoring compares factors

use nexcore_ccp::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    CcpDoseComputeParams, CcpEpisodeAdvanceParams, CcpEpisodeStartParams,
    CcpInteractionCheckParams, CcpPhaseTransitionParams, CcpQualityScoreParams,
};

/// Start a new care episode at the Collect phase.
pub fn episode_start(params: CcpEpisodeStartParams) -> Result<CallToolResult, McpError> {
    let ep = Episode::new(&params.episode_id, params.started_at.unwrap_or(0.0));

    let response = serde_json::json!({
        "episode_id": ep.id,
        "phase": ep.phase,
        "plasma_level": ep.plasma_level.value(),
        "message": "Episode started at Collect phase"
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Compute recommended dose given strategy and target.
pub fn dose_compute(params: CcpDoseComputeParams) -> Result<CallToolResult, McpError> {
    let strategy = parse_strategy(&params.strategy)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let target = PlasmaLevel(params.target_level);

    let bio = BioAvailability::new(params.bioavailability.unwrap_or(0.8))
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let hl = HalfLife::new(params.half_life.unwrap_or(24.0))
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let dose = match strategy {
        DosingStrategy::Loading => compute_loading_dose(target, bio)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?,
        DosingStrategy::Maintenance => compute_maintenance_dose(hl, target)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?,
        _ => {
            let current = PlasmaLevel(params.current_level.unwrap_or(0.0));
            titrate(current, target, strategy)
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?
        }
    };

    let response = serde_json::json!({
        "dose": dose.value(),
        "strategy": params.strategy,
        "target_level": params.target_level,
        "bioavailability": bio.value(),
        "half_life_hours": hl.value(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Administer dose, apply decay, and optionally transition phase.
pub fn episode_advance(params: CcpEpisodeAdvanceParams) -> Result<CallToolResult, McpError> {
    let mut ep = Episode::new(&params.episode_id, 0.0);

    // Set initial phase
    let initial_phase = parse_phase(&params.current_phase)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    ep.phase = initial_phase;
    ep.plasma_level = PlasmaLevel(params.current_plasma.unwrap_or(0.0));

    // Administer dose if provided
    if let Some(dose_val) = params.dose {
        let dose =
            Dose::new(dose_val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let bio = BioAvailability::new(params.bioavailability.unwrap_or(0.8))
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let hl = HalfLife::new(params.half_life.unwrap_or(24.0))
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let strategy = parse_strategy(
            &params
                .strategy
                .clone()
                .unwrap_or_else(|| "therapeutic".to_string()),
        )
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let intervention = Intervention {
            dose,
            bioavailability: bio,
            half_life: hl,
            strategy,
            administered_at: params.timestamp.unwrap_or(0.0),
        };
        ep.administer(intervention);
    }

    // Apply decay
    if let Some(decay_hours) = params.decay_hours {
        ep.decay(decay_hours);
    }

    // Transition phase
    if let Some(ref target_phase) = params.target_phase {
        let target =
            parse_phase(target_phase).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let reason = params.reason.as_deref().unwrap_or("advanced");
        ep.advance_phase(target, reason, params.timestamp.unwrap_or(0.0))
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    }

    let response = serde_json::json!({
        "episode_id": ep.id,
        "phase": ep.phase,
        "plasma_level": ep.plasma_level.value(),
        "interventions": ep.intervention_count(),
        "transitions": ep.transitions.len(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Check interaction effects between two plasma levels.
pub fn interaction_check(params: CcpInteractionCheckParams) -> Result<CallToolResult, McpError> {
    let itype = parse_interaction_type(&params.interaction_type)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let a = PlasmaLevel(params.level_a);
    let b = PlasmaLevel(params.level_b);

    let effect = compute_interaction(a, b, itype);

    let response = serde_json::json!({
        "interaction_type": params.interaction_type,
        "magnitude": effect.magnitude,
        "combined_level": effect.combined_level.value(),
        "individual_sum": a.value() + b.value(),
        "amplification_factor": if a.value() + b.value() > 0.0 {
            effect.combined_level.value() / (a.value() + b.value())
        } else {
            1.0
        },
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Score episode quality on [0, 10] scale.
pub fn quality_score(params: CcpQualityScoreParams) -> Result<CallToolResult, McpError> {
    let mut ep = Episode::new("score-episode", 0.0);

    // Build episode from params
    let bio = BioAvailability::new(params.avg_bioavailability.unwrap_or(0.8))
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let hl = HalfLife::new(params.avg_half_life.unwrap_or(24.0))
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let dose = Dose::new(params.dose.unwrap_or(0.5))
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    ep.administer(Intervention {
        dose,
        bioavailability: bio,
        half_life: hl,
        strategy: DosingStrategy::Therapeutic,
        administered_at: 0.0,
    });
    ep.plasma_level = PlasmaLevel(params.plasma_level);

    let score = score_episode(&ep).map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let response = serde_json::json!({
        "total": score.total,
        "rating": score.rating.to_string(),
        "components": {
            "bioavailability": score.components.bioavailability,
            "stability": score.components.stability,
            "safety_margin": score.components.safety_margin,
            "persistence": score.components.persistence,
        },
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Validate and execute a phase transition.
pub fn phase_transition(params: CcpPhaseTransitionParams) -> Result<CallToolResult, McpError> {
    let from =
        parse_phase(&params.from).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let to = parse_phase(&params.to).map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let can = can_transition(from, to);

    if !can {
        let response = serde_json::json!({
            "valid": false,
            "from": params.from,
            "to": params.to,
            "error": format!("Transition {} → {} is not allowed", from, to),
            "suggestion": from.next().map(|n| n.to_string()),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]));
    }

    let reason = params.reason.as_deref().unwrap_or("requested");
    let transition = execute_transition(from, to, reason, params.timestamp.unwrap_or(0.0))
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let response = serde_json::json!({
        "valid": true,
        "from": transition.from.to_string(),
        "to": transition.to.to_string(),
        "reason": transition.reason,
        "timestamp": transition.timestamp,
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ── Parsers ───────────────────────────────────────────────────────────────

fn parse_phase(s: &str) -> Result<Phase, nexcore_error::NexError> {
    match s.to_lowercase().as_str() {
        "collect" => Ok(Phase::Collect),
        "assess" => Ok(Phase::Assess),
        "plan" => Ok(Phase::Plan),
        "implement" => Ok(Phase::Implement),
        "followup" | "follow_up" | "follow-up" => Ok(Phase::FollowUp),
        _ => Err(nexcore_error::nexerror!(
            "Unknown phase: {s}. Valid: collect, assess, plan, implement, followup"
        )),
    }
}

fn parse_strategy(s: &str) -> Result<DosingStrategy, nexcore_error::NexError> {
    match s.to_lowercase().as_str() {
        "subtherapeutic" => Ok(DosingStrategy::Subtherapeutic),
        "therapeutic" => Ok(DosingStrategy::Therapeutic),
        "loading" => Ok(DosingStrategy::Loading),
        "maintenance" => Ok(DosingStrategy::Maintenance),
        _ => Err(nexcore_error::nexerror!(
            "Unknown strategy: {s}. Valid: subtherapeutic, therapeutic, loading, maintenance"
        )),
    }
}

fn parse_interaction_type(s: &str) -> Result<InteractionType, nexcore_error::NexError> {
    match s.to_lowercase().as_str() {
        "synergistic" => Ok(InteractionType::Synergistic),
        "antagonistic" => Ok(InteractionType::Antagonistic),
        "additive" => Ok(InteractionType::Additive),
        "potentiating" => Ok(InteractionType::Potentiating),
        _ => Err(nexcore_error::nexerror!(
            "Unknown interaction type: {s}. Valid: synergistic, antagonistic, additive, potentiating"
        )),
    }
}
