//! SVG generation primitives.
//!
//! Low-level SVG building blocks: elements, attributes, coordinate math.
//! All visualizations compose these primitives to produce self-contained SVG strings.

#![allow(dead_code)]

use crate::theme::Theme;
use std::fmt::Write as FmtWrite;

/// An SVG document builder.
pub struct SvgDoc {
    width: f64,
    height: f64,
    elements: Vec<String>,
    defs: Vec<String>,
    theme: Theme,
}

impl SvgDoc {
    /// Create a new SVG document with the default dark theme.
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            defs: Vec::new(),
            theme: Theme::dark(),
        }
    }

    /// Create a new SVG document with a specific theme.
    #[must_use]
    pub fn new_with_theme(width: f64, height: f64, theme: Theme) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            defs: Vec::new(),
            theme,
        }
    }

    /// Get a reference to this document's theme.
    #[must_use]
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Add a raw SVG element string.
    pub fn add(&mut self, element: String) {
        self.elements.push(element);
    }

    /// Add a definition (goes in `<defs>`).
    pub fn add_def(&mut self, def: String) {
        self.defs.push(def);
    }

    /// Add an arrowhead marker definition.
    pub fn add_arrowhead(&mut self, id: &str, color: &str, size: f64) {
        self.add_def(arrowhead_marker(id, color, size));
    }

    /// Render to complete SVG string.
    #[must_use]
    pub fn render(&self) -> String {
        let mut svg = String::with_capacity(4096);
        let style = self.theme.svg_style();
        let _ = write!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" style="{style}">"#,
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
    DashedLine::new(x1, y1, x2, y2)
        .color(stroke)
        .stroke_width(sw)
        .dash(dash)
        .to_string()
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
#[allow(clippy::too_many_arguments)]
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
    StrokeRect::new(x, y, w, h)
        .fill(fill)
        .stroke(stroke)
        .stroke_width(sw)
        .rx(rx)
        .to_string()
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
#[allow(clippy::too_many_arguments)]
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
    ArcPath::new(cx, cy, r, start_angle, end_angle)
        .fill(fill)
        .stroke(stroke)
        .stroke_width(sw)
        .to_string()
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
    AnnularArcBuilder::new(cx, cy, inner_r, outer_r, start_angle, end_angle)
        .fill(fill)
        .to_string()
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
#[allow(clippy::too_many_arguments)]
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
    CurvedArrowBuilder::new(x1, y1, cx, cy, x2, y2)
        .color(stroke)
        .stroke_width(sw)
        .to_string()
}

/// Generate an SVG `<marker>` definition for arrowheads.
#[must_use]
pub fn arrowhead_marker(id: &str, color: &str, size: f64) -> String {
    format!(
        r#"<marker id="{id}" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="{size:.1}" markerHeight="{size:.1}" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="{color}"/></marker>"#
    )
}

/// Generate a line referencing a marker-end arrowhead.
#[must_use]
pub fn arrow_with_marker(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    stroke: &str,
    sw: f64,
    marker_id: &str,
) -> String {
    format!(
        r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{stroke}" stroke-width="{sw:.1}" marker-end="url(#{marker_id})"/>"#
    )
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
// Builder patterns for high-parameter functions
// ============================================================================

/// Builder for dashed line elements.
pub struct DashedLine {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color_val: String,
    stroke_width_val: f64,
    dash_val: String,
}

impl Default for DashedLine {
    fn default() -> Self {
        Self {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            color_val: palette::SLATE.to_string(),
            stroke_width_val: 2.0,
            dash_val: "4,4".to_string(),
        }
    }
}

impl DashedLine {
    #[must_use]
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn color(mut self, c: &str) -> Self {
        self.color_val = c.to_string();
        self
    }

    #[must_use]
    pub fn stroke_width(mut self, sw: f64) -> Self {
        self.stroke_width_val = sw;
        self
    }

    #[must_use]
    pub fn dash(mut self, d: &str) -> Self {
        self.dash_val = d.to_string();
        self
    }
}

impl std::fmt::Display for DashedLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{:.1}" stroke-dasharray="{}"/>"#,
            self.x1,
            self.y1,
            self.x2,
            self.y2,
            self.color_val,
            self.stroke_width_val,
            self.dash_val
        )
    }
}

/// Builder for stroked rectangle elements.
pub struct StrokeRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    fill_val: String,
    stroke_val: String,
    stroke_width_val: f64,
    rx_val: f64,
}

impl Default for StrokeRect {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            fill_val: palette::BG_CARD.to_string(),
            stroke_val: palette::SLATE.to_string(),
            stroke_width_val: 2.0,
            rx_val: 0.0,
        }
    }
}

impl StrokeRect {
    #[must_use]
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            x,
            y,
            w,
            h,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn fill(mut self, f: &str) -> Self {
        self.fill_val = f.to_string();
        self
    }

    #[must_use]
    pub fn stroke(mut self, s: &str) -> Self {
        self.stroke_val = s.to_string();
        self
    }

    #[must_use]
    pub fn stroke_width(mut self, sw: f64) -> Self {
        self.stroke_width_val = sw;
        self
    }

    #[must_use]
    pub fn rx(mut self, rx: f64) -> Self {
        self.rx_val = rx;
        self
    }
}

impl std::fmt::Display for StrokeRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" stroke="{}" stroke-width="{:.1}" rx="{:.1}"/>"#,
            self.x,
            self.y,
            self.w,
            self.h,
            self.fill_val,
            self.stroke_val,
            self.stroke_width_val,
            self.rx_val
        )
    }
}

/// Builder for arc path segments (pie/donut slices).
pub struct ArcPath {
    cx: f64,
    cy: f64,
    r: f64,
    start_angle: f64,
    end_angle: f64,
    fill_val: String,
    stroke_val: String,
    stroke_width_val: f64,
}

impl Default for ArcPath {
    fn default() -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            r: 0.0,
            start_angle: 0.0,
            end_angle: 0.0,
            fill_val: palette::SLATE.to_string(),
            stroke_val: "none".to_string(),
            stroke_width_val: 2.0,
        }
    }
}

impl ArcPath {
    #[must_use]
    pub fn new(cx: f64, cy: f64, r: f64, start_angle: f64, end_angle: f64) -> Self {
        Self {
            cx,
            cy,
            r,
            start_angle,
            end_angle,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn fill(mut self, f: &str) -> Self {
        self.fill_val = f.to_string();
        self
    }

    #[must_use]
    pub fn stroke(mut self, s: &str) -> Self {
        self.stroke_val = s.to_string();
        self
    }

    #[must_use]
    pub fn stroke_width(mut self, sw: f64) -> Self {
        self.stroke_width_val = sw;
        self
    }
}

impl std::fmt::Display for ArcPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();

        let x1 = self.cx + self.r * start_rad.cos();
        let y1 = self.cy + self.r * start_rad.sin();
        let x2 = self.cx + self.r * end_rad.cos();
        let y2 = self.cy + self.r * end_rad.sin();

        let large_arc = if (self.end_angle - self.start_angle).abs() > 180.0 {
            1
        } else {
            0
        };

        let d = format!(
            "M {:.1} {:.1} L {x1:.1} {y1:.1} A {:.1} {:.1} 0 {large_arc} 1 {x2:.1} {y2:.1} Z",
            self.cx, self.cy, self.r, self.r
        );
        write!(
            f,
            r#"<path d="{d}" fill="{}" stroke="{}" stroke-width="{:.1}"/>"#,
            self.fill_val, self.stroke_val, self.stroke_width_val
        )
    }
}

/// Builder for annular (ring) arc segments.
pub struct AnnularArcBuilder {
    cx: f64,
    cy: f64,
    inner_r: f64,
    outer_r: f64,
    start_angle: f64,
    end_angle: f64,
    fill_val: String,
}

impl Default for AnnularArcBuilder {
    fn default() -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            inner_r: 0.0,
            outer_r: 0.0,
            start_angle: 0.0,
            end_angle: 0.0,
            fill_val: palette::SLATE.to_string(),
        }
    }
}

impl AnnularArcBuilder {
    #[must_use]
    pub fn new(
        cx: f64,
        cy: f64,
        inner_r: f64,
        outer_r: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Self {
        Self {
            cx,
            cy,
            inner_r,
            outer_r,
            start_angle,
            end_angle,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn fill(mut self, f: &str) -> Self {
        self.fill_val = f.to_string();
        self
    }
}

impl std::fmt::Display for AnnularArcBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.start_angle.to_radians();
        let e = self.end_angle.to_radians();

        let ox1 = self.cx + self.outer_r * s.cos();
        let oy1 = self.cy + self.outer_r * s.sin();
        let ox2 = self.cx + self.outer_r * e.cos();
        let oy2 = self.cy + self.outer_r * e.sin();
        let ix1 = self.cx + self.inner_r * e.cos();
        let iy1 = self.cy + self.inner_r * e.sin();
        let ix2 = self.cx + self.inner_r * s.cos();
        let iy2 = self.cy + self.inner_r * s.sin();

        let large = if (self.end_angle - self.start_angle).abs() > 180.0 {
            1
        } else {
            0
        };

        let d = format!(
            "M {ox1:.1} {oy1:.1} A {:.1} {:.1} 0 {large} 1 {ox2:.1} {oy2:.1} \
             L {ix1:.1} {iy1:.1} A {:.1} {:.1} 0 {large} 0 {ix2:.1} {iy2:.1} Z",
            self.outer_r, self.outer_r, self.inner_r, self.inner_r
        );
        write!(f, r#"<path d="{d}" fill="{}"/>"#, self.fill_val)
    }
}

/// Builder for curved arrow (quadratic bezier) elements.
pub struct CurvedArrowBuilder {
    x1: f64,
    y1: f64,
    cx: f64,
    cy: f64,
    x2: f64,
    y2: f64,
    color_val: String,
    stroke_width_val: f64,
}

impl Default for CurvedArrowBuilder {
    fn default() -> Self {
        Self {
            x1: 0.0,
            y1: 0.0,
            cx: 0.0,
            cy: 0.0,
            x2: 0.0,
            y2: 0.0,
            color_val: palette::SLATE.to_string(),
            stroke_width_val: 2.0,
        }
    }
}

impl CurvedArrowBuilder {
    #[must_use]
    pub fn new(x1: f64, y1: f64, cx: f64, cy: f64, x2: f64, y2: f64) -> Self {
        Self {
            x1,
            y1,
            cx,
            cy,
            x2,
            y2,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn color(mut self, c: &str) -> Self {
        self.color_val = c.to_string();
        self
    }

    #[must_use]
    pub fn stroke_width(mut self, sw: f64) -> Self {
        self.stroke_width_val = sw;
        self
    }
}

impl std::fmt::Display for CurvedArrowBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = format!(
            "M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}",
            self.x1, self.y1, self.cx, self.cy, self.x2, self.y2
        );
        write!(
            f,
            r#"<path d="{d}" fill="none" stroke="{}" stroke-width="{:.1}"/>"#,
            self.color_val, self.stroke_width_val
        )
    }
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

    /// Semantic: safe/pass indicator — emerald green
    pub const EMERALD: &str = "#34d399";
    /// Semantic: warning/clamped — amber
    pub const AMBER: &str = "#fbbf24";
    /// Semantic: info/value highlight — light cyan
    pub const CYAN_LIGHT: &str = "#22d3ee";
    /// Semantic: error/danger — red
    pub const RED: &str = "#ef4444";
    /// Semantic: neutral default stroke — slate
    pub const SLATE: &str = "#64748b";

    /// Append a hex alpha suffix to a hex color.
    #[must_use]
    pub fn with_alpha(hex: &str, alpha_hex: &str) -> String {
        format!("{hex}{alpha_hex}")
    }

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

    /// Get color for proof/evidence type.
    #[must_use]
    pub fn proof_type_color(kind: &str) -> &'static str {
        match kind.to_lowercase().as_str() {
            "asserted" => CYAN_LIGHT,
            "computational" => EMERALD,
            "analytical" => PHYSICS,
            "mapping" => MAPPING,
            "adversarial" => RECURSION,
            "empirical" => AMBER,
            "derived" => "#fb923c",
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
