// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # WebMCP Config Registry
//!
//! NexVigilant's agent routing infrastructure. Stores, serves, and syncs
//! browser automation configs for the MoltBrowser Hub.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod config;
