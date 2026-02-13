//! Color: RGBA f64 with blending operations
//!
//! All channels in [0.0, 1.0]. Pack to u32 ARGB for framebuffer output.

// ============================================================================
// Color
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    // NexVigilant brand palette
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const RED: Self = Self::rgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::rgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::rgba(0.0, 0.0, 1.0, 1.0);

    // Brand colors (NexVigilant)
    pub const NAVY: Self = Self::rgba(0.059, 0.094, 0.169, 1.0); // #0F1829
    pub const ACCENT_CYAN: Self = Self::rgba(0.0, 0.8, 1.0, 1.0);
    pub const ACCENT_GREEN: Self = Self::rgba(0.0, 0.8, 0.4, 1.0);
    pub const ACCENT_GOLD: Self = Self::rgba(1.0, 0.8, 0.0, 1.0);
    pub const ACCENT_RED: Self = Self::rgba(1.0, 0.267, 0.267, 1.0);

    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// From hex: 0xRRGGBB
    pub fn from_hex(hex: u32) -> Self {
        Self::rgb(
            ((hex >> 16) & 0xFF) as f64 / 255.0,
            ((hex >> 8) & 0xFF) as f64 / 255.0,
            (hex & 0xFF) as f64 / 255.0,
        )
    }

    /// From hex with alpha: 0xAARRGGBB
    pub fn from_hex_argb(hex: u32) -> Self {
        Self::rgba(
            ((hex >> 16) & 0xFF) as f64 / 255.0,
            ((hex >> 8) & 0xFF) as f64 / 255.0,
            (hex & 0xFF) as f64 / 255.0,
            ((hex >> 24) & 0xFF) as f64 / 255.0,
        )
    }

    /// Pack to ARGB u32 for framebuffer
    pub fn to_argb_u32(self) -> u32 {
        let r = (self.r.clamp(0.0, 1.0) * 255.0) as u32;
        let g = (self.g.clamp(0.0, 1.0) * 255.0) as u32;
        let b = (self.b.clamp(0.0, 1.0) * 255.0) as u32;
        let a = (self.a.clamp(0.0, 1.0) * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }

    /// Unpack from ARGB u32
    pub fn from_argb_u32(v: u32) -> Self {
        Self::rgba(
            ((v >> 16) & 0xFF) as f64 / 255.0,
            ((v >> 8) & 0xFF) as f64 / 255.0,
            (v & 0xFF) as f64 / 255.0,
            ((v >> 24) & 0xFF) as f64 / 255.0,
        )
    }

    /// Alpha-over compositing: self over dst
    pub fn alpha_over(self, dst: Color) -> Color {
        let sa = self.a;
        let da = dst.a * (1.0 - sa);
        let out_a = sa + da;
        if out_a < f64::EPSILON {
            return Color::TRANSPARENT;
        }
        Color::rgba(
            (self.r * sa + dst.r * da) / out_a,
            (self.g * sa + dst.g * da) / out_a,
            (self.b * sa + dst.b * da) / out_a,
            out_a,
        )
    }

    /// Multiply blend (darken)
    pub fn multiply(self, other: Color) -> Color {
        Color::rgba(
            self.r * other.r,
            self.g * other.g,
            self.b * other.b,
            self.a * other.a,
        )
    }

    /// Screen blend (lighten)
    pub fn screen(self, other: Color) -> Color {
        Color::rgba(
            1.0 - (1.0 - self.r) * (1.0 - other.r),
            1.0 - (1.0 - self.g) * (1.0 - other.g),
            1.0 - (1.0 - self.b) * (1.0 - other.b),
            1.0 - (1.0 - self.a) * (1.0 - other.a),
        )
    }

    /// Linear interpolation
    pub fn lerp(self, other: Color, t: f64) -> Color {
        Color::rgba(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    /// Clamp all channels to [0, 1]
    pub fn clamp(self) -> Color {
        Color::rgba(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
            self.a.clamp(0.0, 1.0),
        )
    }

    /// Premultiply alpha (for correct blending)
    pub fn premultiply(self) -> Color {
        Color::rgba(self.r * self.a, self.g * self.a, self.b * self.a, self.a)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_argb_u32() {
        let c = Color::rgb(1.0, 0.5, 0.0);
        let packed = c.to_argb_u32();
        let unpacked = Color::from_argb_u32(packed);
        assert!((unpacked.r - 1.0).abs() < 0.01);
        assert!((unpacked.g - 0.5).abs() < 0.01);
        assert!((unpacked.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn from_hex() {
        let c = Color::from_hex(0xFF8000); // orange
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.502).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn alpha_over_opaque() {
        let src = Color::RED;
        let dst = Color::BLUE;
        let result = src.alpha_over(dst);
        assert!((result.r - 1.0).abs() < 0.01);
        assert!((result.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn alpha_over_half_transparent() {
        let src = Color::rgba(1.0, 0.0, 0.0, 0.5);
        let dst = Color::rgba(0.0, 0.0, 1.0, 1.0);
        let result = src.alpha_over(dst);
        assert!(result.r > 0.3);
        assert!(result.b > 0.3);
    }

    #[test]
    fn lerp_halfway() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn brand_navy_is_dark() {
        let n = Color::NAVY;
        assert!(n.r < 0.1);
        assert!(n.g < 0.2);
        assert!(n.b < 0.2);
    }
}
