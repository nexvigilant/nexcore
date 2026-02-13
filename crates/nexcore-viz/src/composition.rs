//! Type Composition Visualization
//!
//! Shows how any type decomposes into T1 Lex Primitiva primitives.
//! Renders a node-link diagram: the type at center, T1 primitives radiating out,
//! with the dominant primitive highlighted and tier classification shown.

use crate::svg::{self, palette, SvgDoc};
use std::fmt::Write;

/// A type's primitive composition for visualization.
#[derive(Debug, Clone)]
pub struct TypeComposition {
    /// Type name (e.g., "Machine<I,O>", "Integrity<T>")
    pub type_name: String,
    /// Tier classification (T1, T2-P, T2-C, T3)
    pub tier: String,
    /// Primitives composing this type
    pub primitives: Vec<PrimitiveNode>,
    /// Dominant primitive name (if any)
    pub dominant: Option<String>,
    /// Confidence in the grounding
    pub confidence: f64,
}

/// A single primitive in a composition.
#[derive(Debug, Clone)]
pub struct PrimitiveNode {
    /// Primitive name (e.g., "Mapping", "Sequence")
    pub name: String,
    /// Unicode symbol
    pub symbol: String,
    /// Role description (e.g., "I -> O transformation")
    pub role: String,
}

/// Render a type composition diagram as SVG.
///
/// The type sits at the center. T1 primitives radiate outward.
/// The dominant primitive gets a thicker connection and glow effect.
/// Tier is shown as a colored ring around the center.
#[must_use]
pub fn render_composition(comp: &TypeComposition) -> String {
    let size = 600.0;
    let cx = size / 2.0;
    let cy = size / 2.0 - 20.0;
    let mut doc = SvgDoc::new(size, size);

    let n = comp.primitives.len();
    if n == 0 {
        doc.add(svg::text(
            cx,
            cy,
            "No primitives",
            16.0,
            palette::TEXT_DIM,
            "middle",
        ));
        return doc.render();
    }

    // Radial layout
    let node_r = 180.0;
    let positions = svg::distribute_circular(cx, cy, node_r, n);

    // Draw connections from center to each primitive
    for (i, pos) in positions.iter().enumerate() {
        let prim = &comp.primitives[i];
        let is_dominant = comp
            .dominant
            .as_ref()
            .is_some_and(|d| d.to_uppercase() == prim.name.to_uppercase());

        let color = palette::grounding_color(&prim.name);
        let sw = if is_dominant { 3.0 } else { 1.5 };
        let opacity = if is_dominant { "1.0" } else { "0.5" };

        // Connection line
        let mut line_el = String::new();
        let _ = write!(
            line_el,
            r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{color}" stroke-width="{sw}" opacity="{opacity}"/>"#,
            cx, cy, pos.0, pos.1
        );
        doc.add(line_el);

        // Primitive node
        let node_size = if is_dominant { 40.0 } else { 32.0 };
        if is_dominant {
            // Glow effect for dominant
            doc.add(svg::circle(pos.0, pos.1, node_size + 6.0, color, 0.15));
        }
        doc.add(svg::circle_stroke(
            pos.0,
            pos.1,
            node_size,
            palette::BG_CARD,
            color,
            2.0,
        ));

        // Symbol inside node
        doc.add(svg::text(
            pos.0,
            pos.1 - 6.0,
            &prim.symbol,
            20.0,
            color,
            "middle",
        ));

        // Name below symbol
        doc.add(svg::text(
            pos.0,
            pos.1 + 12.0,
            &prim.name,
            9.0,
            palette::TEXT_DIM,
            "middle",
        ));

        // Role label (below node)
        if !prim.role.is_empty() {
            doc.add(svg::text(
                pos.0,
                pos.1 + node_size + 14.0,
                &prim.role,
                8.0,
                palette::TEXT_DIM,
                "middle",
            ));
        }

        // Dominant badge
        if is_dominant {
            doc.add(svg::text_bold(
                pos.0,
                pos.1 - node_size - 8.0,
                "DOMINANT",
                8.0,
                color,
                "middle",
            ));
        }
    }

    // Center node: the type itself
    let tier_color = palette::tier_color(&comp.tier);
    doc.add(svg::circle(cx, cy, 60.0, palette::BG_CARD, 1.0));
    doc.add(svg::circle_stroke(cx, cy, 60.0, "none", tier_color, 3.0));

    // Type name (split if long)
    let name = &comp.type_name;
    if name.len() > 16 {
        doc.add(svg::text_bold(
            cx,
            cy - 10.0,
            name,
            12.0,
            palette::TEXT,
            "middle",
        ));
    } else {
        doc.add(svg::text_bold(
            cx,
            cy - 6.0,
            name,
            14.0,
            palette::TEXT,
            "middle",
        ));
    }

    // Tier label
    doc.add(svg::text(
        cx,
        cy + 12.0,
        &comp.tier,
        12.0,
        tier_color,
        "middle",
    ));

    // Confidence
    let conf_text = format!("conf: {:.2}", comp.confidence);
    doc.add(svg::text(
        cx,
        cy + 28.0,
        &conf_text,
        10.0,
        palette::TEXT_DIM,
        "middle",
    ));

    // Title
    doc.add(svg::text_bold(
        cx,
        24.0,
        "Type Composition",
        16.0,
        palette::TEXT,
        "middle",
    ));

    // Primitive count subtitle
    let count_text = format!(
        "{} primitives = {}",
        n, comp.tier
    );
    doc.add(svg::text(
        cx,
        42.0,
        &count_text,
        11.0,
        palette::TEXT_DIM,
        "middle",
    ));

    doc.render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_machine_composition() {
        let comp = TypeComposition {
            type_name: "Machine<I,O>".into(),
            tier: "T3".into(),
            primitives: vec![
                PrimitiveNode { name: "Mapping".into(), symbol: "\u{03bc}".into(), role: "I -> O transformation".into() },
                PrimitiveNode { name: "Sequence".into(), symbol: "\u{03c3}".into(), role: "ordered step chain".into() },
                PrimitiveNode { name: "State".into(), symbol: "\u{03c2}".into(), role: "internal counter".into() },
                PrimitiveNode { name: "Causality".into(), symbol: "\u{2192}".into(), role: "mechanism causal chain".into() },
                PrimitiveNode { name: "Comparison".into(), symbol: "\u{03ba}".into(), role: "determinism".into() },
                PrimitiveNode { name: "Quantity".into(), symbol: "N".into(), role: "confidence, counts".into() },
            ],
            dominant: Some("Mapping".into()),
            confidence: 0.80,
        };
        let svg = render_composition(&comp);
        assert!(svg.contains("Machine"));
        assert!(svg.contains("T3"));
        assert!(svg.contains("DOMINANT"));
    }

    #[test]
    fn render_simple_t1() {
        let comp = TypeComposition {
            type_name: "Force".into(),
            tier: "T1".into(),
            primitives: vec![PrimitiveNode {
                name: "Causality".into(),
                symbol: "\u{2192}".into(),
                role: "cause of acceleration".into(),
            }],
            dominant: Some("Causality".into()),
            confidence: 0.95,
        };
        let svg = render_composition(&comp);
        assert!(svg.contains("Force"));
        assert!(svg.contains("T1"));
    }

    #[test]
    fn empty_composition() {
        let comp = TypeComposition {
            type_name: "Empty".into(),
            tier: "T1".into(),
            primitives: vec![],
            dominant: None,
            confidence: 0.0,
        };
        let svg = render_composition(&comp);
        assert!(svg.contains("No primitives"));
    }
}
