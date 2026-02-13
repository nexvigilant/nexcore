// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! NexCore OS REPL — the system shell for inspecting and controlling the OS.
//!
//! Tier: T3 (σ + μ + Σ + ς + ρ + ∂ + π)
//!
//! The REPL provides a command-line interface to all 11 OS subsystems:
//! STOS, services, IPC, security, clearance, vault, network, audio,
//! persistence, secure boot, and users.
//!
//! ## Design
//!
//! - **Parse** is pure: `parse(input) → ReplCommand` (no side effects)
//! - **Execute** is generic: `execute(cmd, os) → ReplOutput` (works with any Platform)
//! - **History** tracks all commands with sequence numbers (π Persistence)
//!
//! ## Primitive Grounding
//!
//! | Component     | Primitives    | Role                        |
//! |---------------|---------------|-----------------------------|
//! | Parse         | μ + Σ         | String → Command mapping    |
//! | Execute       | μ + ς + ∂     | Command → OS state query    |
//! | History       | σ + π         | Ordered persistent record   |
//! | Dispatch      | Σ + ρ         | Sum-type match + help recur |
//! | Validation    | ∂ + κ         | Input boundary checking     |

use crate::kernel::{NexCoreOs, OsState};
use nexcore_pal::Platform;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// REPL COMMAND (Σ Sum — all possible commands)
// ═══════════════════════════════════════════════════════════════

/// A parsed REPL command.
///
/// Tier: T2-C (Σ Sum + μ Mapping — command variants with arguments)
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    // ── System ────────────────────────────────────────────────
    /// Show OS status (state, tick count, form factor, services).
    Status,
    /// List all registered services with their states.
    Services,
    /// Show detail for a specific service.
    ServiceDetail(String),
    /// Show boot sequence log.
    BootLog,
    /// Initiate graceful shutdown.
    Shutdown,
    /// Show available commands.
    Help,
    /// Exit the REPL.
    Exit,

    // ── Security (Guardian) ──────────────────────────────────
    /// Show security level, PAMP/DAMP counts.
    Security,
    /// Report a threat (severity + description).
    Threat {
        severity: String,
        description: String,
    },
    /// List quarantined services.
    Quarantine,

    // ── Network ──────────────────────────────────────────────
    /// Show network state summary.
    Network,
    /// List registered interfaces.
    Interfaces,
    /// DNS cache lookup.
    Dns(String),
    /// Show firewall rules.
    Firewall,

    // ── Audio ────────────────────────────────────────────────
    /// Show audio state summary.
    Audio,
    /// List audio devices.
    AudioDevices,
    /// Get or set master volume (None = get, Some = set).
    Volume(Option<f32>),
    /// Toggle system mute.
    Mute,

    // ── Vault ────────────────────────────────────────────────
    /// Show vault state.
    Vault,
    /// List stored secrets (metadata only).
    Secrets,

    // ── Users ────────────────────────────────────────────────
    /// List all users.
    Users,
    /// Authenticate a user.
    Login { username: String, password: String },

    // ── Boot Chain ───────────────────────────────────────────
    /// Show secure boot verification status.
    BootChain,
    /// Show PCR register values.
    Pcr,

    // ── Persistence ──────────────────────────────────────────
    /// Create a state snapshot.
    Snapshot,

    // ── History ──────────────────────────────────────────────
    /// Show command history.
    History,

    // ── Meta ─────────────────────────────────────────────────
    /// Unrecognized command.
    Unknown(String),
    /// Empty input.
    Empty,
}

// ═══════════════════════════════════════════════════════════════
// REPL OUTPUT
// ═══════════════════════════════════════════════════════════════

/// Output from a REPL command execution.
///
/// Tier: T2-P (μ Mapping — command result)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplOutput {
    /// The formatted output text.
    pub text: String,
    /// Whether the REPL should exit after this output.
    pub should_exit: bool,
    /// Whether the command succeeded.
    pub success: bool,
}

impl ReplOutput {
    /// Create a successful output.
    fn ok(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            should_exit: false,
            success: true,
        }
    }

    /// Create an error output.
    fn err(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            should_exit: false,
            success: false,
        }
    }

    /// Create an exit signal.
    fn exit(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            should_exit: true,
            success: true,
        }
    }
}

impl std::fmt::Display for ReplOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

// ═══════════════════════════════════════════════════════════════
// HISTORY
// ═══════════════════════════════════════════════════════════════

/// A single history entry.
///
/// Tier: T2-P (σ Sequence + π Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Sequence number (monotonically increasing).
    pub seq: u64,
    /// The raw command string.
    pub command: String,
    /// Whether the command succeeded.
    pub success: bool,
}

// ═══════════════════════════════════════════════════════════════
// OS REPL
// ═══════════════════════════════════════════════════════════════

/// The NexCore OS REPL (system shell).
///
/// Tier: T3 (σ + μ + Σ + ς + ρ + ∂ + π)
pub struct OsRepl {
    /// Command history.
    history: Vec<HistoryEntry>,
    /// Next sequence number.
    next_seq: u64,
    /// Maximum history size.
    max_history: usize,
}

impl Default for OsRepl {
    fn default() -> Self {
        Self::new()
    }
}

impl OsRepl {
    /// Create a new OS REPL.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            next_seq: 1,
            max_history: 500,
        }
    }

    /// The REPL prompt string.
    pub fn prompt(&self) -> &'static str {
        "nexcore> "
    }

    /// Parse a command string into a structured command.
    ///
    /// Pure function — no side effects.
    pub fn parse(&self, input: &str) -> ReplCommand {
        parse_command(input)
    }

    /// Execute a command against the OS kernel.
    ///
    /// Records the command in history and returns formatted output.
    pub fn execute<P: Platform>(&mut self, cmd: &ReplCommand, os: &mut NexCoreOs<P>) -> ReplOutput {
        let output = execute_command(cmd, os);

        // Record in history (skip Empty)
        if *cmd != ReplCommand::Empty {
            let entry = HistoryEntry {
                seq: self.next_seq,
                command: format!("{cmd:?}"),
                success: output.success,
            };
            self.next_seq += 1;
            self.history.push(entry);

            // Trim history if over max
            if self.history.len() > self.max_history {
                let drain_count = self.history.len() - self.max_history;
                self.history.drain(..drain_count);
            }
        }

        output
    }

    /// Shortcut: parse + execute in one call.
    pub fn eval<P: Platform>(&mut self, input: &str, os: &mut NexCoreOs<P>) -> ReplOutput {
        let cmd = self.parse(input);
        self.execute(&cmd, os)
    }

    /// Get command history.
    pub fn history(&self) -> &[HistoryEntry] {
        &self.history
    }

    /// Get the last N history entries.
    pub fn last_n(&self, n: usize) -> &[HistoryEntry] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }

    /// Total commands executed.
    pub fn command_count(&self) -> u64 {
        self.next_seq - 1
    }
}

// ═══════════════════════════════════════════════════════════════
// PARSE (pure function — μ Mapping)
// ═══════════════════════════════════════════════════════════════

/// Parse an input string into a `ReplCommand`.
///
/// Tier: T1 (μ Mapping — string → command)
fn parse_command(input: &str) -> ReplCommand {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ReplCommand::Empty;
    }

    let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        // System
        "status" => ReplCommand::Status,
        "services" | "svc" => ReplCommand::Services,
        "service" => {
            if parts.len() > 1 {
                ReplCommand::ServiceDetail(parts[1].to_string())
            } else {
                ReplCommand::Services
            }
        }
        "bootlog" | "boot" => ReplCommand::BootLog,
        "shutdown" => ReplCommand::Shutdown,
        "help" | "?" | "h" => ReplCommand::Help,
        "exit" | "quit" | "q" => ReplCommand::Exit,

        // Security
        "security" | "sec" => ReplCommand::Security,
        "threat" => {
            if parts.len() >= 3 {
                ReplCommand::Threat {
                    severity: parts[1].to_string(),
                    description: parts[2].to_string(),
                }
            } else {
                ReplCommand::Unknown("threat <severity> <description>".to_string())
            }
        }
        "quarantine" | "quar" => ReplCommand::Quarantine,

        // Network
        "network" | "net" => ReplCommand::Network,
        "interfaces" | "iface" | "ifconfig" => ReplCommand::Interfaces,
        "dns" => {
            if parts.len() > 1 {
                ReplCommand::Dns(parts[1].to_string())
            } else {
                ReplCommand::Unknown("dns <hostname>".to_string())
            }
        }
        "firewall" | "fw" => ReplCommand::Firewall,

        // Audio
        "audio" => ReplCommand::Audio,
        "devices" | "audiodevices" => ReplCommand::AudioDevices,
        "volume" | "vol" => {
            if parts.len() > 1 {
                parts[1].parse::<f32>().map_or_else(
                    |_| ReplCommand::Unknown(format!("Invalid volume: {}", parts[1])),
                    |v| ReplCommand::Volume(Some(v)),
                )
            } else {
                ReplCommand::Volume(None)
            }
        }
        "mute" => ReplCommand::Mute,

        // Vault
        "vault" => ReplCommand::Vault,
        "secrets" => ReplCommand::Secrets,

        // Users
        "users" | "who" => ReplCommand::Users,
        "login" => {
            if parts.len() >= 3 {
                ReplCommand::Login {
                    username: parts[1].to_string(),
                    password: parts[2].to_string(),
                }
            } else {
                ReplCommand::Unknown("login <username> <password>".to_string())
            }
        }

        // Boot Chain
        "bootchain" | "chain" => ReplCommand::BootChain,
        "pcr" => ReplCommand::Pcr,

        // Persistence
        "snapshot" | "snap" => ReplCommand::Snapshot,

        // History
        "history" | "hist" => ReplCommand::History,

        // Unknown
        other => ReplCommand::Unknown(other.to_string()),
    }
}

// ═══════════════════════════════════════════════════════════════
// EXECUTE (generic over Platform — ς State inspection)
// ═══════════════════════════════════════════════════════════════

/// Execute a command against the OS kernel.
///
/// Tier: T2-C (μ + ς + ∂ — dispatch + state query + validation)
fn execute_command<P: Platform>(cmd: &ReplCommand, os: &mut NexCoreOs<P>) -> ReplOutput {
    match cmd {
        // ── System ────────────────────────────────────────────
        ReplCommand::Status => exec_status(os),
        ReplCommand::Services => exec_services(os),
        ReplCommand::ServiceDetail(name) => exec_service_detail(os, name),
        ReplCommand::BootLog => exec_boot_log(os),
        ReplCommand::Shutdown => exec_shutdown(os),
        ReplCommand::Help => exec_help(),
        ReplCommand::Exit => ReplOutput::exit("Goodbye."),

        // ── Security ──────────────────────────────────────────
        ReplCommand::Security => exec_security(os),
        ReplCommand::Threat {
            severity,
            description,
        } => exec_threat(os, severity, description),
        ReplCommand::Quarantine => exec_quarantine(os),

        // ── Network ──────────────────────────────────────────
        ReplCommand::Network => exec_network(os),
        ReplCommand::Interfaces => exec_interfaces(os),
        ReplCommand::Dns(hostname) => exec_dns(os, hostname),
        ReplCommand::Firewall => exec_firewall(os),

        // ── Audio ────────────────────────────────────────────
        ReplCommand::Audio => exec_audio(os),
        ReplCommand::AudioDevices => exec_audio_devices(os),
        ReplCommand::Volume(level) => exec_volume(os, *level),
        ReplCommand::Mute => exec_mute(os),

        // ── Vault ────────────────────────────────────────────
        ReplCommand::Vault => exec_vault(os),
        ReplCommand::Secrets => exec_secrets(os),

        // ── Users ────────────────────────────────────────────
        ReplCommand::Users => exec_users(os),
        ReplCommand::Login { username, password } => exec_login(os, username, password),

        // ── Boot Chain ───────────────────────────────────────
        ReplCommand::BootChain => exec_bootchain(os),
        ReplCommand::Pcr => exec_pcr(os),

        // ── Persistence ──────────────────────────────────────
        ReplCommand::Snapshot => exec_snapshot(os),

        // ── History (handled by OsRepl, not kernel) ──────────
        ReplCommand::History => ReplOutput::ok("[history displayed by REPL controller]"),

        // ── Meta ─────────────────────────────────────────────
        ReplCommand::Unknown(cmd) => ReplOutput::err(format!(
            "Unknown command: {cmd}\nType 'help' for available commands."
        )),
        ReplCommand::Empty => ReplOutput::ok(String::new()),
    }
}

// ═══════════════════════════════════════════════════════════════
// COMMAND HANDLERS
// ═══════════════════════════════════════════════════════════════

fn exec_status<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let state = match os.state() {
        OsState::Booting => "BOOTING",
        OsState::Running => "RUNNING",
        OsState::ShuttingDown => "SHUTTING DOWN",
        OsState::Halted => "HALTED",
    };

    let services = os.services();
    let running = services.count_in_state(crate::service::ServiceState::Running);
    let total = services.count();

    ReplOutput::ok(format!(
        "NexCore OS Status\n\
         ─────────────────\n\
         State:        {state}\n\
         Form factor:  {:?}\n\
         Tick count:   {}\n\
         Services:     {running}/{total} running\n\
         Security:     {}\n\
         Network:      {:?}\n\
         Audio:        {:?}\n\
         Vault:        {:?}",
        os.form_factor(),
        os.tick_count(),
        os.security().level(),
        os.network().state(),
        os.audio().state(),
        os.vault().state(),
    ))
}

fn exec_services<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let services = os.services().startup_order();
    let mut lines = Vec::with_capacity(services.len() + 2);
    lines.push("Services".to_string());
    lines.push("────────".to_string());

    for svc in &services {
        let machine = svc
            .machine_id
            .map_or_else(|| "none".to_string(), |id| format!("M{id}"));
        lines.push(format!(
            "  {:20} {:10?} pri={:?}  stos={}",
            svc.name, svc.state, svc.priority, machine
        ));
    }

    lines.push(format!("\n{} services total", services.len()));
    ReplOutput::ok(lines.join("\n"))
}

fn exec_service_detail<P: Platform>(os: &NexCoreOs<P>, name: &str) -> ReplOutput {
    let services = os.services().startup_order();
    let svc = services.iter().find(|s| s.name == name);

    svc.map_or_else(
        || ReplOutput::err(format!("Service not found: {name}")),
        |s| {
            ReplOutput::ok(format!(
                "Service: {}\n\
             ─────────────────\n\
             ID:       {:?}\n\
             State:    {:?}\n\
             Priority: {:?}\n\
             STOS:     {:?}\n\
             Alive:    {}",
                s.name,
                s.id,
                s.state,
                s.priority,
                s.machine_id,
                s.state.is_alive(),
            ))
        },
    )
}

fn exec_boot_log<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let boot = os.boot_sequence();
    let mut lines = vec!["Boot Log".to_string(), "────────".to_string()];
    for entry in boot.log() {
        lines.push(format!("  [{:?}] {}", entry.phase, entry.message));
    }
    lines.push(format!("\nBoot state: {:?}", boot.phase()));
    ReplOutput::ok(lines.join("\n"))
}

fn exec_shutdown<P: Platform>(os: &mut NexCoreOs<P>) -> ReplOutput {
    os.shutdown();
    ReplOutput::exit("NexCore OS shutting down...")
}

fn exec_help() -> ReplOutput {
    ReplOutput::ok(
        "\
NexCore OS Shell — Commands
═══════════════════════════

System
  status              OS state, tick count, subsystem summary
  services | svc      List all services
  service <name>      Detail on a specific service
  bootlog | boot      Boot sequence log
  shutdown            Initiate graceful shutdown
  help | ? | h        This help
  exit | quit | q     Exit REPL

Security (Guardian)
  security | sec      Security level, PAMP/DAMP counts
  threat <sev> <desc> Report a threat (low/medium/high/critical)
  quarantine | quar   List quarantined services

Network
  network | net       Network state summary
  interfaces | iface  List interfaces
  dns <hostname>      DNS cache lookup
  firewall | fw       Firewall rule summary

Audio
  audio               Audio state summary
  devices             List audio devices
  volume | vol [0-1]  Get/set master volume
  mute                Toggle system mute

Vault
  vault               Vault state
  secrets             List secrets (metadata)

Users
  users | who         List users
  login <user> <pass> Authenticate

Boot Chain
  bootchain | chain   Secure boot verification
  pcr                 PCR register values

Persistence
  snapshot | snap     Create state snapshot

History
  history | hist      Command history",
    )
}

// ── Security ──────────────────────────────────────────────────

fn exec_security<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let sec = os.security();
    ReplOutput::ok(format!(
        "Security Monitor\n\
         ────────────────\n\
         Level:      {}\n\
         PAMPs:      {}\n\
         DAMPs:      {}\n\
         Critical:   {}\n\
         Threats:    {}",
        sec.level(),
        sec.pamp_count(),
        sec.damp_count(),
        sec.is_critical(),
        sec.total_threats(),
    ))
}

fn exec_threat<P: Platform>(
    os: &mut NexCoreOs<P>,
    severity: &str,
    description: &str,
) -> ReplOutput {
    let sev = match severity.to_lowercase().as_str() {
        "low" => crate::security::ThreatSeverity::Low,
        "medium" | "med" => crate::security::ThreatSeverity::Medium,
        "high" => crate::security::ThreatSeverity::High,
        "critical" | "crit" => crate::security::ThreatSeverity::Critical,
        other => {
            return ReplOutput::err(format!(
                "Invalid severity: {other}\nValid: low, medium, high, critical"
            ));
        }
    };

    os.report_threat(sev, description, None);
    ReplOutput::ok(format!(
        "Threat reported: [{severity}] {description}\nSecurity level: {}",
        os.security().level()
    ))
}

fn exec_quarantine<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let services = os.services().startup_order();
    let quarantined: Vec<_> = services
        .iter()
        .filter(|s| os.security().is_quarantined(s.id))
        .collect();

    if quarantined.is_empty() {
        ReplOutput::ok("No quarantined services.")
    } else {
        let mut lines = vec![
            "Quarantined Services".to_string(),
            "────────────────────".to_string(),
        ];
        for svc in &quarantined {
            lines.push(format!("  {} (id={:?})", svc.name, svc.id));
        }
        ReplOutput::ok(lines.join("\n"))
    }
}

// ── Network ──────────────────────────────────────────────────

fn exec_network<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    ReplOutput::ok(os.network().summary())
}

fn exec_interfaces<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let count = os.network().interface_count();
    if count == 0 {
        return ReplOutput::ok("No interfaces registered.");
    }

    let mut lines = vec![
        "Network Interfaces".to_string(),
        "──────────────────".to_string(),
        format!("  {} interface(s) registered", count),
    ];

    // Connections info
    let connected = os.network().connected_count();
    lines.push(format!("  {connected} active connection(s)"));
    lines.push(format!(
        "\nFirewall: {} rules",
        os.network().firewall().rule_count()
    ));

    ReplOutput::ok(lines.join("\n"))
}

fn exec_dns<P: Platform>(os: &mut NexCoreOs<P>, hostname: &str) -> ReplOutput {
    os.network_mut().resolve_cached(hostname).map_or_else(
        || ReplOutput::ok(format!("{hostname}: not in DNS cache")),
        |addr| ReplOutput::ok(format!("{hostname} → {addr:?}")),
    )
}

fn exec_firewall<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let fw = os.network().firewall();
    ReplOutput::ok(format!(
        "Firewall\n\
         ────────\n\
         Rules:   {}\n\
         Enabled: true",
        fw.rule_count(),
    ))
}

// ── Audio ────────────────────────────────────────────────────

fn exec_audio<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    ReplOutput::ok(os.audio().summary())
}

fn exec_audio_devices<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let audio = os.audio();
    if audio.device_count() == 0 {
        return ReplOutput::ok("No audio devices registered.");
    }

    let mut lines = vec!["Audio Devices".to_string(), "─────────────".to_string()];

    for dev in audio.devices() {
        lines.push(format!("  {}", dev.summary()));
    }

    lines.push(format!(
        "\n{} output, {} input",
        audio.output_device_count(),
        audio.input_device_count(),
    ));
    ReplOutput::ok(lines.join("\n"))
}

fn exec_volume<P: Platform>(os: &mut NexCoreOs<P>, level: Option<f32>) -> ReplOutput {
    if let Some(v) = level {
        if !(0.0..=1.0).contains(&v) {
            return ReplOutput::err("Volume must be between 0.0 and 1.0");
        }
        os.audio_mut().set_master_volume(v);
        ReplOutput::ok(format!("Volume set to {:.0}%", v * 100.0))
    } else {
        let vol = os.audio().master_volume();
        let muted = if os.audio().is_muted() {
            " (MUTED)"
        } else {
            ""
        };
        ReplOutput::ok(format!("Master volume: {:.0}%{muted}", vol * 100.0))
    }
}

fn exec_mute<P: Platform>(os: &mut NexCoreOs<P>) -> ReplOutput {
    os.audio_mut().toggle_mute();
    let state = if os.audio().is_muted() {
        "MUTED"
    } else {
        "UNMUTED"
    };
    ReplOutput::ok(format!("Audio {state}"))
}

// ── Vault ────────────────────────────────────────────────────

fn exec_vault<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let vault = os.vault();
    ReplOutput::ok(format!(
        "Vault\n\
         ─────\n\
         State:      {:?}\n\
         Operational: {}\n\
         Secrets:    {}\n\
         Operations: {}",
        vault.state(),
        vault.is_operational(),
        vault.secret_count().unwrap_or(0),
        vault.operations(),
    ))
}

fn exec_secrets<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let vault = os.vault();
    if !vault.is_operational() {
        return ReplOutput::err("Vault is not operational. Initialize first.");
    }

    match vault.list_secrets() {
        Ok(secrets) if secrets.is_empty() => ReplOutput::ok("No secrets stored."),
        Ok(secrets) => {
            let mut lines = vec!["Secrets".to_string(), "───────".to_string()];
            for info in &secrets {
                lines.push(format!("  {:20} [{:?}]", info.name, info.category,));
            }
            lines.push(format!("\n{} secrets total", secrets.len()));
            ReplOutput::ok(lines.join("\n"))
        }
        Err(_) => ReplOutput::err("Cannot list secrets — vault locked."),
    }
}

// ── Users ────────────────────────────────────────────────────

fn exec_users<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let users = os.users();
    let list = users.list_users();

    if list.is_empty() {
        return ReplOutput::ok("No users registered.");
    }

    let mut lines = vec!["Users".to_string(), "─────".to_string()];
    for u in &list {
        lines.push(format!("  {:20} {:?} ({:?})", u.username, u.role, u.status,));
    }
    lines.push(format!(
        "\n{} user(s), {} active session(s)",
        list.len(),
        users.active_session_count(),
    ));
    ReplOutput::ok(lines.join("\n"))
}

fn exec_login<P: Platform>(os: &mut NexCoreOs<P>, username: &str, password: &str) -> ReplOutput {
    match os.login(username, password) {
        Ok(session) => ReplOutput::ok(format!(
            "Logged in as {} ({:?})\nSession token: {}...{}",
            username,
            session.role,
            &session.token[..8.min(session.token.len())],
            &session.token[session.token.len().saturating_sub(4)..],
        )),
        Err(e) => ReplOutput::err(format!("Login failed: {e}")),
    }
}

// ── Boot Chain ───────────────────────────────────────────────

fn exec_bootchain<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let chain = os.secure_boot();
    let verification = chain.verify_chain();

    ReplOutput::ok(format!(
        "Secure Boot Chain\n\
         ─────────────────\n\
         Policy:       {:?}\n\
         Records:      {}\n\
         Failures:     {}\n\
         Degraded:     {}\n\
         All verified: {}\n\
         Proceed:      {}",
        chain.policy(),
        chain.record_count(),
        chain.failure_count(),
        chain.is_degraded(),
        verification.all_verified,
        verification.should_proceed(),
    ))
}

fn exec_pcr<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let chain = os.secure_boot();
    let log = chain.attestation_log();

    let mut lines = vec!["PCR Values".to_string(), "──────────".to_string()];

    for record in log {
        lines.push(format!(
            "  PCR[{}] {:12?} verified={} {}",
            record.stage.pcr_index(),
            record.stage,
            record.verified,
            record.description,
        ));
    }

    ReplOutput::ok(lines.join("\n"))
}

// ── Persistence ──────────────────────────────────────────────

fn exec_snapshot<P: Platform>(os: &NexCoreOs<P>) -> ReplOutput {
    let snap = os.create_snapshot(false);
    ReplOutput::ok(format!(
        "Snapshot Created\n\
         ────────────────\n\
         Platform:  {}\n\
         State:     {}\n\
         Services:  {}\n\
         Ticks:     {}\n\
         IPC total: {}\n\
         Security:  {}",
        snap.platform,
        snap.boot_phase,
        snap.services.len(),
        snap.tick_count,
        snap.ipc_events_emitted,
        snap.security_level,
    ))
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_pal::FormFactor;
    use nexcore_pal_linux::LinuxPlatform;

    fn boot_os() -> NexCoreOs<LinuxPlatform> {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        NexCoreOs::boot(platform).ok().unwrap_or_else(|| {
            let p = LinuxPlatform::virtual_platform(FormFactor::Desktop);
            NexCoreOs::boot(p).ok().unwrap_or_else(|| unreachable!())
        })
    }

    // ═══════════════════════════════════════════════════════════
    // PARSE TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn parse_empty() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse(""), ReplCommand::Empty);
        assert_eq!(repl.parse("   "), ReplCommand::Empty);
    }

    #[test]
    fn parse_status() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("status"), ReplCommand::Status);
        assert_eq!(repl.parse("  status  "), ReplCommand::Status);
    }

    #[test]
    fn parse_services() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("services"), ReplCommand::Services);
        assert_eq!(repl.parse("svc"), ReplCommand::Services);
    }

    #[test]
    fn parse_service_detail() {
        let repl = OsRepl::new();
        assert_eq!(
            repl.parse("service guardian"),
            ReplCommand::ServiceDetail("guardian".to_string())
        );
    }

    #[test]
    fn parse_service_no_arg_fallback() {
        let repl = OsRepl::new();
        // "service" with no arg falls back to listing all
        assert_eq!(repl.parse("service"), ReplCommand::Services);
    }

    #[test]
    fn parse_help_aliases() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("help"), ReplCommand::Help);
        assert_eq!(repl.parse("?"), ReplCommand::Help);
        assert_eq!(repl.parse("h"), ReplCommand::Help);
    }

    #[test]
    fn parse_exit_aliases() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("exit"), ReplCommand::Exit);
        assert_eq!(repl.parse("quit"), ReplCommand::Exit);
        assert_eq!(repl.parse("q"), ReplCommand::Exit);
    }

    #[test]
    fn parse_security() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("security"), ReplCommand::Security);
        assert_eq!(repl.parse("sec"), ReplCommand::Security);
    }

    #[test]
    fn parse_threat() {
        let repl = OsRepl::new();
        assert_eq!(
            repl.parse("threat high SSH brute force"),
            ReplCommand::Threat {
                severity: "high".to_string(),
                description: "SSH brute force".to_string(),
            }
        );
    }

    #[test]
    fn parse_threat_missing_args() {
        let repl = OsRepl::new();
        match repl.parse("threat") {
            ReplCommand::Unknown(_) => {}
            other => panic!("Expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn parse_network_aliases() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("network"), ReplCommand::Network);
        assert_eq!(repl.parse("net"), ReplCommand::Network);
    }

    #[test]
    fn parse_interfaces_aliases() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("interfaces"), ReplCommand::Interfaces);
        assert_eq!(repl.parse("iface"), ReplCommand::Interfaces);
        assert_eq!(repl.parse("ifconfig"), ReplCommand::Interfaces);
    }

    #[test]
    fn parse_dns() {
        let repl = OsRepl::new();
        assert_eq!(
            repl.parse("dns example.com"),
            ReplCommand::Dns("example.com".to_string())
        );
    }

    #[test]
    fn parse_dns_missing_arg() {
        let repl = OsRepl::new();
        match repl.parse("dns") {
            ReplCommand::Unknown(_) => {}
            other => panic!("Expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn parse_audio() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("audio"), ReplCommand::Audio);
    }

    #[test]
    fn parse_volume_get() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("volume"), ReplCommand::Volume(None));
        assert_eq!(repl.parse("vol"), ReplCommand::Volume(None));
    }

    #[test]
    fn parse_volume_set() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("volume 0.5"), ReplCommand::Volume(Some(0.5)));
    }

    #[test]
    fn parse_volume_invalid() {
        let repl = OsRepl::new();
        match repl.parse("volume abc") {
            ReplCommand::Unknown(_) => {}
            other => panic!("Expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn parse_mute() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("mute"), ReplCommand::Mute);
    }

    #[test]
    fn parse_vault() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("vault"), ReplCommand::Vault);
    }

    #[test]
    fn parse_users() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("users"), ReplCommand::Users);
        assert_eq!(repl.parse("who"), ReplCommand::Users);
    }

    #[test]
    fn parse_login() {
        let repl = OsRepl::new();
        assert_eq!(
            repl.parse("login admin secret123"),
            ReplCommand::Login {
                username: "admin".to_string(),
                password: "secret123".to_string(),
            }
        );
    }

    #[test]
    fn parse_bootchain() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("bootchain"), ReplCommand::BootChain);
        assert_eq!(repl.parse("chain"), ReplCommand::BootChain);
    }

    #[test]
    fn parse_pcr() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("pcr"), ReplCommand::Pcr);
    }

    #[test]
    fn parse_snapshot() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("snapshot"), ReplCommand::Snapshot);
        assert_eq!(repl.parse("snap"), ReplCommand::Snapshot);
    }

    #[test]
    fn parse_history() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("history"), ReplCommand::History);
        assert_eq!(repl.parse("hist"), ReplCommand::History);
    }

    #[test]
    fn parse_unknown() {
        let repl = OsRepl::new();
        assert_eq!(
            repl.parse("foobar"),
            ReplCommand::Unknown("foobar".to_string())
        );
    }

    #[test]
    fn parse_case_insensitive() {
        let repl = OsRepl::new();
        assert_eq!(repl.parse("STATUS"), ReplCommand::Status);
        assert_eq!(repl.parse("Help"), ReplCommand::Help);
        assert_eq!(repl.parse("SECURITY"), ReplCommand::Security);
    }

    // ═══════════════════════════════════════════════════════════
    // EXECUTE TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn exec_status_shows_running() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Status, &mut os);
        assert!(output.success);
        assert!(output.text.contains("RUNNING"));
        assert!(output.text.contains("Desktop"));
    }

    #[test]
    fn exec_services_lists_all() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Services, &mut os);
        assert!(output.success);
        assert!(output.text.contains("guardian"));
        assert!(output.text.contains("network"));
        assert!(output.text.contains("audio"));
        assert!(output.text.contains("vault"));
        assert!(output.text.contains("11 services"));
    }

    #[test]
    fn exec_service_detail_found() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::ServiceDetail("guardian".to_string()), &mut os);
        assert!(output.success);
        assert!(output.text.contains("guardian"));
        assert!(output.text.contains("Core"));
    }

    #[test]
    fn exec_service_detail_not_found() {
        let mut os = boot_os();
        let output = execute_command(
            &ReplCommand::ServiceDetail("nonexistent".to_string()),
            &mut os,
        );
        assert!(!output.success);
        assert!(output.text.contains("not found"));
    }

    #[test]
    fn exec_help_complete() {
        let output = exec_help();
        assert!(output.success);
        assert!(output.text.contains("status"));
        assert!(output.text.contains("security"));
        assert!(output.text.contains("network"));
        assert!(output.text.contains("audio"));
        assert!(output.text.contains("vault"));
        assert!(output.text.contains("users"));
        assert!(output.text.contains("bootchain"));
        assert!(output.text.contains("snapshot"));
        assert!(output.text.contains("history"));
    }

    #[test]
    fn exec_exit() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Exit, &mut os);
        assert!(output.should_exit);
    }

    #[test]
    fn exec_security_green() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Security, &mut os);
        assert!(output.success);
        assert!(output.text.contains("GREEN"));
        assert!(output.text.contains("PAMPs"));
    }

    #[test]
    fn exec_threat_escalates() {
        let mut os = boot_os();
        let output = execute_command(
            &ReplCommand::Threat {
                severity: "high".to_string(),
                description: "test threat".to_string(),
            },
            &mut os,
        );
        assert!(output.success);
        assert!(output.text.contains("ORANGE"));
    }

    #[test]
    fn exec_threat_invalid_severity() {
        let mut os = boot_os();
        let output = execute_command(
            &ReplCommand::Threat {
                severity: "banana".to_string(),
                description: "test".to_string(),
            },
            &mut os,
        );
        assert!(!output.success);
        assert!(output.text.contains("Invalid severity"));
    }

    #[test]
    fn exec_quarantine_empty() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Quarantine, &mut os);
        assert!(output.success);
        assert!(output.text.contains("No quarantined"));
    }

    #[test]
    fn exec_network_summary() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Network, &mut os);
        assert!(output.success);
        assert!(output.text.contains("Network"));
    }

    #[test]
    fn exec_interfaces_none() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Interfaces, &mut os);
        assert!(output.success);
        assert!(output.text.contains("No interfaces"));
    }

    #[test]
    fn exec_dns_cache_miss() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Dns("example.com".to_string()), &mut os);
        assert!(output.success);
        assert!(output.text.contains("not in DNS cache"));
    }

    #[test]
    fn exec_firewall() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Firewall, &mut os);
        assert!(output.success);
        assert!(output.text.contains("Firewall"));
        assert!(output.text.contains("Rules"));
    }

    #[test]
    fn exec_audio_summary() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Audio, &mut os);
        assert!(output.success);
        assert!(output.text.contains("Audio"));
        assert!(output.text.contains("Ready"));
    }

    #[test]
    fn exec_audio_devices_empty() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::AudioDevices, &mut os);
        assert!(output.success);
        assert!(output.text.contains("No audio devices"));
    }

    #[test]
    fn exec_volume_get() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Volume(None), &mut os);
        assert!(output.success);
        assert!(output.text.contains("75%"));
    }

    #[test]
    fn exec_volume_set() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Volume(Some(0.5)), &mut os);
        assert!(output.success);
        assert!(output.text.contains("50%"));
        assert_eq!(os.audio().master_volume(), 0.5);
    }

    #[test]
    fn exec_volume_out_of_range() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Volume(Some(2.0)), &mut os);
        assert!(!output.success);
        assert!(output.text.contains("between 0.0 and 1.0"));
    }

    #[test]
    fn exec_mute_toggle() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Mute, &mut os);
        assert!(output.success);
        assert!(output.text.contains("MUTED"));

        let output2 = execute_command(&ReplCommand::Mute, &mut os);
        assert!(output2.text.contains("UNMUTED"));
    }

    #[test]
    fn exec_vault_state() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Vault, &mut os);
        assert!(output.success);
        assert!(output.text.contains("Vault"));
        assert!(output.text.contains("Uninitialized") || output.text.contains("State"));
    }

    #[test]
    fn exec_secrets_not_operational() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Secrets, &mut os);
        assert!(!output.success);
        assert!(output.text.contains("not operational"));
    }

    #[test]
    fn exec_users_empty() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Users, &mut os);
        assert!(output.success);
        assert!(output.text.contains("No users"));
    }

    #[test]
    fn exec_bootchain() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::BootChain, &mut os);
        assert!(output.success);
        assert!(output.text.contains("Permissive"));
        assert!(output.text.contains("Records"));
        assert!(output.text.contains("4"));
    }

    #[test]
    fn exec_pcr_values() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Pcr, &mut os);
        assert!(output.success);
        assert!(output.text.contains("PCR["));
        assert!(output.text.contains("NexCoreOs"));
        assert!(output.text.contains("Init"));
        assert!(output.text.contains("Services"));
        assert!(output.text.contains("Shell"));
    }

    #[test]
    fn exec_snapshot() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Snapshot, &mut os);
        assert!(output.success);
        assert!(output.text.contains("Snapshot Created"));
        assert!(output.text.contains("11"));
    }

    #[test]
    fn exec_shutdown_exits() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Shutdown, &mut os);
        assert!(output.should_exit);
        assert_eq!(os.state(), OsState::Halted);
    }

    #[test]
    fn exec_unknown_command() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Unknown("foobar".to_string()), &mut os);
        assert!(!output.success);
        assert!(output.text.contains("Unknown command"));
    }

    #[test]
    fn exec_empty() {
        let mut os = boot_os();
        let output = execute_command(&ReplCommand::Empty, &mut os);
        assert!(output.success);
        assert!(output.text.is_empty());
    }

    // ═══════════════════════════════════════════════════════════
    // OS REPL INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn repl_eval_status() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();
        let output = repl.eval("status", &mut os);
        assert!(output.success);
        assert!(output.text.contains("RUNNING"));
        assert_eq!(repl.command_count(), 1);
    }

    #[test]
    fn repl_history_tracks() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        repl.eval("status", &mut os);
        repl.eval("security", &mut os);
        repl.eval("help", &mut os);

        assert_eq!(repl.command_count(), 3);
        assert_eq!(repl.history().len(), 3);
        assert_eq!(repl.history()[0].seq, 1);
        assert_eq!(repl.history()[2].seq, 3);
    }

    #[test]
    fn repl_history_skips_empty() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        repl.eval("", &mut os);
        repl.eval("   ", &mut os);
        assert_eq!(repl.command_count(), 0);
        assert!(repl.history().is_empty());
    }

    #[test]
    fn repl_last_n() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        for _ in 0..10 {
            repl.eval("status", &mut os);
        }

        let last_3 = repl.last_n(3);
        assert_eq!(last_3.len(), 3);
        assert_eq!(last_3[0].seq, 8);
        assert_eq!(last_3[2].seq, 10);
    }

    #[test]
    fn repl_prompt() {
        let repl = OsRepl::new();
        assert_eq!(repl.prompt(), "nexcore> ");
    }

    #[test]
    fn repl_records_failure() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        repl.eval("foobar", &mut os);
        assert_eq!(repl.history().len(), 1);
        assert!(!repl.history()[0].success);
    }

    #[test]
    fn repl_full_session() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        // Simulate a full diagnostic session
        let status = repl.eval("status", &mut os);
        assert!(status.text.contains("RUNNING"));

        let sec = repl.eval("security", &mut os);
        assert!(sec.text.contains("GREEN"));

        let boot = repl.eval("bootchain", &mut os);
        assert!(boot.text.contains("Permissive"));

        let vol = repl.eval("vol", &mut os);
        assert!(vol.text.contains("75%"));

        let net = repl.eval("net", &mut os);
        assert!(net.text.contains("Network"));

        let snap = repl.eval("snap", &mut os);
        assert!(snap.text.contains("Snapshot Created"));

        assert_eq!(repl.command_count(), 6);
    }

    #[test]
    fn repl_threat_then_security() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        let t = repl.eval("threat high SSH brute force attempt", &mut os);
        assert!(t.success);
        assert!(t.text.contains("ORANGE"));

        let s = repl.eval("sec", &mut os);
        assert!(s.text.contains("ORANGE"));
        assert!(s.text.contains("1")); // threat count
    }

    #[test]
    fn repl_volume_set_then_get() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        let set = repl.eval("vol 0.3", &mut os);
        assert!(set.text.contains("30%"));

        let get = repl.eval("vol", &mut os);
        assert!(get.text.contains("30%"));
    }

    #[test]
    fn repl_mute_cycle() {
        let mut os = boot_os();
        let mut repl = OsRepl::new();

        let m1 = repl.eval("mute", &mut os);
        assert!(m1.text.contains("MUTED"));

        let v = repl.eval("vol", &mut os);
        assert!(v.text.contains("MUTED"));

        let m2 = repl.eval("mute", &mut os);
        assert!(m2.text.contains("UNMUTED"));
    }
}
