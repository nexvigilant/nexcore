//! Build script for nexcore-watch-app.
//!
//! Compiles Slint markup files (`.slint`) into Rust types at build time.
//! The generated `NexVigilantWatch` struct becomes the root UI component.
//!
//! ## Primitive: μ (Mapping) — .slint → Rust types
//! ## Tier: T3

fn main() {
    // Only compile Slint files when targeting Android.
    // On host, the android_main and all Slint types are gated behind
    // #[cfg(target_os = "android")], so we skip compilation entirely.
    #[cfg(feature = "slint-ui")]
    {
        slint_build::compile_with_config(
            "../ui/app.slint",
            slint_build::CompilerConfiguration::new().with_style("fluent-dark".into()),
        )
        .ok();
    }
}
