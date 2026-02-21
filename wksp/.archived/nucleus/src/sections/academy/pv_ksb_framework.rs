//! Canonical PV KSB framework dataset imported from workbook.

use crate::api_client::{Ksb, KsbDomainSummary};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct PvKsbWorkbook {
    pub source_file: String,
    pub generated_at: String,
    pub sheet_names: Vec<String>,
    pub sheet_row_counts: BTreeMap<String, usize>,
    pub domain_overview: Vec<PvDomainRow>,
    pub capability_components: Vec<PvKsbRow>,
    pub epa_master: Vec<PvEpaRow>,
    pub cpa_master: Vec<PvCpaRow>,
    pub epa_domain_mapping: Vec<PvEpaDomainRow>,
    pub cpa_domain_mapping: Vec<PvCpaDomainRow>,
    pub cross_domain_integration: Vec<PvCrossDomainRow>,
    pub all_sheets: BTreeMap<String, Vec<BTreeMap<String, String>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvDomainRow {
    pub domain_id: String,
    pub domain_name: String,
    #[serde(default)]
    pub thematic_cluster: String,
    #[serde(default)]
    pub cluster_name: String,
    #[serde(default)]
    pub definition: String,
    #[serde(default)]
    pub regional_adaptation_notes: String,
    #[serde(default)]
    pub advancement_pathways: String,
    #[serde(default)]
    pub total_ksbs: String,
    #[serde(default)]
    pub last_updated: String,
    #[serde(default)]
    pub has_assessment: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvKsbRow {
    pub ksb_id: String,
    pub domain_id: String,
    #[serde(rename = "type")]
    pub ksb_type: String,
    #[serde(default)]
    pub major_section: String,
    #[serde(default)]
    pub section: String,
    #[serde(default)]
    pub item_name: String,
    #[serde(default)]
    pub item_description: String,
    #[serde(default)]
    pub proficiency_level: String,
    #[serde(default)]
    pub bloom_level: String,
    #[serde(default)]
    pub keywords: String,
    #[serde(default)]
    pub curriculum_ref: String,
    #[serde(default)]
    pub source_file: String,
    #[serde(default)]
    pub source_location: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub valid_domain: String,
    #[serde(default)]
    pub regulatory_refs: String,
    #[serde(default)]
    pub epa_id: String,
    #[serde(default)]
    pub cpa_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvEpaRow {
    pub epa_id: String,
    pub epa_name: String,
    #[serde(default)]
    pub focus_area: String,
    #[serde(default)]
    pub primary_domains: String,
    #[serde(default)]
    pub port_range: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvCpaRow {
    pub cpa_id: String,
    pub cpa_name: String,
    #[serde(default)]
    pub focus_area: String,
    #[serde(default)]
    pub primary_integration: String,
    #[serde(default)]
    pub career_stage: String,
    #[serde(default)]
    pub executive_summary: String,
    #[serde(default)]
    pub ai_integration: String,
    #[serde(default)]
    pub key_epas: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvEpaDomainRow {
    pub epa_id: String,
    pub epa_name: String,
    pub domain_id: String,
    pub domain_name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvCpaDomainRow {
    pub cpa_id: String,
    pub cpa_name: String,
    pub domain_id: String,
    pub domain_name: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvCrossDomainRow {
    pub domain_id: String,
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub related_domain: String,
    #[serde(default)]
    pub integration_point: String,
    #[serde(default)]
    pub data_process_exchange: String,
    #[serde(default)]
    pub prerequisite_level: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub valid_domain: String,
}

#[derive(Debug, Clone)]
pub struct PvFrameworkStats {
    pub sheet_count: usize,
    pub domain_count: usize,
    pub ksb_count: usize,
    pub epa_count: usize,
    pub cpa_count: usize,
    pub integration_edges: usize,
}

static DATA: OnceLock<PvKsbWorkbook> = OnceLock::new();

fn parse_workbook_json(content: &str) -> Result<PvKsbWorkbook, serde_json::Error> {
    serde_json::from_str(content)
}

fn empty_workbook() -> PvKsbWorkbook {
    PvKsbWorkbook {
        source_file: "embedded-fallback".to_string(),
        generated_at: String::new(),
        sheet_names: Vec::new(),
        sheet_row_counts: BTreeMap::new(),
        domain_overview: Vec::new(),
        capability_components: Vec::new(),
        epa_master: Vec::new(),
        cpa_master: Vec::new(),
        epa_domain_mapping: Vec::new(),
        cpa_domain_mapping: Vec::new(),
        cross_domain_integration: Vec::new(),
        all_sheets: BTreeMap::new(),
    }
}

fn workbook_from_json_or_empty(content: &str) -> PvKsbWorkbook {
    match parse_workbook_json(content) {
        Ok(wb) => wb,
        Err(err) => {
            tracing::error!(error = %err, "failed_to_parse_pv_ksb_framework_embedded_json");
            empty_workbook()
        }
    }
}

pub fn workbook() -> &'static PvKsbWorkbook {
    DATA.get_or_init(|| workbook_from_json_or_empty(include_str!("data/pv_ksb_framework_2025_12_08.json")))
}

pub fn framework_stats() -> PvFrameworkStats {
    let wb = workbook();
    PvFrameworkStats {
        sheet_count: wb.sheet_names.len(),
        domain_count: wb.domain_overview.len(),
        ksb_count: wb.capability_components.len(),
        epa_count: wb.epa_master.len(),
        cpa_count: wb.cpa_master.len(),
        integration_edges: wb.cross_domain_integration.len(),
    }
}

pub fn sheet_counts() -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = workbook()
        .sheet_row_counts
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

pub fn fallback_domain_summaries() -> Vec<KsbDomainSummary> {
    let wb = workbook();
    let mut by_domain: HashMap<String, Vec<&PvKsbRow>> = HashMap::new();
    for row in &wb.capability_components {
        by_domain
            .entry(row.domain_id.clone())
            .or_default()
            .push(row);
    }

    wb.domain_overview
        .iter()
        .map(|domain| {
            let rows = by_domain
                .get(&domain.domain_id)
                .cloned()
                .unwrap_or_default();
            let count = rows.len() as u32;
            let examples = rows
                .iter()
                .filter_map(|r| {
                    if r.item_name.is_empty() {
                        None
                    } else {
                        Some(r.item_name.clone())
                    }
                })
                .take(3)
                .collect::<Vec<_>>();
            let (dom_prim, cog_prim) = primitive_pair_for_domain(&domain.domain_id);

            KsbDomainSummary {
                code: domain.domain_id.clone(),
                name: domain.domain_name.clone(),
                ksb_count: count,
                dominant_primitive: dom_prim.to_string(),
                cognitive_primitive: cog_prim.to_string(),
                transfer_confidence: transfer_confidence_for_count(count),
                pvos_layer: if domain.cluster_name.is_empty() {
                    None
                } else {
                    Some(domain.cluster_name.clone())
                },
                example_ksbs: examples,
            }
        })
        .collect()
}

pub fn fallback_ksb_detail(id: &str) -> Option<Ksb> {
    let wb = workbook();
    let id_norm = id.trim().to_ascii_uppercase();

    if let Some(domain) = wb
        .domain_overview
        .iter()
        .find(|d| d.domain_id.eq_ignore_ascii_case(&id_norm))
    {
        let (dom_prim, cog_prim) = primitive_pair_for_domain(&domain.domain_id);
        return Some(Ksb {
            id: domain.domain_id.clone(),
            code: domain.domain_id.clone(),
            title: domain.domain_name.clone(),
            ksb_type: "domain".to_string(),
            domain: domain.domain_name.clone(),
            domain_id: domain.domain_id.clone(),
            bloom_level: Some(3),
            description: domain.definition.clone(),
            grounding_primitives: vec![dom_prim.to_string(), cog_prim.to_string()],
        });
    }

    let domain_names: HashMap<&str, &str> = wb
        .domain_overview
        .iter()
        .map(|d| (d.domain_id.as_str(), d.domain_name.as_str()))
        .collect();

    wb.capability_components
        .iter()
        .find(|k| {
            k.ksb_id.eq_ignore_ascii_case(&id_norm)
                || k.curriculum_ref.eq_ignore_ascii_case(id)
                || k.item_name.eq_ignore_ascii_case(id)
        })
        .map(|k| {
            let (dom_prim, cog_prim) = primitive_pair_for_domain(&k.domain_id);
            let domain_name = domain_names
                .get(k.domain_id.as_str())
                .copied()
                .unwrap_or(k.domain_id.as_str())
                .to_string();
            Ksb {
                id: k.ksb_id.clone(),
                code: k.ksb_id.clone(),
                title: k.item_name.clone(),
                ksb_type: k.ksb_type.to_ascii_lowercase(),
                domain: domain_name,
                domain_id: k.domain_id.clone(),
                bloom_level: bloom_level_to_u8(&k.bloom_level),
                description: k.item_description.clone(),
                grounding_primitives: vec![dom_prim.to_string(), cog_prim.to_string()],
            }
        })
}

pub fn top_epas(limit: usize) -> Vec<PvEpaRow> {
    workbook().epa_master.iter().take(limit).cloned().collect()
}

pub fn top_cpas(limit: usize) -> Vec<PvCpaRow> {
    workbook().cpa_master.iter().take(limit).cloned().collect()
}

pub fn all_epas() -> Vec<PvEpaRow> {
    workbook().epa_master.clone()
}

pub fn all_cpas() -> Vec<PvCpaRow> {
    workbook().cpa_master.clone()
}

pub fn epa_by_id(id: &str) -> Option<PvEpaRow> {
    workbook()
        .epa_master
        .iter()
        .find(|e| e.epa_id.eq_ignore_ascii_case(id.trim()))
        .cloned()
}

pub fn cpa_by_id(id: &str) -> Option<PvCpaRow> {
    workbook()
        .cpa_master
        .iter()
        .find(|c| c.cpa_id.eq_ignore_ascii_case(id.trim()))
        .cloned()
}

pub fn epa_domain_links(epa_id: &str) -> Vec<PvEpaDomainRow> {
    workbook()
        .epa_domain_mapping
        .iter()
        .filter(|m| m.epa_id.eq_ignore_ascii_case(epa_id.trim()))
        .cloned()
        .collect()
}

pub fn cpa_domain_links(cpa_id: &str) -> Vec<PvCpaDomainRow> {
    workbook()
        .cpa_domain_mapping
        .iter()
        .filter(|m| m.cpa_id.eq_ignore_ascii_case(cpa_id.trim()))
        .cloned()
        .collect()
}

fn transfer_confidence_for_count(count: u32) -> f64 {
    if count >= 100 {
        0.90
    } else if count >= 80 {
        0.86
    } else if count >= 60 {
        0.82
    } else if count >= 40 {
        0.78
    } else {
        0.72
    }
}

fn primitive_pair_for_domain(domain_id: &str) -> (&'static str, &'static str) {
    match domain_id.to_ascii_uppercase().as_str() {
        "D01" => ("Existence", "Comparison"),
        "D02" => ("State", "Causality"),
        "D03" => ("Comparison", "Sequence"),
        "D04" => ("Boundary", "Persistence"),
        "D05" => ("Sequence", "Mapping"),
        "D06" => ("Boundary", "Comparison"),
        "D07" => ("Quantity", "Comparison"),
        "D08" => ("Frequency", "Comparison"),
        "D09" => ("Mapping", "Causality"),
        "D10" => ("State", "Location"),
        "D11" => ("Persistence", "Boundary"),
        "D12" => ("Causality", "Quantity"),
        "D13" => ("Location", "Persistence"),
        "D14" => ("Mapping", "Comparison"),
        "D15" => ("Frequency", "Mapping"),
        _ => ("Comparison", "State"),
    }
}

fn bloom_level_to_u8(level: &str) -> Option<u8> {
    match level.trim().to_ascii_lowercase().as_str() {
        "remember" | "recall" => Some(1),
        "understand" => Some(2),
        "apply" => Some(3),
        "analyze" | "analyse" => Some(4),
        "evaluate" => Some(5),
        "create" => Some(6),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fallback_domain_summaries, framework_stats, sheet_counts, workbook,
        workbook_from_json_or_empty,
    };

    #[test]
    fn workbook_import_has_expected_core_counts() {
        let stats = framework_stats();
        assert_eq!(stats.sheet_count, 48);
        assert_eq!(stats.ksb_count, 1462);
        assert_eq!(stats.domain_count, 20);
    }

    #[test]
    fn all_sheet_rows_present() {
        let wb = workbook();
        assert!(wb.all_sheets.contains_key("Capability Components"));
        assert!(sheet_counts().len() >= 48);
    }

    #[test]
    fn fallback_domains_are_non_empty() {
        let domains = fallback_domain_summaries();
        assert!(!domains.is_empty());
        assert!(domains.iter().any(|d| d.code == "D08"));
    }

    #[test]
    fn invalid_json_uses_safe_empty_fallback() {
        let wb = workbook_from_json_or_empty("{not json");
        assert!(wb.sheet_names.is_empty());
        assert!(wb.capability_components.is_empty());
        assert_eq!(wb.source_file, "embedded-fallback");
    }
}
