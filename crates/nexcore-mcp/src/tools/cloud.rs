//! Cloud Intelligence — 17 MCP tools activating nexcloud's 35-type taxonomy.
//!
//! Phase 1: Core Query + Analysis (8 tools)
//! Phase 2: Infrastructure Awareness (4 tools)
//! Phase 3: Cross-Domain Reasoning (5 tools)
//!
//! Tier: T2-C (μ+→+∂+N — mapping, causality, boundary, quantity)

use nexcloud::prelude::*;
use nexcore_lex_primitiva::molecular_weight::MolecularFormula;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::tier::Tier;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};
use serde_json::json;
use std::collections::HashSet;

use crate::params;

// ============================================================================
// Internal: 35-type dispatch table
// ============================================================================

/// All 35 cloud type names grouped by tier.
const CLOUD_T1: &[&str] = &[
    "Identity",
    "Threshold",
    "FeedbackLoop",
    "Idempotency",
    "Immutability",
    "Convergence",
];

const CLOUD_T2P: &[&str] = &[
    "Compute",
    "Storage",
    "NetworkLink",
    "IsolationBoundary",
    "Permission",
    "ResourcePool",
    "Metering",
    "Replication",
    "Routing",
    "Lease",
    "Encryption",
    "Queue",
    "HealthCheck",
    "Elasticity",
];

const CLOUD_T2C: &[&str] = &[
    "VirtualMachine",
    "LoadBalancer",
    "AutoScaling",
    "Iam",
    "EventualConsistency",
    "Tenancy",
    "PayPerUse",
    "ReservedCapacity",
    "SpotPricing",
    "SecretsManagement",
];

const CLOUD_T3: &[&str] = &["Container", "Iaas", "Paas", "Saas", "Serverless"];

fn all_cloud_types() -> Vec<&'static str> {
    let mut v = Vec::with_capacity(35);
    v.extend_from_slice(CLOUD_T1);
    v.extend_from_slice(CLOUD_T2P);
    v.extend_from_slice(CLOUD_T2C);
    v.extend_from_slice(CLOUD_T3);
    v
}

/// Classify a tier from unique primitive count (mirrors Tier::classify logic).
fn tier_from_count(n: usize) -> Tier {
    match n {
        0..=1 => Tier::T1Universal,
        2..=3 => Tier::T2Primitive,
        4..=5 => Tier::T2Composite,
        _ => Tier::T3DomainSpecific,
    }
}

fn tier_label(name: &str) -> &'static str {
    if CLOUD_T1.contains(&name) {
        "T1"
    } else if CLOUD_T2P.contains(&name) {
        "T2-P"
    } else if CLOUD_T2C.contains(&name) {
        "T2-C"
    } else if CLOUD_T3.contains(&name) {
        "T3"
    } else {
        "unknown"
    }
}

/// Lookup composition for any of the 35 cloud types.
fn cloud_composition_for(type_name: &str) -> Option<PrimitiveComposition> {
    macro_rules! dispatch {
        ($($name:literal => $ty:ty),* $(,)?) => {
            match type_name {
                $($name => Some(<$ty as GroundsTo>::primitive_composition()),)*
                _ => None,
            }
        };
    }
    dispatch! {
        // T1
        "Identity" => Identity,
        "Threshold" => Threshold,
        "FeedbackLoop" => FeedbackLoop,
        "Idempotency" => Idempotency,
        "Immutability" => Immutability,
        "Convergence" => Convergence,
        // T2-P
        "Compute" => Compute,
        "Storage" => Storage,
        "NetworkLink" => NetworkLink,
        "IsolationBoundary" => IsolationBoundary,
        "Permission" => Permission,
        "ResourcePool" => ResourcePool,
        "Metering" => Metering,
        "Replication" => Replication,
        "Routing" => Routing,
        "Lease" => Lease,
        "Encryption" => Encryption,
        "Queue" => Queue,
        "HealthCheck" => HealthCheck,
        "Elasticity" => Elasticity,
        // T2-C
        "VirtualMachine" => VirtualMachine,
        "LoadBalancer" => LoadBalancer,
        "AutoScaling" => AutoScaling,
        "Iam" => Iam,
        "EventualConsistency" => EventualConsistency,
        "Tenancy" => Tenancy,
        "PayPerUse" => PayPerUse,
        "ReservedCapacity" => ReservedCapacity,
        "SpotPricing" => SpotPricing,
        "SecretsManagement" => SecretsManagement,
        // T3
        "Container" => Container,
        "Iaas" => Iaas,
        "Paas" => Paas,
        "Saas" => Saas,
        "Serverless" => Serverless,
    }
}

fn require_composition(type_name: &str) -> Result<PrimitiveComposition, McpError> {
    cloud_composition_for(type_name).ok_or_else(|| {
        McpError::new(
            ErrorCode(404),
            format!(
                "Unknown cloud type '{}'. Known: {}",
                type_name,
                all_cloud_types().join(", ")
            ),
            None,
        )
    })
}

fn parse_primitive(input: &str) -> Result<LexPrimitiva, McpError> {
    for p in LexPrimitiva::all() {
        if p.symbol() == input || p.name().eq_ignore_ascii_case(input) {
            return Ok(p);
        }
    }
    Err(McpError::new(
        ErrorCode(400),
        format!("Unknown primitive '{}'. Use name or symbol.", input),
        None,
    ))
}

fn parse_primitive_list(input: &[String]) -> Result<Vec<LexPrimitiva>, McpError> {
    input.iter().map(|s| parse_primitive(s.trim())).collect()
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

fn composition_json(comp: &PrimitiveComposition) -> serde_json::Value {
    json!({
        "primitives": comp.primitives.iter().map(|p| json!({
            "name": p.name(),
            "symbol": p.symbol(),
        })).collect::<Vec<_>>(),
        "dominant": comp.dominant.map(|d| json!({
            "name": d.name(),
            "symbol": d.symbol(),
        })),
        "confidence": round3(comp.confidence),
        "primitive_count": comp.primitives.len(),
    })
}

// ============================================================================
// Phase 1: Core Query + Analysis (8 tools)
// ============================================================================

/// cloud_primitive_composition — Type name → full primitive composition + transfers.
pub fn primitive_composition(
    p: params::CloudCompositionParams,
) -> Result<CallToolResult, McpError> {
    let comp = require_composition(&p.type_name)?;
    let transfers = transfers_for_type(&p.type_name);
    let tier = tier_label(&p.type_name);

    let state_mode = cloud_state_mode(&p.type_name);

    let result = json!({
        "type": p.type_name,
        "tier": tier,
        "composition": composition_json(&comp),
        "state_mode": state_mode,
        "transfers": transfers.iter().map(|t| json!({
            "domain": t.domain,
            "analog": t.analog,
            "confidence": t.confidence,
        })).collect::<Vec<_>>(),
        "transfer_count": transfers.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn cloud_state_mode(type_name: &str) -> Option<&'static str> {
    match type_name {
        "Lease" => Some("Modal"),
        "VirtualMachine" => Some("Mutable"),
        "Iam" => Some("Mutable"),
        "Tenancy" => Some("Modal"),
        "ReservedCapacity" => Some("Modal"),
        "SpotPricing" => Some("Mutable"),
        "SecretsManagement" => Some("Mutable"),
        "Container" => Some("Modal"),
        "Iaas" => Some("Mutable"),
        "Paas" => Some("Mutable"),
        "Saas" => Some("Mutable"),
        "Serverless" => Some("Modal"),
        _ => None,
    }
}

/// transfer_confidence — Type + domain → analog + confidence.
pub fn transfer_confidence(
    p: params::CloudTransferConfidenceParams,
) -> Result<CallToolResult, McpError> {
    let conf = nexcloud::transfer::transfer_confidence(&p.cloud_type, &p.domain);
    let mappings = transfers_for_type(&p.cloud_type);
    let mapping = mappings.iter().find(|m| m.domain == p.domain);

    match (conf, mapping) {
        (Some(c), Some(m)) => {
            let result = json!({
                "cloud_type": p.cloud_type,
                "domain": p.domain,
                "analog": m.analog,
                "confidence": round3(c),
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        _ => {
            let domains: Vec<&str> = mappings.iter().map(|m| m.domain).collect();
            Err(McpError::new(
                ErrorCode(404),
                format!(
                    "No transfer mapping for '{}' → '{}'. Available domains: {:?}",
                    p.cloud_type, p.domain, domains
                ),
                None,
            ))
        }
    }
}

/// cloud_tier_classify — Primitive names → tier classification.
pub fn tier_classify(p: params::CloudTierClassifyParams) -> Result<CallToolResult, McpError> {
    let primitives = parse_primitive_list(&p.primitives)?;
    if primitives.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "primitives list must not be empty",
            None,
        ));
    }

    let unique: HashSet<_> = primitives.iter().collect();
    let tier = tier_from_count(unique.len());

    let result = json!({
        "unique_primitives": unique.len(),
        "tier": format!("{tier:?}"),
        "primitives": unique.iter().map(|p| json!({
            "name": p.name(),
            "symbol": p.symbol(),
        })).collect::<Vec<_>>(),
        "interpretation": match tier {
            Tier::T1Universal => "Universal primitive (1 unique). Transfers everywhere.",
            Tier::T2Primitive => "Cross-domain primitive (2-3 unique). High transferability.",
            Tier::T2Composite => "Composite type (4-5 unique). Moderate transferability.",
            Tier::T3DomainSpecific => "Domain-specific (6+ unique). Low transferability.",
            _ => "Unknown tier.",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_compare_types — Two types → overlap, unique-to-each, Jaccard.
pub fn compare_types(p: params::CloudCompareTypesParams) -> Result<CallToolResult, McpError> {
    let comp_a = require_composition(&p.type_a)?;
    let comp_b = require_composition(&p.type_b)?;

    let set_a: HashSet<_> = comp_a.primitives.iter().collect();
    let set_b: HashSet<_> = comp_b.primitives.iter().collect();

    let shared: Vec<_> = set_a.intersection(&set_b).map(|p| p.name()).collect();
    let only_a: Vec<_> = set_a.difference(&set_b).map(|p| p.name()).collect();
    let only_b: Vec<_> = set_b.difference(&set_a).map(|p| p.name()).collect();

    let union_count = set_a.union(&set_b).count();
    let jaccard = if union_count == 0 {
        0.0
    } else {
        shared.len() as f64 / union_count as f64
    };

    let result = json!({
        "type_a": {
            "name": p.type_a,
            "tier": tier_label(&p.type_a),
            "composition": composition_json(&comp_a),
        },
        "type_b": {
            "name": p.type_b,
            "tier": tier_label(&p.type_b),
            "composition": composition_json(&comp_b),
        },
        "comparison": {
            "shared_primitives": shared,
            "only_in_a": only_a,
            "only_in_b": only_b,
            "jaccard_similarity": round3(jaccard),
            "shared_count": shared.len(),
            "union_count": union_count,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_reverse_synthesize — Primitives → ranked cloud type matches.
pub fn reverse_synthesize(
    p: params::CloudReverseSynthesizeParams,
) -> Result<CallToolResult, McpError> {
    let target = parse_primitive_list(&p.primitives)?;
    if target.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "primitives list must not be empty",
            None,
        ));
    }

    let target_set: HashSet<_> = target.iter().collect();
    let min_conf = p.min_confidence.unwrap_or(0.0);

    let mut matches: Vec<serde_json::Value> = Vec::new();

    for type_name in all_cloud_types() {
        if let Some(comp) = cloud_composition_for(type_name) {
            let type_set: HashSet<_> = comp.primitives.iter().collect();
            let intersection = target_set.intersection(&type_set).count();
            let union = target_set.union(&type_set).count();
            let jaccard = if union == 0 {
                0.0
            } else {
                intersection as f64 / union as f64
            };

            if jaccard >= min_conf {
                matches.push(json!({
                    "type": type_name,
                    "tier": tier_label(type_name),
                    "overlap": intersection,
                    "jaccard": round3(jaccard),
                    "type_primitives": comp.primitives.iter().map(|p| p.name()).collect::<Vec<_>>(),
                    "missing_from_type": target_set.difference(&type_set).map(|p| p.name()).collect::<Vec<_>>(),
                    "extra_in_type": type_set.difference(&target_set).map(|p| p.name()).collect::<Vec<_>>(),
                }));
            }
        }
    }

    matches.sort_by(|a, b| {
        b["jaccard"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["jaccard"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let result = json!({
        "query_primitives": target.iter().map(|p| p.name()).collect::<Vec<_>>(),
        "min_confidence": min_conf,
        "matches": matches,
        "total_matches": matches.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_list_types — Inventory of all 35 types, filterable by tier.
pub fn list_types(p: params::CloudListTypesParams) -> Result<CallToolResult, McpError> {
    let filter = p.tier.as_deref();

    let types: Vec<serde_json::Value> = all_cloud_types()
        .iter()
        .filter(|name| {
            match filter {
                Some(t) => tier_label(name) == t,
                None => true,
            }
        })
        .map(|name| {
            let comp = cloud_composition_for(name);
            let transfers = transfers_for_type(name);
            json!({
                "name": name,
                "tier": tier_label(name),
                "primitives": comp.as_ref().map(|c| c.primitives.iter().map(|p| p.symbol()).collect::<Vec<_>>()).unwrap_or_default(),
                "dominant": comp.as_ref().and_then(|c| c.dominant.map(|d| d.symbol())),
                "confidence": comp.as_ref().map(|c| round3(c.confidence)).unwrap_or(0.0),
                "transfer_domains": transfers.iter().map(|t| t.domain).collect::<Vec<_>>(),
            })
        })
        .collect();

    let result = json!({
        "filter": filter,
        "total": types.len(),
        "types": types,
        "tiers": {
            "T1": CLOUD_T1.len(),
            "T2-P": CLOUD_T2P.len(),
            "T2-C": CLOUD_T2C.len(),
            "T3": CLOUD_T3.len(),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_molecular_weight — Type → Shannon bits, transfer prediction.
pub fn molecular_weight(p: params::CloudMolecularWeightParams) -> Result<CallToolResult, McpError> {
    let comp = require_composition(&p.type_name)?;

    let formula = MolecularFormula::new(&p.type_name).with_all(&comp.primitives);
    let weight = formula.weight();

    let result = json!({
        "type": p.type_name,
        "tier": tier_label(&p.type_name),
        "formula": formula.formula_string(),
        "molecular_weight_daltons": round3(weight.daltons()),
        "primitive_count": weight.primitive_count(),
        "average_mass": round3(weight.average_mass()),
        "transfer_class": format!("{}", weight.transfer_class()),
        "predicted_transfer_confidence": round3(weight.predicted_transfer()),
        "tier_prediction": format!("{}", weight.tier_aware_class()),
        "hybrid_transfer_confidence": round3(weight.predicted_transfer_hybrid()),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_dominant_shift — Type + added primitive → phase transition detection.
pub fn dominant_shift(p: params::CloudDominantShiftParams) -> Result<CallToolResult, McpError> {
    let comp = require_composition(&p.type_name)?;
    let added = parse_primitive(&p.added_primitive)?;

    let mut new_prims = comp.primitives.clone();
    new_prims.push(added);

    let new_unique: HashSet<_> = new_prims.iter().collect();
    let old_unique: HashSet<_> = comp.primitives.iter().collect();

    let old_tier = tier_from_count(old_unique.len());
    let new_tier = tier_from_count(new_unique.len());
    let tier_shifted = old_tier != new_tier;

    // Compute new dominant by frequency
    let mut freq: std::collections::HashMap<LexPrimitiva, usize> = std::collections::HashMap::new();
    for prim in &new_prims {
        *freq.entry(*prim).or_insert(0) += 1;
    }
    let new_dominant = freq
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(prim, _)| *prim);

    let dominant_shifted = comp.dominant != new_dominant;

    let result = json!({
        "type": p.type_name,
        "added_primitive": {
            "name": added.name(),
            "symbol": added.symbol(),
        },
        "before": {
            "tier": format!("{old_tier:?}"),
            "dominant": comp.dominant.map(|d| d.name()),
            "primitive_count": old_unique.len(),
        },
        "after": {
            "tier": format!("{new_tier:?}"),
            "dominant": new_dominant.map(|d| d.name()),
            "primitive_count": new_unique.len(),
        },
        "tier_shifted": tier_shifted,
        "dominant_shifted": dominant_shifted,
        "phase_transition": tier_shifted,
        "interpretation": if tier_shifted {
            format!("Phase transition: {:?} → {:?}. Adding {} caused tier promotion.", old_tier, new_tier, added.name())
        } else if dominant_shifted {
            format!("Dominant shift: {:?} → {:?}. Structure changed without tier change.", comp.dominant.map(|d| d.name()), new_dominant.map(|d| d.name()))
        } else {
            format!("No structural change. {} already present or redundant.", added.name())
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Phase 2: Infrastructure Awareness (4 tools)
// ============================================================================

/// cloud_infra_status — GCE instance list mapped through cloud primitives.
pub async fn infra_status(p: params::CloudInfraStatusParams) -> Result<CallToolResult, McpError> {
    let mut cmd = tokio::process::Command::new("gcloud");
    cmd.args(["compute", "instances", "list", "--format=json"]);
    if let Some(proj) = &p.project {
        cmd.arg("--project").arg(proj);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| McpError::new(ErrorCode(500), format!("Failed to run gcloud: {e}"), None))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "gcloud failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ))]));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let instances: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();

    let mapped: Vec<serde_json::Value> = instances
        .iter()
        .map(|inst| {
            let name = inst["name"].as_str().unwrap_or("unknown");
            let machine_type = inst["machineType"]
                .as_str()
                .unwrap_or("")
                .rsplit('/')
                .next()
                .unwrap_or("unknown");
            let status = inst["status"].as_str().unwrap_or("UNKNOWN");
            let zone = inst["zone"]
                .as_str()
                .unwrap_or("")
                .rsplit('/')
                .next()
                .unwrap_or("unknown");

            let vm_comp = cloud_composition_for("VirtualMachine");

            json!({
                "name": name,
                "machine_type": machine_type,
                "status": status,
                "zone": zone,
                "cloud_model": "VirtualMachine",
                "composition": vm_comp.as_ref().map(|c| composition_json(c)),
                "tier": "T2-C",
            })
        })
        .collect();

    let result = json!({
        "project": p.project,
        "instance_count": mapped.len(),
        "instances": mapped,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_infra_map — Single instance → full model with composition overlay.
pub async fn infra_map(p: params::CloudInfraMapParams) -> Result<CallToolResult, McpError> {
    let mut cmd = tokio::process::Command::new("gcloud");
    cmd.args(["compute", "instances", "describe"]);
    cmd.arg(&p.instance);
    cmd.arg("--format=json");
    if let Some(z) = &p.zone {
        cmd.arg("--zone").arg(z);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| McpError::new(ErrorCode(500), format!("Failed to run gcloud: {e}"), None))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "gcloud failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ))]));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let inst: serde_json::Value = serde_json::from_str(&stdout).unwrap_or(json!({}));

    let name = inst["name"].as_str().unwrap_or("unknown");
    let machine_type = inst["machineType"]
        .as_str()
        .unwrap_or("")
        .rsplit('/')
        .next()
        .unwrap_or("unknown");
    let status = inst["status"].as_str().unwrap_or("UNKNOWN");

    // Map to VirtualMachine composition
    let vm_comp = cloud_composition_for("VirtualMachine");
    // Map disks to Storage
    let storage_comp = cloud_composition_for("Storage");
    // Map network to NetworkLink
    let network_comp = cloud_composition_for("NetworkLink");

    // Collect combined primitives
    let mut combined = HashSet::new();
    if let Some(ref c) = vm_comp {
        for prim in &c.primitives {
            combined.insert(prim.name());
        }
    }
    if let Some(ref c) = storage_comp {
        for prim in &c.primitives {
            combined.insert(prim.name());
        }
    }
    if let Some(ref c) = network_comp {
        for prim in &c.primitives {
            combined.insert(prim.name());
        }
    }
    let combined_list: Vec<_> = combined.into_iter().collect();

    let result = json!({
        "instance": name,
        "machine_type": machine_type,
        "status": status,
        "model_mapping": {
            "compute": {
                "cloud_type": "VirtualMachine",
                "composition": vm_comp.as_ref().map(|c| composition_json(c)),
            },
            "storage": {
                "cloud_type": "Storage",
                "composition": storage_comp.as_ref().map(|c| composition_json(c)),
            },
            "network": {
                "cloud_type": "NetworkLink",
                "composition": network_comp.as_ref().map(|c| composition_json(c)),
            },
        },
        "combined_primitives": combined_list,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_capacity_project — Pure computation: utilization projection over time.
pub fn capacity_project(p: params::CloudCapacityProjectParams) -> Result<CallToolResult, McpError> {
    if p.total_capacity <= 0.0 {
        return Err(McpError::new(
            ErrorCode(400),
            "total_capacity must be positive",
            None,
        ));
    }
    if !p.current_utilization.is_finite()
        || p.current_utilization < 0.0
        || p.current_utilization > 1.0
    {
        return Err(McpError::new(
            ErrorCode(400),
            "current_utilization must be between 0.0 and 1.0",
            None,
        ));
    }

    let current_used = p.current_utilization * p.total_capacity;
    let mut projections = Vec::new();
    let mut days_until_full: Option<u32> = None;

    for day in 0..=p.days {
        let used = current_used + (p.daily_growth * day as f64);
        let util = (used / p.total_capacity).min(1.0);
        projections.push(json!({
            "day": day,
            "utilization": round3(util),
            "used": round3(used),
        }));
        if days_until_full.is_none() && used >= p.total_capacity {
            days_until_full = Some(day);
        }
    }

    let recommendation = match days_until_full {
        Some(d) if d <= 7 => format!(
            "CRITICAL: Capacity exhausted in {} days. Scale immediately.",
            d
        ),
        Some(d) if d <= 30 => format!("WARNING: Capacity exhausted in {} days. Plan scaling.", d),
        Some(d) => format!("INFO: Capacity exhausted in {} days. Monitor growth.", d),
        None => format!("OK: Capacity sufficient for {} day projection.", p.days),
    };

    // Map to cloud primitives
    let pool_comp = cloud_composition_for("ResourcePool");
    let threshold_comp = cloud_composition_for("Threshold");

    let result = json!({
        "current_utilization": round3(p.current_utilization),
        "total_capacity": p.total_capacity,
        "daily_growth": p.daily_growth,
        "days_projected": p.days,
        "days_until_full": days_until_full,
        "recommendation": recommendation,
        "projections": projections,
        "grounding": {
            "resource_pool": pool_comp.as_ref().map(|c| composition_json(c)),
            "threshold": threshold_comp.as_ref().map(|c| composition_json(c)),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_supervisor_health — Nexcloud supervisor status through cloud primitives.
pub fn supervisor_health(
    p: params::CloudSupervisorHealthParams,
) -> Result<CallToolResult, McpError> {
    let health_comp = cloud_composition_for("HealthCheck");
    let feedback_comp = cloud_composition_for("FeedbackLoop");

    let result = json!({
        "supervisor": "nexcloud",
        "model_type": "HealthCheck + FeedbackLoop",
        "composition_overlay": if p.include_composition {
            json!({
                "health_check": health_comp.as_ref().map(|c| composition_json(c)),
                "feedback_loop": feedback_comp.as_ref().map(|c| composition_json(c)),
            })
        } else {
            json!(null)
        },
        "grounding": {
            "primary": "∃ (Existence) — service liveness verification",
            "secondary": "→ (Causality) — feedback-driven correction",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Phase 3: Cross-Domain Reasoning (5 tools)
// ============================================================================

/// cloud_reverse_transfer — Domain concept → matching cloud types.
pub fn reverse_transfer(p: params::CloudReverseTransferParams) -> Result<CallToolResult, McpError> {
    if p.keywords.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "keywords list must not be empty",
            None,
        ));
    }

    let all_mappings = transfer_mappings();
    let domain_lower = p.domain.to_lowercase();
    let keywords_lower: Vec<String> = p.keywords.iter().map(|k| k.to_lowercase()).collect();

    let mut results: Vec<serde_json::Value> = Vec::new();

    for mapping in &all_mappings {
        if mapping.domain.to_lowercase() != domain_lower {
            continue;
        }

        let analog_lower = mapping.analog.to_lowercase();
        let keyword_hits: usize = keywords_lower
            .iter()
            .filter(|kw| analog_lower.contains(kw.as_str()))
            .count();

        if keyword_hits > 0 {
            let relevance = keyword_hits as f64 / keywords_lower.len() as f64;
            results.push(json!({
                "cloud_type": mapping.cloud_type,
                "analog": mapping.analog,
                "transfer_confidence": mapping.confidence,
                "keyword_match_ratio": round3(relevance),
                "combined_score": round3(relevance * mapping.confidence),
            }));
        }
    }

    results.sort_by(|a, b| {
        b["combined_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["combined_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let result = json!({
        "domain": p.domain,
        "keywords": p.keywords,
        "matches": results,
        "total_matches": results.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_transfer_chain — Multi-hop BFS across transfer graph.
pub fn transfer_chain(p: params::CloudTransferChainParams) -> Result<CallToolResult, McpError> {
    // Validate start type exists
    require_composition(&p.start_type)?;
    // BFS currently implements up to 2-hop chains; cap max_hops accordingly
    let max_hops = p.max_hops.min(2);
    let confidence_floor = 0.50;

    let all_mappings = transfer_mappings();
    let target_lower = p.target_domain.to_lowercase();

    // Direct transfer check first
    let direct = all_mappings
        .iter()
        .find(|m| m.cloud_type == p.start_type && m.domain.to_lowercase() == target_lower);

    let mut chains: Vec<serde_json::Value> = Vec::new();

    if let Some(d) = direct {
        chains.push(json!({
            "path": [&p.start_type, &format!("{} ({})", d.analog, d.domain)],
            "hops": 1,
            "cumulative_confidence": d.confidence,
            "chain_type": "direct",
        }));
    }

    // BFS for multi-hop chains (type → domain analog → shared primitive → type → domain)
    if max_hops >= 2 {
        let start_transfers = transfers_for_type(&p.start_type);
        let start_comp = cloud_composition_for(&p.start_type);

        for hop1 in &start_transfers {
            for intermediate_type in all_cloud_types() {
                if *intermediate_type == p.start_type {
                    continue;
                }
                let int_comp = cloud_composition_for(intermediate_type);

                // Check primitive overlap as bridge
                if let (Some(sc), Some(ic)) = (&start_comp, &int_comp) {
                    let s_set: HashSet<_> = sc.primitives.iter().collect();
                    let i_set: HashSet<_> = ic.primitives.iter().collect();
                    let shared = s_set.intersection(&i_set).count();
                    if shared == 0 {
                        continue;
                    }

                    // Check if intermediate has path to target domain
                    let int_to_target = all_mappings.iter().find(|m| {
                        m.cloud_type == &*intermediate_type
                            && m.domain.to_lowercase() == target_lower
                    });

                    if let Some(hop2) = int_to_target {
                        let cumulative = hop1.confidence * hop2.confidence;
                        if cumulative >= confidence_floor {
                            chains.push(json!({
                                "path": [
                                    &p.start_type,
                                    &format!("{} via {}", hop1.analog, hop1.domain),
                                    intermediate_type,
                                    &format!("{} ({})", hop2.analog, hop2.domain),
                                ],
                                "hops": 2,
                                "cumulative_confidence": round3(cumulative),
                                "primitive_bridge": shared,
                                "chain_type": "primitive-bridged",
                            }));
                        }
                    }
                }
            }
        }
    }

    chains.sort_by(|a, b| {
        b["cumulative_confidence"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["cumulative_confidence"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let result = json!({
        "start_type": p.start_type,
        "target_domain": p.target_domain,
        "max_hops": max_hops,
        "confidence_floor": confidence_floor,
        "chains": chains,
        "total_chains": chains.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_architecture_advisor — Primitives + constraints → ranked recommendations.
pub fn architecture_advisor(
    p: params::CloudArchitectureAdvisorParams,
) -> Result<CallToolResult, McpError> {
    let required = parse_primitive_list(&p.required_primitives)?;
    if required.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "required_primitives must not be empty",
            None,
        ));
    }

    let required_set: HashSet<_> = required.iter().collect();
    let top_n = p.top_n.unwrap_or(5);

    let mut candidates: Vec<serde_json::Value> = Vec::new();

    for type_name in all_cloud_types() {
        // Filter by tier if specified
        if let Some(ref pref) = p.preferred_tier {
            if tier_label(type_name) != pref.as_str() {
                continue;
            }
        }

        if let Some(comp) = cloud_composition_for(type_name) {
            let type_set: HashSet<_> = comp.primitives.iter().collect();
            let covered = required_set.intersection(&type_set).count();
            let missing = required_set.difference(&type_set).count();

            if covered == 0 {
                continue;
            }

            // Jaccard-based scoring
            let union = required_set.union(&type_set).count();
            let jaccard = covered as f64 / union as f64;

            // Bonus for exact dominant match
            let dominant_bonus = if let Some(dom) = comp.dominant {
                if required_set.contains(&dom) {
                    0.1
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Penalty for missing required primitives
            let missing_penalty = missing as f64 * 0.05;

            let score = (jaccard + dominant_bonus - missing_penalty).max(0.0);

            candidates.push(json!({
                "type": type_name,
                "tier": tier_label(type_name),
                "score": round3(score),
                "jaccard": round3(jaccard),
                "covered_primitives": covered,
                "missing_primitives": required_set.difference(&type_set).map(|p| p.name()).collect::<Vec<_>>(),
                "extra_primitives": type_set.difference(&required_set).map(|p| p.name()).collect::<Vec<_>>(),
                "dominant": comp.dominant.map(|d| d.name()),
                "dominant_match": comp.dominant.map_or(false, |d| required_set.contains(&d)),
            }));
        }
    }

    candidates.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates.truncate(top_n);

    let result = json!({
        "required_primitives": required.iter().map(|p| p.name()).collect::<Vec<_>>(),
        "preferred_tier": p.preferred_tier,
        "recommendations": candidates,
        "total_candidates": candidates.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_anomaly_detect — Type + observed → drift detection.
pub fn anomaly_detect(p: params::CloudAnomalyDetectParams) -> Result<CallToolResult, McpError> {
    if p.observed_primitives.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "observed_primitives must not be empty",
            None,
        ));
    }
    let expected_comp = require_composition(&p.type_name)?;
    let observed = parse_primitive_list(&p.observed_primitives)?;

    let expected_set: HashSet<_> = expected_comp.primitives.iter().collect();
    let observed_set: HashSet<_> = observed.iter().collect();

    let missing: Vec<_> = expected_set
        .difference(&observed_set)
        .map(|p| p.name())
        .collect();
    let unexpected: Vec<_> = observed_set
        .difference(&expected_set)
        .map(|p| p.name())
        .collect();
    let confirmed: Vec<_> = expected_set
        .intersection(&observed_set)
        .map(|p| p.name())
        .collect();

    let drift_score = if expected_set.is_empty() {
        0.0
    } else {
        (missing.len() + unexpected.len()) as f64 / (expected_set.len() + observed_set.len()) as f64
    };

    let severity = if drift_score == 0.0 {
        "none"
    } else if drift_score < 0.2 {
        "low"
    } else if drift_score < 0.5 {
        "medium"
    } else {
        "high"
    };

    // Check if dominant is preserved
    let dominant_preserved = expected_comp
        .dominant
        .map_or(false, |d| observed_set.contains(&d));

    let result = json!({
        "type": p.type_name,
        "tier": tier_label(&p.type_name),
        "expected_count": expected_set.len(),
        "observed_count": observed_set.len(),
        "confirmed_primitives": confirmed,
        "missing_primitives": missing,
        "unexpected_primitives": unexpected,
        "drift_score": round3(drift_score),
        "severity": severity,
        "dominant_preserved": dominant_preserved,
        "dominant": expected_comp.dominant.map(|d| d.name()),
        "interpretation": if drift_score == 0.0 {
            "Perfect match. No anomalies detected.".to_string()
        } else if !dominant_preserved {
            format!("CRITICAL: Dominant primitive {} is missing. Type identity compromised.",
                expected_comp.dominant.map(|d| d.name()).unwrap_or("unknown"))
        } else {
            format!("Drift detected ({} severity): {} missing, {} unexpected.",
                severity, missing.len(), unexpected.len())
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// cloud_transfer_matrix — Full type×domain confidence matrix.
pub fn transfer_matrix(p: params::CloudTransferMatrixParams) -> Result<CallToolResult, McpError> {
    let all_mappings = transfer_mappings();
    let domains = ["PV", "Biology", "Economics"];

    let types: Vec<&str> = all_cloud_types()
        .into_iter()
        .filter(|name| match p.tier.as_deref() {
            Some(t) => tier_label(name) == t,
            None => true,
        })
        .collect();

    let mut matrix: Vec<serde_json::Value> = Vec::new();

    for type_name in &types {
        let mut row = json!({
            "type": type_name,
            "tier": tier_label(type_name),
        });

        for domain in &domains {
            if let Some(ref filter_domain) = p.domain {
                if domain.to_lowercase() != filter_domain.to_lowercase() {
                    continue;
                }
            }

            let mapping = all_mappings
                .iter()
                .find(|m| m.cloud_type == *type_name && m.domain == *domain);

            if let Some(m) = mapping {
                row[domain.to_lowercase()] = json!({
                    "analog": m.analog,
                    "confidence": m.confidence,
                });
            } else {
                row[domain.to_lowercase()] = json!(null);
            }
        }

        matrix.push(row);
    }

    // Compute domain averages
    let mut domain_stats = json!({});
    for domain in &domains {
        if let Some(ref filter_domain) = p.domain {
            if domain.to_lowercase() != filter_domain.to_lowercase() {
                continue;
            }
        }
        let confs: Vec<f64> = all_mappings
            .iter()
            .filter(|m| m.domain == *domain)
            .filter(|m| types.contains(&m.cloud_type))
            .map(|m| m.confidence)
            .collect();
        if !confs.is_empty() {
            let avg = confs.iter().sum::<f64>() / confs.len() as f64;
            let min = confs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = confs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            domain_stats[domain.to_lowercase()] = json!({
                "count": confs.len(),
                "avg_confidence": round3(avg),
                "min_confidence": round3(min),
                "max_confidence": round3(max),
            });
        }
    }

    let result = json!({
        "filter_tier": p.tier,
        "filter_domain": p.domain,
        "type_count": types.len(),
        "matrix": matrix,
        "domain_stats": domain_stats,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_35_types_resolve() {
        for name in all_cloud_types() {
            let comp = cloud_composition_for(name);
            assert!(comp.is_some(), "Failed to resolve cloud type: {}", name);
        }
    }

    #[test]
    fn test_primitive_composition() {
        let result = primitive_composition(params::CloudCompositionParams {
            type_name: "VirtualMachine".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_primitive_composition_unknown() {
        let result = primitive_composition(params::CloudCompositionParams {
            type_name: "NonExistent".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_confidence_found() {
        let result = transfer_confidence(params::CloudTransferConfidenceParams {
            cloud_type: "Identity".to_string(),
            domain: "PV".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_transfer_confidence_missing() {
        let result = transfer_confidence(params::CloudTransferConfidenceParams {
            cloud_type: "VirtualMachine".to_string(),
            domain: "PV".to_string(),
        });
        // VirtualMachine has no transfers — should error
        assert!(result.is_err());
    }

    #[test]
    fn test_tier_classify_tool() {
        let result = tier_classify(params::CloudTierClassifyParams {
            primitives: vec!["quantity".to_string(), "boundary".to_string()],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_compare_types_tool() {
        let result = compare_types(params::CloudCompareTypesParams {
            type_a: "Compute".to_string(),
            type_b: "Metering".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_reverse_synthesize_tool() {
        let result = reverse_synthesize(params::CloudReverseSynthesizeParams {
            primitives: vec!["quantity".to_string(), "frequency".to_string()],
            min_confidence: Some(0.3),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_types_all() {
        let result = list_types(params::CloudListTypesParams { tier: None });
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_types_filtered() {
        let result = list_types(params::CloudListTypesParams {
            tier: Some("T1".to_string()),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_molecular_weight_tool() {
        let result = molecular_weight(params::CloudMolecularWeightParams {
            type_name: "Serverless".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_dominant_shift_no_change() {
        let result = dominant_shift(params::CloudDominantShiftParams {
            type_name: "Compute".to_string(),
            added_primitive: "quantity".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_dominant_shift_with_change() {
        let result = dominant_shift(params::CloudDominantShiftParams {
            type_name: "Identity".to_string(),
            added_primitive: "boundary".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_capacity_project_tool() {
        let result = capacity_project(params::CloudCapacityProjectParams {
            current_utilization: 0.7,
            total_capacity: 100.0,
            daily_growth: 2.0,
            days: 30,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_supervisor_health_tool() {
        let result = supervisor_health(params::CloudSupervisorHealthParams {
            include_composition: true,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_reverse_transfer_tool() {
        let result = reverse_transfer(params::CloudReverseTransferParams {
            domain: "PV".to_string(),
            keywords: vec!["signal".to_string(), "detection".to_string()],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_transfer_chain_tool() {
        let result = transfer_chain(params::CloudTransferChainParams {
            start_type: "Threshold".to_string(),
            target_domain: "Biology".to_string(),
            max_hops: 3,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_architecture_advisor_tool() {
        let result = architecture_advisor(params::CloudArchitectureAdvisorParams {
            required_primitives: vec![
                "boundary".to_string(),
                "mapping".to_string(),
                "persistence".to_string(),
            ],
            preferred_tier: None,
            top_n: Some(3),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_anomaly_detect_no_drift() {
        let result = anomaly_detect(params::CloudAnomalyDetectParams {
            type_name: "Compute".to_string(),
            observed_primitives: vec!["quantity".to_string(), "frequency".to_string()],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_anomaly_detect_with_drift() {
        let result = anomaly_detect(params::CloudAnomalyDetectParams {
            type_name: "Compute".to_string(),
            observed_primitives: vec!["boundary".to_string()],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_transfer_matrix_tool() {
        let result = transfer_matrix(params::CloudTransferMatrixParams {
            tier: Some("T1".to_string()),
            domain: None,
        });
        assert!(result.is_ok());
    }
}
