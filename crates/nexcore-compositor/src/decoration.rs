// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Window decorations — title bar, borders, and control buttons.
//!
//! ## Primitive Grounding
//!
//! - ∂ Boundary: Window chrome defines visual edges
//! - λ Location: Button/title positioning within chrome
//! - μ Mapping: Surface state → decoration appearance
//! - κ Comparison: Active vs inactive window styling

use crate::surface::Rect;

/// Theme colors for window decorations.
///
/// Tier: T2-C (μ + κ — maps state to visual appearance)
#[derive(Debug, Clone, Copy)]
pub struct DecorationTheme {
    /// Active window title bar color (RGBA).
    pub active_title: [u8; 4],
    /// Inactive window title bar color (RGBA).
    pub inactive_title: [u8; 4],
    /// Window border color (RGBA).
    pub border: [u8; 4],
    /// Close button color (RGBA).
    pub close_button: [u8; 4],
    /// Close button hover color (RGBA).
    pub close_hover: [u8; 4],
    /// Maximize button color (RGBA).
    pub maximize_button: [u8; 4],
    /// Minimize button color (RGBA).
    pub minimize_button: [u8; 4],
    /// Title text color (RGBA).
    pub title_text: [u8; 4],
    /// Title bar height in pixels.
    pub title_bar_height: u32,
    /// Border width in pixels.
    pub border_width: u32,
    /// Button width in title bar.
    pub button_width: u32,
    /// Button height in title bar.
    pub button_height: u32,
}

impl Default for DecorationTheme {
    fn default() -> Self {
        Self {
            active_title: [40, 40, 60, 255],      // Dark slate
            inactive_title: [50, 50, 55, 255],    // Dimmer slate
            border: [60, 60, 80, 255],            // Subtle border
            close_button: [200, 60, 60, 255],     // Red
            close_hover: [240, 80, 80, 255],      // Bright red
            maximize_button: [60, 180, 60, 255],  // Green
            minimize_button: [200, 180, 60, 255], // Yellow
            title_text: [220, 220, 230, 255],     // Light gray
            title_bar_height: 30,
            border_width: 1,
            button_width: 32,
            button_height: 24,
        }
    }
}

/// Rendered decoration frame for a single window.
///
/// Tier: T2-C (∂ + λ — positioned boundary elements)
///
/// Contains the RGBA pixel data for the window chrome
/// (title bar, borders, buttons). Rendered separately from
/// the surface content and composited on top.
#[derive(Debug, Clone)]
pub struct DecorationFrame {
    /// Full bounds including decorations.
    pub outer_bounds: Rect,
    /// Title bar region.
    pub title_bar: Rect,
    /// Close button region.
    pub close_button: Rect,
    /// Maximize button region.
    pub maximize_button: Rect,
    /// Minimize button region.
    pub minimize_button: Rect,
    /// Framebuffer for the decoration (RGBA, row-major).
    /// Covers the outer_bounds area.
    pub framebuffer: Vec<u8>,
}

/// Decoration renderer — draws window chrome.
///
/// Tier: T3 (∂ + λ + μ + κ — full decoration pipeline)
#[derive(Debug, Clone)]
pub struct DecorationRenderer {
    /// Theme for rendering.
    theme: DecorationTheme,
}

impl DecorationRenderer {
    /// Create a new decoration renderer with the default theme.
    pub fn new() -> Self {
        Self {
            theme: DecorationTheme::default(),
        }
    }

    /// Create with a custom theme.
    pub fn with_theme(theme: DecorationTheme) -> Self {
        Self { theme }
    }

    /// Get the current theme.
    pub fn theme(&self) -> &DecorationTheme {
        &self.theme
    }

    /// Title bar height.
    pub const fn title_bar_height(&self) -> u32 {
        self.theme.title_bar_height
    }

    /// Border width.
    pub const fn border_width(&self) -> u32 {
        self.theme.border_width
    }

    /// Compute outer bounds for a surface (surface bounds + decorations).
    #[allow(clippy::cast_possible_wrap)]
    pub fn outer_bounds(&self, surface_bounds: &Rect) -> Rect {
        let bw = self.theme.border_width as i32;
        Rect::new(
            surface_bounds.x - bw,
            surface_bounds.y - self.theme.title_bar_height as i32 - bw,
            surface_bounds.width + self.theme.border_width * 2,
            surface_bounds.height + self.theme.title_bar_height + self.theme.border_width * 2,
        )
    }

    /// Render decorations for a window.
    ///
    /// Returns a `DecorationFrame` with the RGBA framebuffer for the
    /// decoration area. The surface content is NOT included — it gets
    /// composited separately.
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub fn render(&self, surface_bounds: &Rect, title: &str, is_focused: bool) -> DecorationFrame {
        let outer = self.outer_bounds(surface_bounds);
        let fb_size = outer.width as usize * outer.height as usize * 4;
        let mut fb = vec![0u8; fb_size];
        let bw = self.theme.border_width;
        let tbh = self.theme.title_bar_height;

        // Title bar
        let title_color = if is_focused {
            self.theme.active_title
        } else {
            self.theme.inactive_title
        };
        let title_bar = Rect::new(
            outer.x + bw as i32,
            outer.y + bw as i32,
            surface_bounds.width,
            tbh,
        );
        fill_rect_in_buffer(
            &mut fb,
            outer.width,
            bw as i32,
            bw as i32,
            surface_bounds.width,
            tbh,
            title_color,
        );

        // Borders and buttons
        self.draw_borders(&mut fb, &outer, bw);
        let (close_x, max_x, min_x, btn_y) = self.draw_buttons(&mut fb, &outer, bw, tbh);

        // Title text placeholder (real text via cosmic-text later)
        self.draw_title_text(&mut fb, title, outer.width, surface_bounds.width, bw);

        // Button rects in screen coordinates
        let btn_w = self.theme.button_width;
        let btn_h = self.theme.button_height;
        DecorationFrame {
            outer_bounds: outer,
            title_bar,
            close_button: Rect::new(outer.x + close_x, outer.y + btn_y, btn_w, btn_h),
            maximize_button: Rect::new(outer.x + max_x, outer.y + btn_y, btn_w, btn_h),
            minimize_button: Rect::new(outer.x + min_x, outer.y + btn_y, btn_w, btn_h),
            framebuffer: fb,
        }
    }

    /// Draw the four borders around the decoration frame.
    #[allow(clippy::cast_possible_wrap)]
    fn draw_borders(&self, fb: &mut [u8], outer: &Rect, bw: u32) {
        let c = self.theme.border;
        fill_rect_in_buffer(fb, outer.width, 0, 0, outer.width, bw, c);
        fill_rect_in_buffer(
            fb,
            outer.width,
            0,
            (outer.height - bw) as i32,
            outer.width,
            bw,
            c,
        );
        fill_rect_in_buffer(fb, outer.width, 0, 0, bw, outer.height, c);
        fill_rect_in_buffer(
            fb,
            outer.width,
            (outer.width - bw) as i32,
            0,
            bw,
            outer.height,
            c,
        );
    }

    /// Draw the three title bar buttons. Returns (close_x, max_x, min_x, btn_y).
    #[allow(clippy::cast_possible_wrap)]
    fn draw_buttons(&self, fb: &mut [u8], outer: &Rect, bw: u32, tbh: u32) -> (i32, i32, i32, i32) {
        let btn_w = self.theme.button_width;
        let btn_h = self.theme.button_height;
        let btn_y = bw as i32 + (tbh as i32 - btn_h as i32) / 2;
        let close_x = (outer.width - bw - btn_w) as i32;
        let max_x = close_x - btn_w as i32;
        let min_x = max_x - btn_w as i32;
        fill_rect_in_buffer(
            fb,
            outer.width,
            close_x,
            btn_y,
            btn_w,
            btn_h,
            self.theme.close_button,
        );
        fill_rect_in_buffer(
            fb,
            outer.width,
            max_x,
            btn_y,
            btn_w,
            btn_h,
            self.theme.maximize_button,
        );
        fill_rect_in_buffer(
            fb,
            outer.width,
            min_x,
            btn_y,
            btn_w,
            btn_h,
            self.theme.minimize_button,
        );
        (close_x, max_x, min_x, btn_y)
    }

    /// Draw a placeholder title text block.
    #[allow(clippy::cast_possible_wrap)]
    fn draw_title_text(&self, fb: &mut [u8], title: &str, outer_w: u32, surface_w: u32, bw: u32) {
        if !title.is_empty() {
            let text_x = bw as i32 + 8;
            let text_y = bw as i32 + 6;
            let max_w = surface_w.saturating_sub(self.theme.button_width * 3 + 16);
            let char_w = core::cmp::min(title.len() as u32 * 8, max_w);
            fill_rect_in_buffer(
                fb,
                outer_w,
                text_x,
                text_y,
                char_w,
                16,
                self.theme.title_text,
            );
        }
    }
}

impl Default for DecorationRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Fill a rectangle within a buffer.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
fn fill_rect_in_buffer(
    buf: &mut [u8],
    buf_width: u32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    color: [u8; 4],
) {
    for row in 0..h {
        let py = y + row as i32;
        if py < 0 {
            continue;
        }
        let py = py as u32;

        for col in 0..w {
            let px = x + col as i32;
            if px < 0 {
                continue;
            }
            let px = px as u32;

            if px < buf_width {
                let idx = (py as usize * buf_width as usize + px as usize) * 4;
                if idx + 3 < buf.len() {
                    buf[idx] = color[0];
                    buf[idx + 1] = color[1];
                    buf[idx + 2] = color[2];
                    buf[idx + 3] = color[3];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme() {
        let theme = DecorationTheme::default();
        assert_eq!(theme.title_bar_height, 30);
        assert_eq!(theme.border_width, 1);
        assert_eq!(theme.button_width, 32);
        assert_eq!(theme.close_button, [200, 60, 60, 255]);
    }

    #[test]
    fn outer_bounds_expands() {
        let renderer = DecorationRenderer::new();
        let surface = Rect::new(100, 130, 400, 300);
        let outer = renderer.outer_bounds(&surface);

        // Should expand by title_bar_height (30) on top + border (1) on all sides
        assert_eq!(outer.x, 99);
        assert_eq!(outer.y, 99);
        assert_eq!(outer.width, 402);
        assert_eq!(outer.height, 332);
    }

    #[test]
    fn render_decoration_frame() {
        let renderer = DecorationRenderer::new();
        let surface = Rect::new(100, 130, 200, 100);

        let frame = renderer.render(&surface, "Test Window", true);

        // Framebuffer size = outer_bounds area * 4
        let expected_size =
            frame.outer_bounds.width as usize * frame.outer_bounds.height as usize * 4;
        assert_eq!(frame.framebuffer.len(), expected_size);
    }

    #[test]
    fn render_focused_vs_unfocused() {
        let renderer = DecorationRenderer::new();
        let surface = Rect::new(0, 30, 100, 50);

        let focused = renderer.render(&surface, "Win", true);
        let unfocused = renderer.render(&surface, "Win", false);

        // Focused and unfocused should have different title bar colors
        // Check a pixel in the title bar area (past the border)
        let theme = renderer.theme();
        let bw = theme.border_width as usize;
        let title_pixel_idx = ((bw + 2) * focused.outer_bounds.width as usize + bw + 2) * 4;

        if title_pixel_idx + 3 < focused.framebuffer.len() {
            // Focused should have active_title color
            assert_eq!(focused.framebuffer[title_pixel_idx], theme.active_title[0]);
            // Unfocused should have inactive_title color
            assert_eq!(
                unfocused.framebuffer[title_pixel_idx],
                theme.inactive_title[0]
            );
        }
    }

    #[test]
    fn close_button_positioned_right() {
        let renderer = DecorationRenderer::new();
        let surface = Rect::new(100, 130, 400, 300);

        let frame = renderer.render(&surface, "Window", true);

        // Close button should be rightmost
        assert!(frame.close_button.x > frame.maximize_button.x);
        assert!(frame.maximize_button.x > frame.minimize_button.x);
    }

    #[test]
    fn button_order_matches_convention() {
        let renderer = DecorationRenderer::new();
        let surface = Rect::new(0, 30, 300, 200);

        let frame = renderer.render(&surface, "App", false);

        // Buttons: minimize | maximize | close (left to right)
        let btn_w = renderer.theme().button_width as i32;
        assert_eq!(frame.close_button.x - frame.maximize_button.x, btn_w);
        assert_eq!(frame.maximize_button.x - frame.minimize_button.x, btn_w);
    }

    #[test]
    fn empty_title_no_crash() {
        let renderer = DecorationRenderer::new();
        let surface = Rect::new(0, 30, 100, 50);
        let _frame = renderer.render(&surface, "", true);
        // Should not panic
    }

    #[test]
    fn fill_rect_basic() {
        let mut buf = vec![0u8; 4 * 4 * 4]; // 4x4 RGBA
        fill_rect_in_buffer(&mut buf, 4, 1, 1, 2, 2, [255, 0, 0, 255]);

        // Pixel (1,1) should be red
        let idx = (1 * 4 + 1) * 4;
        assert_eq!(buf[idx], 255);
        assert_eq!(buf[idx + 1], 0);
        assert_eq!(buf[idx + 2], 0);

        // Pixel (0,0) should still be black
        assert_eq!(buf[0], 0);
    }
}
