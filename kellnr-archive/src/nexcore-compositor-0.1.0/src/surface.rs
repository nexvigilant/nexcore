// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Surface management — rendering targets for applications.
//!
//! ## Primitive Grounding
//!
//! - ∂ Boundary: Surface bounds (x, y, width, height)
//! - λ Location: Surface position on screen
//! - ∃ Existence: Surface lifecycle (created/destroyed)
//! - π Persistence: Surface content survives redraws

use serde::{Deserialize, Serialize};

/// Unique surface identifier.
///
/// Tier: T2-P (∃ Existence — surface identity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SurfaceId(u32);

impl SurfaceId {
    /// Create a new surface ID.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Rectangle defining a surface's bounds.
///
/// Tier: T2-P (∂ Boundary + λ Location)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    /// X position (left edge).
    pub x: i32,
    /// Y position (top edge).
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Rect {
    /// Create a new rectangle.
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Full-screen rectangle at the origin.
    pub const fn full_screen(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// Whether a point is inside this rectangle.
    #[allow(clippy::cast_possible_wrap)]
    pub const fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && py >= self.y
            && px < self.x + self.width as i32
            && py < self.y + self.height as i32
    }

    /// Area in pixels.
    pub const fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// Surface visibility state.
///
/// Tier: T2-P (ς State — visibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// Fully visible.
    Visible,
    /// Partially occluded by another surface.
    Occluded,
    /// Minimized / off-screen.
    Hidden,
}

/// A compositing surface — the rendering target for an application.
///
/// Tier: T3 (∂ + λ + ∃ + π — bounded, located, existing, persistent)
///
/// Each application gets one or more surfaces. The compositor
/// arranges surfaces according to the active mode (single-app,
/// stack, or window manager).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Surface {
    /// Unique identifier.
    pub id: SurfaceId,
    /// Owning service/app name.
    pub owner: String,
    /// Surface title (window title).
    pub title: String,
    /// Bounds on screen.
    pub bounds: Rect,
    /// Z-order (higher = closer to viewer).
    pub z_order: u32,
    /// Current visibility.
    pub visibility: Visibility,
    /// Whether this surface has input focus.
    pub focused: bool,
    /// Framebuffer (RGBA, row-major).
    /// `None` until first paint.
    #[serde(skip)]
    pub framebuffer: Option<Vec<u8>>,
}

impl Surface {
    /// Create a new surface.
    pub fn new(id: SurfaceId, owner: impl Into<String>, bounds: Rect) -> Self {
        Self {
            id,
            owner: owner.into(),
            title: String::new(),
            bounds,
            z_order: 0,
            visibility: Visibility::Visible,
            focused: false,
            framebuffer: None,
        }
    }

    /// Set the surface title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the z-order.
    #[must_use]
    pub fn with_z_order(mut self, z: u32) -> Self {
        self.z_order = z;
        self
    }

    /// Allocate the framebuffer (RGBA, 4 bytes per pixel).
    pub fn allocate_framebuffer(&mut self) {
        let size = self.bounds.width as usize * self.bounds.height as usize * 4;
        self.framebuffer = Some(vec![0u8; size]);
    }

    /// Fill the framebuffer with a solid color.
    pub fn fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        if let Some(ref mut fb) = self.framebuffer {
            for pixel in fb.chunks_exact_mut(4) {
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
                pixel[3] = a;
            }
        }
    }

    /// Whether the surface has content to render.
    pub fn has_content(&self) -> bool {
        self.framebuffer.is_some()
    }

    /// Stride (bytes per row).
    pub fn stride(&self) -> usize {
        self.bounds.width as usize * 4
    }
}

/// A surface manager that tracks all active surfaces.
///
/// Tier: T3 (Σ Sum + σ Sequence — ordered collection)
pub struct SurfaceManager {
    /// All active surfaces.
    surfaces: Vec<Surface>,
    /// Next surface ID to allocate.
    next_id: u32,
}

impl SurfaceManager {
    /// Create a new surface manager.
    pub fn new() -> Self {
        Self {
            surfaces: Vec::new(),
            next_id: 1,
        }
    }

    /// Create a new surface.
    pub fn create_surface(&mut self, owner: impl Into<String>, bounds: Rect) -> SurfaceId {
        let id = SurfaceId::new(self.next_id);
        self.next_id += 1;

        let mut surface = Surface::new(id, owner, bounds);
        surface.z_order = self.surfaces.len() as u32;
        self.surfaces.push(surface);
        id
    }

    /// Destroy a surface.
    pub fn destroy_surface(&mut self, id: SurfaceId) {
        self.surfaces.retain(|s| s.id != id);
    }

    /// Get a surface by ID.
    pub fn get(&self, id: SurfaceId) -> Option<&Surface> {
        self.surfaces.iter().find(|s| s.id == id)
    }

    /// Get a mutable reference to a surface.
    pub fn get_mut(&mut self, id: SurfaceId) -> Option<&mut Surface> {
        self.surfaces.iter_mut().find(|s| s.id == id)
    }

    /// Get the currently focused surface.
    pub fn focused(&self) -> Option<&Surface> {
        self.surfaces.iter().find(|s| s.focused)
    }

    /// Set focus to a surface (unfocuses all others).
    pub fn set_focus(&mut self, id: SurfaceId) {
        for s in &mut self.surfaces {
            s.focused = s.id == id;
        }
    }

    /// Get surfaces in z-order (back to front).
    pub fn in_z_order(&self) -> Vec<&Surface> {
        let mut sorted: Vec<_> = self.surfaces.iter().collect();
        sorted.sort_by_key(|s| s.z_order);
        sorted
    }

    /// Number of active surfaces.
    pub fn count(&self) -> usize {
        self.surfaces.len()
    }

    /// Find the surface at a given point (topmost, front-to-back).
    pub fn surface_at(&self, x: i32, y: i32) -> Option<&Surface> {
        self.surfaces
            .iter()
            .filter(|s| s.visibility == Visibility::Visible)
            .filter(|s| s.bounds.contains(x, y))
            .max_by_key(|s| s.z_order)
    }
}

impl Default for SurfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_basics() {
        let r = Rect::new(10, 20, 100, 50);
        assert!(r.contains(10, 20));
        assert!(r.contains(50, 40));
        assert!(!r.contains(110, 20)); // x=110 is past right edge
        assert!(!r.contains(10, 70)); // y=70 is past bottom
        assert_eq!(r.area(), 5000);
    }

    #[test]
    fn rect_full_screen() {
        let r = Rect::full_screen(1920, 1080);
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
        assert!(r.contains(0, 0));
        assert!(r.contains(1919, 1079));
        assert!(!r.contains(1920, 1080));
    }

    #[test]
    fn surface_creation() {
        let mut mgr = SurfaceManager::new();
        let id = mgr.create_surface("test-app", Rect::full_screen(800, 600));

        assert_eq!(mgr.count(), 1);
        let s = mgr.get(id);
        assert!(s.is_some());

        if let Some(surface) = s {
            assert_eq!(surface.owner, "test-app");
            assert_eq!(surface.bounds.width, 800);
        }
    }

    #[test]
    fn surface_focus() {
        let mut mgr = SurfaceManager::new();
        let id1 = mgr.create_surface("app1", Rect::new(0, 0, 400, 300));
        let id2 = mgr.create_surface("app2", Rect::new(400, 0, 400, 300));

        mgr.set_focus(id1);
        assert!(mgr.get(id1).map_or(false, |s| s.focused));
        assert!(mgr.get(id2).map_or(false, |s| !s.focused));

        mgr.set_focus(id2);
        assert!(mgr.get(id1).map_or(false, |s| !s.focused));
        assert!(mgr.get(id2).map_or(false, |s| s.focused));
    }

    #[test]
    fn surface_destroy() {
        let mut mgr = SurfaceManager::new();
        let id = mgr.create_surface("temp", Rect::full_screen(100, 100));
        assert_eq!(mgr.count(), 1);

        mgr.destroy_surface(id);
        assert_eq!(mgr.count(), 0);
        assert!(mgr.get(id).is_none());
    }

    #[test]
    fn z_order_sorting() {
        let mut mgr = SurfaceManager::new();
        let id1 = mgr.create_surface("back", Rect::full_screen(100, 100));
        let _id2 = mgr.create_surface("front", Rect::full_screen(100, 100));

        // id1 has z=0 (back), id2 has z=1 (front)
        let ordered = mgr.in_z_order();
        assert_eq!(ordered[0].id, id1); // Back first
    }

    #[test]
    fn surface_at_point() {
        let mut mgr = SurfaceManager::new();
        let _bg = mgr.create_surface("bg", Rect::full_screen(800, 600));
        let fg = mgr.create_surface("fg", Rect::new(100, 100, 200, 200));

        // Point inside foreground → returns fg (higher z-order)
        let hit = mgr.surface_at(150, 150);
        assert!(hit.is_some());
        if let Some(s) = hit {
            assert_eq!(s.id, fg);
        }

        // Point outside foreground but inside background → returns bg
        let hit = mgr.surface_at(50, 50);
        assert!(hit.is_some());
    }

    #[test]
    fn framebuffer_allocation() {
        let mut surface = Surface::new(SurfaceId::new(1), "test", Rect::full_screen(100, 50));
        assert!(!surface.has_content());

        surface.allocate_framebuffer();
        assert!(surface.has_content());

        // 100 * 50 * 4 bytes (RGBA)
        if let Some(ref fb) = surface.framebuffer {
            assert_eq!(fb.len(), 20_000);
        }
    }

    #[test]
    fn surface_fill() {
        let mut surface = Surface::new(SurfaceId::new(1), "test", Rect::new(0, 0, 2, 2));
        surface.allocate_framebuffer();
        surface.fill(255, 0, 0, 255); // Red

        if let Some(ref fb) = surface.framebuffer {
            // First pixel: R=255, G=0, B=0, A=255
            assert_eq!(fb[0], 255);
            assert_eq!(fb[1], 0);
            assert_eq!(fb[2], 0);
            assert_eq!(fb[3], 255);
        }
    }

    #[test]
    fn surface_stride() {
        let surface = Surface::new(SurfaceId::new(1), "test", Rect::new(0, 0, 640, 480));
        assert_eq!(surface.stride(), 2560); // 640 * 4
    }
}
