//! Signal Receiver Daemon
//!
//! Processes neurotransmitter signals emitted by hooks.
//! Runs as a background process, consuming signals asynchronously.
//!
//! # T1 Primitive Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | Signal consumption | Sequence | σ |
//! | Aggregation | Mapping | μ |
//! | State accumulation | State | ς |
//! | File watching | Persistence | π |

use nexcore_hook_lib::neurotransmitter::{Signal, read_signals, rotate_signals, signals};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::Duration;

const SIGNAL_PATH: &str = "/home/matthew/.claude/brain/telemetry/signals.jsonl";
const METRICS_PATH: &str = "/home/matthew/.claude/brain/telemetry/metrics.json";
const CYTOKINE_METRICS_PATH: &str = "/home/matthew/.claude/brain/telemetry/cytokine_metrics.json";
const POLL_INTERVAL_MS: u64 = 500;
const RECENT_CYTOKINE_LIMIT: usize = 100;

/// Aggregated metrics from signals.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Metrics {
    signals_processed: u64,
    skill_invocations: HashMap<String, u64>,
    tool_blocks: HashMap<String, u64>,
    hook_timings: HashMap<String, Vec<u64>>,
    custom_metrics: HashMap<String, f64>,
    last_update_ms: u128,
}

impl Metrics {
    fn load() -> Self {
        std::fs::read_to_string(METRICS_PATH)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(METRICS_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    fn process_signal(&mut self, signal: &Signal) {
        self.signals_processed += 1;
        self.last_update_ms = signal.timestamp_ms;

        match signal.signal_type.as_str() {
            signals::SKILL_INVOKED => self.record_skill(signal),
            signals::TOOL_BLOCKED => self.record_block(signal),
            signals::HOOK_END => self.record_timing(signal),
            signals::METRIC => self.record_metric(signal),
            _ => {}
        }
    }

    fn record_skill(&mut self, signal: &Signal) {
        if let Some(skill) = signal.data.get("skill") {
            *self.skill_invocations.entry(skill.clone()).or_default() += 1;
        }
    }

    fn record_block(&mut self, signal: &Signal) {
        if let Some(hook) = signal.data.get("hook") {
            *self.tool_blocks.entry(hook.clone()).or_default() += 1;
        }
    }

    fn record_timing(&mut self, signal: &Signal) {
        let hook = match signal.data.get("hook") {
            Some(h) => h,
            None => return,
        };
        let duration = match signal.data.get("duration_us").and_then(|s| s.parse().ok()) {
            Some(d) => d,
            None => return,
        };
        let timings = self.hook_timings.entry(hook.clone()).or_default();
        timings.push(duration);
        if timings.len() > 100 {
            timings.remove(0);
        }
    }

    fn record_metric(&mut self, signal: &Signal) {
        let name = match signal.data.get("name") {
            Some(n) => n,
            None => return,
        };
        let value = match signal.data.get("value").and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => return,
        };
        self.custom_metrics.insert(name.clone(), value);
    }
}

// ============================================================================
// Cytokine-Aware Metrics
// ============================================================================

/// A recent cytokine signal snapshot for the rolling buffer.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecentCytokine {
    timestamp_ms: u128,
    family: String,
    hook: String,
    severity: String,
    signal_type: String,
}

/// Aggregated cytokine metrics from signals with `cytokine:` prefix.
///
/// # Tier: T2-C
/// Grounds to: T1(u64, String) via HashMap counters + VecDeque buffer.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CytokineMetrics {
    /// Count by cytokine family: "tnf_alpha" → N
    by_family: HashMap<String, u64>,
    /// Count by originating hook: "unwrap-guardian" → N
    by_hook: HashMap<String, u64>,
    /// Count by severity level: "critical" → N
    by_severity: HashMap<String, u64>,
    /// Total pro-inflammatory signals (IL-1, IL-6, TNF-alpha, IFN-gamma)
    activating_count: u64,
    /// Total anti-inflammatory signals (IL-10, TGF-beta, CSF)
    suppressing_count: u64,
    /// Rolling buffer of most recent cytokines
    recent: VecDeque<RecentCytokine>,
    /// Total cytokine signals processed
    total: u64,
    /// Last update timestamp (ms since epoch)
    last_update_ms: u128,
}

impl CytokineMetrics {
    fn load() -> Self {
        std::fs::read_to_string(CYTOKINE_METRICS_PATH)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(CYTOKINE_METRICS_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Returns true if this signal is a cytokine (starts with "cytokine:")
    fn is_cytokine(signal: &Signal) -> bool {
        signal.signal_type.starts_with("cytokine:")
    }

    /// Parse a cytokine signal_type "cytokine:{family}:{name}" into (family, name).
    fn parse_signal_type(signal_type: &str) -> (String, String) {
        let stripped = signal_type.strip_prefix("cytokine:").unwrap_or(signal_type);
        match stripped.split_once(':') {
            Some((family, name)) => (family.to_string(), name.to_string()),
            None => (stripped.to_string(), String::new()),
        }
    }

    /// Classify whether a family is pro-inflammatory (activating) or anti-inflammatory (suppressing).
    fn is_activating(family: &str) -> bool {
        matches!(family, "il1" | "il6" | "tnf_alpha" | "ifn_gamma")
    }

    fn process_cytokine(&mut self, signal: &Signal) {
        let (family, _name) = Self::parse_signal_type(&signal.signal_type);

        self.total += 1;
        self.last_update_ms = signal.timestamp_ms;

        // By family
        *self.by_family.entry(family.clone()).or_default() += 1;

        // By hook (from data map)
        if let Some(hook) = signal
            .data
            .get("source")
            .or_else(|| signal.data.get("payload_hook"))
        {
            *self.by_hook.entry(hook.clone()).or_default() += 1;
        }

        // By severity
        if let Some(sev) = signal.data.get("severity") {
            *self.by_severity.entry(sev.clone()).or_default() += 1;
        }

        // Activating vs suppressing
        if Self::is_activating(&family) {
            self.activating_count += 1;
        } else {
            self.suppressing_count += 1;
        }

        // Recent buffer (bounded)
        let hook = signal
            .data
            .get("source")
            .or_else(|| signal.data.get("payload_hook"))
            .cloned()
            .unwrap_or_default();

        self.recent.push_back(RecentCytokine {
            timestamp_ms: signal.timestamp_ms,
            family: family.clone(),
            hook,
            severity: signal.data.get("severity").cloned().unwrap_or_default(),
            signal_type: signal.signal_type.clone(),
        });
        while self.recent.len() > RECENT_CYTOKINE_LIMIT {
            self.recent.pop_front();
        }
    }
}

fn truncate_name(name: &str, max: usize) -> String {
    if name.len() > max {
        format!("{}...", &name[..max.saturating_sub(3)])
    } else {
        name.to_string()
    }
}

fn format_header(out: &mut String, total: u64) {
    out.push_str("╔══════════════════════════════════════════════╗\n");
    out.push_str("║         SIGNAL RECEIVER METRICS              ║\n");
    out.push_str("╠══════════════════════════════════════════════╣\n");
    out.push_str(&format!("║ Signals Processed: {:>24} ║\n", total));
    out.push_str("╠══════════════════════════════════════════════╣\n");
}

fn format_skills(out: &mut String, skills: &HashMap<String, u64>) {
    out.push_str("║ SKILL INVOCATIONS                            ║\n");
    for (skill, count) in skills {
        let name = truncate_name(skill, 30);
        out.push_str(&format!("║   {:<30} {:>10} ║\n", name, count));
    }
}

fn format_blocks(out: &mut String, blocks: &HashMap<String, u64>) {
    if blocks.is_empty() {
        return;
    }
    out.push_str("╠══════════════════════════════════════════════╣\n");
    out.push_str("║ TOOL BLOCKS BY HOOK                          ║\n");
    for (hook, count) in blocks {
        let name = truncate_name(hook, 30);
        out.push_str(&format!("║   {:<30} {:>10} ║\n", name, count));
    }
}

fn format_timings(out: &mut String, timings: &HashMap<String, Vec<u64>>) {
    if timings.is_empty() {
        return;
    }
    out.push_str("╠══════════════════════════════════════════════╣\n");
    out.push_str("║ HOOK TIMINGS (avg μs)                        ║\n");
    for (hook, samples) in timings {
        if samples.is_empty() {
            continue;
        }
        let avg: u64 = samples.iter().sum::<u64>() / samples.len() as u64;
        let name = truncate_name(hook, 30);
        out.push_str(&format!("║   {:<30} {:>10} ║\n", name, avg));
    }
}

fn summary(metrics: &Metrics) -> String {
    let mut out = String::new();
    format_header(&mut out, metrics.signals_processed);
    format_skills(&mut out, &metrics.skill_invocations);
    format_blocks(&mut out, &metrics.tool_blocks);
    format_timings(&mut out, &metrics.hook_timings);
    out.push_str("╚══════════════════════════════════════════════╝\n");
    out
}

fn process_signals_once(
    metrics: &mut Metrics,
    cytokine_metrics: &mut CytokineMetrics,
) -> std::io::Result<usize> {
    let signals = read_signals(Some(SIGNAL_PATH))?;
    let count = signals.len();
    let mut has_cytokines = false;

    for signal in &signals {
        metrics.process_signal(signal);
        if CytokineMetrics::is_cytokine(signal) {
            cytokine_metrics.process_cytokine(signal);
            has_cytokines = true;
        }
    }

    if count > 0 {
        let _ = rotate_signals();
        metrics.save()?;
        if has_cytokines {
            let _ = cytokine_metrics.save();
        }
    }

    Ok(count)
}

fn run_daemon() {
    eprintln!("Signal receiver started. Watching: {}", SIGNAL_PATH);
    let mut metrics = Metrics::load();
    let mut cytokine_metrics = CytokineMetrics::load();
    let poll = Duration::from_millis(POLL_INTERVAL_MS);

    loop {
        match process_signals_once(&mut metrics, &mut cytokine_metrics) {
            Ok(n) if n > 0 => eprintln!("Processed {} signals", n),
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                eprintln!("Error: {}", e);
            }
            _ => {}
        }
        std::thread::sleep(poll);
    }
}

fn cytokine_summary(cm: &CytokineMetrics) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════╗\n");
    out.push_str("║         CYTOKINE SIGNAL METRICS              ║\n");
    out.push_str("╠══════════════════════════════════════════════╣\n");
    out.push_str(&format!("║ Total Cytokines: {:>25} ║\n", cm.total));
    out.push_str(&format!(
        "║ Activating (pro-inflam): {:>17} ║\n",
        cm.activating_count
    ));
    out.push_str(&format!(
        "║ Suppressing (anti-inflam): {:>15} ║\n",
        cm.suppressing_count
    ));
    out.push_str("╠══════════════════════════════════════════════╣\n");
    out.push_str("║ BY FAMILY                                    ║\n");
    for (family, count) in &cm.by_family {
        let name = truncate_name(family, 30);
        out.push_str(&format!("║   {:<30} {:>10} ║\n", name, count));
    }
    if !cm.by_hook.is_empty() {
        out.push_str("╠══════════════════════════════════════════════╣\n");
        out.push_str("║ BY HOOK                                      ║\n");
        for (hook, count) in &cm.by_hook {
            let name = truncate_name(hook, 30);
            out.push_str(&format!("║   {:<30} {:>10} ║\n", name, count));
        }
    }
    if !cm.by_severity.is_empty() {
        out.push_str("╠══════════════════════════════════════════════╣\n");
        out.push_str("║ BY SEVERITY                                  ║\n");
        for (sev, count) in &cm.by_severity {
            let name = truncate_name(sev, 30);
            out.push_str(&format!("║   {:<30} {:>10} ║\n", name, count));
        }
    }
    out.push_str("╚══════════════════════════════════════════════╝\n");
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("--once") => {
            let mut metrics = Metrics::load();
            let mut cytokine_metrics = CytokineMetrics::load();
            match process_signals_once(&mut metrics, &mut cytokine_metrics) {
                Ok(n) => println!("Processed {} signals", n),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Some("--summary") => {
            let metrics = Metrics::load();
            println!("{}", summary(&metrics));
        }
        Some("--cytokines") => {
            let cm = CytokineMetrics::load();
            println!("{}", cytokine_summary(&cm));
        }
        Some("--help") | Some("-h") => {
            println!("Signal Receiver - Processes neurotransmitter signals\n");
            println!("  signal-receiver              Run as daemon");
            println!("  signal-receiver --once       Process once");
            println!("  signal-receiver --summary    Show metrics");
            println!("  signal-receiver --cytokines  Show cytokine metrics");
        }
        _ => run_daemon(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_hook_lib::neurotransmitter::Signal;

    fn make_cytokine_signal(signal_type: &str, severity: &str, source: &str) -> Signal {
        Signal::new(signal_type)
            .with_data("severity", severity)
            .with_data("source", source)
            .with_data("family", signal_type.split(':').nth(1).unwrap_or("unknown"))
    }

    #[test]
    fn test_cytokine_signal_detection() {
        let cytokine = Signal::new("cytokine:tnf_alpha:hook_blocked:test");
        let regular = Signal::new("skill_invoked");
        let also_regular = Signal::new("tool_blocked");

        assert!(CytokineMetrics::is_cytokine(&cytokine));
        assert!(!CytokineMetrics::is_cytokine(&regular));
        assert!(!CytokineMetrics::is_cytokine(&also_regular));
    }

    #[test]
    fn test_family_aggregation() {
        let mut cm = CytokineMetrics::default();

        // 5 TNF-alpha
        for i in 0..5 {
            let sig = make_cytokine_signal(
                &format!("cytokine:tnf_alpha:blocked:{i}"),
                "high",
                "hook-lib",
            );
            cm.process_cytokine(&sig);
        }

        // 3 IL-6
        for i in 0..3 {
            let sig = make_cytokine_signal(
                &format!("cytokine:il6:check_failed:{i}"),
                "high",
                "hook-lib",
            );
            cm.process_cytokine(&sig);
        }

        assert_eq!(cm.by_family.get("tnf_alpha").copied().unwrap_or(0), 5);
        assert_eq!(cm.by_family.get("il6").copied().unwrap_or(0), 3);
        assert_eq!(cm.total, 8);
        assert_eq!(cm.activating_count, 8); // both are pro-inflammatory
        assert_eq!(cm.suppressing_count, 0);
    }

    #[test]
    fn test_recent_buffer_bounded() {
        let mut cm = CytokineMetrics::default();

        // Insert 150 signals (over the 100 limit)
        for i in 0..150 {
            let sig = make_cytokine_signal(&format!("cytokine:il2:skill:{i}"), "low", "hook-lib");
            cm.process_cytokine(&sig);
        }

        assert_eq!(cm.recent.len(), RECENT_CYTOKINE_LIMIT);
        assert_eq!(cm.total, 150);
    }

    #[test]
    fn test_mixed_signal_processing() {
        let mut metrics = Metrics::default();
        let mut cm = CytokineMetrics::default();

        let signals = vec![
            Signal::new("skill_invoked").with_data("skill", "forge"),
            Signal::new("cytokine:tnf_alpha:blocked:test")
                .with_data("severity", "critical")
                .with_data("source", "unwrap-guardian"),
            Signal::new("tool_blocked")
                .with_data("hook", "panic-detector")
                .with_data("tool", "Write"),
            Signal::new("cytokine:tgf_beta:completed:test")
                .with_data("severity", "trace")
                .with_data("source", "compile-verifier"),
        ];

        for signal in &signals {
            metrics.process_signal(signal);
            if CytokineMetrics::is_cytokine(signal) {
                cm.process_cytokine(signal);
            }
        }

        // Regular metrics processed all 4
        assert_eq!(metrics.signals_processed, 4);

        // Cytokine metrics only got the 2 cytokine-prefixed ones
        assert_eq!(cm.total, 2);
        assert_eq!(cm.by_family.get("tnf_alpha").copied().unwrap_or(0), 1);
        assert_eq!(cm.by_family.get("tgf_beta").copied().unwrap_or(0), 1);
        assert_eq!(cm.activating_count, 1); // tnf_alpha
        assert_eq!(cm.suppressing_count, 1); // tgf_beta
    }
}
