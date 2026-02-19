//! # CTVP Library
//!
//! Clinical Trial Validation Paradigm (CTVP) implementation in Rust.
//!
//! This library provides systematic software validation using pharmaceutical
//! clinical trial methodology, mapping drug development phases to software
//! testing stages.
//!
//! ## Core Principle
//!
//! > "Mock testing is testing theater—it validates that your simulation of
//! > reality works, not that your code works in reality."
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nexcore_ctvp::prelude::*;
//!
//! // Define a capability to validate
//! let capability = Capability::builder()
//!     .id("CAP-001")
//!     .name("User Authentication")
//!     .desired_effect("Users can securely authenticate")
//!     .measurement("auth_success_rate")
//!     .threshold(Threshold::gte(0.999))
//!     .build()
//!     .unwrap();
//!
//! // Create validator with config
//! let config = ValidatorConfig {
//!     deliverable_path: Some("./src".into()),
//!     ..Default::default()
//! };
//! let validator = CapabilityValidator::with_config(capability, config);
//! let results = validator.validate_all().unwrap();
//!
//! // Calculate reality score
//! let score = RealityGradient::calculate(&results);
//! println!("Reality Score: {:.2}", score.value);
//! ```
//!
//! ## Phase Mapping
//!
//! | Phase | Pharmaceutical | Software Equivalent |
//! |-------|---------------|---------------------|
//! | 0 | Preclinical | Unit tests, mocks, property tests |
//! | 1 | Safety | Chaos engineering, fault injection |
//! | 2 | Efficacy | Real data, SLO measurement |
//! | 3 | Confirmation | Shadow/canary deployment |
//! | 4 | Surveillance | Drift detection, continuous validation |

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod capability;
pub mod error;
pub mod evidence;
pub mod five_problems;
pub mod reality_gradient;
pub mod types;
pub mod validation;

#[cfg(feature = "drift")]
pub mod drift;

#[cfg(feature = "testcontainers")]
pub mod containers;

#[cfg(feature = "llm")]
pub mod llm;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::capability::*;
    pub use crate::error::*;
    pub use crate::evidence::*;
    pub use crate::five_problems::*;
    pub use crate::reality_gradient::*;
    pub use crate::types::*;
    pub use crate::validation::*;

    #[cfg(feature = "drift")]
    pub use crate::drift::*;

    #[cfg(feature = "testcontainers")]
    pub use crate::containers::*;

    #[cfg(feature = "llm")]
    pub use crate::llm::*;
}

pub use prelude::*;
