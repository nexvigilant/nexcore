//! STEM Taxonomy Visualization
//!
//! Generates a radial sunburst diagram showing:
//! - Inner ring: 4 STEM domains (Science, Chemistry, Physics, Mathematics)
//! - Middle ring: 32 traits, grouped by domain
//! - Outer ring: T1 primitive groundings (color-coded)
//!
//! The visual makes the STEM structure immediately comprehensible:
//! MAPPING dominates (14 traits), showing that transformation is
//! the most universal cross-domain primitive.

use crate::svg::{self, palette, SvgDoc};
use std::fmt::Write;

/// A trait entry for visualization.
#[derive(Debug, Clone)]
pub struct TraitEntry {
    /// Trait name (e.g., "Sense", "Classify")
    pub name: String,
    /// Domain (Science, Chemistry, Physics, Mathematics)
    pub domain: String,
    /// T1 grounding (MAPPING, SEQUENCE, etc.)
    pub grounding: String,
    /// Cross-domain transfer description
    pub transfer: String,
}

/// Generate the STEM taxonomy sunburst SVG.
///
/// Returns a self-contained SVG string showing all 32 traits
/// organized by domain and colored by T1 grounding.
#[must_use]
pub fn render_taxonomy(traits: &[TraitEntry], title: &str) -> String {
    let size = 800.0;
    let cx = size / 2.0;
    let cy = size / 2.0;
    let mut doc = SvgDoc::new(size, size);

    // Title
    doc.add(svg::text_bold(
        cx,
        30.0,
        title,
        18.0,
        palette::TEXT,
        "middle",
    ));

    // Subtitle: trait count and distribution
    let subtitle = format!(
        "{} traits across 4 domains",
        traits.len()
    );
    doc.add(svg::text(
        cx,
        52.0,
        &subtitle,
        12.0,
        palette::TEXT_DIM,
        "middle",
    ));

    // Group traits by domain
    let domains = ["Science", "Chemistry", "Physics", "Mathematics"];
    let mut grouped: Vec<(&str, Vec<&TraitEntry>)> = Vec::new();
    for d in &domains {
        let entries: Vec<&TraitEntry> = traits.iter().filter(|t| t.domain == *d).collect();
        if !entries.is_empty() {
            grouped.push((d, entries));
        }
    }

    let total_traits = traits.len() as f64;
    let inner_r = 80.0;
    let mid_r = 180.0;
    let outer_r = 260.0;
    let label_r = 310.0;

    // Draw rings from inside out
    let mut angle_offset = -90.0; // Start at top

    for (domain, entries) in &grouped {
        let domain_span = (entries.len() as f64 / total_traits) * 360.0;
        let domain_mid = angle_offset + domain_span / 2.0;
        let domain_color = palette::domain_color(domain);

        // Inner ring: domain arc
        doc.add(svg::annular_arc(
            cx,
            cy,
            inner_r - 30.0,
            inner_r,
            angle_offset + 0.5,
            angle_offset + domain_span - 0.5,
            domain_color,
        ));

        // Domain label (inside the inner ring)
        let (lx, ly) = svg::polar_to_cart(cx, cy, inner_r - 50.0, domain_mid);
        doc.add(svg::text_bold(
            lx,
            ly,
            domain,
            10.0,
            palette::TEXT,
            "middle",
        ));

        // Middle ring: individual traits
        let trait_span = domain_span / entries.len() as f64;
        for (i, entry) in entries.iter().enumerate() {
            let t_start = angle_offset + trait_span * i as f64;
            let t_end = t_start + trait_span;
            let t_mid = t_start + trait_span / 2.0;
            let grounding_color = palette::grounding_color(&entry.grounding);

            // Middle ring arc (trait, colored by grounding)
            doc.add(svg::annular_arc(
                cx,
                cy,
                inner_r + 4.0,
                mid_r,
                t_start + 0.3,
                t_end - 0.3,
                grounding_color,
            ));

            // Trait name label
            let (tx, ty) = svg::polar_to_cart(cx, cy, (inner_r + mid_r) / 2.0, t_mid);
            let rotation = if t_mid > 90.0 && t_mid < 270.0 {
                t_mid + 180.0
            } else {
                t_mid
            };
            let mut label = String::new();
            let _ = write!(
                label,
                r#"<text x="{tx:.1}" y="{ty:.1}" font-size="9" fill="{}" text-anchor="middle" dominant-baseline="middle" transform="rotate({rotation:.1},{tx:.1},{ty:.1})">{}</text>"#,
                palette::BG,
                svg::escape_xml(&entry.name)
            );
            doc.add(label);

            // Outer ring: grounding indicator
            doc.add(svg::annular_arc(
                cx,
                cy,
                mid_r + 4.0,
                outer_r,
                t_start + 0.3,
                t_end - 0.3,
                &format!("{grounding_color}40"),
            ));

            // Grounding symbol in outer ring
            let symbol = grounding_symbol(&entry.grounding);
            let (sx, sy) = svg::polar_to_cart(cx, cy, (mid_r + outer_r) / 2.0, t_mid);
            doc.add(svg::text(sx, sy, symbol, 14.0, grounding_color, "middle"));
        }

        angle_offset += domain_span;
    }

    // Center label
    doc.add(svg::circle(cx, cy, inner_r - 34.0, palette::BG_CARD, 1.0));
    doc.add(svg::circle_stroke(
        cx,
        cy,
        inner_r - 34.0,
        "none",
        palette::BORDER,
        1.5,
    ));
    doc.add(svg::text_bold(
        cx,
        cy - 8.0,
        "STEM",
        20.0,
        palette::TEXT,
        "middle",
    ));
    doc.add(svg::text(
        cx,
        cy + 12.0,
        "Primitives",
        12.0,
        palette::TEXT_DIM,
        "middle",
    ));

    // Legend
    render_legend(&mut doc, size);

    doc.render()
}

/// Render the grounding color legend.
fn render_legend(doc: &mut SvgDoc, size: f64) {
    let groundings = [
        ("MAPPING", "mu", palette::MAPPING),
        ("SEQUENCE", "sigma", palette::SEQUENCE),
        ("RECURSION", "rho", palette::RECURSION),
        ("STATE", "varsigma", palette::STATE),
        ("PERSISTENCE", "pi", palette::PERSISTENCE),
        ("BOUNDARY", "partial", palette::BOUNDARY),
        ("SUM", "Sigma", palette::SUM),
    ];

    let legend_x = size - 160.0;
    let legend_y = size - 20.0 - (groundings.len() as f64 * 18.0);

    doc.add(svg::rect(
        legend_x - 10.0,
        legend_y - 16.0,
        170.0,
        groundings.len() as f64 * 18.0 + 28.0,
        palette::BG_CARD,
        6.0,
    ));
    doc.add(svg::text_bold(
        legend_x,
        legend_y,
        "T1 Groundings",
        10.0,
        palette::TEXT,
        "start",
    ));

    for (i, (name, symbol, color)) in groundings.iter().enumerate() {
        let y = legend_y + 18.0 + i as f64 * 18.0;
        doc.add(svg::circle(legend_x + 6.0, y, 5.0, color, 1.0));
        let label = format!("{symbol} {name}");
        doc.add(svg::text(
            legend_x + 18.0,
            y,
            &label,
            10.0,
            palette::TEXT_DIM,
            "start",
        ));
    }
}

/// Get the unicode symbol for a T1 grounding.
#[must_use]
fn grounding_symbol(grounding: &str) -> &'static str {
    match grounding.to_uppercase().as_str() {
        "MAPPING" => "\u{03bc}",     // mu
        "SEQUENCE" => "\u{03c3}",    // sigma
        "RECURSION" => "\u{03c1}",   // rho
        "STATE" => "\u{03c2}",       // varsigma
        "PERSISTENCE" => "\u{03c0}", // pi
        "BOUNDARY" => "\u{2202}",    // partial
        "SUM" => "\u{03a3}",         // Sigma
        _ => "?",
    }
}

/// Build the standard STEM taxonomy entries from the live data.
///
/// This is the canonical 32-trait taxonomy.
#[must_use]
pub fn standard_taxonomy() -> Vec<TraitEntry> {
    vec![
        // Science (7 traits)
        TraitEntry { name: "Sense".into(), domain: "Science".into(), grounding: "MAPPING".into(), transfer: "Environment -> Signal".into() },
        TraitEntry { name: "Classify".into(), domain: "Science".into(), grounding: "MAPPING".into(), transfer: "Signal -> Category".into() },
        TraitEntry { name: "Infer".into(), domain: "Science".into(), grounding: "RECURSION".into(), transfer: "Pattern x Data -> Prediction".into() },
        TraitEntry { name: "Experiment".into(), domain: "Science".into(), grounding: "SEQUENCE".into(), transfer: "Action -> Outcome".into() },
        TraitEntry { name: "Normalize".into(), domain: "Science".into(), grounding: "STATE".into(), transfer: "Prior x Evidence -> Posterior".into() },
        TraitEntry { name: "Codify".into(), domain: "Science".into(), grounding: "MAPPING".into(), transfer: "Belief -> Representation".into() },
        TraitEntry { name: "Extend".into(), domain: "Science".into(), grounding: "MAPPING".into(), transfer: "Source -> Target domain".into() },
        // Chemistry (9 traits)
        TraitEntry { name: "Concentrate".into(), domain: "Chemistry".into(), grounding: "MAPPING".into(), transfer: "Substance -> Ratio".into() },
        TraitEntry { name: "Harmonize".into(), domain: "Chemistry".into(), grounding: "STATE".into(), transfer: "System -> Equilibrium".into() },
        TraitEntry { name: "Energize".into(), domain: "Chemistry".into(), grounding: "MAPPING".into(), transfer: "Energy -> Rate".into() },
        TraitEntry { name: "Modulate".into(), domain: "Chemistry".into(), grounding: "RECURSION".into(), transfer: "Catalyst -> Rate change".into() },
        TraitEntry { name: "Interact".into(), domain: "Chemistry".into(), grounding: "SEQUENCE".into(), transfer: "Ligand -> Affinity binding".into() },
        TraitEntry { name: "Saturate".into(), domain: "Chemistry".into(), grounding: "STATE".into(), transfer: "Capacity -> Fraction".into() },
        TraitEntry { name: "Transform".into(), domain: "Chemistry".into(), grounding: "MAPPING".into(), transfer: "Reactants -> Products".into() },
        TraitEntry { name: "Regulate".into(), domain: "Chemistry".into(), grounding: "RECURSION".into(), transfer: "Inhibitor -> Rate decrease".into() },
        TraitEntry { name: "Yield".into(), domain: "Chemistry".into(), grounding: "MAPPING".into(), transfer: "Actual / Theoretical".into() },
        // Physics (7 traits)
        TraitEntry { name: "Preserve".into(), domain: "Physics".into(), grounding: "PERSISTENCE".into(), transfer: "Quantity unchanged across transform".into() },
        TraitEntry { name: "Harmonics".into(), domain: "Physics".into(), grounding: "RECURSION".into(), transfer: "Oscillation around center".into() },
        TraitEntry { name: "YieldForce".into(), domain: "Physics".into(), grounding: "MAPPING".into(), transfer: "Force -> Acceleration".into() },
        TraitEntry { name: "Superpose".into(), domain: "Physics".into(), grounding: "SUM".into(), transfer: "Sum of parts = whole".into() },
        TraitEntry { name: "Inertia".into(), domain: "Physics".into(), grounding: "PERSISTENCE".into(), transfer: "Resistance to change".into() },
        TraitEntry { name: "Couple".into(), domain: "Physics".into(), grounding: "SEQUENCE".into(), transfer: "Action -> Reaction".into() },
        TraitEntry { name: "Scale".into(), domain: "Physics".into(), grounding: "MAPPING".into(), transfer: "Proportional transform".into() },
        // Mathematics (9 traits)
        TraitEntry { name: "Membership".into(), domain: "Mathematics".into(), grounding: "MAPPING".into(), transfer: "Element in Set".into() },
        TraitEntry { name: "Associate".into(), domain: "Mathematics".into(), grounding: "RECURSION".into(), transfer: "(a*b)*c = a*(b*c)".into() },
        TraitEntry { name: "Transit".into(), domain: "Mathematics".into(), grounding: "SEQUENCE".into(), transfer: "a->b ^ b->c => a->c".into() },
        TraitEntry { name: "Homeomorph".into(), domain: "Mathematics".into(), grounding: "MAPPING".into(), transfer: "Structure-preserving map".into() },
        TraitEntry { name: "Symmetric".into(), domain: "Mathematics".into(), grounding: "MAPPING".into(), transfer: "a~b => b~a".into() },
        TraitEntry { name: "Bound".into(), domain: "Mathematics".into(), grounding: "BOUNDARY".into(), transfer: "Upper/lower limits".into() },
        TraitEntry { name: "Prove".into(), domain: "Mathematics".into(), grounding: "SEQUENCE".into(), transfer: "Premises -> Conclusion".into() },
        TraitEntry { name: "Commute".into(), domain: "Mathematics".into(), grounding: "MAPPING".into(), transfer: "a*b = b*a".into() },
        TraitEntry { name: "Identify".into(), domain: "Mathematics".into(), grounding: "STATE".into(), transfer: "Neutral element".into() },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_taxonomy_has_32_traits() {
        let t = standard_taxonomy();
        assert_eq!(t.len(), 32);
    }

    #[test]
    fn render_produces_valid_svg() {
        let t = standard_taxonomy();
        let svg = render_taxonomy(&t, "STEM Taxonomy");
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("STEM"));
    }

    #[test]
    fn domain_distribution_correct() {
        let t = standard_taxonomy();
        let science = t.iter().filter(|e| e.domain == "Science").count();
        let chemistry = t.iter().filter(|e| e.domain == "Chemistry").count();
        let physics = t.iter().filter(|e| e.domain == "Physics").count();
        let math = t.iter().filter(|e| e.domain == "Mathematics").count();
        assert_eq!(science, 7);
        assert_eq!(chemistry, 9);
        assert_eq!(physics, 7);
        assert_eq!(math, 9);
    }

    #[test]
    fn mapping_dominates_groundings() {
        let t = standard_taxonomy();
        let mapping_count = t.iter().filter(|e| e.grounding == "MAPPING").count();
        assert_eq!(mapping_count, 14);
    }
}
