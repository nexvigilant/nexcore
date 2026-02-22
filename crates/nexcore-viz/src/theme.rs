//! Theming System
//!
//! Extracts visual configuration into a `Theme` struct, enabling dark,
//! light, and high-contrast variants from a single diagram codebase.
//!
//! The existing `palette` module in `svg.rs` provides domain-specific
//! semantic colors (SCIENCE, MAPPING, etc.). `Theme` provides structural
//! colors (background, text, borders) and typography settings.
//!
//! Grounded: ∂ (Boundary) config boundary, μ (Mapping) theme→visual,
//!           κ (Comparison) theme variants.

use crate::svg::palette;

/// Complete visual theme for SVG rendering.
///
/// Controls background, text, borders, typography, and spacing.
/// Domain-specific colors (SCIENCE, MAPPING, etc.) are NOT in the theme —
/// they live in `palette` because they encode semantic meaning, not visual
/// preference.
#[derive(Debug, Clone)]
pub struct Theme {
    /// SVG background color.
    pub background: &'static str,
    /// Card/panel background.
    pub card_bg: &'static str,
    /// Primary text color.
    pub text: &'static str,
    /// Dimmed/secondary text color.
    pub text_dim: &'static str,
    /// Border/grid line color.
    pub border: &'static str,
    /// Danger/error color.
    pub danger: &'static str,
    /// Success/safe color.
    pub success: &'static str,
    /// Warning color.
    pub warning: &'static str,
    /// Font family CSS value.
    pub font_family: &'static str,
    /// Base font size (px).
    pub base_font_size: f64,
    /// Line height multiplier.
    pub line_height: f64,
    /// Default border radius (px).
    pub border_radius: f64,
    /// Base spacing unit (px). Used as `spacing * N` for margins/padding.
    pub spacing: f64,
    /// Stroke width for primary strokes.
    pub stroke_width: f64,
    /// Stroke width for thin/subtle strokes.
    pub stroke_width_thin: f64,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme — the current nexcore-viz default.
    /// Matches the existing palette constants.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: palette::BG,
            card_bg: palette::BG_CARD,
            text: palette::TEXT,
            text_dim: palette::TEXT_DIM,
            border: palette::BORDER,
            danger: palette::RED,
            success: palette::EMERALD,
            warning: palette::AMBER,
            font_family: "'Segoe UI',system-ui,sans-serif",
            base_font_size: 14.0,
            line_height: 1.2,
            border_radius: 4.0,
            spacing: 8.0,
            stroke_width: 2.0,
            stroke_width_thin: 1.0,
        }
    }

    /// Light theme — for print, PDF export, and light-mode UIs.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: "#ffffff",
            card_bg: "#f6f8fa",
            text: "#1f2328",
            text_dim: "#656d76",
            border: "#d0d7de",
            danger: "#cf222e",
            success: "#1a7f37",
            warning: "#9a6700",
            font_family: "'Segoe UI',system-ui,sans-serif",
            base_font_size: 14.0,
            line_height: 1.2,
            border_radius: 4.0,
            spacing: 8.0,
            stroke_width: 2.0,
            stroke_width_thin: 1.0,
        }
    }

    /// High-contrast theme — WCAG AAA compliance.
    /// All text:background pairs exceed 7:1 contrast ratio.
    #[must_use]
    pub const fn high_contrast() -> Self {
        Self {
            background: "#000000",
            card_bg: "#0a0a0a",
            text: "#ffffff",
            text_dim: "#d0d0d0",
            border: "#808080",
            danger: "#ff6666",
            success: "#66ff66",
            warning: "#ffcc00",
            font_family: "'Segoe UI',system-ui,sans-serif",
            base_font_size: 16.0,
            line_height: 1.3,
            border_radius: 4.0,
            spacing: 10.0,
            stroke_width: 2.5,
            stroke_width_thin: 1.5,
        }
    }

    /// Print theme — optimized for white paper, no background fills.
    #[must_use]
    pub const fn print() -> Self {
        Self {
            background: "none",
            card_bg: "none",
            text: "#000000",
            text_dim: "#555555",
            border: "#cccccc",
            danger: "#cc0000",
            success: "#006600",
            warning: "#996600",
            font_family: "'Times New Roman',Georgia,serif",
            base_font_size: 12.0,
            line_height: 1.25,
            border_radius: 0.0,
            spacing: 6.0,
            stroke_width: 1.5,
            stroke_width_thin: 0.75,
        }
    }

    /// Generate the CSS `style` attribute for the SVG root element.
    #[must_use]
    pub fn svg_style(&self) -> String {
        if self.background == "none" {
            format!("font-family:{}", self.font_family)
        } else {
            format!(
                "font-family:{};background:{}",
                self.font_family, self.background
            )
        }
    }

    /// Scale a font size relative to the theme's base.
    ///
    /// `factor` of 1.0 returns `base_font_size`.
    /// Useful for consistent type scale: title=1.4, subtitle=1.1, caption=0.8.
    #[must_use]
    pub fn font_size(&self, factor: f64) -> f64 {
        self.base_font_size * factor
    }

    /// Compute spacing: `self.spacing * multiplier`.
    #[must_use]
    pub fn space(&self, multiplier: f64) -> f64 {
        self.spacing * multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_theme_matches_palette() {
        let t = Theme::dark();
        assert_eq!(t.background, palette::BG);
        assert_eq!(t.text, palette::TEXT);
        assert_eq!(t.card_bg, palette::BG_CARD);
        assert_eq!(t.border, palette::BORDER);
    }

    #[test]
    fn light_theme_has_white_bg() {
        let t = Theme::light();
        assert_eq!(t.background, "#ffffff");
        assert_eq!(t.text, "#1f2328");
    }

    #[test]
    fn high_contrast_has_black_bg() {
        let t = Theme::high_contrast();
        assert_eq!(t.background, "#000000");
        assert_eq!(t.text, "#ffffff");
    }

    #[test]
    fn print_theme_transparent_bg() {
        let t = Theme::print();
        assert_eq!(t.background, "none");
        assert_eq!(t.card_bg, "none");
    }

    #[test]
    fn svg_style_dark() {
        let t = Theme::dark();
        let style = t.svg_style();
        assert!(style.contains("font-family:"));
        assert!(style.contains(palette::BG));
    }

    #[test]
    fn svg_style_print_no_bg() {
        let t = Theme::print();
        let style = t.svg_style();
        assert!(!style.contains("background"));
    }

    #[test]
    fn font_size_scaling() {
        let t = Theme::dark();
        assert!((t.font_size(1.0) - 14.0).abs() < 0.01);
        assert!((t.font_size(2.0) - 28.0).abs() < 0.01);
    }

    #[test]
    fn spacing_scaling() {
        let t = Theme::dark();
        assert!((t.space(1.0) - 8.0).abs() < 0.01);
        assert!((t.space(2.0) - 16.0).abs() < 0.01);
    }

    #[test]
    fn default_is_dark() {
        let default = Theme::default();
        let dark = Theme::dark();
        assert_eq!(default.background, dark.background);
        assert_eq!(default.text, dark.text);
    }
}
