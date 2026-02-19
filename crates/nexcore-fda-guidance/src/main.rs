#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

//! FDA Guidance CLI — search, get, and refresh FDA guidance documents.
//! Build with: cargo build -p nexcore-fda-guidance --features cli

fn main() {
    eprintln!("Build with --features cli to enable the FDA guidance CLI.");
}
