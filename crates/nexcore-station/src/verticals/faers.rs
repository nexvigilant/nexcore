//! WebMCP Hub config for the FDA FAERS / openFDA vertical.
//!
//! Domain: `api.fda.gov`
//!
//! Covers the openFDA REST API — adverse event queries, drug labeling,
//! enforcement actions, and NDC directory lookups. All tools are public
//! (no API key required for the unauthenticated tier).

use crate::builder::StationBuilder;
use crate::config::PvVertical;
use crate::config::StationConfig;
use serde_json::json;

/// Build the FAERS / openFDA station config.
///
/// Produces 9 tools covering the core openFDA endpoints used in
/// pharmacovigilance: ICSR search, drug-event counts, label extraction,
/// enforcement queries, and NDC resolution.
pub fn config() -> StationConfig {
    StationBuilder::new(PvVertical::Faers, "FDA FAERS — openFDA Adverse Event API")
        .description(
            "Query the FDA Adverse Event Reporting System (FAERS) via the openFDA REST API. \
             Supports signal detection, ICSR retrieval, drug-event disproportionality \
             queries, drug labeling extraction, enforcement record lookup, and NDC directory \
             resolution. Use for pharmacovigilance signal mining and regulatory intelligence.",
        )
        .tags([
            "pharmacovigilance",
            "nexvigilant",
            "patient-safety",
            "faers",
            "adverse-events",
            "signal-detection",
            "openfda",
            "icsr",
        ])
        // --- Adverse Event Endpoints ---
        .fill_tool(
            "faers-search-adverse-events",
            "Search FAERS ICSRs by drug name, reaction, or report date. Accepts openFDA \
             query syntax (e.g., patient.drug.medicinalproduct:\"aspirin\"). Returns \
             case-level report data including patient demographics, suspect drugs, and \
             reactions. Primary entry point for signal detection queries.",
            "/drug/event.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "openFDA search query, e.g. \
                            'patient.drug.medicinalproduct:\"warfarin\" AND \
                            patient.reaction.reactionmeddrapt:\"hemorrhage\"'"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Number of results to return (1–1000, default 1)",
                        "minimum": 1,
                        "maximum": 1000,
                        "default": 100
                    },
                    "skip": {
                        "type": "integer",
                        "description": "Offset for pagination (default 0)",
                        "default": 0
                    }
                }
            }),
        )
        .fill_tool(
            "faers-count-drug-reactions",
            "Aggregate FAERS reports by reaction term for a given drug. Returns a ranked \
             frequency table of MedDRA reaction terms — the raw numerator for PRR and \
             ROR disproportionality calculations. Use to build the 2×2 contingency table \
             for signal detection.",
            "/drug/event.json",
            json!({
                "type": "object",
                "required": ["search", "count"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Drug filter, e.g. \
                            'patient.drug.medicinalproduct:\"metformin\"'"
                    },
                    "count": {
                        "type": "string",
                        "description": "Field to count on. Use \
                            'patient.reaction.reactionmeddrapt.exact' for reaction \
                            term frequency tables.",
                        "default": "patient.reaction.reactionmeddrapt.exact"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Top N terms to return (default 100, max 1000)",
                        "default": 100,
                        "maximum": 1000
                    }
                }
            }),
        )
        .fill_tool(
            "faers-count-report-seriousness",
            "Count FAERS reports for a drug stratified by seriousness outcome \
             (hospitalization, death, life-threatening, disability). Supports public \
             health impact assessment during signal prioritization. Returns frequency \
             distribution across outcome fields.",
            "/drug/event.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Drug filter query, e.g. \
                            'patient.drug.openfda.generic_name:\"atorvastatin\"'"
                    },
                    "count": {
                        "type": "string",
                        "description": "Outcome field to stratify on. \
                            Options: 'serious', 'seriousnessdeath', \
                            'seriousnesshospitalization', 'seriousnesslifethreatening', \
                            'seriousnessdisabling'.",
                        "default": "serious"
                    }
                }
            }),
        )
        .fill_tool(
            "faers-reporter-country-breakdown",
            "Count FAERS reports for a drug by reporter country and reporter \
             qualification (physician, consumer, pharmacist). Supports reporting bias \
             analysis — identifies Weber effect patterns, notoriety bias, and \
             stimulated reporting from post-market surveillance programs.",
            "/drug/event.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Drug or reaction filter query"
                    },
                    "count": {
                        "type": "string",
                        "description": "Dimension to aggregate: 'occurcountry.exact' \
                            for geography, 'primarysource.qualification' for \
                            reporter type.",
                        "default": "occurcountry.exact"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                }
            }),
        )
        // --- Drug Labeling Endpoints ---
        .fill_tool(
            "faers-get-drug-label",
            "Retrieve structured drug label (SPL) data via openFDA, including \
             adverse reactions section, boxed warnings, contraindications, \
             postmarket surveillance requirements, and REMS references. \
             Cross-reference against FAERS signals to evaluate labeled vs. \
             unlabeled reactions.",
            "/drug/label.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Label search query, e.g. \
                            'openfda.generic_name:\"methotrexate\"' or \
                            'openfda.brand_name:\"Humira\"'"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Number of label records to return (default 1)",
                        "default": 1,
                        "maximum": 10
                    }
                }
            }),
        )
        .fill_tool(
            "faers-extract-label-warnings",
            "Extract boxed warnings and warnings-and-precautions text from drug \
             labels. Use to assess whether a detected FAERS signal is already \
             labeled — a key step in signal validation per ICH E2C(R2). \
             Returns structured warning text for NLP downstream processing.",
            "/drug/label.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Drug identifier query, e.g. \
                            'openfda.generic_name:\"clozapine\"'"
                    }
                }
            }),
        )
        // --- Enforcement / Recall Endpoints ---
        .fill_tool(
            "faers-search-enforcement-actions",
            "Search FDA enforcement actions (recalls, market withdrawals, safety \
             alerts) for a drug product. Returns recall classification (Class I/II/III), \
             reason for recall, and distribution scope. Useful for validating signals \
             that have progressed to regulatory action.",
            "/drug/enforcement.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Enforcement search query, e.g. \
                            'product_description:\"acetaminophen\"' or \
                            'recalling_firm:\"Pfizer\"'"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 20,
                        "maximum": 100
                    }
                }
            }),
        )
        // --- NDC Directory Endpoint ---
        .fill_tool(
            "faers-resolve-ndc",
            "Look up National Drug Code (NDC) details for a drug product including \
             active ingredients, dosage form, route of administration, marketing \
             category, and application number. Use to canonicalize drug identifiers \
             before querying FAERS and to link to NDA/BLA regulatory dossiers.",
            "/drug/ndc.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "NDC query, e.g. \
                            'generic_name:\"warfarin\"' or \
                            'product_ndc:\"0069-0154\"'"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 10,
                        "maximum": 100
                    }
                }
            }),
        )
        // --- Device Adverse Events ---
        .fill_tool(
            "faers-search-device-events",
            "Search FDA Medical Device Adverse Event reports (MAUDE database) via \
             openFDA. Returns device malfunction, injury, and death reports. \
             Supplements FAERS drug signal detection for combination drug-device \
             products (e.g., auto-injectors, prefilled syringes, drug-eluting stents).",
            "/device/event.json",
            json!({
                "type": "object",
                "required": ["search"],
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Device event query, e.g. \
                            'device.generic_name:\"insulin pump\"' or \
                            'event_type:\"Malfunction\"'"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50,
                        "maximum": 1000
                    },
                    "count": {
                        "type": "string",
                        "description": "Optional count field for aggregation, e.g. \
                            'event_type.exact'",
                        "default": null
                    }
                }
            }),
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PvVertical;

    #[test]
    fn faers_config_domain() {
        let c = config();
        assert_eq!(c.domain, "api.fda.gov");
        assert_eq!(c.vertical, PvVertical::Faers);
    }

    #[test]
    fn faers_config_tool_count() {
        let c = config();
        // 9 tools defined above
        assert_eq!(c.total_tools(), 9);
    }

    #[test]
    fn faers_config_all_public() {
        let c = config();
        assert_eq!(c.public_tool_count(), c.total_tools());
    }

    #[test]
    fn faers_config_has_required_tags() {
        let c = config();
        let tags: Vec<&str> = c.tags.iter().map(|s| s.as_str()).collect();
        assert!(tags.contains(&"pharmacovigilance"));
        assert!(tags.contains(&"nexvigilant"));
        assert!(tags.contains(&"patient-safety"));
    }

    #[test]
    fn faers_config_disclaimer_present() {
        let c = config();
        assert!(c.description.contains("DISCLAIMER"));
        assert!(c.description.contains("NexVigilant"));
    }

    #[test]
    fn faers_fill_tools_have_input_schema() {
        let c = config();
        // Every Fill tool must carry an input schema
        use crate::config::ExecutionType;
        for tool in c
            .tools
            .iter()
            .filter(|t| t.execution_type == ExecutionType::Fill)
        {
            assert!(
                tool.input_schema.is_some(),
                "Fill tool '{}' is missing input_schema",
                tool.name
            );
        }
    }

    #[test]
    fn faers_tool_names_are_kebab_case() {
        let c = config();
        for tool in &c.tools {
            assert!(
                tool.name
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch == '-'),
                "Tool name '{}' is not kebab-case",
                tool.name
            );
        }
    }
}
