//! Platform vertical config — NexVigilant pharmacovigilance platform tools.
//!
//! Exposes the full surface of the NexVigilant portal to AI agents via WebMCP Hub.
//! Public tools cover discovery and onboarding (no auth). Gated tools require a
//! NexVigilant membership and target the Nucleus vigilance workflow pages.

use crate::builder::StationBuilder;
use crate::config::{AccessTier, ExecutionType, PvVertical, StationConfig, StationTool};

/// Build the NexVigilant platform station config.
///
/// Returns a fully-wired [`StationConfig`] with 5 public navigation tools and
/// 13 gated vigilance-workflow tools, tagged for pharmacovigilance discovery.
/// The config ID `ebf48680-488b-4682-bf17-0bf0fc1b6857` is the live WebMCP Hub UUID.
pub fn config() -> StationConfig {
    let mut config = StationBuilder::new(
        PvVertical::Platform,
        "NexVigilant \u{2014} Pharmacovigilance Platform",
    )
    .description(
        "AI-accessible tools for the NexVigilant pharmacovigilance platform. \
        Public tools support discovery and onboarding; gated tools require a \
        NexVigilant membership and surface the full PV workflow: signal detection, \
        causality assessment, benefit-risk evaluation, FAERS queries, and more.",
    )
    .tags(["pharmacovigilance", "pv", "signal-detection", "causality", "faers", "drug-safety"])
    // ── Public navigation tools (no auth required) ───────────────────────────
    .navigate_tool(
        "navigate-services",
        "Navigate to the NexVigilant services overview page.",
        "/services",
    )
    .navigate_tool(
        "navigate-community",
        "Navigate to the NexVigilant community hub.",
        "/community",
    )
    .navigate_tool(
        "navigate-about",
        "Navigate to the NexVigilant about page.",
        "/about",
    )
    .navigate_tool(
        "navigate-academy",
        "Navigate to NexVigilant Academy — pharmacovigilance learning resources.",
        "/academy",
    )
    .navigate_tool(
        "sign-up",
        "Navigate to the NexVigilant sign-up page to create a new account.",
        "/auth/signup",
    )
    // ── Gated vigilance-workflow tools (membership required) ─────────────────
    .tool(StationTool {
        name: "compute-prr-signal".into(),
        description: "Compute Proportional Reporting Ratio (PRR) signal score for a drug-event pair using FAERS data. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/signals".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["signal-detection".into(), "prr".into()],
    })
    .tool(StationTool {
        name: "assess-naranjo-causality".into(),
        description: "Run a Naranjo algorithm causality assessment for an adverse drug reaction. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/causality".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["causality".into(), "naranjo".into()],
    })
    .tool(StationTool {
        name: "classify-case-seriousness".into(),
        description: "Classify the seriousness of an adverse event case per ICH E2A criteria. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/seriousness".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["seriousness".into(), "ich-e2a".into()],
    })
    .tool(StationTool {
        name: "compute-benefit-risk".into(),
        description: "Run a quantitative benefit-risk integration (QBRI) analysis for a drug. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/qbri".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["benefit-risk".into(), "qbri".into()],
    })
    .tool(StationTool {
        name: "query-faers-data".into(),
        description: "Query FDA FAERS adverse event reports for a drug or drug-event pair. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/faers".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["faers".into(), "fda".into()],
    })
    .tool(StationTool {
        name: "assess-drug-safety".into(),
        description: "Run a comprehensive drug safety assessment combining signal detection and causality data. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/drug-safety".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["drug-safety".into()],
    })
    .tool(StationTool {
        name: "assess-expectedness".into(),
        description: "Assess whether an adverse event is expected or unexpected relative to the reference safety information. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/expectedness".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["expectedness".into(), "rsi".into()],
    })
    .tool(StationTool {
        name: "grade-severity".into(),
        description: "Grade the severity of an adverse event using CTCAE or NCI grading criteria. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/severity".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["severity".into(), "ctcae".into()],
    })
    .tool(StationTool {
        name: "resolve-drug-name".into(),
        description: "Resolve a drug name to its canonical form, RxNorm concept, and preferred MedDRA coding. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/drug-resolver".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["drug-resolver".into(), "rxnorm".into()],
    })
    .tool(StationTool {
        name: "open-pv-dashboard".into(),
        description: "Open the NexVigilant pharmacovigilance dashboard with live signal and case summaries. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/dashboard".into(),
        execution_type: ExecutionType::Navigate,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["dashboard".into()],
    })
    .tool(StationTool {
        name: "run-pvdsl-query".into(),
        description: "Execute a PVDSL (pharmacovigilance domain-specific language) query against the NexVigilant signal engine. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/pvdsl".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["pvdsl".into(), "signal-detection".into()],
    })
    .tool(StationTool {
        name: "browse-guidelines".into(),
        description: "Browse ICH, EMA, and FDA pharmacovigilance guidelines indexed in NexVigilant. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/guidelines".into(),
        execution_type: ExecutionType::Navigate,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["guidelines".into(), "ich".into(), "ema".into(), "fda".into()],
    })
    .tool(StationTool {
        name: "lookup-terminology".into(),
        description: "Look up MedDRA terms, WHO-DD drug dictionary entries, and PV terminology definitions. Requires NexVigilant membership.".into(),
        route: "/nucleus/vigilance/terminology".into(),
        execution_type: ExecutionType::Fill,
        access_tier: AccessTier::Gated,
        input_schema: None,
        tags: vec!["terminology".into(), "meddra".into(), "who-dd".into()],
    })
    .build();

    config.id = Some("ebf48680-488b-4682-bf17-0bf0fc1b6857".into());
    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AccessTier;

    #[test]
    fn platform_config_id_set() {
        let c = config();
        assert_eq!(
            c.id.as_deref(),
            Some("ebf48680-488b-4682-bf17-0bf0fc1b6857")
        );
    }

    #[test]
    fn platform_domain_is_nexvigilant() {
        let c = config();
        assert_eq!(c.domain, "nexvigilant.com");
    }

    #[test]
    fn platform_tool_counts() {
        let c = config();
        assert_eq!(c.public_tool_count(), 5, "expected 5 public tools");
        assert_eq!(c.gated_tool_count(), 13, "expected 13 gated tools");
        assert_eq!(c.total_tools(), 18, "expected 18 tools total");
    }

    #[test]
    fn all_gated_tools_have_gated_tier() {
        let c = config();
        let gated_names = [
            "compute-prr-signal",
            "assess-naranjo-causality",
            "classify-case-seriousness",
            "compute-benefit-risk",
            "query-faers-data",
            "assess-drug-safety",
            "assess-expectedness",
            "grade-severity",
            "resolve-drug-name",
            "open-pv-dashboard",
            "run-pvdsl-query",
            "browse-guidelines",
            "lookup-terminology",
        ];
        for name in gated_names {
            let tool = c.tools.iter().find(|t| t.name == name).unwrap_or_else(|| {
                panic!("gated tool '{name}' not found in platform config")
            });
            assert_eq!(
                tool.access_tier,
                AccessTier::Gated,
                "tool '{name}' should be Gated"
            );
        }
    }

    #[test]
    fn all_public_tools_have_public_tier() {
        let c = config();
        let public_names = [
            "navigate-services",
            "navigate-community",
            "navigate-about",
            "navigate-academy",
            "sign-up",
        ];
        for name in public_names {
            let tool = c.tools.iter().find(|t| t.name == name).unwrap_or_else(|| {
                panic!("public tool '{name}' not found in platform config")
            });
            assert_eq!(
                tool.access_tier,
                AccessTier::Public,
                "tool '{name}' should be Public"
            );
        }
    }

    #[test]
    fn platform_config_has_disclaimer() {
        let c = config();
        assert!(
            c.description.contains("DISCLAIMER"),
            "description must contain DISCLAIMER (auto-appended by builder)"
        );
    }

    #[test]
    fn platform_config_vertical_is_platform() {
        let c = config();
        assert_eq!(c.vertical, PvVertical::Platform);
    }
}
