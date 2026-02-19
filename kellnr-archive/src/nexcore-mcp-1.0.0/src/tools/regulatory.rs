//! Regulatory Primitives tools: extract, audit, compare
//!
//! MCP tools for FDA/ICH/CIOMS regulatory primitive extraction and audit.
//! Built on T1 primitives: SEQUENCE, MAPPING, RECURSION, STATE, CLASSIFICATION.

use crate::params::{RegulatoryAuditParams, RegulatoryCompareParams, RegulatoryExtractParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// T1 PRIMITIVE: STATE - Regulatory term definitions
// ============================================================================

/// T1/T2/T3 primitive classification for regulatory terminology
///
/// Tier: T2-P (Cross-domain primitive classification)
/// Grounds to: T1 primitive `u8` via explicit discriminants
/// Ord: Implemented (tier ordering)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrimitiveTier {
    /// T1: Universal primitives (sequence, mapping, recursion, state)
    T1Universal = 1,
    /// T2-P: Cross-domain primitives
    T2Primitive = 2,
    /// T2-C: Cross-domain composite patterns
    T2Composite = 3,
    /// T3: Domain-specific terminology
    T3DomainSpecific = 4,
}

/// Quantified code for PrimitiveTier.
///
/// Tier: T2-P (Cross-domain primitive code)
/// Grounds to: T1 primitive `u8`
/// Ord: Implemented (numeric code ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrimitiveTierCode(pub u8);

impl From<PrimitiveTier> for PrimitiveTierCode {
    fn from(value: PrimitiveTier) -> Self {
        PrimitiveTierCode(value as u8)
    }
}

impl PrimitiveTier {
    fn as_str(&self) -> &'static str {
        match self {
            PrimitiveTier::T1Universal => "T1-Universal",
            PrimitiveTier::T2Primitive => "T2-P",
            PrimitiveTier::T2Composite => "T2-C",
            PrimitiveTier::T3DomainSpecific => "T3-Domain",
        }
    }
}

/// Extracted primitive with provenance
///
/// Tier: T3 (Domain-specific regulatory primitive)
/// Grounds to T1 Concepts via String and Vec fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone)]
struct ExtractedPrimitive {
    name: String,
    tier: PrimitiveTier,
    definition: String,
    source: String,
    components: Vec<String>,
}

// ============================================================================
// T1 PRIMITIVE: MAPPING - Source classification
// ============================================================================

fn classify_source(source: &str) -> &'static str {
    let lower = source.to_lowercase();
    if lower.contains("fda") || lower.contains("314.80") || lower.contains("cfr") {
        "FDA"
    } else if lower.contains("ich") || lower.contains("e2a") || lower.contains("e2b") {
        "ICH"
    } else if lower.contains("cioms") {
        "CIOMS"
    } else if lower.contains("ema") || lower.contains("gvp") {
        "EMA"
    } else {
        "UNKNOWN"
    }
}

// ============================================================================
// T1 PRIMITIVE: CLASSIFICATION - Term tier assignment
// ============================================================================

fn classify_tier(term: &str, definition: &str) -> PrimitiveTier {
    let lower_term = term.to_lowercase();
    let lower_def = definition.to_lowercase();

    // T1 Universal: foundational concepts
    let t1_terms = [
        "cause",
        "sequence",
        "threshold",
        "duration",
        "probability",
        "severity",
        "comparison",
        "classification",
        "state",
    ];
    if t1_terms.iter().any(|t| lower_term.contains(t)) {
        return PrimitiveTier::T1Universal;
    }

    // T3 Domain-specific: regulatory artifacts
    let t3_terms = [
        "icsr",
        "pbrer",
        "dsur",
        "psur",
        "rmp",
        "aesi",
        "prr",
        "ror",
        "labeling",
        "applicant",
        "hospitalization",
    ];
    if t3_terms.iter().any(|t| lower_term.contains(t)) {
        return PrimitiveTier::T3DomainSpecific;
    }

    // T2-C Composite: built from other terms
    let t2c_terms = [
        "serious adverse",
        "unexpected adverse",
        "alert report",
        "safety signal",
        "benefit-risk",
        "causality assessment",
    ];
    if t2c_terms.iter().any(|t| lower_def.contains(t)) {
        return PrimitiveTier::T2Composite;
    }

    // T2-P Primitive: cross-domain atomic
    let t2p_terms = [
        "harm",
        "risk",
        "benefit",
        "signal",
        "expectedness",
        "obligation",
        "disproportionality",
    ];
    if t2p_terms.iter().any(|t| lower_term.contains(t)) {
        return PrimitiveTier::T2Primitive;
    }

    // Default to T2-C for composite definitions
    PrimitiveTier::T2Composite
}

// ============================================================================
// T1 PRIMITIVE: RECURSION - Component extraction
// ============================================================================

fn extract_components(_term: &str, definition: &str) -> Vec<String> {
    let mut components = Vec::new();
    let lower_def = definition.to_lowercase();

    // Check for T1 primitives in definition
    let primitives = [
        ("cause", "CAUSATION"),
        ("threshold", "THRESHOLD"),
        ("severity", "SEVERITY"),
        ("probability", "PROBABILITY"),
        ("duration", "DURATION"),
        ("sequence", "SEQUENCE"),
    ];

    for (pattern, component) in primitives {
        if lower_def.contains(pattern) {
            components.push(component.to_string());
        }
    }

    // Check for T2 primitives
    let t2_primitives = [
        ("harm", "HARM"),
        ("risk", "RISK"),
        ("benefit", "BENEFIT"),
        ("signal", "SIGNAL"),
        ("serious", "SEVERITY"),
        ("unexpected", "EXPECTEDNESS"),
    ];

    for (pattern, component) in t2_primitives {
        if lower_def.contains(pattern) && !components.contains(&component.to_string()) {
            components.push(component.to_string());
        }
    }

    components
}

// ============================================================================
// TOOL 1: regulatory_primitives_extract
// ============================================================================

/// Extract primitives from regulatory source
pub fn extract(params: RegulatoryExtractParams) -> Result<CallToolResult, McpError> {
    let source_type = classify_source(&params.source);

    // Extract from content or use built-in definitions
    let primitives = if params.content.is_empty() {
        get_builtin_primitives(source_type)
    } else {
        extract_from_content(&params.content, source_type)
    };

    // Filter by max tier if specified
    let filtered: Vec<_> = primitives
        .into_iter()
        .filter(|p| match params.max_tier {
            1 => p.tier == PrimitiveTier::T1Universal,
            2 => matches!(
                p.tier,
                PrimitiveTier::T1Universal | PrimitiveTier::T2Primitive
            ),
            _ => true,
        })
        .collect();

    // Build JSON response
    let inventory: Vec<_> = filtered
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "tier": p.tier.as_str(),
                "definition": p.definition,
                "source": p.source,
                "components": p.components,
            })
        })
        .collect();

    // Count by tier
    let t1_count = filtered
        .iter()
        .filter(|p| p.tier == PrimitiveTier::T1Universal)
        .count();
    let t2p_count = filtered
        .iter()
        .filter(|p| p.tier == PrimitiveTier::T2Primitive)
        .count();
    let t2c_count = filtered
        .iter()
        .filter(|p| p.tier == PrimitiveTier::T2Composite)
        .count();
    let t3_count = filtered
        .iter()
        .filter(|p| p.tier == PrimitiveTier::T3DomainSpecific)
        .count();

    let result = json!({
        "source": source_type,
        "total_primitives": filtered.len(),
        "tier_counts": {
            "T1_Universal": t1_count,
            "T2_Primitive": t2p_count,
            "T2_Composite": t2c_count,
            "T3_Domain": t3_count,
        },
        "primitives": inventory,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// TOOL 2: regulatory_primitives_audit
// ============================================================================

/// Audit FDA vs CIOMS/ICH consistency
pub fn audit(params: RegulatoryAuditParams) -> Result<CallToolResult, McpError> {
    // Get definitions for comparison
    let fda_def = get_fda_definition(&params.fda_term);
    let cioms_def = get_cioms_definition(&params.cioms_term);

    // Compute structural match (word overlap)
    let structural = compute_structural_match(&fda_def, &cioms_def);

    // Compute semantic match (concept overlap)
    let semantic = compute_semantic_match(&fda_def, &cioms_def);

    // Identify discrepancies
    let discrepancies =
        find_discrepancies(&params.fda_term, &params.cioms_term, &fda_def, &cioms_def);

    // Determine verdict
    let verdict = if structural >= 0.9 && semantic >= 0.9 {
        "Aligned"
    } else if structural >= 0.7 || semantic >= 0.8 {
        "MinorDeviation"
    } else if structural >= 0.5 || semantic >= 0.5 {
        "MajorDeviation"
    } else {
        "NoCorrespondence"
    };

    // Component-level audit if requested
    let component_audit = if params.include_components.unwrap_or(true) {
        let fda_components = extract_components(&params.fda_term, &fda_def);
        let cioms_components = extract_components(&params.cioms_term, &cioms_def);
        Some(json!({
            "fda_components": fda_components,
            "cioms_components": cioms_components,
            "shared": fda_components.iter()
                .filter(|c| cioms_components.contains(c))
                .collect::<Vec<_>>(),
        }))
    } else {
        None
    };

    let result = json!({
        "audit": {
            "fda_term": params.fda_term,
            "fda_definition": fda_def,
            "cioms_term": params.cioms_term,
            "cioms_definition": cioms_def,
        },
        "match_scores": {
            "structural": format!("{:.0}%", structural * 100.0),
            "semantic": format!("{:.0}%", semantic * 100.0),
        },
        "verdict": verdict,
        "discrepancies": discrepancies,
        "component_audit": component_audit,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// TOOL 3: regulatory_primitives_compare
// ============================================================================

/// Cross-domain transfer analysis
pub fn compare(params: RegulatoryCompareParams) -> Result<CallToolResult, McpError> {
    let domain1 = normalize_domain(&params.domain1);
    let domain2 = normalize_domain(&params.domain2);

    // Get primitive inventories for each domain
    let inv1 = get_domain_inventory(&domain1);
    let inv2 = get_domain_inventory(&domain2);

    // Compute transfer mappings
    let mut transfers = Vec::new();
    for p1 in &inv1 {
        if let Some(analog) = find_analog(p1, &inv2) {
            let confidence = compute_transfer_confidence(p1, &analog);
            if confidence >= params.confidence_threshold {
                transfers.push(json!({
                    "source_primitive": p1.0,
                    "source_domain": domain1,
                    "target_analog": analog.0,
                    "target_domain": domain2,
                    "tier": p1.1,
                    "confidence": format!("{:.2}", confidence),
                    "transfer_notes": get_transfer_notes(&p1.0, &analog.0),
                }));
            }
        }
    }

    let result = json!({
        "comparison": {
            "domain1": domain1,
            "domain2": domain2,
            "confidence_threshold": params.confidence_threshold,
        },
        "transfers": transfers,
        "transfer_count": transfers.len(),
        "high_confidence_count": transfers.iter()
            .filter(|t| t["confidence"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0) >= 0.7)
            .count(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_builtin_primitives(source: &str) -> Vec<ExtractedPrimitive> {
    match source {
        "FDA" => vec![
            ExtractedPrimitive {
                name: "SERIOUS".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Any adverse experience resulting in death, life-threatening, hospitalization, disability, or birth defect".into(),
                source: "21 CFR 314.80".into(),
                components: vec!["HARM".into(), "SEVERITY".into(), "THRESHOLD".into()],
            },
            ExtractedPrimitive {
                name: "UNEXPECTED".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Not previously observed (not in labeling)".into(),
                source: "21 CFR 314.80".into(),
                components: vec!["EXPECTEDNESS".into()],
            },
            ExtractedPrimitive {
                name: "REASONABLE_POSSIBILITY".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Cannot be ruled out (non-exclusion standard)".into(),
                source: "21 CFR 314.80".into(),
                components: vec!["CAUSATION".into(), "THRESHOLD".into()],
            },
            ExtractedPrimitive {
                name: "ALERT_REPORT".into(),
                tier: PrimitiveTier::T3DomainSpecific,
                definition: "Report submitted within 15 days for serious AND unexpected".into(),
                source: "21 CFR 314.80".into(),
                components: vec!["SERIOUS".into(), "UNEXPECTED".into(), "DURATION".into(), "OBLIGATION".into()],
            },
        ],
        "ICH" | "CIOMS" => vec![
            ExtractedPrimitive {
                name: "ADVERSE_EVENT".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Any untoward medical occurrence in a patient administered a pharmaceutical product".into(),
                source: "ICH E2A II.A.1".into(),
                components: vec!["HARM".into(), "OCCURRENCE".into()],
            },
            ExtractedPrimitive {
                name: "SERIOUS_AE".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Results in death, is life-threatening, requires hospitalization, disability, or birth defect".into(),
                source: "ICH E2A II.A.4".into(),
                components: vec!["ADVERSE_EVENT".into(), "SEVERITY".into(), "THRESHOLD".into()],
            },
            ExtractedPrimitive {
                name: "UNEXPECTED_ADR".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Not consistent with applicable product information".into(),
                source: "ICH E2A II.C".into(),
                components: vec!["ADR".into(), "EXPECTEDNESS".into()],
            },
            ExtractedPrimitive {
                name: "CAUSALITY_ASSESSMENT".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Evaluation of likelihood that medicinal product caused adverse event".into(),
                source: "ICH E2B(R3) 2.12".into(),
                components: vec!["CAUSATION".into(), "PROBABILITY".into()],
            },
            ExtractedPrimitive {
                name: "SIGNAL_DETECTION".into(),
                tier: PrimitiveTier::T2Composite,
                definition: "Looking for and identifying signals using event data".into(),
                source: "ICH E2E 2.2".into(),
                components: vec!["SIGNAL".into(), "DISPROPORTIONALITY".into()],
            },
            ExtractedPrimitive {
                name: "ICSR".into(),
                tier: PrimitiveTier::T3DomainSpecific,
                definition: "Report of suspected adverse reaction in individual patient".into(),
                source: "ICH E2B(R3)".into(),
                components: vec!["ADVERSE_EVENT".into()],
            },
            ExtractedPrimitive {
                name: "PBRER".into(),
                tier: PrimitiveTier::T3DomainSpecific,
                definition: "Periodic report evaluating benefit-risk balance".into(),
                source: "ICH E2C(R2)".into(),
                components: vec!["BENEFIT".into(), "RISK".into(), "COMPARISON".into()],
            },
        ],
        _ => vec![],
    }
}

fn extract_from_content(content: &str, source: &str) -> Vec<ExtractedPrimitive> {
    // Simplified extraction - in production would use NLP
    let mut primitives = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        if line.contains(':') {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let term = parts[0].trim().to_uppercase().replace(' ', "_");
                let definition = parts[1].trim().to_string();
                let tier = classify_tier(&term, &definition);
                let components = extract_components(&term, &definition);

                primitives.push(ExtractedPrimitive {
                    name: term,
                    tier,
                    definition,
                    source: source.to_string(),
                    components,
                });
            }
        }
    }

    primitives
}

fn get_fda_definition(term: &str) -> String {
    let lower = term.to_lowercase();
    if lower.contains("serious") {
        "Any adverse drug experience occurring at any dose that results in death, is life-threatening, requires inpatient hospitalization or prolongation of existing hospitalization, results in persistent or significant disability/incapacity, or is a congenital anomaly/birth defect".into()
    } else if lower.contains("unexpected") {
        "An adverse drug experience that is not listed in the current labeling for the drug product"
            .into()
    } else if lower.contains("reasonable") || lower.contains("possibility") {
        "There is evidence to suggest a causal relationship between the drug and the adverse experience".into()
    } else {
        format!("FDA definition for '{}' not found", term)
    }
}

fn get_cioms_definition(term: &str) -> String {
    let lower = term.to_lowercase();
    if lower.contains("serious") {
        "Any untoward medical occurrence that at any dose results in death, is life-threatening, requires inpatient hospitalization or prolongation of existing hospitalization, results in persistent or significant disability/incapacity, or is a congenital anomaly/birth defect".into()
    } else if lower.contains("unexpected") {
        "An adverse reaction, the nature or severity of which is not consistent with the applicable product information".into()
    } else if lower.contains("causal") || lower.contains("assessment") {
        "The evaluation of the likelihood that a medicinal product was the cause of an observed adverse event".into()
    } else {
        format!("CIOMS/ICH definition for '{}' not found", term)
    }
}

fn compute_structural_match(def1: &str, def2: &str) -> f64 {
    // Bind lowercased strings to extend lifetime beyond split_whitespace()
    let lower1 = def1.to_lowercase();
    let lower2 = def2.to_lowercase();
    let words1: std::collections::HashSet<_> = lower1.split_whitespace().collect();
    let words2: std::collections::HashSet<_> = lower2.split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn compute_semantic_match(def1: &str, def2: &str) -> f64 {
    // Simplified: check for key concept presence
    let key_concepts = [
        "death",
        "life-threatening",
        "hospitalization",
        "disability",
        "birth defect",
        "congenital",
        "serious",
        "adverse",
    ];

    let def1_lower = def1.to_lowercase();
    let def2_lower = def2.to_lowercase();

    let mut matches = 0;
    for concept in key_concepts {
        let in_def1 = def1_lower.contains(concept);
        let in_def2 = def2_lower.contains(concept);
        if in_def1 == in_def2 {
            matches += 1;
        }
    }

    matches as f64 / key_concepts.len() as f64
}

fn find_discrepancies(
    fda_term: &str,
    _cioms_term: &str,
    fda_def: &str,
    cioms_def: &str,
) -> Vec<String> {
    let mut discrepancies = Vec::new();

    // Check for "medically important" which FDA includes but ICH separates
    if fda_def.to_lowercase().contains("medical") && !cioms_def.to_lowercase().contains("medical") {
        discrepancies
            .push("FDA includes 'medically important events'; ICH treats separately".into());
    }

    // Check for labeling vs product information wording
    if fda_def.to_lowercase().contains("labeling")
        && cioms_def.to_lowercase().contains("product information")
    {
        discrepancies.push("FDA uses 'labeling'; ICH uses 'product information'".into());
    }

    // Check for causality standard differences
    if fda_term.to_lowercase().contains("possibility") {
        discrepancies.push(
            "FDA 'reasonable possibility' is binary (yes/no); ICH causality is graded".into(),
        );
    }

    discrepancies
}

fn normalize_domain(domain: &str) -> String {
    let lower = domain.to_lowercase();
    if lower.contains("pv") || lower.contains("pharma") || lower.contains("drug") {
        "pharmacovigilance".into()
    } else if lower.contains("cloud") || lower.contains("sre") || lower.contains("infra") {
        "cloud".into()
    } else if lower.contains("ai") || lower.contains("ml") || lower.contains("algo") {
        "ai_safety".into()
    } else if lower.contains("finance") || lower.contains("risk") {
        "finance".into()
    } else {
        domain.to_lowercase()
    }
}

fn get_domain_inventory(domain: &str) -> Vec<(String, String)> {
    match domain {
        "pharmacovigilance" => vec![
            ("HARM".into(), "T2-P"),
            ("SEVERITY".into(), "T2-P"),
            ("EXPECTEDNESS".into(), "T2-P"),
            ("SIGNAL".into(), "T2-P"),
            ("SERIOUS_AE".into(), "T2-C"),
            ("UNEXPECTED_ADR".into(), "T2-C"),
            ("ICSR".into(), "T3"),
        ]
        .into_iter()
        .map(|(a, b)| (a, b.into()))
        .collect(),
        "cloud" => vec![
            ("HARM".into(), "T2-P"),
            ("SEVERITY".into(), "T2-P"),
            ("EXPECTEDNESS".into(), "T2-P"),
            ("SIGNAL".into(), "T2-P"),
            ("CRITICAL_INCIDENT".into(), "T2-C"),
            ("NOVEL_FAILURE".into(), "T2-C"),
            ("PAGERDUTY_ALERT".into(), "T3"),
        ]
        .into_iter()
        .map(|(a, b)| (a, b.into()))
        .collect(),
        "ai_safety" => vec![
            ("HARM".into(), "T2-P"),
            ("SEVERITY".into(), "T2-P"),
            ("EXPECTEDNESS".into(), "T2-P"),
            ("SIGNAL".into(), "T2-P"),
            ("HIGH_SEVERITY_INCIDENT".into(), "T2-C"),
            ("OOD_FAILURE".into(), "T2-C"),
            ("MODEL_CARD".into(), "T3"),
        ]
        .into_iter()
        .map(|(a, b)| (a, b.into()))
        .collect(),
        _ => vec![],
    }
}

fn find_analog(
    primitive: &(String, String),
    target_inventory: &[(String, String)],
) -> Option<(String, String)> {
    // Direct name match
    if let Some(found) = target_inventory
        .iter()
        .find(|(name, _)| name == &primitive.0)
    {
        return Some(found.clone());
    }

    // Semantic mapping
    let mappings: HashMap<&str, &str> = [
        ("SERIOUS_AE", "CRITICAL_INCIDENT"),
        ("SERIOUS_AE", "HIGH_SEVERITY_INCIDENT"),
        ("UNEXPECTED_ADR", "NOVEL_FAILURE"),
        ("UNEXPECTED_ADR", "OOD_FAILURE"),
        ("ICSR", "PAGERDUTY_ALERT"),
        ("ICSR", "AI_INCIDENT_REPORT"),
        ("CRITICAL_INCIDENT", "SERIOUS_AE"),
        ("NOVEL_FAILURE", "UNEXPECTED_ADR"),
    ]
    .into_iter()
    .collect();

    if let Some(&analog_name) = mappings.get(primitive.0.as_str()) {
        if let Some(found) = target_inventory
            .iter()
            .find(|(name, _)| name == analog_name)
        {
            return Some(found.clone());
        }
    }

    None
}

fn compute_transfer_confidence(source: &(String, String), target: &(String, String)) -> f64 {
    let structural = if source.1 == target.1 { 1.0 } else { 0.5 };
    let functional = if source.0 == target.0 { 1.0 } else { 0.8 };
    let contextual = 0.6; // Cross-domain context penalty

    (structural * 0.4) + (functional * 0.4) + (contextual * 0.2)
}

fn get_transfer_notes(source: &str, target: &str) -> String {
    if source == target {
        "Direct transfer - same primitive".into()
    } else if source.contains("SERIOUS")
        || target.contains("CRITICAL")
        || target.contains("HIGH_SEVERITY")
    {
        "Severity-based incident mapping".into()
    } else if source.contains("UNEXPECTED") || target.contains("NOVEL") || target.contains("OOD") {
        "Expectedness-based anomaly mapping".into()
    } else if source.contains("ICSR") || target.contains("ALERT") || target.contains("REPORT") {
        "Incident report artifact mapping".into()
    } else {
        "Structural correspondence".into()
    }
}

// ============================================================================
// TOOL 4: regulatory_effectiveness_assess
// ============================================================================

/// Assess FDA effectiveness endpoint for approval pathway compatibility
pub fn effectiveness_assess(
    params: crate::params::EffectivenessAssessParams,
) -> Result<CallToolResult, McpError> {
    // Parse pathway
    let pathway = match params.pathway.to_lowercase().as_str() {
        "traditional" => "Traditional",
        "accelerated" => "Accelerated",
        "breakthrough" => "Breakthrough",
        "fast_track" | "fasttrack" => "FastTrack",
        "priority" | "priority_review" => "PriorityReview",
        _ => "Traditional",
    };

    // Parse endpoint tier
    let tier = match params.endpoint_tier.to_lowercase().as_str() {
        "primary" => "Primary",
        "secondary" => "Secondary",
        "exploratory" => "Exploratory",
        _ => "Primary",
    };

    // Parse endpoint type
    let endpoint_type = match params.endpoint_type.to_lowercase().as_str() {
        "clinical" => "Clinical",
        "surrogate" => "Surrogate",
        "intermediate" | "intermediate_clinical" => "IntermediateClinical",
        "biomarker" => "Biomarker",
        "patient_reported" | "pro" => "PatientReported",
        "digital" | "digital_health" | "dht" => "DigitalHealth",
        _ => "Clinical",
    };

    // Parse multiplicity method
    let method = match params.multiplicity_method.to_lowercase().as_str() {
        "bonferroni" => "Bonferroni",
        "holm" => "Holm",
        "hochberg" => "Hochberg",
        "fixed_sequence" | "fixedsequence" => "FixedSequence",
        "fallback" => "Fallback",
        "graphical" => "Graphical",
        _ => "Bonferroni",
    };

    // Calculate adjusted alpha
    let adjusted_alpha = if method == "Bonferroni" && params.n_comparisons > 0 {
        params.alpha / params.n_comparisons as f64
    } else {
        params.alpha
    };

    // Check pathway compatibility
    let supports_traditional = matches!(endpoint_type, "Clinical" | "PatientReported");
    let supports_accelerated = matches!(
        endpoint_type,
        "Surrogate" | "IntermediateClinical" | "Biomarker"
    );

    let pathway_compatible = match pathway {
        "Traditional" => supports_traditional,
        "Accelerated" => supports_accelerated,
        _ => true,
    };

    // Determine significance
    let significant = params.p_value.map(|p| p < adjusted_alpha).unwrap_or(false);

    // Get CFR reference
    let cfr = match pathway {
        "Traditional" => "21 CFR 314.50",
        "Accelerated" => "21 CFR 314.510",
        "Breakthrough" => "21 CFR 312.320",
        "FastTrack" => "21 CFR 312.300",
        "PriorityReview" => "21 CFR 314.107",
        _ => "21 CFR 314.50",
    };

    // Requires confirmatory trial?
    let requires_confirmatory = pathway == "Accelerated";

    // Build result
    let result = json!({
        "assessment": {
            "endpoint_name": params.endpoint_name,
            "endpoint_tier": tier,
            "endpoint_type": endpoint_type,
            "pathway": pathway,
        },
        "approval_compatibility": {
            "supports_traditional": supports_traditional,
            "supports_accelerated": supports_accelerated,
            "pathway_compatible": pathway_compatible,
            "requires_confirmatory_trial": requires_confirmatory,
        },
        "statistical": {
            "alpha": params.alpha,
            "adjusted_alpha": adjusted_alpha,
            "n_comparisons": params.n_comparisons,
            "multiplicity_method": method,
            "p_value": params.p_value,
            "significant": significant,
            "success": params.success.unwrap_or(significant),
        },
        "regulatory": {
            "cfr_reference": cfr,
            "tier_required_for_approval": tier == "Primary",
            "requires_multiplicity_control": tier != "Exploratory",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
