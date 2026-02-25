//! # NexVigilant Core — stdio-proxy
//!
//! Generic stdio proxy with hot-reload — protocol-agnostic child process
//! management. Provides the reusable T2-P primitives for building protocol-
//! specific hot-reload proxies.
//!
//! ## Architecture
//!
//! ```text
//! Client ←stdio→ StdioProxy<P> ←stdio→ Child Process
//!                      ↑
//!                 BinaryWatcher
//!                 (notify + Debouncer)
//! ```
//!
//! ## Primitive Foundation
//!
//! | Primitive | Module | Manifestation |
//! |-----------|--------|---------------|
//! | T1: State (ς) | proxy | State machine lifecycle |
//! | T1: Sequence (σ) | child, queue, debounce | Line I/O, FIFO, event streams |
//! | T1: Mapping (μ) | protocol | Capture/replay abstraction |
//! | T1: Void (∅) | error | Error as absence of valid state |

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod child;
pub mod debounce;
pub mod error;
pub mod protocol;
pub mod proxy;
pub mod queue;
pub mod watcher;

// Re-exports for ergonomic use
pub use child::{ChildLine, ManagedChild};
pub use error::{ProxyError, Result};
pub use protocol::{NoCapture, ProtocolCapture};
pub use proxy::{ProxyConfig, ProxyState, StdioProxy};
pub use queue::MessageQueue;
pub use watcher::BinaryWatcher;
