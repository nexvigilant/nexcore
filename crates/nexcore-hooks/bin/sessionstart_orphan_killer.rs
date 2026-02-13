//! Orphan Claude Process Killer
//!
//! SessionStart hook that kills orphaned Claude processes from previous sessions.
//! This prevents memory buildup when sessions are terminated via ctrl-c or terminal close.
//!
//! # Problem Statement
//! When users close terminals abruptly (ctrl-c, window close), Claude processes may
//! become orphaned (reparented to init/systemd with PPID=1). These zombies accumulate
//! memory over time, degrading system performance.
//!
//! # Solution
//! On each SessionStart, scan for orphaned Claude processes (PPID=1, age > 60s) and
//! terminate them with SIGKILL. This is a homeostatic control mechanism.
//!
//! # Safety Axiom Reference
//! - Axiom 3 (Resource Conservation): Prevents unbounded memory consumption
//! - Axiom 1 (Non-Maleficence): Only kills orphaned processes, never active sessions
//! - UACA L1: Core logic < 20 LOC
//!
//! # Hook Protocol
//! - Event: SessionStart
//! - Output: Session context message if orphans killed, skip otherwise
//! - Exit: 0 (always continues)

use nexcore_hooks::protocol::{HookInput, HookOutput};
use std::io::{self, Read};
use std::process::Command;

fn main() {
    let mut input_json = String::new();
    if io::stdin().read_to_string(&mut input_json).is_err() {
        HookOutput::skip_session().emit();
        return;
    }

    let input: HookInput = match serde_json::from_str(&input_json) {
        Ok(i) => i,
        Err(_) => {
            HookOutput::skip_session().emit();
            return;
        }
    };

    // Only run on SessionStart
    if input.hook_event_name != "SessionStart" {
        HookOutput::skip_session().emit();
        return;
    }

    // Get current process ID (this is the new session)
    let current_pid = std::process::id();

    // Find all claude processes older than 5 minutes (orphans)
    let killed = kill_orphan_claude_processes(current_pid);

    if killed > 0 {
        let msg = format!(
            "🧹 **ORPHAN CLEANUP** ─────────────────────────────────\n\
             Killed {} orphaned Claude process(es) from previous sessions.\n\
             Memory recovered. Run `free -h` to verify.\n\
             ───────────────────────────────────────────────────────\n",
            killed
        );
        HookOutput::with_session_context(msg).emit();
    } else {
        HookOutput::skip_session().emit();
    }
}

/// Kill orphaned Claude processes, return count killed
fn kill_orphan_claude_processes(_current_pid: u32) -> usize {
    // Find claude processes (main binary only, not subprocesses)
    let output = Command::new("pgrep")
        .args(["-x", "claude"]) // Exact match on "claude"
        .output();

    let pids: Vec<u32> = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        Err(_) => return 0,
    };

    let mut killed = 0;

    for pid in pids {
        // Get parent PID - orphaned processes have PPID=1 (reparented to init)
        let ppid = match get_ppid(pid) {
            Some(p) => p,
            None => continue,
        };

        // Only kill processes with PPID=1 (orphaned)
        // Active sessions have real parent PIDs (terminal, IDE, etc.)
        if ppid != 1 {
            continue;
        }

        // Check if process is old enough (started > 60 seconds ago)
        if !is_process_old(pid, 60) {
            continue;
        }

        // Kill the orphan
        if Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            killed += 1;
        }
    }

    killed
}

/// Get parent PID of a process
fn get_ppid(pid: u32) -> Option<u32> {
    let stat_path = format!("/proc/{}/stat", pid);
    let content = std::fs::read_to_string(stat_path).ok()?;
    // Format: pid (name) state ppid ...
    // Find closing paren, then split rest
    let close_paren = content.rfind(')')?;
    let rest = &content[close_paren + 2..];
    let parts: Vec<&str> = rest.split_whitespace().collect();
    // parts[0] is state, parts[1] is ppid
    parts.get(1)?.parse().ok()
}

/// Check if process started more than `min_age_secs` seconds ago
fn is_process_old(pid: u32, min_age_secs: u64) -> bool {
    let stat_path = format!("/proc/{}/stat", pid);
    let content = match std::fs::read_to_string(&stat_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Get process start time (field 22, 0-indexed after parsing)
    let close_paren = match content.rfind(')') {
        Some(p) => p,
        None => return false,
    };
    let rest = &content[close_paren + 2..];
    let parts: Vec<&str> = rest.split_whitespace().collect();

    // Field 22 (starttime) is at index 19 after state
    let start_ticks: u64 = match parts.get(19).and_then(|s| s.parse().ok()) {
        Some(t) => t,
        None => return false,
    };

    // Get system uptime
    let uptime: f64 = match std::fs::read_to_string("/proc/uptime") {
        Ok(s) => s
            .split_whitespace()
            .next()
            .and_then(|u| u.parse().ok())
            .unwrap_or(0.0),
        Err(_) => return false,
    };

    // Get clock ticks per second (usually 100)
    let ticks_per_sec: u64 = 100; // sysconf(_SC_CLK_TCK) is typically 100 on Linux

    // Calculate process age in seconds
    let start_secs = start_ticks / ticks_per_sec;
    let uptime_secs = uptime as u64;
    let process_age = uptime_secs.saturating_sub(start_secs);

    process_age >= min_age_secs
}
