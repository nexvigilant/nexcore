#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

//! Typed artifact schemas for The Foundry assembly line architecture.
//!
//! Defines all artifacts that flow between stations and across bridges
//! in the dual-pipeline (builder + analyst) assembly line.

pub mod analyst;
pub mod artifact;
pub mod bridge;
pub mod governance;
pub mod report;
pub mod station;
