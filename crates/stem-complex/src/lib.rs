//! # stem-complex
//!
//! Complex number arithmetic for mathematical computing.
//!
//! Provides the [`Complex`] number type with:
//! - Full arithmetic (`+`, `-`, `*`, `div`) and geometric operations
//! - Elementary functions: `exp`, `ln`, `pow`, `sqrt`, `sin`, `cos`, `sinh`, `cosh`
//! - The complex Gamma function via the Lanczos approximation (g=7)
//! - Primitive grounding via [`nexcore_lex_primitiva`]
//!
//! # Quick Start
//!
//! ```
//! use stem_complex::prelude::*;
//! use stem_complex::functions;
//! use std::f64::consts::PI;
//!
//! // Euler's identity: exp(i·π) = -1
//! let result = functions::exp(Complex::new(0.0, PI));
//! assert!((result.re + 1.0).abs() < 1e-10);
//! assert!(result.im.abs() < 1e-10);
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod complex;
pub mod error;
pub mod functions;
pub mod gamma;
pub mod grounding;
pub mod traits;

pub use complex::Complex;
pub use error::ComplexError;

/// Commonly used items — import with `use stem_complex::prelude::*`.
pub mod prelude {
    pub use crate::complex::Complex;
    pub use crate::error::ComplexError;
    pub use crate::traits::{AnalyticFunction, ComplexField};
}
