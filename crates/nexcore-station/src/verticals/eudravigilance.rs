//! WebMCP Hub config for www.adrreports.eu — EudraVigilance ADR Reports.
//!
//! Covers EudraVigilance adverse drug reaction (ADR) line listings, summary
//! tabulations, signal detection data, and MedDRA-coded ICSR statistics for
//! EU/EEA medicinal products.

use crate::builder::StationBuilder;
use crate::config::{PvVertical, StationConfig};
use serde_json::json;

/// Build the EudraVigilance (adrreports.eu) station config.
///
/// Targets `www.adrreports.eu` with 6 tools covering:
/// - ADR line listings by substance and SOC/PT
/// - Summary tabulations (fatalities, serious/non-serious)
/// - Suspected medicine search with MedDRA coding
/// - ICSR counts by reporting country
/// - Age/sex distribution data
/// - Raw case narrative export for signal evaluation
pub fn config() -> StationConfig {
    StationBuilder::new(
        PvVertical::EudraVigilance,
        "EudraVigilance ADR Reports",
    )
    .description(
        "Access EudraVigilance adverse drug reaction (ADR) data via adrreports.eu. \
         Retrieve line listings, summary tabulations, MedDRA-coded ICSR statistics, \
         and signal detection data for EU/EEA authorised medicines. Supports \
         pharmacovigilance signal evaluation, PSUR data collection, and QPPV \
         compliance under EU GVP Module IX and Regulation (EC) No 726/2004.",
    )
    .tags([
        "pharmacovigilance",
        "nexvigilant",
        "patient-safety",
        "eudravigilance",
        "adr",
        "icsr",
        "line-listing",
        "signal-detection",
        "meddra",
        "europe",
        "eu-gvp",
    ])
    // --- ADR line listing by substance
    .fill_tool(
        "adr-line-listing",
        "Retrieve individual case safety report (ICSR) line listings for a suspected \
         medicine from EudraVigilance. Returns case-level data including reaction \
         MedDRA PT, seriousness, outcome, reporter country, and patient demographics. \
         Essential for signal evaluation and PSUR section 16 data assembly. \
         Input: active substance name and optional MedDRA SOC filter.",
        "/en/reports.html",
        json!({
            "type": "object",
            "properties": {
                "substance": {
                    "type": "string",
                    "description": "Active substance or INN as listed in EudraVigilance"
                },
                "soc": {
                    "type": "string",
                    "description": "MedDRA System Organ Class (SOC) to filter reactions (optional)"
                },
                "serious_only": {
                    "type": "boolean",
                    "description": "Restrict to serious adverse reactions only",
                    "default": false
                },
                "year_from": {
                    "type": "integer",
                    "description": "Start year for case inclusion (e.g. 2020)"
                },
                "year_to": {
                    "type": "integer",
                    "description": "End year for case inclusion (e.g. 2024)"
                }
            },
            "required": ["substance"]
        }),
    )
    // --- Summary tabulation (aggregate counts by PT)
    .extract_tool(
        "adr-summary-tabulation",
        "Extract summary tabulation of ADR counts for a medicine by MedDRA System Organ \
         Class and Preferred Term. Returns total ICSRs, serious vs. non-serious split, \
         and fatal case count. Used for disproportionality analysis input and PSUR \
         cumulative safety data tables. Covers all reports in EudraVigilance since \
         first authorisation.",
        "/en/reports.html",
    )
    // --- Suspected medicine search
    .fill_tool(
        "search-suspected-medicine",
        "Search EudraVigilance for a medicine by name or active substance to retrieve \
         the total ICSR count and SOC-level breakdown. Returns the medicine's \
         EudraVigilance identifier, reporting volume by year, and links to detailed \
         line listings. Used for signal triage and benefit-risk monitoring under \
         EU GVP Module IX.",
        "/en/reports.html",
        json!({
            "type": "object",
            "properties": {
                "medicine_name": {
                    "type": "string",
                    "description": "Proprietary name or active substance (INN) to search"
                },
                "match_type": {
                    "type": "string",
                    "enum": ["exact", "contains"],
                    "description": "Name matching strategy",
                    "default": "contains"
                }
            },
            "required": ["medicine_name"]
        }),
    )
    // --- ICSR counts by reporting country
    .extract_tool(
        "icsr-by-country",
        "Extract ICSR submission counts broken down by EU/EEA member state for a \
         given medicine. Identifies geographic clustering of ADR reports — a key \
         signal detection indicator. Useful for QPPV country-level safety monitoring \
         and inspection readiness evidence that surveillance is region-aware.",
        "/en/reports.html",
    )
    // --- Age and sex distribution
    .extract_tool(
        "adr-demographics",
        "Extract patient age group and sex distribution for ICSRs reported against a \
         medicine in EudraVigilance. Returns breakdown across paediatric, adult, and \
         elderly cohorts for both sexes. Supports subgroup safety analyses required \
         in PSURs and RMP pharmacovigilance plans.",
        "/en/reports.html",
    )
    // --- Navigate to EudraVigilance public portal overview
    .navigate_tool(
        "eudravigilance-portal",
        "Navigate to the EudraVigilance public access portal landing page \
         (adrreports.eu). Provides entry to all ADR report types: line listings, \
         summary tabulations, and interactive charts. Use as the starting point \
         before targeted tool calls for substance-specific data.",
        "/en/",
    )
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eudravigilance_config_has_correct_domain() {
        let cfg = config();
        assert_eq!(cfg.domain, "www.adrreports.eu");
    }

    #[test]
    fn eudravigilance_config_has_required_tags() {
        let cfg = config();
        assert!(cfg.tags.contains(&"pharmacovigilance".to_string()));
        assert!(cfg.tags.contains(&"nexvigilant".to_string()));
        assert!(cfg.tags.contains(&"patient-safety".to_string()));
    }

    #[test]
    fn eudravigilance_config_tool_count() {
        let cfg = config();
        assert_eq!(cfg.total_tools(), 6);
    }

    #[test]
    fn eudravigilance_config_has_disclaimer() {
        let cfg = config();
        assert!(cfg.description.contains("DISCLAIMER"));
        assert!(cfg.description.contains("NexVigilant"));
    }

    #[test]
    fn eudravigilance_all_tools_are_public() {
        let cfg = config();
        assert_eq!(cfg.public_tool_count(), 6);
        assert_eq!(cfg.gated_tool_count(), 0);
        assert_eq!(cfg.premium_tool_count(), 0);
    }

    #[test]
    fn eudravigilance_fill_tools_have_schemas() {
        use crate::config::ExecutionType;
        let cfg = config();
        for tool in &cfg.tools {
            if tool.execution_type == ExecutionType::Fill {
                assert!(
                    tool.input_schema.is_some(),
                    "Fill tool '{}' is missing input_schema",
                    tool.name
                );
            }
        }
    }

    #[test]
    fn eudravigilance_tool_names_are_kebab_case() {
        let cfg = config();
        for tool in &cfg.tools {
            assert!(
                !tool.name.contains(' '),
                "Tool name '{}' contains spaces — must be kebab-case",
                tool.name
            );
            assert!(
                !tool.name.contains('_'),
                "Tool name '{}' contains underscores — must be kebab-case",
                tool.name
            );
        }
    }

    #[test]
    fn adr_line_listing_schema_requires_substance() {
        use crate::config::ExecutionType;
        let cfg = config();
        let tool = cfg
            .tools
            .iter()
            .find(|t| t.name == "adr-line-listing")
            .expect("adr-line-listing tool must exist");
        assert_eq!(tool.execution_type, ExecutionType::Fill);
        let schema = tool.input_schema.as_ref().expect("must have schema");
        let required = schema["required"]
            .as_array()
            .expect("required must be array");
        assert!(
            required.iter().any(|v| v == "substance"),
            "substance must be a required field"
        );
    }
}
