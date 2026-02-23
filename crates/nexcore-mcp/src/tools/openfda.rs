//! OpenFDA MCP tools.
//!
//! Live access to FDA drug, device, food, and substance databases via the
//! OpenFDA REST API. Supports search, pagination, and fan-out across all
//! major endpoints.

use std::sync::OnceLock;

use nexcore_openfda::{OpenFdaClient, QueryParams};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::openfda::{
    OpenfdaDeviceEventsParams, OpenfdaDeviceRecallsParams, OpenfdaDrugEventsParams,
    OpenfdaDrugLabelsParams, OpenfdaDrugNdcParams, OpenfdaDrugRecallsParams,
    OpenfdaDrugsAtFdaParams, OpenfdaFanOutParams, OpenfdaFoodEventsParams,
    OpenfdaFoodRecallsParams, OpenfdaSubstancesParams,
};

// ── Client singleton ─────────────────────────────────────────────────────

static CLIENT: OnceLock<Option<OpenFdaClient>> = OnceLock::new();

fn client() -> Result<&'static OpenFdaClient, McpError> {
    let maybe = CLIENT.get_or_init(|| {
        if let Ok(key) = std::env::var("OPENFDA_API_KEY") {
            if let Ok(c) = OpenFdaClient::with_api_key(key) {
                return Some(c);
            }
        }
        OpenFdaClient::new().ok()
    });
    maybe.as_ref().ok_or_else(|| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: "OpenFDA client initialization failed".into(),
        data: None,
    })
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn make_params(
    search: &str,
    limit: Option<u32>,
    skip: Option<u32>,
    sort: Option<&str>,
) -> QueryParams {
    let mut qp = QueryParams::search(search, limit.unwrap_or(10).min(1000));
    qp.skip = skip;
    qp.sort = sort.map(|s| s.to_string());
    qp
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Search drug adverse event reports from FAERS.
pub async fn openfda_drug_events(p: OpenfdaDrugEventsParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = make_params(&p.search, p.limit, p.skip, p.sort.as_deref());
    match nexcore_openfda::endpoints::fetch_drug_events(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|e| json!({
                "safety_report_id": e.safetyreportid,
                "receipt_date": e.receiptdate,
                "serious": e.serious,
                "seriousness_death": e.seriousnessdeath,
                "seriousness_hospitalization": e.seriousnesshospitalization,
                "patient": e.patient.as_ref().map(|pat| json!({
                    "age": pat.patientonsetage,
                    "sex": pat.patientsex,
                    "drugs": pat.drug.iter().map(|d| json!({
                        "name": d.medicinalproduct,
                        "characterization": d.drugcharacterization,
                    })).collect::<Vec<_>>(),
                    "reactions": pat.reaction.iter().map(|r| json!({
                        "reaction": r.reactionmeddrapt,
                        "outcome": r.reactionoutcome,
                    })).collect::<Vec<_>>(),
                })),
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA drug events error: {e}")),
    }
}

/// Search drug product labels (SPL).
pub async fn openfda_drug_labels(p: OpenfdaDrugLabelsParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_drug_labels(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(20).map(|l| json!({
                "set_id": l.set_id,
                "effective_time": l.effective_time,
                "brand_name": l.openfda.brand_name,
                "generic_name": l.openfda.generic_name,
                "manufacturer": l.openfda.manufacturer_name,
                "boxed_warning": l.boxed_warning,
                "warnings": l.warnings,
                "indications": l.indications_and_usage,
                "adverse_reactions": l.adverse_reactions,
                "contraindications": l.contraindications,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA drug labels error: {e}")),
    }
}

/// Search drug recall enforcement actions.
pub async fn openfda_drug_recalls(p: OpenfdaDrugRecallsParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_drug_recalls(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|r| json!({
                "recall_number": r.recall_number,
                "classification": r.classification,
                "recalling_firm": r.recalling_firm,
                "reason": r.reason_for_recall,
                "product": r.product_description,
                "status": r.status,
                "date": r.recall_initiation_date,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA drug recalls error: {e}")),
    }
}

/// Search National Drug Code directory.
pub async fn openfda_drug_ndc(p: OpenfdaDrugNdcParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let search = nexcore_openfda::endpoints::drug::drug_ndc_search_by_name(&p.name);
    let qp = QueryParams::search(&search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_drug_ndc(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|n| json!({
                "product_ndc": n.product_ndc,
                "brand_name": n.brand_name,
                "generic_name": n.generic_name,
                "dosage_form": n.dosage_form,
                "route": n.route,
                "labeler": n.labeler_name,
                "active_ingredients": n.active_ingredients.iter().map(|i| json!({
                    "name": i.name,
                    "strength": i.strength,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA NDC error: {e}")),
    }
}

/// Search Drugs@FDA applications (NDA/BLA/ANDA).
pub async fn openfda_drugs_at_fda(p: OpenfdaDrugsAtFdaParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_drugs_at_fda(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(20).map(|a| json!({
                "application_number": a.application_number,
                "sponsor": a.sponsor_name,
                "products": a.products.iter().map(|p| json!({
                    "brand_name": p.brand_name,
                    "dosage_form": p.dosage_form,
                    "route": p.route,
                    "marketing_status": p.marketing_status,
                })).collect::<Vec<_>>(),
                "submissions": a.submissions.len(),
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA Drugs@FDA error: {e}")),
    }
}

/// Search medical device adverse event reports (MDR).
pub async fn openfda_device_events(
    p: OpenfdaDeviceEventsParams,
) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_device_events(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|e| json!({
                "mdr_report_key": e.mdr_report_key,
                "event_type": e.event_type,
                "date_received": e.date_received,
                "manufacturer": e.manufacturer_name,
                "devices": e.device.iter().map(|d| json!({
                    "brand_name": d.brand_name,
                    "generic_name": d.generic_name,
                    "manufacturer": d.manufacturer_d_name,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA device events error: {e}")),
    }
}

/// Search device recall enforcement actions.
pub async fn openfda_device_recalls(
    p: OpenfdaDeviceRecallsParams,
) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_device_recalls(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|r| json!({
                "recall_number": r.recall_number,
                "classification": r.classification,
                "recalling_firm": r.recalling_firm,
                "reason": r.reason_for_recall,
                "product": r.product_description,
                "status": r.status,
                "date": r.recall_initiation_date,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA device recalls error: {e}")),
    }
}

/// Search food recall enforcement actions.
pub async fn openfda_food_recalls(p: OpenfdaFoodRecallsParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_food_recalls(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|r| json!({
                "recall_number": r.recall_number,
                "classification": r.classification,
                "recalling_firm": r.recalling_firm,
                "reason": r.reason_for_recall,
                "product": r.product_description,
                "status": r.status,
                "date": r.recall_initiation_date,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA food recalls error: {e}")),
    }
}

/// Search food adverse event reports (CAERS).
pub async fn openfda_food_events(p: OpenfdaFoodEventsParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_food_events(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|e| json!({
                "report_number": e.report_number,
                "date_created": e.date_created,
                "outcomes": e.outcomes.iter().map(|o| &o.outcome).collect::<Vec<_>>(),
                "products": e.products.iter().map(|p| json!({
                    "name": p.name_brand,
                    "industry": p.industry_name,
                    "role": p.role,
                })).collect::<Vec<_>>(),
                "reactions": e.reactions.iter().map(|r| &r.reaction_coded).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA food events error: {e}")),
    }
}

/// Search FDA substance registry.
pub async fn openfda_substances(p: OpenfdaSubstancesParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let qp = QueryParams::search(&p.search, p.limit.unwrap_or(10).min(1000));
    match nexcore_openfda::endpoints::fetch_substances(c, &qp).await {
        Ok(resp) => ok_json(json!({
            "total": resp.meta.results.total,
            "returned": resp.results.len(),
            "results": resp.results.iter().take(50).map(|s| json!({
                "name": s.substance_name,
                "unii": s.unii,
                "cas": s.cas,
                "molecular_formula": s.molecular_formula,
                "molecular_weight": s.molecular_weight,
                "class": s.substance_class,
                "inchikey": s.inchikey,
                "smiles": s.smiles,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("OpenFDA substances error: {e}")),
    }
}

/// Fan-out search across all major OpenFDA endpoints simultaneously.
pub async fn openfda_fan_out(p: OpenfdaFanOutParams) -> Result<CallToolResult, McpError> {
    let c = client()?;
    let limit = p.limit.map(|l| l.min(100));
    let results = nexcore_openfda::fan_out_search(c, &p.term, limit).await;

    ok_json(json!({
        "term": p.term,
        "successful_endpoints": results.successful_endpoints(),
        "total_results": results.total_results(),
        "drug_events": results.drug_events.as_ref().map(|r| r.meta.results.total).ok(),
        "drug_labels": results.drug_labels.as_ref().map(|r| r.meta.results.total).ok(),
        "drug_recalls": results.drug_recalls.as_ref().map(|r| r.meta.results.total).ok(),
        "drug_ndc": results.drug_ndc.as_ref().map(|r| r.meta.results.total).ok(),
        "device_events": results.device_events.as_ref().map(|r| r.meta.results.total).ok(),
        "device_recalls": results.device_recalls.as_ref().map(|r| r.meta.results.total).ok(),
        "food_recalls": results.food_recalls.as_ref().map(|r| r.meta.results.total).ok(),
        "food_events": results.food_events.as_ref().map(|r| r.meta.results.total).ok(),
        "substances": results.substances.as_ref().map(|r| r.meta.results.total).ok(),
    }))
}
