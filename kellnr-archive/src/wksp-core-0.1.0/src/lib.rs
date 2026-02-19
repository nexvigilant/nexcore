//! wksp-core — Shared foundation for workspace apps and crates

pub mod error;
pub mod config;

pub use error::{ WkspError, Result };
pub use config::WkspConfig;
