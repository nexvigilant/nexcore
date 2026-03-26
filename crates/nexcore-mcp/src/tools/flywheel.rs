//! Flywheel Loop Engine MCP tool implementations.
//!
//! Four tools exposing the nexcore-flywheel five-loop cascade engine:
//!
//! | Tool | Purpose |
//! |------|---------|
//! | `flywheel_vitals` | Return a FlywheelVitals snapshot with optional field overrides |
//! | `flywheel_cascade` | Run the full five-loop interaction cascade and return CascadeResult |
//! | `flywheel_reality` | Cascade + VDAG Reality Gradient with evidence grading |
//! | `flywheel_learn` | Analyze cascade history for learning loop insights |
//!
//! ## T1 Primitive Grounding: ς (State) + κ (Comparison) + ∂ (Boundary) + → (Causality) + N (Quantity)

use crate::params::flywheel::{
    FlywheelCascadeParams, FlywheelLearnParams, FlywheelRealityParams, FlywheelVitalsParams,
};
use nexcore_flywheel::{
    loops::{cascade, CascadeInput, ElasticInput, FrictionInput, GyroscopicInput, MomentumInput, RimInput},
    thresholds::FlywheelThresholds,
    loops::SystemState,
    vdag::{self, CascadeRecord, FlywheelGoal},
    vitals::FlywheelVitals,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// Tool 1: flywheel_vitals
// ============================================================================

/// Return the current FlywheelVitals snapshot.
///
/// Starts from `FlywheelVitals::default()` and applies any provided field overrides.
/// Useful for establishing a baseline or inspecting the default configuration.
pub fn flywheel_vitals(params: FlywheelVitalsParams) -> Result<CallToolResult, McpError> {
    let mut vitals = FlywheelVitals::default();

    // Apply optional overrides
    if let Some(v) = params.value_density {
        vitals.value_density = v;
    }
    if let Some(v) = params.churn_rate {
        vitals.churn_rate = v;
    }
    if let Some(v) = params.switching_cost_index {
        vitals.switching_cost_index = v;
    }
    if let Some(v) = params.knowledge_base_growth {
        vitals.knowledge_base_growth = v;
    }
    if let Some(v) = params.execution_velocity {
        vitals.execution_velocity = v;
    }
    if let Some(v) = params.momentum {
        vitals.momentum = v;
    }
    if let Some(v) = params.automation_coverage {
        vitals.automation_coverage = v;
    }
    if let Some(v) = params.manual_touchpoints {
        vitals.manual_touchpoints = v;
    }
    if let Some(v) = params.overhead_ratio {
        vitals.overhead_ratio = v;
    }
    if let Some(v) = params.mission_alignment_score {
        vitals.mission_alignment_score = v;
    }
    if let Some(v) = params.scope_creep_incidents {
        vitals.scope_creep_incidents = v;
    }
    if let Some(v) = params.pivot_resistance {
        vitals.pivot_resistance = v;
    }
    if let Some(v) = params.contributor_load {
        vitals.contributor_load = v;
    }
    if let Some(v) = params.fatigue_cycle_count {
        vitals.fatigue_cycle_count = v;
    }
    if let Some(v) = params.recovery_time_days {
        vitals.recovery_time_days = v;
    }

    let result = json!({
        "vitals": {
            "loop_1_rim_integrity": {
                "value_density": vitals.value_density,
                "churn_rate": vitals.churn_rate,
                "switching_cost_index": vitals.switching_cost_index,
            },
            "loop_2_momentum": {
                "knowledge_base_growth": vitals.knowledge_base_growth,
                "execution_velocity": vitals.execution_velocity,
                "momentum": vitals.momentum,
            },
            "loop_3_friction": {
                "automation_coverage": vitals.automation_coverage,
                "manual_touchpoints": vitals.manual_touchpoints,
                "overhead_ratio": vitals.overhead_ratio,
            },
            "loop_4_gyroscopic": {
                "mission_alignment_score": vitals.mission_alignment_score,
                "scope_creep_incidents": vitals.scope_creep_incidents,
                "pivot_resistance": vitals.pivot_resistance,
            },
            "loop_5_elastic": {
                "contributor_load": vitals.contributor_load,
                "fatigue_cycle_count": vitals.fatigue_cycle_count,
                "recovery_time_days": vitals.recovery_time_days,
            }
        },
        "field_count": 15,
        "source": "nexcore-flywheel FlywheelVitals::default() with overrides"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tool 2: flywheel_cascade
// ============================================================================

/// Run the full five-loop interaction cascade and return CascadeResult.
///
/// Cascade order: Loop 5 (Elastic) → Loop 1 (Rim) → Loop 3 (Friction) → Loop 2 (Momentum) → Loop 4 (Gyroscopic).
/// Friction net_drain is automatically added to momentum friction_drain.
/// Momentum L is automatically fed into gyroscopic input.
pub fn flywheel_cascade(params: FlywheelCascadeParams) -> Result<CallToolResult, McpError> {
    let input = CascadeInput {
        rim: RimInput {
            tensile_strength: params.rim_tensile_strength,
            centrifugal_force: params.rim_centrifugal_force,
        },
        momentum: MomentumInput {
            inertia: params.momentum_inertia,
            omega: params.momentum_omega,
            friction_drain: params.momentum_friction_drain,
        },
        friction: FrictionInput {
            manual_processes: params.friction_manual_processes,
            human_touchpoints: params.friction_human_touchpoints,
            velocity: params.friction_velocity,
            automation_coverage: params.friction_automation_coverage,
        },
        gyroscopic: GyroscopicInput {
            momentum_l: params.gyroscopic_momentum_l,
            perturbation_torque: params.gyroscopic_perturbation_torque,
            critical_momentum: params.gyroscopic_critical_momentum,
        },
        elastic: ElasticInput {
            stress: params.elastic_stress,
            yield_point: params.elastic_yield_point,
            fatigue_cycles: params.elastic_fatigue_cycles,
            fatigue_limit: params.elastic_fatigue_limit,
        },
    };

    let thresholds = FlywheelThresholds::default();
    let result_data = cascade(&input, &thresholds);

    let result = json!({
        "system_state": result_data.system_state,
        "loops": {
            "rim": {
                "state": result_data.rim.state,
                "margin": result_data.rim.margin,
                "ratio": result_data.rim.ratio,
            },
            "momentum": {
                "l": result_data.momentum.l,
                "classification": result_data.momentum.classification.to_string(),
                "above_critical": result_data.momentum.above_critical,
            },
            "friction": {
                "contact_friction": result_data.friction.contact_friction,
                "aero_drag": result_data.friction.aero_drag,
                "total_drain": result_data.friction.total_drain,
                "net_drain": result_data.friction.net_drain,
                "classification": result_data.friction.classification.to_string(),
            },
            "gyroscopic": {
                "score": result_data.gyroscopic.score,
                "state": result_data.gyroscopic.state,
                "stability_ratio": result_data.gyroscopic.stability_ratio,
            },
            "elastic": {
                "state": result_data.elastic.state,
                "strain": result_data.elastic.strain,
                "cycles_remaining": result_data.elastic.cycles_remaining,
                "permanent_deformation": result_data.elastic.permanent_deformation,
            }
        },
        "cascade_order": "5(Elastic)→1(Rim)→3(Friction)→2(Momentum)→4(Gyroscopic)",
        "thresholds_used": "FlywheelThresholds::default()",
        "source": "nexcore-flywheel loops::cascade()"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tool 3: flywheel_reality
// ============================================================================

/// Run the cascade with VDAG evidence grading and Reality Gradient scoring.
///
/// Returns cascade result + reality gradient (0.0-1.0) + per-loop evidence quality.
/// Reality < 0.20 = testing theater (not executable).
pub fn flywheel_reality(params: FlywheelRealityParams) -> Result<CallToolResult, McpError> {
    let input = CascadeInput {
        rim: RimInput {
            tensile_strength: params.rim_tensile_strength,
            centrifugal_force: params.rim_centrifugal_force,
        },
        momentum: MomentumInput {
            inertia: params.momentum_inertia,
            omega: params.momentum_omega,
            friction_drain: params.momentum_friction_drain,
        },
        friction: FrictionInput {
            manual_processes: params.friction_manual_processes,
            human_touchpoints: params.friction_human_touchpoints,
            velocity: params.friction_velocity,
            automation_coverage: params.friction_automation_coverage,
        },
        gyroscopic: GyroscopicInput {
            momentum_l: params.gyroscopic_momentum_l,
            perturbation_torque: params.gyroscopic_perturbation_torque,
            critical_momentum: params.gyroscopic_critical_momentum,
        },
        elastic: ElasticInput {
            stress: params.elastic_stress,
            yield_point: params.elastic_yield_point,
            fatigue_cycles: params.elastic_fatigue_cycles,
            fatigue_limit: params.elastic_fatigue_limit,
        },
    };

    let target = params
        .target_state
        .as_deref()
        .map(|s| match s {
            "stressed" => SystemState::Stressed,
            "critical" => SystemState::Critical,
            "failed" => SystemState::Failed,
            _ => SystemState::Thriving,
        })
        .unwrap_or(SystemState::Thriving);

    let goal = FlywheelGoal {
        description: params
            .goal_description
            .unwrap_or_else(|| "All loops healthy".to_string()),
        target_state: target,
        loop_weights: [
            params.rim_weight.unwrap_or(0.2),
            params.momentum_weight.unwrap_or(0.2),
            params.friction_weight.unwrap_or(0.2),
            params.gyroscopic_weight.unwrap_or(0.2),
            params.elastic_weight.unwrap_or(0.2),
        ],
    };

    let thresholds = FlywheelThresholds::default();
    let graded = vdag::evaluate(&input, &thresholds, &goal);

    let per_loop: Vec<serde_json::Value> = graded
        .reality
        .per_loop
        .iter()
        .map(|e| {
            json!({
                "loop_name": e.loop_name,
                "quality": e.quality.to_string(),
                "score": e.quality.score(),
                "weight": e.weight,
                "achieved_target": e.achieved_target,
            })
        })
        .collect();

    let result = json!({
        "system_state": graded.cascade.system_state,
        "loops": {
            "rim": {
                "state": graded.cascade.rim.state,
                "margin": graded.cascade.rim.margin,
                "ratio": graded.cascade.rim.ratio,
            },
            "momentum": {
                "l": graded.cascade.momentum.l,
                "classification": graded.cascade.momentum.classification.to_string(),
                "above_critical": graded.cascade.momentum.above_critical,
            },
            "friction": {
                "contact_friction": graded.cascade.friction.contact_friction,
                "aero_drag": graded.cascade.friction.aero_drag,
                "total_drain": graded.cascade.friction.total_drain,
                "net_drain": graded.cascade.friction.net_drain,
                "classification": graded.cascade.friction.classification.to_string(),
            },
            "gyroscopic": {
                "score": graded.cascade.gyroscopic.score,
                "state": graded.cascade.gyroscopic.state,
                "stability_ratio": graded.cascade.gyroscopic.stability_ratio,
            },
            "elastic": {
                "state": graded.cascade.elastic.state,
                "strain": graded.cascade.elastic.strain,
                "cycles_remaining": graded.cascade.elastic.cycles_remaining,
                "permanent_deformation": graded.cascade.elastic.permanent_deformation,
            }
        },
        "reality": {
            "score": graded.reality.score,
            "rating": graded.reality.rating.to_string(),
            "executable": graded.reality.executable,
            "per_loop": per_loop,
        },
        "goal": {
            "description": graded.goal.description,
            "target_state": graded.goal.target_state,
            "loop_weights": graded.goal.loop_weights,
        },
        "source": "nexcore-flywheel vdag::evaluate()"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tool 4: flywheel_learn
// ============================================================================

/// Analyze cascade history for learning loop insights and threshold recommendations.
///
/// - Single loop: fix execution errors (most recent failure).
/// - Double loop: question thresholds (failure rate > 20%).
/// - Triple loop: question the model itself (every 5th analysis).
pub fn flywheel_learn(params: FlywheelLearnParams) -> Result<CallToolResult, McpError> {
    let records: Vec<CascadeRecord> = params
        .history
        .iter()
        .filter_map(|v| serde_json::from_value::<CascadeRecord>(v.clone()).ok())
        .collect();

    let skipped = params.history.len() - records.len();

    // All items failed to parse — report error instead of silent empty insights
    if records.is_empty() && !params.history.is_empty() {
        let err = json!({
            "error": "history_deserialization_failed",
            "message": format!(
                "Attempted to parse {} history items, got 0 valid CascadeRecord objects",
                params.history.len()
            ),
            "records_parsed": 0,
            "records_skipped": params.history.len(),
        });
        return Ok(CallToolResult::error(vec![Content::text(err.to_string())]));
    }

    let thresholds = FlywheelThresholds::default();
    let insights = vdag::analyze_history(&records, &thresholds);

    let insights_json: Vec<serde_json::Value> = insights
        .iter()
        .map(|insight| {
            let adjustments: Vec<serde_json::Value> = insight
                .suggested_adjustments
                .iter()
                .map(|adj| {
                    json!({
                        "parameter": adj.parameter,
                        "current_value": adj.current_value,
                        "suggested_value": adj.suggested_value,
                        "confidence": adj.confidence,
                        "reason": adj.reason,
                    })
                })
                .collect();

            json!({
                "loop_type": insight.loop_type,
                "observation": insight.observation,
                "suggested_adjustments": adjustments,
            })
        })
        .collect();

    let result = json!({
        "records_parsed": records.len(),
        "records_skipped": skipped,
        "insights": insights_json,
        "source": "nexcore-flywheel vdag::analyze_history()"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
