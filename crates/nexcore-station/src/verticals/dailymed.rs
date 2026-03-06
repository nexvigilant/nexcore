//! DailyMed WebMCP Hub config — FDA drug labeling (SPL) database.
//!
//! DailyMed is the official provider of FDA drug label information. Structured
//! Product Labeling (SPL) documents contain adverse reactions, warnings, drug
//! interactions, clinical pharmacology, and medication guides — the primary
//! reference safety information (RSI) source for US expectedness determinations.

use crate::builder::StationBuilder;
use crate::config::{AccessTier, ExecutionType, PvVertical, StationConfig, StationTool};
use serde_json::json;

/// Build the DailyMed station config for WebMCP Hub.
///
/// Covers SPL drug label search, adverse reactions extraction, medication guide
/// retrieval, drug class browsing, and labeling history navigation.
pub fn config() -> StationConfig {
    StationBuilder::new(
        PvVertical::DailyMed,
        "DailyMed — FDA Structured Product Labeling (SPL) Database",
    )
    .description(
        "Search and extract FDA drug labeling data from DailyMed, the official \
        repository of Structured Product Labeling (SPL). Access prescribing \
        information, adverse reactions, warnings, drug interactions, medication \
        guides, and labeling history for expectedness determination in \
        pharmacovigilance case assessment.",
    )
    .tags([
        "pharmacovigilance",
        "nexvigilant",
        "patient-safety",
        "dailymed",
        "drug-labeling",
        "spl",
        "adverse-reactions",
        "expectedness",
        "fda",
        "prescribing-information",
        "medication-guide",
        "reference-safety-information",
    ])
    // ── Search ───────────────────────────────────────────────────────────────
    .fill_tool(
        "search-drug-labels",
        "Search DailyMed for drug labels by drug name (brand or generic), \
        active ingredient, NDC, application number, or manufacturer. Returns \
        matching SPL documents with product names, labeler, dosage form, and \
        route of administration. Primary entry point for finding reference \
        safety information for ICSR expectedness assessment.",
        "/search.cfm",
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Drug name (brand or INN/generic), active ingredient, NDC, or NDA/ANDA number"
                },
                "labeltype": {
                    "type": "string",
                    "enum": [
                        "human prescription drug",
                        "human otc drug",
                        "vaccine",
                        "plasma derivative",
                        "cellular therapy",
                        "combination product",
                        "medication guide"
                    ],
                    "description": "Filter by product type"
                },
                "audience": {
                    "type": "string",
                    "enum": ["professional", "consumer"],
                    "description": "Label audience: professional (prescribing info) or consumer (medication guide)"
                }
            },
            "required": ["query"]
        }),
    )
    // ── SPL document extraction ──────────────────────────────────────────────
    .tool(StationTool {
        name: "get-adverse-reactions-section".to_string(),
        description: "Extract the Adverse Reactions section (Section 6) from an SPL \
        drug label. Returns structured list of adverse events with frequency data \
        from clinical trials and spontaneous reports. Used for expectedness \
        determination: an ADR listed here is 'expected' per US labeling RSI. \
        Requires setId (UUID) from search results."
            .to_string(),
        route: "/dailymed/drugInfo.cfm".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "setId": {
                    "type": "string",
                    "format": "uuid",
                    "description": "DailyMed SPL setId (UUID) from search results"
                }
            },
            "required": ["setId"]
        })),
        tags: vec![
            "adverse-reactions".to_string(),
            "expectedness".to_string(),
            "rsi".to_string(),
            "section-6".to_string(),
        ],
    })
    .tool(StationTool {
        name: "get-warnings-and-precautions-section".to_string(),
        description: "Extract the Warnings and Precautions section (Section 5) from \
        an SPL drug label. Contains boxed warnings, serious but less frequent adverse \
        events, and monitoring requirements. Essential for identifying labeled serious \
        ADRs and risk minimization measures in ICSR causality assessment. \
        Requires setId (UUID)."
            .to_string(),
        route: "/dailymed/drugInfo.cfm".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "setId": {
                    "type": "string",
                    "format": "uuid",
                    "description": "DailyMed SPL setId (UUID) from search results"
                }
            },
            "required": ["setId"]
        })),
        tags: vec![
            "warnings".to_string(),
            "boxed-warning".to_string(),
            "serious-adrs".to_string(),
            "causality".to_string(),
        ],
    })
    .tool(StationTool {
        name: "get-drug-interactions-section".to_string(),
        description: "Extract the Drug Interactions section (Section 7) from an SPL \
        drug label. Covers clinically significant DDIs with mechanism (PK/PD), \
        magnitude, and management recommendations. Used when assessing concomitant \
        medication contributions to reported adverse events in ICSR narratives."
            .to_string(),
        route: "/dailymed/drugInfo.cfm".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "setId": {
                    "type": "string",
                    "format": "uuid",
                    "description": "DailyMed SPL setId (UUID) from search results"
                }
            },
            "required": ["setId"]
        })),
        tags: vec![
            "drug-interactions".to_string(),
            "ddi".to_string(),
            "pk".to_string(),
            "concomitant-medications".to_string(),
        ],
    })
    .tool(StationTool {
        name: "get-full-prescribing-information".to_string(),
        description: "Extract the full Prescribing Information (PI) document for a \
        drug product — all SPL sections including indications, dosage, contraindications, \
        warnings, adverse reactions, drug interactions, use in specific populations, \
        clinical pharmacology, and clinical studies. Use when comprehensive RSI \
        context is needed for complex ICSR causality determinations."
            .to_string(),
        route: "/dailymed/drugInfo.cfm".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "setId": {
                    "type": "string",
                    "format": "uuid",
                    "description": "DailyMed SPL setId (UUID) from search results"
                },
                "format": {
                    "type": "string",
                    "enum": ["full", "structured"],
                    "description": "full = complete text; structured = section-by-section JSON"
                }
            },
            "required": ["setId"]
        })),
        tags: vec![
            "prescribing-information".to_string(),
            "full-label".to_string(),
            "rsi".to_string(),
        ],
    })
    // ── Medication guides ────────────────────────────────────────────────────
    .tool(StationTool {
        name: "get-medication-guide".to_string(),
        description: "Retrieve the FDA-approved Medication Guide for a drug product. \
        Medication guides are patient-facing documents required for drugs with serious \
        and significant public health concerns. Contains plain-language serious warnings \
        that supplement the PI's technical language. Useful for patient-reported ICSR \
        narratives referencing guide language."
            .to_string(),
        route: "/dailymed/drugInfo.cfm".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "setId": {
                    "type": "string",
                    "format": "uuid",
                    "description": "DailyMed SPL setId (UUID) for a medication guide document"
                }
            },
            "required": ["setId"]
        })),
        tags: vec![
            "medication-guide".to_string(),
            "patient-safety".to_string(),
            "consumer-labeling".to_string(),
        ],
    })
    // ── Browse by class ──────────────────────────────────────────────────────
    .fill_tool(
        "browse-by-drug-class",
        "Browse DailyMed drug labels by pharmacological drug class using \
        NDF-RT (National Drug File — Reference Terminology) or MeSH pharmacological \
        action classification. Returns all labeled products in a class — useful for \
        class-effect signal analysis across ICSR databases and for identifying \
        comparator labels during aggregate reporting.",
        "/browse-drug-classes.cfm",
        json!({
            "type": "object",
            "properties": {
                "class_name": {
                    "type": "string",
                    "description": "Drug class name (e.g. 'ACE Inhibitors', 'SGLT2 Inhibitors', 'Anti-TNF Agents')"
                },
                "classification_type": {
                    "type": "string",
                    "enum": ["EPC", "MOA", "PE", "CS"],
                    "description": "FDA classification type: EPC=Established Pharmacologic Class, MOA=Mechanism of Action, PE=Physiologic Effect, CS=Chemical Structure"
                }
            },
            "required": ["class_name"]
        }),
    )
    // ── Labeling history ─────────────────────────────────────────────────────
    .tool(StationTool {
        name: "get-labeling-history".to_string(),
        description: "Retrieve the labeling history for a drug product — all SPL \
        versions with dates and change summaries. Critical for PV case assessment: \
        determines whether an ADR was labeled at the time of occurrence (historical \
        expectedness), tracks label changes driven by safety signals, and supports \
        PBRER and DSUR cumulative safety analyses."
            .to_string(),
        route: "/dailymed/history.cfm".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "setId": {
                    "type": "string",
                    "format": "uuid",
                    "description": "DailyMed SPL setId (UUID) for the product"
                }
            },
            "required": ["setId"]
        })),
        tags: vec![
            "labeling-history".to_string(),
            "label-changes".to_string(),
            "historical-expectedness".to_string(),
            "signal-outcome".to_string(),
        ],
    })
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dailymed_config_domain() {
        let cfg = config();
        assert_eq!(cfg.domain, "dailymed.nlm.nih.gov");
    }

    #[test]
    fn dailymed_config_tool_count() {
        let cfg = config();
        assert!(
            cfg.total_tools() >= 6,
            "expected at least 6 tools, got {}",
            cfg.total_tools()
        );
    }

    #[test]
    fn dailymed_config_required_tags() {
        let cfg = config();
        let tags: Vec<&str> = cfg.tags.iter().map(|s| s.as_str()).collect();
        assert!(tags.contains(&"pharmacovigilance"), "missing pharmacovigilance tag");
        assert!(tags.contains(&"nexvigilant"), "missing nexvigilant tag");
        assert!(tags.contains(&"patient-safety"), "missing patient-safety tag");
    }

    #[test]
    fn dailymed_config_all_tools_public() {
        let cfg = config();
        assert_eq!(cfg.public_tool_count(), cfg.total_tools());
    }

    #[test]
    fn dailymed_config_has_disclaimer() {
        let cfg = config();
        assert!(cfg.description.contains("DISCLAIMER"));
        assert!(cfg.description.contains("NexVigilant"));
    }

    #[test]
    fn dailymed_config_contributor() {
        let cfg = config();
        assert_eq!(cfg.contributor, "MatthewCampCorp");
    }

    #[test]
    fn dailymed_fill_tools_have_schemas() {
        let cfg = config();
        for tool in &cfg.tools {
            if tool.execution_type == crate::config::ExecutionType::Fill {
                assert!(
                    tool.input_schema.is_some(),
                    "fill tool '{}' missing input_schema",
                    tool.name
                );
            }
        }
    }
}
