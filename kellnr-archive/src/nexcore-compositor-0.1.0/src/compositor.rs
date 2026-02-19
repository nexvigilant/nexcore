// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Main compositor — orchestrates surfaces, input, and display output.
//!
//! ## Primitive Grounding
//!
//! - Σ Sum: Composites multiple surfaces into one framebuffer
//! - μ Mapping: Maps input events to focused surface
//! - σ Sequence: Frame rendering pipeline
//! - ς State: Compositor lifecycle

use nexcore_pal::{FormFactor, Platform};

use crate::mode::CompositorMode;
use crate::surface::{Rect, Surface, SurfaceId, SurfaceManager, Visibility};

/// Compositor lifecycle state.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositorState {
    /// Not yet initialized.
    Idle,
    /// Running and compositing frames.
    Running,
    /// Suspended (device sleeping, etc.)
    Suspended,
    /// Shut down.
    Stopped,
}

/// The NexCore compositor — display server for all form factors.
///
/// Tier: T3 (Σ + μ + σ + ς — full compositor)
///
/// Manages surfaces, composites them into a display framebuffer,
/// and routes input events to the focused surface.
pub struct Compositor {
    /// Operating mode (determined by form factor).
    mode: CompositorMode,
    /// Surface manager.
    surfaces: SurfaceManager,
    /// Display resolution.
    resolution: (u32, u32),
    /// Composited framebuffer (RGBA).
    framebuffer: Vec<u8>,
    /// Compositor state.
    state: CompositorState,
    /// Frame counter.
    frame_count: u64,
    /// Background color (RGBA).
    background: [u8; 4],
}

impl Compositor {
    /// Create a new compositor for the given form factor and resolution.
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(form_factor: FormFactor, width: u32, height: u32) -> Self {
        let mode = CompositorMode::for_form_factor(form_factor);
        let fb_size = width as usize * height as usize * 4;

        Self {
            mode,
            surfaces: SurfaceManager::new(),
            resolution: (width, height),
            framebuffer: vec![0u8; fb_size],
            state: CompositorState::Idle,
            frame_count: 0,
            background: [15, 15, 26, 255], // Dark blue-black (#0f0f1a)
        }
    }

    /// Create from a platform's form factor and display.
    pub fn from_platform<P: Platform>(platform: &P) -> Self {
        let ff = platform.form_factor();
        let res = ff.min_resolution();
        Self::new(ff, res.width, res.height)
    }

    /// Start the compositor.
    pub fn start(&mut self) {
        self.state = CompositorState::Running;
    }

    /// Suspend the compositor (device sleep).
    pub fn suspend(&mut self) {
        self.state = CompositorState::Suspended;
    }

    /// Resume from suspension.
    pub fn resume(&mut self) {
        if self.state == CompositorState::Suspended {
            self.state = CompositorState::Running;
        }
    }

    /// Stop the compositor.
    pub fn stop(&mut self) {
        self.state = CompositorState::Stopped;
    }

    /// Create a new surface for an application.
    pub fn create_surface(&mut self, owner: impl Into<String>, bounds: Rect) -> SurfaceId {
        let id = self.surfaces.create_surface(owner, bounds);

        // In single-app mode, auto-focus the first surface
        if self.mode == CompositorMode::SingleApp && self.surfaces.count() == 1 {
            self.surfaces.set_focus(id);
        }

        id
    }

    /// Create a full-screen surface (for watch/phone modes).
    pub fn create_fullscreen_surface(&mut self, owner: impl Into<String>) -> SurfaceId {
        let (w, h) = self.resolution;
        self.create_surface(owner, Rect::full_screen(w, h))
    }

    /// Destroy a surface.
    pub fn destroy_surface(&mut self, id: SurfaceId) {
        self.surfaces.destroy_surface(id);
    }

    /// Set focus to a surface.
    pub fn focus_surface(&mut self, id: SurfaceId) {
        self.surfaces.set_focus(id);
    }

    /// Get the focused surface.
    pub fn focused_surface(&self) -> Option<&Surface> {
        self.surfaces.focused()
    }

    /// Get a surface by ID.
    pub fn surface(&self, id: SurfaceId) -> Option<&Surface> {
        self.surfaces.get(id)
    }

    /// Get a mutable surface by ID.
    pub fn surface_mut(&mut self, id: SurfaceId) -> Option<&mut Surface> {
        self.surfaces.get_mut(id)
    }

    /// Composite all visible surfaces into the output framebuffer.
    ///
    /// Software compositor — copies surface framebuffers into the
    /// composited framebuffer in z-order.
    pub fn composite(&mut self) {
        if self.state != CompositorState::Running {
            return;
        }

        // Clear to background color
        for pixel in self.framebuffer.chunks_exact_mut(4) {
            pixel[0] = self.background[0];
            pixel[1] = self.background[1];
            pixel[2] = self.background[2];
            pixel[3] = self.background[3];
        }

        let (display_w, _display_h) = self.resolution;

        // Blit surfaces in z-order (back to front)
        let surfaces = self.surfaces.in_z_order();
        for surface in surfaces {
            if surface.visibility != Visibility::Visible {
                continue;
            }
            if let Some(ref fb) = surface.framebuffer {
                blit_surface(
                    &mut self.framebuffer,
                    self.resolution,
                    fb,
                    &surface.bounds,
                    display_w,
                );
            }
        }

        self.frame_count += 1;
    }

    /// Get the composited framebuffer.
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Get the compositor mode.
    pub fn mode(&self) -> CompositorMode {
        self.mode
    }

    /// Get the compositor state.
    pub fn state(&self) -> CompositorState {
        self.state
    }

    /// Get the display resolution.
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    /// Get the frame count.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Number of active surfaces.
    pub fn surface_count(&self) -> usize {
        self.surfaces.count()
    }

    /// Set the background color.
    pub fn set_background(&mut self, r: u8, g: u8, b: u8) {
        self.background = [r, g, b, 255];
    }

    /// Find which surface is at a screen point.
    pub fn surface_at(&self, x: i32, y: i32) -> Option<&Surface> {
        self.surfaces.surface_at(x, y)
    }
}

/// Blit a surface's framebuffer into the composited framebuffer.
///
/// Free function to avoid borrow-checker conflict (surfaces borrowed
/// immutably for z-order iteration while framebuffer needs &mut).
///
/// Pixel-level math requires integer casts between i32/u32/usize —
/// bounds are validated before each cast.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::cast_lossless
)]
fn blit_surface(dst: &mut [u8], resolution: (u32, u32), src: &[u8], bounds: &Rect, display_w: u32) {
    let src_stride = bounds.width as usize * 4;

    for row in 0..bounds.height {
        let dst_y = bounds.y + row as i32;
        if dst_y < 0 || dst_y >= resolution.1 as i32 {
            continue;
        }

        let src_offset = row as usize * src_stride;
        let src_end = src_offset + src_stride;
        if src_end > src.len() {
            continue;
        }

        for col in 0..bounds.width {
            let dst_x = bounds.x + col as i32;
            if dst_x < 0 || dst_x >= display_w as i32 {
                continue;
            }

            let src_px = src_offset + col as usize * 4;
            let dst_idx = (dst_y as usize * display_w as usize + dst_x as usize) * 4;

            if src_px + 3 < src.len() && dst_idx + 3 < dst.len() {
                // Simple alpha-over compositing
                let alpha = u16::from(src[src_px + 3]);
                if alpha == 255 {
                    // Opaque — direct copy
                    dst[dst_idx] = src[src_px];
                    dst[dst_idx + 1] = src[src_px + 1];
                    dst[dst_idx + 2] = src[src_px + 2];
                    dst[dst_idx + 3] = 255;
                } else if alpha > 0 {
                    // Alpha blend
                    let inv_alpha = 255 - alpha;
                    for c in 0..3 {
                        let s = u16::from(src[src_px + c]);
                        let d = u16::from(dst[dst_idx + c]);
                        dst[dst_idx + c] = ((s * alpha + d * inv_alpha) / 255) as u8;
                    }
                    dst[dst_idx + 3] = 255;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_compositor() {
        let comp = Compositor::new(FormFactor::Desktop, 1920, 1080);
        assert_eq!(comp.mode(), CompositorMode::WindowManager);
        assert_eq!(comp.resolution(), (1920, 1080));
        assert_eq!(comp.state(), CompositorState::Idle);
        assert_eq!(comp.frame_count(), 0);
    }

    #[test]
    fn compositor_lifecycle() {
        let mut comp = Compositor::new(FormFactor::Phone, 1080, 2400);
        assert_eq!(comp.state(), CompositorState::Idle);

        comp.start();
        assert_eq!(comp.state(), CompositorState::Running);

        comp.suspend();
        assert_eq!(comp.state(), CompositorState::Suspended);

        comp.resume();
        assert_eq!(comp.state(), CompositorState::Running);

        comp.stop();
        assert_eq!(comp.state(), CompositorState::Stopped);
    }

    #[test]
    fn watch_single_app_mode() {
        let mut comp = Compositor::new(FormFactor::Watch, 450, 450);
        assert_eq!(comp.mode(), CompositorMode::SingleApp);

        let id = comp.create_fullscreen_surface("guardian");
        assert_eq!(comp.surface_count(), 1);

        // Should auto-focus in single-app mode
        let focused = comp.focused_surface();
        assert!(focused.is_some());
        if let Some(s) = focused {
            assert_eq!(s.id, id);
        }
    }

    #[test]
    fn desktop_window_management() {
        let mut comp = Compositor::new(FormFactor::Desktop, 1920, 1080);
        comp.start();

        let win1 = comp.create_surface("editor", Rect::new(0, 0, 960, 1080));
        let win2 = comp.create_surface("terminal", Rect::new(960, 0, 960, 1080));

        assert_eq!(comp.surface_count(), 2);

        comp.focus_surface(win2);
        let focused = comp.focused_surface();
        assert!(focused.is_some());
        if let Some(s) = focused {
            assert_eq!(s.owner, "terminal");
        }

        comp.destroy_surface(win1);
        assert_eq!(comp.surface_count(), 1);
    }

    #[test]
    fn composite_empty() {
        let mut comp = Compositor::new(FormFactor::Desktop, 4, 4);
        comp.start();
        comp.composite();

        // Should be background color (0x0f0f1aff)
        let fb = comp.framebuffer();
        assert_eq!(fb[0], 15); // R
        assert_eq!(fb[1], 15); // G
        assert_eq!(fb[2], 26); // B
        assert_eq!(fb[3], 255); // A
        assert_eq!(comp.frame_count(), 1);
    }

    #[test]
    fn composite_with_surface() {
        let mut comp = Compositor::new(FormFactor::Desktop, 4, 4);
        comp.start();

        let id = comp.create_fullscreen_surface("app");
        if let Some(s) = comp.surface_mut(id) {
            s.allocate_framebuffer();
            s.fill(255, 0, 0, 255); // Red
        }

        comp.composite();

        // Entire framebuffer should be red
        let fb = comp.framebuffer();
        assert_eq!(fb[0], 255); // R
        assert_eq!(fb[1], 0); // G
        assert_eq!(fb[2], 0); // B
    }

    #[test]
    fn composite_z_order() {
        let mut comp = Compositor::new(FormFactor::Desktop, 4, 4);
        comp.start();

        // Background surface — green
        let bg = comp.create_fullscreen_surface("bg");
        if let Some(s) = comp.surface_mut(bg) {
            s.allocate_framebuffer();
            s.fill(0, 255, 0, 255); // Green
        }

        // Foreground surface — blue (higher z-order)
        let fg = comp.create_fullscreen_surface("fg");
        if let Some(s) = comp.surface_mut(fg) {
            s.allocate_framebuffer();
            s.fill(0, 0, 255, 255); // Blue
        }

        comp.composite();

        // Should be blue (fg is on top)
        let fb = comp.framebuffer();
        assert_eq!(fb[0], 0); // R
        assert_eq!(fb[1], 0); // G
        assert_eq!(fb[2], 255); // B
    }

    #[test]
    fn no_composite_when_stopped() {
        let mut comp = Compositor::new(FormFactor::Desktop, 4, 4);
        // Don't start — state is Idle
        comp.composite();
        assert_eq!(comp.frame_count(), 0); // No frame composited
    }

    #[test]
    fn surface_at_point() {
        let mut comp = Compositor::new(FormFactor::Desktop, 800, 600);
        comp.start();

        let _bg = comp.create_fullscreen_surface("desktop");
        let win = comp.create_surface("window", Rect::new(100, 100, 200, 200));

        let hit = comp.surface_at(150, 150);
        assert!(hit.is_some());
        if let Some(s) = hit {
            assert_eq!(s.id, win);
        }
    }

    #[test]
    fn from_platform() {
        let platform = nexcore_pal_linux::LinuxPlatform::virtual_platform(FormFactor::Watch);
        let comp = Compositor::from_platform(&platform);
        assert_eq!(comp.mode(), CompositorMode::SingleApp);
    }
}
