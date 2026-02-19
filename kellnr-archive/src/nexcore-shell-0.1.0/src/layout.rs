// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Shell layout — form-factor-specific screen layouts.
//!
//! ## Primitive Grounding
//!
//! - ∂ Boundary: Layout constraints per form factor
//! - λ Location: Widget positions on screen
//! - N Quantity: Dimension values

use nexcore_pal::FormFactor;

use nexcore_compositor::surface::Rect;

/// Layout region — a named rectangular area on screen.
///
/// Tier: T2-P (∂ Boundary + λ Location)
#[derive(Debug, Clone)]
pub struct LayoutRegion {
    /// Region name (e.g., "status_bar", "content", "nav_bar").
    pub name: String,
    /// Screen bounds.
    pub bounds: Rect,
}

impl LayoutRegion {
    /// Create a new layout region.
    pub fn new(name: impl Into<String>, bounds: Rect) -> Self {
        Self {
            name: name.into(),
            bounds,
        }
    }
}

/// Shell layout — defines the screen regions for a form factor.
///
/// Tier: T2-C (∂ + λ + N — bounded regions at locations with dimensions)
#[derive(Debug, Clone)]
pub struct ShellLayout {
    /// Form factor this layout targets.
    pub form_factor: FormFactor,
    /// Screen width.
    pub width: u32,
    /// Screen height.
    pub height: u32,
    /// Layout regions (in render order).
    pub regions: Vec<LayoutRegion>,
}

impl ShellLayout {
    /// Create a watch layout (450x450, single content area).
    pub fn watch() -> Self {
        let w = 450;
        let h = 450;
        Self {
            form_factor: FormFactor::Watch,
            width: w,
            height: h,
            regions: vec![
                LayoutRegion::new("status_bar", Rect::new(0, 0, w, 40)),
                LayoutRegion::new("content", Rect::new(0, 40, w, h - 40)),
            ],
        }
    }

    /// Create a phone layout (1080x2400, status + content + nav).
    #[allow(clippy::cast_possible_wrap)]
    pub fn phone() -> Self {
        let w = 1080;
        let h = 2400;
        let status_h = 80;
        let nav_h = 120;
        let content_h = h - status_h - nav_h;
        Self {
            form_factor: FormFactor::Phone,
            width: w,
            height: h,
            regions: vec![
                LayoutRegion::new("status_bar", Rect::new(0, 0, w, status_h)),
                LayoutRegion::new("content", Rect::new(0, status_h as i32, w, content_h)),
                LayoutRegion::new(
                    "nav_bar",
                    Rect::new(0, (status_h + content_h) as i32, w, nav_h),
                ),
            ],
        }
    }

    /// Create a desktop layout (1920x1080, taskbar + content).
    #[allow(clippy::cast_possible_wrap)]
    pub fn desktop() -> Self {
        let w = 1920;
        let h = 1080;
        let taskbar_h = 48;
        let content_h = h - taskbar_h;
        Self {
            form_factor: FormFactor::Desktop,
            width: w,
            height: h,
            regions: vec![
                LayoutRegion::new("content", Rect::new(0, 0, w, content_h)),
                LayoutRegion::new("taskbar", Rect::new(0, content_h as i32, w, taskbar_h)),
            ],
        }
    }

    /// Select layout for a given form factor.
    pub fn for_form_factor(ff: FormFactor) -> Self {
        match ff {
            FormFactor::Watch => Self::watch(),
            FormFactor::Phone => Self::phone(),
            FormFactor::Desktop => Self::desktop(),
        }
    }

    /// Get a region by name.
    pub fn region(&self, name: &str) -> Option<&LayoutRegion> {
        self.regions.iter().find(|r| r.name == name)
    }

    /// Get the content region bounds.
    pub fn content_bounds(&self) -> Option<Rect> {
        self.region("content").map(|r| r.bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_layout() {
        let layout = ShellLayout::watch();
        assert_eq!(layout.form_factor, FormFactor::Watch);
        assert_eq!(layout.width, 450);
        assert_eq!(layout.regions.len(), 2);
        assert!(layout.region("status_bar").is_some());
        assert!(layout.region("content").is_some());
    }

    #[test]
    fn phone_layout() {
        let layout = ShellLayout::phone();
        assert_eq!(layout.form_factor, FormFactor::Phone);
        assert_eq!(layout.regions.len(), 3);
        assert!(layout.region("nav_bar").is_some());
    }

    #[test]
    fn desktop_layout() {
        let layout = ShellLayout::desktop();
        assert_eq!(layout.form_factor, FormFactor::Desktop);
        assert_eq!(layout.regions.len(), 2);
        assert!(layout.region("taskbar").is_some());
    }

    #[test]
    fn for_form_factor() {
        let w = ShellLayout::for_form_factor(FormFactor::Watch);
        assert_eq!(w.width, 450);

        let p = ShellLayout::for_form_factor(FormFactor::Phone);
        assert_eq!(p.width, 1080);

        let d = ShellLayout::for_form_factor(FormFactor::Desktop);
        assert_eq!(d.width, 1920);
    }

    #[test]
    fn content_bounds() {
        let layout = ShellLayout::desktop();
        let bounds = layout.content_bounds();
        assert!(bounds.is_some());
        if let Some(b) = bounds {
            assert_eq!(b.x, 0);
            assert_eq!(b.y, 0);
            assert_eq!(b.width, 1920);
            assert_eq!(b.height, 1032); // 1080 - 48 taskbar
        }
    }

    #[test]
    fn region_not_found() {
        let layout = ShellLayout::watch();
        assert!(layout.region("nonexistent").is_none());
    }
}
