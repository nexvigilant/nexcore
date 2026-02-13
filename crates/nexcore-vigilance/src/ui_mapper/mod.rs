//! # CLI-UI Mapper
//!
//! **CLI as the capability contract**: If the CLI can't do it, the UI shouldn't promise it.
//!
//! This crate enforces API-first design by mapping CLI commands to UI components
//! and validating that the UI never exceeds CLI capabilities.
//!
//! ## Core Principle
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    CLI (Source of Truth)                    │
//! │  nexvigilant guardian scan --target=FDA --depth=3 --async   │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Mapping Layer                          │
//! │  Command    → Route/Page                                    │
//! │  Subcommand → Section/Tab                                   │
//! │  --flag     → Toggle/Checkbox                               │
//! │  --arg=val  → Input Field                                   │
//! │  <required> → Required Field (*)                            │
//! │  [optional] → Optional Field                                │
//! │  Output     → Display Component                             │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     UI (Derived)                            │
//! │  /guardian/scan → ScanPage with form fields matching CLI    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Validation Rules (The Contract)
//!
//! | Rule | Description |
//! |------|-------------|
//! | **NO_ORPHAN_UI** | Every UI action MUST map to a CLI command |
//! | **NO_GHOST_FIELDS** | Every form field MUST map to a CLI arg/flag |
//! | **TYPE_PARITY** | UI input types MUST match CLI arg types |
//! | **REQUIRED_MATCH** | UI required fields = CLI required args |
//! | **OUTPUT_PARITY** | UI displays only what CLI outputs |

pub mod error;
pub mod generator;
pub mod mapper;
pub mod parser;
pub mod types;
pub mod validator;

pub use error::*;
pub use generator::*;
pub use mapper::*;
pub use parser::*;
pub use types::*;
pub use validator::*;

/// Re-export commonly used types
pub mod prelude {
    pub use super::error::MapperError;
    pub use super::generator::UiMappingGenerator;
    pub use super::mapper::CliUiMapper;
    pub use super::parser::CliParser;
    pub use super::types::*;
    pub use super::validator::{ValidationRule, Validator};
}
