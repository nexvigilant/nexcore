//! Compliance tools: SAM.gov exclusions, OSCAL-inspired assessment, ICH controls
//!
//! Federal compliance integration for pharmacovigilance systems.

use crate::params::{
    ComplianceAssessParams, ComplianceCatalogParams, ComplianceCheckExclusionParams,
    ComplianceSecFilingsParams, ComplianceSecPharmaParams,
};
use nexcore_compliance::dsl::{Assessment, Finding, FindingSeverity};
use nexcore_compliance::oscal::{Control, ControlCatalog, ControlStatus};
use nexcore_compliance::sam::{ExclusionQuery, SamClient};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Check SAM.gov for entity exclusions (debarment, suspension, etc.)
pub async fn check_exclusion(
    params: ComplianceCheckExclusionParams,
) -> Result<CallToolResult, McpError> {
    // Get API key from environment
    let api_key = std::env::var("SAM_GOV_API_KEY").unwrap_or_default();

    if api_key.is_empty() {
        let json = json!({
            "error": "SAM_GOV_API_KEY not set",
            "note": "Get API key from https://api.sam.gov/. Free tier: 1000 requests/day.",
            "instructions": "Set SAM_GOV_API_KEY environment variable",
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    let client = match SamClient::new(api_key) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = e.to_string();
            let json = json!({
                "error": "Failed to create SAM client",
                "details": err_msg,
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    // Build query based on provided identifiers
    let mut query = ExclusionQuery::new();

    if let Some(uei) = &params.uei {
        query = query.uei(uei);
    }
    if let Some(cage) = &params.cage_code {
        query = query.cage_code(cage);
    }
    if let Some(name) = &params.entity_name {
        query = query.name(name);
    }

    match client.query_exclusions(&query).await {
        Ok(response) => {
            let has_active = response.exclusion_data.iter().any(|e| e.is_active());

            let exclusions: Vec<serde_json::Value> = response
                .exclusion_data
                .iter()
                .map(|e| {
                    json!({
                        "uei": e.uei_sam,
                        "entity_name": e.name,
                        "classification": e.classification,
                        "exclusion_type": e.exclusion_type,
                        "activation_date": e.activation_date,
                        "termination_date": e.termination_date,
                        "is_active": e.is_active(),
                        "risk_score": e.risk_score(),
                        "excluding_agency": e.excluding_agency_name,
                    })
                })
                .collect();

            let json = json!({
                "query": {
                    "uei": params.uei,
                    "cage_code": params.cage_code,
                    "entity_name": params.entity_name,
                },
                "total_records": response.total_records,
                "exclusions": exclusions,
                "has_active_exclusion": has_active,
                "recommendation": if has_active {
                    "DO NOT PROCEED - Active federal exclusion found"
                } else if exclusions.is_empty() {
                    "CLEAR - No exclusions found"
                } else {
                    "REVIEW REQUIRED - Historical exclusions exist"
                },
            });

            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let err_msg = e.to_string();
            let json = json!({
                "error": "SAM.gov query failed",
                "details": err_msg,
                "query": {
                    "uei": params.uei,
                    "cage_code": params.cage_code,
                    "entity_name": params.entity_name,
                },
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Run compliance assessment on controls
pub fn assess(params: ComplianceAssessParams) -> Result<CallToolResult, McpError> {
    let mut assessment = Assessment::new(&params.assessment_id);

    // Add controls from the request
    for ctrl in &params.controls {
        let status = match ctrl.status.to_lowercase().as_str() {
            "implemented" => ControlStatus::Implemented,
            "partial" => ControlStatus::Partial,
            "not_applicable" | "na" => ControlStatus::NotApplicable,
            _ => ControlStatus::NotImplemented,
        };

        assessment.add_control(Control {
            id: ctrl.id.clone(),
            title: ctrl.title.clone(),
            description: ctrl.description.clone().unwrap_or_default(),
            catalog: ctrl.catalog.clone().unwrap_or_else(|| "Custom".to_string()),
            status,
        });
    }

    // Add any findings
    for finding in &params.findings {
        let severity = match finding.severity.to_lowercase().as_str() {
            "critical" => FindingSeverity::Critical,
            "high" => FindingSeverity::High,
            "medium" => FindingSeverity::Medium,
            "low" => FindingSeverity::Low,
            _ => FindingSeverity::Info,
        };

        assessment.add_finding(Finding {
            control_id: finding.control_id.clone(),
            severity,
            title: finding.title.clone(),
            description: finding.description.clone(),
            remediation: finding.remediation.clone(),
        });
    }

    // Evaluate result
    assessment.evaluate();

    let json = json!({
        "assessment_id": assessment.id,
        "result": assessment.result.map(|r| match r {
            nexcore_compliance::dsl::ComplianceResult::Compliant => "Compliant",
            nexcore_compliance::dsl::ComplianceResult::NonCompliant => "NonCompliant",
            nexcore_compliance::dsl::ComplianceResult::Inconclusive => "Inconclusive",
        }),
        "controls_count": assessment.controls.len(),
        "findings_count": assessment.findings.len(),
        "findings_by_severity": {
            "critical": assessment.finding_count(FindingSeverity::Critical),
            "high": assessment.finding_count(FindingSeverity::High),
            "medium": assessment.finding_count(FindingSeverity::Medium),
            "low": assessment.finding_count(FindingSeverity::Low),
            "info": assessment.finding_count(FindingSeverity::Info),
        },
        "controls": assessment.controls.iter().map(|c| json!({
            "id": c.id,
            "title": c.title,
            "catalog": c.catalog,
            "status": match c.status {
                ControlStatus::NotImplemented => "NotImplemented",
                ControlStatus::Partial => "Partial",
                ControlStatus::Implemented => "Implemented",
                ControlStatus::NotApplicable => "NotApplicable",
            },
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get pre-populated ICH control catalog
pub fn catalog_ich(params: ComplianceCatalogParams) -> Result<CallToolResult, McpError> {
    let mut catalog = ControlCatalog::new("ICH Guidelines");

    // ICH E2A: Clinical Safety Data Management
    catalog.add_control(Control {
        id: "ICH-E2A-1".to_string(),
        title: "ICSR Collection".to_string(),
        description: "Systems for collecting individual case safety reports".to_string(),
        catalog: "ICH-E2A".to_string(),
        status: ControlStatus::NotImplemented,
    });
    catalog.add_control(Control {
        id: "ICH-E2A-2".to_string(),
        title: "Expedited Reporting".to_string(),
        description: "15-day expedited reporting for serious unexpected ADRs".to_string(),
        catalog: "ICH-E2A".to_string(),
        status: ControlStatus::NotImplemented,
    });
    catalog.add_control(Control {
        id: "ICH-E2A-3".to_string(),
        title: "Periodic Reporting".to_string(),
        description: "Periodic safety update reports (PSURs)".to_string(),
        catalog: "ICH-E2A".to_string(),
        status: ControlStatus::NotImplemented,
    });

    // ICH E2B: Electronic Transmission of ICSRs
    catalog.add_control(Control {
        id: "ICH-E2B-1".to_string(),
        title: "E2B(R3) Format".to_string(),
        description: "ICSR data elements in ICH E2B(R3) format".to_string(),
        catalog: "ICH-E2B".to_string(),
        status: ControlStatus::NotImplemented,
    });
    catalog.add_control(Control {
        id: "ICH-E2B-2".to_string(),
        title: "Electronic Gateway".to_string(),
        description: "Electronic gateway connectivity to regulatory authorities".to_string(),
        catalog: "ICH-E2B".to_string(),
        status: ControlStatus::NotImplemented,
    });

    // ICH E2C: Periodic Safety Update Reports
    catalog.add_control(Control {
        id: "ICH-E2C-1".to_string(),
        title: "PSUR Format".to_string(),
        description: "PSUR format and content per ICH E2C(R2)".to_string(),
        catalog: "ICH-E2C".to_string(),
        status: ControlStatus::NotImplemented,
    });
    catalog.add_control(Control {
        id: "ICH-E2C-2".to_string(),
        title: "Signal Detection".to_string(),
        description: "Systematic signal detection methodology".to_string(),
        catalog: "ICH-E2C".to_string(),
        status: ControlStatus::NotImplemented,
    });
    catalog.add_control(Control {
        id: "ICH-E2C-3".to_string(),
        title: "Benefit-Risk Evaluation".to_string(),
        description: "Integrated benefit-risk evaluation for authorized products".to_string(),
        catalog: "ICH-E2C".to_string(),
        status: ControlStatus::NotImplemented,
    });

    // ICH E2D: Post-Approval Safety Data Management
    catalog.add_control(Control {
        id: "ICH-E2D-1".to_string(),
        title: "Solicited Reports".to_string(),
        description: "Handling of solicited reports from organized data collection".to_string(),
        catalog: "ICH-E2D".to_string(),
        status: ControlStatus::NotImplemented,
    });
    catalog.add_control(Control {
        id: "ICH-E2D-2".to_string(),
        title: "Literature Monitoring".to_string(),
        description: "Scientific literature monitoring for safety information".to_string(),
        catalog: "ICH-E2D".to_string(),
        status: ControlStatus::NotImplemented,
    });

    // ICH E2E: Pharmacovigilance Planning
    catalog.add_control(Control {
        id: "ICH-E2E-1".to_string(),
        title: "Safety Specification".to_string(),
        description: "Summary of safety profile and important identified/potential risks"
            .to_string(),
        catalog: "ICH-E2E".to_string(),
        status: ControlStatus::NotImplemented,
    });
    catalog.add_control(Control {
        id: "ICH-E2E-2".to_string(),
        title: "Pharmacovigilance Plan".to_string(),
        description: "Proposed actions for safety concerns and missing information".to_string(),
        catalog: "ICH-E2E".to_string(),
        status: ControlStatus::NotImplemented,
    });

    // Filter by guideline if specified
    let controls: Vec<_> = if let Some(filter) = &params.guideline_filter {
        catalog
            .controls
            .iter()
            .filter(|c| c.catalog.to_lowercase().contains(&filter.to_lowercase()))
            .cloned()
            .collect()
    } else {
        catalog.controls.clone()
    };

    let json = json!({
        "catalog": "ICH Guidelines",
        "filter": params.guideline_filter,
        "controls_count": controls.len(),
        "compliance_percentage": catalog.compliance_percentage(),
        "guidelines_covered": ["ICH-E2A", "ICH-E2B", "ICH-E2C", "ICH-E2D", "ICH-E2E"],
        "controls": controls.iter().map(|c| json!({
            "id": c.id,
            "title": c.title,
            "description": c.description,
            "catalog": c.catalog,
            "status": match c.status {
                ControlStatus::NotImplemented => "NotImplemented",
                ControlStatus::Partial => "Partial",
                ControlStatus::Implemented => "Implemented",
                ControlStatus::NotApplicable => "NotApplicable",
            },
        })).collect::<Vec<_>>(),
        "usage": "Use compliance_assess to evaluate control implementation status",
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get SEC EDGAR filings for a company by CIK
pub async fn sec_filings(params: ComplianceSecFilingsParams) -> Result<CallToolResult, McpError> {
    let client = match nexcore_compliance::sec::SecClient::new() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = e.to_string();
            let json = json!({
                "error": "Failed to create SEC client",
                "details": err_msg,
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    match client.get_submissions(&params.cik).await {
        Ok(submissions) => {
            let mut filings = nexcore_compliance::sec::SecClient::extract_filings(&submissions);

            // Filter by form type if specified
            if let Some(form) = &params.form_filter {
                filings =
                    nexcore_compliance::sec::SecClient::filter_by_form(&filings, &[form.as_str()]);
            }

            // Limit results
            let limit = params.limit.unwrap_or(20);
            filings.truncate(limit);

            let json = json!({
                "company": {
                    "cik": submissions.cik,
                    "name": submissions.name,
                    "tickers": submissions.tickers,
                    "sic": submissions.sic,
                    "sic_description": submissions.sic_description,
                },
                "filings_count": filings.len(),
                "form_filter": params.form_filter,
                "filings": filings.iter().map(|f| json!({
                    "accession_number": f.accession_number,
                    "form": f.form,
                    "filing_date": f.filing_date,
                    "report_date": f.report_date,
                    "document": f.primary_document,
                    "description": f.description,
                })).collect::<Vec<_>>(),
            });

            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let err_msg = e.to_string();
            let json = json!({
                "error": "SEC EDGAR query failed",
                "details": err_msg,
                "cik": params.cik,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Get SEC filings for known pharma companies
pub async fn sec_pharma(params: ComplianceSecPharmaParams) -> Result<CallToolResult, McpError> {
    use nexcore_compliance::sec::pharma_ciks;

    let cik = match params.company.to_lowercase().as_str() {
        "pfizer" => pharma_ciks::PFIZER,
        "jnj" | "johnson" => pharma_ciks::JNJ,
        "merck" => pharma_ciks::MERCK,
        "abbvie" => pharma_ciks::ABBVIE,
        "bms" | "bristol" => pharma_ciks::BMS,
        "lilly" | "eli" => pharma_ciks::LILLY,
        "amgen" => pharma_ciks::AMGEN,
        "gilead" => pharma_ciks::GILEAD,
        "regeneron" => pharma_ciks::REGENERON,
        "moderna" => pharma_ciks::MODERNA,
        _ => {
            let json = json!({
                "error": "Unknown pharma company",
                "company": params.company,
                "available": ["pfizer", "jnj", "merck", "abbvie", "bms", "lilly", "amgen", "gilead", "regeneron", "moderna"],
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    // Reuse sec_filings with 10-K filter
    sec_filings(ComplianceSecFilingsParams {
        cik: cik.to_string(),
        form_filter: Some("10-K".to_string()),
        limit: Some(5),
    })
    .await
}
