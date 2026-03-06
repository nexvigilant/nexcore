//! WebMCP Hub config for www.ema.europa.eu — European Medicines Agency.
//!
//! Covers EMA pharmacovigilance guidance, PRAC signals, medicine database,
//! EPARs, RMPs, and safety referrals.

use crate::builder::StationBuilder;
use crate::config::{PvVertical, StationConfig};
use serde_json::json;

/// Build the EMA station config.
///
/// Targets `www.ema.europa.eu` with 7 tools covering:
/// - PRAC meeting minutes and signal assessments
/// - European Public Assessment Reports (EPARs)
/// - Risk Management Plans (RMPs)
/// - Human medicines database search
/// - Safety referral procedures
/// - Periodic Safety Update Reports (PSURs)
/// - EU pharmacovigilance legislation and GVP modules
pub fn config() -> StationConfig {
    StationBuilder::new(PvVertical::Ema, "EMA Pharmacovigilance Hub")
        .description(
            "Navigate and extract pharmacovigilance resources from the European Medicines Agency \
             (EMA) including PRAC safety signals, EPARs, RMPs, PSURs, GVP modules, and safety \
             referrals. Supports regulatory intelligence workflows for EU GVP compliance.",
        )
        .tags([
            "pharmacovigilance",
            "nexvigilant",
            "patient-safety",
            "ema",
            "prac",
            "eu-gvp",
            "epar",
            "rmp",
            "psur",
            "signal-detection",
            "europe",
        ])
        // --- PRAC signal assessment outcomes
        .extract_tool(
            "prac-signal-assessments",
            "Extract PRAC meeting signal assessment outcomes. Lists medicines reviewed by PRAC \
             for new safety signals, recommended actions (label update, referral, etc.), and \
             assessment conclusions. Essential for regulatory signal tracking under EU GVP \
             Module IX.",
            "/en/about-us/committees/prac/prac-agendas-minutes-highlights",
        )
        // --- PRAC recommendations (full recommendation texts)
        .navigate_tool(
            "prac-recommendations",
            "Navigate to PRAC recommendations on human medicines safety. Covers post-marketing \
             signal recommendations, RMP updates, and PSUR outcomes reported quarterly. Used \
             for QPPV signal intelligence and label change tracking.",
            "/en/medicines/human-regulatory-overview/post-authorisation/pharmacovigilance/prac",
        )
        // --- EPAR search by medicine name
        .fill_tool(
            "search-epar",
            "Search the EMA medicines database for a medicine's European Public Assessment \
             Report (EPAR). Returns product authorisation status, indications, safety \
             updates, and linked RMPs. Input: medicine name or INN.",
            "/en/medicines/search",
            json!({
                "type": "object",
                "properties": {
                    "medicine_name": {
                        "type": "string",
                        "description": "Medicine name, INN, or active substance to search"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["human", "veterinary"],
                        "description": "Medicine category (default: human)",
                        "default": "human"
                    }
                },
                "required": ["medicine_name"]
            }),
        )
        // --- RMP catalogue
        .extract_tool(
            "extract-rmp-catalogue",
            "Extract the EU Risk Management Plan (RMP) public catalogue for a specific \
             medicine. Returns safety concern categories (identified/potential risks, missing \
             information), pharmacovigilance plan activities, and risk minimisation measures. \
             Critical for QPPV RMP oversight.",
            "/en/medicines/human-regulatory-overview/post-authorisation/pharmacovigilance/\
             risk-management-plans-rmps",
        )
        // --- GVP module navigation
        .navigate_tool(
            "gvp-modules",
            "Navigate the EU Good Pharmacovigilance Practice (GVP) module library. Covers all \
             16 GVP modules including Module I (PV systems), Module V (risk management), \
             Module VII (PSUR), Module IX (signal management), and Module XVI (risk \
             minimisation). Regulatory reference for EU PV compliance.",
            "/en/documents/scientific-guideline/guideline-good-pharmacovigilance-practices-gvp",
        )
        // --- Safety referrals (Article 20, Article 31, Article 107)
        .extract_tool(
            "safety-referrals",
            "Extract active and completed EU safety referral procedures under Article 20 \
             (Union interest), Article 31 (community interest), and Article 107 (urgent \
             union procedure). Returns referral trigger, scope, CHMP/PRAC conclusions, and \
             labelling outcomes. Used for signal escalation intelligence.",
            "/en/medicines/human-regulatory-overview/post-authorisation/referral-procedures",
        )
        // --- PSUR work sharing programme
        .fill_tool(
            "search-psur-repository",
            "Search the PSUR repository for Periodic Safety Update Reports submitted under \
             the EU PSUR work sharing programme. Returns assessment report status, DLP \
             (data lock point), and PRAC conclusions by active substance. Input: INN or \
             active substance name.",
            "/en/human-regulatory-overview/post-authorisation/pharmacovigilance/\
             periodic-safety-update-reports-psurs",
            json!({
                "type": "object",
                "properties": {
                    "active_substance": {
                        "type": "string",
                        "description": "INN or active substance name for PSUR lookup"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["ongoing", "completed", "all"],
                        "description": "Assessment status filter",
                        "default": "all"
                    }
                },
                "required": ["active_substance"]
            }),
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ema_config_has_correct_domain() {
        let cfg = config();
        assert_eq!(cfg.domain, "www.ema.europa.eu");
    }

    #[test]
    fn ema_config_has_required_tags() {
        let cfg = config();
        assert!(cfg.tags.contains(&"pharmacovigilance".to_string()));
        assert!(cfg.tags.contains(&"nexvigilant".to_string()));
        assert!(cfg.tags.contains(&"patient-safety".to_string()));
    }

    #[test]
    fn ema_config_tool_count() {
        let cfg = config();
        assert_eq!(cfg.total_tools(), 7);
    }

    #[test]
    fn ema_config_has_disclaimer() {
        let cfg = config();
        assert!(cfg.description.contains("DISCLAIMER"));
        assert!(cfg.description.contains("NexVigilant"));
    }

    #[test]
    fn ema_all_tools_are_public() {
        let cfg = config();
        assert_eq!(cfg.public_tool_count(), 7);
        assert_eq!(cfg.gated_tool_count(), 0);
        assert_eq!(cfg.premium_tool_count(), 0);
    }

    #[test]
    fn ema_fill_tools_have_schemas() {
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
    fn ema_tool_names_are_kebab_case() {
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
}
