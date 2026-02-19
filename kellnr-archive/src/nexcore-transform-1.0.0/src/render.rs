//! Markdown rendering for plan, ledger, and fidelity artifacts.
//!
//! Produces human-readable markdown summaries of transformation artifacts.

use crate::fidelity::FidelityReport;
use crate::ledger::TransferLedger;
use crate::plan::TransformationPlan;

/// Render a transformation plan as markdown.
pub fn render_plan(plan: &TransformationPlan) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Transformation Plan: {}\n\n", plan.plan_id));
    out.push_str(&format!(
        "- **Source:** {} ({} paragraphs, {} words)\n",
        plan.source.title,
        plan.source.paragraphs.len(),
        plan.source.total_words,
    ));
    out.push_str(&format!("- **Source domain:** {}\n", plan.source_domain));
    out.push_str(&format!("- **Target profile:** {}\n", plan.target_profile));
    out.push_str(&format!(
        "- **Total replacements:** {}\n",
        plan.total_replacements()
    ));
    out.push_str(&format!(
        "- **Unmapped concepts:** {}\n",
        plan.mapping_table.unmapped_count
    ));
    out.push_str(&format!(
        "- **Aggregate confidence:** {:.2}\n\n",
        plan.mapping_table.aggregate_confidence
    ));

    out.push_str("## Paragraph Instructions\n\n");
    for inst in &plan.instructions {
        out.push_str(&format!(
            "### Paragraph {} ({:?})\n\n",
            inst.paragraph_index, inst.rhetorical_role
        ));
        out.push_str(&format!("> {}\n\n", truncate(&inst.original_text, 120)));
        if inst.replacements.is_empty() {
            out.push_str("*No concept replacements (passthrough)*\n\n");
        } else {
            out.push_str("| Source | Target |\n|--------|--------|\n");
            for (src, tgt) in &inst.replacements {
                out.push_str(&format!("| {} | {} |\n", src, tgt));
            }
            out.push('\n');
        }
        out.push_str(&format!("**Guidance:** {}\n\n", inst.guidance));
    }

    out
}

/// Render a transfer ledger as markdown.
pub fn render_ledger(ledger: &TransferLedger) -> String {
    let mut out = String::new();

    out.push_str("# Transfer Ledger\n\n");
    out.push_str(&format!(
        "**Total mappings:** {} | **Bridged:** {} | **LLM:** {} | **Unmapped:** {}\n",
        ledger.summary.total,
        ledger.summary.bridged,
        ledger.summary.llm_assisted,
        ledger.summary.unmapped,
    ));
    out.push_str(&format!(
        "**Aggregate confidence:** {:.2}\n\n",
        ledger.summary.aggregate_confidence,
    ));

    if !ledger.entries.is_empty() {
        out.push_str("| Source | Target | Confidence | Method |\n");
        out.push_str("|--------|--------|------------|--------|\n");
        for entry in &ledger.entries {
            out.push_str(&format!(
                "| {} | {} | {:.2} | {} |\n",
                entry.source, entry.target, entry.confidence, entry.method,
            ));
        }
        out.push('\n');
    }

    out
}

/// Render a fidelity report as markdown.
pub fn render_fidelity(report: &FidelityReport) -> String {
    let mut out = String::new();

    out.push_str("# Fidelity Report\n\n");
    out.push_str(&format!("**Plan:** {}\n\n", report.plan_id));
    out.push_str("| Metric | Value |\n|--------|-------|\n");
    out.push_str(&format!(
        "| Source paragraphs | {} |\n",
        report.source_paragraphs
    ));
    out.push_str(&format!(
        "| Output paragraphs | {} |\n",
        report.output_paragraphs
    ));
    out.push_str(&format!(
        "| Paragraph match | {:.2} |\n",
        report.paragraph_match
    ));
    out.push_str(&format!(
        "| Mean coverage | {:.2} |\n",
        report.mean_coverage
    ));
    out.push_str(&format!(
        "| Aggregate confidence | {:.2} |\n",
        report.aggregate_confidence
    ));
    out.push_str(&format!(
        "| **Fidelity score** | **{:.2}** |\n\n",
        report.fidelity_score
    ));

    let grade = if report.fidelity_score >= 0.85 {
        "Excellent"
    } else if report.fidelity_score >= 0.70 {
        "Good"
    } else if report.fidelity_score >= 0.50 {
        "Acceptable"
    } else {
        "Poor"
    };
    out.push_str(&format!("**Grade:** {}\n", grade));

    out
}

/// Truncate text to max length, appending "..." if truncated.
fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        let boundary = text
            .char_indices()
            .take_while(|&(i, _)| i < max.saturating_sub(3))
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &text[..boundary])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fidelity::score_fidelity;
    use crate::ledger::build_ledger;
    use crate::plan::compile_plan;
    use crate::profile::builtin_pharmacovigilance;

    const SAMPLE: &str = "The citizen faces danger.\n\nVigilance is our duty.";

    #[test]
    fn test_render_plan_not_empty() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);
        let md = render_plan(&plan);
        assert!(md.contains("# Transformation Plan"));
        assert!(md.contains("Paragraph Instructions"));
    }

    #[test]
    fn test_render_ledger_table() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);
        let ledger = build_ledger(&plan.mapping_table);
        let md = render_ledger(&ledger);
        assert!(md.contains("# Transfer Ledger"));
        assert!(md.contains("Source | Target"));
    }

    #[test]
    fn test_render_fidelity_grade() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);
        let report = score_fidelity(&plan, 2, &[]);
        let md = render_fidelity(&report);
        assert!(md.contains("# Fidelity Report"));
        assert!(md.contains("Grade:"));
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        let result = truncate("this is a long string that should be truncated", 20);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 23); // 20 + "..."
    }

    #[test]
    fn test_render_plan_contains_profile_info() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);
        let md = render_plan(&plan);
        assert!(md.contains("pharmacovigilance"));
        assert!(md.contains("politics"));
    }
}
