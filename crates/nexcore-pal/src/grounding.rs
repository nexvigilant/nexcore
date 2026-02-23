// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Lex Primitiva Grounding
//!
//! `GroundsTo` implementations for all `nexcore-pal` public types.
//!
//! ## Dominant Primitive Distribution
//!
//! - `FormFactor`, `DisplayShape`, `TouchPhase`, `KeyState`, `PointerButton` —
//!   Classification enums ground to **Comparison** (κ) because they partition a
//!   continuous domain into discrete, mutually exclusive categories.
//! - `PowerState` — **State** (ς) dominant: lifecycle state machine.
//! - `Resolution`, `HapticPulse`, `CrownEvent` — **Quantity** (N) dominant:
//!   fundamentally numeric descriptors.
//! - `PixelFormat` — **Mapping** (μ) dominant: encoding scheme maps pixel
//!   data to a wire format.
//! - `Modifiers` — **Sum** (Σ) dominant: bitfield composition of modifier keys.
//! - `InputEvent`, `TouchEvent`, `KeyEvent`, `PointerEvent` — **Sequence** (σ)
//!   dominant: ordered event streams.
//! - `PalError` and sub-errors — **Boundary** (∂) dominant: error at a
//!   subsystem interface boundary.
//!
//! This module is only compiled when the `grounding` feature is enabled.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::error::{
    DisplayError, HapticsError, InitError, InputError, NetworkError, PalError, PowerError,
    StorageError,
};
use crate::types::{
    CrownEvent, DisplayShape, FormFactor, HapticPulse, InputEvent, KeyCode, KeyEvent, KeyState,
    Modifiers, PixelFormat, PointerButton, PointerEvent, PowerState, Resolution, TouchEvent,
    TouchPhase,
};

// ---------------------------------------------------------------------------
// T1 Pure Primitives
// ---------------------------------------------------------------------------

/// `TouchPhase`: T1, Dominant ς State
///
/// Pure lifecycle state: Started → Moved → Ended | Cancelled.
/// A single state variable with no other primitive in play.
impl GroundsTo for TouchPhase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

/// `KeyState`: T1, Dominant ς State
///
/// Pure lifecycle state: Pressed | Released | Repeat.
impl GroundsTo for KeyState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

// ---------------------------------------------------------------------------
// T2-P Cross-Domain Primitives
// ---------------------------------------------------------------------------

/// `FormFactor`: T2-P (κ Comparison + ∂ Boundary), dominant κ
///
/// Classifies device form into Watch | Phone | Desktop.
/// Comparison-dominant: the purpose is to compare device characteristics
/// (screen size, input modality) and place them in a named category.
/// Boundary is secondary: each form factor defines hard capability limits
/// (min resolution, primary input).
impl GroundsTo for FormFactor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- device class comparison
            LexPrimitiva::Boundary,   // ∂ -- capability constraints per class
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// `DisplayShape`: T2-P (∂ Boundary + κ Comparison), dominant ∂
///
/// Describes the physical boundary of the display region.
/// Boundary-dominant: the shape *is* the boundary definition.
/// Comparison is secondary: shapes are compared to select rendering path.
impl GroundsTo for DisplayShape {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- defines display region boundary
            LexPrimitiva::Comparison, // κ -- rect vs circle vs rounded comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// `PixelFormat`: T2-P (μ Mapping + N Quantity), dominant μ
///
/// Encoding scheme mapping pixel data to a wire representation.
/// Mapping-dominant: the format *maps* colour channels to bytes.
/// Quantity is secondary: bytes_per_pixel() is numeric.
impl GroundsTo for PixelFormat {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ -- colour channels → byte layout
            LexPrimitiva::Quantity, // N -- bytes per pixel count
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// `Resolution`: T2-P (N Quantity + ∂ Boundary), dominant N
///
/// Width × height pixel dimensions.
/// Quantity-dominant: fundamentally two u32 numeric values.
/// Boundary is secondary: resolution defines the framebuffer size boundary.
impl GroundsTo for Resolution {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- width, height pixel counts
            LexPrimitiva::Boundary, // ∂ -- framebuffer size limit
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// `PowerState`: T2-P (ς State + N Quantity), dominant ς
///
/// Battery lifecycle state machine: Battery | Charging | Full | AcPower | Unknown.
/// State-dominant: the variant encodes the current power lifecycle state.
/// Quantity is secondary: percent field is a numeric measurement.
impl GroundsTo for PowerState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- power lifecycle state
            LexPrimitiva::Quantity, // N -- battery percentage value
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `Modifiers`: T2-P (Σ Sum + ∃ Existence), dominant Σ
///
/// Bitfield composition of active modifier keys.
/// Sum-dominant: the value is a bitwise OR (sum) of modifier flags.
/// Existence is secondary: each flag tests whether a modifier exists.
impl GroundsTo for Modifiers {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- bitfield OR composition
            LexPrimitiva::Existence, // ∃ -- modifier present/absent check
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// `HapticPulse`: T2-P (ν Frequency + N Quantity), dominant ν
///
/// A single timed vibration element: duration, intensity, pause.
/// Frequency-dominant: the pulse is a timed, repeatable waveform element.
/// Quantity is secondary: duration_ms, intensity, pause_ms are numeric.
impl GroundsTo for HapticPulse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // ν -- timed pulse pattern
            LexPrimitiva::Quantity,  // N -- duration, intensity, pause values
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

/// `CrownEvent`: T2-P (N Quantity + ς State), dominant N
///
/// Watch crown rotation delta and press state.
/// Quantity-dominant: the delta (f32) is the primary information carrier.
/// State is secondary: pressed is a boolean state.
impl GroundsTo for CrownEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- rotation delta value
            LexPrimitiva::State,    // ς -- pressed/released state
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// `PointerButton`: T2-P (κ Comparison + Σ Sum), dominant κ
///
/// Mouse button identifier: Left | Right | Middle | Extra(u8).
/// Comparison-dominant: partitions buttons into named categories for dispatch.
/// Sum is secondary: Extra(u8) enumerates beyond named variants.
impl GroundsTo for PointerButton {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- button identity classification
            LexPrimitiva::Sum,        // Σ -- enumeration of button variants
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// `KeyCode`: T2-P (Σ Sum + μ Mapping), dominant Σ
///
/// Platform-agnostic key code enumeration.
/// Sum-dominant: large enumeration of all possible key identities.
/// Mapping is secondary: maps from platform scan codes to this agnostic form.
impl GroundsTo for KeyCode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Σ -- enumeration of key identities
            LexPrimitiva::Mapping, // μ -- platform scancode → KeyCode mapping
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// Error types — all ground to Boundary (∂) as subsystem interface violations

/// `PalError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for PalError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- subsystem interface boundary violation
            LexPrimitiva::Sum,      // Σ -- enumeration of subsystem error variants
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `DisplayError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for DisplayError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- display subsystem boundary error
            LexPrimitiva::Sum,      // Σ -- error variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `InputError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for InputError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- input subsystem boundary error
            LexPrimitiva::Sum,      // Σ -- error variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `NetworkError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for NetworkError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- network subsystem boundary error
            LexPrimitiva::Sum,      // Σ -- error variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `StorageError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for StorageError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- storage subsystem boundary error
            LexPrimitiva::Sum,      // Σ -- error variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `HapticsError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for HapticsError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- haptics subsystem boundary error
            LexPrimitiva::Sum,      // Σ -- error variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `PowerError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for PowerError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- power subsystem boundary error
            LexPrimitiva::Sum,      // Σ -- error variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `InitError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
impl GroundsTo for InitError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- platform init boundary error
            LexPrimitiva::Sum,      // Σ -- error variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T2-C Cross-Domain Composites
// ---------------------------------------------------------------------------

/// `TouchEvent`: T2-C (σ Sequence + λ Location + ς State + N Quantity), dominant σ
///
/// An ordered touch event with position, pressure, and lifecycle phase.
/// Sequence-dominant: touch events form a directed stream (Started→Moved→Ended).
/// Location is secondary: (x, y) is a spatial position.
/// State is tertiary: TouchPhase encodes lifecycle state.
/// Quantity is quaternary: pressure and id are numeric.
impl GroundsTo for TouchEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- ordered event in touch stream
            LexPrimitiva::Location, // λ -- (x, y) spatial position
            LexPrimitiva::State,    // ς -- TouchPhase lifecycle
            LexPrimitiva::Quantity, // N -- pressure, id
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// `KeyEvent`: T2-C (σ Sequence + μ Mapping + ς State), dominant σ
///
/// A keyboard event mapping a physical key to a platform-agnostic code.
/// Sequence-dominant: key events form an ordered stream.
/// Mapping is secondary: maps scancode to `KeyCode`.
/// State is tertiary: Pressed | Released | Repeat lifecycle.
impl GroundsTo for KeyEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- ordered key event stream
            LexPrimitiva::Mapping,  // μ -- physical key → KeyCode mapping
            LexPrimitiva::State,    // ς -- pressed/released state
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// `PointerEvent`: T2-C (σ Sequence + λ Location + N Quantity + ∃ Existence), dominant σ
///
/// A mouse/trackpad event with position, optional button, and scroll.
/// Sequence-dominant: pointer events form an ordered stream.
/// Location is secondary: (x, y) spatial position.
/// Quantity is tertiary: scroll delta (f32, f32).
/// Existence is quaternary: button and state are Option fields.
impl GroundsTo for PointerEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- ordered pointer event stream
            LexPrimitiva::Location,  // λ -- (x, y) spatial position
            LexPrimitiva::Quantity,  // N -- scroll delta values
            LexPrimitiva::Existence, // ∃ -- optional button/state fields
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// `InputEvent`: T2-C (σ Sequence + μ Mapping + ∃ Existence + Σ Sum), dominant σ
///
/// The top-level input event discriminant: Touch | Key | Pointer | Crown.
/// Sequence-dominant: all input arrives as an ordered event sequence.
/// Mapping is secondary: maps device-specific events to the unified enum.
/// Existence is tertiary: specific event kind may or may not be present.
/// Sum is quaternary: union of all event variant types.
impl GroundsTo for InputEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- ordered input event stream
            LexPrimitiva::Mapping,   // μ -- device events → InputEvent variants
            LexPrimitiva::Existence, // ∃ -- which event variant is present
            LexPrimitiva::Sum,       // Σ -- union of Touch|Key|Pointer|Crown
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // Tier classification tests

    #[test]
    fn touch_phase_is_t1() {
        assert_eq!(TouchPhase::tier(), Tier::T1Universal);
        assert_eq!(TouchPhase::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn key_state_is_t1() {
        assert_eq!(KeyState::tier(), Tier::T1Universal);
        assert_eq!(KeyState::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn form_factor_is_t2p() {
        assert_eq!(FormFactor::tier(), Tier::T2Primitive);
        assert_eq!(
            FormFactor::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn display_shape_is_t2p() {
        assert_eq!(DisplayShape::tier(), Tier::T2Primitive);
        assert_eq!(
            DisplayShape::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn pixel_format_is_t2p() {
        assert_eq!(PixelFormat::tier(), Tier::T2Primitive);
        assert_eq!(
            PixelFormat::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn resolution_is_t2p() {
        assert_eq!(Resolution::tier(), Tier::T2Primitive);
        assert_eq!(
            Resolution::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn power_state_is_t2p() {
        assert_eq!(PowerState::tier(), Tier::T2Primitive);
        assert_eq!(PowerState::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn haptic_pulse_is_t2p() {
        assert_eq!(HapticPulse::tier(), Tier::T2Primitive);
        assert_eq!(
            HapticPulse::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn touch_event_is_t2c() {
        let tier = TouchEvent::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            TouchEvent::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn key_event_is_t2p_or_t2c() {
        let tier = KeyEvent::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2-P or T2-C, got {tier:?}"
        );
        assert_eq!(KeyEvent::dominant_primitive(), Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn input_event_is_t2c() {
        let tier = InputEvent::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            InputEvent::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn pal_error_is_t2p() {
        assert_eq!(PalError::tier(), Tier::T2Primitive);
        assert_eq!(PalError::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    // All types have a dominant primitive

    #[test]
    fn all_types_have_dominant() {
        assert!(TouchPhase::dominant_primitive().is_some());
        assert!(KeyState::dominant_primitive().is_some());
        assert!(FormFactor::dominant_primitive().is_some());
        assert!(DisplayShape::dominant_primitive().is_some());
        assert!(PixelFormat::dominant_primitive().is_some());
        assert!(Resolution::dominant_primitive().is_some());
        assert!(PowerState::dominant_primitive().is_some());
        assert!(Modifiers::dominant_primitive().is_some());
        assert!(HapticPulse::dominant_primitive().is_some());
        assert!(CrownEvent::dominant_primitive().is_some());
        assert!(TouchEvent::dominant_primitive().is_some());
        assert!(KeyEvent::dominant_primitive().is_some());
        assert!(PointerEvent::dominant_primitive().is_some());
        assert!(InputEvent::dominant_primitive().is_some());
        assert!(PalError::dominant_primitive().is_some());
    }

    // Composition content spot-checks

    #[test]
    fn touch_event_contains_location() {
        let comp = TouchEvent::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    #[test]
    fn input_event_contains_all_four() {
        let comp = InputEvent::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
    }
}
