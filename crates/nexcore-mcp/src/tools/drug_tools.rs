//! Drug entity MCP tools — query individual drug safety profiles from the catalog.
//!
//! Four read-only tools:
//! - `drug_profile`:       Full Drug JSON for a named drug
//! - `drug_signals`:       Vec<SignalEntry> portfolio (PRR/ROR/IC values)
//! - `drug_compare`:       Per-event PRR comparison between two drugs
//! - `drug_class_members`: All catalog drugs belonging to a given DrugClass
//!
//! ## Registry
//!
//! `resolve_drug(name)` maps lowercase generic names to the static `catalog::drug()`
//! constructor from each per-drug crate. Only drugs present in the catalog return
//! `Some(Drug)` — unknown names return `None` which surfaces as a typed error.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Drug name → Drug struct | Mapping | μ |
//! | Unknown drug name | Void | ∅ |
//! | Signal portfolio | Sequence | σ |
//! | Per-event comparison | Comparison | κ |
//! | All class members | Sum | Σ |

use nexcore_drug::{
    Drug,
    analysis::{DefaultDrugAnalysis, DrugAnalysis},
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

use crate::params::{
    DrugClassMembersParams, DrugCompareParams, DrugProfileParams, DrugSignalsParams,
};

// ---------------------------------------------------------------------------
// Registry — map generic name → Drug
// ---------------------------------------------------------------------------

/// Resolve a generic drug name (case-insensitive) to its catalog `Drug` struct.
///
/// Returns `None` for any name not present in the 10-drug catalog.
fn resolve_drug(name: &str) -> Option<Drug> {
    match name.to_lowercase().as_str() {
        "tirzepatide" => Some(nexcore_drug_tirzepatide::catalog::drug()),
        "semaglutide" => Some(nexcore_drug_semaglutide::catalog::drug()),
        "donanemab" => Some(nexcore_drug_donanemab::catalog::drug()),
        "pembrolizumab" => Some(nexcore_drug_pembrolizumab::catalog::drug()),
        "adalimumab" => Some(nexcore_drug_adalimumab::catalog::drug()),
        "apixaban" => Some(nexcore_drug_apixaban::catalog::drug()),
        "dapagliflozin" => Some(nexcore_drug_dapagliflozin::catalog::drug()),
        "osimertinib" => Some(nexcore_drug_osimertinib::catalog::drug()),
        "secukinumab" => Some(nexcore_drug_secukinumab::catalog::drug()),
        "upadacitinib" => Some(nexcore_drug_upadacitinib::catalog::drug()),
        _ => None,
    }
}

/// Return all 10 catalog drugs.
fn all_catalog_drugs() -> Vec<Drug> {
    vec![
        nexcore_drug_tirzepatide::catalog::drug(),
        nexcore_drug_semaglutide::catalog::drug(),
        nexcore_drug_donanemab::catalog::drug(),
        nexcore_drug_pembrolizumab::catalog::drug(),
        nexcore_drug_adalimumab::catalog::drug(),
        nexcore_drug_apixaban::catalog::drug(),
        nexcore_drug_dapagliflozin::catalog::drug(),
        nexcore_drug_osimertinib::catalog::drug(),
        nexcore_drug_secukinumab::catalog::drug(),
        nexcore_drug_upadacitinib::catalog::drug(),
    ]
}

/// Human-readable list of supported drug names for error messages.
const KNOWN_DRUGS: &str = "tirzepatide, semaglutide, donanemab, pembrolizumab, adalimumab, \
     apixaban, dapagliflozin, osimertinib, secukinumab, upadacitinib";

// ---------------------------------------------------------------------------
// drug_profile
// ---------------------------------------------------------------------------

/// Get the complete drug profile including signals, indications, and label status.
pub fn drug_profile(params: DrugProfileParams) -> Result<CallToolResult, McpError> {
    let drug = resolve_drug(&params.drug_name).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown drug '{}'. Supported drugs: {}",
                params.drug_name, KNOWN_DRUGS
            ),
            None,
        )
    })?;

    let response = json!({
        "drug_name": drug.generic_name,
        "brand_names": drug.brand_names,
        "rxcui": drug.rxcui,
        "mechanism": drug.mechanism,
        "drug_class": drug.drug_class,
        "indications": drug.indications,
        "contraindications": drug.contraindications,
        "safety_signals": drug.safety_signals,
        "label_status": drug.label_status,
        "owner": drug.owner,
        "signal_count": drug.signal_count(),
        "indication_count": drug.indication_count(),
        "has_boxed_warning": drug.has_boxed_warning(),
        "has_rems": drug.has_rems(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// drug_signals
// ---------------------------------------------------------------------------

/// Get all safety signals for a drug with PRR/ROR/IC values.
pub fn drug_signals(params: DrugSignalsParams) -> Result<CallToolResult, McpError> {
    let drug = resolve_drug(&params.drug_name).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown drug '{}'. Supported drugs: {}",
                params.drug_name, KNOWN_DRUGS
            ),
            None,
        )
    })?;

    let analysis = DefaultDrugAnalysis::new(&drug);
    let signals = analysis.signal_portfolio();

    let strongest = analysis.strongest_signal().map(|s| &s.event);

    let response = json!({
        "drug_name": drug.generic_name,
        "signal_count": signals.len(),
        "strongest_signal": strongest,
        "on_label_count": analysis.on_label_signals().len(),
        "off_label_count": analysis.off_label_signals().len(),
        "signals": signals,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// drug_compare
// ---------------------------------------------------------------------------

/// Compare safety profiles of two drugs using per-event PRR comparison.
pub fn drug_compare(params: DrugCompareParams) -> Result<CallToolResult, McpError> {
    let drug_a = resolve_drug(&params.drug_a).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown drug_a '{}'. Supported drugs: {}",
                params.drug_a, KNOWN_DRUGS
            ),
            None,
        )
    })?;

    let drug_b = resolve_drug(&params.drug_b).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown drug_b '{}'. Supported drugs: {}",
                params.drug_b, KNOWN_DRUGS
            ),
            None,
        )
    })?;

    let analysis_a = DefaultDrugAnalysis::new(&drug_a);
    let analysis_b = DefaultDrugAnalysis::new(&drug_b);
    let comparisons = analysis_a.compare_signals(&analysis_b);

    let drug_a_advantages = comparisons
        .iter()
        .filter(|c| matches!(c.advantage, nexcore_drug::analysis::ComparisonResult::DrugA))
        .count();
    let drug_b_advantages = comparisons
        .iter()
        .filter(|c| matches!(c.advantage, nexcore_drug::analysis::ComparisonResult::DrugB))
        .count();
    let neutral_count = comparisons
        .iter()
        .filter(|c| {
            matches!(
                c.advantage,
                nexcore_drug::analysis::ComparisonResult::Neutral
            )
        })
        .count();

    let response = json!({
        "drug_a": drug_a.generic_name,
        "drug_b": drug_b.generic_name,
        "events_compared": comparisons.len(),
        "drug_a_advantages": drug_a_advantages,
        "drug_b_advantages": drug_b_advantages,
        "neutral": neutral_count,
        "summary": format!(
            "{} has lower PRR on {} events; {} on {}; {} neutral",
            drug_a.generic_name, drug_a_advantages,
            drug_b.generic_name, drug_b_advantages,
            neutral_count
        ),
        "comparisons": comparisons,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// drug_class_members
// ---------------------------------------------------------------------------

/// List all drugs in the catalog that belong to the given drug class.
pub fn drug_class_members(params: DrugClassMembersParams) -> Result<CallToolResult, McpError> {
    let all = all_catalog_drugs();

    let members: Vec<_> = if params.drug_class.eq_ignore_ascii_case("all") {
        all.iter()
            .map(|d| {
                json!({
                    "generic_name": d.generic_name,
                    "brand_names": d.brand_names,
                    "drug_class": d.drug_class.to_string(),
                    "owner": d.owner,
                    "signal_count": d.signal_count(),
                    "has_boxed_warning": d.has_boxed_warning(),
                })
            })
            .collect()
    } else {
        // Normalize the query: lowercase, strip spaces and hyphens for fuzzy matching
        let query = params.drug_class.to_lowercase().replace([' ', '-'], "");
        all.iter()
            .filter(|d| {
                let class_str = format!("{:?}", d.drug_class)
                    .to_lowercase()
                    .replace([' ', '-'], "");
                let class_display = d
                    .drug_class
                    .to_string()
                    .to_lowercase()
                    .replace([' ', '-', '/'], "");
                class_str.contains(&query) || class_display.contains(&query)
            })
            .map(|d| {
                json!({
                    "generic_name": d.generic_name,
                    "brand_names": d.brand_names,
                    "drug_class": d.drug_class.to_string(),
                    "owner": d.owner,
                    "signal_count": d.signal_count(),
                    "has_boxed_warning": d.has_boxed_warning(),
                })
            })
            .collect()
    };

    let response = json!({
        "drug_class_query": params.drug_class,
        "member_count": members.len(),
        "members": members,
        "available_classes": [
            "GLP1ReceptorAgonist", "GLP1GIPDualAgonist", "AntiAmyloid",
            "CheckpointInhibitor", "JAKInhibitor", "SGLT2Inhibitor",
            "EGFRTKInhibitor", "AntiTNF", "AntiIL17", "Anticoagulant"
        ],
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
