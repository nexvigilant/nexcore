// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Core types for the Platform Abstraction Layer.
//!
//! All types are `no_std` compatible — no heap allocation required.

/// Display resolution descriptor.
///
/// Tier: T2-P (N Quantity + ∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Resolution {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Resolution {
    /// Create a new resolution.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Total pixel count.
    #[allow(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        reason = "pixel count is safe from overflow"
    )]
    pub const fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Aspect ratio as (width_factor, height_factor).
    /// Returns None if either dimension is zero.
    pub const fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    /// Check if square (e.g., smartwatch round displays crop to square).
    pub const fn is_square(&self) -> bool {
        self.width == self.height
    }
}

/// Display shape for per-device rendering.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DisplayShape {
    /// Rectangular display (phone, desktop).
    Rectangle,
    /// Circular display (smartwatch).
    Circle,
    /// Rounded rectangle (phone with corners).
    RoundedRect { corner_radius: u32 },
}

/// Pixel format for framebuffer data.
///
/// Tier: T2-P (μ Mapping — pixel encoding scheme)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum PixelFormat {
    /// 32-bit RGBA (8 bits per channel).
    Rgba8,
    /// 32-bit BGRA (8 bits per channel, common on many GPUs).
    Bgra8,
    /// 16-bit RGB (5-6-5, low-power displays).
    Rgb565,
    /// 24-bit RGB (8 bits per channel, no alpha).
    Rgb8,
}

impl PixelFormat {
    /// Bytes per pixel for this format.
    pub const fn bytes_per_pixel(&self) -> u32 {
        match self {
            Self::Rgba8 | Self::Bgra8 => 4,
            Self::Rgb8 => 3,
            Self::Rgb565 => 2,
        }
    }
}

/// Input event from any input device.
///
/// Tier: T2-C (σ Sequence + ∃ Existence + μ Mapping)
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum InputEvent {
    /// Touch event (phone, watch).
    Touch(TouchEvent),
    /// Key event (desktop keyboard).
    Key(KeyEvent),
    /// Pointer event (desktop mouse/trackpad).
    Pointer(PointerEvent),
    /// Crown/bezel rotation (watch).
    Crown(CrownEvent),
}

/// Touch input event.
///
/// Tier: T2-P (λ Location + σ Sequence)
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct TouchEvent {
    /// Touch ID for multi-touch tracking.
    pub id: u32,
    /// X coordinate (pixels from left).
    pub x: f32,
    /// Y coordinate (pixels from top).
    pub y: f32,
    /// Touch pressure (0.0 = no pressure, 1.0 = max).
    pub pressure: f32,
    /// Touch phase.
    pub phase: TouchPhase,
}

impl TouchEvent {
    /// Create a touch event.
    #[must_use]
    pub const fn new(id: u32, x: f32, y: f32, pressure: f32, phase: TouchPhase) -> Self {
        Self {
            id,
            x,
            y,
            pressure,
            phase,
        }
    }
}

/// Touch lifecycle phase.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TouchPhase {
    /// Finger just touched the screen.
    Started,
    /// Finger is moving on the screen.
    Moved,
    /// Finger lifted from the screen.
    Ended,
    /// Touch was cancelled (e.g., system gesture).
    Cancelled,
}

/// Key event (physical or virtual keyboard).
///
/// Tier: T2-P (σ Sequence)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct KeyEvent {
    /// Key code (platform-agnostic).
    pub code: KeyCode,
    /// Press or release.
    pub state: KeyState,
    /// Modifier keys held.
    pub modifiers: Modifiers,
}

impl KeyEvent {
    /// Create a key event.
    #[must_use]
    pub const fn new(code: KeyCode, state: KeyState, modifiers: Modifiers) -> Self {
        Self {
            code,
            state,
            modifiers,
        }
    }
}

/// Key press/release state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyState {
    Pressed,
    Released,
    Repeat,
}

/// Platform-agnostic key codes (subset — extend as needed).
///
/// Tier: T2-P (Σ Sum — enumeration of key identity)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyCode {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    // Editing
    Enter,
    Space,
    Tab,
    Backspace,
    Delete,
    Escape,
    // Modifiers (also reported as key events)
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    LeftSuper,
    RightSuper,
    /// Unknown/unmapped key.
    Unknown(u32),
}

/// Modifier key state bitmask.
///
/// Tier: T1 (Σ Sum — bitfield composition)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CTRL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);

    /// Check if a modifier is active.
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine two modifier sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Pointer (mouse/trackpad) event.
///
/// Tier: T2-P (λ Location + σ Sequence)
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct PointerEvent {
    /// X position in pixels.
    pub x: f32,
    /// Y position in pixels.
    pub y: f32,
    /// Button pressed/released (None = motion only).
    pub button: Option<PointerButton>,
    /// Button state.
    pub state: Option<KeyState>,
    /// Scroll delta (horizontal, vertical).
    pub scroll: (f32, f32),
}

impl PointerEvent {
    /// Create a pointer event.
    #[must_use]
    pub const fn new(
        x: f32,
        y: f32,
        button: Option<PointerButton>,
        state: Option<KeyState>,
        scroll: (f32, f32),
    ) -> Self {
        Self {
            x,
            y,
            button,
            state,
            scroll,
        }
    }
}

/// Mouse/trackpad button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    Extra(u8),
}

/// Crown/bezel rotation event (smartwatch).
///
/// Tier: T2-P (N Quantity — rotation delta)
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct CrownEvent {
    /// Rotation delta (positive = clockwise).
    pub delta: f32,
    /// Whether crown is being pressed.
    pub pressed: bool,
}

impl CrownEvent {
    /// Create a crown event.
    #[must_use]
    pub const fn new(delta: f32, pressed: bool) -> Self {
        Self { delta, pressed }
    }
}

/// Haptic pulse pattern element.
///
/// Tier: T2-P (ν Frequency + N Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct HapticPulse {
    /// Duration in milliseconds.
    pub duration_ms: u32,
    /// Intensity (0-255).
    pub intensity: u8,
    /// Pause after this pulse in milliseconds.
    pub pause_ms: u32,
}

impl HapticPulse {
    /// Create a simple pulse.
    pub const fn new(duration_ms: u32, intensity: u8, pause_ms: u32) -> Self {
        Self {
            duration_ms,
            intensity,
            pause_ms,
        }
    }

    /// Quick vibration tap.
    pub const fn tap() -> Self {
        Self::new(50, 200, 0)
    }

    /// Short vibration for notifications.
    pub const fn notification() -> Self {
        Self::new(100, 255, 50)
    }
}

/// Device form factor.
///
/// Tier: T2-P (∂ Boundary — defines device constraints)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum FormFactor {
    /// Smartwatch (small round/square display, touch + crown).
    Watch,
    /// Smartphone (medium display, touch primary).
    Phone,
    /// Desktop/laptop (large display, keyboard + pointer).
    Desktop,
}

impl FormFactor {
    /// Typical minimum resolution for this form factor.
    pub const fn min_resolution(&self) -> Resolution {
        match self {
            Self::Watch => Resolution::new(360, 360),
            Self::Phone => Resolution::new(720, 1280),
            Self::Desktop => Resolution::new(1280, 720),
        }
    }

    /// Whether touch is the primary input method.
    pub const fn touch_primary(&self) -> bool {
        match self {
            Self::Watch | Self::Phone => true,
            Self::Desktop => false,
        }
    }
}

/// Power state of the device.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum PowerState {
    /// Running on battery.
    Battery {
        /// Battery percentage (0-100).
        percent: u8,
    },
    /// Charging.
    Charging {
        /// Current battery percentage.
        percent: u8,
    },
    /// Plugged in, fully charged.
    Full,
    /// No battery (desktop with AC power).
    AcPower,
    /// Power state unknown.
    Unknown,
}

impl PowerState {
    /// Get battery percentage if available.
    pub const fn battery_pct(&self) -> Option<u8> {
        match self {
            Self::Battery { percent } | Self::Charging { percent } => Some(*percent),
            Self::Full => Some(100),
            Self::AcPower | Self::Unknown => None,
        }
    }

    /// Whether the device is currently charging.
    pub const fn is_charging(&self) -> bool {
        matches!(self, Self::Charging { .. } | Self::Full)
    }

    /// Whether battery is critically low (< 10%).
    pub const fn is_critical(&self) -> bool {
        match self {
            Self::Battery { percent } => *percent < 10,
            Self::Charging { .. } | Self::Full | Self::AcPower | Self::Unknown => false,
        }
    }
}
