//! Text Measurement Approximation
//!
//! Estimates text dimensions for system-ui/sans-serif fonts without
//! requiring a font parser or external dependency. Uses per-character
//! width lookup tables calibrated against common web fonts.
//!
//! Accuracy: ±10% of true rendered width — sufficient for layout
//! decisions (collision avoidance, auto-sizing) but not pixel-perfect
//! typesetting.
//!
//! Grounded: μ (Mapping) char→width, N (Quantity) pixel measurement.

/// Measured dimensions of a text string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextExtent {
    /// Approximate rendered width in SVG user units (pixels at 1:1).
    pub width: f64,
    /// Approximate rendered height (ascent + descent).
    pub height: f64,
}

impl TextExtent {
    /// Create a new text extent.
    #[must_use]
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Zero-size extent.
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };
}

/// Configurable text metrics engine.
///
/// Holds a character width table calibrated for a specific font family.
/// The default is calibrated for `system-ui, -apple-system, sans-serif`
/// which is what nexcore-viz SVGs use.
#[derive(Debug, Clone)]
pub struct TextMetrics {
    /// Reference font size the width table was measured at.
    reference_size: f64,
    /// Default line-height multiplier (height = size * line_height).
    line_height: f64,
    /// Bold width multiplier (bold text is wider).
    bold_factor: f64,
}

impl Default for TextMetrics {
    fn default() -> Self {
        Self {
            reference_size: 16.0,
            line_height: 1.2,
            bold_factor: 1.06,
        }
    }
}

impl TextMetrics {
    /// Create with custom parameters.
    #[must_use]
    pub const fn new(reference_size: f64, line_height: f64, bold_factor: f64) -> Self {
        Self {
            reference_size,
            line_height,
            bold_factor,
        }
    }

    /// Measure text at a given font size.
    #[must_use]
    pub fn measure(&self, text: &str, font_size: f64) -> TextExtent {
        let scale = font_size / self.reference_size;
        let width: f64 = text.chars().map(|c| char_width(c) * scale).sum();
        let height = font_size * self.line_height;
        TextExtent::new(width, height)
    }

    /// Measure bold text at a given font size.
    #[must_use]
    pub fn measure_bold(&self, text: &str, font_size: f64) -> TextExtent {
        let mut ext = self.measure(text, font_size);
        ext.width *= self.bold_factor;
        ext
    }

    /// Measure and return the extent plus a padding margin.
    #[must_use]
    pub fn measure_padded(&self, text: &str, font_size: f64, pad_x: f64, pad_y: f64) -> TextExtent {
        let ext = self.measure(text, font_size);
        TextExtent::new(ext.width + 2.0 * pad_x, ext.height + 2.0 * pad_y)
    }

    /// Check if two text labels would overlap at given positions.
    #[must_use]
    pub fn overlaps(
        &self,
        text_a: &str,
        x_a: f64,
        size_a: f64,
        text_b: &str,
        x_b: f64,
        size_b: f64,
    ) -> bool {
        let ext_a = self.measure(text_a, size_a);
        let ext_b = self.measure(text_b, size_b);
        // Assume center-anchored text
        let left_a = x_a - ext_a.width / 2.0;
        let right_a = x_a + ext_a.width / 2.0;
        let left_b = x_b - ext_b.width / 2.0;
        let right_b = x_b + ext_b.width / 2.0;
        left_a < right_b && left_b < right_a
    }
}

// ============================================================================
// Module-level convenience functions (use default TextMetrics)
// ============================================================================

/// Measure text with default metrics.
#[must_use]
pub fn measure_text(text: &str, font_size: f64) -> TextExtent {
    TextMetrics::default().measure(text, font_size)
}

/// Measure bold text with default metrics.
#[must_use]
pub fn measure_text_bold(text: &str, font_size: f64) -> TextExtent {
    TextMetrics::default().measure_bold(text, font_size)
}

// ============================================================================
// Character width table
// ============================================================================

/// Approximate width of a single character at the reference font size (16px).
///
/// Calibrated against `system-ui` / `-apple-system` / `Segoe UI` metrics.
/// Characters are grouped by width class to keep the table manageable.
///
/// Width classes (at 16px reference):
/// - Narrow (≈4.5px): i, l, !, |, .,:, ;, '
/// - Slim (≈6.0px): f, j, t, r, 1, (, ), [, ]
/// - Regular (≈8.0px): most lowercase, digits 0-9
/// - Wide (≈9.5px): most uppercase, m, w
/// - Extra-wide (≈11px): M, W
/// - Space (≈4.0px): whitespace
#[must_use]
fn char_width(c: char) -> f64 {
    match c {
        // Narrow
        'i' | 'l' | '!' | '|' | '.' | ',' | ':' | ';' | '\'' | '`' => 4.5,
        // Slim
        'f' | 'j' | 't' | 'r' | '(' | ')' | '[' | ']' | '{' | '}' | '/' | '\\' | '"' => 6.0,
        '1' => 6.0,
        // Regular lowercase
        'a' | 'b' | 'c' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q' | 's' | 'u' | 'v'
        | 'x' | 'y' | 'z' => 8.0,
        // Slightly wider lowercase
        'w' => 10.5,
        'm' => 11.5,
        // Regular digits (0, 2-9 are tabular-width in most fonts)
        '0' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => 8.0,
        // Regular uppercase
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'L' | 'N' | 'O' | 'P' | 'Q' | 'R'
        | 'S' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 9.5,
        // Narrow uppercase
        'I' => 5.0,
        'J' => 6.5,
        // Wide uppercase
        'M' => 11.5,
        'W' => 12.5,
        // Punctuation and symbols
        ' ' => 4.0,
        '-' | '–' => 5.5,
        '—' => 10.0,
        '_' => 8.0,
        '+' | '=' | '<' | '>' | '~' | '^' => 8.0,
        '#' | '$' | '%' | '&' | '@' => 9.5,
        '*' => 6.0,
        '?' => 7.0,
        // Greek letters (common in scientific viz)
        '\u{03bc}' => 9.0,              // μ (mu)
        '\u{03c3}' => 8.5,              // σ (sigma)
        '\u{03c1}' => 8.5,              // ρ (rho)
        '\u{03c2}' => 7.5,              // ς (varsigma)
        '\u{2192}' => 10.0,             // → (right arrow)
        '\u{2202}' => 9.0,              // ∂ (partial)
        '\u{2264}' | '\u{2265}' => 8.5, // ≤ ≥
        '\u{00f6}' => 8.0,              // ö
        '\u{2713}' | '\u{2717}' => 9.0, // ✓ ✗
        '\u{26a0}' => 10.0,             // ⚠
        // Fallback for any other character
        _ => 8.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_empty_string() {
        let ext = measure_text("", 14.0);
        assert!((ext.width - 0.0).abs() < f64::EPSILON);
        assert!(ext.height > 0.0);
    }

    #[test]
    fn measure_hello_world() {
        let ext = measure_text("Hello World", 16.0);
        // "Hello World" at 16px should be roughly 75-85px wide
        assert!(ext.width > 60.0, "width {} too narrow", ext.width);
        assert!(ext.width < 100.0, "width {} too wide", ext.width);
        assert!((ext.height - 19.2).abs() < 0.1); // 16 * 1.2
    }

    #[test]
    fn bold_is_wider() {
        let regular = measure_text("Machine<I,O>", 14.0);
        let bold = measure_text_bold("Machine<I,O>", 14.0);
        assert!(bold.width > regular.width);
    }

    #[test]
    fn scales_with_font_size() {
        let small = measure_text("test", 10.0);
        let large = measure_text("test", 20.0);
        // 20px should be exactly 2x the width of 10px
        assert!((large.width / small.width - 2.0).abs() < 0.01);
    }

    #[test]
    fn narrow_chars_are_narrower() {
        let narrow = measure_text("iiii", 16.0);
        let wide = measure_text("MMMM", 16.0);
        assert!(
            wide.width > narrow.width * 2.0,
            "MMMM ({}) should be >2x iiii ({})",
            wide.width,
            narrow.width
        );
    }

    #[test]
    fn overlap_detection() {
        let m = TextMetrics::default();
        // Two labels 10px apart, each ~80px wide — should overlap
        assert!(m.overlaps("Hello World", 100.0, 16.0, "Hello World", 110.0, 16.0));
        // Two labels 200px apart — should not overlap
        assert!(!m.overlaps("Hi", 100.0, 16.0, "Hi", 300.0, 16.0));
    }

    #[test]
    fn greek_symbols_measured() {
        let ext = measure_text("\u{03bc}", 16.0);
        assert!(ext.width > 0.0);
    }

    #[test]
    fn padded_measurement() {
        let m = TextMetrics::default();
        let plain = m.measure("test", 16.0);
        let padded = m.measure_padded("test", 16.0, 8.0, 4.0);
        assert!((padded.width - plain.width - 16.0).abs() < 0.01);
        assert!((padded.height - plain.height - 8.0).abs() < 0.01);
    }
}
