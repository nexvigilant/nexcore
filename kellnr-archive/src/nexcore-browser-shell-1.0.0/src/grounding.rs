//! # GroundsTo implementations for nexcore-browser-shell types
//!
//! Connects browser shell types to the Lex Primitiva type system.
//!
//! ## σ (Sequence) Focus
//!
//! The browser shell is a pipeline: user input → IPC → browser → render → display.
//! TabInfo sequences pages, BrowserShellState sequences operations,
//! ShellError classifies failure boundaries.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::ShellError;
use crate::models::TabInfo;
use crate::state::BrowserShellState;

// ---------------------------------------------------------------------------
// Models — UI data transfer types
// ---------------------------------------------------------------------------

/// TabInfo: T2-C (λ · ∃ · ς · σ), dominant λ
///
/// A browser tab's display information: URL, title, loading state.
/// Location-dominant: a tab IS a location (URL) with associated metadata.
impl GroundsTo for TabInfo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,  // λ — URL location
            LexPrimitiva::Existence, // ∃ — optional title/favicon
            LexPrimitiva::State,     // ς — loading state
            LexPrimitiva::Sequence,  // σ — tab ordering
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// ShellError: T2-P (∂ · Σ), dominant ∂
///
/// Error variants: Browser | State | Ipc.
/// Boundary-dominant: errors represent boundary violations.
impl GroundsTo for ShellError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — error boundary
            LexPrimitiva::Sum,      // Σ — three-variant enum
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// State — application state wrapper
// ---------------------------------------------------------------------------

/// BrowserShellState: T2-C (ς · σ · λ · μ), dominant ς
///
/// The shell's application state wrapping nexcore-browser operations.
/// State-dominant: it IS the mutable application state container.
impl GroundsTo for BrowserShellState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς — application state
            LexPrimitiva::Sequence, // σ — operation sequencing
            LexPrimitiva::Location, // λ — URL navigation
            LexPrimitiva::Mapping,  // μ — IPC command dispatch
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn tab_info_is_location_dominant() {
        let comp = TabInfo::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert_eq!(TabInfo::tier(), Tier::T2Composite);
    }

    #[test]
    fn shell_error_is_boundary_dominant() {
        let comp = ShellError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(ShellError::tier(), Tier::T2Primitive);
    }

    #[test]
    fn browser_shell_state_is_state_dominant() {
        let comp = BrowserShellState::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(BrowserShellState::tier(), Tier::T2Composite);
    }

    #[test]
    fn shell_has_ipc_mapping() {
        let comp = BrowserShellState::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
    }
}
