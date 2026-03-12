//! Ouverse Chain MCP tool
//!
//! The ouverse is the third direction of the conservation law ∃ = ∂(×(ς, ∅)),
//! pivoting on ∃ forward into what it enables:
//!
//!   inverse:   →⁻¹(∃) = {∂, ς, ∅}     — what made this? (Five Whys)
//!   direct:    →({∂, ς, ∅}) = ∃        — how to build this? (Engineering)
//!   ouverse:   →(∃ → π(∃) at ∂_next) → ∃_next  — what does this make possible? (Anti-Why)
//!
//! Source: primitives.ipynb Cell 123, entries 21-23.
//! Equation: ouverse(∃) = σ[∃₁ → π₁ → ∃₂ → π₂ → ... → ∃ₙ]
//!
//! Each link: existence is caught (∃), persisted (π), then enters the next
//! equation as input, producing the next existence. The ouverse is this chain
//! read forward. The inverse is this chain read backward.
//!
//! Failure mode #6: Dead ouverse — ∃ → () — existence that produces nothing
//! downstream. Composability lost.

use crate::params::ouverse::PrimitiveBrainOuverseParams;
use crate::tools::lex_primitiva::find_primitive;
use nexcore_vigilance::lex_primitiva::LexPrimitiva;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::BTreeSet;

/// Compute an ouverse chain with per-link conservation, dead ouverse detection,
/// and primitive coverage metrics.
pub fn ouverse(params: PrimitiveBrainOuverseParams) -> Result<CallToolResult, McpError> {
    if params.chain.is_empty() {
        let json = json!({
            "success": false,
            "error": "Ouverse chain must contain at least one link",
            "hint": "Each link needs: existence (what exists), persistence (how it endures), primitives (T1 names)",
        });
        return Ok(CallToolResult::error(vec![Content::text(json.to_string())]));
    }

    let all_primitives: BTreeSet<LexPrimitiva> = LexPrimitiva::all().iter().copied().collect();
    let mut chain_primitives_union: BTreeSet<LexPrimitiva> = BTreeSet::new();
    let mut links_json = Vec::new();
    let mut dead_links: Vec<usize> = Vec::new();

    for (i, link) in params.chain.iter().enumerate() {
        let level = i + 1;
        let resolved: BTreeSet<LexPrimitiva> = link
            .primitives
            .iter()
            .filter_map(|name| find_primitive(name))
            .collect();

        let unresolved: Vec<&String> = link
            .primitives
            .iter()
            .filter(|name| find_primitive(name).is_none())
            .collect();

        // Dead ouverse test: a link with no primitives produces nothing
        let is_dead = resolved.is_empty() && !link.primitives.is_empty();
        if is_dead {
            dead_links.push(level);
        }

        // Conservation check at this link
        let has_existence = resolved.contains(&LexPrimitiva::Existence);
        let has_boundary = resolved.contains(&LexPrimitiva::Boundary);
        let has_state = resolved.contains(&LexPrimitiva::State);
        let has_void = resolved.contains(&LexPrimitiva::Void);
        let has_causality = resolved.contains(&LexPrimitiva::Causality);
        let has_persistence = resolved.contains(&LexPrimitiva::Persistence);

        let conservation_terms = [has_existence, has_boundary, has_state, has_void]
            .iter()
            .filter(|&&b| b)
            .count();

        // Track union across all links
        chain_primitives_union.extend(resolved.iter());

        let primitives_json: Vec<serde_json::Value> = resolved
            .iter()
            .map(|p| json!({ "name": format!("{p:?}"), "symbol": p.symbol() }))
            .collect();

        let mut link_json = json!({
            "level": level,
            "label": format!("∃{}", subscript(level)),
            "existence": link.existence,
            "persistence": link.persistence,
            "persistence_label": format!("π{}", subscript(level)),
            "primitives": primitives_json,
            "primitive_count": resolved.len(),
            "conservation": {
                "∃": has_existence,
                "∂": has_boundary,
                "ς": has_state,
                "∅": has_void,
                "terms_present": conservation_terms,
                "conserved": conservation_terms == 4,
            },
            "has_causality": has_causality,
            "has_persistence": has_persistence,
        });

        if !unresolved.is_empty() {
            if let serde_json::Value::Object(ref mut map) = link_json {
                map.insert("unresolved".to_string(), json!(unresolved));
            }
        }

        if is_dead {
            if let serde_json::Value::Object(ref mut map) = link_json {
                map.insert(
                    "dead_ouverse".to_string(),
                    json!("FAILURE MODE #6: ∃ → () — no valid primitives resolved"),
                );
            }
        }

        links_json.push(link_json);
    }

    // Chain-level metrics
    let chain_length = params.chain.len();
    let total_unique_primitives = chain_primitives_union.len();

    // Coverage: how many of the 15 operational primitives does the chain use?
    // Exclude Product (×) from count — it's the operator, not a coordinate
    let operational_count = all_primitives.len();
    let coverage = total_unique_primitives as f64 / operational_count as f64;

    let missing: Vec<serde_json::Value> = all_primitives
        .difference(&chain_primitives_union)
        .map(|p| json!({ "name": format!("{p:?}"), "symbol": p.symbol() }))
        .collect();

    // Root strength analysis
    let root = &params.chain[0];
    let root_primitives: BTreeSet<LexPrimitiva> = root
        .primitives
        .iter()
        .filter_map(|name| find_primitive(name))
        .collect();

    // Ouverse-specific primitive check: {→, ∃, π, ∂} are the ouverse's own composition
    let ouverse_primitives_present = [
        root_primitives.contains(&LexPrimitiva::Causality),
        root_primitives.contains(&LexPrimitiva::Existence),
        root_primitives.contains(&LexPrimitiva::Persistence),
        root_primitives.contains(&LexPrimitiva::Boundary),
    ]
    .iter()
    .filter(|&&b| b)
    .count();

    // Build the equation string
    let equation_parts: Vec<String> = params
        .chain
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let level = i + 1;
            if level == chain_length {
                format!("∃{}", subscript(level))
            } else {
                format!("∃{} → π{}", subscript(level), subscript(level))
            }
        })
        .collect();
    let equation = format!("ouverse = σ[{}]", equation_parts.join(" → "));

    let mut result = json!({
        "success": true,
        "name": params.name,
        "direction": "ouverse — forward enablement (opposite of inverse/Five Whys)",
        "equation": equation,
        "chain": links_json,
        "metrics": {
            "chain_length": chain_length,
            "unique_primitives": total_unique_primitives,
            "operational_total": operational_count,
            "coverage": format!("{:.0}%", coverage * 100.0),
            "coverage_raw": coverage,
            "missing_primitives": missing,
        },
        "root_strength": {
            "level": "∃₁",
            "existence": root.existence,
            "persistence": root.persistence,
            "ouverse_primitives": {
                "→ (Causality)": root_primitives.contains(&LexPrimitiva::Causality),
                "∃ (Existence)": root_primitives.contains(&LexPrimitiva::Existence),
                "π (Persistence)": root_primitives.contains(&LexPrimitiva::Persistence),
                "∂ (Boundary)": root_primitives.contains(&LexPrimitiva::Boundary),
            },
            "ouverse_primitives_present": ouverse_primitives_present,
            "dead_ouverse_test": "If ∃₁ is absent, entire chain collapses to ()",
        },
        "verdict": if !dead_links.is_empty() {
            format!("DEAD OUVERSE at link(s) {:?} — chain broken, ∃ → ()", dead_links)
        } else if coverage >= 0.80 {
            format!("STRONG — {chain_length} links, {:.0}% primitive coverage, no dead links", coverage * 100.0)
        } else if coverage >= 0.50 {
            format!("MODERATE — {chain_length} links, {:.0}% coverage, consider expanding primitive diversity", coverage * 100.0)
        } else {
            format!("THIN — {chain_length} links, {:.0}% coverage, chain uses few primitives", coverage * 100.0)
        },
    });

    if params.persist {
        let artifact_name = format!("ouverse:{}", params.name);
        let content_snapshot =
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
        if let serde_json::Value::Object(ref mut map) = result {
            map.insert(
                "artifact".to_string(),
                json!({
                    "name": artifact_name,
                    "action": "Use brain_artifact_save to persist this ouverse chain",
                    "content": content_snapshot,
                }),
            );
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Unicode subscript digits for ∃₁, π₂, etc.
fn subscript(n: usize) -> String {
    const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
    if n < 10 {
        SUBSCRIPTS[n].to_string()
    } else {
        n.to_string()
            .chars()
            .map(|c| {
                let digit = c.to_digit(10).unwrap_or(0) as usize;
                SUBSCRIPTS[digit]
            })
            .collect()
    }
}
