// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! NexVigilant Terminal — Tauri desktop application entry point.
//!
//! Wires three nexcore subsystems behind Tauri IPC:
//! - nexcore-terminal (session management, routing, PTY)
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

use nexvigilant_terminal::commands::cloud::*;
use nexvigilant_terminal::commands::shell::*;
use nexvigilant_terminal::commands::terminal::*;

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
        .invoke_handler(tauri::generate_handler![
            // Cloud observability
            cloud_list_services,
            cloud_overview,
            cloud_events,
            // Terminal sessions
            terminal_create_session,
            terminal_list_sessions,
            terminal_send_input,
            terminal_switch_mode,
            terminal_resize,
            terminal_get_session,
            // Shell / app launcher
            shell_status,
            shell_list_apps,
            shell_launch_app,
            shell_command_palette,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!("NexVigilant Terminal failed: {e}");
        std::process::exit(1);
    }
}
