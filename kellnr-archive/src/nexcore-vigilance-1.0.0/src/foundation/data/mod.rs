//! # Data Processing Module
//!
//! YAML, JSON, and TOML parsing with validation.
//!
//! ## Submodules
//!
//! - **yaml** - YAML/TOML parsing and schema validation
//! - **json** - JSON processing utilities

pub mod json;
pub mod mapper;
pub mod yaml;

pub use json::{json_get, json_merge, json_set};
pub use yaml::{ParseResult, ValidationResult, parse_config, parse_toml, parse_yaml};
