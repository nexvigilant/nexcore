//! Hormone (Endocrine System) tools for .claude
//!
//! Exposes the endocrine system state to Claude Code for visibility
//! and manual stimulus application.
//!
//! ## Tools
//! - hormone_status: Get full endocrine state
//! - hormone_get: Get specific hormone level
//! - hormone_stimulus: Apply stimulus to state
//! - hormone_modifiers: Get current behavioral modifiers

use crate::params::{HormoneGetParams, HormoneStimulusParams};
use nexcore_hormones::{BehavioralModifiers, EndocrineState, HormoneType, Stimulus};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Get full endocrine state with all hormone levels
pub fn status() -> Result<CallToolResult, McpError> {
    let state = EndocrineState::load();
    let modifiers = BehavioralModifiers::from(&state);

    let hormones: Vec<_> = HormoneType::ALL
        .iter()
        .map(|h| {
            let level = state.get(*h);
            json!({
                "name": h.name(),
                "type": format!("{:?}", h),
                "level": level.value(),
                "percent": (level.value() * 100.0) as u32,
                "decay_rate": h.decay_rate(),
            })
        })
        .collect();

    let result = json!({
        "hormones": hormones,
        "mood_score": state.mood_score(),
        "risk_tolerance": state.risk_tolerance(),
        "session_count": state.session_count,
        "last_updated": state.last_updated.to_rfc3339(),
        "modifiers": {
            "risk_tolerance": modifiers.risk_tolerance,
            "validation_depth": modifiers.validation_depth,
            "exploration_rate": modifiers.exploration_rate,
            "verbosity": modifiers.verbosity,
            "crisis_mode": modifiers.crisis_mode,
            "partnership_mode": modifiers.partnership_mode,
            "rest_recommended": modifiers.rest_recommended,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Get a specific hormone level
pub fn get(params: HormoneGetParams) -> Result<CallToolResult, McpError> {
    let state = EndocrineState::load();

    let hormone = match params.hormone.to_lowercase().as_str() {
        "cortisol" | "stress" => HormoneType::Cortisol,
        "dopamine" | "reward" => HormoneType::Dopamine,
        "serotonin" | "stability" => HormoneType::Serotonin,
        "adrenaline" | "crisis" => HormoneType::Adrenaline,
        "oxytocin" | "trust" => HormoneType::Oxytocin,
        "melatonin" | "rest" => HormoneType::Melatonin,
        _ => {
            let result = json!({
                "error": "Unknown hormone",
                "valid_hormones": ["cortisol", "dopamine", "serotonin", "adrenaline", "oxytocin", "melatonin"],
            });
            return Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]));
        }
    };

    let level = state.get(hormone);

    let result = json!({
        "hormone": hormone.name(),
        "type": format!("{:?}", hormone),
        "level": level.value(),
        "percent": (level.value() * 100.0) as u32,
        "decay_rate": hormone.decay_rate(),
        "interpretation": interpret_level(hormone, level.value()),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Apply a stimulus to the endocrine state
pub fn stimulus(params: HormoneStimulusParams) -> Result<CallToolResult, McpError> {
    let mut state = EndocrineState::load();
    let before = capture_state(&state);

    let stimulus = match params.stimulus_type.to_lowercase().as_str() {
        // Cortisol triggers
        "error" => Stimulus::ErrorEncountered {
            severity: params.intensity.unwrap_or(0.5),
        },
        "deadline" => Stimulus::DeadlinePressure {
            urgency: params.intensity.unwrap_or(0.5),
        },
        "uncertainty" => Stimulus::UncertaintyDetected {
            confidence_gap: params.intensity.unwrap_or(0.3),
        },

        // Dopamine triggers
        "task_completed" | "completed" => Stimulus::TaskCompleted {
            complexity: params.intensity.unwrap_or(0.5),
        },
        "positive_feedback" | "feedback" => Stimulus::PositiveFeedback {
            intensity: params.intensity.unwrap_or(0.5),
        },
        "pattern_success" | "pattern" => Stimulus::PatternSuccess {
            reuse_count: params.count.unwrap_or(1),
        },

        // Serotonin triggers
        "consistent" | "consistency" => Stimulus::ConsistentSession {
            variance: 1.0 - params.intensity.unwrap_or(0.8),
        },
        "predictable" | "predicted" => Stimulus::PredictableOutcome {
            accuracy: params.intensity.unwrap_or(0.9),
        },

        // Adrenaline triggers
        "critical_error" | "critical" => Stimulus::CriticalError {
            recoverable: params.recoverable.unwrap_or(true),
        },
        "time_pressure" | "time" => Stimulus::TimeConstraint {
            remaining_pct: params.intensity.unwrap_or(0.1),
        },
        "high_stakes" | "stakes" => Stimulus::HighStakesDecision {
            impact: params.intensity.unwrap_or(0.7),
        },

        // Oxytocin triggers
        "partnership" | "trust" => Stimulus::PartnershipReinforced {
            signal: params.intensity.unwrap_or(0.5),
        },
        "mutual_success" | "shared_win" => Stimulus::MutualSuccess { shared_win: true },
        "transparent" | "communication" => Stimulus::TransparentCommunication {
            clarity: params.intensity.unwrap_or(0.8),
        },

        // Melatonin triggers
        "duration" | "session_long" => Stimulus::SessionDuration {
            minutes: params.count.unwrap_or(90) as u64,
        },
        "context_full" | "context" => Stimulus::ContextUtilization {
            pct: params.intensity.unwrap_or(0.8),
        },
        "completion" | "done" => Stimulus::CompletionSignal {
            tasks_done: params.count.unwrap_or(1),
        },

        "planetary" | "alignment" => Stimulus::PlanetaryAlignment {
            distance_au: params.intensity.unwrap_or(1.5), // Use intensity for AU if not count
            days_since_opposition: params.count.unwrap_or(390),
        },

        _ => {
            let result = json!({
                "error": "Unknown stimulus type",
                "valid_types": {
                    "cortisol_triggers": ["error", "deadline", "uncertainty"],
                    "dopamine_triggers": ["task_completed", "positive_feedback", "pattern_success"],
                    "serotonin_triggers": ["consistent", "predictable"],
                    "adrenaline_triggers": ["critical_error", "time_pressure", "high_stakes"],
                    "oxytocin_triggers": ["partnership", "mutual_success", "transparent"],
                    "melatonin_triggers": ["duration", "context_full", "completion"],
                    "planetary_triggers": ["planetary", "alignment"],
                },
            });
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]));
        }
    };

    // Apply stimulus
    stimulus.apply(&mut state);

    // Save updated state
    if let Err(e) = state.save() {
        let result = json!({
            "error": format!("Failed to save state: {}", e),
            "stimulus_applied": false,
        });
        return Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]));
    }

    let after = capture_state(&state);

    let result = json!({
        "stimulus_applied": true,
        "stimulus_type": params.stimulus_type,
        "before": before,
        "after": after,
        "mood_change": after["mood_score"].as_f64().unwrap_or(0.0)
            - before["mood_score"].as_f64().unwrap_or(0.0),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Get current behavioral modifiers
pub fn modifiers() -> Result<CallToolResult, McpError> {
    let state = EndocrineState::load();
    let modifiers = BehavioralModifiers::from(&state);

    let result = json!({
        "modifiers": {
            "risk_tolerance": {
                "value": modifiers.risk_tolerance,
                "interpretation": if modifiers.risk_tolerance < 0.3 { "Very cautious" }
                    else if modifiers.risk_tolerance < 0.5 { "Cautious" }
                    else if modifiers.risk_tolerance < 0.7 { "Balanced" }
                    else { "Bold" },
            },
            "validation_depth": {
                "value": modifiers.validation_depth,
                "interpretation": if modifiers.validation_depth < 0.3 { "Minimal" }
                    else if modifiers.validation_depth < 0.5 { "Standard" }
                    else if modifiers.validation_depth < 0.7 { "Thorough" }
                    else { "Exhaustive" },
            },
            "exploration_rate": {
                "value": modifiers.exploration_rate,
                "interpretation": if modifiers.exploration_rate < 0.3 { "Stick to known" }
                    else if modifiers.exploration_rate < 0.5 { "Conservative" }
                    else if modifiers.exploration_rate < 0.7 { "Balanced" }
                    else { "Experimental" },
            },
            "verbosity": {
                "value": modifiers.verbosity,
                "interpretation": if modifiers.verbosity < 0.3 { "Terse" }
                    else if modifiers.verbosity < 0.5 { "Concise" }
                    else if modifiers.verbosity < 0.7 { "Moderate" }
                    else { "Verbose" },
            },
        },
        "active_modes": {
            "crisis_mode": modifiers.crisis_mode,
            "partnership_mode": modifiers.partnership_mode,
            "rest_recommended": modifiers.rest_recommended,
        },
        "recommendations": generate_recommendations(&modifiers),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// Helper functions

fn interpret_level(hormone: HormoneType, level: f64) -> &'static str {
    match hormone {
        HormoneType::Cortisol => {
            if level < 0.3 {
                "Low stress - relaxed state"
            } else if level < 0.7 {
                "Normal stress - alert and focused"
            } else {
                "High stress - increased caution"
            }
        }
        HormoneType::Dopamine => {
            if level < 0.3 {
                "Low reward drive - conservative"
            } else if level < 0.7 {
                "Normal motivation - balanced exploration"
            } else {
                "High reward drive - experimental"
            }
        }
        HormoneType::Serotonin => {
            if level < 0.3 {
                "Low stability - volatile behavior"
            } else if level < 0.7 {
                "Normal stability - consistent"
            } else {
                "High stability - very predictable"
            }
        }
        HormoneType::Adrenaline => {
            if level < 0.3 {
                "Calm - normal operation"
            } else if level < 0.7 {
                "Alert - heightened awareness"
            } else {
                "Crisis mode - aggressive problem-solving"
            }
        }
        HormoneType::Oxytocin => {
            if level < 0.3 {
                "Low trust - guarded interaction"
            } else if level < 0.6 {
                "Building trust - collaborative"
            } else {
                "High trust - partnership mode"
            }
        }
        HormoneType::Melatonin => {
            if level < 0.3 {
                "Alert - ready for work"
            } else if level < 0.7 {
                "Normal - sustainable pace"
            } else {
                "Fatigue signal - consider wrap-up"
            }
        }
    }
}

fn capture_state(state: &EndocrineState) -> serde_json::Value {
    json!({
        "cortisol": state.cortisol.value(),
        "dopamine": state.dopamine.value(),
        "serotonin": state.serotonin.value(),
        "adrenaline": state.adrenaline.value(),
        "oxytocin": state.oxytocin.value(),
        "melatonin": state.melatonin.value(),
        "mood_score": state.mood_score(),
        "risk_tolerance": state.risk_tolerance(),
    })
}

fn generate_recommendations(modifiers: &BehavioralModifiers) -> Vec<&'static str> {
    let mut recommendations = Vec::new();

    if modifiers.crisis_mode {
        recommendations.push("⚡ Crisis mode active - focus on immediate problem resolution");
    }

    if modifiers.rest_recommended {
        recommendations.push("😴 Consider wrapping up the session - fatigue signals detected");
    }

    if modifiers.partnership_mode {
        recommendations.push("🤝 Partnership mode - high trust enables more autonomous action");
    }

    if modifiers.risk_tolerance < 0.3 {
        recommendations.push("🛡️ Low risk tolerance - prefer conservative approaches");
    }

    if modifiers.exploration_rate > 0.7 {
        recommendations.push("🔬 High exploration drive - good time to try new approaches");
    }

    if modifiers.validation_depth > 0.7 {
        recommendations.push("✅ High validation mode - thorough checking recommended");
    }

    if recommendations.is_empty() {
        recommendations.push("✨ Balanced state - normal operation");
    }

    recommendations
}
