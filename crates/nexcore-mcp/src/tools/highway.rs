//! Digital Highway MCP tools: Infrastructure acceleration framework.
//!
//! Source: Chatburn, "Highways and Highway Transportation" (1923)
//! Transfers physical highway engineering to digital tool infrastructure.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Highway Classification | Comparison + Boundary | κ, ∂ |
//! | 7 Ideal Qualities | Quantity + Comparison | N, κ |
//! | Traffic Census | Frequency + Sequence | ν, σ |
//! | Destructive Factors | Quantity + Causality | N, → |
//! | Legitimate Field | Comparison + Sum | κ, Σ |

use crate::params::highway::{
    HighwayClassifyParams, HighwayDestructiveParams, HighwayGradeSeparateParams,
    HighwayInterchangeParams, HighwayLegitimateFieldParams, HighwayParallelPlanParams,
    HighwayQualityParams, HighwayTrafficCensusParams, ToolCallSpec,
};
use grounded::Confidence;
use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// Highway Classification (Chatburn Ch.8 — Road Type Selection)
// ============================================================================

/// Classify a tool into Digital Highway tiers I-IV.
pub fn highway_classify(params: HighwayClassifyParams) -> Result<CallToolResult, McpError> {
    let (class, class_name, sla_ms, description) = if params.internal_deps <= 3
        && params.avg_response_ms < 50.0
        && !params.calls_external
        && !params.stateful
    {
        (
            1,
            "Interstate",
            10,
            "Foundation — zero deps, pure functions, always available",
        )
    } else if params.internal_deps <= 25 && params.avg_response_ms < 200.0 && !params.calls_external
    {
        (
            2,
            "State",
            100,
            "Domain — typed params, domain errors, validated inputs",
        )
    } else if params.internal_deps <= 10 && !params.calls_external {
        (
            3,
            "County",
            500,
            "Orchestration — stateful, session-aware, composable",
        )
    } else {
        (
            4,
            "Township",
            5000,
            "Service — rate-limited, external APIs, monitored",
        )
    };

    let sla_met = params.avg_response_ms <= sla_ms as f64;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "tool": params.tool_name,
            "highway_class": class,
            "class_name": class_name,
            "description": description,
            "sla_ms": sla_ms,
            "actual_ms": params.avg_response_ms,
            "sla_met": sla_met,
            "factors": {
                "internal_deps": params.internal_deps,
                "calls_external": params.calls_external,
                "stateful": params.stateful
            },
            "recommendation": if sla_met {
                format!("Class {} ({}) — SLA met", class, class_name)
            } else {
                format!("Class {} ({}) — SLA VIOLATED ({:.0}ms > {}ms). Optimize or reclassify.",
                    class, class_name, params.avg_response_ms, sla_ms)
            }
        })
        .to_string(),
    )]))
}

// ============================================================================
// 7 Ideal Tool Qualities (Chatburn Ch.8 — Ideal Road Qualities)
// ============================================================================

/// Score a tool against the 7 Ideal Tool Qualities.
pub fn highway_quality(params: HighwayQualityParams) -> Result<CallToolResult, McpError> {
    // Q1: Fast to implement (fewer lines = better, cap at 500)
    let q1_construction = 1.0 - (params.impl_lines as f64 / 500.0).min(1.0);

    // Q2: Durable without maintenance (stable versions)
    let q2_durability = (params.stable_versions as f64 / 5.0).min(1.0);

    // Q3: Low cognitive overhead (fewer params = better, cap at 10)
    let q3_resistance = 1.0 - (params.param_count as f64 / 10.0).min(1.0);

    // Q4: Error-resistant (input validation)
    let q4_slip = if params.validates_input { 1.0 } else { 0.3 };

    // Q5: Clean error propagation (typed errors)
    let q5_drainage = if params.typed_errors { 1.0 } else { 0.2 };

    // Q6: Minimal output noise (inverse of param count, proxy)
    let q6_noise = 1.0 - (params.param_count as f64 / 15.0).min(1.0);

    // Q7: Developer adoption (calls per session)
    let q7_adoption = (params.calls_per_session / 10.0).min(1.0);

    // Weighted score (Chatburn favors durability and low resistance)
    let weights = [0.10, 0.20, 0.20, 0.15, 0.15, 0.05, 0.15];
    let scores = [
        q1_construction,
        q2_durability,
        q3_resistance,
        q4_slip,
        q5_drainage,
        q6_noise,
        q7_adoption,
    ];
    let total: f64 = weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum();

    let grade = if total >= 0.85 {
        "A — Excellent highway"
    } else if total >= 0.70 {
        "B — Good road"
    } else if total >= 0.50 {
        "C — Serviceable path"
    } else if total >= 0.30 {
        "D — Needs improvement"
    } else {
        "F — Trail, not a road"
    };

    let qualities = vec![
        json!({"name": "Construction cost", "score": q1_construction, "weight": weights[0], "chatburn": "Minimal initial expense"}),
        json!({"name": "Durability", "score": q2_durability, "weight": weights[1], "chatburn": "Extended service life"}),
        json!({"name": "Cognitive resistance", "score": q3_resistance, "weight": weights[2], "chatburn": "Low traction resistance"}),
        json!({"name": "Error resistance", "score": q4_slip, "weight": weights[3], "chatburn": "High slip resistance"}),
        json!({"name": "Error propagation", "score": q5_drainage, "weight": weights[4], "chatburn": "Sanitary drainage"}),
        json!({"name": "Output noise", "score": q6_noise, "weight": weights[5], "chatburn": "Acceptable noise"}),
        json!({"name": "Adoption", "score": q7_adoption, "weight": weights[6], "chatburn": "Community acceptability"}),
    ];

    // Find weakest quality
    let min_idx = scores
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let quality_names = [
        "construction cost",
        "durability",
        "cognitive resistance",
        "error resistance",
        "error propagation",
        "output noise",
        "adoption",
    ];

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "tool": params.tool_name,
            "total_score": (total * 1000.0).round() / 1000.0,
            "grade": grade,
            "qualities": qualities,
            "weakest": quality_names[min_idx],
            "recommendation": format!("Improve {} (score: {:.2}) for greatest impact",
                quality_names[min_idx], scores[min_idx])
        })
        .to_string(),
    )]))
}

// ============================================================================
// Destructive Factors (Chatburn Ch.8)
// ============================================================================

/// Compute destructive factor score for infrastructure stress.
pub fn highway_destructive(params: HighwayDestructiveParams) -> Result<CallToolResult, McpError> {
    // Chatburn: resistance = coefficient × weight × (1 + grade/100)
    // Digital: stress = frequency × payload × (1 + error_rate)
    let density_factor = (params.calls_per_hour / 100.0).min(1.0);
    let weight_factor = (params.avg_payload_bytes as f64 / 10000.0).min(1.0);
    let speed_factor = (params.avg_response_ms / 1000.0).min(1.0);
    let error_factor = 1.0 + params.error_rate * 10.0; // errors amplify stress

    let stress = density_factor * weight_factor * speed_factor * error_factor;

    let severity = if stress >= 0.8 {
        "CRITICAL — road is failing under load"
    } else if stress >= 0.5 {
        "HIGH — significant wear, maintenance urgent"
    } else if stress >= 0.2 {
        "MODERATE — normal wear, monitor"
    } else {
        "LOW — road in good condition"
    };

    // Chatburn: which factor dominates?
    let factors = [
        ("density (call frequency)", density_factor),
        ("weight (payload size)", weight_factor),
        ("speed (response latency)", speed_factor),
    ];
    let dominant = factors
        .iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(name, _)| *name)
        .unwrap_or("unknown");

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "tool": params.tool_name,
            "stress_score": (stress * 1000.0).round() / 1000.0,
            "severity": severity,
            "factors": {
                "density": density_factor,
                "weight": weight_factor,
                "speed": speed_factor,
                "error_amplifier": error_factor
            },
            "dominant_factor": dominant,
            "chatburn_formula": "resistance = coefficient × weight × (1 + grade/100)",
            "digital_formula": "stress = density × weight × speed × (1 + error_rate×10)",
            "mitigation": match dominant {
                "density (call frequency)" => "Apply rate limiting (rate_limit_token_bucket)",
                "weight (payload size)" => "Paginate responses, reduce payload",
                "speed (response latency)" => "Cache results, optimize hot path",
                _ => "Monitor and reassess"
            }
        })
        .to_string(),
    )]))
}

// ============================================================================
// Legitimate Field (Chatburn Ch.6)
// ============================================================================

/// Check if a tool is being used in its legitimate transportation field.
pub fn highway_legitimate_field(
    params: HighwayLegitimateFieldParams,
) -> Result<CallToolResult, McpError> {
    // Chatburn: each mode has a legitimate field
    // Class I (Interstate/Foundation): pure computation, no state
    // Class II (State/Domain): domain-specific structured analysis
    // Class III (County/Orchestration): workflow coordination, state management
    // Class IV (Township/Service): external APIs, user-facing interfaces

    let use_lower = params.use_case.to_lowercase();

    let (appropriate, reason) = match params.highway_class {
        1 => {
            if use_lower.contains("state")
                || use_lower.contains("session")
                || use_lower.contains("api")
            {
                (
                    false,
                    "Foundation tools should be stateless pure functions — use a Class III/IV tool",
                )
            } else {
                (
                    true,
                    "Pure computation is the legitimate field for Class I (Interstate)",
                )
            }
        }
        2 => {
            if use_lower.contains("external")
                || use_lower.contains("api call")
                || use_lower.contains("network")
            {
                (
                    false,
                    "Domain tools shouldn't call external APIs — use a Class IV tool",
                )
            } else {
                (
                    true,
                    "Domain analysis is the legitimate field for Class II (State)",
                )
            }
        }
        3 => {
            if use_lower.contains("pure") || use_lower.contains("stateless") {
                (
                    false,
                    "Orchestration tools are for stateful workflows — use a Class I tool for pure computation",
                )
            } else {
                (
                    true,
                    "Workflow coordination is the legitimate field for Class III (County)",
                )
            }
        }
        4 => (
            true,
            "Service tools handle all external interfaces (Class IV Township)",
        ),
        _ => (false, "Invalid highway class (must be 1-4)"),
    };

    let chatburn_quote = match params.highway_class {
        1 => "Like footpaths: simple, direct, no tolls",
        2 => "Like state highways: engineered for regional commerce",
        3 => "Like county roads: connecting communities with local knowledge",
        4 => "Like toll roads: premium service with maintained infrastructure",
        _ => "Unknown class",
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "tool": params.tool_name,
            "use_case": params.use_case,
            "highway_class": params.highway_class,
            "in_legitimate_field": appropriate,
            "reason": reason,
            "chatburn_analogy": chatburn_quote,
            "doctrine": "There is room and need for all forms of transportation — when each occupies its appropriate niche."
        })
        .to_string(),
    )]))
}

// ============================================================================
// Traffic Census (Chatburn Ch.8 — Traffic Counting)
// ============================================================================

/// Run a traffic census on tool usage — reads brain.db telemetry.
pub fn highway_traffic_census(
    params: HighwayTrafficCensusParams,
) -> Result<CallToolResult, McpError> {
    // Chatburn: "A traffic census is the first step in scientific road planning"
    // Digital: Read tool_usage from brain.db to get frequency/pattern data
    let brain_path = dirs::home_dir()
        .map(|h| h.join(".claude/brain/brain.db"))
        .unwrap_or_default();

    let category = params.category.to_lowercase();

    // Attempt to read from brain.db
    let census = if brain_path.exists() {
        match rusqlite::Connection::open_with_flags(
            &brain_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            Ok(conn) => {
                // Get tool usage from brain.db — schema: tool_name, total_calls, success_count, failure_count, last_used
                let mut stmt = conn
                    .prepare(
                        "SELECT tool_name, total_calls, success_count, failure_count \
                         FROM tool_usage \
                         WHERE tool_name LIKE ?1 \
                         ORDER BY total_calls DESC \
                         LIMIT 50",
                    )
                    .map_err(|e| McpError::internal_error(format!("SQL prepare: {e}"), None))?;

                let pattern = if category == "all" {
                    "%".to_string()
                } else {
                    format!("{category}%")
                };

                let rows: Vec<serde_json::Value> = stmt
                    .query_map([&pattern], |row| {
                        let tool: String = row.get(0)?;
                        let calls: i64 = row.get(1)?;
                        let successes: i64 = row.get(2)?;
                        let failures: i64 = row.get(3)?;
                        let error_rate = if calls > 0 {
                            failures as f64 / calls as f64
                        } else {
                            0.0
                        };
                        Ok(json!({
                            "tool": tool,
                            "calls": calls,
                            "successes": successes,
                            "failures": failures,
                            "error_rate": (error_rate * 1000.0).round() / 1000.0,
                            "highway_class": classify_by_name(&tool),
                        }))
                    })
                    .map_err(|e| McpError::internal_error(format!("SQL query: {e}"), None))?
                    .filter_map(|r| r.ok())
                    .collect();

                let total_calls: i64 = rows
                    .iter()
                    .filter_map(|r| r.get("calls").and_then(|v| v.as_i64()))
                    .sum();

                json!({
                    "source": "brain.db",
                    "category": category,
                    "tools_counted": rows.len(),
                    "total_calls": total_calls,
                    "tools": rows,
                })
            }
            Err(e) => json!({
                "source": "error",
                "error": format!("Could not open brain.db: {e}"),
                "tools": [],
            }),
        }
    } else {
        json!({
            "source": "not_found",
            "message": "brain.db not found — no telemetry data available",
            "tools": [],
        })
    };

    // Chatburn: classify traffic by type
    let class_distribution = json!({
        "class_I_interstate": "Foundation tools: pure math, string ops, hashing",
        "class_II_state": "Domain tools: PV signals, chemistry, STEM",
        "class_III_county": "Orchestration tools: brain, guardian, vigil",
        "class_IV_township": "Service tools: FAERS, Wolfram, Perplexity, GCloud",
    });

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "census": census,
            "class_distribution": class_distribution,
            "chatburn": "A traffic census is the first step in scientific road planning — count before you build.",
            "recommendation": "Use census data to identify hot paths (candidates for Class I optimization) and cold paths (candidates for removal)."
        })
        .to_string(),
    )]))
}

/// Heuristic classification by tool name prefix.
fn classify_by_name(tool: &str) -> u32 {
    if tool.starts_with("foundation_")
        || tool.starts_with("edit_distance_")
        || tool.starts_with("lex_primitiva_")
    {
        1 // Interstate — Foundation
    } else if tool.starts_with("pv_")
        || tool.starts_with("chemistry_")
        || tool.starts_with("stem_")
        || tool.starts_with("vigilance_")
        || tool.starts_with("validation_")
    {
        2 // State — Domain
    } else if tool.starts_with("brain_")
        || tool.starts_with("guardian_")
        || tool.starts_with("vigil_")
        || tool.starts_with("cytokine_")
        || tool.starts_with("grounded_")
    {
        3 // County — Orchestration
    } else if tool.starts_with("faers_")
        || tool.starts_with("wolfram_")
        || tool.starts_with("perplexity_")
        || tool.starts_with("gcloud_")
    {
        4 // Township — Service
    } else {
        2 // Default to Domain
    }
}

// ============================================================================
// Parallel Plan (Chatburn Multi-Lane Highway Design)
// ============================================================================

/// Plan parallel lanes for a batch of tool calls with grade separation.
pub fn highway_parallel_plan(
    params: HighwayParallelPlanParams,
) -> Result<CallToolResult, McpError> {
    // Chatburn: "Traffic lanes should be separated by speed and type"
    // Digital: Group tools by class, assign to parallel batches

    let mut express: Vec<&ToolCallSpec> = Vec::new(); // Class I (<10ms)
    let mut through: Vec<&ToolCallSpec> = Vec::new(); // Class II (<100ms)
    let mut local: Vec<&ToolCallSpec> = Vec::new(); // Class III (<500ms)
    let mut service: Vec<&ToolCallSpec> = Vec::new(); // Class IV (<5000ms)
    let mut dependent: Vec<&ToolCallSpec> = Vec::new(); // Has depends_on

    for tool in &params.tools {
        if tool.depends_on.is_some() {
            dependent.push(tool);
        } else {
            match tool.highway_class {
                1 => express.push(tool),
                2 => through.push(tool),
                3 => local.push(tool),
                4 => service.push(tool),
                _ => through.push(tool),
            }
        }
    }

    // Compute composed confidence for each parallel batch
    let express_conf = batch_confidence(&express);
    let through_conf = batch_confidence(&through);
    let local_conf = batch_confidence(&local);
    let service_conf = batch_confidence(&service);

    // Grade separation: can express + through run together? (within 10x latency)
    let can_merge_express_through = express.iter().all(|t| t.estimated_ms < 100.0)
        && through.iter().all(|t| t.estimated_ms < 200.0);

    // Build execution plan as ordered batches
    let mut batches: Vec<serde_json::Value> = Vec::new();
    let mut batch_num = 0;

    // Batch 0: Fast parallel (express + through if compatible)
    if !express.is_empty() || !through.is_empty() {
        batch_num += 1;
        if can_merge_express_through && !express.is_empty() && !through.is_empty() {
            let mut tools: Vec<&str> = express.iter().map(|t| t.tool.as_str()).collect();
            tools.extend(through.iter().map(|t| t.tool.as_str()));
            batches.push(json!({
                "batch": batch_num,
                "lane": "Express+Through (grade-merged)",
                "tools": tools,
                "parallel": true,
                "max_latency_ms": express.iter().chain(through.iter())
                    .map(|t| t.estimated_ms)
                    .fold(0.0_f64, f64::max),
                "composed_confidence": compose_all(&express.iter().chain(through.iter()).map(|t| t.confidence).collect::<Vec<_>>()),
            }));
        } else {
            if !express.is_empty() {
                batches.push(json!({
                    "batch": batch_num,
                    "lane": "Express (Class I Interstate)",
                    "tools": express.iter().map(|t| &t.tool).collect::<Vec<_>>(),
                    "parallel": true,
                    "max_latency_ms": express.iter().map(|t| t.estimated_ms).fold(0.0_f64, f64::max),
                    "composed_confidence": express_conf,
                }));
                batch_num += 1;
            }
            if !through.is_empty() {
                batches.push(json!({
                    "batch": batch_num,
                    "lane": "Through (Class II State)",
                    "tools": through.iter().map(|t| &t.tool).collect::<Vec<_>>(),
                    "parallel": true,
                    "max_latency_ms": through.iter().map(|t| t.estimated_ms).fold(0.0_f64, f64::max),
                    "composed_confidence": through_conf,
                }));
            }
        }
    }

    // Batch N-1: Local/Orchestration
    if !local.is_empty() {
        batch_num += 1;
        batches.push(json!({
            "batch": batch_num,
            "lane": "Local (Class III County)",
            "tools": local.iter().map(|t| &t.tool).collect::<Vec<_>>(),
            "parallel": true,
            "max_latency_ms": local.iter().map(|t| t.estimated_ms).fold(0.0_f64, f64::max),
            "composed_confidence": local_conf,
        }));
    }

    // Batch N: Service (slowest)
    if !service.is_empty() {
        batch_num += 1;
        batches.push(json!({
            "batch": batch_num,
            "lane": "Service (Class IV Township)",
            "tools": service.iter().map(|t| &t.tool).collect::<Vec<_>>(),
            "parallel": true,
            "max_latency_ms": service.iter().map(|t| t.estimated_ms).fold(0.0_f64, f64::max),
            "composed_confidence": service_conf,
        }));
    }

    // Dependent tools go last (merge lane)
    if !dependent.is_empty() {
        batch_num += 1;
        batches.push(json!({
            "batch": batch_num,
            "lane": "Merge (sequential dependencies)",
            "tools": dependent.iter().map(|t| json!({
                "tool": &t.tool,
                "depends_on": &t.depends_on,
            })).collect::<Vec<_>>(),
            "parallel": false,
            "note": "Execute sequentially — each depends on prior output",
        }));
    }

    // Overall composed confidence
    let all_confidences: Vec<f64> = params.tools.iter().map(|t| t.confidence).collect();
    let overall = compose_all(&all_confidences);

    let meets_gate = overall >= params.min_confidence;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "total_tools": params.tools.len(),
            "batches": batches,
            "total_batches": batch_num,
            "grade_separation": {
                "express_count": express.len(),
                "through_count": through.len(),
                "local_count": local.len(),
                "service_count": service.len(),
                "dependent_count": dependent.len(),
            },
            "composed_confidence": overall,
            "min_confidence": params.min_confidence,
            "meets_confidence_gate": meets_gate,
            "chatburn": "Separate traffic by speed and type. Fast lanes unimpeded by slow traffic.",
            "recommendation": if !meets_gate {
                format!("Composed confidence {:.1}% < gate {:.1}%. Consider reducing chain depth or adding evidence steps.",
                    overall * 100.0, params.min_confidence * 100.0)
            } else {
                format!("Plan is grade-separated into {} batches. Fire batch 1 first, then subsequent batches.",
                    batch_num)
            }
        })
        .to_string(),
    )]))
}

/// Compute multiplicative confidence for a batch of tools.
fn batch_confidence(tools: &[&ToolCallSpec]) -> f64 {
    if tools.is_empty() {
        return 1.0;
    }
    tools.iter().map(|t| t.confidence).product()
}

/// Compose all confidences multiplicatively.
fn compose_all(confidences: &[f64]) -> f64 {
    if confidences.is_empty() {
        return 1.0;
    }
    confidences.iter().copied().product()
}

// ============================================================================
// Interchange (Chatburn Ch.8 — Highway Interchange / Merging)
// ============================================================================

/// Merge N parallel tool results through grounded confidence composition.
pub fn highway_interchange(params: HighwayInterchangeParams) -> Result<CallToolResult, McpError> {
    // Chatburn: "The interchange is where traffic from multiple highways merges safely"
    // Digital: Compose confidence from N parallel results using selected strategy

    if params.results.is_empty() {
        return Err(McpError::invalid_params("No results to merge", None));
    }

    let confidences: Vec<f64> = params.results.iter().map(|r| r.confidence).collect();
    let weights: Vec<f64> = params.results.iter().map(|r| r.weight).collect();

    let merged_confidence = match params.strategy.to_lowercase().as_str() {
        "multiplicative" | "product" => {
            // P(A∧B∧C) = P(A) × P(B) × P(C)
            confidences.iter().copied().product()
        }
        "minimum" | "min" | "conservative" => {
            // Weakest link determines chain strength
            confidences.iter().copied().fold(1.0_f64, f64::min)
        }
        "average" | "mean" => {
            let sum: f64 = confidences.iter().sum();
            sum / confidences.len() as f64
        }
        "weighted" => {
            // Weighted average: Σ(wi × ci) / Σ(wi)
            let total_weight: f64 = weights.iter().sum();
            if total_weight <= 0.0 {
                return Err(McpError::invalid_params("Weights must sum to > 0", None));
            }
            let weighted_sum: f64 = confidences
                .iter()
                .zip(weights.iter())
                .map(|(c, w)| c * w)
                .sum();
            weighted_sum / total_weight
        }
        other => {
            return Err(McpError::invalid_params(
                format!(
                    "Unknown strategy '{other}'. Use: multiplicative, minimum, average, weighted"
                ),
                None,
            ));
        }
    };

    // Clamp to [0, 1]
    let merged_confidence = merged_confidence.clamp(0.0, 1.0);

    // Create grounded confidence for band classification
    let confidence = Confidence::new(merged_confidence)
        .map_err(|e| McpError::internal_error(format!("Confidence error: {e}"), None))?;

    let band = confidence.band();
    let meets_gate = merged_confidence >= params.min_confidence;

    // Collect merged values
    let merged_values: Vec<serde_json::Value> = params
        .results
        .iter()
        .map(|r| {
            json!({
                "tool": r.tool,
                "value": r.value,
                "confidence": r.confidence,
                "weight": r.weight,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "strategy": params.strategy,
            "lanes_merged": params.results.len(),
            "merged_confidence": merged_confidence,
            "confidence_pct": format!("{confidence}"),
            "band": format!("{band}"),
            "min_confidence": params.min_confidence,
            "meets_gate": meets_gate,
            "label": params.label,
            "lane_results": merged_values,
            "action_guidance": match band {
                grounded::ConfidenceBand::High => "PROCEED — high confidence from merged lanes",
                grounded::ConfidenceBand::Medium => "PROCEED WITH CAUTION — prepare fallback",
                grounded::ConfidenceBand::Low => "GATHER MORE EVIDENCE — add lanes or strengthen existing",
                grounded::ConfidenceBand::Negligible => "STOP — insufficient basis from all lanes",
            },
            "chatburn": "The interchange merges traffic safely — no lane crosses another unsafely.",
            "degradation_warning": if merged_confidence < 0.5 {
                Some("Multiplicative composition degrades fast. Consider using 'weighted' or 'average' strategy.")
            } else {
                None
            }
        })
        .to_string(),
    )]))
}

// ============================================================================
// Grade Separate (Chatburn Ch.8 — Grade Separation)
// ============================================================================

/// Sort tools into grade-separated batches by highway class.
pub fn highway_grade_separate(
    params: HighwayGradeSeparateParams,
) -> Result<CallToolResult, McpError> {
    // Chatburn: "Grade separation eliminates the crossing at grade"
    // Digital: Don't mix fast and slow tools in the same parallel batch

    let mut class_i: Vec<&ToolCallSpec> = Vec::new();
    let mut class_ii: Vec<&ToolCallSpec> = Vec::new();
    let mut class_iii: Vec<&ToolCallSpec> = Vec::new();
    let mut class_iv: Vec<&ToolCallSpec> = Vec::new();

    for tool in &params.tools {
        match tool.highway_class {
            1 => class_i.push(tool),
            2 => class_ii.push(tool),
            3 => class_iii.push(tool),
            4 => class_iv.push(tool),
            _ => class_ii.push(tool),
        }
    }

    let sla = |class: u32| -> u32 {
        match class {
            1 => 10,
            2 => 100,
            3 => 500,
            4 => 5000,
            _ => 100,
        }
    };

    let format_batch = |class: u32, name: &str, tools: &[&ToolCallSpec]| -> serde_json::Value {
        json!({
            "highway_class": class,
            "class_name": name,
            "sla_ms": sla(class),
            "tool_count": tools.len(),
            "tools": tools.iter().map(|t| json!({
                "tool": &t.tool,
                "estimated_ms": t.estimated_ms,
                "confidence": t.confidence,
                "sla_met": t.estimated_ms <= sla(class) as f64,
            })).collect::<Vec<_>>(),
            "can_parallel": true,
            "max_latency_ms": tools.iter().map(|t| t.estimated_ms).fold(0.0_f64, f64::max),
        })
    };

    let mut batches = Vec::new();
    if !class_i.is_empty() {
        batches.push(format_batch(1, "Interstate (Foundation)", &class_i));
    }
    if !class_ii.is_empty() {
        batches.push(format_batch(2, "State (Domain)", &class_ii));
    }
    if !class_iii.is_empty() {
        batches.push(format_batch(3, "County (Orchestration)", &class_iii));
    }
    if !class_iv.is_empty() {
        batches.push(format_batch(4, "Township (Service)", &class_iv));
    }

    // Mixing penalty: if tools span >2 classes, warn
    let classes_present = [
        !class_i.is_empty(),
        !class_ii.is_empty(),
        !class_iii.is_empty(),
        !class_iv.is_empty(),
    ]
    .iter()
    .filter(|&&b| b)
    .count();

    let speed_ratio = if !class_i.is_empty() && !class_iv.is_empty() {
        let fastest = class_i
            .iter()
            .map(|t| t.estimated_ms)
            .fold(f64::MAX, f64::min);
        let slowest = class_iv
            .iter()
            .map(|t| t.estimated_ms)
            .fold(0.0_f64, f64::max);
        if fastest > 0.0 {
            slowest / fastest
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "total_tools": params.tools.len(),
            "classes_present": classes_present,
            "batches": batches,
            "speed_ratio": (speed_ratio * 10.0).round() / 10.0,
            "mixing_warning": if speed_ratio > 100.0 {
                Some(format!("Speed ratio {:.0}x — fast tools will idle waiting for slow. Grade-separate into {} batches.", speed_ratio, classes_present))
            } else {
                None
            },
            "chatburn": "Grade separation eliminates the crossing at grade — fast traffic never waits for slow.",
            "execution_order": "Fire batches in order: Class I first (fastest), Class IV last (slowest). Results merge at the interchange."
        })
        .to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::highway::*;

    /// Extract JSON from a CallToolResult's first content item.
    fn extract_json(r: &CallToolResult) -> serde_json::Value {
        // Content is Annotated<RawContent> — serialize to JSON, extract text field
        let content_json = serde_json::to_value(&r.content[0]).expect("serialize content");
        let text = content_json["text"].as_str().expect("text field");
        serde_json::from_str(text).expect("parse tool output JSON")
    }

    #[test]
    fn test_classify_foundation_tool() {
        let r = highway_classify(HighwayClassifyParams {
            tool_name: "foundation_levenshtein".into(),
            internal_deps: 0,
            avg_response_ms: 2.0,
            calls_external: false,
            stateful: false,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        assert_eq!(json["highway_class"], 1);
        assert_eq!(json["class_name"], "Interstate");
        assert_eq!(json["sla_met"], true);
    }

    #[test]
    fn test_classify_service_tool() {
        let r = highway_classify(HighwayClassifyParams {
            tool_name: "faers_search".into(),
            internal_deps: 15,
            avg_response_ms: 2000.0,
            calls_external: true,
            stateful: false,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        assert_eq!(json["highway_class"], 4);
        assert_eq!(json["class_name"], "Township");
        assert_eq!(json["sla_met"], true); // 2000 < 5000
    }

    #[test]
    fn test_quality_scoring() {
        let r = highway_quality(HighwayQualityParams {
            tool_name: "pv_signal_complete".into(),
            impl_lines: 120,
            param_count: 4,
            typed_errors: true,
            validates_input: true,
            calls_per_session: 8.0,
            stable_versions: 3,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        let score = json["total_score"].as_f64().unwrap();
        assert!(
            score > 0.5,
            "Well-built tool should score > 0.5, got {score}"
        );
    }

    #[test]
    fn test_destructive_low_stress() {
        let r = highway_destructive(HighwayDestructiveParams {
            tool_name: "foundation_sha256".into(),
            calls_per_hour: 5.0,
            avg_payload_bytes: 100,
            avg_response_ms: 2.0,
            error_rate: 0.0,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        assert!(json["stress_score"].as_f64().unwrap() < 0.2);
        assert!(json["severity"].as_str().unwrap().contains("LOW"));
    }

    #[test]
    fn test_legitimate_field_correct() {
        let r = highway_legitimate_field(HighwayLegitimateFieldParams {
            tool_name: "foundation_sha256".into(),
            use_case: "hashing a string for deduplication".into(),
            highway_class: 1,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        assert_eq!(json["in_legitimate_field"], true);
    }

    #[test]
    fn test_legitimate_field_wrong() {
        let r = highway_legitimate_field(HighwayLegitimateFieldParams {
            tool_name: "foundation_sha256".into(),
            use_case: "managing user session state".into(),
            highway_class: 1,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        assert_eq!(json["in_legitimate_field"], false);
    }

    #[test]
    fn test_traffic_census() {
        let result = highway_traffic_census(HighwayTrafficCensusParams {
            category: "all".into(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_parallel_plan_grade_separation() {
        let r = highway_parallel_plan(HighwayParallelPlanParams {
            tools: vec![
                ToolCallSpec {
                    tool: "foundation_levenshtein".into(),
                    highway_class: 1,
                    estimated_ms: 5.0,
                    confidence: 0.95,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "pv_signal_complete".into(),
                    highway_class: 2,
                    estimated_ms: 50.0,
                    confidence: 0.9,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "faers_search".into(),
                    highway_class: 4,
                    estimated_ms: 2000.0,
                    confidence: 0.8,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "grounded_compose".into(),
                    highway_class: 3,
                    estimated_ms: 10.0,
                    confidence: 0.95,
                    depends_on: Some("pv_signal_complete".into()),
                },
            ],
            min_confidence: 0.5,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        assert_eq!(json["total_tools"], 4);
        assert!(json["total_batches"].as_u64().unwrap() >= 2);
        assert_eq!(json["grade_separation"]["dependent_count"], 1);
    }

    #[test]
    fn test_interchange_multiplicative() {
        let r = highway_interchange(HighwayInterchangeParams {
            results: vec![
                LaneResult {
                    tool: "tool_a".into(),
                    value: json!({"ok": true}),
                    confidence: 0.9,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "tool_b".into(),
                    value: json!({"ok": true}),
                    confidence: 0.8,
                    weight: 1.0,
                },
            ],
            strategy: "multiplicative".into(),
            min_confidence: 0.7,
            label: Some("test merge".into()),
        })
        .expect("should succeed");
        let json = extract_json(&r);
        let conf = json["merged_confidence"].as_f64().unwrap();
        assert!((conf - 0.72).abs() < 0.01, "0.9 * 0.8 = 0.72, got {conf}");
        assert_eq!(json["meets_gate"], true);
    }

    #[test]
    fn test_interchange_minimum() {
        let r = highway_interchange(HighwayInterchangeParams {
            results: vec![
                LaneResult {
                    tool: "a".into(),
                    value: json!(1),
                    confidence: 0.95,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "b".into(),
                    value: json!(2),
                    confidence: 0.6,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "c".into(),
                    value: json!(3),
                    confidence: 0.85,
                    weight: 1.0,
                },
            ],
            strategy: "minimum".into(),
            min_confidence: 0.7,
            label: None,
        })
        .expect("should succeed");
        let json = extract_json(&r);
        let conf = json["merged_confidence"].as_f64().unwrap();
        assert!((conf - 0.6).abs() < 0.01, "min should be 0.6, got {conf}");
        assert_eq!(json["meets_gate"], false); // 0.6 < 0.7
    }

    #[test]
    fn test_interchange_weighted() {
        let r = highway_interchange(HighwayInterchangeParams {
            results: vec![
                LaneResult {
                    tool: "trusted".into(),
                    value: json!("high"),
                    confidence: 0.9,
                    weight: 3.0,
                },
                LaneResult {
                    tool: "untrusted".into(),
                    value: json!("low"),
                    confidence: 0.5,
                    weight: 1.0,
                },
            ],
            strategy: "weighted".into(),
            min_confidence: 0.7,
            label: Some("weighted test".into()),
        })
        .expect("should succeed");
        let json = extract_json(&r);
        let conf = json["merged_confidence"].as_f64().unwrap();
        // (0.9*3 + 0.5*1) / (3+1) = 3.2/4 = 0.8
        assert!(
            (conf - 0.8).abs() < 0.01,
            "weighted should be 0.8, got {conf}"
        );
        assert_eq!(json["meets_gate"], true);
    }

    #[test]
    fn test_grade_separate() {
        let r = highway_grade_separate(HighwayGradeSeparateParams {
            tools: vec![
                ToolCallSpec {
                    tool: "foundation_sha256".into(),
                    highway_class: 1,
                    estimated_ms: 2.0,
                    confidence: 0.99,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "stem_taxonomy".into(),
                    highway_class: 2,
                    estimated_ms: 30.0,
                    confidence: 0.9,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "wolfram_calculate".into(),
                    highway_class: 4,
                    estimated_ms: 3000.0,
                    confidence: 0.85,
                    depends_on: None,
                },
            ],
        })
        .expect("should succeed");
        let json = extract_json(&r);
        assert_eq!(json["total_tools"], 3);
        assert_eq!(json["classes_present"], 3);
        assert!(json["speed_ratio"].as_f64().unwrap() > 100.0);
        assert!(json["mixing_warning"].is_string());
    }

    #[test]
    fn test_classify_by_name_heuristic() {
        assert_eq!(classify_by_name("foundation_levenshtein"), 1);
        assert_eq!(classify_by_name("pv_signal_complete"), 2);
        assert_eq!(classify_by_name("brain_session_new"), 3);
        assert_eq!(classify_by_name("faers_search"), 4);
        assert_eq!(classify_by_name("unknown_tool"), 2);
    }

    // ================================================================
    // USE CASE 1: Drug Safety Investigation Pipeline
    // Real scenario: investigating a drug-event pair requires
    // parallel FAERS query + PV signal + literature search,
    // then merging results with grounded confidence.
    // ================================================================
    #[test]
    fn usecase_drug_safety_pipeline() {
        // Step 1: Plan the parallel execution
        let plan = highway_parallel_plan(HighwayParallelPlanParams {
            tools: vec![
                ToolCallSpec {
                    tool: "faers_search".into(),
                    highway_class: 4,
                    estimated_ms: 2000.0,
                    confidence: 0.85,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "pv_signal_complete".into(),
                    highway_class: 2,
                    estimated_ms: 50.0,
                    confidence: 0.95,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "foundation_concept_grep".into(),
                    highway_class: 1,
                    estimated_ms: 5.0,
                    confidence: 0.9,
                    depends_on: None,
                },
                // guardian_evaluate_pv depends on pv_signal_complete
                ToolCallSpec {
                    tool: "guardian_evaluate_pv".into(),
                    highway_class: 3,
                    estimated_ms: 100.0,
                    confidence: 0.9,
                    depends_on: Some("pv_signal_complete".into()),
                },
            ],
            min_confidence: 0.7,
        })
        .expect("plan");
        let plan_json = extract_json(&plan);

        // Grade separation should fire Class I first, Class IV last
        assert!(plan_json["total_batches"].as_u64().unwrap_or(0) >= 2);
        assert_eq!(plan_json["grade_separation"]["dependent_count"], 1);

        // Step 2: Simulate results and merge via interchange
        let merged = highway_interchange(HighwayInterchangeParams {
            results: vec![
                LaneResult {
                    tool: "faers_search".into(),
                    value: json!({"cases": 142, "serious": 38}),
                    confidence: 0.85,
                    weight: 2.0,
                },
                LaneResult {
                    tool: "pv_signal_complete".into(),
                    value: json!({"prr": 3.2, "ror": 2.8, "signal": true}),
                    confidence: 0.95,
                    weight: 3.0,
                },
                LaneResult {
                    tool: "foundation_concept_grep".into(),
                    value: json!({"matches": 7}),
                    confidence: 0.9,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "guardian_evaluate_pv".into(),
                    value: json!({"risk": "medium", "action": "monitor"}),
                    confidence: 0.9,
                    weight: 2.0,
                },
            ],
            strategy: "weighted".into(),
            min_confidence: 0.7,
            label: Some("Drug X + Hepatotoxicity safety assessment".into()),
        })
        .expect("interchange");
        let merged_json = extract_json(&merged);

        // Weighted merge: (0.85*2 + 0.95*3 + 0.9*1 + 0.9*2) / (2+3+1+2) = 7.25/8 = 0.906
        assert!(merged_json["merged_confidence"].as_f64().unwrap_or(0.0) > 0.85);
        assert_eq!(merged_json["meets_gate"], true);
        // 0.906 is ≥0.80 but <0.95 → MEDIUM band per grounded::ConfidenceBand
        assert_eq!(merged_json["band"], "MEDIUM (≥80%)");
    }

    // ================================================================
    // USE CASE 2: Crate Quality Gate (before publishing)
    // Score a tool's quality, check its highway class, and verify
    // it's being used in its legitimate field.
    // ================================================================
    #[test]
    fn usecase_crate_quality_gate() {
        // Classify the tool
        let class = highway_classify(HighwayClassifyParams {
            tool_name: "pv_signal_complete".into(),
            internal_deps: 5,
            avg_response_ms: 45.0,
            calls_external: false,
            stateful: false,
        })
        .expect("classify");
        let class_json = extract_json(&class);
        assert_eq!(class_json["highway_class"], 2); // Domain tool

        // Score quality
        let quality = highway_quality(HighwayQualityParams {
            tool_name: "pv_signal_complete".into(),
            impl_lines: 120,
            param_count: 4,
            typed_errors: true,
            validates_input: true,
            calls_per_session: 8.0,
            stable_versions: 3,
        })
        .expect("quality");
        let quality_json = extract_json(&quality);
        let grade = quality_json["grade"].as_str().unwrap_or("F");
        // Grade includes suffix like "B — Good road"
        assert!(
            grade.starts_with("A") || grade.starts_with("B"),
            "Well-built tool should be A or B, got {grade}"
        );

        // Check legitimate field
        let legit = highway_legitimate_field(HighwayLegitimateFieldParams {
            tool_name: "pv_signal_complete".into(),
            use_case: "computing disproportionality signals for drug-event pairs".into(),
            highway_class: 2,
        })
        .expect("legitimate");
        let legit_json = extract_json(&legit);
        assert_eq!(legit_json["in_legitimate_field"], true);
    }

    // ================================================================
    // USE CASE 3: Session Start Health Check
    // Traffic census → identify overloaded tools → destructive analysis
    // ================================================================
    #[test]
    fn usecase_session_health_check() {
        // Run traffic census
        let census = highway_traffic_census(HighwayTrafficCensusParams {
            category: "all".into(),
        })
        .expect("census");
        let census_json = extract_json(&census);
        // Should have some data (we know brain.db has tool_usage)
        assert!(
            census_json["census"]["tools_counted"].as_u64().unwrap_or(0) > 0
                || census_json["census"]["source"] == "not_found"
        );

        // Simulate destructive analysis on a heavily-used tool (Bash: 8599 calls)
        let stress = highway_destructive(HighwayDestructiveParams {
            tool_name: "Bash".into(),
            calls_per_hour: 100.0,   // heavy usage
            avg_payload_bytes: 5000, // substantial payloads
            avg_response_ms: 5000.0, // can be slow
            error_rate: 0.05,
        })
        .expect("destructive");
        let stress_json = extract_json(&stress);
        // density=1.0, weight=0.5, speed=1.0, error=1.5 → stress=0.75
        assert!(stress_json["stress_score"].as_f64().unwrap_or(0.0) > 0.5);
    }

    // ================================================================
    // USE CASE 4: Multi-Source Research (Parallel Evidence Gathering)
    // Searching for a concept across multiple sources simultaneously,
    // then merging with confidence gating.
    // ================================================================
    #[test]
    fn usecase_parallel_evidence_gathering() {
        // Grade separate: don't mix fast foundation tools with slow service tools
        let separated = highway_grade_separate(HighwayGradeSeparateParams {
            tools: vec![
                ToolCallSpec {
                    tool: "foundation_concept_grep".into(),
                    highway_class: 1,
                    estimated_ms: 5.0,
                    confidence: 0.9,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "guidelines_search".into(),
                    highway_class: 2,
                    estimated_ms: 30.0,
                    confidence: 0.85,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "faers_drug_events".into(),
                    highway_class: 4,
                    estimated_ms: 3000.0,
                    confidence: 0.8,
                    depends_on: None,
                },
                ToolCallSpec {
                    tool: "perplexity_search".into(),
                    highway_class: 4,
                    estimated_ms: 4000.0,
                    confidence: 0.7,
                    depends_on: None,
                },
            ],
        })
        .expect("grade separate");
        let sep_json = extract_json(&separated);

        // Should warn about mixing Class I and IV
        assert!(sep_json["mixing_warning"].is_string());
        assert!(sep_json["speed_ratio"].as_f64().unwrap_or(0.0) > 100.0);

        // Merge results with minimum strategy (weakest-link for safety decisions)
        let merged = highway_interchange(HighwayInterchangeParams {
            results: vec![
                LaneResult {
                    tool: "foundation_concept_grep".into(),
                    value: json!({"matches": 12}),
                    confidence: 0.9,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "guidelines_search".into(),
                    value: json!({"ich_ref": "E2A Section 3.2"}),
                    confidence: 0.85,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "faers_drug_events".into(),
                    value: json!({"total": 450}),
                    confidence: 0.8,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "perplexity_search".into(),
                    value: json!({"summary": "established risk"}),
                    confidence: 0.7,
                    weight: 1.0,
                },
            ],
            strategy: "minimum".into(),
            min_confidence: 0.75,
            label: Some("Multi-source hepatotoxicity evidence".into()),
        })
        .expect("interchange");
        let merged_json = extract_json(&merged);

        // Minimum strategy: weakest link is perplexity at 0.7
        assert!((merged_json["merged_confidence"].as_f64().unwrap_or(0.0) - 0.7).abs() < 0.01);
        assert_eq!(merged_json["meets_gate"], false); // 0.7 < 0.75 — need more evidence
    }

    // ================================================================
    // USE CASE 5: Confidence Decay Detection
    // 3-hop tool chain degrades confidence multiplicatively.
    // Highway tools make this visible.
    // ================================================================
    #[test]
    fn usecase_confidence_decay_chain() {
        // Simulate a 3-hop chain: faers → pv_signal → guardian_evaluate
        // Each hop has good confidence individually, but chain multiplies
        let chain = highway_interchange(HighwayInterchangeParams {
            results: vec![
                LaneResult {
                    tool: "faers_search".into(),
                    value: json!({"cases": 50}),
                    confidence: 0.88,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "pv_signal_complete".into(),
                    value: json!({"prr": 2.1}),
                    confidence: 0.92,
                    weight: 1.0,
                },
                LaneResult {
                    tool: "guardian_evaluate_pv".into(),
                    value: json!({"risk": "low"}),
                    confidence: 0.85,
                    weight: 1.0,
                },
            ],
            strategy: "multiplicative".into(),
            min_confidence: 0.7,
            label: Some("3-hop chain confidence decay".into()),
        })
        .expect("chain");
        let chain_json = extract_json(&chain);

        // 0.88 * 0.92 * 0.85 = 0.688 — BELOW the 0.7 gate!
        let conf = chain_json["merged_confidence"].as_f64().unwrap_or(1.0);
        assert!(conf < 0.7, "3-hop chain should decay below 0.7, got {conf}");
        assert_eq!(chain_json["meets_gate"], false);
        // This is exactly why highway tools matter: makes invisible decay visible
    }
}
