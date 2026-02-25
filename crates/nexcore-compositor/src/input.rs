// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Input event routing — dispatches input to the correct surface.
//!
//! ## Primitive Grounding
//!
//! - μ Mapping: Input event → target surface
//! - λ Location: Hit-testing at screen coordinates
//! - → Causality: Input causes surface state change
//! - ∂ Boundary: Surface bounds constrain hit regions

use nexcore_pal::InputEvent;

use crate::surface::{Rect, SurfaceId};

/// The result of routing an input event.
///
/// Tier: T2-C (μ + λ + →)
#[derive(Debug, Clone)]
pub enum InputTarget {
    /// Event should go to a specific surface.
    Surface(SurfaceId),
    /// Event hit a window decoration (title bar, resize handle, etc.).
    Decoration(SurfaceId, DecorationZone),
    /// Event hit the desktop background (no surface).
    Desktop,
    /// Event is a global hotkey (handled by compositor).
    Global(GlobalAction),
}

/// Zones within a window decoration.
///
/// Tier: T2-P (λ Location — decoration subregions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationZone {
    /// Title bar (for dragging).
    TitleBar,
    /// Close button.
    CloseButton,
    /// Minimize button.
    MinimizeButton,
    /// Maximize/restore button.
    MaximizeButton,
    /// Resize handle (edge or corner).
    ResizeHandle(ResizeEdge),
}

/// Edge/corner for resize operations.
///
/// Tier: T2-P (∂ Boundary — edge identification)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    /// Top edge.
    Top,
    /// Bottom edge.
    Bottom,
    /// Left edge.
    Left,
    /// Right edge.
    Right,
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
}

/// Global compositor actions triggered by input.
///
/// Tier: T2-P (→ Causality — action triggers)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalAction {
    /// Switch to next app (Alt+Tab, swipe gesture).
    SwitchApp,
    /// Open app launcher.
    OpenLauncher,
    /// Lock screen.
    LockScreen,
    /// Take screenshot.
    Screenshot,
    /// Open notification panel.
    NotificationPanel,
}

/// Input router — determines where input events go.
///
/// Tier: T3 (μ + λ + → + ∂)
pub struct InputRouter {
    /// Title bar height in pixels.
    title_bar_height: u32,
    /// Resize handle size in pixels.
    resize_handle_size: u32,
    /// Button width in title bar.
    button_width: u32,
}

impl InputRouter {
    /// Create a new input router with default decoration sizes.
    pub fn new() -> Self {
        Self {
            title_bar_height: 30,
            resize_handle_size: 6,
            button_width: 32,
        }
    }

    /// Set title bar height.
    #[must_use]
    pub const fn with_title_bar_height(mut self, height: u32) -> Self {
        self.title_bar_height = height;
        self
    }

    /// Set resize handle size.
    #[must_use]
    pub const fn with_resize_handle_size(mut self, size: u32) -> Self {
        self.resize_handle_size = size;
        self
    }

    /// Title bar height.
    pub const fn title_bar_height(&self) -> u32 {
        self.title_bar_height
    }

    /// Route a pointer event to the appropriate target.
    ///
    /// Checks surfaces front-to-back (highest z-order first),
    /// testing decoration zones before client area.
    #[allow(clippy::cast_possible_wrap)]
    pub fn route_pointer(
        &self,
        x: i32,
        y: i32,
        surfaces: &[(SurfaceId, Rect, u32, bool)], // (id, bounds, z_order, has_decorations)
    ) -> InputTarget {
        // Sort by z-order descending (front to back)
        let mut sorted: Vec<_> = surfaces.iter().collect();
        sorted.sort_by(|a, b| b.2.cmp(&a.2));

        for &(id, bounds, _, has_decorations) in &sorted {
            // Expand bounds to include decorations
            let hit_bounds = if *has_decorations {
                Rect::new(
                    bounds.x - self.resize_handle_size as i32,
                    bounds.y - self.title_bar_height as i32,
                    bounds.width + self.resize_handle_size * 2,
                    bounds.height + self.title_bar_height + self.resize_handle_size,
                )
            } else {
                *bounds
            };

            if !hit_bounds.contains(x, y) {
                continue;
            }

            if *has_decorations {
                // Check decoration zones
                if let Some(zone) = self.hit_test_decorations(x, y, bounds) {
                    return InputTarget::Decoration(*id, zone);
                }
            }

            // Check client area
            if bounds.contains(x, y) {
                return InputTarget::Surface(*id);
            }
        }

        InputTarget::Desktop
    }

    /// Route a keyboard event to the focused surface.
    pub fn route_keyboard(&self, _event: &InputEvent, focused: Option<SurfaceId>) -> InputTarget {
        self.route_focused_surface(focused)
    }

    fn route_focused_surface(&self, focused: Option<SurfaceId>) -> InputTarget {
        focused.map_or(InputTarget::Desktop, InputTarget::Surface)
    }

    /// Hit-test decoration zones for a decorated window.
    #[allow(clippy::cast_possible_wrap)]
    fn hit_test_decorations(&self, x: i32, y: i32, bounds: &Rect) -> Option<DecorationZone> {
        let title_top = bounds.y - self.title_bar_height as i32;
        let title_bottom = bounds.y;

        // Title bar region
        if y >= title_top && y < title_bottom && x >= bounds.x && x < bounds.x + bounds.width as i32
        {
            // Check title bar buttons (right side)
            let right_edge = bounds.x + bounds.width as i32;

            // Close button (rightmost)
            if x >= right_edge - self.button_width as i32 {
                return Some(DecorationZone::CloseButton);
            }

            // Maximize button (second from right)
            if x >= right_edge - (self.button_width * 2) as i32 {
                return Some(DecorationZone::MaximizeButton);
            }

            // Minimize button (third from right)
            if x >= right_edge - (self.button_width * 3) as i32 {
                return Some(DecorationZone::MinimizeButton);
            }

            return Some(DecorationZone::TitleBar);
        }

        // Resize edges
        let handle = self.resize_handle_size as i32;
        let left = bounds.x;
        let right = bounds.x + bounds.width as i32;
        let top = bounds.y;
        let bottom = bounds.y + bounds.height as i32;

        // Corners first (more specific)
        if x < left + handle && y < top + handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::TopLeft));
        }
        if x >= right - handle && y < top + handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::TopRight));
        }
        if x < left + handle && y >= bottom - handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::BottomLeft));
        }
        if x >= right - handle && y >= bottom - handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::BottomRight));
        }

        // Edges
        if x < left + handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::Left));
        }
        if x >= right - handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::Right));
        }
        if y < top + handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::Top));
        }
        if y >= bottom - handle {
            return Some(DecorationZone::ResizeHandle(ResizeEdge::Bottom));
        }

        None
    }
}

impl Default for InputRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_surfaces() -> Vec<(SurfaceId, Rect, u32, bool)> {
        vec![
            (SurfaceId::new(1), Rect::new(0, 0, 800, 600), 0, false), // bg, no decorations
            (SurfaceId::new(2), Rect::new(100, 130, 400, 300), 1, true), // window with decorations (title bar at y=100)
        ]
    }

    #[test]
    fn route_to_surface() {
        let router = InputRouter::new();
        let surfaces = test_surfaces();

        // Click inside window client area
        let target = router.route_pointer(200, 200, &surfaces);
        assert!(matches!(target, InputTarget::Surface(id) if id == SurfaceId::new(2)));
    }

    #[test]
    fn route_to_desktop() {
        let router = InputRouter::new();
        // No surfaces — should hit desktop
        let target = router.route_pointer(500, 500, &[]);
        assert!(matches!(target, InputTarget::Desktop));
    }

    #[test]
    fn route_to_title_bar() {
        let router = InputRouter::new();
        let surfaces = test_surfaces();

        // Click in title bar (y = 110, which is between title_top=100 and title_bottom=130)
        let target = router.route_pointer(200, 110, &surfaces);
        assert!(
            matches!(target, InputTarget::Decoration(id, DecorationZone::TitleBar) if id == SurfaceId::new(2))
        );
    }

    #[test]
    fn route_to_close_button() {
        let router = InputRouter::new();
        let surfaces = test_surfaces();

        // Close button is rightmost in title bar (x=480..500, y=100..130)
        let target = router.route_pointer(490, 110, &surfaces);
        assert!(
            matches!(target, InputTarget::Decoration(id, DecorationZone::CloseButton) if id == SurfaceId::new(2))
        );
    }

    #[test]
    fn route_keyboard_to_focused() {
        let router = InputRouter::new();

        let target = router.route_focused_surface(Some(SurfaceId::new(5)));
        assert!(matches!(target, InputTarget::Surface(id) if id == SurfaceId::new(5)));
    }

    #[test]
    fn route_keyboard_no_focus() {
        let router = InputRouter::new();

        let target = router.route_focused_surface(None);
        assert!(matches!(target, InputTarget::Desktop));
    }

    #[test]
    fn route_z_order_front_to_back() {
        let router = InputRouter::new();
        let surfaces = vec![
            (SurfaceId::new(1), Rect::new(0, 0, 400, 400), 0, false),
            (SurfaceId::new(2), Rect::new(0, 0, 400, 400), 1, false), // same bounds, higher z
        ];

        // Should hit surface 2 (higher z-order)
        let target = router.route_pointer(200, 200, &surfaces);
        assert!(matches!(target, InputTarget::Surface(id) if id == SurfaceId::new(2)));
    }

    #[test]
    fn resize_handle_corners() {
        let router = InputRouter::new();
        let surfaces = vec![(SurfaceId::new(1), Rect::new(100, 130, 400, 300), 0, true)];

        // Bottom-right corner (x=494..500, y=424..430)
        let target = router.route_pointer(498, 428, &surfaces);
        assert!(matches!(
            target,
            InputTarget::Decoration(_, DecorationZone::ResizeHandle(ResizeEdge::BottomRight))
        ));
    }

    #[test]
    fn no_decorations_bypass() {
        let router = InputRouter::new();
        let surfaces = vec![
            (SurfaceId::new(1), Rect::new(100, 100, 400, 300), 0, false), // no decorations
        ];

        // Click where title bar would be if decorated — should miss since no decorations
        let target = router.route_pointer(200, 80, &surfaces);
        assert!(matches!(target, InputTarget::Desktop));
    }
}
