//! Pharma company entity MCP tools.
//!
//! Four read-only tools that expose company profiles, signal portfolios,
//! pipeline candidates, and boxed-warning product lists to AI agents.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Company name → struct dispatch | Mapping | μ |
//! | Unknown company | Void | ∅ |
//! | Phase filter predicate | Comparison | κ |
//! | Signal/product counts | Quantity | N |
//! | JSON serialization | Persistence | π |
//! | Tool → result chain | Causality | → |

use crate::params::{
    PharmaBoxedWarningsParams, PharmaCompanyProfileParams, PharmaPipelineParams,
    PharmaSignalPortfolioParams,
};
use nexcore_pharma::{CompanyAnalysis, Phase};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// Company registry
// ---------------------------------------------------------------------------

/// Load a company by name as a boxed `CompanyAnalysis` trait object.
///
/// Returns `None` for unrecognised names. The owned company struct is held
/// inside each per-company type, so lifetimes are trivially satisfied.
fn load_company(name: &str) -> Option<Box<dyn CompanyAnalysis>> {
    match name
        .to_lowercase()
        .replace([' ', '_'], "-")
        .trim_matches('-')
        .to_string()
        .as_str()
    {
        "abbvie" => Some(Box::new(nexcore_pharma_abbvie::AbbVie::load())),
        "astrazeneca" | "astra-zeneca" => {
            Some(Box::new(nexcore_pharma_astrazeneca::AstraZeneca::load()))
        }
        "bms" | "bristol-myers-squibb" | "bristol-myers" => {
            Some(Box::new(nexcore_pharma_bms::BristolMyersSquibb::load()))
        }
        "gsk" | "glaxosmithkline" => Some(Box::new(nexcore_pharma_gsk::Gsk::load())),
        "jnj" | "johnson-and-johnson" | "janssen" => {
            Some(Box::new(nexcore_pharma_jnj::JohnsonAndJohnson::load()))
        }
        "lilly" | "eli-lilly" | "eli-lilly-and-company" => {
            Some(Box::new(nexcore_pharma_lilly::Lilly::load()))
        }
        "merck" | "msd" => Some(Box::new(nexcore_pharma_merck::Merck::load())),
        "novartis" => Some(Box::new(nexcore_pharma_novartis::Novartis::load())),
        "novo-nordisk" | "novonordisk" | "novo" => {
            Some(Box::new(nexcore_pharma_novo_nordisk::NovoNordisk::load()))
        }
        "pfizer" => Some(Box::new(nexcore_pharma_pfizer::Pfizer::load())),
        "roche" | "hoffmann-la-roche" | "genentech" => {
            Some(Box::new(nexcore_pharma_roche::Roche::load()))
        }
        "takeda" => Some(Box::new(nexcore_pharma_takeda::Takeda::load())),
        _ => None,
    }
}

/// Parse a phase string into a `Phase` enum variant.
fn parse_phase(s: &str) -> Option<Phase> {
    match s.to_lowercase().replace([' ', '_', '-'], "").as_str() {
        "preclinical" | "0" => Some(Phase::Preclinical),
        "phase1" | "1" => Some(Phase::Phase1),
        "phase2" | "2" => Some(Phase::Phase2),
        "phase3" | "3" => Some(Phase::Phase3),
        "filed" | "nda" | "bla" | "4" => Some(Phase::Filed),
        "approved" | "5" => Some(Phase::Approved),
        _ => None,
    }
}

/// Produce a standardised "company not found" error result.
fn unknown_company_error(name: &str) -> Result<CallToolResult, McpError> {
    let msg = json!({
        "error": "company_not_found",
        "company": name,
        "accepted": [
            "abbvie", "astrazeneca", "bms", "gsk", "jnj",
            "lilly", "merck", "novartis", "novo-nordisk",
            "pfizer", "roche", "takeda"
        ]
    });
    Ok(CallToolResult::error(vec![Content::text(
        serde_json::to_string_pretty(&msg).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// pharma_company_profile
// ---------------------------------------------------------------------------

/// Return the full company profile as JSON.
///
/// Includes: id, name, ticker, headquarters, therapeutic areas, all products
/// (with safety profiles and signals), pipeline candidates, and safety
/// communications.
pub fn pharma_company_profile(
    params: PharmaCompanyProfileParams,
) -> Result<CallToolResult, McpError> {
    let handle = match load_company(&params.company) {
        Some(h) => h,
        None => return unknown_company_error(&params.company),
    };

    let co = handle.company();
    let focus = handle.therapeutic_focus();

    let response = json!({
        "id": co.id.as_str(),
        "name": co.name,
        "ticker": co.ticker,
        "headquarters": co.headquarters,
        "therapeutic_areas": co.therapeutic_areas.iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>(),
        "therapeutic_focus": focus.iter().map(|(area, count)| json!({
            "area": area.to_string(),
            "product_count": count,
        })).collect::<Vec<_>>(),
        "product_count": co.products.len(),
        "products": co.products.iter().map(|p| json!({
            "generic_name": p.generic_name,
            "brand_names": p.brand_names,
            "rxcui": p.rxcui,
            "therapeutic_area": p.therapeutic_area.to_string(),
            "approval_year": p.approval_year,
            "safety_profile": {
                "boxed_warning": p.safety_profile.boxed_warning,
                "rems": p.safety_profile.rems,
                "signal_count": p.safety_profile.signals.len(),
                "label_warnings": p.safety_profile.label_warnings,
                "signals": p.safety_profile.signals.iter().map(|s| json!({
                    "event": s.event,
                    "prr": s.prr,
                    "ror": s.ror,
                    "cases": s.cases,
                    "on_label": s.on_label,
                    "verdict": s.verdict.to_string(),
                })).collect::<Vec<_>>(),
            },
        })).collect::<Vec<_>>(),
        "pipeline_count": co.pipeline.len(),
        "pipeline": co.pipeline.iter().map(|c| json!({
            "name": c.name,
            "mechanism": c.mechanism,
            "phase": c.phase.to_string(),
            "indication": c.indication,
            "therapeutic_area": c.therapeutic_area.to_string(),
        })).collect::<Vec<_>>(),
        "safety_communication_count": co.safety_communications.len(),
        "safety_communications": co.safety_communications.iter().map(|sc| json!({
            "title": sc.title,
            "date": sc.date,
            "comm_type": sc.comm_type.to_string(),
            "product": sc.product,
            "summary": sc.summary,
            "is_urgent": sc.is_urgent(),
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// pharma_signal_portfolio
// ---------------------------------------------------------------------------

/// Return all safety signals across all products for a company, flattened.
///
/// Useful for competitive signal surveillance — e.g. "what signals has Pfizer
/// accumulated across its entire portfolio?"
pub fn pharma_signal_portfolio(
    params: PharmaSignalPortfolioParams,
) -> Result<CallToolResult, McpError> {
    let handle = match load_company(&params.company) {
        Some(h) => h,
        None => return unknown_company_error(&params.company),
    };

    let co = handle.company();
    let signals = handle.signal_portfolio();

    let signals_json: Vec<_> = signals
        .iter()
        .map(|s| {
            json!({
                "event": s.event,
                "prr": s.prr,
                "ror": s.ror,
                "cases": s.cases,
                "on_label": s.on_label,
                "verdict": s.verdict.to_string(),
            })
        })
        .collect();

    let response = json!({
        "company": co.name,
        "ticker": co.ticker,
        "total_signals": signals_json.len(),
        "signals": signals_json,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// pharma_pipeline
// ---------------------------------------------------------------------------

/// Return pipeline candidates for a company, optionally filtered to one phase.
///
/// Phase values: preclinical, phase1, phase2, phase3, filed, approved.
/// Omitting `phase` returns all candidates.
pub fn pharma_pipeline(params: PharmaPipelineParams) -> Result<CallToolResult, McpError> {
    let handle = match load_company(&params.company) {
        Some(h) => h,
        None => return unknown_company_error(&params.company),
    };

    let co = handle.company();

    let (phase_filter_display, candidates_json): (Option<String>, Vec<_>) = match &params.phase {
        Some(phase_str) => {
            let phase = match parse_phase(phase_str) {
                Some(p) => p,
                None => {
                    let msg = json!({
                        "error": "invalid_phase",
                        "phase": phase_str,
                        "accepted": [
                            "preclinical", "phase1", "phase2",
                            "phase3", "filed", "approved"
                        ],
                    });
                    return Ok(CallToolResult::error(vec![Content::text(
                        serde_json::to_string_pretty(&msg).unwrap_or_else(|_| "{}".to_string()),
                    )]));
                }
            };
            let label = phase.to_string();
            let filtered = handle
                .pipeline_by_phase(phase)
                .into_iter()
                .map(|c| {
                    json!({
                        "name": c.name,
                        "mechanism": c.mechanism,
                        "phase": c.phase.to_string(),
                        "indication": c.indication,
                        "therapeutic_area": c.therapeutic_area.to_string(),
                    })
                })
                .collect();
            (Some(label), filtered)
        }
        None => {
            let all = co
                .pipeline
                .iter()
                .map(|c| {
                    json!({
                        "name": c.name,
                        "mechanism": c.mechanism,
                        "phase": c.phase.to_string(),
                        "indication": c.indication,
                        "therapeutic_area": c.therapeutic_area.to_string(),
                    })
                })
                .collect();
            (None, all)
        }
    };

    let response = json!({
        "company": co.name,
        "ticker": co.ticker,
        "phase_filter": phase_filter_display,
        "total_candidates": candidates_json.len(),
        "candidates": candidates_json,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// pharma_boxed_warnings
// ---------------------------------------------------------------------------

/// Return all products carrying an FDA boxed warning for a company.
///
/// Useful for safety surveillance and competitive black-box analysis.
pub fn pharma_boxed_warnings(
    params: PharmaBoxedWarningsParams,
) -> Result<CallToolResult, McpError> {
    let handle = match load_company(&params.company) {
        Some(h) => h,
        None => return unknown_company_error(&params.company),
    };

    let co = handle.company();
    let boxed = handle.products_with_boxed_warnings();

    let products_json: Vec<_> = boxed
        .iter()
        .map(|p| {
            json!({
                "generic_name": p.generic_name,
                "brand_names": p.brand_names,
                "rxcui": p.rxcui,
                "therapeutic_area": p.therapeutic_area.to_string(),
                "approval_year": p.approval_year,
                "rems": p.safety_profile.rems,
                "signal_count": p.safety_profile.signals.len(),
                "label_warnings": p.safety_profile.label_warnings,
            })
        })
        .collect();

    let response = json!({
        "company": co.name,
        "ticker": co.ticker,
        "total_products": co.products.len(),
        "boxed_warning_count": products_json.len(),
        "products_with_boxed_warnings": products_json,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
