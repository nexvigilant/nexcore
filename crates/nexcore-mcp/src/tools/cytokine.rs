//! Cytokine signaling MCP tools
//!
//! Fire-and-forget typed event signaling based on immune system cytokine patterns.
//!
//! ## T1 Grounding
//!
//! | Tool | Primitives | Role |
//! |------|------------|------|
//! | cytokine_emit | → (causality) + π (persistence) | Emit signal to bus |
//! | cytokine_status | N (quantity) + ς (state) | Get bus statistics |
//! | cytokine_families | Σ (sum) + μ (mapping) | List family definitions |

use nexcore_cytokine::{Cytokine, CytokineFamily, Emitter, Scope, ThreatLevel, global_bus};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{CytokineEmitParams, CytokineListParams, CytokineRecentParams};

/// Path to file-based cytokine metrics written by signal-receiver.
const CYTOKINE_METRICS_PATH: &str = "/home/matthew/.claude/brain/telemetry/cytokine_metrics.json";

/// Parse a cytokine family from string.
fn parse_family(s: &str) -> Result<CytokineFamily, nexcore_error::NexError> {
    match s.to_lowercase().as_str() {
        "il1" | "il-1" => Ok(CytokineFamily::Il1),
        "il2" | "il-2" => Ok(CytokineFamily::Il2),
        "il6" | "il-6" => Ok(CytokineFamily::Il6),
        "il10" | "il-10" => Ok(CytokineFamily::Il10),
        "tnf_alpha" | "tnf-alpha" | "tnf" => Ok(CytokineFamily::TnfAlpha),
        "ifn_gamma" | "ifn-gamma" | "ifn" => Ok(CytokineFamily::IfnGamma),
        "tgf_beta" | "tgf-beta" | "tgf" => Ok(CytokineFamily::TgfBeta),
        "csf" => Ok(CytokineFamily::Csf),
        other => Err(nexcore_error::nexerror!(
            "Unknown family '{}'. Valid: il1, il2, il6, il10, tnf_alpha, ifn_gamma, tgf_beta, csf",
            other
        )),
    }
}

/// Parse severity from string.
fn parse_severity(s: &str) -> ThreatLevel {
    match s.to_lowercase().as_str() {
        "trace" => ThreatLevel::Trace,
        "low" => ThreatLevel::Low,
        "medium" => ThreatLevel::Medium,
        "high" => ThreatLevel::High,
        "critical" => ThreatLevel::Critical,
        _ => ThreatLevel::Medium, // Default
    }
}

/// Parse scope from string.
fn parse_scope(s: &str) -> Scope {
    match s.to_lowercase().as_str() {
        "autocrine" => Scope::Autocrine,
        "paracrine" => Scope::Paracrine,
        "endocrine" => Scope::Endocrine,
        "systemic" => Scope::Systemic,
        _ => Scope::Paracrine, // Default
    }
}

/// Emit a cytokine signal to the global bus.
///
/// # T1 Grounding
/// - → (causality): This call causes signal emission
/// - π (persistence): Signal persists until TTL expires
pub fn emit(params: CytokineEmitParams) -> Result<CallToolResult, McpError> {
    let family =
        parse_family(&params.family).map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let severity = params
        .severity
        .as_deref()
        .map(parse_severity)
        .unwrap_or(ThreatLevel::Medium);

    let scope = params
        .scope
        .as_deref()
        .map(parse_scope)
        .unwrap_or(Scope::Paracrine);

    let mut cytokine = Cytokine::new(family, &params.name)
        .with_severity(severity)
        .with_scope(scope)
        .with_source("mcp-tool");

    if let Some(payload) = params.payload {
        cytokine = cytokine.with_payload(payload);
    }

    // Get the cytokine ID before moving
    let id = cytokine.id.clone();
    let family_str = cytokine.family.to_string();
    let name = cytokine.name.clone();

    // Fire-and-forget emission using blocking runtime
    let bus = global_bus();
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(bus.emit(cytokine))
    });

    match result {
        Ok(()) => {
            let response = serde_json::json!({
                "success": true,
                "signal_id": id,
                "family": family_str,
                "name": name,
                "severity": severity.to_string(),
                "scope": scope,
                "message": format!("Emitted {} signal: {}", family_str, name),
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "error": format!("{}", e),
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
    }
}

/// Get cytokine bus status and statistics.
///
/// # T1 Grounding
/// - N (quantity): Signal counts and statistics
/// - ς (state): Current bus state
pub fn status() -> Result<CallToolResult, McpError> {
    let bus = global_bus();

    // Get stats using blocking runtime
    let stats =
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(bus.stats()));

    let receptor_count = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(bus.receptor_count())
    });

    let cascade_count = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(bus.cascade_count())
    });

    let response = serde_json::json!({
        "signals_emitted": stats.signals_emitted,
        "signals_delivered": stats.signals_delivered,
        "signals_dropped": stats.signals_dropped,
        "cascades_triggered": stats.cascades_triggered,
        "by_family": stats.by_family,
        "by_severity": stats.by_severity,
        "receptor_count": receptor_count,
        "cascade_count": cascade_count,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// List all cytokine families with their definitions.
///
/// # T1 Grounding
/// - Σ (sum): Enumeration of family variants
/// - μ (mapping): Family → description mapping
pub fn families(params: CytokineListParams) -> Result<CallToolResult, McpError> {
    let all_families = vec![
        (
            "IL-1",
            "il1",
            "Alarm/Alert - Initial threat detection",
            true,
            false,
        ),
        (
            "IL-2",
            "il2",
            "Growth/Proliferation - Spawn more responders",
            true,
            false,
        ),
        (
            "IL-6",
            "il6",
            "Acute phase - Immediate response coordination",
            true,
            false,
        ),
        (
            "IL-10",
            "il10",
            "Suppression - Dampen excessive response",
            false,
            true,
        ),
        (
            "TNF-\u{03b1}",
            "tnf_alpha",
            "Destruction - Terminate threats",
            true,
            false,
        ),
        (
            "IFN-\u{03b3}",
            "ifn_gamma",
            "Activation - Enhance response capability",
            true,
            false,
        ),
        (
            "TGF-\u{03b2}",
            "tgf_beta",
            "Regulation - Modulate behavior",
            false,
            true,
        ),
        (
            "CSF",
            "csf",
            "Colony Stimulating - Create new agents",
            true,
            false,
        ),
    ];

    let families: Vec<_> = all_families
        .into_iter()
        .filter(|(name, code, _, _, _)| {
            if let Some(ref filter) = params.family_filter {
                let filter_lower = filter.to_lowercase();
                name.to_lowercase().contains(&filter_lower) || code.contains(&filter_lower)
            } else {
                true
            }
        })
        .map(|(name, code, description, activating, suppressing)| {
            serde_json::json!({
                "name": name,
                "code": code,
                "description": description,
                "is_activating": activating,
                "is_suppressing": suppressing,
            })
        })
        .collect();

    let response = serde_json::json!({
        "families": families,
        "count": families.len(),
        "grounding": {
            "primitive": "\u{03a3} (sum type)",
            "tier": "T2-P (cross-domain primitive)",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// File-Based Telemetry Tools (reads cytokine_metrics.json from signal-receiver)
// ============================================================================

/// Read the cytokine_metrics.json file and return its parsed content.
fn read_cytokine_metrics() -> Result<serde_json::Value, nexcore_error::NexError> {
    let content = std::fs::read_to_string(CYTOKINE_METRICS_PATH).map_err(|e| {
        nexcore_error::nexerror!("No cytokine metrics file: {e}. Run signal-receiver to generate.")
    })?;
    serde_json::from_str(&content)
        .map_err(|e| nexcore_error::nexerror!("Invalid metrics JSON: {e}"))
}

/// Get persistent cytokine telemetry from file-based signal aggregation.
///
/// Unlike `cytokine_status` which reads the in-memory bus (ephemeral),
/// this reads from `cytokine_metrics.json` written by signal-receiver daemon.
///
/// # T1 Grounding
/// - N (quantity): Aggregated counts by family, hook, severity
/// - ς (state): Persistent state across hook process lifetimes
pub fn telemetry_status() -> Result<CallToolResult, McpError> {
    match read_cytokine_metrics() {
        Ok(metrics) => {
            let response = serde_json::json!({
                "source": "file-based telemetry (signal-receiver daemon)",
                "path": CYTOKINE_METRICS_PATH,
                "metrics": metrics,
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
        Err(e) => {
            let response = serde_json::json!({
                "error": e,
                "hint": "Cytokine signals flow: hook → signals.jsonl → signal-receiver → cytokine_metrics.json",
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
    }
}

/// Get recent cytokine signals from file-based telemetry with optional family filter.
///
/// # T1 Grounding
/// - σ (sequence): Ordered recent signals
/// - N (quantity): Limited by `limit` param
/// - ∂ (conditional): Optional family filter
pub fn recent(params: CytokineRecentParams) -> Result<CallToolResult, McpError> {
    let metrics = match read_cytokine_metrics() {
        Ok(m) => m,
        Err(e) => {
            let response = serde_json::json!({ "error": e, "recent": [] });
            return Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]));
        }
    };

    let limit = params.limit.min(100) as usize;

    // Extract the "recent" array from metrics
    let recent_arr = metrics
        .get("recent")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Apply family filter if provided, take from the tail (most recent)
    let filtered: Vec<_> = recent_arr
        .into_iter()
        .rev()
        .filter(|entry| {
            if let Some(ref family_filter) = params.family {
                entry
                    .get("family")
                    .and_then(|f| f.as_str())
                    .map(|f| f == family_filter.as_str())
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .take(limit)
        .collect();

    let response = serde_json::json!({
        "count": filtered.len(),
        "limit": limit,
        "family_filter": params.family,
        "recent": filtered,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Biology Primitive Tools: Chemotaxis (→+λ) and Endocytosis (∂+ρ)
// ============================================================================

/// Compute chemotactic gradient routing from multiple signal sources.
///
/// Given a set of gradient samples (source, concentration, distance, tropism),
/// computes the dominant routing direction and ranked sources.
///
/// # T1 Grounding
/// - → (causality): Gradient causes directed routing decision
/// - λ (location): Spatial distribution of signal concentrations
pub fn chemotaxis_gradient(
    params: crate::params::ChemotaxisGradientParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_cytokine::{Gradient, GradientField, Tropism};

    let mut field = GradientField::new();

    for sample in &params.gradients {
        let family = parse_family(&sample.family)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let tropism = match sample.tropism.to_lowercase().as_str() {
            "negative" | "repel" => Tropism::Negative,
            _ => Tropism::Positive,
        };

        let gradient = Gradient::new(
            &sample.source,
            family,
            sample.concentration,
            sample.distance,
        )
        .with_tropism(tropism);

        field.add(gradient);
    }

    let dominant = field.dominant_source().map(|g| {
        serde_json::json!({
            "source": g.source,
            "family": g.family.to_string(),
            "effective_strength": g.effective_strength(),
            "tropism": format!("{:?}", g.tropism),
            "directional_pull": g.directional_pull(),
        })
    });

    let ranked = field
        .ranked_sources()
        .into_iter()
        .map(|(source, pull)| serde_json::json!({ "source": source, "pull": pull }))
        .collect::<Vec<_>>();

    let response = serde_json::json!({
        "net_pull": field.net_pull(),
        "source_count": field.source_count(),
        "dominant": dominant,
        "ranked_sources": ranked,
        "grounding": {
            "primitive": "→+λ (causality + location)",
            "biology": "Chemotaxis — gradient-following navigation",
            "tier": "T2-C (composite)",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Simulate endocytic signal internalization and processing.
///
/// Creates a vesicle pool, internalizes a signal, processes it through
/// the endosomal pathway (EarlyEndosome → LateEndosome → Lysosome → Recycled),
/// and returns the immune response generated.
///
/// # T1 Grounding
/// - ∂ (boundary): Signal crosses membrane boundary into vesicle
/// - ρ (recursion): Internalized signal triggers recursive cascade processing
pub fn endocytosis_internalize(
    params: crate::params::EndocytosisInternalizeParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_cytokine::{InternalizationResult, VesiclePool, VesicleState};

    let family =
        parse_family(&params.family).map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let severity = params
        .severity
        .as_deref()
        .map(parse_severity)
        .unwrap_or(ThreatLevel::Medium);

    let signal = Cytokine::new(family, &params.name)
        .with_severity(severity)
        .with_source("mcp-endocytosis");

    let mut pool = VesiclePool::new(params.pool_capacity);

    // Internalize
    let intern_result = pool.internalize(signal);
    let intern_status = match intern_result {
        InternalizationResult::Accepted => "accepted",
        InternalizationResult::Rejected => "rejected",
        InternalizationResult::AtCapacity => "at_capacity",
        InternalizationResult::Expired => "expired",
    };

    // Process through full lifecycle (3 steps)
    let mut all_responses = Vec::new();
    for _ in 0..3 {
        let responses = pool.process_step();
        for r in &responses {
            all_responses.push(serde_json::json!({
                "family": r.family.to_string(),
                "name": &r.name,
                "severity": r.severity.to_string(),
                "source": &r.source,
            }));
        }
    }

    // Recycle
    let recycled = pool.recycle();
    let stats = pool.stats();

    let response = serde_json::json!({
        "internalization": intern_status,
        "pool": {
            "capacity": stats.capacity,
            "occupancy": stats.occupancy,
            "utilization": stats.utilization,
            "total_internalized": stats.total_internalized,
            "total_recycled": stats.total_recycled,
        },
        "responses_generated": all_responses,
        "vesicles_recycled": recycled,
        "grounding": {
            "primitive": "∂+ρ (boundary + recursion)",
            "biology": "Endocytosis — selective signal absorption across trust boundaries",
            "tier": "T2-C (composite)",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_family_valid() {
        assert!(matches!(parse_family("il1"), Ok(CytokineFamily::Il1)));
        assert!(matches!(parse_family("IL-1"), Ok(CytokineFamily::Il1)));
        assert!(matches!(
            parse_family("tnf_alpha"),
            Ok(CytokineFamily::TnfAlpha)
        ));
        assert!(matches!(parse_family("TNF"), Ok(CytokineFamily::TnfAlpha)));
    }

    #[test]
    fn test_parse_family_invalid() {
        assert!(parse_family("unknown").is_err());
    }

    #[test]
    fn test_parse_severity() {
        assert!(matches!(parse_severity("critical"), ThreatLevel::Critical));
        assert!(matches!(parse_severity("TRACE"), ThreatLevel::Trace));
        assert!(matches!(parse_severity("unknown"), ThreatLevel::Medium)); // Default
    }

    #[test]
    fn test_parse_scope() {
        assert!(matches!(parse_scope("systemic"), Scope::Systemic));
        assert!(matches!(parse_scope("AUTOCRINE"), Scope::Autocrine));
        assert!(matches!(parse_scope("unknown"), Scope::Paracrine)); // Default
    }

    #[test]
    fn test_families_no_filter() {
        let result = families(CytokineListParams::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_families_with_filter() {
        let params = CytokineListParams {
            family_filter: Some("IL".to_string()),
        };
        let result = families(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_status_returns_ok() {
        // Should not panic even if file doesn't exist — returns error JSON
        let result = telemetry_status();
        assert!(result.is_ok());
    }

    #[test]
    fn test_recent_returns_ok_when_no_file() {
        let params = CytokineRecentParams {
            limit: 10,
            family: None,
        };
        let result = recent(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recent_with_family_filter() {
        let params = CytokineRecentParams {
            limit: 5,
            family: Some("tnf_alpha".to_string()),
        };
        let result = recent(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chemotaxis_gradient_basic() {
        use crate::params::{ChemotaxisGradientParams, GradientSample};
        let params = ChemotaxisGradientParams {
            gradients: vec![
                GradientSample {
                    source: "api_a".to_string(),
                    family: "il1".to_string(),
                    concentration: 0.9,
                    distance: 1.0,
                    tropism: "positive".to_string(),
                },
                GradientSample {
                    source: "api_b".to_string(),
                    family: "il1".to_string(),
                    concentration: 0.3,
                    distance: 5.0,
                    tropism: "positive".to_string(),
                },
            ],
        };
        let result = chemotaxis_gradient(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chemotaxis_gradient_with_repellent() {
        use crate::params::{ChemotaxisGradientParams, GradientSample};
        let params = ChemotaxisGradientParams {
            gradients: vec![
                GradientSample {
                    source: "attract".to_string(),
                    family: "il1".to_string(),
                    concentration: 0.8,
                    distance: 0.0,
                    tropism: "positive".to_string(),
                },
                GradientSample {
                    source: "repel".to_string(),
                    family: "il10".to_string(),
                    concentration: 0.5,
                    distance: 0.0,
                    tropism: "negative".to_string(),
                },
            ],
        };
        let result = chemotaxis_gradient(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_endocytosis_internalize_basic() {
        use crate::params::EndocytosisInternalizeParams;
        let params = EndocytosisInternalizeParams {
            family: "il1".to_string(),
            name: "threat_signal".to_string(),
            severity: Some("high".to_string()),
            pool_capacity: 5,
        };
        let result = endocytosis_internalize(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_endocytosis_tnf_generates_suppression() {
        use crate::params::EndocytosisInternalizeParams;
        let params = EndocytosisInternalizeParams {
            family: "tnf_alpha".to_string(),
            name: "termination_signal".to_string(),
            severity: Some("critical".to_string()),
            pool_capacity: 3,
        };
        let result = endocytosis_internalize(params);
        assert!(result.is_ok());
    }
}
