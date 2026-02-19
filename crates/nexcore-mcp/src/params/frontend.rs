//! Frontend & Accessibility Parameters
//! Tier: T2-C (cross-domain — design + perception + accessibility)
//!
//! WCAG contrast, touch targets, type scale, spacing scale, and a11y audit.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for WCAG contrast ratio calculation.
/// Accepts colors as `[r, g, b]` (0-255) or `[r, g, b, a]` (a: 0.0-1.0).
/// When foreground has alpha < 1.0, it is blended onto the background first.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WcagContrastParams {
    /// Foreground color as [r, g, b] or [r, g, b, a]. Values 0-255 for RGB, 0.0-1.0 for alpha.
    pub foreground: Vec<f64>,
    /// Background color as [r, g, b]. Must be opaque (no alpha). Values 0-255.
    pub background: Vec<f64>,
    /// Font size in pixels (used to determine if "large text" thresholds apply). Default: 16.
    #[serde(default = "default_font_size")]
    pub font_size_px: f64,
    /// Font weight (100-900). Bold (>=700) lowers "large text" threshold to 18.66px. Default: 400.
    #[serde(default = "default_font_weight")]
    pub font_weight: u16,
}

fn default_font_size() -> f64 {
    16.0
}
fn default_font_weight() -> u16 {
    400
}

/// Parameters for blending an RGBA foreground color onto an opaque background.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ColorBlendParams {
    /// Foreground color as [r, g, b, a]. RGB: 0-255, alpha: 0.0-1.0.
    pub foreground: Vec<f64>,
    /// Background color as [r, g, b]. Must be opaque. Values 0-255.
    pub background: Vec<f64>,
}

/// Parameters for touch target compliance check (WCAG 2.5.5 / 2.5.8).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TouchTargetParams {
    /// Element width in CSS pixels.
    pub width: f64,
    /// Element height in CSS pixels.
    pub height: f64,
    /// Compliance level: "aa" (44x44) or "aaa" (48x48). Default: "aa".
    #[serde(default = "default_level")]
    pub level: String,
}

fn default_level() -> String {
    "aa".to_string()
}

/// Parameters for type scale audit against a modular scale ratio.
/// Checks a set of font sizes for consistent ratio progression and identifies gaps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TypeScaleAuditParams {
    /// Array of font sizes in use (in px). Will be sorted automatically.
    pub sizes: Vec<f64>,
    /// Target modular scale ratio. Default: 1.618 (golden ratio).
    #[serde(default = "default_ratio")]
    pub target_ratio: f64,
    /// Maximum acceptable deviation from target ratio (0.0-1.0). Default: 0.15 (15%).
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

fn default_ratio() -> f64 {
    1.618
}
fn default_tolerance() -> f64 {
    0.15
}

/// Parameters for spacing scale audit against a modular scale.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SpacingAuditParams {
    /// Array of spacing values in use (in px). Will be sorted automatically.
    pub values: Vec<f64>,
    /// Base value of the spacing scale (in px). Default: 8.0.
    #[serde(default = "default_base")]
    pub base: f64,
    /// Scale ratio. Default: 1.618 (golden ratio).
    #[serde(default = "default_ratio")]
    pub ratio: f64,
}

fn default_base() -> f64 {
    8.0
}

/// Combined accessibility audit. Checks multiple contrast pairs, touch targets, and heading hierarchy in one call.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct A11yAuditParams {
    /// Array of color pairs to check. Each: { "name": "label", "fg": [r,g,b] or [r,g,b,a], "bg": [r,g,b], "font_size_px": 16, "font_weight": 400 }
    #[serde(default)]
    pub contrast_pairs: Vec<ContrastPair>,
    /// Array of interactive elements to check. Each: { "name": "label", "width": px, "height": px }
    #[serde(default)]
    pub touch_targets: Vec<TouchTarget>,
    /// Heading levels present on page, in document order (e.g., [1, 2, 3, 3, 2, 3]).
    #[serde(default)]
    pub heading_levels: Vec<u8>,
}

/// A single contrast pair for batch checking.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ContrastPair {
    /// Label for this pair (e.g., "nav-link on header").
    pub name: String,
    /// Foreground color [r,g,b] or [r,g,b,a].
    pub fg: Vec<f64>,
    /// Background color [r,g,b].
    pub bg: Vec<f64>,
    /// Font size in px. Default: 16.
    #[serde(default = "default_font_size")]
    pub font_size_px: f64,
    /// Font weight. Default: 400.
    #[serde(default = "default_font_weight")]
    pub font_weight: u16,
}

/// A single touch target element for batch checking.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TouchTarget {
    /// Label for this element.
    pub name: String,
    /// Width in CSS pixels.
    pub width: f64,
    /// Height in CSS pixels.
    pub height: f64,
}
