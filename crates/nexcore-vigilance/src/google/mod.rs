//! # nexcore Google
//!
//! Google Workspace integrations for nexcore.
//!
//! ## Features
//!
//! - **Apps Script API**: Execute Google Apps Script functions remotely

#![forbid(unsafe_code)]

pub mod script;

pub use script::AppsScriptAPI;
