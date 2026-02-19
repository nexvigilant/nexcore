//! Framebuffer: pixel buffer for CPU rendering
//!
//! Stores pixels as ARGB u32. This is the final output — what gets blitted to screen.

use crate::math::Color;

#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    /// ARGB packed pixels, row-major: pixel[y * width + x]
    pub pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0xFF000000; (width * height) as usize], // opaque black
        }
    }

    /// Total pixel count
    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    /// Clear to a solid color
    pub fn clear(&mut self, color: Color) {
        let packed = color.to_argb_u32();
        self.pixels.fill(packed);
    }

    /// Set pixel at (x, y). Out of bounds = no-op.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx] = color.to_argb_u32();
        }
    }

    /// Set pixel with alpha blending over existing pixel
    pub fn blend_pixel(&mut self, x: u32, y: u32, src: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            let dst = Color::from_argb_u32(self.pixels[idx]);
            self.pixels[idx] = src.alpha_over(dst).to_argb_u32();
        }
    }

    /// Get pixel color at (x, y). Out of bounds = transparent black.
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            Color::from_argb_u32(self.pixels[idx])
        } else {
            Color::TRANSPARENT
        }
    }

    /// Raw pixel data as RGBA bytes (for PNG export or display)
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);
        for &pixel in &self.pixels {
            bytes.push(((pixel >> 16) & 0xFF) as u8); // R
            bytes.push(((pixel >> 8) & 0xFF) as u8); // G
            bytes.push((pixel & 0xFF) as u8); // B
            bytes.push(((pixel >> 24) & 0xFF) as u8); // A
        }
        bytes
    }

    /// Raw pixel data as BGRA bytes (for some windowing systems)
    pub fn to_bgra_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);
        for &pixel in &self.pixels {
            bytes.push((pixel & 0xFF) as u8); // B
            bytes.push(((pixel >> 8) & 0xFF) as u8); // G
            bytes.push(((pixel >> 16) & 0xFF) as u8); // R
            bytes.push(((pixel >> 24) & 0xFF) as u8); // A
        }
        bytes
    }

    /// Blit another framebuffer onto this one at (dx, dy)
    pub fn blit(&mut self, src: &Framebuffer, dx: i32, dy: i32) {
        for sy in 0..src.height {
            let ty = dy + sy as i32;
            if ty < 0 || ty >= self.height as i32 {
                continue;
            }
            for sx in 0..src.width {
                let tx = dx + sx as i32;
                if tx < 0 || tx >= self.width as i32 {
                    continue;
                }
                let src_color = src.get_pixel(sx, sy);
                self.blend_pixel(tx as u32, ty as u32, src_color);
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_framebuffer_is_black() {
        let fb = Framebuffer::new(100, 100);
        assert_eq!(fb.len(), 10000);
        let c = fb.get_pixel(50, 50);
        assert!((c.r - 0.0).abs() < 0.01);
        assert!((c.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn set_and_get_pixel() {
        let mut fb = Framebuffer::new(10, 10);
        fb.set_pixel(5, 5, Color::RED);
        let c = fb.get_pixel(5, 5);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
    }

    #[test]
    fn out_of_bounds_noop() {
        let mut fb = Framebuffer::new(10, 10);
        fb.set_pixel(100, 100, Color::RED); // no panic
        let c = fb.get_pixel(100, 100);
        assert!((c.a - 0.0).abs() < 0.01); // transparent
    }

    #[test]
    fn clear_to_color() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::BLUE);
        let c = fb.get_pixel(0, 0);
        assert!((c.b - 1.0).abs() < 0.01);
    }

    #[test]
    fn rgba_bytes_length() {
        let fb = Framebuffer::new(8, 8);
        assert_eq!(fb.to_rgba_bytes().len(), 8 * 8 * 4);
    }

    #[test]
    fn blit_small_onto_large() {
        let mut dst = Framebuffer::new(20, 20);
        dst.clear(Color::BLACK);

        let mut src = Framebuffer::new(5, 5);
        src.clear(Color::RED);

        dst.blit(&src, 10, 10);

        // Inside blit region
        let c = dst.get_pixel(12, 12);
        assert!((c.r - 1.0).abs() < 0.01);

        // Outside blit region
        let c2 = dst.get_pixel(0, 0);
        assert!((c2.r - 0.0).abs() < 0.01);
    }
}
