//! Science Method Loop Visualization
//!
//! Renders the 8-step scientific method as a circular flow diagram:
//! SENSE -> CLASSIFY -> INFER -> EXPERIMENT -> NORMALIZE -> CODIFY -> EMIT -> EXTEND
//!          ^                                                                 |
//!          +----------------------------------------------------------------+
//!
//! Each step shows:
//! - The trait name and its T1 grounding symbol
//! - The transformation it performs
//! - Directional arrows connecting the steps
//!
//! The three unfixable limits are annotated at their relevant steps:
//! - Heisenberg at SENSE (observation alters the observed)
//! - Godel at the self-referential loop closure
//! - Shannon at CODIFY (irreducible information loss)

use crate::svg::{self, SvgDoc, palette};
use std::fmt::Write;

/// A step in the science method loop.
#[derive(Debug, Clone)]
pub struct LoopStep {
    /// Trait name
    pub name: String,
    /// T1 grounding
    pub grounding: String,
    /// Greek symbol
    pub symbol: String,
    /// Transformation description
    pub transform: String,
    /// Optional limit annotation (Heisenberg, Godel, Shannon)
    pub limit: Option<String>,
}

/// Render the science method loop as SVG.
#[must_use]
pub fn render_science_loop(steps: &[LoopStep], composite_name: &str) -> String {
    let size = 700.0;
    let cx = size / 2.0;
    let cy = size / 2.0;
    let mut doc = SvgDoc::new(size, size);

    // Title
    doc.add(svg::text_bold(
        cx,
        28.0,
        &format!("The {composite_name} Loop"),
        18.0,
        palette::TEXT,
        "middle",
    ));

    let n = steps.len();
    if n == 0 {
        return doc.render();
    }

    let r = 220.0;
    let node_r = 44.0;
    let positions = svg::distribute_circular(cx, cy, r, n);

    // Draw connecting arrows between steps
    for i in 0..n {
        let next = (i + 1) % n;
        let (x1, y1) = positions[i];
        let (x2, y2) = positions[next];

        // Shorten line to not overlap with nodes
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        let ux = dx / len;
        let uy = dy / len;
        let sx = x1 + ux * (node_r + 8.0);
        let sy = y1 + uy * (node_r + 8.0);
        let ex = x2 - ux * (node_r + 8.0);
        let ey = y2 - uy * (node_r + 8.0);

        let color = palette::grounding_color(&steps[i].grounding);
        doc.add(svg::arrow(sx, sy, ex, ey, &format!("{color}80"), 2.0));
    }

    // Draw step nodes
    for (i, (x, y)) in positions.iter().enumerate() {
        let step = &steps[i];
        let color = palette::grounding_color(&step.grounding);

        // Node circle
        doc.add(svg::circle(*x, *y, node_r, palette::BG_CARD, 1.0));
        doc.add(svg::circle_stroke(*x, *y, node_r, "none", color, 2.5));

        // Step letter (first letter, large)
        let letter = step
            .name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default();
        doc.add(svg::text_bold(
            *x,
            *y - 12.0,
            &letter,
            22.0,
            color,
            "middle",
        ));

        // Grounding symbol
        doc.add(svg::text(
            *x,
            *y + 8.0,
            &step.symbol,
            14.0,
            palette::TEXT_DIM,
            "middle",
        ));

        // Trait name below node
        doc.add(svg::text_bold(
            *x,
            *y + node_r + 14.0,
            &step.name,
            11.0,
            palette::TEXT,
            "middle",
        ));

        // Transform description
        doc.add(svg::text(
            *x,
            *y + node_r + 28.0,
            &step.transform,
            8.0,
            palette::TEXT_DIM,
            "middle",
        ));

        // Limit annotation (warning triangle)
        if let Some(ref limit) = step.limit {
            let lx = *x;
            let ly = *y - node_r - 12.0;
            doc.add(svg::text(lx, ly, limit, 8.0, "#ef4444", "middle"));
        }
    }

    // Center info
    doc.add(svg::circle(cx, cy, 50.0, palette::BG_CARD, 1.0));
    doc.add(svg::circle_stroke(
        cx,
        cy,
        50.0,
        "none",
        palette::BORDER,
        1.5,
    ));
    doc.add(svg::text_bold(
        cx,
        cy - 8.0,
        composite_name,
        14.0,
        palette::TEXT,
        "middle",
    ));
    doc.add(svg::text(
        cx,
        cy + 10.0,
        "T2-C Loop",
        10.0,
        palette::TEXT_DIM,
        "middle",
    ));

    doc.render()
}

/// Build the standard SCIENCE loop steps.
#[must_use]
pub fn science_loop() -> Vec<LoopStep> {
    vec![
        LoopStep {
            name: "Sense".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Environment -> Signal".into(),
            limit: Some("\u{26a0} Heisenberg".into()),
        },
        LoopStep {
            name: "Classify".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Signal -> Category".into(),
            limit: None,
        },
        LoopStep {
            name: "Infer".into(),
            grounding: "RECURSION".into(),
            symbol: "\u{03c1}".into(),
            transform: "Pattern x Data -> Prediction".into(),
            limit: None,
        },
        LoopStep {
            name: "Experiment".into(),
            grounding: "SEQUENCE".into(),
            symbol: "\u{03c3}".into(),
            transform: "Action -> Outcome".into(),
            limit: None,
        },
        LoopStep {
            name: "Normalize".into(),
            grounding: "STATE".into(),
            symbol: "\u{03c2}".into(),
            transform: "Prior x Evidence -> Posterior".into(),
            limit: None,
        },
        LoopStep {
            name: "Codify".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Belief -> Representation".into(),
            limit: Some("\u{26a0} Shannon".into()),
        },
        LoopStep {
            name: "Emit".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "State -> Signal".into(),
            limit: None,
        },
        LoopStep {
            name: "Extend".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Source -> Target Domain".into(),
            limit: Some("\u{26a0} G\u{00f6}del (self-ref)".into()),
        },
    ]
}

/// Build the CHEMISTRY loop steps.
#[must_use]
pub fn chemistry_loop() -> Vec<LoopStep> {
    vec![
        LoopStep {
            name: "Concentrate".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Substance -> Ratio".into(),
            limit: None,
        },
        LoopStep {
            name: "Harmonize".into(),
            grounding: "STATE".into(),
            symbol: "\u{03c2}".into(),
            transform: "System -> Equilibrium".into(),
            limit: None,
        },
        LoopStep {
            name: "Energize".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Energy -> Rate".into(),
            limit: None,
        },
        LoopStep {
            name: "Modulate".into(),
            grounding: "RECURSION".into(),
            symbol: "\u{03c1}".into(),
            transform: "Catalyst -> Rate Change".into(),
            limit: None,
        },
        LoopStep {
            name: "Interact".into(),
            grounding: "SEQUENCE".into(),
            symbol: "\u{03c3}".into(),
            transform: "Ligand -> Affinity".into(),
            limit: None,
        },
        LoopStep {
            name: "Saturate".into(),
            grounding: "STATE".into(),
            symbol: "\u{03c2}".into(),
            transform: "Capacity -> Fraction".into(),
            limit: None,
        },
        LoopStep {
            name: "Transform".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Reactants -> Products".into(),
            limit: None,
        },
        LoopStep {
            name: "Regulate".into(),
            grounding: "RECURSION".into(),
            symbol: "\u{03c1}".into(),
            transform: "Inhibitor -> Rate Decrease".into(),
            limit: None,
        },
        LoopStep {
            name: "Yield".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Actual / Theoretical".into(),
            limit: None,
        },
    ]
}

/// Build the MATHS loop steps.
#[must_use]
pub fn math_loop() -> Vec<LoopStep> {
    vec![
        LoopStep {
            name: "Membership".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Element in Set".into(),
            limit: None,
        },
        LoopStep {
            name: "Associate".into(),
            grounding: "RECURSION".into(),
            symbol: "\u{03c1}".into(),
            transform: "(a*b)*c = a*(b*c)".into(),
            limit: None,
        },
        LoopStep {
            name: "Transit".into(),
            grounding: "SEQUENCE".into(),
            symbol: "\u{03c3}".into(),
            transform: "a->b ^ b->c => a->c".into(),
            limit: None,
        },
        LoopStep {
            name: "Homeomorph".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "Structure-preserving map".into(),
            limit: None,
        },
        LoopStep {
            name: "Symmetric".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "a~b => b~a".into(),
            limit: None,
        },
        LoopStep {
            name: "Bound".into(),
            grounding: "BOUNDARY".into(),
            symbol: "\u{2202}".into(),
            transform: "Upper/lower limits".into(),
            limit: None,
        },
        LoopStep {
            name: "Prove".into(),
            grounding: "SEQUENCE".into(),
            symbol: "\u{03c3}".into(),
            transform: "Premises -> Conclusion".into(),
            limit: Some("\u{26a0} G\u{00f6}del".into()),
        },
        LoopStep {
            name: "Commute".into(),
            grounding: "MAPPING".into(),
            symbol: "\u{03bc}".into(),
            transform: "a*b = b*a".into(),
            limit: None,
        },
        LoopStep {
            name: "Identify".into(),
            grounding: "STATE".into(),
            symbol: "\u{03c2}".into(),
            transform: "Neutral element".into(),
            limit: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn science_loop_has_8_steps() {
        assert_eq!(science_loop().len(), 8);
    }

    #[test]
    fn chemistry_loop_has_9_steps() {
        assert_eq!(chemistry_loop().len(), 9);
    }

    #[test]
    fn math_loop_has_9_steps() {
        assert_eq!(math_loop().len(), 9);
    }

    #[test]
    fn render_science_produces_svg() {
        let steps = science_loop();
        let svg = render_science_loop(&steps, "SCIENCE");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Sense"));
        assert!(svg.contains("Heisenberg"));
        assert!(svg.contains("Shannon"));
    }
}
