//! Low Memory Alert Hook
//!
//! UserPromptSubmit hook that warns when available RAM drops below threshold.
//!
//! # Safety Axiom Reference
//! - Axiom 3 (Resource Conservation): Early warning prevents OOM conditions
//! - UACA L1: < 20 LOC core logic

use nexcore_hooks::protocol::{HookInput, HookOutput};
use std::io::{self, Read};

/// Threshold in MB - warn when available RAM drops below this
const THRESHOLD_MB: u64 = 1024; // 1 GB

fn main() {
    let mut input_json = String::new();
    if io::stdin().read_to_string(&mut input_json).is_err() {
        return;
    }

    let input: HookInput = match serde_json::from_str(&input_json) {
        Ok(i) => i,
        Err(_) => return,
    };

    // Only run on UserPromptSubmit
    if input.hook_event_name != "UserPromptSubmit" {
        return;
    }

    // Check available memory
    if let Some((available_mb, used_mb, total_mb)) = get_memory_info() {
        if available_mb < THRESHOLD_MB {
            let msg = format!(
                "⚠️ **LOW MEMORY WARNING** ────────────────────────────\n\
                 Available: {} MB (threshold: {} MB)\n\
                 Used: {} MB / {} MB ({:.0}%)\n\
                 \n\
                 Consider:\n\
                 • Close unused applications\n\
                 • Kill orphaned processes: `pgrep -x claude`\n\
                 • Drop caches: `sync && echo 3 | sudo tee /proc/sys/vm/drop_caches`\n\
                 ───────────────────────────────────────────────────────\n",
                available_mb,
                THRESHOLD_MB,
                used_mb,
                total_mb,
                (used_mb as f64 / total_mb as f64) * 100.0
            );
            HookOutput::with_context(msg).emit();
            return;
        }
    }

    // No warning needed - emit empty context
    println!("{{}}");
}

/// Get memory info from /proc/meminfo
/// Returns (available_mb, used_mb, total_mb)
fn get_memory_info() -> Option<(u64, u64, u64)> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;

    let mut total_kb = 0u64;
    let mut available_kb = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = parse_meminfo_value(line)?;
        } else if line.starts_with("MemAvailable:") {
            available_kb = parse_meminfo_value(line)?;
        }
    }

    if total_kb == 0 {
        return None;
    }

    let total_mb = total_kb / 1024;
    let available_mb = available_kb / 1024;
    let used_mb = total_mb.saturating_sub(available_mb);

    Some((available_mb, used_mb, total_mb))
}

/// Parse a line like "MemTotal:       16000000 kB"
fn parse_meminfo_value(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1)?.parse().ok()
}
