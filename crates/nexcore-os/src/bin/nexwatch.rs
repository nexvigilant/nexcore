// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! `nexwatch` — NexCore OS for Galaxy Watch 7 (SM-L315U).
//!
//! Boots the full OS kernel on real hardware via the Linux PAL.
//! Target: Samsung Exynos W1000 (s5e5535), 480x480 AMOLED, LTE.
//!
//! Cross-compile:
//! ```bash
//! cargo build -p nexcore-os --bin nexwatch --target aarch64-unknown-linux-musl --release
//! ```

#![forbid(unsafe_code)]

use nexcore_os::kernel::NexCoreOs;
use nexcore_pal::FormFactor;
use nexcore_pal_linux::LinuxPlatform;

fn main() {
    // Boot on real watch hardware — probes DRM, evdev, ALSA, sysfs
    let platform = LinuxPlatform::new(FormFactor::Watch, "/data/nexcore");

    eprintln!("[nexwatch] Booting NexCore OS on Galaxy Watch 7...");
    eprintln!("[nexwatch] Display: 480x480 AMOLED @ /dev/dri/card0");
    eprintln!("[nexwatch] Touch: Zinitix ZTM730 @ /dev/input/event3");
    eprintln!("[nexwatch] Audio: AUD9002X @ /dev/snd/pcmC0D0p");

    let mut os = match NexCoreOs::boot_with_actors(platform) {
        Ok(os) => {
            eprintln!("[nexwatch] Boot complete — NexCore OS running");
            os
        }
        Err(e) => {
            eprintln!("[nexwatch] BOOT FAILED: {e}");
            std::process::exit(1);
        }
    };

    // Run the OS event loop
    loop {
        if !os.tick() {
            break;
        }
    }

    eprintln!("[nexwatch] Shutdown");
}
