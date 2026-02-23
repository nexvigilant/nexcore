//! Pharmacovigilance taxonomy MCP tools — WHO-grounded PV concept encoder.
//!
//! Pure-function wrappers exposing nexcore-pharmacovigilance's 4-tier taxonomy:
//! T1 Lex Primitiva, T2-P primitives, T2-C composites, T3 domain concepts,
//! Chomsky classification, WHO pillar complexity, and cross-domain transfer.

use nexcore_pharmacovigilance::{
    AnalyticsConcept, AssessmentConcept, DetectionConcept, InfrastructureConcept, LexSymbol,
    OperationsConcept, PreventionConcept, PrimitiveComposition, PvComposite, PvPrimitive,
    PvSubsystem, RegulatoryConcept, SafetyCommsConcept, ScopeConcept, SpecialPopulationConcept,
    TransferDomain, UnderstandingConcept, lookup_transfer, taxonomy_summary, transfer_matrix,
    who_pillar_complexity,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::pharmacovigilance::{
    PvChomskyLookupParams, PvCompositeLookupParams, PvConceptLookupParams, PvLexSymbolsParams,
    PvPrimitiveLookupParams, PvTaxonomySummaryParams, PvTransferLookupParams,
    PvTransferMatrixParams, PvWhoPillarsParams,
};

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

/// Convert CamelCase to snake_case for enum variant matching.
fn camel_to_snake(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 8);
    for (i, c) in s.char_indices() {
        if c.is_ascii_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

/// Normalize user input: lowercase, replace hyphens/spaces with underscores.
fn normalize(s: &str) -> String {
    s.trim().to_lowercase().replace(['-', ' '], "_")
}

fn format_grounding(comp: &PrimitiveComposition) -> serde_json::Value {
    let symbols: Vec<&str> = comp.symbols().iter().map(|s| s.glyph()).collect();
    json!({
        "symbols": symbols,
        "tier": format!("{:?}", comp.tier()),
        "count": comp.unique_count(),
    })
}

// ── Enum resolvers ───────────────────────────────────────────────────────

fn resolve_primitive(name: &str) -> Option<PvPrimitive> {
    let norm = normalize(name);
    PvPrimitive::ALL
        .iter()
        .copied()
        .find(|p| camel_to_snake(&format!("{p:?}")) == norm)
}

fn resolve_composite(name: &str) -> Option<PvComposite> {
    let norm = normalize(name);
    PvComposite::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == norm)
}

fn resolve_subsystem(name: &str) -> Option<PvSubsystem> {
    let norm = normalize(name);
    PvSubsystem::ALL
        .iter()
        .copied()
        .find(|s| camel_to_snake(&format!("{s:?}")) == norm)
}

fn resolve_transfer_domain(name: &str) -> Option<TransferDomain> {
    match normalize(name).as_str() {
        "clinical_trials" => Some(TransferDomain::ClinicalTrials),
        "regulatory_affairs" => Some(TransferDomain::RegulatoryAffairs),
        "epidemiology" => Some(TransferDomain::Epidemiology),
        "health_economics" => Some(TransferDomain::HealthEconomics),
        _ => None,
    }
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Get the full PV taxonomy summary (T1/T2-P/T2-C/T3 counts).
pub fn pv_taxonomy_summary(_p: PvTaxonomySummaryParams) -> Result<CallToolResult, McpError> {
    let s = taxonomy_summary();
    ok_json(json!({
        "t1_lex_primitiva": s.t1,
        "t2p_primitives": s.t2p,
        "t2c_composites": s.t2c,
        "t3_domain_concepts": s.t3,
        "total": s.total,
    }))
}

/// Look up a T2-P primitive by name.
pub fn pv_taxonomy_primitive(p: PvPrimitiveLookupParams) -> Result<CallToolResult, McpError> {
    let prim = match resolve_primitive(&p.name) {
        Some(v) => v,
        None => {
            let all: Vec<String> = PvPrimitive::ALL
                .iter()
                .map(|p| camel_to_snake(&format!("{p:?}")))
                .collect();
            return err_result(&format!(
                "unknown primitive '{}'. Valid: {}",
                p.name,
                all.join(", ")
            ));
        }
    };

    ok_json(json!({
        "name": format!("{prim:?}"),
        "description": prim.description(),
        "tier": format!("{:?}", prim.tier()),
        "grounding": format_grounding(&prim.grounding()),
        "domains": prim.domains(),
    }))
}

/// Look up a T2-C composite by name.
pub fn pv_taxonomy_composite(p: PvCompositeLookupParams) -> Result<CallToolResult, McpError> {
    let comp = match resolve_composite(&p.name) {
        Some(v) => v,
        None => {
            let all: Vec<String> = PvComposite::ALL
                .iter()
                .map(|c| camel_to_snake(&format!("{c:?}")))
                .collect();
            return err_result(&format!(
                "unknown composite '{}'. Valid: {}",
                p.name,
                all.join(", ")
            ));
        }
    };

    let deps: Vec<String> = comp
        .dependencies()
        .iter()
        .map(|d| format!("{d:?}"))
        .collect();

    ok_json(json!({
        "name": format!("{comp:?}"),
        "description": comp.description(),
        "grounding": format_grounding(&comp.grounding()),
        "dependencies": deps,
        "is_recursive": comp.is_recursive(),
        "depth": comp.depth(),
    }))
}

/// Look up a T3 concept by pillar and name.
pub fn pv_taxonomy_concept(p: PvConceptLookupParams) -> Result<CallToolResult, McpError> {
    let pillar_norm = normalize(&p.pillar);
    let name_norm = normalize(&p.name);

    match pillar_norm.as_str() {
        "detection" => lookup_detection(&name_norm, &p.name),
        "assessment" => lookup_assessment(&name_norm, &p.name),
        "understanding" => lookup_understanding(&name_norm, &p.name),
        "prevention" => lookup_prevention(&name_norm, &p.name),
        "scope" => lookup_scope(&name_norm, &p.name),
        "regulatory" => lookup_regulatory(&name_norm, &p.name),
        "infrastructure" => lookup_infrastructure(&name_norm, &p.name),
        "operations" => lookup_operations(&name_norm, &p.name),
        "analytics" => lookup_analytics(&name_norm, &p.name),
        "safety_comms" => lookup_safety_comms(&name_norm, &p.name),
        "special_populations" => lookup_special_populations(&name_norm, &p.name),
        _ => err_result(&format!(
            "unknown pillar '{}'. Valid: detection, assessment, understanding, \
             prevention, scope, regulatory, infrastructure, operations, \
             analytics, safety_comms, special_populations",
            p.pillar
        )),
    }
}

// ── Pillar-specific T3 lookups ───────────────────────────────────────────

fn lookup_detection(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = DetectionConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "detection",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "chomsky_level": c.chomsky_level(),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown detection concept: {raw}")),
    }
}

fn lookup_assessment(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = AssessmentConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "assessment",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown assessment concept: {raw}")),
    }
}

fn lookup_understanding(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = UnderstandingConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "understanding",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "is_bradford_hill": c.is_bradford_hill(),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown understanding concept: {raw}")),
    }
}

fn lookup_prevention(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = PreventionConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "prevention",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown prevention concept: {raw}")),
    }
}

fn lookup_scope(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = ScopeConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "scope",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown scope concept: {raw}")),
    }
}

fn lookup_regulatory(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = RegulatoryConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "regulatory",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown regulatory concept: {raw}")),
    }
}

fn lookup_infrastructure(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = InfrastructureConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "infrastructure",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
        })),
        None => err_result(&format!("unknown infrastructure concept: {raw}")),
    }
}

fn lookup_operations(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = OperationsConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "operations",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
        })),
        None => err_result(&format!("unknown operations concept: {raw}")),
    }
}

fn lookup_analytics(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = AnalyticsConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "analytics",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown analytics concept: {raw}")),
    }
}

fn lookup_safety_comms(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = SafetyCommsConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "safety_comms",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
        })),
        None => err_result(&format!("unknown safety_comms concept: {raw}")),
    }
}

fn lookup_special_populations(norm: &str, raw: &str) -> Result<CallToolResult, McpError> {
    let concept = SpecialPopulationConcept::ALL
        .iter()
        .copied()
        .find(|c| camel_to_snake(&format!("{c:?}")) == *norm);
    match concept {
        Some(c) => ok_json(json!({
            "pillar": "special_populations",
            "name": format!("{c:?}"),
            "description": c.description(),
            "grounding": format_grounding(&c.grounding()),
            "source": c.source(),
        })),
        None => err_result(&format!("unknown special_populations concept: {raw}")),
    }
}

// ── Remaining tools ──────────────────────────────────────────────────────

/// Get Chomsky classification for a PV subsystem.
pub fn pv_taxonomy_chomsky(p: PvChomskyLookupParams) -> Result<CallToolResult, McpError> {
    let sub = match resolve_subsystem(&p.subsystem) {
        Some(v) => v,
        None => {
            let all: Vec<String> = PvSubsystem::ALL
                .iter()
                .map(|s| camel_to_snake(&format!("{s:?}")))
                .collect();
            return err_result(&format!(
                "unknown subsystem '{}'. Valid: {}",
                p.subsystem,
                all.join(", ")
            ));
        }
    };

    let level = sub.chomsky_level();
    ok_json(json!({
        "subsystem": format!("{sub:?}"),
        "chomsky_level": format!("{level:?}"),
        "automaton": level.automaton(),
        "generator_count": level.generator_count(),
        "justification": sub.justification(),
        "who_pillar": sub.who_pillar(),
    }))
}

/// Get WHO pillar complexity classification.
pub fn pv_taxonomy_who_pillars(_p: PvWhoPillarsParams) -> Result<CallToolResult, McpError> {
    let pillars = who_pillar_complexity();
    let items: Vec<serde_json::Value> = pillars
        .iter()
        .map(|(name, level)| {
            json!({
                "pillar": name,
                "chomsky_level": format!("{level:?}"),
                "automaton": level.automaton(),
                "generator_count": level.generator_count(),
            })
        })
        .collect();

    ok_json(json!({ "pillars": items }))
}

/// Look up cross-domain transfer confidence for a target domain.
pub fn pv_taxonomy_transfer(p: PvTransferLookupParams) -> Result<CallToolResult, McpError> {
    let domain = match resolve_transfer_domain(&p.domain) {
        Some(d) => d,
        None => {
            return err_result(
                "unknown domain. Valid: clinical_trials, regulatory_affairs, \
                 epidemiology, health_economics",
            );
        }
    };

    let tc = lookup_transfer(domain);
    ok_json(json!({
        "domain": format!("{:?}", tc.domain),
        "score": tc.score(),
        "label": tc.label(),
        "structural": tc.structural,
        "functional": tc.functional,
        "contextual": tc.contextual,
        "limiting_factor": tc.limiting_factor,
        "mappings": tc.mappings.iter().map(|(a, b)| json!({"from": a, "to": b})).collect::<Vec<_>>(),
        "caveat": tc.caveat,
    }))
}

/// Get the full transfer confidence matrix (all 4 domains).
pub fn pv_taxonomy_transfer_matrix(_p: PvTransferMatrixParams) -> Result<CallToolResult, McpError> {
    let matrix = transfer_matrix();
    let items: Vec<serde_json::Value> = matrix
        .iter()
        .map(|tc| {
            json!({
                "domain": format!("{:?}", tc.domain),
                "score": tc.score(),
                "label": tc.label(),
                "structural": tc.structural,
                "functional": tc.functional,
                "contextual": tc.contextual,
                "limiting_factor": tc.limiting_factor,
            })
        })
        .collect();

    ok_json(json!({ "transfers": items }))
}

/// Get all T1 Lex Primitiva symbols used in grounding.
pub fn pv_taxonomy_lex_symbols(_p: PvLexSymbolsParams) -> Result<CallToolResult, McpError> {
    let symbols: Vec<serde_json::Value> = LexSymbol::all()
        .iter()
        .map(|s| {
            json!({
                "name": format!("{s:?}"),
                "glyph": s.glyph(),
            })
        })
        .collect();

    ok_json(json!({
        "count": symbols.len(),
        "symbols": symbols,
    }))
}
