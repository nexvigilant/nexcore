//! # nexcore Universal Validation
//!
//! L1-L5 validation engine for cross-domain validation.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nexcore_vigilance::validation::prelude::*;
//!
//! // Create a domain validator
//! let domain = GenericDomainValidator::new("skill");
//!
//! // Run validation
//! let validator = UniversalValidator::new(Box::new(domain));
//! let result = validator.validate("./my-skill", ValidationLevel::L3Functional);
//!
//! // Check result
//! if result.overall_status == ValidationStatus::Green {
//!     println!("Validated to L3!");
//! }
//! ```
//!
//! ## The L1-L5 Stack
//!
//! | Level | Question | Timeframe |
//! |-------|----------|-----------|
//! | L1 | Is this internally consistent? | ms-seconds |
//! | L2 | Is this built correctly per spec? | seconds-minutes |
//! | L3 | Does this produce correct outputs? | hours-days |
//! | L4 | Does this work reliably? | days-weeks |
//! | L5 | Does this achieve outcomes? | weeks-months |
//!
//! ## Level Dependency Rule
//!
//! Level N validation is meaningful ONLY IF Level N-1 passes.
//! Always validate bottom-up. Fix lowest failing level first.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod domain;
pub mod engine;
pub mod registry;
pub mod rust_extractor;
pub mod test_taxonomy;
pub mod types;

/// Prelude for convenient imports
pub mod prelude {
    pub use super::domain::{DomainValidator, GenericDomainValidator};
    pub use super::engine::UniversalValidator;
    pub use super::registry::DomainRegistry;
    pub use super::types::{
        AndonSignal, CheckResult, CheckSeverity, LevelResult, ValidationLevel, ValidationResult,
        ValidationStatus,
    };
}

pub use domain::{DomainValidator, GenericDomainValidator};
pub use engine::UniversalValidator;
pub use registry::DomainRegistry;
pub use types::{
    AndonSignal, CheckResult, CheckSeverity, LevelResult, ValidationLevel, ValidationResult,
    ValidationStatus,
};

// Test taxonomy and Rust extractor
pub use rust_extractor::{
    ExtractorError, classify_tests, extract_from_directory, extract_from_file,
};
pub use test_taxonomy::{
    CategoryCounts, ClassificationPatterns, ClassifiedTest, CoverageMetrics, TestCategory,
    TestClassification, build_classification,
};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
