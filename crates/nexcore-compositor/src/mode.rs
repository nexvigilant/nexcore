// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Compositor mode — form-factor-specific display strategies.
//!
//! ## Primitive Grounding
//!
//! - κ Comparison: Mode selection based on form factor
//! - ∂ Boundary: Each mode defines surface layout constraints
//! - σ Sequence: App switching order (stack mode)

use nexcore_pal::FormFactor;
use serde::{Deserialize, Serialize};

/// Display compositor mode.
///
/// Tier: T2-C (κ + ∂ + σ — mode selection with layout constraints)
///
/// Determines how surfaces are arranged on screen.
/// Selected automatically based on the device form factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositorMode {
    /// Watch: Single full-screen application.
    /// No window chrome, no switching UI. Direct touch input.
    SingleApp,

    /// Phone: Full-screen app stack with gesture navigation.
    /// Swipe to switch apps. One visible at a time.
    AppStack,

    /// Desktop: Floating/tiling window manager.
    /// Multiple visible windows with focus management.
    WindowManager,
}

impl CompositorMode {
    /// Select compositor mode for a given form factor.
    pub const fn for_form_factor(ff: FormFactor) -> Self {
        match ff {
            FormFactor::Watch => Self::SingleApp,
            FormFactor::Phone => Self::AppStack,
            FormFactor::Desktop => Self::WindowManager,
        }
    }

    /// Whether this mode supports multiple visible surfaces.
    pub const fn supports_multi_window(self) -> bool {
        matches!(self, Self::WindowManager)
    }

    /// Whether this mode shows window decorations (title bar, borders).
    pub const fn has_decorations(self) -> bool {
        matches!(self, Self::WindowManager)
    }

    /// Maximum simultaneously visible surfaces.
    pub const fn max_visible(self) -> usize {
        match self {
            Self::SingleApp | Self::AppStack => 1,
            Self::WindowManager => 64,
        }
    }

    /// Human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::SingleApp => "Single App",
            Self::AppStack => "App Stack",
            Self::WindowManager => "Window Manager",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_selection() {
        assert_eq!(
            CompositorMode::for_form_factor(FormFactor::Watch),
            CompositorMode::SingleApp,
        );
        assert_eq!(
            CompositorMode::for_form_factor(FormFactor::Phone),
            CompositorMode::AppStack,
        );
        assert_eq!(
            CompositorMode::for_form_factor(FormFactor::Desktop),
            CompositorMode::WindowManager,
        );
    }

    #[test]
    fn multi_window_support() {
        assert!(!CompositorMode::SingleApp.supports_multi_window());
        assert!(!CompositorMode::AppStack.supports_multi_window());
        assert!(CompositorMode::WindowManager.supports_multi_window());
    }

    #[test]
    fn decorations() {
        assert!(!CompositorMode::SingleApp.has_decorations());
        assert!(CompositorMode::WindowManager.has_decorations());
    }

    #[test]
    fn max_visible() {
        assert_eq!(CompositorMode::SingleApp.max_visible(), 1);
        assert_eq!(CompositorMode::WindowManager.max_visible(), 64);
    }
}
