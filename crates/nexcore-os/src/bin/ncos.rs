// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! `ncos` — NexCore OS emulator.
//!
//! Boots the OS kernel in actor mode on LinuxPlatform, then runs
//! the interactive REPL (system shell) on stdin/stdout.
//!
//! Integrations:
//! - **Guardian bridge**: reads/writes guardian homeostasis state files
//! - **Brain bridge**: persists OS snapshots as brain artifacts
//!
//! ```bash
//! cargo run -p nexcore-os --bin ncos --features emulator
//! ```

#![forbid(unsafe_code)]

use std::io::{self, BufRead, Write};

use nexcore_os::brain_bridge::BrainBridge;
use nexcore_os::kernel::NexCoreOs;
use nexcore_os::repl::OsRepl;
use nexcore_pal::FormFactor;
use nexcore_pal_linux::LinuxPlatform;

fn main() {
    let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);

    let mut os = match NexCoreOs::boot_with_actors(platform) {
        Ok(os) => os,
        Err(e) => {
            eprintln!("BOOT FAILED: {e}");
            std::process::exit(1);
        }
    };

    let brain = match BrainBridge::new() {
        Ok(b) => {
            let sid: &str = b.session_id();
            eprintln!("[brain] Session created: {sid}");
            Some(b)
        }
        Err(e) => {
            eprintln!("[brain] Not available: {e}");
            None
        }
    };

    print_banner(&mut os, brain.as_ref());
    emit_boot_signal(&os);
    run_repl(&mut os, brain.as_ref());
    on_shutdown(&mut os, brain.as_ref());
}

fn print_banner<P: nexcore_pal::Platform>(os: &mut NexCoreOs<P>, brain: Option<&BrainBridge>) {
    let mut repl = OsRepl::new();

    let guardian_status = os
        .guardian()
        .is_connected()
        .then(|| {
            let g = os.guardian();
            format!("Guardian:     CONNECTED ({})", g.threat_level())
        })
        .unwrap_or_else(|| "Guardian:     DISABLED".to_string());

    let brain_status = brain.map_or_else(
        || "Brain:        NOT AVAILABLE".to_string(),
        |b: &BrainBridge| format!("Brain:        {} (session)", &b.session_id()[..8]),
    );

    let status = repl.eval("status", os);
    println!("╔══════════════════════════════════════╗");
    println!("║        NexCore OS  v0.1.0            ║");
    println!("║        Actor Mode: ACTIVE            ║");
    println!("╚══════════════════════════════════════╝");
    println!();
    println!("{status}");
    println!("{guardian_status}");
    println!("{brain_status}");
    println!();
    println!("Type 'help' for commands, 'exit' to quit.");
    println!("Brain commands: 'brain save', 'brain load', 'brain status'");
    println!();
}

fn emit_boot_signal<P: nexcore_pal::Platform>(os: &NexCoreOs<P>) {
    let guardian = os.guardian();
    guardian.emit_signal(
        "Info",
        "os_boot_complete",
        "nexcore-os",
        serde_json::json!({
            "mode": "actor",
            "actors": 6,
            "services": 11,
        }),
    );
}

fn run_repl<P: nexcore_pal::Platform>(os: &mut NexCoreOs<P>, brain: Option<&BrainBridge>) {
    let mut repl = OsRepl::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("{}", repl.prompt());
        if stdout.flush().is_err() {
            break;
        }

        let mut line = String::new();
        let read_result = stdin.lock().read_line(&mut line);
        match read_result {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if let Some(brain_output) = handle_brain_command(input, brain, os) {
            println!("{brain_output}");
            continue;
        }

        os.tick_actors();

        let output = repl.eval(input, os);
        println!("{output}");

        if output.should_exit {
            break;
        }
    }
}

fn on_shutdown<P: nexcore_pal::Platform>(os: &mut NexCoreOs<P>, brain: Option<&BrainBridge>) {
    if let Some(brain) = brain {
        let snap = os.create_snapshot(false);
        let content = format!(
            "# NexCore OS Final Snapshot\n\n\
             Platform: {}\n\
             State: {}\n\
             Services: {}\n\
             Ticks: {}\n\
             IPC events: {}\n\
             Security: {}\n\
             Timestamp: {}\n",
            snap.platform,
            snap.boot_phase,
            snap.services.len(),
            snap.tick_count,
            snap.ipc_events_emitted,
            snap.security_level,
            nexcore_chrono::DateTime::now().to_rfc3339(),
        );
        if let Err(e) = brain.save_snapshot(&content) {
            eprintln!("[brain] Final snapshot save failed: {e}");
        } else {
            match brain.resolve_snapshot() {
                Ok(v) => eprintln!("[brain] Final snapshot resolved as version {v}"),
                Err(e) => eprintln!("[brain] Resolve failed: {e}"),
            }
        }
    }

    if let Some(guardian) = os.guardian().is_connected().then(|| os.guardian()) {
        guardian.emit_signal(
            "Info",
            "os_shutdown",
            "nexcore-os",
            serde_json::json!({
                "tick_count": os.tick_count(),
            }),
        );
    }

    os.shutdown_actors();
}

fn handle_brain_command<P: nexcore_pal::Platform>(
    input: &str,
    brain: Option<&BrainBridge>,
    os: &NexCoreOs<P>,
) -> Option<String> {
    let parts: Vec<&str> = input.splitn(3, ' ').collect();

    if parts.first().copied() != Some("brain") {
        return None;
    }

    let subcmd = parts.get(1).copied().unwrap_or("status");

    let Some(brain) = brain else {
        return Some("Brain: not available (session creation failed)".to_string());
    };

    match subcmd {
        "save" => {
            let snap = os.create_snapshot(false);
            let content = format!(
                "# NexCore OS Snapshot\n\n\
                 Platform: {}\n\
                 State: {}\n\
                 Services: {}\n\
                 Ticks: {}\n\
                 IPC events: {}\n\
                 Security: {}\n\
                 Timestamp: {}\n",
                snap.platform,
                snap.boot_phase,
                snap.services.len(),
                snap.tick_count,
                snap.ipc_events_emitted,
                snap.security_level,
                nexcore_chrono::DateTime::now().to_rfc3339(),
            );
            match brain.save_snapshot(&content) {
                Ok(()) => Some("Brain: snapshot saved".to_string()),
                Err(e) => Some(format!("Brain: save failed — {e}")),
            }
        }
        "resolve" => match brain.resolve_snapshot() {
            Ok(v) => Some(format!("Brain: snapshot resolved as version {v}")),
            Err(e) => Some(format!("Brain: resolve failed — {e}")),
        },
        "load" => match brain.load_snapshot() {
            Ok(content) => Some(format!("Brain: loaded snapshot\n{content}")),
            Err(e) => Some(format!("Brain: load failed — {e}")),
        },
        "status" | "info" => Some(brain.format_status()),
        _ => Some(
            "Brain commands: brain save | brain resolve | brain load | brain status".to_string(),
        ),
    }
}
