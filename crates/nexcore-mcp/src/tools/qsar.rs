// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Granular QSAR per-endpoint MCP tool implementations.
//!
//! 4 tools exposing individual QSAR toxicity endpoints and domain assessment:
//!
//! | Tool | Endpoint |
//! |------|----------|
//! | [`chem_predict_mutagenicity`] | Ames-surrogate mutagenicity |
//! | [`chem_predict_hepatotoxicity`] | Drug-induced liver injury (DILI) |
//! | [`chem_predict_cardiotoxicity`] | hERG-channel QT prolongation |
//! | [`chem_assess_applicability_domain`] | Descriptor bounding-box domain check |
//!
//! Each tool accepts a SMILES string and internally handles parsing → graph →
//! descriptors → prediction, reusing the same helpers as the existing
//! `chemivigilance` tools.

use nexcore_molcore::descriptor::calculate_descriptors;
use nexcore_molcore::graph::MolGraph;
use nexcore_molcore::smiles::parse;
use nexcore_qsar::applicability::assess_domain;
use nexcore_qsar::cardiotoxicity::predict_cardiotoxicity;
use nexcore_qsar::hepatotoxicity::predict_hepatotoxicity;
use nexcore_qsar::mutagenicity::predict_mutagenicity;
use nexcore_qsar::types::DomainStatus;
use nexcore_structural_alerts::{AlertCategory, AlertLibrary, scan_smiles};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::qsar::{
    ChemAssessDomainParams, ChemPredictCardiotoxicityParams, ChemPredictHepatotoxicityParams,
    ChemPredictMutagenicityParams,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn json_result(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_default(),
    )]))
}

fn error_result(message: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        format!("Error: {message}"),
    )]))
}

/// Classify risk level from probability for human-readable output.
fn risk_label(probability: f64) -> &'static str {
    if probability >= 0.7 {
        "very_high"
    } else if probability >= 0.5 {
        "high"
    } else if probability >= 0.3 {
        "medium"
    } else {
        "low"
    }
}

/// Count alerts matching a specific category from the default ICH M7 library.
///
/// Returns `None` if the scan fails (caller falls back to error_result).
fn count_alerts_by_category(smiles: &str, category: AlertCategory) -> Option<usize> {
    let library = AlertLibrary::default_library();
    let matches = scan_smiles(smiles, &library).ok()?;
    Some(
        matches
            .iter()
            .filter(|m| m.alert.category == category)
            .count(),
    )
}

// ---------------------------------------------------------------------------
// Tool 1: chem_predict_mutagenicity
// ---------------------------------------------------------------------------

/// Per-endpoint mutagenicity prediction (Ames-surrogate).
///
/// Parses SMILES → MolGraph → descriptors, auto-scans structural alerts
/// (unless overridden), and returns the mutagenicity prediction with
/// probability, classification, confidence, and risk level.
pub fn chem_predict_mutagenicity(
    params: ChemPredictMutagenicityParams,
) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let descriptors = calculate_descriptors(&graph);

    // Use caller-provided alert count or auto-scan.
    let alert_count = match params.structural_alert_count {
        Some(n) => n,
        None => match count_alerts_by_category(&params.smiles, AlertCategory::Mutagenicity) {
            Some(n) => n,
            None => return error_result("Structural alert scan failed for mutagenicity"),
        },
    };

    let result = predict_mutagenicity(&descriptors, alert_count);

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "endpoint": "mutagenicity",
        "probability": result.probability,
        "classification": format!("{:?}", result.classification),
        "confidence": result.confidence,
        "in_domain": result.in_domain,
        "model_version": result.model_version,
        "risk_level": risk_label(result.probability),
        "structural_alert_count": alert_count,
        "descriptors_used": {
            "molecular_weight": descriptors.molecular_weight,
            "logp": descriptors.logp,
            "num_aromatic_rings": descriptors.num_aromatic_rings,
        },
    }))
}

// ---------------------------------------------------------------------------
// Tool 2: chem_predict_hepatotoxicity
// ---------------------------------------------------------------------------

/// Per-endpoint hepatotoxicity (DILI) prediction.
///
/// Parses SMILES → MolGraph → descriptors, auto-scans reactive metabolite
/// alerts (unless overridden), and returns the hepatotoxicity prediction.
pub fn chem_predict_hepatotoxicity(
    params: ChemPredictHepatotoxicityParams,
) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let descriptors = calculate_descriptors(&graph);

    // Use caller-provided reactive alert count or auto-scan hepatotoxicity alerts.
    let reactive_alerts = match params.reactive_alert_count {
        Some(n) => n,
        None => match count_alerts_by_category(&params.smiles, AlertCategory::Hepatotoxicity) {
            Some(n) => n,
            None => return error_result("Structural alert scan failed for hepatotoxicity"),
        },
    };

    let result = predict_hepatotoxicity(&descriptors, reactive_alerts);

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "endpoint": "hepatotoxicity",
        "probability": result.probability,
        "classification": format!("{:?}", result.classification),
        "confidence": result.confidence,
        "in_domain": result.in_domain,
        "model_version": result.model_version,
        "risk_level": risk_label(result.probability),
        "reactive_alert_count": reactive_alerts,
        "descriptors_used": {
            "molecular_weight": descriptors.molecular_weight,
            "logp": descriptors.logp,
            "tpsa": descriptors.tpsa,
        },
    }))
}

// ---------------------------------------------------------------------------
// Tool 3: chem_predict_cardiotoxicity
// ---------------------------------------------------------------------------

/// Per-endpoint hERG-channel cardiotoxicity prediction.
///
/// Parses SMILES → MolGraph → descriptors and returns the cardiotoxicity
/// (QT prolongation risk) prediction. No structural alerts needed for this
/// descriptor-only endpoint.
pub fn chem_predict_cardiotoxicity(
    params: ChemPredictCardiotoxicityParams,
) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let descriptors = calculate_descriptors(&graph);

    let result = predict_cardiotoxicity(&descriptors);

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "endpoint": "cardiotoxicity",
        "probability": result.probability,
        "classification": format!("{:?}", result.classification),
        "confidence": result.confidence,
        "in_domain": result.in_domain,
        "model_version": result.model_version,
        "risk_level": risk_label(result.probability),
        "descriptors_used": {
            "molecular_weight": descriptors.molecular_weight,
            "logp": descriptors.logp,
            "tpsa": descriptors.tpsa,
            "num_aromatic_rings": descriptors.num_aromatic_rings,
        },
    }))
}

// ---------------------------------------------------------------------------
// Tool 4: chem_assess_applicability_domain
// ---------------------------------------------------------------------------

/// Standalone applicability domain assessment.
///
/// Checks whether a compound falls within the descriptor bounding-box of the
/// QSAR training set without running full toxicity predictions. Returns
/// domain status, confidence, any violated boundaries, and the descriptor
/// values used for the assessment.
pub fn chem_assess_applicability_domain(
    params: ChemAssessDomainParams,
) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let descriptors = calculate_descriptors(&graph);
    let status = assess_domain(&descriptors);

    let (domain_status, confidence, warning) = match &status {
        DomainStatus::InDomain { confidence } => ("in_domain", *confidence, String::new()),
        DomainStatus::Borderline {
            confidence,
            warning,
        } => ("borderline", *confidence, warning.clone()),
        DomainStatus::OutOfDomain { warning, .. } => {
            ("out_of_domain", 0.0, warning.clone())
        }
    };

    let violations_count = match &status {
        DomainStatus::InDomain { .. } => 0,
        DomainStatus::Borderline { .. } => 1,
        DomainStatus::OutOfDomain { distance, .. } => *distance as usize,
    };

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "domain_status": domain_status,
        "confidence": confidence,
        "warning": if warning.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(warning) },
        "violations_count": violations_count,
        "boundaries": {
            "mw_range": "100–800 Da",
            "logp_range": "−3 to 8",
            "tpsa_max": "250 Å²",
            "hba_max": 15,
            "hbd_max": 8,
        },
        "descriptors": {
            "molecular_weight": descriptors.molecular_weight,
            "logp": descriptors.logp,
            "tpsa": descriptors.tpsa,
            "hba": descriptors.hba,
            "hbd": descriptors.hbd,
            "rotatable_bonds": descriptors.rotatable_bonds,
            "num_rings": descriptors.num_rings,
            "heavy_atom_count": descriptors.heavy_atom_count,
        },
    }))
}
