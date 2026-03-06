//! Consultants's Client Care Process — 5-phase PV consulting workflow engine.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]
pub mod assess;
pub mod collect;
pub mod engagement;
pub mod follow_up;
pub mod implement;
pub mod pipeline;
pub mod plan;
