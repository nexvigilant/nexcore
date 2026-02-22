//! Confidence Propagation Visualization
//!
//! Shows how confidence flows through derivation chains.
//! Core rule: conf(child) <= min(conf(parents))
//!
//! Renders a waterfall chart where:
//! - Each bar represents a claim's confidence
//! - Bars are color-coded by proof type
//! - Derivation arrows show parent->child relationships
//! - The system confidence (min of all) is shown at the bottom

use crate::svg::{self, SvgDoc, palette};
use crate::theme::Theme;

/// A claim in a confidence chain.
#[derive(Debug, Clone)]
pub struct Claim {
    /// Claim text
    pub text: String,
    /// Confidence value [0.0, 1.0]
    pub confidence: f64,
    /// Proof type (asserted, computational, analytical, empirical, derived)
    pub proof_type: String,
    /// Index of parent claim (if derived)
    pub parent: Option<usize>,
}

/// Render a confidence propagation waterfall chart.
#[must_use]
pub fn render_confidence_chain(claims: &[Claim], title: &str, theme: &Theme) -> String {
    let bar_h = 36.0;
    let gap = 12.0;
    let left_margin = 200.0;
    let right_margin = 80.0;
    let bar_width = 300.0;
    let top = 70.0;
    let width = left_margin + bar_width + right_margin;
    let height = top + (claims.len() as f64) * (bar_h + gap) + 60.0;

    let mut doc = SvgDoc::new(width, height);

    // Title
    doc.add(svg::text_bold(
        width / 2.0,
        28.0,
        title,
        16.0,
        theme.text,
        "middle",
    ));

    // Subtitle
    doc.add(svg::text(
        width / 2.0,
        48.0,
        "conf(child) \u{2264} min(conf(parents))",
        11.0,
        theme.text_dim,
        "middle",
    ));

    // Background grid lines
    for pct in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let x = left_margin + bar_width * pct;
        doc.add(svg::line_dashed(
            x,
            top - 10.0,
            x,
            height - 40.0,
            theme.border,
            0.5,
            "4,4",
        ));
        let label = format!("{:.0}%", pct * 100.0);
        doc.add(svg::text(
            x,
            top - 16.0,
            &label,
            9.0,
            theme.text_dim,
            "middle",
        ));
    }

    // Render each claim as a horizontal bar
    let mut min_conf = 1.0_f64;

    for (i, claim) in claims.iter().enumerate() {
        let y = top + i as f64 * (bar_h + gap);
        let bar_w = bar_width * claim.confidence.clamp(0.0, 1.0);
        let color = palette::proof_type_color(&claim.proof_type);

        if claim.confidence < min_conf {
            min_conf = claim.confidence;
        }

        // Draw parent->child derivation arrow
        if let Some(parent_idx) = claim.parent {
            if parent_idx < claims.len() {
                let parent_y = top + parent_idx as f64 * (bar_h + gap) + bar_h / 2.0;
                let child_y = y + bar_h / 2.0;
                let arrow_x = left_margin - 20.0;
                doc.add(svg::curved_arrow(
                    arrow_x,
                    parent_y,
                    arrow_x - 14.0,
                    (parent_y + child_y) / 2.0,
                    arrow_x,
                    child_y,
                    &palette::with_alpha(color, "80"),
                    1.5,
                ));
            }
        }

        // Bar
        doc.add(svg::rect(
            left_margin,
            y,
            bar_w,
            bar_h,
            &palette::with_alpha(color, "cc"),
            4.0,
        ));

        // Claim text (left-aligned)
        doc.add(svg::text(
            left_margin - 8.0,
            y + bar_h / 2.0,
            &claim.text,
            10.0,
            theme.text,
            "end",
        ));

        // Confidence value (right of bar)
        let conf_text = format!("{:.2}", claim.confidence);
        doc.add(svg::text_bold(
            left_margin + bar_w + 8.0,
            y + bar_h / 2.0 - 4.0,
            &conf_text,
            11.0,
            color,
            "start",
        ));

        // Proof type label
        doc.add(svg::text(
            left_margin + bar_w + 8.0,
            y + bar_h / 2.0 + 10.0,
            &claim.proof_type,
            8.0,
            theme.text_dim,
            "start",
        ));
    }

    // System confidence line
    let sys_x = left_margin + bar_width * min_conf;
    let sys_y = height - 30.0;
    doc.add(svg::line_dashed(
        sys_x,
        top - 10.0,
        sys_x,
        sys_y,
        theme.danger,
        1.5,
        "6,3",
    ));

    let sys_label = format!("system_conf = {min_conf:.2}");
    doc.add(svg::text_bold(
        sys_x,
        sys_y + 14.0,
        &sys_label,
        11.0,
        theme.danger,
        "middle",
    ));

    doc.render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_confidence_chain_basic() {
        let claims = vec![
            Claim {
                text: "D1: Use Leptos 0.7".into(),
                confidence: 0.95,
                proof_type: "analytical".into(),
                parent: None,
            },
            Claim {
                text: "D2: Firebase REST API".into(),
                confidence: 0.90,
                proof_type: "empirical".into(),
                parent: None,
            },
            Claim {
                text: "D3: Token bucket rate limiter".into(),
                confidence: 0.85,
                proof_type: "derived".into(),
                parent: Some(0),
            },
        ];
        let svg = render_confidence_chain(&claims, "Confidence Propagation", &Theme::default());
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("system_conf"));
        assert!(svg.contains("0.85")); // min confidence
    }

    #[test]
    fn system_conf_is_minimum() {
        let claims = vec![
            Claim {
                text: "A".into(),
                confidence: 0.99,
                proof_type: "asserted".into(),
                parent: None,
            },
            Claim {
                text: "B".into(),
                confidence: 0.70,
                proof_type: "empirical".into(),
                parent: None,
            },
            Claim {
                text: "C".into(),
                confidence: 0.95,
                proof_type: "computational".into(),
                parent: None,
            },
        ];
        let svg = render_confidence_chain(&claims, "Test", &Theme::default());
        assert!(svg.contains("0.70")); // system_conf should be min
    }
}
