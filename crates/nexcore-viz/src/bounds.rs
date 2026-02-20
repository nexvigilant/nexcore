//! Bounds Visualization
//!
//! Renders the mathematical concept of bounded values on a number line.
//! Shows the value, its lower/upper bounds, the clamped position,
//! and whether the value is in-bounds or out-of-bounds.
//!
//! Cross-domain: confidence intervals, price limits, range types.

use crate::svg::{self, SvgDoc, palette};

/// A bounded value for visualization.
#[derive(Debug, Clone)]
pub struct BoundedValue {
    /// The actual value
    pub value: f64,
    /// Lower bound (if any)
    pub lower: Option<f64>,
    /// Upper bound (if any)
    pub upper: Option<f64>,
    /// Label for the value
    pub label: String,
}

/// Render a bounded value on a number line.
#[must_use]
pub fn render_bounds(bounded: &BoundedValue) -> String {
    let width = 600.0;
    let height = 200.0;
    let mut doc = SvgDoc::new(width, height);

    let margin = 60.0;
    let line_y = 100.0;
    let line_left = margin;
    let line_right = width - margin;
    let line_width = line_right - line_left;

    // Determine range for the number line
    let lo = bounded
        .lower
        .unwrap_or(bounded.value - 10.0)
        .min(bounded.value)
        - 2.0;
    let hi = bounded
        .upper
        .unwrap_or(bounded.value + 10.0)
        .max(bounded.value)
        + 2.0;
    let range = hi - lo;

    let val_to_x = |v: f64| -> f64 { line_left + ((v - lo) / range) * line_width };

    // Title
    doc.add(svg::text_bold(
        width / 2.0,
        24.0,
        &format!("Bounds: {}", bounded.label),
        14.0,
        palette::TEXT,
        "middle",
    ));

    // Number line axis
    doc.add(svg::line(
        line_left,
        line_y,
        line_right,
        line_y,
        palette::TEXT_DIM,
        1.5,
    ));

    // Bounded region fill
    if let (Some(lower), Some(upper)) = (bounded.lower, bounded.upper) {
        let lx = val_to_x(lower);
        let ux = val_to_x(upper);
        doc.add(svg::rect(
            lx,
            line_y - 20.0,
            ux - lx,
            40.0,
            "#34d39920",
            4.0,
        ));
        doc.add(svg::rect_stroke(
            lx,
            line_y - 20.0,
            ux - lx,
            40.0,
            "none",
            "#34d399",
            1.0,
            4.0,
        ));
    }

    // Lower bound marker
    if let Some(lower) = bounded.lower {
        let x = val_to_x(lower);
        doc.add(svg::line(
            x,
            line_y - 25.0,
            x,
            line_y + 25.0,
            "#34d399",
            2.0,
        ));
        let label = format!("lower: {lower:.1}");
        doc.add(svg::text(
            x,
            line_y + 40.0,
            &label,
            10.0,
            "#34d399",
            "middle",
        ));
    }

    // Upper bound marker
    if let Some(upper) = bounded.upper {
        let x = val_to_x(upper);
        doc.add(svg::line(
            x,
            line_y - 25.0,
            x,
            line_y + 25.0,
            "#34d399",
            2.0,
        ));
        let label = format!("upper: {upper:.1}");
        doc.add(svg::text(
            x,
            line_y + 40.0,
            &label,
            10.0,
            "#34d399",
            "middle",
        ));
    }

    // Check in-bounds
    let in_bounds = bounded.lower.map_or(true, |l| bounded.value >= l)
        && bounded.upper.map_or(true, |u| bounded.value <= u);

    // Value marker
    let vx = val_to_x(bounded.value);
    let value_color = if in_bounds { "#22d3ee" } else { "#ef4444" };
    doc.add(svg::circle(vx, line_y, 8.0, value_color, 1.0));
    let vlabel = format!("{:.1}", bounded.value);
    doc.add(svg::text_bold(
        vx,
        line_y - 30.0,
        &vlabel,
        12.0,
        value_color,
        "middle",
    ));

    // Clamped position (if out of bounds)
    if !in_bounds {
        let clamped = bounded.value.clamp(
            bounded.lower.unwrap_or(f64::NEG_INFINITY),
            bounded.upper.unwrap_or(f64::INFINITY),
        );
        let cx_pos = val_to_x(clamped);
        doc.add(svg::circle_stroke(
            cx_pos, line_y, 8.0, "none", "#fbbf24", 2.0,
        ));
        doc.add(svg::line_dashed(
            vx, line_y, cx_pos, line_y, "#fbbf24", 1.0, "4,2",
        ));
        let clamp_label = format!("clamped: {clamped:.1}");
        doc.add(svg::text(
            cx_pos,
            line_y - 30.0,
            &clamp_label,
            10.0,
            "#fbbf24",
            "middle",
        ));
    }

    // Status
    let status = if in_bounds {
        "\u{2713} IN BOUNDS"
    } else {
        "\u{2717} OUT OF BOUNDS"
    };
    doc.add(svg::text_bold(
        width / 2.0,
        height - 20.0,
        status,
        13.0,
        value_color,
        "middle",
    ));

    // T1 grounding annotation
    doc.add(svg::text(
        width / 2.0,
        50.0,
        "Grounded: \u{2202} Boundary + \u{03c2} State",
        10.0,
        palette::TEXT_DIM,
        "middle",
    ));

    doc.render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_bounds_value() {
        let b = BoundedValue {
            value: 5.0,
            lower: Some(0.0),
            upper: Some(10.0),
            label: "confidence".into(),
        };
        let svg = render_bounds(&b);
        assert!(svg.contains("IN BOUNDS"));
    }

    #[test]
    fn out_of_bounds_value() {
        let b = BoundedValue {
            value: 15.0,
            lower: Some(0.0),
            upper: Some(10.0),
            label: "error rate".into(),
        };
        let svg = render_bounds(&b);
        assert!(svg.contains("OUT OF BOUNDS"));
        assert!(svg.contains("clamped"));
    }

    #[test]
    fn unbounded_always_in_bounds() {
        let b = BoundedValue {
            value: 9999.0,
            lower: None,
            upper: None,
            label: "unbounded".into(),
        };
        let svg = render_bounds(&b);
        assert!(svg.contains("IN BOUNDS"));
    }
}
