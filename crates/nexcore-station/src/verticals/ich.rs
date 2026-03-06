//! ICH WebMCP Hub config — ICH harmonized tripartite guidelines.
//!
//! Covers the full ICH guideline corpus: E series (efficacy/safety reporting),
//! S series (preclinical safety), M series (multidisciplinary incl. MedDRA),
//! and Q series (quality). Optimized for PV agents navigating E2A–E2F and M1.

use crate::builder::StationBuilder;
use crate::config::{AccessTier, ExecutionType, PvVertical, StationConfig, StationTool};
use serde_json::json;

/// Build the ICH station config for WebMCP Hub.
///
/// Covers guideline browsing by series (E, S, M, Q), full-text search,
/// individual guideline extraction, and step history navigation.
pub fn config() -> StationConfig {
    StationBuilder::new(
        PvVertical::Ich,
        "ICH Harmonized Guidelines — Pharmacovigilance & Safety Reporting",
    )
    .description(
        "Browse and extract ICH harmonized tripartite guidelines covering \
        pharmacovigilance (E2A–E2F), GCP (E6), MedDRA (M1), preclinical \
        safety (S series), and quality (Q series). Essential reference for \
        ICSRs, signal detection, aggregate reporting, and E2B(R3) submission.",
    )
    .tags([
        "pharmacovigilance",
        "nexvigilant",
        "patient-safety",
        "ich",
        "regulatory",
        "harmonization",
        "e2b",
        "icsr",
        "aggregate-reporting",
        "gcp",
    ])
    // ── E series: Efficacy / Safety Reporting ────────────────────────────────
    .fill_tool(
        "search-ich-guidelines",
        "Search ICH guidelines by keyword, series (E/S/M/Q), or guideline code \
        (e.g. E2A, E2B(R3), M1). Returns matching guidelines with step status \
        and links to full text.",
        "/products/guidelines",
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Keyword or guideline code (e.g. 'expedited reporting', 'E2A', 'MedDRA')"
                },
                "series": {
                    "type": "string",
                    "enum": ["E", "S", "M", "Q"],
                    "description": "Guideline series filter"
                },
                "step": {
                    "type": "string",
                    "enum": ["Step 1", "Step 2", "Step 3", "Step 4", "Step 5"],
                    "description": "ICH process step filter (Step 4/5 = finalized)"
                }
            },
            "required": ["query"]
        }),
    )
    .navigate_tool(
        "browse-e-series",
        "Navigate to the ICH E series (Efficacy) guideline index. Covers clinical \
        safety reporting: E2A (expedited reporting), E2B(R3) (ICSR data elements), \
        E2C (PSURs), E2D (post-approval expedited reporting), E2E (pharmacovigilance \
        planning), E2F (development safety update reports). Foundational for PV compliance.",
        "/products/guidelines/efficacy",
    )
    .navigate_tool(
        "browse-s-series",
        "Navigate to the ICH S series (Safety) guideline index. Covers preclinical \
        safety evaluation: genotoxicity (S2), carcinogenicity (S1), reproductive \
        toxicity (S5), immunotoxicity (S8), juvenile animal studies (S11). \
        Critical for linking preclinical findings to clinical ADR signals.",
        "/products/guidelines/safety",
    )
    .navigate_tool(
        "browse-m-series",
        "Navigate to the ICH M series (Multidisciplinary) guideline index. \
        Includes M1 (MedDRA medical terminology), M4 (CTD format), M8 (eCTD), \
        M12 (drug interaction studies). M1 governs MedDRA term selection \
        for ICSR coding.",
        "/products/guidelines/multidisciplinary",
    )
    .navigate_tool(
        "browse-q-series",
        "Navigate to the ICH Q series (Quality) guideline index. Covers \
        pharmaceutical development, stability testing (Q1), specifications (Q6), \
        pharmaceutical quality system (Q10). Referenced in PV for product \
        quality complaints and manufacturing defect ICSRs.",
        "/products/guidelines/quality",
    )
    // ── Individual guideline extraction ──────────────────────────────────────
    .tool(StationTool {
        name: "get-e2b-r3-guideline".to_string(),
        description: "Extract the full E2B(R3) guideline — 'Electronic Transmission of \
        Individual Case Safety Reports (ICSRs)'. The definitive reference for ICSR \
        data element definitions, ICH M2 message structures, and regulatory submission \
        requirements for expedited and periodic reporting."
            .to_string(),
        route: "/products/guidelines/efficacy/e2b".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: None,
        tags: vec![
            "e2b".to_string(),
            "icsr".to_string(),
            "data-elements".to_string(),
            "electronic-submission".to_string(),
        ],
    })
    .tool(StationTool {
        name: "get-e2a-guideline".to_string(),
        description: "Extract the E2A guideline — 'Clinical Safety Data Management: \
        Definitions and Standards for Expedited Reporting'. Defines serious/unexpected \
        ADR criteria, 7-day and 15-day reporting timelines, and the foundational \
        seriousness criteria (death, life-threatening, hospitalization, disability, \
        congenital anomaly, medically important events) used globally."
            .to_string(),
        route: "/products/guidelines/efficacy/e2a".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: None,
        tags: vec![
            "expedited-reporting".to_string(),
            "seriousness".to_string(),
            "causality".to_string(),
            "15-day".to_string(),
            "7-day".to_string(),
        ],
    })
    .tool(StationTool {
        name: "get-e2c-r2-guideline".to_string(),
        description: "Extract the E2C(R2) guideline — 'Periodic Benefit-Risk Evaluation \
        Report (PBRER)'. Defines the structure and content of periodic aggregate safety \
        reports submitted post-approval. Replaces the legacy PSUR format with a \
        benefit-risk framework."
            .to_string(),
        route: "/products/guidelines/efficacy/e2c".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: None,
        tags: vec![
            "pbrer".to_string(),
            "psur".to_string(),
            "aggregate-reporting".to_string(),
            "benefit-risk".to_string(),
        ],
    })
    .tool(StationTool {
        name: "get-e2e-guideline".to_string(),
        description: "Extract the E2E guideline — 'Pharmacovigilance Planning'. \
        Defines the Pharmacovigilance Plan (PVP) requirements for development-stage \
        compounds and the Risk Management Plan (RMP) framework. Covers signal \
        detection methodology, safety specification, and pharmacoepidemiology study design."
            .to_string(),
        route: "/products/guidelines/efficacy/e2e".to_string(),
        execution_type: ExecutionType::Extract,
        access_tier: AccessTier::Public,
        input_schema: None,
        tags: vec![
            "pharmacovigilance-planning".to_string(),
            "risk-management".to_string(),
            "signal-detection".to_string(),
            "rmp".to_string(),
        ],
    })
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ich_config_domain() {
        let cfg = config();
        assert_eq!(cfg.domain, "www.ich.org");
    }

    #[test]
    fn ich_config_tool_count() {
        let cfg = config();
        assert!(
            cfg.total_tools() >= 5,
            "expected at least 5 tools, got {}",
            cfg.total_tools()
        );
    }

    #[test]
    fn ich_config_required_tags() {
        let cfg = config();
        let tags: Vec<&str> = cfg.tags.iter().map(|s| s.as_str()).collect();
        assert!(tags.contains(&"pharmacovigilance"), "missing pharmacovigilance tag");
        assert!(tags.contains(&"nexvigilant"), "missing nexvigilant tag");
        assert!(tags.contains(&"patient-safety"), "missing patient-safety tag");
    }

    #[test]
    fn ich_config_all_tools_public() {
        let cfg = config();
        assert_eq!(cfg.public_tool_count(), cfg.total_tools());
    }

    #[test]
    fn ich_config_has_disclaimer() {
        let cfg = config();
        assert!(
            cfg.description.contains("DISCLAIMER"),
            "missing DISCLAIMER in description"
        );
        assert!(
            cfg.description.contains("NexVigilant"),
            "missing NexVigilant in description"
        );
    }

    #[test]
    fn ich_config_contributor() {
        let cfg = config();
        assert_eq!(cfg.contributor, "MatthewCampCorp");
    }
}
