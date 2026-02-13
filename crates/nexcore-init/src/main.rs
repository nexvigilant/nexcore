// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexCore Init — PID 1
//!
//! The init process for NexCore OS. When running as PID 1 on a Linux
//! system, this binary boots the OS kernel, starts system services,
//! launches the shell, and runs the main event loop.
//!
//! ## Boot Sequence
//!
//! ```text
//! nexcore-init
//!   ├── 1. Init logging (tracing)
//!   ├── 2. Detect form factor / parse args
//!   ├── 3. Create Linux platform (PAL)
//!   ├── 4. Boot NexCore OS kernel
//!   │     ├── PAL init
//!   │     ├── STOS kernel boot
//!   │     ├── Service registration
//!   │     └── Shell launch phase
//!   ├── 5. Create compositor + shell
//!   ├── 6. Boot shell (register system apps)
//!   └── 7. Main event loop (tick OS + shell)
//! ```
//!
//! ## Usage
//!
//! ```text
//! nexcore-init [--form-factor watch|phone|desktop] [--virtual] [--ticks N]
//! ```
//!
//! - `--form-factor`: Override detected form factor (default: desktop)
//! - `--virtual`: Use virtual platform (no real hardware access)
//! - `--ticks`: Run for N ticks then exit (for testing; 0 = infinite)

#![forbid(unsafe_code)]

use nexcore_os::NexCoreOs;
use nexcore_pal::{FormFactor, Platform};
use nexcore_pal_linux::LinuxPlatform;
use nexcore_shell::{AppId, Shell};

/// Parse form factor from string argument.
fn parse_form_factor(s: &str) -> Option<FormFactor> {
    match s.to_lowercase().as_str() {
        "watch" => Some(FormFactor::Watch),
        "phone" => Some(FormFactor::Phone),
        "desktop" => Some(FormFactor::Desktop),
        _ => None,
    }
}

/// Minimal argument parser (no clap dependency).
struct Args {
    form_factor: FormFactor,
    virtual_mode: bool,
    max_ticks: u64,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut form_factor = FormFactor::Desktop;
        let mut virtual_mode = false;
        let mut max_ticks: u64 = 0;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--form-factor" | "-f" => {
                    i += 1;
                    if i < args.len() {
                        if let Some(ff) = parse_form_factor(&args[i]) {
                            form_factor = ff;
                        } else {
                            eprintln!("Unknown form factor: {}. Using desktop.", args[i]);
                        }
                    }
                }
                "--virtual" | "-v" => {
                    virtual_mode = true;
                }
                "--ticks" | "-t" => {
                    i += 1;
                    if i < args.len() {
                        max_ticks = args[i].parse().unwrap_or(0);
                    }
                }
                "--help" | "-h" => {
                    println!("NexCore Init — PID 1 for NexCore OS");
                    println!();
                    println!("Usage: nexcore-init [OPTIONS]");
                    println!();
                    println!("Options:");
                    println!(
                        "  -f, --form-factor <FACTOR>  watch, phone, or desktop (default: desktop)"
                    );
                    println!("  -v, --virtual               Use virtual platform (no hardware)");
                    println!(
                        "  -t, --ticks <N>             Run for N ticks then exit (0 = infinite)"
                    );
                    println!("  -h, --help                  Show this help");
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {}", args[i]);
                }
            }
            i += 1;
        }

        Self {
            form_factor,
            virtual_mode,
            max_ticks,
        }
    }
}

fn main() {
    // 1. Init logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();

    let pid = std::process::id();
    tracing::info!("NexCore Init starting (PID {pid})");

    // 2. Parse arguments
    let args = Args::parse();
    tracing::info!(
        "Form factor: {:?}, virtual: {}, max_ticks: {}",
        args.form_factor,
        args.virtual_mode,
        args.max_ticks
    );

    // 3. Create platform
    let platform = if args.virtual_mode {
        LinuxPlatform::virtual_platform(args.form_factor)
    } else {
        LinuxPlatform::new(args.form_factor, "/var/lib/nexcore")
    };
    tracing::info!("Platform: {}", platform.name());

    // 4. Boot NexCore OS kernel
    tracing::info!("Booting NexCore OS...");
    let mut os = match NexCoreOs::boot(platform) {
        Ok(os) => {
            tracing::info!("NexCore OS booted successfully");
            os
        }
        Err(e) => {
            tracing::error!("Boot failed: {e}");
            std::process::exit(1);
        }
    };

    // 5. Create shell
    tracing::info!("Initializing shell...");
    let mut shell = Shell::new(args.form_factor);

    // 6. Boot shell
    shell.boot();
    tracing::info!(
        "Shell booted: {:?} mode, {} regions, {} apps",
        shell.form_factor(),
        shell.layout().regions.len(),
        shell.apps().count(),
    );

    // Launch the built-in launcher app
    let launcher_id = AppId::new("launcher");
    if shell.launch_app(&launcher_id) {
        tracing::info!("Launcher app started");
    }

    // 7. Main event loop
    tracing::info!("Entering main event loop...");
    let mut ticks: u64 = 0;

    loop {
        // Tick the OS kernel (process state machines, input, security)
        if !os.tick() {
            tracing::info!("OS kernel signaled shutdown");
            break;
        }

        // Tick the shell (composite frame)
        shell.tick();

        ticks += 1;

        // Bounded tick mode (for testing)
        if args.max_ticks > 0 && ticks >= args.max_ticks {
            tracing::info!("Reached max ticks ({ticks}), shutting down");
            break;
        }
    }

    // Graceful shutdown
    tracing::info!("Shutting down NexCore OS...");
    shell.shutdown();
    os.shutdown();

    tracing::info!(
        "NexCore OS halted after {ticks} ticks, {} frames rendered",
        shell.frame_count()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_form_factors() {
        assert_eq!(parse_form_factor("watch"), Some(FormFactor::Watch));
        assert_eq!(parse_form_factor("phone"), Some(FormFactor::Phone));
        assert_eq!(parse_form_factor("desktop"), Some(FormFactor::Desktop));
        assert_eq!(parse_form_factor("WATCH"), Some(FormFactor::Watch));
        assert_eq!(parse_form_factor("invalid"), None);
    }

    #[test]
    fn full_boot_and_tick_virtual() {
        // End-to-end: boot OS + shell, tick 10 times, shutdown
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let boot_result = NexCoreOs::boot(platform);
        assert!(boot_result.is_ok(), "OS boot must succeed");

        if let Ok(mut os) = boot_result {
            let mut shell = Shell::new(FormFactor::Desktop);
            shell.boot();

            let launcher_id = AppId::new("launcher");
            assert!(shell.launch_app(&launcher_id));

            // Run 10 ticks
            for _ in 0..10 {
                assert!(os.tick());
                shell.tick();
            }

            assert_eq!(shell.frame_count(), 10);

            shell.shutdown();
            os.shutdown();
        }
    }

    #[test]
    fn watch_mode_boot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Watch);
        let boot_result = NexCoreOs::boot(platform);
        assert!(boot_result.is_ok(), "Watch boot must succeed");

        if let Ok(mut os) = boot_result {
            let mut shell = Shell::new(FormFactor::Watch);
            shell.boot();
            assert_eq!(shell.layout().width, 450);

            for _ in 0..5 {
                os.tick();
                shell.tick();
            }

            shell.shutdown();
            os.shutdown();
        }
    }
}
