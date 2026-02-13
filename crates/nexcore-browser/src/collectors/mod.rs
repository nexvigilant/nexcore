//! Event collectors for browser automation
//!
//! Provides FIFO-bounded collectors for console and network events.
//! Collectors are thread-safe and designed for concurrent access.

pub mod console;
pub mod network;
