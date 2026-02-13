//! Trust tools — Bayesian trust engine with patient safety integration.
//!
//! Consolidated from `nexcore-trust` crate.
//! 8 tools: trust_score, trust_record, trust_snapshot, trust_decide,
//!          trust_harm_weight, trust_velocity, trust_multi_score, trust_network_chain.
//!
//! Uses OnceLock-based lazy state for trust engine instances.
//!
//! Tier: T3 (ς State + ∂ Boundary + N Quantity + → Causality + κ Comparison + ν Frequency)

use std::collections::HashMap;
use std::sync::OnceLock;

use nexcore_trust::{
    CausalityAssessment, DimensionWeights, Evidence, HarmSeverity, MultiTrustEngine, PolicyConfig,
    TrustConfig, TrustDimension, TrustEngine, TrustVelocity, chain_trust, decide, decide_simple,
    harm_adjusted_weight, naranjo_to_causality, safety_config, who_umc_to_causality,
};
use parking_lot::RwLock;
use serde_json::json;

use crate::params::{
    TrustDecideParams, TrustHarmWeightParams, TrustMultiScoreParams, TrustNetworkChainParams,
    TrustRecordParams, TrustScoreParams, TrustSnapshotParams, TrustVelocityParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};

// ============================================================================
// Lazy state
// ============================================================================

struct TrustState {
    engines: HashMap<String, TrustEngine>,
    multi_engines: HashMap<String, MultiTrustEngine>,
    velocities: HashMap<String, TrustVelocity>,
}

static STATE: OnceLock<RwLock<TrustState>> = OnceLock::new();

fn state() -> &'static RwLock<TrustState> {
    STATE.get_or_init(|| {
        RwLock::new(TrustState {
            engines: HashMap::new(),
            multi_engines: HashMap::new(),
            velocities: HashMap::new(),
        })
    })
}

fn get_or_create_engine(entity_id: &str, safety_mode: bool) -> TrustEngine {
    let guard = state().read();
    if let Some(engine) = guard.engines.get(entity_id) {
        return engine.clone();
    }
    drop(guard);
    if safety_mode {
        TrustEngine::with_config(safety_config())
    } else {
        TrustEngine::new()
    }
}

fn save_engine(entity_id: &str, engine: TrustEngine) {
    let mut guard = state().write();
    guard.engines.insert(entity_id.to_string(), engine);
}

// ============================================================================
// Tool implementations
// ============================================================================

/// Get trust score for an entity.
pub fn trust_score(params: TrustScoreParams) -> Result<CallToolResult, McpError> {
    let engine = get_or_create_engine(&params.entity_id, params.safety_mode.unwrap_or(false));
    let snap = engine.snapshot();

    let result = json!({
        "entity_id": params.entity_id,
        "score": round4(snap.score),
        "level": format!("{}", snap.level),
        "uncertainty": round4(snap.uncertainty),
        "significant": snap.significant,
        "alpha": round4(snap.alpha),
        "beta": round4(snap.beta),
        "interactions": snap.interactions,
        "time_since_evidence": round2(snap.time_since_evidence),
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Record evidence for an entity and return updated score.
pub fn trust_record(params: TrustRecordParams) -> Result<CallToolResult, McpError> {
    let safety = params.safety_mode.unwrap_or(false);
    let mut engine = get_or_create_engine(&params.entity_id, safety);
    let before = engine.score();

    let evidence = match params.evidence_type.to_lowercase().as_str() {
        "positive" => Evidence::positive_weighted(params.weight.unwrap_or(1.0)),
        "negative" => Evidence::negative_weighted(params.weight.unwrap_or(1.0)),
        "neutral" => Evidence::Neutral,
        _ => {
            return Err(McpError::new(
                ErrorCode(400),
                "evidence_type must be 'positive', 'negative', or 'neutral'",
                None,
            ));
        }
    };

    engine.record(evidence);

    // Optionally advance time
    if let Some(dt) = params.time_delta {
        engine.advance_time(dt);
    }

    let after = engine.score();
    let snap = engine.snapshot();

    // Update velocity tracker
    {
        let mut guard = state().write();
        let velocity = guard
            .velocities
            .entry(params.entity_id.clone())
            .or_insert_with(TrustVelocity::default);
        velocity.update(after - before);
    }

    save_engine(&params.entity_id, engine);

    let result = json!({
        "entity_id": params.entity_id,
        "evidence": format!("{evidence}"),
        "score_before": round4(before),
        "score_after": round4(after),
        "delta": round4(after - before),
        "level": format!("{}", snap.level),
        "significant": snap.significant,
        "interactions": snap.interactions,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get a full snapshot of an entity's trust state.
pub fn trust_snapshot(params: TrustSnapshotParams) -> Result<CallToolResult, McpError> {
    let engine = get_or_create_engine(&params.entity_id, false);
    let snap = engine.snapshot();

    let velocity_info = {
        let guard = state().read();
        guard.velocities.get(&params.entity_id).map(|v| {
            json!({
                "velocity": round4(v.velocity()),
                "direction": format!("{}", v.direction()),
                "peak_magnitude": round4(v.peak_magnitude()),
                "update_count": v.update_count(),
                "anomalous_at_005": v.is_anomalous(0.05),
                "anomalous_at_010": v.is_anomalous(0.10),
            })
        })
    };

    let result = json!({
        "entity_id": params.entity_id,
        "score": round4(snap.score),
        "level": format!("{}", snap.level),
        "uncertainty": round4(snap.uncertainty),
        "significant": snap.significant,
        "alpha": round4(snap.alpha),
        "beta": round4(snap.beta),
        "interactions": snap.interactions,
        "time_since_evidence": round2(snap.time_since_evidence),
        "velocity": velocity_info,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Make a trust-based policy decision for an entity.
pub fn trust_decide(params: TrustDecideParams) -> Result<CallToolResult, McpError> {
    let engine = get_or_create_engine(&params.entity_id, params.safety_mode.unwrap_or(false));

    let policy = match params.policy.as_deref() {
        Some("strict") => PolicyConfig::strict(),
        Some("permissive") => PolicyConfig::permissive(),
        _ => PolicyConfig::default(),
    };

    let velocity = {
        let guard = state().read();
        guard.velocities.get(&params.entity_id).cloned()
    };

    let rationale = decide(&engine, velocity.as_ref(), &policy);

    let result = json!({
        "entity_id": params.entity_id,
        "decision": format!("{}", rationale.decision),
        "permitted": rationale.decision.is_permitted(),
        "blocked": rationale.decision.is_blocked(),
        "primary_factor": format!("{}", rationale.primary_factor),
        "score": round4(rationale.score),
        "level": format!("{}", rationale.level),
        "significant": rationale.significant,
        "direction": rationale.direction.map(|d| format!("{d}")),
        "anomalous": rationale.anomalous,
        "confidence": round4(rationale.confidence),
        "policy": params.policy.as_deref().unwrap_or("default"),
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compute harm-adjusted evidence weight for patient safety.
pub fn trust_harm_weight(params: TrustHarmWeightParams) -> Result<CallToolResult, McpError> {
    let severity = parse_severity(&params.severity)?;
    let causality = parse_causality(&params.causality)?;
    let base = params.base_weight.unwrap_or(1.0);

    let weight = harm_adjusted_weight(base, severity, causality);

    // Optionally record into an entity's engine
    if let Some(ref entity_id) = params.entity_id {
        let mut engine = get_or_create_engine(entity_id, true);
        let before = engine.score();
        if weight > 0.0 {
            engine.record(Evidence::negative_weighted(weight));
        }
        let after = engine.score();
        save_engine(entity_id, engine);

        let result = json!({
            "base_weight": base,
            "severity": params.severity,
            "severity_multiplier": severity.trust_multiplier(),
            "causality": params.causality,
            "causality_weight": causality.evidence_weight(),
            "effective_weight": round4(weight),
            "entity_id": entity_id,
            "score_before": round4(before),
            "score_after": round4(after),
            "impact": round4(before - after),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]));
    }

    let result = json!({
        "base_weight": base,
        "severity": params.severity,
        "severity_multiplier": severity.trust_multiplier(),
        "causality": params.causality,
        "causality_weight": causality.evidence_weight(),
        "effective_weight": round4(weight),
        "formula": "base_weight * severity_multiplier * causality_weight",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get trust velocity (rate of change) for an entity.
pub fn trust_velocity(params: TrustVelocityParams) -> Result<CallToolResult, McpError> {
    let guard = state().read();
    let velocity = guard.velocities.get(&params.entity_id);

    match velocity {
        Some(v) => {
            let threshold = params.anomaly_threshold.unwrap_or(0.05);
            let result = json!({
                "entity_id": params.entity_id,
                "velocity": round4(v.velocity()),
                "direction": format!("{}", v.direction()),
                "peak_magnitude": round4(v.peak_magnitude()),
                "update_count": v.update_count(),
                "is_anomalous": v.is_anomalous(threshold),
                "anomaly_threshold": threshold,
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        None => {
            let result = json!({
                "entity_id": params.entity_id,
                "error": "No velocity data. Record evidence first via trust_record.",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

/// Compute multi-dimensional trust score (Ability/Benevolence/Integrity).
pub fn trust_multi_score(params: TrustMultiScoreParams) -> Result<CallToolResult, McpError> {
    let mut guard = state().write();
    let engine = guard
        .multi_engines
        .entry(params.entity_id.clone())
        .or_insert_with(MultiTrustEngine::new);

    // Record evidence if provided
    if let Some(ref ev_type) = params.evidence_type {
        let dimension = match params.dimension.as_deref() {
            Some("ability") => Some(TrustDimension::Ability),
            Some("benevolence") => Some(TrustDimension::Benevolence),
            Some("integrity") => Some(TrustDimension::Integrity),
            Some("all") | None => None,
            Some(other) => {
                return Err(McpError::new(
                    ErrorCode(400),
                    format!(
                        "dimension must be 'ability', 'benevolence', 'integrity', or 'all', got '{other}'"
                    ),
                    None,
                ));
            }
        };

        let evidence = match ev_type.to_lowercase().as_str() {
            "positive" => Evidence::positive_weighted(params.weight.unwrap_or(1.0)),
            "negative" => Evidence::negative_weighted(params.weight.unwrap_or(1.0)),
            "neutral" => Evidence::Neutral,
            _ => {
                return Err(McpError::new(
                    ErrorCode(400),
                    "evidence_type must be 'positive', 'negative', or 'neutral'",
                    None,
                ));
            }
        };

        match dimension {
            Some(dim) => engine.record(dim, evidence),
            None => engine.record_all(evidence),
        }
    }

    let snap = engine.snapshot();
    let (weakest_dim, weakest_score) = (snap.weakest.0, snap.weakest.1);

    let result = json!({
        "entity_id": params.entity_id,
        "ability": {
            "score": round4(snap.ability.score),
            "level": format!("{}", snap.ability.level),
            "interactions": snap.ability.interactions,
        },
        "benevolence": {
            "score": round4(snap.benevolence.score),
            "level": format!("{}", snap.benevolence.level),
            "interactions": snap.benevolence.interactions,
        },
        "integrity": {
            "score": round4(snap.integrity.score),
            "level": format!("{}", snap.integrity.level),
            "interactions": snap.integrity.interactions,
        },
        "composite": {
            "score": round4(snap.composite_score),
            "level": format!("{}", snap.composite_level),
        },
        "minimum": {
            "score": round4(snap.minimum_score),
            "level": format!("{}", snap.minimum_level),
        },
        "weakest_dimension": format!("{weakest_dim}"),
        "weakest_score": round4(weakest_score),
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compute transitive trust through a chain of intermediaries.
pub fn trust_network_chain(params: TrustNetworkChainParams) -> Result<CallToolResult, McpError> {
    if params.scores.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "scores array must not be empty",
            None,
        ));
    }

    let damping = params.damping.unwrap_or(0.8);
    let result_score = chain_trust(&params.scores, damping);

    let hops = params.scores.len().saturating_sub(1);
    let result = json!({
        "chain_length": params.scores.len(),
        "hops": hops,
        "damping": damping,
        "scores": params.scores,
        "transitive_trust": round4(result_score),
        "interpretation": if result_score >= 0.6 {
            "Transitive trust sufficient"
        } else if result_score >= 0.4 {
            "Transitive trust borderline — verify directly"
        } else {
            "Transitive trust insufficient — direct verification required"
        },
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Helpers
// ============================================================================

fn parse_severity(s: &str) -> Result<HarmSeverity, McpError> {
    match s.to_lowercase().as_str() {
        "non-serious" | "nonserious" | "non_serious" => Ok(HarmSeverity::NonSerious),
        "other-serious" | "other_serious" | "otherserious" => Ok(HarmSeverity::OtherSerious),
        "disability" => Ok(HarmSeverity::Disability),
        "congenital-anomaly" | "congenital_anomaly" | "congenitalanomaly" => {
            Ok(HarmSeverity::CongenitalAnomaly)
        }
        "hospitalization" => Ok(HarmSeverity::Hospitalization),
        "life-threatening" | "life_threatening" | "lifethreatening" => {
            Ok(HarmSeverity::LifeThreatening)
        }
        "death" => Ok(HarmSeverity::Death),
        _ => Err(McpError::new(
            ErrorCode(400),
            format!(
                "Unknown severity '{}'. Use: non-serious, other-serious, disability, \
                 congenital-anomaly, hospitalization, life-threatening, death",
                s
            ),
            None,
        )),
    }
}

fn parse_causality(s: &str) -> Result<CausalityAssessment, McpError> {
    // Try WHO-UMC string mapping first
    if let Some(assessment) = who_umc_to_causality(s) {
        return Ok(assessment);
    }
    // Try Naranjo numeric score
    if let Ok(score) = s.parse::<i32>() {
        return Ok(naranjo_to_causality(score));
    }
    Err(McpError::new(
        ErrorCode(400),
        format!(
            "Unknown causality '{}'. Use WHO-UMC terms (certain, probable, possible, \
             unlikely, unassessable) or Naranjo score (-4 to 13)",
            s
        ),
        None,
    ))
}

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_score_default_entity() {
        let params = TrustScoreParams {
            entity_id: "test-entity-1".to_string(),
            safety_mode: None,
        };
        let result = trust_score(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        assert!(text.contains("\"score\""));
        assert!(text.contains("\"level\""));
    }

    #[test]
    fn trust_score_safety_mode() {
        let params = TrustScoreParams {
            entity_id: "test-safety-entity".to_string(),
            safety_mode: Some(true),
        };
        let result = trust_score(params);
        assert!(result.is_ok());
    }

    #[test]
    fn trust_record_positive_evidence() {
        let params = TrustRecordParams {
            entity_id: "test-record-pos".to_string(),
            evidence_type: "positive".to_string(),
            weight: Some(1.5),
            safety_mode: None,
            time_delta: None,
        };
        let result = trust_record(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        assert!(text.contains("\"delta\""));
    }

    #[test]
    fn trust_record_negative_evidence() {
        let params = TrustRecordParams {
            entity_id: "test-record-neg".to_string(),
            evidence_type: "negative".to_string(),
            weight: Some(2.0),
            safety_mode: Some(true),
            time_delta: Some(3600.0),
        };
        let result = trust_record(params);
        assert!(result.is_ok());
    }

    #[test]
    fn trust_record_rejects_invalid_evidence() {
        let params = TrustRecordParams {
            entity_id: "test-bad".to_string(),
            evidence_type: "invalid".to_string(),
            weight: None,
            safety_mode: None,
            time_delta: None,
        };
        assert!(trust_record(params).is_err());
    }

    #[test]
    fn trust_snapshot_returns_state() {
        let params = TrustSnapshotParams {
            entity_id: "test-snapshot".to_string(),
        };
        let result = trust_snapshot(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        assert!(text.contains("\"alpha\""));
        assert!(text.contains("\"beta\""));
    }

    #[test]
    fn trust_decide_default_policy() {
        let params = TrustDecideParams {
            entity_id: "test-decide".to_string(),
            policy: None,
            safety_mode: None,
        };
        let result = trust_decide(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        assert!(text.contains("\"decision\""));
        assert!(text.contains("\"permitted\""));
    }

    #[test]
    fn trust_decide_strict_policy() {
        let params = TrustDecideParams {
            entity_id: "test-decide-strict".to_string(),
            policy: Some("strict".to_string()),
            safety_mode: Some(true),
        };
        let result = trust_decide(params);
        assert!(result.is_ok());
    }

    #[test]
    fn trust_harm_weight_death_certain() {
        let params = TrustHarmWeightParams {
            severity: "death".to_string(),
            causality: "certain".to_string(),
            base_weight: None,
            entity_id: None,
        };
        let result = trust_harm_weight(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        // Death × Certain should produce a large effective weight
        assert!(text.contains("\"effective_weight\""));
        let val: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
        let w = val
            .get("effective_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        assert!(w >= 4.0, "Death × Certain weight should be >= 4.0, got {w}");
    }

    #[test]
    fn trust_harm_weight_with_entity() {
        let params = TrustHarmWeightParams {
            severity: "hospitalization".to_string(),
            causality: "probable".to_string(),
            base_weight: Some(1.0),
            entity_id: Some("test-harm-entity".to_string()),
        };
        let result = trust_harm_weight(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        assert!(text.contains("\"score_before\""));
        assert!(text.contains("\"score_after\""));
    }

    #[test]
    fn trust_harm_weight_rejects_bad_severity() {
        let params = TrustHarmWeightParams {
            severity: "oopsie".to_string(),
            causality: "certain".to_string(),
            base_weight: None,
            entity_id: None,
        };
        assert!(trust_harm_weight(params).is_err());
    }

    #[test]
    fn trust_harm_weight_rejects_bad_causality() {
        let params = TrustHarmWeightParams {
            severity: "death".to_string(),
            causality: "totally-not-a-thing".to_string(),
            base_weight: None,
            entity_id: None,
        };
        assert!(trust_harm_weight(params).is_err());
    }

    #[test]
    fn trust_harm_weight_naranjo_score() {
        let params = TrustHarmWeightParams {
            severity: "life-threatening".to_string(),
            causality: "7".to_string(), // Naranjo probable
            base_weight: Some(1.0),
            entity_id: None,
        };
        let result = trust_harm_weight(params);
        assert!(result.is_ok());
    }

    #[test]
    fn trust_velocity_no_data() {
        let params = TrustVelocityParams {
            entity_id: "nonexistent-entity-velocity".to_string(),
            anomaly_threshold: None,
        };
        let result = trust_velocity(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        // Should return error message, not an actual error
        assert!(text.contains("No velocity data"));
    }

    #[test]
    fn trust_multi_score_fresh_entity() {
        let params = TrustMultiScoreParams {
            entity_id: "test-multi-fresh".to_string(),
            evidence_type: None,
            dimension: None,
            weight: None,
        };
        let result = trust_multi_score(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        assert!(text.contains("\"ability\""));
        assert!(text.contains("\"benevolence\""));
        assert!(text.contains("\"integrity\""));
        assert!(text.contains("\"composite\""));
    }

    #[test]
    fn trust_multi_score_with_evidence() {
        let params = TrustMultiScoreParams {
            entity_id: "test-multi-ev".to_string(),
            evidence_type: Some("positive".to_string()),
            dimension: Some("ability".to_string()),
            weight: Some(2.0),
        };
        let result = trust_multi_score(params);
        assert!(result.is_ok());
    }

    #[test]
    fn trust_multi_score_rejects_bad_dimension() {
        let params = TrustMultiScoreParams {
            entity_id: "test-multi-bad".to_string(),
            evidence_type: Some("positive".to_string()),
            dimension: Some("charisma".to_string()),
            weight: None,
        };
        assert!(trust_multi_score(params).is_err());
    }

    #[test]
    fn trust_network_chain_basic() {
        let params = TrustNetworkChainParams {
            scores: vec![0.9, 0.8, 0.7],
            damping: None,
        };
        let result = trust_network_chain(params);
        assert!(result.is_ok());
        let text = result
            .as_ref()
            .ok()
            .and_then(|r| r.content.first())
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("");
        assert!(text.contains("\"transitive_trust\""));
        let val: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
        let t = val
            .get("transitive_trust")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        assert!(
            t > 0.0 && t < 1.0,
            "Transitive trust should be in (0,1), got {t}"
        );
    }

    #[test]
    fn trust_network_chain_rejects_empty() {
        let params = TrustNetworkChainParams {
            scores: vec![],
            damping: None,
        };
        assert!(trust_network_chain(params).is_err());
    }

    #[test]
    fn trust_network_chain_custom_damping() {
        let params = TrustNetworkChainParams {
            scores: vec![0.95, 0.90],
            damping: Some(0.5),
        };
        let result = trust_network_chain(params);
        assert!(result.is_ok());
    }
}
