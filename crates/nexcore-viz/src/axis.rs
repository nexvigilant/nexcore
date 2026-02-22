//! Axis Rendering
//!
//! Renders labeled axes using Scale + TextMetrics. Produces axis lines,
//! tick marks, tick labels, and optional grid lines as SVG elements
//! added to an `SvgDoc`.
//!
//! Four orientations: Bottom (x-axis below), Top (x-axis above),
//! Left (y-axis left), Right (y-axis right).
//!
//! Grounded: σ (Sequence) ordered ticks, μ (Mapping) value→label,
//!           ∂ (Boundary) axis extent, N (Quantity) tick values.

use crate::metrics;
use crate::scale::Scale;
use crate::svg::{self, SvgDoc};
use crate::theme::Theme;
use std::fmt::Write;

/// Axis orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Horizontal axis below the chart area.
    Bottom,
    /// Horizontal axis above the chart area.
    Top,
    /// Vertical axis to the left of the chart area.
    Left,
    /// Vertical axis to the right of the chart area.
    Right,
}

impl Orientation {
    /// Is this a horizontal axis?
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }
}

/// How to format tick labels.
#[derive(Debug, Clone, Default)]
pub enum TickFormat {
    /// Automatic: use `format_tick()` from scale module.
    #[default]
    Auto,
    /// Fixed decimal places.
    Fixed(usize),
    /// Percentage (multiply by 100, add %).
    Percent,
    /// Custom format string (uses `format!` with the value).
    Custom(String),
}

impl TickFormat {
    fn format(&self, value: f64) -> String {
        match self {
            Self::Auto => crate::scale::format_tick(value),
            Self::Fixed(decimals) => format!("{:.prec$}", value, prec = *decimals),
            Self::Percent => crate::scale::format_percent(value),
            Self::Custom(fmt) => fmt.replace("{}", &format!("{value}")),
        }
    }
}

/// Bundled context for internal axis render methods.
struct AxisCtx<'a> {
    scale: &'a dyn Scale,
    anchor: f64,
    theme: &'a Theme,
    tick_values: &'a [f64],
    range_start: f64,
    range_end: f64,
}

/// Configuration for an axis.
#[derive(Debug, Clone)]
pub struct Axis {
    /// Orientation (Bottom, Top, Left, Right).
    pub orientation: Orientation,
    /// Number of desired tick marks.
    pub tick_count: usize,
    /// Tick mark length in pixels.
    pub tick_size: f64,
    /// How to format tick labels.
    pub tick_format: TickFormat,
    /// Optional axis label (e.g., "Confidence", "Time (s)").
    pub label: Option<String>,
    /// Whether to draw grid lines across the chart area.
    pub grid: bool,
    /// Grid line extent (how far grid lines extend from the axis).
    pub grid_extent: f64,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            orientation: Orientation::Bottom,
            tick_count: 5,
            tick_size: 6.0,
            tick_format: TickFormat::default(),
            label: None,
            grid: false,
            grid_extent: 0.0,
        }
    }
}

impl Axis {
    /// Create an axis with the given orientation.
    #[must_use]
    pub fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            ..Default::default()
        }
    }

    /// Bottom axis (horizontal, below chart).
    #[must_use]
    pub fn bottom() -> Self {
        Self::new(Orientation::Bottom)
    }

    /// Top axis (horizontal, above chart).
    #[must_use]
    pub fn top() -> Self {
        Self::new(Orientation::Top)
    }

    /// Left axis (vertical, left of chart).
    #[must_use]
    pub fn left() -> Self {
        Self::new(Orientation::Left)
    }

    /// Right axis (vertical, right of chart).
    #[must_use]
    pub fn right() -> Self {
        Self::new(Orientation::Right)
    }

    /// Set the number of ticks.
    #[must_use]
    pub fn ticks(mut self, count: usize) -> Self {
        self.tick_count = count;
        self
    }

    /// Set tick label format.
    #[must_use]
    pub fn format(mut self, fmt: TickFormat) -> Self {
        self.tick_format = fmt;
        self
    }

    /// Set an axis label.
    #[must_use]
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Enable grid lines extending `extent` pixels from the axis.
    #[must_use]
    pub fn with_grid(mut self, extent: f64) -> Self {
        self.grid = true;
        self.grid_extent = extent;
        self
    }

    /// Set tick mark size.
    #[must_use]
    pub fn tick_size(mut self, size: f64) -> Self {
        self.tick_size = size;
        self
    }

    /// Render this axis into an `SvgDoc` using the given scale and position.
    ///
    /// `anchor` is the pixel coordinate where the axis sits:
    /// - Bottom/Top: the y-coordinate of the axis line (full range spans the scale's range).
    /// - Left/Right: the x-coordinate of the axis line (full range spans the scale's range).
    pub fn render(&self, doc: &mut SvgDoc, scale: &dyn Scale, anchor: f64, theme: &Theme) {
        let tick_values = scale.ticks(self.tick_count);
        let (range_start, range_end) = scale.range();

        let ctx = AxisCtx {
            scale,
            anchor,
            theme,
            tick_values: &tick_values,
            range_start,
            range_end,
        };

        match self.orientation {
            Orientation::Bottom => self.render_bottom(doc, &ctx),
            Orientation::Top => self.render_top(doc, &ctx),
            Orientation::Left => self.render_left(doc, &ctx),
            Orientation::Right => self.render_right(doc, &ctx),
        }
    }

    fn render_bottom(&self, doc: &mut SvgDoc, ctx: &AxisCtx<'_>) {
        let y = ctx.anchor;
        let theme = ctx.theme;

        // Axis line
        doc.add(svg::line(
            ctx.range_start,
            y,
            ctx.range_end,
            y,
            theme.border,
            theme.stroke_width_thin,
        ));

        // Ticks and labels
        for &val in ctx.tick_values {
            let x = ctx.scale.map(val);
            // Tick mark
            doc.add(svg::line(
                x,
                y,
                x,
                y + self.tick_size,
                theme.text_dim,
                theme.stroke_width_thin,
            ));
            // Label
            let label = self.tick_format.format(val);
            doc.add(svg::text(
                x,
                y + self.tick_size + 12.0,
                &label,
                theme.font_size(0.7),
                theme.text_dim,
                "middle",
            ));
            // Grid line
            if self.grid {
                doc.add(svg::line_dashed(
                    x,
                    y,
                    x,
                    y - self.grid_extent,
                    theme.border,
                    0.5,
                    "4,4",
                ));
            }
        }

        // Axis label
        if let Some(ref label) = self.label {
            let mid = (ctx.range_start + ctx.range_end) / 2.0;
            doc.add(svg::text(
                mid,
                y + self.tick_size + 28.0,
                label,
                theme.font_size(0.85),
                theme.text,
                "middle",
            ));
        }
    }

    fn render_top(&self, doc: &mut SvgDoc, ctx: &AxisCtx<'_>) {
        let y = ctx.anchor;
        let theme = ctx.theme;

        doc.add(svg::line(
            ctx.range_start,
            y,
            ctx.range_end,
            y,
            theme.border,
            theme.stroke_width_thin,
        ));

        for &val in ctx.tick_values {
            let x = ctx.scale.map(val);
            doc.add(svg::line(
                x,
                y,
                x,
                y - self.tick_size,
                theme.text_dim,
                theme.stroke_width_thin,
            ));
            let label = self.tick_format.format(val);
            doc.add(svg::text(
                x,
                y - self.tick_size - 4.0,
                &label,
                theme.font_size(0.7),
                theme.text_dim,
                "middle",
            ));
            if self.grid {
                doc.add(svg::line_dashed(
                    x,
                    y,
                    x,
                    y + self.grid_extent,
                    theme.border,
                    0.5,
                    "4,4",
                ));
            }
        }

        if let Some(ref label) = self.label {
            let mid = (ctx.range_start + ctx.range_end) / 2.0;
            doc.add(svg::text(
                mid,
                y - self.tick_size - 20.0,
                label,
                theme.font_size(0.85),
                theme.text,
                "middle",
            ));
        }
    }

    fn render_left(&self, doc: &mut SvgDoc, ctx: &AxisCtx<'_>) {
        let x = ctx.anchor;
        let theme = ctx.theme;

        doc.add(svg::line(
            x,
            ctx.range_start,
            x,
            ctx.range_end,
            theme.border,
            theme.stroke_width_thin,
        ));

        for &val in ctx.tick_values {
            let y = ctx.scale.map(val);
            doc.add(svg::line(
                x,
                y,
                x - self.tick_size,
                y,
                theme.text_dim,
                theme.stroke_width_thin,
            ));
            let label = self.tick_format.format(val);
            doc.add(svg::text(
                x - self.tick_size - 4.0,
                y,
                &label,
                theme.font_size(0.7),
                theme.text_dim,
                "end",
            ));
            if self.grid {
                doc.add(svg::line_dashed(
                    x,
                    y,
                    x + self.grid_extent,
                    y,
                    theme.border,
                    0.5,
                    "4,4",
                ));
            }
        }

        if let Some(ref label) = self.label {
            let mid = (ctx.range_start + ctx.range_end) / 2.0;
            // Rotated label for vertical axis
            let mut el = String::new();
            let _ = write!(
                el,
                r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" fill="{}" text-anchor="middle" dominant-baseline="middle" transform="rotate(-90,{:.1},{:.1})">{}</text>"#,
                x - self.tick_size - 30.0,
                mid,
                theme.font_size(0.85),
                theme.text,
                x - self.tick_size - 30.0,
                mid,
                svg::escape_xml(label),
            );
            doc.add(el);
        }
    }

    fn render_right(&self, doc: &mut SvgDoc, ctx: &AxisCtx<'_>) {
        let x = ctx.anchor;
        let theme = ctx.theme;

        doc.add(svg::line(
            x,
            ctx.range_start,
            x,
            ctx.range_end,
            theme.border,
            theme.stroke_width_thin,
        ));

        for &val in ctx.tick_values {
            let y = ctx.scale.map(val);
            doc.add(svg::line(
                x,
                y,
                x + self.tick_size,
                y,
                theme.text_dim,
                theme.stroke_width_thin,
            ));
            let label = self.tick_format.format(val);
            doc.add(svg::text(
                x + self.tick_size + 4.0,
                y,
                &label,
                theme.font_size(0.7),
                theme.text_dim,
                "start",
            ));
            if self.grid {
                doc.add(svg::line_dashed(
                    x,
                    y,
                    x - self.grid_extent,
                    y,
                    theme.border,
                    0.5,
                    "4,4",
                ));
            }
        }

        if let Some(ref label) = self.label {
            let mid = (ctx.range_start + ctx.range_end) / 2.0;
            let mut el = String::new();
            let _ = write!(
                el,
                r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" fill="{}" text-anchor="middle" dominant-baseline="middle" transform="rotate(90,{:.1},{:.1})">{}</text>"#,
                x + self.tick_size + 30.0,
                mid,
                theme.font_size(0.85),
                theme.text,
                x + self.tick_size + 30.0,
                mid,
                svg::escape_xml(label),
            );
            doc.add(el);
        }
    }
}

// ============================================================================
// Legend
// ============================================================================

/// An entry in a legend.
#[derive(Debug, Clone)]
pub struct LegendEntry {
    /// Color swatch.
    pub color: String,
    /// Label text.
    pub label: String,
}

impl LegendEntry {
    /// Create a new legend entry.
    #[must_use]
    pub fn new(color: &str, label: &str) -> Self {
        Self {
            color: color.to_string(),
            label: label.to_string(),
        }
    }
}

/// Render a horizontal legend bar at a given position.
///
/// Returns the total width consumed.
#[must_use]
pub fn render_legend(
    doc: &mut SvgDoc,
    entries: &[LegendEntry],
    x: f64,
    y: f64,
    theme: &Theme,
) -> f64 {
    let tm = metrics::TextMetrics::default();
    let swatch_size = 10.0;
    let gap = 6.0;
    let entry_gap = 16.0;
    let font_size = theme.font_size(0.75);

    let mut cx = x;
    for entry in entries {
        // Color swatch
        doc.add(svg::rect(
            cx,
            y - swatch_size / 2.0,
            swatch_size,
            swatch_size,
            &entry.color,
            2.0,
        ));
        cx += swatch_size + gap;
        // Label
        doc.add(svg::text(
            cx,
            y,
            &entry.label,
            font_size,
            theme.text_dim,
            "start",
        ));
        let ext = tm.measure(&entry.label, font_size);
        cx += ext.width + entry_gap;
    }
    cx - x
}

/// Render a vertical legend at a given position.
///
/// Returns the total height consumed.
#[must_use]
pub fn render_legend_vertical(
    doc: &mut SvgDoc,
    entries: &[LegendEntry],
    x: f64,
    y: f64,
    theme: &Theme,
) -> f64 {
    let swatch_size = 10.0;
    let gap = 6.0;
    let line_height = 18.0;
    let font_size = theme.font_size(0.75);

    for (i, entry) in entries.iter().enumerate() {
        let cy = y + i as f64 * line_height;
        doc.add(svg::rect(
            x,
            cy - swatch_size / 2.0,
            swatch_size,
            swatch_size,
            &entry.color,
            2.0,
        ));
        doc.add(svg::text(
            x + swatch_size + gap,
            cy,
            &entry.label,
            font_size,
            theme.text_dim,
            "start",
        ));
    }
    entries.len() as f64 * line_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::LinearScale;
    use crate::svg::palette;

    fn test_scale() -> LinearScale {
        LinearScale::new(0.0, 100.0, 50.0, 450.0)
    }

    #[test]
    fn render_bottom_axis() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let scale = test_scale();
        let theme = Theme::dark();
        let axis = Axis::bottom().ticks(5);
        axis.render(&mut doc, &scale, 250.0, &theme);
        let svg = doc.render();
        assert!(svg.contains("<line")); // axis line + ticks
        assert!(svg.contains("<text")); // tick labels
    }

    #[test]
    fn render_left_axis() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let scale = test_scale();
        let theme = Theme::dark();
        let axis = Axis::left().ticks(5).with_label("Value");
        axis.render(&mut doc, &scale, 50.0, &theme);
        let svg = doc.render();
        assert!(svg.contains("Value"));
        assert!(svg.contains("rotate(-90"));
    }

    #[test]
    fn render_with_grid() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let scale = test_scale();
        let theme = Theme::dark();
        let axis = Axis::bottom().ticks(5).with_grid(200.0);
        axis.render(&mut doc, &scale, 250.0, &theme);
        let svg = doc.render();
        assert!(svg.contains("stroke-dasharray")); // grid lines are dashed
    }

    #[test]
    fn render_percent_format() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let scale = LinearScale::new(0.0, 1.0, 50.0, 450.0);
        let theme = Theme::dark();
        let axis = Axis::bottom().ticks(5).format(TickFormat::Percent);
        axis.render(&mut doc, &scale, 250.0, &theme);
        let svg = doc.render();
        assert!(svg.contains("%"));
    }

    #[test]
    fn render_top_axis() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let scale = test_scale();
        let theme = Theme::light();
        let axis = Axis::top().ticks(3);
        axis.render(&mut doc, &scale, 50.0, &theme);
        let svg = doc.render();
        assert!(svg.contains("<line"));
    }

    #[test]
    fn render_right_axis() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let scale = test_scale();
        let theme = Theme::dark();
        let axis = Axis::right().ticks(4).with_label("Score");
        axis.render(&mut doc, &scale, 450.0, &theme);
        let svg = doc.render();
        assert!(svg.contains("Score"));
        assert!(svg.contains("rotate(90"));
    }

    #[test]
    fn orientation_is_horizontal() {
        assert!(Orientation::Bottom.is_horizontal());
        assert!(Orientation::Top.is_horizontal());
        assert!(!Orientation::Left.is_horizontal());
        assert!(!Orientation::Right.is_horizontal());
    }

    // -- Legend --

    #[test]
    fn render_horizontal_legend() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let theme = Theme::dark();
        let entries = vec![
            LegendEntry::new(palette::EMERALD, "Pass"),
            LegendEntry::new(palette::RED, "Fail"),
        ];
        let width = render_legend(&mut doc, &entries, 10.0, 20.0, &theme);
        assert!(width > 0.0);
        let svg = doc.render();
        assert!(svg.contains("Pass"));
        assert!(svg.contains("Fail"));
    }

    #[test]
    fn render_vertical_legend_entries() {
        let mut doc = SvgDoc::new(500.0, 300.0);
        let theme = Theme::dark();
        let entries = vec![
            LegendEntry::new(palette::SCIENCE, "Science"),
            LegendEntry::new(palette::CHEMISTRY, "Chemistry"),
            LegendEntry::new(palette::PHYSICS, "Physics"),
        ];
        let height = render_legend_vertical(&mut doc, &entries, 10.0, 20.0, &theme);
        assert!((height - 54.0).abs() < 0.01); // 3 entries * 18px
        let svg = doc.render();
        assert!(svg.contains("Science"));
        assert!(svg.contains("Physics"));
    }

    #[test]
    fn tick_format_auto() {
        let fmt = TickFormat::Auto;
        assert_eq!(fmt.format(50.0), "50");
    }

    #[test]
    fn tick_format_fixed() {
        let fmt = TickFormat::Fixed(2);
        assert_eq!(fmt.format(3.1), "3.10");
    }

    #[test]
    fn tick_format_percent() {
        let fmt = TickFormat::Percent;
        assert_eq!(fmt.format(0.75), "75%");
    }
}
