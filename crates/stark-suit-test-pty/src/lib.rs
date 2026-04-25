//! # stark-suit-test-pty
//!
//! Pseudoterminal (PTY) loopback fixture for `SerialBmsSource` testing.
//!
//! Spawns a kernel-managed master/slave PTY pair via `nix::pty::openpty`. Tests
//! write NDJSON to the master end (a `tokio::fs::File`); a `SerialBmsSource`
//! opens the slave path and reads it as if it were a real serial port. The
//! kernel does not distinguish `/dev/pts/N` from `/dev/ttyUSB0` — `tokio-serial`
//! opens both via the same path.
//!
//! Linux + macOS only. `cfg(windows)` users have ConPTY, which is a different
//! API and is explicitly out of scope. BMS hardware does not run on Windows.
//!
//! ## Why pty, not socat
//!
//! - **Pure-Rust**: zero system-binary dependency. Tests are reproducible
//!   across machines without "is socat installed?" preflight.
//! - **Deterministic teardown**: `Drop` closes both fds; the kernel reaps the
//!   pty pair. No leaked entries in `/dev/pts/`.
//! - **Path discovery**: `nix::unistd::ttyname()` returns the slave path.
//!   Hardcoded `/dev/pts/N` paths are unstable across runs.

#![forbid(unsafe_code)]
#![cfg_attr(windows, allow(unused))]

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use unix::TestPty;

#[cfg(windows)]
compile_error!("stark-suit-test-pty is Linux + macOS only — pty is not available on Windows");
