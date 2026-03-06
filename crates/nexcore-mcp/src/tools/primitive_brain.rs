//! Primitive Brain MCP tools
//!
//! Makes the 15 T1 Lex Primitiva first-class citizens in working memory.
//! Decompose concepts, query by primitive, measure distance, check conservation,
//! and compose new structures — all persisted through the brain artifact system.
//!
//! Conservation law (source: primitives.ipynb Part II):
//!   ∃ = ∂(×(ς, ∅))
//! Existence is boundary applied to the product of state and nothing.

use crate::params::primitive_brain::{
    PrimitiveBrainComposeParams, PrimitiveBrainConserveParams, PrimitiveBrainDecomposeParams,
    PrimitiveBrainDistanceParams, PrimitiveBrainQueryParams,
};
use crate::tools::lex_primitiva::find_primitive;
use nexcore_vigilance::lex_primitiva::{GroundingTier, LexPrimitiva, PrimitiveComposition};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::BTreeSet;

// ============================================================================
// Tool Implementations
// ============================================================================

/// Decompose a concept into T1 primitives and optionally persist as brain artifact.
pub fn decompose(params: PrimitiveBrainDecomposeParams) -> Result<CallToolResult, McpError> {
    let resolved: Vec<LexPrimitiva> = params
        .primitives
        .iter()
        .filter_map(|name| find_primitive(name))
        .collect();

    if resolved.is_empty() {
        let json = json!({
            "success": false,
            "error": "No valid T1 primitives found in input",
            "hint": "Use names like 'Causality', 'Boundary', 'State', 'Comparison', etc.",
        });
        return Ok(CallToolResult::error(vec![Content::text(json.to_string())]));
    }

    let unresolved: Vec<&String> = params
        .primitives
        .iter()
        .filter(|name| find_primitive(name).is_none())
        .collect();

    // Deduplicate via BTreeSet for consistent ordering
    let unique: BTreeSet<LexPrimitiva> = resolved.iter().copied().collect();
    let unique_vec: Vec<LexPrimitiva> = unique.iter().copied().collect();

    let dominant = params
        .dominant
        .as_ref()
        .and_then(|d| find_primitive(d))
        .or_else(|| unique_vec.first().copied());
    let confidence = params.confidence.unwrap_or(0.8);

    let mut comp = PrimitiveComposition::new(unique_vec);
    comp.dominant = dominant;
    comp.confidence = confidence;
    let tier = GroundingTier::classify(&comp);

    // Build the decomposition artifact content
    let primitives_json: Vec<serde_json::Value> = unique
        .iter()
        .map(|p| json!({ "name": format!("{p:?}"), "symbol": p.symbol() }))
        .collect();

    let artifact_content = json!({
        "concept": params.concept,
        "primitives": primitives_json,
        "dominant": dominant.map(|d| json!({ "name": format!("{d:?}"), "symbol": d.symbol() })),
        "confidence": confidence,
        "tier": format!("{tier:?}"),
        "primitive_count": unique.len(),
        "domain": params.domain,
        "conservation": conservation_check(&unique),
    });

    let mut result = json!({
        "success": true,
        "decomposition": artifact_content,
    });

    if !unresolved.is_empty() {
        if let serde_json::Value::Object(ref mut map) = result {
            map.insert(
                "warnings".to_string(),
                json!({
                    "unresolved_primitives": unresolved,
                    "message": "Some inputs were not valid T1 primitives and were skipped",
                }),
            );
        }
    }

    if params.persist {
        let artifact_name = format!("decomposition:{}", params.concept);
        if let serde_json::Value::Object(ref mut map) = result {
            map.insert(
                "artifact".to_string(),
                json!({
                    "name": artifact_name,
                    "action": "Use brain_artifact_save to persist this decomposition",
                    "content": serde_json::to_string_pretty(&artifact_content)
                        .unwrap_or_else(|_| artifact_content.to_string()),
                }),
            );
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Query brain state by T1 primitive.
pub fn query(params: PrimitiveBrainQueryParams) -> Result<CallToolResult, McpError> {
    let primitive = find_primitive(&params.primitive);
    let Some(prim) = primitive else {
        let json = json!({
            "success": false,
            "error": format!("Unknown primitive: {}", params.primitive),
            "available": LexPrimitiva::all().iter().map(|p| format!("{p:?}")).collect::<Vec<_>>(),
        });
        return Ok(CallToolResult::error(vec![Content::text(json.to_string())]));
    };

    let scope = params.scope.as_deref().unwrap_or("all");
    let limit = params.limit.unwrap_or(10);

    let json = json!({
        "success": true,
        "primitive": {
            "name": format!("{prim:?}"),
            "symbol": prim.symbol(),
            "description": prim.description(),
        },
        "scope": scope,
        "limit": limit,
        "query_templates": {
            "beliefs": format!(
                "Query brain for beliefs with t1_grounding = '{}'. Use implicit_patterns_by_grounding with grounding='{}'.",
                prim, prim
            ),
            "patterns": format!(
                "Query brain for patterns with t1_grounding = '{}'. Use implicit_patterns_by_grounding with grounding='{}'.",
                prim, prim
            ),
            "artifacts": format!(
                "Query brain artifacts matching 'decomposition:*' and filter for primitive '{}' in content.",
                prim
            ),
        },
        "cross_domain": cross_domain_map(&prim),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Compute symmetric difference distance |A△B| between two primitive sets.
pub fn distance(params: PrimitiveBrainDistanceParams) -> Result<CallToolResult, McpError> {
    let set_a: BTreeSet<LexPrimitiva> = params.a.iter().filter_map(|n| find_primitive(n)).collect();
    let set_b: BTreeSet<LexPrimitiva> = params.b.iter().filter_map(|n| find_primitive(n)).collect();

    if set_a.is_empty() || set_b.is_empty() {
        let json = json!({
            "success": false,
            "error": "Both sets must contain at least one valid T1 primitive",
        });
        return Ok(CallToolResult::error(vec![Content::text(json.to_string())]));
    }

    // Symmetric difference: elements in A or B but not both
    let sym_diff: BTreeSet<&LexPrimitiva> = set_a.symmetric_difference(&set_b).collect();
    let intersection: BTreeSet<&LexPrimitiva> = set_a.intersection(&set_b).collect();
    let union: BTreeSet<&LexPrimitiva> = set_a.union(&set_b).collect();

    let distance = sym_diff.len();
    let jaccard = if union.is_empty() {
        0.0
    } else {
        intersection.len() as f64 / union.len() as f64
    };

    let json = json!({
        "success": true,
        "distance": distance,
        "jaccard_similarity": jaccard,
        "set_a": set_a.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>(),
        "set_b": set_b.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>(),
        "symmetric_difference": sym_diff.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>(),
        "intersection": intersection.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>(),
        "verdict": if distance == 0 {
            "identical — same primitive composition"
        } else if distance <= 2 {
            "near — high transfer potential"
        } else if distance <= 5 {
            "moderate — partial structural overlap"
        } else {
            "far — distinct primitive structures"
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Check the conservation law ∃ = ∂(×(ς, ∅)) against a primitive composition.
///
/// The four conservation terms:
/// - ∃ (Existence): the composition exists as a coherent concept
/// - ∂ (Boundary): the composition has a defined boundary
/// - ς (State): the composition carries mutable state
/// - ∅ (Void): the composition has a ground/null state
///
/// If any term is missing, identity collapses — the composition is incomplete.
pub fn conserve(params: PrimitiveBrainConserveParams) -> Result<CallToolResult, McpError> {
    let resolved: BTreeSet<LexPrimitiva> = params
        .primitives
        .iter()
        .filter_map(|n| find_primitive(n))
        .collect();

    if resolved.is_empty() {
        let json = json!({
            "success": false,
            "error": "No valid T1 primitives found",
        });
        return Ok(CallToolResult::error(vec![Content::text(json.to_string())]));
    }

    let check = conservation_check(&resolved);
    let concept = params.concept.unwrap_or_else(|| "unnamed".to_string());

    let json = json!({
        "success": true,
        "concept": concept,
        "conservation": check,
        "primitives": resolved.iter().map(|p| json!({ "name": format!("{p:?}"), "symbol": p.symbol() })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Compose primitives into a structure with tier classification.
pub fn compose(params: PrimitiveBrainComposeParams) -> Result<CallToolResult, McpError> {
    let unique: BTreeSet<LexPrimitiva> = params
        .primitives
        .iter()
        .filter_map(|n| find_primitive(n))
        .collect();

    if unique.is_empty() {
        let json = json!({
            "success": false,
            "error": "No valid T1 primitives found",
        });
        return Ok(CallToolResult::error(vec![Content::text(json.to_string())]));
    }

    let unique_vec: Vec<LexPrimitiva> = unique.iter().copied().collect();
    let comp = PrimitiveComposition::new(unique_vec);
    let tier = GroundingTier::classify(&comp);

    let name = params.name.unwrap_or_else(|| "anonymous".to_string());
    let primitives_json: Vec<serde_json::Value> = unique
        .iter()
        .map(|p| json!({ "name": format!("{p:?}"), "symbol": p.symbol() }))
        .collect();

    let result = json!({
        "success": true,
        "composition": {
            "name": name,
            "primitives": primitives_json,
            "primitive_count": unique.len(),
            "tier": format!("{tier:?}"),
            "dominant": comp.dominant.map(|d| json!({ "name": format!("{d:?}"), "symbol": d.symbol() })),
            "conservation": conservation_check(&unique),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check conservation law terms against a primitive set.
fn conservation_check(primitives: &BTreeSet<LexPrimitiva>) -> serde_json::Value {
    let has_existence = primitives.contains(&LexPrimitiva::Existence);
    let has_boundary = primitives.contains(&LexPrimitiva::Boundary);
    let has_state = primitives.contains(&LexPrimitiva::State);
    let has_void = primitives.contains(&LexPrimitiva::Void);

    let terms_present = [has_existence, has_boundary, has_state, has_void]
        .iter()
        .filter(|&&b| b)
        .count();

    let conserved = terms_present == 4;
    let partial = terms_present > 0 && terms_present < 4;

    json!({
        "law": "∃ = ∂(×(ς, ∅))",
        "conserved": conserved,
        "terms": {
            "∃ (Existence)": has_existence,
            "∂ (Boundary)": has_boundary,
            "ς (State)": has_state,
            "∅ (Void)": has_void,
        },
        "terms_present": terms_present,
        "terms_total": 4,
        "verdict": if conserved {
            "CONSERVED — all four terms present, identity holds"
        } else if partial {
            "PARTIAL — identity at risk, missing terms should be investigated"
        } else {
            "ABSENT — no conservation terms present"
        },
        "missing": {
            "existence": !has_existence,
            "boundary": !has_boundary,
            "state": !has_state,
            "void": !has_void,
        },
    })
}

/// Map a primitive to its cross-domain analogs.
/// (source: primitives.ipynb Part II Dictionary + transfer.rs)
fn cross_domain_map(prim: &LexPrimitiva) -> serde_json::Value {
    match prim {
        LexPrimitiva::Causality => json!({
            "pv": "adverse event → drug reaction causality chain",
            "biology": "stimulus → response pathway",
            "physics": "force → acceleration (F=ma)",
            "software": "function call → return value",
        }),
        LexPrimitiva::Boundary => json!({
            "pv": "safety margin, therapeutic window",
            "biology": "cell membrane, blood-brain barrier",
            "physics": "event horizon, phase boundary",
            "software": "API boundary, module interface",
        }),
        LexPrimitiva::State => json!({
            "pv": "patient condition, case status",
            "biology": "homeostasis, cellular state",
            "physics": "quantum state, thermodynamic state",
            "software": "variable, struct field",
        }),
        LexPrimitiva::Comparison => json!({
            "pv": "signal detection (observed vs expected)",
            "biology": "immune recognition (self vs non-self)",
            "physics": "measurement, interferometry",
            "software": "equality test, diff, assertion",
        }),
        LexPrimitiva::Sequence => json!({
            "pv": "case processing pipeline, reporting timeline",
            "biology": "DNA sequence, metabolic pathway",
            "physics": "time series, causal ordering",
            "software": "iterator, pipeline, workflow",
        }),
        LexPrimitiva::Persistence => json!({
            "pv": "case record retention, safety database",
            "biology": "DNA, epigenetic memory",
            "physics": "conservation law, mass",
            "software": "database, file, artifact",
        }),
        LexPrimitiva::Existence => json!({
            "pv": "case validity, signal confirmation",
            "biology": "cell viability, organism alive/dead",
            "physics": "particle existence, observation",
            "software": "Option::Some, non-null reference",
        }),
        LexPrimitiva::Void => json!({
            "pv": "no adverse event, clean safety profile",
            "biology": "apoptosis, extinction",
            "physics": "vacuum state, zero-point energy",
            "software": "Option::None, (), Default::default()",
        }),
        LexPrimitiva::Mapping => json!({
            "pv": "MedDRA coding, drug-event pairing",
            "biology": "enzyme catalysis, gene expression",
            "physics": "coordinate transformation",
            "software": "From/Into, map(), type conversion",
        }),
        LexPrimitiva::Quantity => json!({
            "pv": "case count, PRR score, reporting rate",
            "biology": "cell count, concentration, dosage",
            "physics": "mass, charge, energy value",
            "software": "usize, f64, counter",
        }),
        LexPrimitiva::Frequency => json!({
            "pv": "reporting rate, signal frequency",
            "biology": "heart rate, allele frequency",
            "physics": "wave frequency, oscillation period",
            "software": "event rate, polling interval",
        }),
        LexPrimitiva::Recursion => json!({
            "pv": "follow-up case referencing initial report",
            "biology": "self-replicating DNA, autoimmune response",
            "physics": "fractal structure, self-similarity",
            "software": "recursive function, Arc<Mutex<T>>",
        }),
        LexPrimitiva::Location => json!({
            "pv": "reporting country, study site",
            "biology": "organ, tissue location, receptor site",
            "physics": "position vector, coordinates",
            "software": "memory address, file path, URL",
        }),
        LexPrimitiva::Irreversibility => json!({
            "pv": "death, permanent disability (ICH E2A serious criteria)",
            "biology": "cell death, extinction event",
            "physics": "entropy increase, arrow of time",
            "software": "destructive operation, DROP TABLE",
        }),
        LexPrimitiva::Sum => json!({
            "pv": "causality category (certain|probable|possible|unlikely)",
            "biology": "phenotype expression (one of N variants)",
            "physics": "quantum superposition collapse",
            "software": "enum, match, tagged union",
        }),
        _ => json!({}),
    }
}
