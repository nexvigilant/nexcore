//! SVG generation primitives.
//!
//! Low-level SVG building blocks: elements, attributes, coordinate math.
//! All visualizations compose these primitives to produce self-contained SVG strings.

#![allow(dead_code)]

use std::fmt::Write as FmtWrite;

/// An SVG document builder.
pub struct SvgDoc {
    width: f64,
    height: f64,
    elements: Vec<String>,
    defs: Vec<String>,
}

impl SvgDoc {
    /// Create a new SVG document.
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            defs: Vec::new(),
        }
    }

    /// Add a raw SVG element string.
    pub fn add(&mut self, element: String) {
        self.elements.push(element);
    }

    /// Add a definition (goes in `<defs>`).
    pub fn add_def(&mut self, def: String) {
        self.defs.push(def);
    }

    /// Render to complete SVG string.
    #[must_use]
    pub fn render(&self) -> String {
        let mut svg = String::with_capacity(4096);
        let _ = write!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" style="font-family:'Segoe UI',system-ui,sans-serif;background:#0d1117">"#,
            self.width, self.height, self.width, self.height
        );

        if !self.defs.is_empty() {
            svg.push_str("<defs>");
            for d in &self.defs {
                svg.push_str(d);
            }
            svg.push_str("</defs>");
        }

        for el in &self.elements {
            svg.push_str(el);
        }

        svg.push_str("</svg>");
        svg
    }
}

// ============================================================================
// Shape primitives
// ============================================================================

/// Generate a circle element.
#[must_use]
pub fn circle(cx: f64, cy: f64, r: f64, fill: &str, opacity: f64) -> String {
    format!(
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{fill}" opacity="{opacity:.2}"/>"#
    )
}

/// Generate a circle with stroke.
#[must_use]
pub fn circle_stroke(cx: f64, cy: f64, r: f64, fill: &str, stroke: &str, sw: f64) -> String {
    format!(
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{fill}" stroke="{stroke}" stroke-width="{sw:.1}"/>"#
    )
}

/// Generate a line element.
#[must_use]
pub fn line(x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, sw: f64) -> String {
    format!(
        r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{stroke}" stroke-width="{sw:.1}"/>"#
    )
}

/// Generate a line with optional dash pattern.
#[must_use]
pub fn line_dashed(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    stroke: &str,
    sw: f64,
    dash: &str,
) -> String {
    format!(
        r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{stroke}" stroke-width="{sw:.1}" stroke-dasharray="{dash}"/>"#
    )
}

/// Generate a rectangle.
#[must_use]
pub fn rect(x: f64, y: f64, w: f64, h: f64, fill: &str, rx: f64) -> String {
    format!(
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{fill}" rx="{rx:.1}"/>"#
    )
}

/// Generate a rectangle with stroke.
#[must_use]
pub fn rect_stroke(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    fill: &str,
    stroke: &str,
    sw: f64,
    rx: f64,
) -> String {
    format!(
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{fill}" stroke="{stroke}" stroke-width="{sw:.1}" rx="{rx:.1}"/>"#
    )
}

/// Generate text at a position.
#[must_use]
pub fn text(x: f64, y: f64, content: &str, size: f64, fill: &str, anchor: &str) -> String {
    format!(
        r#"<text x="{x:.1}" y="{y:.1}" font-size="{size:.1}" fill="{fill}" text-anchor="{anchor}" dominant-baseline="middle">{}</text>"#,
        escape_xml(content)
    )
}

/// Generate bold text.
#[must_use]
pub fn text_bold(x: f64, y: f64, content: &str, size: f64, fill: &str, anchor: &str) -> String {
    format!(
        r#"<text x="{x:.1}" y="{y:.1}" font-size="{size:.1}" fill="{fill}" text-anchor="{anchor}" dominant-baseline="middle" font-weight="bold">{}</text>"#,
        escape_xml(content)
    )
}

/// Generate a path element.
#[must_use]
pub fn path(d: &str, fill: &str, stroke: &str, sw: f64) -> String {
    format!(r#"<path d="{d}" fill="{fill}" stroke="{stroke}" stroke-width="{sw:.1}"/>"#)
}

/// Generate an arc path segment (for pie/donut slices).
#[must_use]
pub fn arc_path(
    cx: f64,
    cy: f64,
    r: f64,
    start_angle: f64,
    end_angle: f64,
    fill: &str,
    stroke: &str,
    sw: f64,
) -> String {
    let start_rad = start_angle.to_radians();
    let end_rad = end_angle.to_radians();

    let x1 = cx + r * start_rad.cos();
    let y1 = cy + r * start_rad.sin();
    let x2 = cx + r * end_rad.cos();
    let y2 = cy + r * end_rad.sin();

    let large_arc = if (end_angle - start_angle).abs() > 180.0 {
        1
    } else {
        0
    };

    let d = format!(
        "M {cx:.1} {cy:.1} L {x1:.1} {y1:.1} A {r:.1} {r:.1} 0 {large_arc} 1 {x2:.1} {y2:.1} Z"
    );
    path(&d, fill, stroke, sw)
}

/// Generate an annular (ring) arc segment.
#[must_use]
pub fn annular_arc(
    cx: f64,
    cy: f64,
    inner_r: f64,
    outer_r: f64,
    start_angle: f64,
    end_angle: f64,
    fill: &str,
) -> String {
    let s = start_angle.to_radians();
    let e = end_angle.to_radians();

    let ox1 = cx + outer_r * s.cos();
    let oy1 = cy + outer_r * s.sin();
    let ox2 = cx + outer_r * e.cos();
    let oy2 = cy + outer_r * e.sin();
    let ix1 = cx + inner_r * e.cos();
    let iy1 = cy + inner_r * e.sin();
    let ix2 = cx + inner_r * s.cos();
    let iy2 = cy + inner_r * s.sin();

    let large = if (end_angle - start_angle).abs() > 180.0 {
        1
    } else {
        0
    };

    let d = format!(
        "M {ox1:.1} {oy1:.1} A {outer_r:.1} {outer_r:.1} 0 {large} 1 {ox2:.1} {oy2:.1} \
         L {ix1:.1} {iy1:.1} A {inner_r:.1} {inner_r:.1} 0 {large} 0 {ix2:.1} {iy2:.1} Z"
    );
    format!(r#"<path d="{d}" fill="{fill}"/>"#)
}

/// Generate a group with transform.
#[must_use]
pub fn group_open(transform: &str) -> String {
    if transform.is_empty() {
        "<g>".to_string()
    } else {
        format!(r#"<g transform="{transform}">"#)
    }
}

/// Close a group.
#[must_use]
pub fn group_close() -> &'static str {
    "</g>"
}

/// Generate an arrow line (line + arrowhead).
#[must_use]
pub fn arrow(x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, sw: f64) -> String {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.01 {
        return String::new();
    }
    let ux = dx / len;
    let uy = dy / len;

    // Arrowhead size
    let head = 8.0;
    let hx1 = x2 - head * ux + (head * 0.4) * uy;
    let hy1 = y2 - head * uy - (head * 0.4) * ux;
    let hx2 = x2 - head * ux - (head * 0.4) * uy;
    let hy2 = y2 - head * uy + (head * 0.4) * ux;

    let mut s = line(x1, y1, x2, y2, stroke, sw);
    let _ = write!(
        s,
        r#"<polygon points="{x2:.1},{y2:.1} {hx1:.1},{hy1:.1} {hx2:.1},{hy2:.1}" fill="{stroke}"/>"#
    );
    s
}

/// Generate a curved arrow (quadratic bezier).
#[must_use]
pub fn curved_arrow(
    x1: f64,
    y1: f64,
    cx: f64,
    cy: f64,
    x2: f64,
    y2: f64,
    stroke: &str,
    sw: f64,
) -> String {
    let d = format!("M {x1:.1} {y1:.1} Q {cx:.1} {cy:.1} {x2:.1} {y2:.1}");
    format!(r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{sw:.1}"/>"#)
}

/// Escape XML special characters.
#[must_use]
pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ============================================================================
// Color palette (dark theme, accessible)
// ============================================================================

/// Domain colors for STEM visualization.
pub mod palette {
    /// Science domain — teal
    pub const SCIENCE: &str = "#2dd4bf";
    /// Chemistry domain — amber
    pub const CHEMISTRY: &str = "#fbbf24";
    /// Physics domain — violet
    pub const PHYSICS: &str = "#a78bfa";
    /// Mathematics domain — rose
    pub const MATHEMATICS: &str = "#fb7185";

    /// T1 primitive colors
    pub const MAPPING: &str = "#60a5fa";
    pub const SEQUENCE: &str = "#34d399";
    pub const RECURSION: &str = "#f472b6";
    pub const STATE: &str = "#facc15";
    pub const PERSISTENCE: &str = "#c084fc";
    pub const BOUNDARY: &str = "#f87171";
    pub const SUM: &str = "#38bdf8";

    /// Tier colors
    pub const TIER_T1: &str = "#22d3ee";
    pub const TIER_T2P: &str = "#a3e635";
    pub const TIER_T2C: &str = "#fbbf24";
    pub const TIER_T3: &str = "#f97316";

    /// Background and text
    pub const BG: &str = "#0d1117";
    pub const BG_CARD: &str = "#161b22";
    pub const TEXT: &str = "#e6edf3";
    pub const TEXT_DIM: &str = "#8b949e";
    pub const BORDER: &str = "#30363d";

    /// Get domain color by name.
    #[must_use]
    pub fn domain_color(domain: &str) -> &'static str {
        match domain.to_lowercase().as_str() {
            "science" => SCIENCE,
            "chemistry" => CHEMISTRY,
            "physics" => PHYSICS,
            "mathematics" | "math" => MATHEMATICS,
            _ => TEXT_DIM,
        }
    }

    /// Get T1 grounding color by name.
    #[must_use]
    pub fn grounding_color(grounding: &str) -> &'static str {
        match grounding.to_uppercase().as_str() {
            "MAPPING" => MAPPING,
            "SEQUENCE" => SEQUENCE,
            "RECURSION" => RECURSION,
            "STATE" => STATE,
            "PERSISTENCE" => PERSISTENCE,
            "BOUNDARY" => BOUNDARY,
            "SUM" => SUM,
            _ => TEXT_DIM,
        }
    }

    /// Get tier color.
    #[must_use]
    pub fn tier_color(tier: &str) -> &'static str {
        match tier {
            "T1" => TIER_T1,
            "T2-P" => TIER_T2P,
            "T2-C" => TIER_T2C,
            "T3" => TIER_T3,
            _ => TEXT_DIM,
        }
    }
}

// ============================================================================
// Coordinate math
// ============================================================================

/// Polar to cartesian conversion.
#[must_use]
pub fn polar_to_cart(cx: f64, cy: f64, r: f64, angle_deg: f64) -> (f64, f64) {
    let rad = angle_deg.to_radians();
    (cx + r * rad.cos(), cy + r * rad.sin())
}

/// Distribute N items evenly around a circle.
#[must_use]
pub fn distribute_circular(cx: f64, cy: f64, r: f64, n: usize) -> Vec<(f64, f64)> {
    if n == 0 {
        return vec![];
    }
    let step = 360.0 / n as f64;
    (0..n)
        .map(|i| polar_to_cart(cx, cy, r, -90.0 + step * i as f64))
        .collect()
}
