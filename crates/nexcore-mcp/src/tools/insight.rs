//! Insight engine MCP tools — pattern detection, novelty, connection, compression.
//!
//! # T1 Grounding
//! INSIGHT ≡ ⟨σ, κ, μ, ∃, ς, ∅, N, ∂⟩
//! - σ (sequence): Observation ordering, pipeline stages
//! - κ (comparison): Pattern matching, threshold crossing
//! - μ (mapping): Connection relationships, compression
//! - ∃ (existence): Novelty detection
//! - ∅ (void): Absence in prior state
//! - N (quantity): Counts, ratios, compression ratio
//! - ∂ (boundary): Suddenness thresholds
//! - ς (state): Engine state accumulation (ς-acc) — **persisted to disk**

use nexcore_insight::Observation;
use nexcore_insight::persist;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use nexcore_insight::engine::InsightEvent;

use crate::params::{
    InsightCompressAutoParams, InsightCompressParams, InsightConfigParams, InsightConnectParams,
    InsightIngestParams, InsightNoveltiesParams, InsightPatternsParams, InsightQueryParams,
    InsightResetParams, InsightStatusParams, InsightSystemIngestParams,
    InsightSystemRegisterParams, InsightSystemResetParams, InsightSystemStatusParams,
};

/// Helper: build an observation from MCP input params.
fn build_observation(o: &crate::params::InsightObservationInput) -> Observation {
    let mut obs = if let Some(numeric) = o.numeric_value {
        Observation::with_numeric(&o.key, numeric)
    } else {
        Observation::new(&o.key, o.value.as_deref().unwrap_or(""))
    };
    for tag in o.tags.as_deref().unwrap_or(&[]) {
        obs = obs.with_tag(tag);
    }
    obs
}

/// Helper: serialize the current config as a JSON value for responses.
fn config_json(config: &nexcore_insight::InsightConfig) -> serde_json::Value {
    serde_json::json!({
        "pattern_min_occurrences": config.pattern_min_occurrences,
        "pattern_confidence_threshold": config.pattern_confidence_threshold,
        "connection_strength_threshold": config.connection_strength_threshold,
        "compression_min_ratio": config.compression_min_ratio,
        "enable_suddenness": config.enable_suddenness,
        "suddenness_threshold": config.suddenness_threshold,
        "enable_recursive_learning": config.enable_recursive_learning,
    })
}

/// Ingest observations into the **persistent** engine and return produced events.
///
/// Config is **sticky** — saved values persist across calls. Only fields you
/// explicitly provide are overridden; omitted fields retain their saved values.
///
/// Tier: T3 (orchestrates all 6 composites via the Insight trait)
pub fn insight_ingest(params: InsightIngestParams) -> Result<CallToolResult, McpError> {
    let mut engine = persist::load_or_create()
        .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;

    // Selective override — only patch fields the caller provided
    engine.config.apply_overrides(
        params.pattern_min_occurrences,
        params.pattern_confidence_threshold,
        params.connection_strength_threshold,
        params.compression_min_ratio,
        params.enable_suddenness,
        params.suddenness_threshold,
        params.enable_recursive_learning,
    );

    let observations: Vec<Observation> =
        params.observations.iter().map(build_observation).collect();

    let events = engine.ingest_batch(observations);

    // Persist accumulated state + config
    persist::save(&engine)
        .map_err(|e| McpError::internal_error(format!("Failed to save engine: {e}"), None))?;

    let (state_bytes, _) = persist::stats();

    let response = serde_json::json!({
        "events": events.iter().map(|e| format!("{e}")).collect::<Vec<_>>(),
        "event_count": events.len(),
        "observation_count": engine.observation_count(),
        "unique_keys": engine.unique_key_count(),
        "pattern_count": engine.pattern_count(),
        "patterns": engine.patterns().iter().map(|p| serde_json::json!({
            "label": p.label,
            "members": p.members,
            "occurrence_count": p.occurrence_count,
            "confidence": p.confidence,
        })).collect::<Vec<_>>(),
        "connection_count": engine.connections().len(),
        "config": config_json(&engine.config),
        "persistent": true,
        "state_bytes": state_bytes,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Get persistent engine status. No observations needed — reads from disk.
///
/// Tier: T2-C (ς + N — state query + quantity)
pub fn insight_status(params: InsightStatusParams) -> Result<CallToolResult, McpError> {
    let mut engine = persist::load_or_create()
        .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;

    // If additional observations provided, ingest them too
    for o in params.observations.unwrap_or_default() {
        let obs = build_observation(&o);
        let _events = engine.ingest(obs);
    }

    let (state_bytes, persisted) = persist::stats();

    let response = serde_json::json!({
        "observation_count": engine.observation_count(),
        "unique_keys": engine.unique_key_count(),
        "pattern_count": engine.pattern_count(),
        "connection_count": engine.connections().len(),
        "event_count": engine.events().len(),
        "config": config_json(&engine.config),
        "persistent": persisted,
        "state_bytes": state_bytes,
        "state_path": persist::engine_path().to_string_lossy(),
        "tier": "T3",
        "grounding": "INSIGHT ≡ ⟨σ, κ, μ, ∃, ς, ∅, N, ∂⟩",
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// View or update the persistent InsightEngine configuration.
///
/// Call with no params to view current config. Provide fields to update them —
/// only specified fields are changed, all others retain their saved values.
/// Changes persist across all subsequent calls.
///
/// Tier: T2-P (∂ + ς — boundary configuration + state persistence)
pub fn insight_config(params: InsightConfigParams) -> Result<CallToolResult, McpError> {
    let mut engine = persist::load_or_create()
        .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;

    let has_changes = params.pattern_min_occurrences.is_some()
        || params.pattern_confidence_threshold.is_some()
        || params.connection_strength_threshold.is_some()
        || params.compression_min_ratio.is_some()
        || params.enable_suddenness.is_some()
        || params.suddenness_threshold.is_some()
        || params.enable_recursive_learning.is_some();

    let before = config_json(&engine.config);

    engine.config.apply_overrides(
        params.pattern_min_occurrences,
        params.pattern_confidence_threshold,
        params.connection_strength_threshold,
        params.compression_min_ratio,
        params.enable_suddenness,
        params.suddenness_threshold,
        params.enable_recursive_learning,
    );

    if has_changes {
        persist::save(&engine)
            .map_err(|e| McpError::internal_error(format!("Failed to save engine: {e}"), None))?;
    }

    let after = config_json(&engine.config);

    let response = serde_json::json!({
        "config": after,
        "changed": has_changes,
        "before": if has_changes { Some(before) } else { None },
        "persistent": true,
        "state_path": persist::engine_path().to_string_lossy(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Establish a connection in the persistent engine.
///
/// Tier: T2-C (μ + κ + ς — mapping + comparison + state)
pub fn insight_connect(params: InsightConnectParams) -> Result<CallToolResult, McpError> {
    let mut engine = persist::load_or_create()
        .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;

    let conn = engine.connect(&params.from, &params.to, &params.relation, params.strength);

    persist::save(&engine)
        .map_err(|e| McpError::internal_error(format!("Failed to save engine: {e}"), None))?;

    let response = serde_json::json!({
        "connection": {
            "from": conn.from,
            "to": conn.to,
            "relation": conn.relation,
            "strength": conn.strength,
            "is_strong": conn.strength > 0.5,
        },
        "total_connections": engine.connections().len(),
        "persistent": true,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Compress observation keys into a unifying principle (persistent engine).
///
/// Tier: T2-C (N + μ + κ — quantity reduction via mapping and comparison)
pub fn insight_compress(params: InsightCompressParams) -> Result<CallToolResult, McpError> {
    let mut engine = persist::load_or_create()
        .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;

    let compression = engine.compress_by_keys(params.keys, &params.principle);

    let response = serde_json::json!({
        "compression": {
            "principle": compression.principle,
            "input_count": compression.input_count,
            "output_count": compression.output_count,
            "ratio": compression.ratio,
            "is_meaningful": compression.is_meaningful(),
            "quality": compression.quality(),
            "entropy_bits": compression.entropy_bits(),
        },
        "persistent": true,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Auto-compress observations using tag-based and prefix-based clustering.
///
/// Ingests observations into persistent engine, then runs auto-compression.
/// Config is sticky — only provided fields are overridden.
///
/// Tier: T2-C (N + μ + κ — automatic quantity reduction via clustering)
pub fn insight_compress_auto(
    params: InsightCompressAutoParams,
) -> Result<CallToolResult, McpError> {
    let mut engine = persist::load_or_create()
        .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;

    // Selective override
    engine.config.apply_overrides(
        params.pattern_min_occurrences,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    for o in &params.observations {
        let obs = build_observation(o);
        let _events = engine.ingest(obs);
    }

    let compressions = engine.compress_auto();

    persist::save(&engine)
        .map_err(|e| McpError::internal_error(format!("Failed to save engine: {e}"), None))?;

    let response = serde_json::json!({
        "compression_count": compressions.len(),
        "compressions": compressions.iter().map(|c| serde_json::json!({
            "principle": c.principle,
            "input_count": c.input_count,
            "output_count": c.output_count,
            "ratio": c.ratio,
            "quality": c.quality(),
            "entropy_bits": c.entropy_bits(),
            "is_meaningful": c.is_meaningful(),
            "compressed_keys": c.compressed_keys,
        })).collect::<Vec<_>>(),
        "observation_count": engine.observation_count(),
        "unique_keys": engine.unique_key_count(),
        "persistent": true,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Get all detected patterns from persistent engine (with optional new observations).
///
/// Config is sticky — only provided fields are overridden.
///
/// Tier: T2-C (σ + κ + μ — sequence co-occurrence via comparison and mapping)
pub fn insight_patterns(params: InsightPatternsParams) -> Result<CallToolResult, McpError> {
    let mut engine = persist::load_or_create()
        .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;

    // Selective override
    engine.config.apply_overrides(
        params.pattern_min_occurrences,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    for o in &params.observations {
        let obs = build_observation(o);
        let _events = engine.ingest(obs);
    }

    persist::save(&engine)
        .map_err(|e| McpError::internal_error(format!("Failed to save engine: {e}"), None))?;

    let patterns: Vec<_> = engine
        .patterns()
        .iter()
        .map(|p| {
            serde_json::json!({
                "label": p.label,
                "members": p.members,
                "occurrence_count": p.occurrence_count,
                "confidence": p.confidence,
            })
        })
        .collect();

    let response = serde_json::json!({
        "pattern_count": patterns.len(),
        "patterns": patterns,
        "observation_count": engine.observation_count(),
        "persistent": true,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Reset the persistent engine state (delete accumulated data).
///
/// Tier: T1 (∅ — Void / clearing state)
pub fn insight_reset(_params: InsightResetParams) -> Result<CallToolResult, McpError> {
    persist::reset()
        .map_err(|e| McpError::internal_error(format!("Failed to reset engine: {e}"), None))?;

    let response = serde_json::json!({
        "reset": true,
        "message": "InsightEngine state cleared. Next ingest will start fresh.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// System-Level NexCoreInsight Tools (4 tools)
// ============================================================================

/// Get status of the system-level NexCoreInsight compositor.
///
/// Shows registered domains, per-domain observation counts, aggregate
/// pattern/connection/event counts, and config. This is the "NexCore IS
/// an InsightEngine" view — all domains unified in one engine.
///
/// Tier: T2-C (ς + N + Σ — state query + quantity + aggregation)
pub fn insight_system_status(
    _params: InsightSystemStatusParams,
) -> Result<CallToolResult, McpError> {
    let mut system = persist::load_system()
        .map_err(|e| McpError::internal_error(format!("Failed to load system: {e}"), None))?;

    // Auto-drain the observation queue (written by PostToolUse hook)
    let drained = persist::drain_queue(&mut system).unwrap_or(0);
    if drained > 0 {
        persist::save_system(&system).map_err(|e| {
            McpError::internal_error(format!("Failed to save after drain: {e}"), None)
        })?;
    }

    let summary = system.summary();
    let (state_bytes, persisted) = persist::system_stats();
    let (queue_lines, queue_bytes, queue_exists) = persist::queue_stats();

    let response = serde_json::json!({
        "identity": "NexCore IS an InsightEngine",
        "grounding": "NexCoreInsight ≡ ⟨σ, κ, μ, ∃, ς, ∅, N, ∂, Σ⟩",
        "domain_count": summary.domain_count,
        "domains": system.domains().iter().map(|d| serde_json::json!({
            "name": d.name,
            "description": d.description,
            "observation_count": d.observation_count,
        })).collect::<Vec<_>>(),
        "total_observations": summary.total_observations,
        "unique_keys": summary.unique_keys,
        "pattern_count": summary.pattern_count,
        "connection_count": summary.connection_count,
        "event_count": summary.event_count,
        "config": config_json(system.config()),
        "persistent": persisted,
        "state_bytes": state_bytes,
        "state_path": persist::system_path().to_string_lossy(),
        "queue": {
            "drained_this_call": drained,
            "pending": queue_lines,
            "bytes": queue_bytes,
            "exists": queue_exists,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Ingest observations into the system-level NexCoreInsight compositor.
///
/// Observations are auto-tagged with the domain name, enabling cross-domain
/// pattern detection. The domain is auto-registered if not already known.
///
/// Tier: T3 (full pipeline through unified engine)
pub fn insight_system_ingest(
    params: InsightSystemIngestParams,
) -> Result<CallToolResult, McpError> {
    let mut system = persist::load_system()
        .map_err(|e| McpError::internal_error(format!("Failed to load system: {e}"), None))?;

    let observations: Vec<nexcore_insight::Observation> =
        params.observations.iter().map(build_observation).collect();

    let events = system.ingest_batch_from(&params.domain, observations);

    persist::save_system(&system)
        .map_err(|e| McpError::internal_error(format!("Failed to save system: {e}"), None))?;

    let summary = system.summary();

    let response = serde_json::json!({
        "domain": params.domain,
        "events": events.iter().map(|e| format!("{e}")).collect::<Vec<_>>(),
        "event_count": events.len(),
        "domain_observations": system.domain_observation_count(&params.domain),
        "total_observations": summary.total_observations,
        "unique_keys": summary.unique_keys,
        "pattern_count": summary.pattern_count,
        "domain_count": summary.domain_count,
        "persistent": true,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Register a domain in the system-level NexCoreInsight compositor.
///
/// Domains are named subsystems that contribute observations:
/// - "guardian" — threat sensing, risk evaluation
/// - "brain" — implicit learning, artifact tracking
/// - "pv" — signal detection, adverse events
/// - "faers" — FDA adverse event reports
///
/// Tier: T2-P (λ + ∃ — location + existence)
pub fn insight_system_register(
    params: InsightSystemRegisterParams,
) -> Result<CallToolResult, McpError> {
    let mut system = persist::load_system()
        .map_err(|e| McpError::internal_error(format!("Failed to load system: {e}"), None))?;

    let desc = params.description.unwrap_or_default();
    system.register_domain(&params.name, &desc);

    persist::save_system(&system)
        .map_err(|e| McpError::internal_error(format!("Failed to save system: {e}"), None))?;

    let response = serde_json::json!({
        "registered": params.name,
        "description": desc,
        "domain_count": system.domain_count(),
        "domains": system.domains().iter().map(|d| &d.name).collect::<Vec<_>>(),
        "persistent": true,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Reset the system-level NexCoreInsight compositor state.
///
/// Tier: T1 (∅ — Void / clearing state)
pub fn insight_system_reset(_params: InsightSystemResetParams) -> Result<CallToolResult, McpError> {
    persist::reset_system()
        .map_err(|e| McpError::internal_error(format!("Failed to reset system: {e}"), None))?;

    let response = serde_json::json!({
        "reset": true,
        "message": "NexCoreInsight system state cleared. All domains and observations removed.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ── Read-Side Query Tools ───────────────────────────────────────────────────

/// Query insight engine observations and patterns with flexible filters.
///
/// Supports filtering by key prefix, tag, domain, and result limit.
/// Queries both domain-level and system-level engines.
///
/// Tier: T2-C (κ + μ + σ — comparison-filtered query)
pub fn insight_query(params: InsightQueryParams) -> Result<CallToolResult, McpError> {
    let limit = params.limit.unwrap_or(20);

    // Try system-level first (richer), fall back to domain-level
    let system_result = persist::load_system();
    let engine_result = persist::load_or_create();

    let engine = match (&system_result, &engine_result) {
        (Ok(sys), _) => sys.engine(),
        (_, Ok(eng)) => eng,
        (Err(e1), Err(e2)) => {
            return Err(McpError::internal_error(
                format!("Failed to load engines: system={e1}, domain={e2}"),
                None,
            ));
        }
    };

    // Collect matching patterns
    let mut matching_patterns: Vec<serde_json::Value> = engine
        .patterns()
        .into_iter()
        .filter(|p| {
            if let Some(ref prefix) = params.key_prefix {
                p.members.iter().any(|m| m.starts_with(prefix.as_str()))
            } else {
                true
            }
        })
        .take(limit)
        .map(|p| {
            serde_json::json!({
                "label": p.label,
                "members": p.members,
                "confidence": p.confidence,
                "occurrence_count": p.occurrence_count,
            })
        })
        .collect();

    // Collect matching connections
    let connections: Vec<serde_json::Value> = engine
        .connections()
        .iter()
        .filter(|c| {
            if let Some(ref prefix) = params.key_prefix {
                c.from.starts_with(prefix.as_str()) || c.to.starts_with(prefix.as_str())
            } else {
                true
            }
        })
        .take(limit)
        .map(|c| {
            serde_json::json!({
                "from": c.from,
                "to": c.to,
                "relation": c.relation,
                "strength": c.strength,
            })
        })
        .collect();

    // Collect matching novelties from events
    let novelties: Vec<serde_json::Value> = engine
        .events()
        .iter()
        .filter_map(|e| match e {
            InsightEvent::NoveltyDetected(n) => {
                if let Some(ref prefix) = params.key_prefix {
                    if n.novel_key.starts_with(prefix.as_str()) {
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    Some(n)
                }
            }
            _ => None,
        })
        .take(limit)
        .map(|n| {
            serde_json::json!({
                "key": n.novel_key,
                "score": n.score,
                "reason": format!("{:?}", n.reason),
            })
        })
        .collect();

    // Add domain info if querying system-level
    let domain_info = if let Ok(ref sys) = system_result {
        if let Some(ref domain_filter) = params.domain {
            matching_patterns.retain(|p| {
                p["members"].as_array().map_or(false, |members| {
                    members.iter().any(|m| {
                        m.as_str()
                            .map_or(false, |s| s.contains(domain_filter.as_str()))
                    })
                })
            });
        }
        Some(serde_json::json!({
            "domains": sys.domains().iter().map(|d| {
                serde_json::json!({
                    "name": d.name,
                    "observation_count": d.observation_count,
                })
            }).collect::<Vec<_>>(),
        }))
    } else {
        None
    };

    let response = serde_json::json!({
        "patterns": matching_patterns,
        "connections": connections,
        "novelties": novelties,
        "total_observations": engine.observation_count(),
        "total_patterns": engine.pattern_count(),
        "domain_info": domain_info,
        "filters_applied": {
            "key_prefix": params.key_prefix,
            "tag": params.tag,
            "domain": params.domain,
            "limit": limit,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// List detected novelties from the insight engine.
///
/// Returns observations that were flagged as novel (not matching existing patterns),
/// optionally filtered by minimum novelty score.
///
/// Tier: T2-C (∅ + ∃ + σ — void/existence detection in sequence)
pub fn insight_novelties(params: InsightNoveltiesParams) -> Result<CallToolResult, McpError> {
    let min_score = params.min_score.unwrap_or(0.0);
    let limit = params.limit.unwrap_or(20);

    // Try system-level first, fall back to domain-level
    let events = if let Ok(sys) = persist::load_system() {
        sys.engine().events().to_vec()
    } else {
        let engine = persist::load_or_create()
            .map_err(|e| McpError::internal_error(format!("Failed to load engine: {e}"), None))?;
        engine.events().to_vec()
    };

    let novelties: Vec<serde_json::Value> = events
        .iter()
        .filter_map(|e| match e {
            InsightEvent::NoveltyDetected(n) if n.score >= min_score => Some(n),
            _ => None,
        })
        .take(limit)
        .map(|n| {
            serde_json::json!({
                "key": n.novel_key,
                "score": n.score,
                "reason": format!("{:?}", n.reason),
                "is_highly_novel": n.is_highly_novel(),
                "timestamp": n.timestamp.to_rfc3339(),
            })
        })
        .collect();

    // Also collect suddenness events (threshold crossings)
    let suddenness: Vec<serde_json::Value> = events
        .iter()
        .filter_map(|e| match e {
            InsightEvent::SuddennessTrigger(s) => Some(s),
            _ => None,
        })
        .take(limit)
        .map(|s| {
            serde_json::json!({
                "metric": s.metric,
                "threshold": s.threshold,
                "value_before": s.value_before,
                "value_at_crossing": s.value_at_crossing,
                "magnitude": s.magnitude(),
                "rate_of_change": s.rate_of_change,
                "crossed_from_below": s.crossed_from_below(),
                "timestamp": s.detected_at.to_rfc3339(),
            })
        })
        .collect();

    let response = serde_json::json!({
        "novelties": novelties,
        "novelty_count": novelties.len(),
        "suddenness_events": suddenness,
        "suddenness_count": suddenness.len(),
        "filters": {
            "min_score": min_score,
            "limit": limit,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
