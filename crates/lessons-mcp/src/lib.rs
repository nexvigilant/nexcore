//! Lessons Learned MCP Server
//! Tier: T2-C (composes storage, extraction, protocol handling)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod extract;
pub mod models;
pub mod protocol;
pub mod storage;
pub mod tools;
