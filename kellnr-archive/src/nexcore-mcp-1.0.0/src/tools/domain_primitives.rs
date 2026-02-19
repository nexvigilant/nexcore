//! Domain primitive extraction MCP tools — tier taxonomy, dependency graphs, transfer confidence.
//!
//! # T1 Grounding
//! - σ (sequence): dependency chains between primitives
//! - κ (comparison): tier classification, transfer confidence ranking
//! - ρ (recursion): decomposition trees
//! - ∃ (existence): registry listing

use nexcore_domain_primitives::TaxonomyRegistry;
use nexcore_domain_primitives::taxonomy::Tier;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    DomainPrimitivesBottlenecksParams, DomainPrimitivesCompareParams,
    DomainPrimitivesCriticalPathsParams, DomainPrimitivesDecomposeParams,
    DomainPrimitivesListParams, DomainPrimitivesLoadParams, DomainPrimitivesRegistryParams,
    DomainPrimitivesSaveParams, DomainPrimitivesTopoSortParams,
    DomainPrimitivesTransferMatrixParams, DomainPrimitivesTransferParams,
};

/// Resolve taxonomy name to a built-in taxonomy from the registry.
fn resolve_taxonomy(name: Option<&str>) -> nexcore_domain_primitives::DomainTaxonomy {
    let reg = TaxonomyRegistry::new();
    let key = name.unwrap_or("golden-dome");
    reg.get(key).cloned().unwrap_or_else(|| {
        reg.get("golden-dome").cloned().unwrap_or_else(|| {
            nexcore_domain_primitives::DomainTaxonomy::new("empty", "no taxonomy found")
        })
    })
}

/// List all primitives in a taxonomy, optionally filtered by tier.
pub fn domain_primitives_list(
    params: DomainPrimitivesListParams,
) -> Result<CallToolResult, McpError> {
    let tax = resolve_taxonomy(params.taxonomy.as_deref());
    let tier_filter = params.tier.as_deref().and_then(parse_tier);

    let primitives: Vec<serde_json::Value> = tax
        .primitives
        .iter()
        .filter(|p| tier_filter.is_none_or(|t| p.tier == t))
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "definition": p.definition,
                "tier": p.tier.label(),
                "dependencies": p.dependencies,
                "domain_examples": p.domain_examples,
            })
        })
        .collect();

    let counts = tax.tier_counts();
    let response = serde_json::json!({
        "taxonomy": tax.name,
        "description": tax.description,
        "total": primitives.len(),
        "tier_counts": {
            "T1": counts.get(&Tier::T1).copied().unwrap_or(0),
            "T2-P": counts.get(&Tier::T2P).copied().unwrap_or(0),
            "T2-C": counts.get(&Tier::T2C).copied().unwrap_or(0),
            "T3": counts.get(&Tier::T3).copied().unwrap_or(0),
        },
        "irreducible_atoms": tax.irreducible_atoms().len(),
        "primitives": primitives,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Compute cross-domain transfer confidence for a primitive.
pub fn domain_primitives_transfer(
    params: DomainPrimitivesTransferParams,
) -> Result<CallToolResult, McpError> {
    let tax = resolve_taxonomy(params.taxonomy.as_deref());

    let transfers: Vec<serde_json::Value> = tax
        .transfers
        .iter()
        .filter(|t| {
            let name_match = params.primitive_name.is_none()
                || params.primitive_name.as_deref() == Some(t.primitive_name.as_str());
            let domain_match = params.target_domain.is_none()
                || params.target_domain.as_deref() == Some(t.target_domain.as_str());
            name_match && domain_match
        })
        .map(|t| {
            serde_json::json!({
                "primitive": t.primitive_name,
                "target_domain": t.target_domain,
                "confidence": (t.confidence() * 1000.0).round() / 1000.0,
                "structural": t.score.structural,
                "functional": t.score.functional,
                "contextual": t.score.contextual,
                "limiting_factor": t.score.limiting_factor(),
                "limiting_description": t.limiting_description,
            })
        })
        .collect();

    let response = serde_json::json!({
        "taxonomy": tax.name,
        "count": transfers.len(),
        "transfers": transfers,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Decompose a primitive into its T1 foundation tree.
pub fn domain_primitives_decompose(
    params: DomainPrimitivesDecomposeParams,
) -> Result<CallToolResult, McpError> {
    let tax = resolve_taxonomy(params.taxonomy.as_deref());

    let node = tax.decompose(&params.primitive_name);
    let response = match node {
        Some(n) => {
            let leaves = n.leaves();
            serde_json::json!({
                "taxonomy": tax.name,
                "primitive": params.primitive_name,
                "depth": n.depth(),
                "foundation_atoms": leaves,
                "foundation_count": leaves.len(),
                "tree": serialize_node(&n),
            })
        }
        None => {
            serde_json::json!({
                "error": format!("Primitive '{}' not found in {} taxonomy", params.primitive_name, tax.name),
                "available": tax.primitives.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            })
        }
    };

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Bottleneck analysis: primitives ranked by transitive fan-out.
pub fn domain_primitives_bottlenecks(
    params: DomainPrimitivesBottlenecksParams,
) -> Result<CallToolResult, McpError> {
    let tax = resolve_taxonomy(params.taxonomy.as_deref());
    let limit = params.limit.unwrap_or(10);

    let bn = nexcore_domain_primitives::bottlenecks(&tax);
    let results: Vec<serde_json::Value> = bn
        .iter()
        .take(limit)
        .map(|b| {
            serde_json::json!({
                "name": b.name,
                "tier": b.tier.label(),
                "fan_out": b.fan_out,
                "reach_pct": (b.reach_pct * 10.0).round() / 10.0,
            })
        })
        .collect();

    let response = serde_json::json!({
        "taxonomy": tax.name,
        "total_primitives": tax.primitives.len(),
        "showing": results.len(),
        "bottlenecks": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Compare two domain taxonomies to find shared and unique primitives.
pub fn domain_primitives_compare(
    params: DomainPrimitivesCompareParams,
) -> Result<CallToolResult, McpError> {
    let tax_a = resolve_taxonomy(params.taxonomy_a.as_deref());
    let tax_b = resolve_taxonomy(Some(
        params.taxonomy_b.as_deref().unwrap_or("pharmacovigilance"),
    ));

    let cmp = nexcore_domain_primitives::compare(&tax_a, &tax_b);
    let aligned = nexcore_domain_primitives::compare::tier_aligned(&cmp);

    let shared: Vec<serde_json::Value> = cmp
        .shared
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "tier_a": s.tier_a.label(),
                "tier_b": s.tier_b.label(),
                "tier_match": s.tier_match,
            })
        })
        .collect();

    let response = serde_json::json!({
        "taxonomy_a": cmp.taxonomy_a,
        "taxonomy_b": cmp.taxonomy_b,
        "jaccard_similarity": (cmp.jaccard * 1000.0).round() / 1000.0,
        "shared_count": cmp.shared.len(),
        "tier_aligned_count": aligned.len(),
        "unique_a_count": cmp.unique_a.len(),
        "unique_b_count": cmp.unique_b.len(),
        "shared": shared,
        "unique_a": cmp.unique_a,
        "unique_b": cmp.unique_b,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Topological ordering of primitives (dependencies before dependents).
pub fn domain_primitives_topo_sort(
    params: DomainPrimitivesTopoSortParams,
) -> Result<CallToolResult, McpError> {
    let tax = resolve_taxonomy(params.taxonomy.as_deref());

    let response = match nexcore_domain_primitives::topological_sort(&tax) {
        Ok(sorted) => {
            serde_json::json!({
                "taxonomy": tax.name,
                "total": sorted.len(),
                "order": sorted,
            })
        }
        Err(e) => {
            serde_json::json!({
                "error": format!("Cycle detected in {}: {}", tax.name, e),
            })
        }
    };

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Critical paths from T1 roots to T3 leaves (longest dependency chains).
pub fn domain_primitives_critical_paths(
    params: DomainPrimitivesCriticalPathsParams,
) -> Result<CallToolResult, McpError> {
    let tax = resolve_taxonomy(params.taxonomy.as_deref());
    let limit = params.limit.unwrap_or(5);

    let paths = nexcore_domain_primitives::critical_paths(&tax);
    let results: Vec<serde_json::Value> = paths
        .iter()
        .take(limit)
        .enumerate()
        .map(|(i, path)| {
            serde_json::json!({
                "rank": i + 1,
                "length": path.len(),
                "path": path,
            })
        })
        .collect();

    let response = serde_json::json!({
        "taxonomy": tax.name,
        "total_paths": paths.len(),
        "showing": results.len(),
        "longest_length": paths.first().map(|p| p.len()).unwrap_or(0),
        "critical_paths": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// List all registered domain taxonomies.
pub fn domain_primitives_registry(
    _params: DomainPrimitivesRegistryParams,
) -> Result<CallToolResult, McpError> {
    let reg = TaxonomyRegistry::new();

    let taxonomies: Vec<serde_json::Value> = reg
        .list()
        .iter()
        .filter_map(|&name| {
            reg.get(name).map(|tax| {
                let counts = tax.tier_counts();
                serde_json::json!({
                    "name": tax.name,
                    "description": tax.description,
                    "primitives": tax.primitives.len(),
                    "transfers": tax.transfers.len(),
                    "tier_counts": {
                        "T1": counts.get(&Tier::T1).copied().unwrap_or(0),
                        "T2-P": counts.get(&Tier::T2P).copied().unwrap_or(0),
                        "T2-C": counts.get(&Tier::T2C).copied().unwrap_or(0),
                        "T3": counts.get(&Tier::T3).copied().unwrap_or(0),
                    },
                    "irreducible_atoms": tax.irreducible_atoms().len(),
                })
            })
        })
        .collect();

    let response = serde_json::json!({
        "registry_size": reg.len(),
        "total_primitives": reg.total_primitives(),
        "taxonomies": taxonomies,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Save a taxonomy to a JSON file.
pub fn domain_primitives_save(
    params: DomainPrimitivesSaveParams,
) -> Result<CallToolResult, McpError> {
    let reg = TaxonomyRegistry::new();
    let path = std::path::Path::new(&params.path);

    match reg.save_json(&params.taxonomy, path) {
        Ok(()) => {
            let response = serde_json::json!({
                "status": "saved",
                "taxonomy": params.taxonomy,
                "path": params.path,
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
        Err(e) => {
            let response = serde_json::json!({
                "error": format!("Failed to save: {}", e),
                "taxonomy": params.taxonomy,
                "available": reg.list(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
    }
}

/// Load a taxonomy from a JSON file.
pub fn domain_primitives_load(
    params: DomainPrimitivesLoadParams,
) -> Result<CallToolResult, McpError> {
    let mut reg = TaxonomyRegistry::empty();
    let path = std::path::Path::new(&params.path);

    match reg.load_json(path) {
        Ok(name) => {
            let tax = reg.get(&name);
            let prim_count = tax.map(|t| t.primitives.len()).unwrap_or(0);
            let transfer_count = tax.map(|t| t.transfers.len()).unwrap_or(0);
            let response = serde_json::json!({
                "status": "loaded",
                "taxonomy": name,
                "primitives": prim_count,
                "transfers": transfer_count,
                "path": params.path,
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
        Err(e) => {
            let response = serde_json::json!({
                "error": format!("Failed to load: {}", e),
                "path": params.path,
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
    }
}

/// Compute the cross-domain transfer matrix between all registered taxonomies.
pub fn domain_primitives_transfer_matrix(
    params: DomainPrimitivesTransferMatrixParams,
) -> Result<CallToolResult, McpError> {
    let reg = TaxonomyRegistry::new();
    let matrix = nexcore_domain_primitives::transfer_matrix::compute(&reg);
    let limit = params.limit.unwrap_or(10);

    let cells: Vec<serde_json::Value> = matrix
        .cells
        .iter()
        .map(|c| {
            serde_json::json!({
                "from": c.from,
                "to": c.to,
                "avg_confidence": (c.avg_confidence * 1000.0).round() / 1000.0,
                "transfer_count": c.transfer_count,
                "shared_primitives": c.shared_count,
                "tier_aligned": c.tier_aligned_count,
                "strongest_primitive": c.strongest_primitive,
                "strongest_confidence": (c.strongest_confidence * 1000.0).round() / 1000.0,
            })
        })
        .collect();

    let top = nexcore_domain_primitives::transfer_matrix::top_bridges(&matrix, limit);
    let bridges: Vec<serde_json::Value> = top
        .iter()
        .map(|b| {
            serde_json::json!({
                "name": b.name,
                "tier": b.tier.label(),
                "appears_in": b.appears_in,
                "avg_confidence": (b.avg_confidence * 1000.0).round() / 1000.0,
                "tier_consistent": b.tier_consistent,
            })
        })
        .collect();

    let response = serde_json::json!({
        "domains": matrix.domains,
        "cell_count": matrix.cells.len(),
        "bridge_count": matrix.bridges.len(),
        "cells": cells,
        "top_bridges": bridges,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

fn serialize_node(node: &nexcore_domain_primitives::DecompositionNode) -> serde_json::Value {
    serde_json::json!({
        "name": node.name,
        "tier": node.tier.label(),
        "children": node.children.iter().map(serialize_node).collect::<Vec<_>>(),
    })
}

fn parse_tier(s: &str) -> Option<Tier> {
    match s.to_uppercase().as_str() {
        "T1" => Some(Tier::T1),
        "T2P" | "T2-P" => Some(Tier::T2P),
        "T2C" | "T2-C" => Some(Tier::T2C),
        "T3" => Some(Tier::T3),
        _ => None,
    }
}
