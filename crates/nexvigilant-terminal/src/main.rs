// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! NexVigilant Terminal — Tauri desktop application entry point.
//!
//! Wires four nexcore subsystems behind Tauri IPC:
//! - nexcore-terminal PTY (real POSIX PTY with async I/O)
//! - nexcore-terminal sessions (lifecycle, modes, metadata)
//! - nexcloud (CloudSupervisor, agent processes, health)
//! - nexcore-shell (app launcher, login, AI partner)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use nexvigilant_terminal::commands::pty::PtyState;
use nexvigilant_terminal::commands::terminal::TerminalState;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nexvigilant_terminal=debug".parse().unwrap_or_default()),
        )
        .init();

    tracing::info!("Starting NexVigilant Terminal");

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(PtyState::new())
        .manage(TerminalState::new())
        .invoke_handler(tauri::generate_handler![
            // PTY process management (real POSIX PTY)
            nexvigilant_terminal::commands::pty::pty_spawn,
            nexvigilant_terminal::commands::pty::pty_write,
            nexvigilant_terminal::commands::pty::pty_resize,
            nexvigilant_terminal::commands::pty::pty_kill,
            nexvigilant_terminal::commands::pty::pty_reconnect,
            // Cloud observability
            nexvigilant_terminal::commands::cloud::cloud_list_services,
            nexvigilant_terminal::commands::cloud::cloud_overview,
            nexvigilant_terminal::commands::cloud::cloud_events,
            // Terminal sessions (metadata/lifecycle)
            nexvigilant_terminal::commands::terminal::terminal_create_session,
            nexvigilant_terminal::commands::terminal::terminal_list_sessions,
            nexvigilant_terminal::commands::terminal::terminal_send_input,
            nexvigilant_terminal::commands::terminal::terminal_switch_mode,
            nexvigilant_terminal::commands::terminal::terminal_resize,
            nexvigilant_terminal::commands::terminal::terminal_get_session,
            // Shell / app launcher
            nexvigilant_terminal::commands::shell::shell_status,
            nexvigilant_terminal::commands::shell::shell_list_apps,
            nexvigilant_terminal::commands::shell::shell_launch_app,
            nexvigilant_terminal::commands::shell::shell_command_palette,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!("NexVigilant Terminal failed: {e}");
        std::process::exit(1);
    }
}
