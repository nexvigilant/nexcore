//! # Vault Prelude
//!
//! Convenience re-exports for the most common vault types.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nexcore_vault::prelude::*;
//!
//! // Open or create a vault
//! let config = VaultConfig::default();
//! // let mut vault = Vault::create(config, "password").unwrap();
//! ```

// Core vault type
pub use crate::store::Vault;

// Types
pub use crate::types::PlaintextExport;
pub use crate::types::Salt;
pub use crate::types::SecretName;
pub use crate::types::VaultEntry;
pub use crate::types::VaultFile;

// Configuration
pub use crate::config::VaultConfig;

// Error handling
pub use crate::error::Result;
pub use crate::error::VaultError;
