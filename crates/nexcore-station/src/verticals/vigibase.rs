//! WHO VigiAccess — global ICSR database config.
//!
//! VigiAccess is the public face of VigiBase, the WHO Programme for International
//! Drug Monitoring's database of over 30 million individual case safety reports
//! spanning 150+ member countries since 1968.
//!
//! Tools cover medicine search, ADR statistics by System Organ Class, country
//! breakdown, age/sex distributions, and seriousness filtering — the full
//! exploratory surface an agent needs for global signal triage.

use serde_json::json;

use crate::config::PvVertical;
use crate::StationBuilder;
use crate::config::StationConfig;

/// Standard tags applied to every VigiBase tool.
const TAGS: &[&str] = &[
    "pharmacovigilance",
    "nexvigilant",
    "patient-safety",
    "who",
    "vigibase",
    "icsr",
    "global-surveillance",
];

/// Build the WHO VigiAccess station config.
///
/// Returns a [`StationConfig`] with 7 tools covering medicine search,
/// ADR frequency tables, SOC browsing, country distribution, age/sex
/// demographics, seriousness classification, and report timeline.
pub fn config() -> StationConfig {
    StationBuilder::new(PvVertical::VigiBase, "WHO VigiAccess — Global ICSR Database")
        .description(
            "Search and analyse over 30 million individual case safety reports \
             from 150+ countries in the WHO VigiBase. Tools support medicine \
             name search, adverse drug reaction (ADR) statistics by System Organ \
             Class (SOC), country of origin breakdown, reporter demographics, \
             seriousness profiling, and temporal trend exploration. \
             Designed for global pharmacovigilance signal triage.",
        )
        .tags(TAGS.iter().copied())
        // Tool 1 — medicine name search (the primary entry point on vigiaccess.org)
        .fill_tool(
            "search-medicine",
            "Search VigiAccess by medicine name or active substance to retrieve the \
             aggregate ADR report count and seriousness summary. Returns the medicine \
             detail page for the matched substance.",
            "/",
            json!({
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "INN, brand name, or active substance (e.g. 'atorvastatin', 'Lipitor')"
                    }
                }
            }),
        )
        // Tool 2 — ADR statistics table for a named medicine
        .fill_tool(
            "get-adr-statistics",
            "Retrieve the full ADR frequency table for a specific medicine from \
             VigiAccess, broken down by System Organ Class (SOC). Returns counts, \
             percentages, and seriousness flags across all reported reactions.",
            "/result",
            json!({
                "type": "object",
                "required": ["substanceName"],
                "properties": {
                    "substanceName": {
                        "type": "string",
                        "description": "Exact substance name as returned by search-medicine"
                    },
                    "reportType": {
                        "type": "string",
                        "enum": ["all", "serious", "non-serious"],
                        "default": "all",
                        "description": "Filter by seriousness classification"
                    }
                }
            }),
        )
        // Tool 3 — browse by SOC (System Organ Class)
        .fill_tool(
            "browse-by-soc",
            "Browse VigiAccess reports for a medicine filtered to a specific MedDRA \
             System Organ Class (SOC), such as 'Cardiac disorders' or \
             'Nervous system disorders'. Returns preferred terms (PTs) ranked by \
             report frequency within that SOC.",
            "/result",
            json!({
                "type": "object",
                "required": ["substanceName", "soc"],
                "properties": {
                    "substanceName": {
                        "type": "string",
                        "description": "Substance name as returned by search-medicine"
                    },
                    "soc": {
                        "type": "string",
                        "description": "MedDRA System Organ Class label (e.g. 'Cardiac disorders')"
                    }
                }
            }),
        )
        // Tool 4 — country distribution
        .fill_tool(
            "get-country-distribution",
            "Retrieve the country-of-origin breakdown for all ICSR reports filed \
             against a medicine in VigiBase. Returns a ranked list of reporting \
             countries with report counts and percentage contribution. Useful for \
             identifying regional signal heterogeneity.",
            "/result",
            json!({
                "type": "object",
                "required": ["substanceName"],
                "properties": {
                    "substanceName": {
                        "type": "string",
                        "description": "Substance name as returned by search-medicine"
                    },
                    "region": {
                        "type": "string",
                        "description": "Optional WHO region filter (e.g. 'AFRO', 'EURO', 'AMRO')"
                    }
                }
            }),
        )
        // Tool 5 — age and sex demographics
        .fill_tool(
            "get-demographics",
            "Extract age group and sex distribution from VigiAccess reports for a \
             given medicine. Returns histogram data for reporter-provided age brackets \
             (0-1, 2-11, 12-17, 18-44, 45-64, 65-74, 75+) and M/F/Unknown sex \
             breakdown. Supports paediatric and geriatric signal assessment.",
            "/result",
            json!({
                "type": "object",
                "required": ["substanceName"],
                "properties": {
                    "substanceName": {
                        "type": "string",
                        "description": "Substance name as returned by search-medicine"
                    },
                    "ageGroup": {
                        "type": "string",
                        "enum": ["0-1", "2-11", "12-17", "18-44", "45-64", "65-74", "75+"],
                        "description": "Optional age group filter"
                    }
                }
            }),
        )
        // Tool 6 — seriousness profile
        .fill_tool(
            "get-seriousness-profile",
            "Retrieve the seriousness outcome breakdown for a medicine's ICSR reports \
             in VigiBase. Returns counts for death, life-threatening, hospitalisation, \
             disabling, congenital anomaly, and other serious outcomes. Critical for \
             PSUR and signal evaluation.",
            "/result",
            json!({
                "type": "object",
                "required": ["substanceName"],
                "properties": {
                    "substanceName": {
                        "type": "string",
                        "description": "Substance name as returned by search-medicine"
                    }
                }
            }),
        )
        // Tool 7 — yearly report timeline
        .fill_tool(
            "get-report-timeline",
            "Retrieve the year-by-year report count for a medicine in VigiBase, \
             spanning from first report to present. Returns an annual series suitable \
             for temporal trend analysis and disproportionality time-to-signal studies.",
            "/result",
            json!({
                "type": "object",
                "required": ["substanceName"],
                "properties": {
                    "substanceName": {
                        "type": "string",
                        "description": "Substance name as returned by search-medicine"
                    },
                    "startYear": {
                        "type": "integer",
                        "description": "Optional start year filter (e.g. 2010)"
                    },
                    "endYear": {
                        "type": "integer",
                        "description": "Optional end year filter (e.g. 2024)"
                    }
                }
            }),
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AccessTier, ExecutionType, PvVertical};

    #[test]
    fn vigibase_config_domain() {
        let cfg = config();
        assert_eq!(cfg.domain, "vigiaccess.org");
        assert_eq!(cfg.vertical, PvVertical::VigiBase);
    }

    #[test]
    fn vigibase_tool_count() {
        let cfg = config();
        assert_eq!(cfg.total_tools(), 7);
    }

    #[test]
    fn vigibase_all_tools_public() {
        let cfg = config();
        assert_eq!(cfg.public_tool_count(), 7);
        assert_eq!(cfg.gated_tool_count(), 0);
        assert_eq!(cfg.premium_tool_count(), 0);
    }

    #[test]
    fn vigibase_all_tools_fill() {
        let cfg = config();
        for tool in &cfg.tools {
            assert_eq!(
                tool.execution_type,
                ExecutionType::Fill,
                "tool '{}' should be Fill (all VigiAccess tools are parameterised searches)",
                tool.name
            );
        }
    }

    #[test]
    fn vigibase_tools_have_input_schema() {
        let cfg = config();
        for tool in &cfg.tools {
            assert!(
                tool.input_schema.is_some(),
                "tool '{}' is missing input_schema",
                tool.name
            );
        }
    }

    #[test]
    fn vigibase_required_tags() {
        let cfg = config();
        let required = ["pharmacovigilance", "nexvigilant", "patient-safety"];
        for tag in required {
            assert!(
                cfg.tags.iter().any(|t| t == tag),
                "config missing required tag '{tag}'"
            );
        }
    }

    #[test]
    fn vigibase_has_disclaimer() {
        let cfg = config();
        assert!(cfg.description.contains("DISCLAIMER"));
        assert!(cfg.description.contains("NexVigilant"));
    }

    #[test]
    fn vigibase_contributor() {
        let cfg = config();
        assert_eq!(cfg.contributor, "MatthewCampCorp");
    }

    #[test]
    fn search_medicine_schema_has_required_query() {
        let cfg = config();
        let tool = cfg.tools.iter().find(|t| t.name == "search-medicine").unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "query"));
    }

    #[test]
    fn get_demographics_schema_lists_age_groups() {
        let cfg = config();
        let tool = cfg.tools.iter().find(|t| t.name == "get-demographics").unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let age_enum = &schema["properties"]["ageGroup"]["enum"];
        assert!(age_enum.is_array());
        assert_eq!(age_enum.as_array().unwrap().len(), 7);
    }
}
