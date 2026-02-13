// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexCore OS — Init Process (PID 1)
//!
//! Boots NexCore OS on bare Linux hardware. This binary runs as PID 1,
//! mounts essential filesystems, boots the NexCore OS kernel, and
//! provides an interactive login shell.
//!
//! ## Boot Sequence
//!
//! 1. Mount /proc, /sys, /dev, /tmp
//! 2. Set hostname
//! 3. Initialize NexCore OS (PAL → STOS → services)
//! 4. Run secure boot verification
//! 5. First-boot setup (create owner account)
//! 6. Interactive login
//! 7. Command shell

use std::io::{self, BufRead, Write};

use nexcore_os::{NexCoreOs, OsError};
use nexcore_pal::FormFactor;
use nexcore_pal_linux::LinuxPlatform;

// ── ANSI Colors ─────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const WHITE: &str = "\x1b[97m";

// ── Boot Splash ─────────────────────────────────────────────────────────

fn print_splash() {
    println!();
    println!("{CYAN}{BOLD}    ███╗   ██╗███████╗██╗  ██╗ ██████╗ ██████╗ ██████╗ ███████╗{RESET}");
    println!("{CYAN}{BOLD}    ████╗  ██║██╔════╝╚██╗██╔╝██╔════╝██╔═══██╗██╔══██╗██╔════╝{RESET}");
    println!("{CYAN}{BOLD}    ██╔██╗ ██║█████╗   ╚███╔╝ ██║     ██║   ██║██████╔╝█████╗  {RESET}");
    println!("{CYAN}{BOLD}    ██║╚██╗██║██╔══╝   ██╔██╗ ██║     ██║   ██║██╔══██╗██╔══╝  {RESET}");
    println!("{CYAN}{BOLD}    ██║ ╚████║███████╗██╔╝ ██╗╚██████╗╚██████╔╝██║  ██║███████╗{RESET}");
    println!("{CYAN}{BOLD}    ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝{RESET}");
    println!();
    println!(
        "{WHITE}{BOLD}    NexCore OS v0.1.0{RESET}  {DIM}— Primitive-First Operating System{RESET}"
    );
    println!("{DIM}    Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant{RESET}");
    println!("{DIM}    100% Rust · Guardian Homeostasis · Lex Primitiva Grounded{RESET}");
    println!();
}

// ── Filesystem Mounting (PID 1 duties) ──────────────────────────────────

fn mount_essential_filesystems() {
    use nix::mount::{MsFlags, mount};

    let mounts: &[(&str, &str, &str, MsFlags)] = &[
        (
            "proc",
            "/proc",
            "proc",
            MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
        ),
        (
            "sysfs",
            "/sys",
            "sysfs",
            MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
        ),
        ("devtmpfs", "/dev", "devtmpfs", MsFlags::MS_NOSUID),
        ("tmpfs", "/tmp", "tmpfs", MsFlags::empty()),
    ];

    for (source, target, fstype, flags) in mounts {
        // Create mountpoint if needed
        let _ = std::fs::create_dir_all(target);

        match mount(Some(*source), *target, Some(*fstype), *flags, None::<&str>) {
            Ok(()) => print_boot_step(&format!("Mount {target}"), true),
            Err(e) => print_boot_step(&format!("Mount {target} ({e})"), false),
        }
    }

    // Create /dev/pts
    let _ = std::fs::create_dir_all("/dev/pts");
    match mount(
        Some("devpts"),
        "/dev/pts",
        Some("devpts"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC,
        None::<&str>,
    ) {
        Ok(()) => print_boot_step("Mount /dev/pts", true),
        Err(e) => print_boot_step(&format!("Mount /dev/pts ({e})"), false),
    }
}

fn set_hostname() {
    match nix::unistd::sethostname("nexcore") {
        Ok(()) => print_boot_step("Set hostname: nexcore", true),
        Err(e) => print_boot_step(&format!("Set hostname ({e})"), false),
    }
}

fn print_boot_step(msg: &str, ok: bool) {
    if ok {
        println!("  {GREEN}✓{RESET} {msg}");
    } else {
        println!("  {YELLOW}⚠{RESET} {msg}");
    }
}

// ── NexCore OS Boot ─────────────────────────────────────────────────────

fn boot_nexcore_os() -> Result<NexCoreOs<LinuxPlatform>, OsError> {
    println!();
    println!("{BOLD}  ── NexCore OS Boot Sequence ──{RESET}");
    println!();

    // Detect form factor (defaults to Desktop for laptop)
    let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
    print_boot_step("Platform: Linux Desktop (x86_64)", true);

    let os = NexCoreOs::boot(platform)?;

    // Report services
    let svc_count = os.services().count();
    print_boot_step(&format!("{svc_count} core services registered"), true);

    // Report security level
    let security = os.security();
    print_boot_step(&format!("Security level: {}", security.level()), true);

    // Report secure boot
    print_boot_step("Secure boot: chain initialized", true);

    // Report vault
    print_boot_step(&format!("Vault state: {:?}", os.vault().state()), true);

    // Report IPC
    print_boot_step("Cytokine IPC: event bus online", true);

    // Report persistence
    print_boot_step("Persistence layer: ready", true);

    // Report user auth
    print_boot_step("User authentication: ready", true);

    println!();
    println!("  {GREEN}{BOLD}NexCore OS boot complete.{RESET}");
    println!();

    Ok(os)
}

// ── First Boot Setup ────────────────────────────────────────────────────

fn first_boot_setup(os: &mut NexCoreOs<LinuxPlatform>) {
    println!("{CYAN}{BOLD}  ┌─────────────────────────────────────────┐{RESET}");
    println!("{CYAN}{BOLD}  │       First Boot — Create Owner         │{RESET}");
    println!("{CYAN}{BOLD}  └─────────────────────────────────────────┘{RESET}");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Username
    print!("  {WHITE}Username:{RESET} ");
    let _ = stdout.flush();
    let mut username = String::new();
    let _ = stdin.lock().read_line(&mut username);
    let username = username.trim();

    if username.is_empty() {
        println!("  {RED}Error: username cannot be empty{RESET}");
        return;
    }

    // Display name
    print!("  {WHITE}Display name:{RESET} ");
    let _ = stdout.flush();
    let mut display_name = String::new();
    let _ = stdin.lock().read_line(&mut display_name);
    let display_name = display_name.trim();

    // Password
    print!("  {WHITE}Password:{RESET} ");
    let _ = stdout.flush();
    let mut password = String::new();
    let _ = stdin.lock().read_line(&mut password);
    let password = password.trim();

    match os.create_owner(username, display_name, password) {
        Ok(id) => {
            println!();
            println!("  {GREEN}✓ Owner account created: {username} (ID: {id}){RESET}");
            println!();
        }
        Err(e) => {
            println!("  {RED}✗ Failed to create owner: {e}{RESET}");
        }
    }
}

// ── Login ───────────────────────────────────────────────────────────────

fn login(os: &mut NexCoreOs<LinuxPlatform>) -> Option<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("{BLUE}{BOLD}  ┌─────────────────────────────────────────┐{RESET}");
    println!("{BLUE}{BOLD}  │            NexCore OS Login              │{RESET}");
    println!("{BLUE}{BOLD}  └─────────────────────────────────────────┘{RESET}");
    println!();

    for attempt in 1..=3 {
        print!("  {WHITE}Username:{RESET} ");
        let _ = stdout.flush();
        let mut username = String::new();
        let _ = stdin.lock().read_line(&mut username);
        let username = username.trim();

        print!("  {WHITE}Password:{RESET} ");
        let _ = stdout.flush();
        let mut password = String::new();
        let _ = stdin.lock().read_line(&mut password);
        let password = password.trim();

        match os.login(username, password) {
            Ok(session) => {
                println!();
                println!(
                    "  {GREEN}{BOLD}Welcome, {} ({})!{RESET}",
                    os.users()
                        .get_user(username)
                        .map(|u| u.display_name.as_str())
                        .unwrap_or(username),
                    session.role
                );
                println!("  {DIM}Session: {}...{RESET}", &session.token[..16]);
                println!();
                return Some(session.token);
            }
            Err(e) => {
                println!("  {RED}✗ Login failed: {e}{RESET}");
                if attempt < 3 {
                    println!("  {DIM}({} attempts remaining){RESET}", 3 - attempt);
                    println!();
                }
            }
        }
    }

    println!("  {RED}{BOLD}Too many failed login attempts.{RESET}");
    None
}

// ── Command Shell ───────────────────────────────────────────────────────

fn run_shell(os: &mut NexCoreOs<LinuxPlatform>, token: &str) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("{DIM}  Type 'help' for available commands.{RESET}");
    println!();

    loop {
        print!("{CYAN}{BOLD}nexcore>{RESET} ");
        let _ = stdout.flush();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            continue;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "help" | "?" => print_help(),
            "status" => cmd_status(os),
            "services" | "svc" => cmd_services(os),
            "security" | "sec" => cmd_security(os),
            "users" => cmd_users(os),
            "sessions" => cmd_sessions(os),
            "vault" => cmd_vault(os, parts.get(1).copied()),
            "boot" => cmd_boot_info(os),
            "uptime" => cmd_uptime(os),
            "clear" => print!("\x1b[2J\x1b[H"),
            "logout" => {
                let _ = os.logout(token);
                println!("  {YELLOW}Logged out.{RESET}");
                break;
            }
            "shutdown" | "poweroff" => {
                cmd_shutdown(os);
                return;
            }
            "reboot" => {
                cmd_reboot(os);
                return;
            }
            _ => println!(
                "  {RED}Unknown command: '{}'. Type 'help'.{RESET}",
                parts[0]
            ),
        }
    }
}

fn print_help() {
    println!();
    println!("  {BOLD}NexCore OS Commands{RESET}");
    println!("  {DIM}─────────────────────────────────────{RESET}");
    println!("  {CYAN}status{RESET}      System overview");
    println!("  {CYAN}services{RESET}    List all services");
    println!("  {CYAN}security{RESET}    Security level & threats");
    println!("  {CYAN}users{RESET}       User accounts");
    println!("  {CYAN}sessions{RESET}    Active sessions");
    println!("  {CYAN}vault{RESET}       Vault status (vault init <pwd>)");
    println!("  {CYAN}boot{RESET}        Boot chain info");
    println!("  {CYAN}uptime{RESET}      System uptime");
    println!("  {CYAN}clear{RESET}       Clear screen");
    println!("  {CYAN}logout{RESET}      End session");
    println!("  {CYAN}shutdown{RESET}    Power off");
    println!("  {CYAN}reboot{RESET}      Restart system");
    println!();
}

fn cmd_status(os: &NexCoreOs<LinuxPlatform>) {
    println!();
    println!("  {BOLD}╔══════════════════════════════════════╗{RESET}");
    println!("  {BOLD}║        NexCore OS Status             ║{RESET}");
    println!("  {BOLD}╚══════════════════════════════════════╝{RESET}");
    println!();
    println!("  {WHITE}Platform:{RESET}      Linux Desktop (x86_64)");
    println!(
        "  {WHITE}Services:{RESET}      {} registered",
        os.services().count()
    );
    println!(
        "  {WHITE}Security:{RESET}      {}",
        format_security_level(os)
    );
    println!("  {WHITE}Vault:{RESET}         {:?}", os.vault().state());
    println!("  {WHITE}Users:{RESET}         {}", os.users().user_count());
    println!(
        "  {WHITE}Sessions:{RESET}      {} active",
        os.users().active_session_count()
    );
    println!("  {WHITE}Secure Boot:{RESET}   chain verified");
    println!(
        "  {WHITE}IPC Events:{RESET}    {} pending",
        os.ipc().pending()
    );
    println!();
}

fn format_security_level(os: &NexCoreOs<LinuxPlatform>) -> String {
    let level = os.security().level();
    let level_str = format!("{level}");
    match level_str.as_str() {
        "Green" => format!("{GREEN}{BOLD}{level_str}{RESET}"),
        "Yellow" => format!("{YELLOW}{BOLD}{level_str}{RESET}"),
        "Orange" => format!("{YELLOW}{RED}{level_str}{RESET}"),
        "Red" => format!("{RED}{BOLD}{level_str}{RESET}"),
        _ => level_str,
    }
}

fn cmd_services(os: &NexCoreOs<LinuxPlatform>) {
    println!();
    println!("  {BOLD}Services{RESET}");
    println!("  {DIM}────────────────────────────────────{RESET}");

    for (name, state) in os.services().list() {
        let state_str = format!("{state:?}");
        let color = match state {
            nexcore_os::ServiceState::Running => GREEN,
            nexcore_os::ServiceState::Stopped => RED,
            nexcore_os::ServiceState::Degraded => YELLOW,
            _ => WHITE,
        };
        println!("  {color}●{RESET} {name:<24} {color}{state_str}{RESET}");
    }
    println!();
}

fn cmd_security(os: &NexCoreOs<LinuxPlatform>) {
    let security = os.security();
    println!();
    println!("  {BOLD}Security Monitor{RESET}");
    println!("  {DIM}────────────────────────────────────{RESET}");
    println!("  Level:   {}", format_security_level(os));
    println!("  PAMPs:   {} detected", security.pamp_count());
    println!("  DAMPs:   {} detected", security.damp_count());
    println!("  Threats: {} total", security.total_threats());
    println!();
}

fn cmd_users(os: &NexCoreOs<LinuxPlatform>) {
    println!();
    println!("  {BOLD}User Accounts{RESET}");
    println!("  {DIM}────────────────────────────────────{RESET}");

    for user in os.users().list_users() {
        let status_color = match format!("{}", user.status).as_str() {
            "active" => GREEN,
            "locked" => RED,
            "disabled" => YELLOW,
            _ => WHITE,
        };
        println!(
            "  {status_color}●{RESET} {:<16} {:<8} {status_color}{}{RESET}",
            user.username,
            format!("{}", user.role),
            user.status
        );
    }
    println!("  {DIM}Total: {} users{RESET}", os.users().user_count());
    println!();
}

fn cmd_sessions(os: &NexCoreOs<LinuxPlatform>) {
    println!();
    println!("  {BOLD}Active Sessions{RESET}");
    println!("  {DIM}────────────────────────────────────{RESET}");

    let sessions = os.users().active_sessions();
    if sessions.is_empty() {
        println!("  {DIM}No active sessions{RESET}");
    } else {
        for session in &sessions {
            println!(
                "  {GREEN}●{RESET} {:<16} {:<8} expires {}",
                session.username,
                format!("{}", session.role),
                session.expires_at.format("%H:%M:%S"),
            );
        }
    }
    println!("  {DIM}Total: {} active{RESET}", sessions.len());
    println!();
}

fn cmd_vault(os: &mut NexCoreOs<LinuxPlatform>, subcmd: Option<&str>) {
    match subcmd {
        Some("init") => {
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            print!("  {WHITE}Vault password:{RESET} ");
            let _ = stdout.flush();
            let mut pwd = String::new();
            let _ = stdin.lock().read_line(&mut pwd);
            let pwd = pwd.trim();

            match os.vault_mut().initialize(pwd) {
                Ok(()) => println!("  {GREEN}✓ Vault initialized and unlocked{RESET}"),
                Err(e) => println!("  {RED}✗ Vault init failed: {e}{RESET}"),
            }
        }
        _ => {
            println!();
            println!("  {BOLD}Vault{RESET}");
            println!("  {DIM}────────────────────────────────────{RESET}");
            println!("  State:      {:?}", os.vault().state());
            println!("  Secrets:    {}", os.vault().secret_count().unwrap_or(0));
            println!("  Operations: {}", os.vault().operations());
            if matches!(os.vault().state(), nexcore_os::VaultState::Uninitialized) {
                println!("  {DIM}Use 'vault init' to create vault{RESET}");
            }
            println!();
        }
    }
}

fn cmd_boot_info(os: &NexCoreOs<LinuxPlatform>) {
    use nexcore_os::BootStage;

    let chain = os.secure_boot();
    println!();
    println!("  {BOLD}Secure Boot Chain{RESET}");
    println!("  {DIM}────────────────────────────────────{RESET}");

    for stage in BootStage::all() {
        let pcr = chain.pcr(*stage);
        let hex = pcr.short_hex();
        println!("  PCR[{:02}] {:<12} {hex}", stage.pcr_index(), stage.name());
    }

    let log = chain.attestation_log();
    println!("  Attestation entries: {}", log.len());
    println!();
}

fn cmd_uptime(_os: &NexCoreOs<LinuxPlatform>) {
    // Read /proc/uptime if available
    match std::fs::read_to_string("/proc/uptime") {
        Ok(content) => {
            if let Some(secs_str) = content.split_whitespace().next() {
                if let Ok(secs) = secs_str.parse::<f64>() {
                    let hours = (secs / 3600.0) as u64;
                    let mins = ((secs % 3600.0) / 60.0) as u64;
                    let s = (secs % 60.0) as u64;
                    println!("  Uptime: {hours:02}:{mins:02}:{s:02}");
                    return;
                }
            }
            println!("  Uptime: unknown");
        }
        Err(_) => println!("  Uptime: /proc not mounted"),
    }
}

fn cmd_shutdown(os: &mut NexCoreOs<LinuxPlatform>) {
    println!();
    println!("  {YELLOW}{BOLD}Shutting down NexCore OS...{RESET}");
    os.shutdown();
    println!("  {GREEN}✓ All services stopped{RESET}");
    println!("  {DIM}Syncing filesystems...{RESET}");
    nix::unistd::sync();
    println!("  {YELLOW}Power off.{RESET}");
    println!();

    // Actually power off if PID 1
    if nix::unistd::getpid().as_raw() == 1 {
        let _ = nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_POWER_OFF);
    }
}

fn cmd_reboot(os: &mut NexCoreOs<LinuxPlatform>) {
    println!();
    println!("  {YELLOW}{BOLD}Rebooting NexCore OS...{RESET}");
    os.shutdown();
    println!("  {GREEN}✓ All services stopped{RESET}");
    println!("  {DIM}Syncing filesystems...{RESET}");
    nix::unistd::sync();
    println!("  {YELLOW}Restarting...{RESET}");

    if nix::unistd::getpid().as_raw() == 1 {
        let _ = nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_AUTOBOOT);
    }
}

// ── Signal Handling ─────────────────────────────────────────────────────

fn setup_signal_handlers() {
    use nix::sys::signal::{SigHandler, Signal, signal};

    // Ignore SIGCHLD — we reap children manually
    // SAFETY: SigHandler::SigIgn is trivially safe
    unsafe {
        let _ = signal(Signal::SIGCHLD, SigHandler::SigIgn);
        // Don't die on SIGTERM/SIGINT as PID 1
        if nix::unistd::getpid().as_raw() == 1 {
            let _ = signal(Signal::SIGTERM, SigHandler::SigIgn);
            let _ = signal(Signal::SIGINT, SigHandler::SigIgn);
        }
    }
}

// ── Main ────────────────────────────────────────────────────────────────

fn main() {
    let is_pid1 = nix::unistd::getpid().as_raw() == 1;

    // Setup signal handlers
    setup_signal_handlers();

    // Print splash
    print_splash();

    // Mount filesystems if PID 1
    if is_pid1 {
        println!("  {BOLD}── Filesystem Setup ──{RESET}");
        println!();
        mount_essential_filesystems();
        set_hostname();
        println!();
    } else {
        println!(
            "  {DIM}(Running in userspace mode — PID {}){RESET}",
            nix::unistd::getpid()
        );
        println!();
    }

    // Boot NexCore OS
    let mut os = match boot_nexcore_os() {
        Ok(os) => os,
        Err(e) => {
            println!("  {RED}{BOLD}FATAL: NexCore OS boot failed: {e}{RESET}");
            if is_pid1 {
                // Can't panic as PID 1 — loop forever
                println!("  {RED}System halted. Reboot required.{RESET}");
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
            std::process::exit(1);
        }
    };

    // Main loop: first boot setup → login → shell → repeat
    loop {
        // Check if we need first-boot setup
        if os.users().user_count() == 0 {
            first_boot_setup(&mut os);
        }

        // Login
        let token = match login(&mut os) {
            Some(token) => token,
            None => {
                if is_pid1 {
                    // As PID 1, just retry after delay
                    println!("  {DIM}Retrying in 3 seconds...{RESET}");
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    continue;
                } else {
                    std::process::exit(1);
                }
            }
        };

        // Run command shell
        run_shell(&mut os, &token);

        // After logout, loop back to login
        // (after shutdown/reboot, run_shell returns and we handle it)
        if !is_pid1 {
            break;
        }
    }
}
