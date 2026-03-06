//! WebMCP Hub config for the FDA.gov regulatory vertical.
//!
//! Domain: `www.fda.gov`
//!
//! Covers the FDA's public-facing web presence — drug safety communications,
//! MedWatch voluntary reporting portal, REMS program database, Drugs@FDA,
//! drug shortage alerts, and post-market safety surveillance resources.
//! All tools are public (no login required).

use crate::builder::StationBuilder;
use crate::config::PvVertical;
use crate::config::StationConfig;
use serde_json::json;

/// Build the FDA.gov station config.
///
/// Produces 9 tools covering FDA regulatory navigation and data extraction:
/// drug safety communications, MedWatch, REMS, Drugs@FDA, drug shortages,
/// FDA calendars, and post-market requirements.
pub fn config() -> StationConfig {
    StationBuilder::new(
        PvVertical::Fda,
        "FDA.gov — Drug Safety & Regulatory Intelligence",
    )
    .description(
        "Navigate and extract pharmacovigilance-relevant content from FDA.gov: \
             drug safety communications, MedWatch voluntary reporting, REMS program \
             listings, Drugs@FDA approval history, drug shortage alerts, and FDA \
             advisory committee calendars. Use for signal validation, regulatory \
             intelligence, and post-market safety monitoring workflows.",
    )
    .tags([
        "pharmacovigilance",
        "nexvigilant",
        "patient-safety",
        "fda",
        "drug-safety",
        "regulatory",
        "rems",
        "medwatch",
        "signal-validation",
    ])
    // --- Drug Safety Communications ---
    .fill_tool(
        "fda-search-safety-communications",
        "Search FDA Drug Safety Communications — official safety labeling changes, \
             market withdrawals, and new risk information. Returns titles, dates, \
             and affected products. Primary source for signal validation: a safety \
             communication confirms regulatory action was taken on a detected signal.",
        "/drugs/drug-safety-and-availability/drug-safety-communications",
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Drug name, reaction, or keyword to filter \
                        safety communications, e.g. 'metformin lactic acidosis'"
                },
                "date_from": {
                    "type": "string",
                    "format": "date",
                    "description": "Filter communications on or after this date \
                        (ISO 8601, e.g. '2020-01-01')"
                },
                "date_to": {
                    "type": "string",
                    "format": "date",
                    "description": "Filter communications on or before this date"
                }
            }
        }),
    )
    // --- MedWatch ---
    .navigate_tool(
        "fda-medwatch-portal",
        "Navigate to MedWatch — FDA's voluntary adverse event reporting portal \
             for healthcare professionals and consumers. Use to access reporting \
             forms (Form FDA 3500 and 3500A), submission instructions, and \
             MedWatch Online reporting workflows. Starting point for any \
             pharmacovigilance reporting workflow.",
        "/safety/medwatch",
    )
    .fill_tool(
        "fda-medwatch-search-alerts",
        "Search MedWatch Safety Alerts — FDA's real-time feed of drug recalls, \
             market withdrawals, and product safety alerts. Returns alerts by drug \
             name, date range, or alert type. Use to cross-reference FAERS signals \
             against official FDA safety actions for signal validation.",
        "/safety/medwatch/safety-alerts",
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Drug name or product description, \
                        e.g. 'acetaminophen' or 'insulin glargine'"
                },
                "alert_type": {
                    "type": "string",
                    "enum": [
                        "recalls",
                        "market-withdrawals",
                        "safety-alerts",
                        "public-health-notifications"
                    ],
                    "description": "Filter by alert category"
                }
            }
        }),
    )
    // --- REMS ---
    .navigate_tool(
        "fda-rems-database",
        "Navigate to the FDA REMS (Risk Evaluation and Mitigation Strategy) \
             database. Lists all active REMS programs with links to full program \
             documentation. Use to determine whether a drug has post-market \
             safety restrictions — a key signal context factor when evaluating \
             whether a detected signal is already risk-managed.",
        "/drugs/drug-safety-and-availability/risk-evaluation-and-mitigation-strategies-rems",
    )
    .fill_tool(
        "fda-search-rems-program",
        "Search for a specific REMS program by drug name. Returns the REMS \
             program name, required elements (ETASU), enrollment requirements, \
             pharmacy certification requirements, and relevant FDA communications. \
             Use during signal prioritization to assess existing risk management.",
        "/drugs/drug-safety-and-availability/risk-evaluation-and-mitigation-strategies-rems",
        json!({
            "type": "object",
            "required": ["drug_name"],
            "properties": {
                "drug_name": {
                    "type": "string",
                    "description": "Generic or brand name of the drug, \
                        e.g. 'clozapine', 'isotretinoin', 'thalidomide'"
                }
            }
        }),
    )
    // --- Drugs@FDA ---
    .fill_tool(
        "fda-drugsatfda-approval-history",
        "Search Drugs@FDA for a drug product's NDA/BLA approval history, \
             original approval date, supplemental approvals, and regulatory \
             documents including medical reviews, approval letters, and labeling. \
             Use to establish the regulatory timeline for signal causality assessment \
             and labeled indication review.",
        "/cder/daf/index.cfm",
        json!({
            "type": "object",
            "properties": {
                "drug_name": {
                    "type": "string",
                    "description": "Generic or brand name to search, \
                        e.g. 'atorvastatin' or 'Lipitor'"
                },
                "application_number": {
                    "type": "string",
                    "description": "NDA or BLA number if known, \
                        e.g. 'NDA021567' or 'BLA125514'"
                }
            }
        }),
    )
    .extract_tool(
        "fda-extract-drug-label-spl",
        "Extract the structured product label (SPL) for a drug directly from \
             FDA.gov Drugs@FDA. Returns sections including Indications and Usage, \
             Contraindications, Warnings and Precautions, Adverse Reactions, and \
             Postmarketing Experience. Use alongside FAERS signal data to assess \
             whether a reaction is labeled or unlabeled.",
        "/drugs/drug-approvals-and-databases/structured-product-labeling-resources",
    )
    // --- Drug Shortages ---
    .fill_tool(
        "fda-drug-shortages",
        "Search the FDA Drug Shortage database for current and resolved \
             drug shortages. Returns shortage status, reason, and available \
             alternatives. Relevant to pharmacovigilance when shortage conditions \
             affect reporting patterns (e.g., substitution-related adverse events, \
             compounding-related incidents).",
        "/drugs/drug-safety-and-availability/drug-shortages",
        json!({
            "type": "object",
            "properties": {
                "drug_name": {
                    "type": "string",
                    "description": "Drug name to search for shortage status, \
                        e.g. 'amoxicillin', 'cisplatin'"
                },
                "status": {
                    "type": "string",
                    "enum": ["current", "resolved", "all"],
                    "description": "Filter by shortage status (default: current)",
                    "default": "current"
                }
            }
        }),
    )
    // --- Advisory Committees ---
    .navigate_tool(
        "fda-advisory-committee-calendar",
        "Navigate to the FDA Advisory Committee calendar. Lists upcoming \
             and past advisory committee meetings across CDER, CBER, CDRH, and \
             other centers. Use during signal management to identify scheduled \
             regulatory reviews of drugs with active signals — advisory committee \
             decisions are key signal disposition milestones.",
        "/advisory-committees/advisory-committee-calendar",
    )
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PvVertical;

    #[test]
    fn fda_config_domain() {
        let c = config();
        assert_eq!(c.domain, "www.fda.gov");
        assert_eq!(c.vertical, PvVertical::Fda);
    }

    #[test]
    fn fda_config_tool_count() {
        let c = config();
        // 9 tools: 5 fill + 3 navigate + 1 extract
        assert_eq!(c.total_tools(), 9);
    }

    #[test]
    fn fda_config_all_public() {
        let c = config();
        assert_eq!(c.public_tool_count(), c.total_tools());
    }

    #[test]
    fn fda_config_has_required_tags() {
        let c = config();
        let tags: Vec<&str> = c.tags.iter().map(|s| s.as_str()).collect();
        assert!(tags.contains(&"pharmacovigilance"));
        assert!(tags.contains(&"nexvigilant"));
        assert!(tags.contains(&"patient-safety"));
    }

    #[test]
    fn fda_config_disclaimer_present() {
        let c = config();
        assert!(c.description.contains("DISCLAIMER"));
        assert!(c.description.contains("NexVigilant"));
    }

    #[test]
    fn fda_fill_tools_have_input_schema() {
        let c = config();
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
    fn fda_tool_names_are_kebab_case() {
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

    #[test]
    fn fda_navigate_tools_have_no_input_schema() {
        let c = config();
        use crate::config::ExecutionType;
        for tool in c
            .tools
            .iter()
            .filter(|t| t.execution_type == ExecutionType::Navigate)
        {
            assert!(
                tool.input_schema.is_none(),
                "Navigate tool '{}' should not have input_schema",
                tool.name
            );
        }
    }
}
