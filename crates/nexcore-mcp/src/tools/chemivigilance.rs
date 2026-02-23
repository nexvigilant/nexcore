// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Chemivigilance MCP tool implementations.
//!
//! 15 tools covering SMILES parsing, molecular descriptors, Morgan fingerprints,
//! Tanimoto/Dice similarity, ICH M7 structural alert scanning, QSAR toxicity
//! prediction, Phase I/II metabolite and degradant prediction, SafetyBrief
//! generation, substructure matching, watchlist screening, and ring/aromaticity
//! analysis.

use nexcore_chemivigilance::pipeline::{ChemivigilanceConfig, generate_safety_brief};
use nexcore_chemivigilance::watchlist::{check_ich_m7_flag, check_watchlist};
use nexcore_metabolite::predict::predict_from_smiles as metabolite_predict_from_smiles;
use nexcore_molcore::arom::detect_aromaticity;
use nexcore_molcore::descriptor::calculate_descriptors;
use nexcore_molcore::fingerprint::{dice, morgan_fingerprint, tanimoto};
use nexcore_molcore::graph::MolGraph;
use nexcore_molcore::ring::find_sssr;
use nexcore_molcore::smiles::parse;
use nexcore_molcore::substruct::{count_matches, has_substructure, substructure_match};
use nexcore_qsar::predict::predict_from_smiles as qsar_predict_from_smiles;
use nexcore_structural_alerts::{AlertCategory, AlertLibrary, scan_smiles};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::chemivigilance::{
    ChemAlertLibraryParams, ChemAromaticityParams, ChemDescriptorParams, ChemFingerprintParams,
    ChemMolecularFormulaParams, ChemParseSmilesParams, ChemPredictDegradantsParams,
    ChemPredictMetabolitesParams, ChemPredictToxicityParams, ChemRingScanParams,
    ChemSafetyBriefParams, ChemSimilarityParams, ChemStructuralAlertsParams,
    ChemSubstructureParams, ChemWatchlistParams,
};

// ---------------------------------------------------------------------------
// Helper — build a text CallToolResult from a JSON value.
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

// ---------------------------------------------------------------------------
// Tool 1: chem_parse_smiles
// ---------------------------------------------------------------------------

/// Parse a SMILES string and return basic molecular graph information.
pub fn chem_parse_smiles(params: ChemParseSmilesParams) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let components = graph.connected_components();

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "atom_count": graph.atom_count(),
        "bond_count": graph.bond_count(),
        "connected_components": components.len(),
        "valid": true,
    }))
}

// ---------------------------------------------------------------------------
// Tool 2: chem_descriptors
// ---------------------------------------------------------------------------

/// Calculate Lipinski/physicochemical molecular descriptors.
pub fn chem_descriptors(params: ChemDescriptorParams) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let d = calculate_descriptors(&graph);

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "molecular_weight": d.molecular_weight,
        "logp": d.logp,
        "tpsa": d.tpsa,
        "hba": d.hba,
        "hbd": d.hbd,
        "rotatable_bonds": d.rotatable_bonds,
        "num_rings": d.num_rings,
        "num_aromatic_rings": d.num_aromatic_rings,
        "heavy_atom_count": d.heavy_atom_count,
        "lipinski_ro5": {
            "mw_ok": d.molecular_weight <= 500.0,
            "logp_ok": d.logp <= 5.0,
            "hba_ok": d.hba <= 10,
            "hbd_ok": d.hbd <= 5,
            "passes": d.molecular_weight <= 500.0 && d.logp <= 5.0
                   && d.hba <= 10 && d.hbd <= 5,
        },
    }))
}

// ---------------------------------------------------------------------------
// Tool 3: chem_fingerprint
// ---------------------------------------------------------------------------

/// Generate a Morgan/ECFP circular fingerprint.
pub fn chem_fingerprint(params: ChemFingerprintParams) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);

    let radius = params.radius.unwrap_or(2).min(255) as u8;
    let nbits = params.nbits.unwrap_or(2048);

    let fp = morgan_fingerprint(&graph, radius, nbits);

    // Collect the indices of set bits (capped at 256 for response size).
    let set_bits: Vec<usize> = (0..fp.size).filter(|&i| fp.get(i)).take(256).collect();

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "radius": radius,
        "nbits": nbits,
        "popcount": fp.popcount(),
        "density": fp.popcount() as f64 / nbits.max(1) as f64,
        "set_bits_sample": set_bits,
        "set_bits_truncated": fp.popcount() > 256,
    }))
}

// ---------------------------------------------------------------------------
// Tool 4: chem_similarity
// ---------------------------------------------------------------------------

/// Compute Tanimoto or Dice similarity between two molecules.
pub fn chem_similarity(params: ChemSimilarityParams) -> Result<CallToolResult, McpError> {
    let mol_a = match parse(&params.smiles_a) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("smiles_a parse failed: {e}")),
    };
    let mol_b = match parse(&params.smiles_b) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("smiles_b parse failed: {e}")),
    };

    let graph_a = MolGraph::from_molecule(mol_a);
    let graph_b = MolGraph::from_molecule(mol_b);

    let fp_a = morgan_fingerprint(&graph_a, 2, 2048);
    let fp_b = morgan_fingerprint(&graph_b, 2, 2048);

    let metric = params.metric.as_deref().unwrap_or("tanimoto");
    let score = match metric {
        "dice" => dice(&fp_a, &fp_b),
        _ => tanimoto(&fp_a, &fp_b),
    };

    json_result(serde_json::json!({
        "smiles_a": params.smiles_a,
        "smiles_b": params.smiles_b,
        "metric": if metric == "dice" { "dice" } else { "tanimoto" },
        "similarity": score,
        "interpretation": if score >= 0.85 {
            "very similar"
        } else if score >= 0.6 {
            "similar"
        } else if score >= 0.4 {
            "moderately similar"
        } else {
            "dissimilar"
        },
    }))
}

// ---------------------------------------------------------------------------
// Tool 5: chem_structural_alerts
// ---------------------------------------------------------------------------

/// Scan a molecule for ICH M7 structural alerts.
pub fn chem_structural_alerts(
    params: ChemStructuralAlertsParams,
) -> Result<CallToolResult, McpError> {
    let library = AlertLibrary::default_library();
    let matches = match scan_smiles(&params.smiles, &library) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("Alert scan failed: {e}")),
    };

    let alerts: Vec<serde_json::Value> = matches
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.alert.id,
                "name": m.alert.name,
                "category": format!("{:?}", m.alert.category),
                "match_count": m.match_count,
                "confidence": m.alert.confidence,
                "description": m.alert.description,
            })
        })
        .collect();

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "alert_count": matches.len(),
        "alerts": alerts,
        "has_mutagenicity_alerts": matches.iter().any(|m| m.alert.category == AlertCategory::Mutagenicity),
        "has_genotoxicity_alerts": matches.iter().any(|m| m.alert.category == AlertCategory::Genotoxicity),
        "has_carcinogenicity_alerts": matches.iter().any(|m| m.alert.category == AlertCategory::Carcinogenicity),
        "has_hepatotoxicity_alerts": matches.iter().any(|m| m.alert.category == AlertCategory::Hepatotoxicity),
    }))
}

// ---------------------------------------------------------------------------
// Tool 6: chem_predict_toxicity
// ---------------------------------------------------------------------------

/// QSAR toxicity prediction (mutagenicity, hepatotoxicity, cardiotoxicity).
pub fn chem_predict_toxicity(
    params: ChemPredictToxicityParams,
) -> Result<CallToolResult, McpError> {
    let profile = match qsar_predict_from_smiles(&params.smiles, 0, 0) {
        Ok(p) => p,
        Err(e) => return error_result(&format!("QSAR prediction failed: {e}")),
    };

    let domain_str = match &profile.applicability_domain {
        nexcore_qsar::types::DomainStatus::InDomain { confidence } => {
            format!("in_domain (confidence={confidence:.2})")
        }
        nexcore_qsar::types::DomainStatus::OutOfDomain { warning, .. } => {
            format!("out_of_domain ({warning})")
        }
        nexcore_qsar::types::DomainStatus::Borderline { warning, .. } => {
            format!("borderline ({warning})")
        }
    };

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "mutagenicity": {
            "probability": profile.mutagenicity.probability,
            "classification": format!("{:?}", profile.mutagenicity.classification),
            "confidence": profile.mutagenicity.confidence,
            "model_version": profile.mutagenicity.model_version,
        },
        "hepatotoxicity": {
            "probability": profile.hepatotoxicity.probability,
            "classification": format!("{:?}", profile.hepatotoxicity.classification),
            "confidence": profile.hepatotoxicity.confidence,
            "model_version": profile.hepatotoxicity.model_version,
        },
        "cardiotoxicity": {
            "probability": profile.cardiotoxicity.probability,
            "classification": format!("{:?}", profile.cardiotoxicity.classification),
            "confidence": profile.cardiotoxicity.confidence,
            "model_version": profile.cardiotoxicity.model_version,
        },
        "overall_risk": format!("{:?}", profile.overall_risk),
        "applicability_domain": domain_str,
    }))
}

// ---------------------------------------------------------------------------
// Tool 7: chem_predict_metabolites
// ---------------------------------------------------------------------------

/// Predict Phase I/II metabolites and reactive intermediates.
pub fn chem_predict_metabolites(
    params: ChemPredictMetabolitesParams,
) -> Result<CallToolResult, McpError> {
    let tree = match metabolite_predict_from_smiles(&params.smiles) {
        Ok(t) => t,
        Err(e) => return error_result(&format!("Metabolite prediction failed: {e}")),
    };

    let phase1: Vec<serde_json::Value> = tree
        .phase1
        .iter()
        .map(|m| {
            serde_json::json!({
                "transformation": format!("{:?}", m.transformation),
                "site_description": m.site_description,
                "probability": m.probability,
                "reactive_intermediate": m.reactive_intermediate,
                "enzyme": m.enzyme,
            })
        })
        .collect();

    let phase2: Vec<serde_json::Value> = tree
        .phase2
        .iter()
        .map(|m| {
            serde_json::json!({
                "transformation": format!("{:?}", m.transformation),
                "site_description": m.site_description,
                "probability": m.probability,
                "enzyme": m.enzyme,
            })
        })
        .collect();

    let reactive: Vec<serde_json::Value> = tree
        .reactive_intermediates
        .iter()
        .map(|m| {
            serde_json::json!({
                "transformation": format!("{:?}", m.transformation),
                "site_description": m.site_description,
                "probability": m.probability,
            })
        })
        .collect();

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "phase1_count": tree.phase1.len(),
        "phase2_count": tree.phase2.len(),
        "reactive_intermediate_count": tree.reactive_intermediates.len(),
        "degradant_count": tree.degradants.len(),
        "phase1": phase1,
        "phase2": phase2,
        "reactive_intermediates": reactive,
    }))
}

// ---------------------------------------------------------------------------
// Tool 8: chem_predict_degradants
// ---------------------------------------------------------------------------

/// Predict degradation products only (hydrolysis, oxidation, photolysis).
pub fn chem_predict_degradants(
    params: ChemPredictDegradantsParams,
) -> Result<CallToolResult, McpError> {
    let tree = match metabolite_predict_from_smiles(&params.smiles) {
        Ok(t) => t,
        Err(e) => return error_result(&format!("Degradant prediction failed: {e}")),
    };

    let degradants: Vec<serde_json::Value> = tree
        .degradants
        .iter()
        .map(|m| {
            serde_json::json!({
                "transformation": format!("{:?}", m.transformation),
                "site_description": m.site_description,
                "probability": m.probability,
                "enzyme": m.enzyme,
            })
        })
        .collect();

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "degradant_count": tree.degradants.len(),
        "degradants": degradants,
    }))
}

// ---------------------------------------------------------------------------
// Tool 9: chem_safety_brief
// ---------------------------------------------------------------------------

/// Run the full chemivigilance pipeline and return a SafetyBrief.
pub fn chem_safety_brief(params: ChemSafetyBriefParams) -> Result<CallToolResult, McpError> {
    let config = ChemivigilanceConfig::default();
    let brief = match generate_safety_brief(&params.smiles, &config) {
        Ok(b) => b,
        Err(e) => return error_result(&format!("Safety brief generation failed: {e}")),
    };

    let value = match serde_json::to_value(&brief) {
        Ok(v) => v,
        Err(e) => return error_result(&format!("Serialization failed: {e}")),
    };

    json_result(value)
}

// ---------------------------------------------------------------------------
// Tool 10: chem_substructure
// ---------------------------------------------------------------------------

/// VF2 substructure matching — check if a pattern occurs in a molecule.
pub fn chem_substructure(params: ChemSubstructureParams) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.molecule) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("molecule parse failed: {e}")),
    };
    let pat = match parse(&params.pattern) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("pattern parse failed: {e}")),
    };

    let mol_graph = MolGraph::from_molecule(mol);
    let pat_graph = MolGraph::from_molecule(pat);

    let found = has_substructure(&mol_graph, &pat_graph);
    let match_count = count_matches(&mol_graph, &pat_graph);

    // First mapping (if any) for inspection.
    let first_mapping: Vec<serde_json::Value> = if found {
        let all = substructure_match(&mol_graph, &pat_graph);
        all.into_iter()
            .next()
            .unwrap_or_default()
            .into_iter()
            .map(|(p, m)| serde_json::json!({ "pattern_atom": p, "molecule_atom": m }))
            .collect()
    } else {
        Vec::new()
    };

    json_result(serde_json::json!({
        "molecule": params.molecule,
        "pattern": params.pattern,
        "matches": found,
        "match_count": match_count,
        "first_mapping": first_mapping,
    }))
}

// ---------------------------------------------------------------------------
// Tool 11: chem_watchlist
// ---------------------------------------------------------------------------

/// Regulatory watchlist screening — quick flag check for a molecule.
pub fn chem_watchlist(params: ChemWatchlistParams) -> Result<CallToolResult, McpError> {
    // Run structural alerts.
    let library = AlertLibrary::default_library();
    let alert_matches = match scan_smiles(&params.smiles, &library) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("Alert scan failed: {e}")),
    };

    // Run QSAR.
    let alert_count = alert_matches.len();
    let profile = match qsar_predict_from_smiles(&params.smiles, alert_count, 0) {
        Ok(p) => p,
        Err(e) => return error_result(&format!("QSAR prediction failed: {e}")),
    };

    // Run metabolite prediction.
    let tree = match metabolite_predict_from_smiles(&params.smiles) {
        Ok(t) => t,
        Err(e) => return error_result(&format!("Metabolite prediction failed: {e}")),
    };

    // Build alert summaries for ICH M7 flag check.
    let alert_summaries: Vec<nexcore_chemivigilance::AlertSummary> = alert_matches
        .iter()
        .map(|m| nexcore_chemivigilance::AlertSummary {
            alert_id: m.alert.id.clone(),
            alert_name: m.alert.name.clone(),
            category: format!("{:?}", m.alert.category),
            match_count: m.match_count,
            confidence: m.alert.confidence,
        })
        .collect();

    let mut flags = check_watchlist(alert_count, &profile, &tree);
    let ich_flags = check_ich_m7_flag(&alert_summaries);
    flags.extend(ich_flags);

    let flag_json: Vec<serde_json::Value> = flags
        .iter()
        .map(|f| {
            serde_json::json!({
                "flag_type": format!("{:?}", f.flag_type),
                "description": f.description,
                "reference": f.reference,
            })
        })
        .collect();

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "alert_count": alert_count,
        "flag_count": flags.len(),
        "flags": flag_json,
        "overall_risk": format!("{:?}", profile.overall_risk),
    }))
}

// ---------------------------------------------------------------------------
// Tool 12: chem_alert_library
// ---------------------------------------------------------------------------

/// Query the built-in ICH M7 structural alert library.
pub fn chem_alert_library(params: ChemAlertLibraryParams) -> Result<CallToolResult, McpError> {
    let library = AlertLibrary::default_library();

    // Normalise the category filter (case-insensitive prefix match).
    let filtered = match params.category.as_deref() {
        None => library.alerts().to_vec(),
        Some(cat) => {
            let cat_lower = cat.to_lowercase();
            library
                .alerts()
                .iter()
                .filter(|a| {
                    let cat_str = format!("{:?}", a.category).to_lowercase();
                    cat_str.contains(&cat_lower)
                })
                .cloned()
                .collect()
        }
    };

    let alerts: Vec<serde_json::Value> = filtered
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "name": a.name,
                "smiles_pattern": a.smiles_pattern,
                "category": format!("{:?}", a.category),
                "source": format!("{:?}", a.source),
                "confidence": a.confidence,
                "description": a.description,
            })
        })
        .collect();

    json_result(serde_json::json!({
        "total_in_library": library.len(),
        "returned": filtered.len(),
        "category_filter": params.category,
        "alerts": alerts,
    }))
}

// ---------------------------------------------------------------------------
// Tool 13: chem_ring_scan
// ---------------------------------------------------------------------------

/// SSSR ring perception — find all smallest rings in the molecule.
pub fn chem_ring_scan(params: ChemRingScanParams) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let rings = find_sssr(&graph);

    let ring_sizes: Vec<usize> = rings.iter().map(|r| r.len()).collect();

    // Aromatic rings for context.
    let aromatic = detect_aromaticity(&graph);

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "ring_count": rings.len(),
        "aromatic_ring_count": aromatic.len(),
        "ring_sizes": ring_sizes,
        "rings": rings,
    }))
}

// ---------------------------------------------------------------------------
// Tool 14: chem_aromaticity
// ---------------------------------------------------------------------------

/// Hückel aromaticity detection — identify aromatic rings and π-electron counts.
pub fn chem_aromaticity(params: ChemAromaticityParams) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };
    let graph = MolGraph::from_molecule(mol);
    let aromatic_rings = detect_aromaticity(&graph);

    let rings_json: Vec<serde_json::Value> = aromatic_rings
        .iter()
        .enumerate()
        .map(|(i, r)| {
            serde_json::json!({
                "ring_index": i,
                "atom_indices": r.atoms,
                "ring_size": r.atoms.len(),
                "pi_electrons": r.pi_electrons,
            })
        })
        .collect();

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "aromatic_ring_count": aromatic_rings.len(),
        "aromatic_rings": rings_json,
    }))
}

// ---------------------------------------------------------------------------
// Tool 15: chem_molecular_formula
// ---------------------------------------------------------------------------

/// Compute Hill-notation molecular formula and molecular weight.
pub fn chem_molecular_formula(
    params: ChemMolecularFormulaParams,
) -> Result<CallToolResult, McpError> {
    let mol = match parse(&params.smiles) {
        Ok(m) => m,
        Err(e) => return error_result(&format!("SMILES parse failed: {e}")),
    };

    let molecular_weight = mol.molecular_weight();

    // Collect element counts (Hill order: C first, H second, then alphabetical).
    let mut counts: std::collections::BTreeMap<u8, usize> = std::collections::BTreeMap::new();
    for atom in &mol.atoms {
        *counts.entry(atom.atomic_number).or_insert(0) += 1;
        *counts.entry(1).or_insert(0) += usize::from(atom.implicit_h);
    }

    let element_symbol = |an: u8| -> &'static str {
        match an {
            1 => "H",
            6 => "C",
            7 => "N",
            8 => "O",
            9 => "F",
            15 => "P",
            16 => "S",
            17 => "Cl",
            35 => "Br",
            53 => "I",
            _ => "?",
        }
    };

    let hill_order: &[u8] = &[6, 1, 7, 8, 9, 15, 16, 17, 35, 53];
    let mut formula = String::new();
    for &an in hill_order {
        if let Some(&count) = counts.get(&an) {
            formula.push_str(element_symbol(an));
            if count > 1 {
                formula.push_str(&count.to_string());
            }
        }
    }
    if formula.is_empty() {
        formula = "unknown".to_string();
    }

    json_result(serde_json::json!({
        "smiles": params.smiles,
        "molecular_formula": formula,
        "molecular_weight": molecular_weight,
        "atom_counts": counts.iter().map(|(&an, &count)| {
            serde_json::json!({
                "element": element_symbol(an),
                "atomic_number": an,
                "count": count,
            })
        }).collect::<Vec<_>>(),
    }))
}
