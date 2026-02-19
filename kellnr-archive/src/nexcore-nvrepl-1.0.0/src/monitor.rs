//! Monitoring and health commands
//!
//! Tier: T3 (wires nexcore-guardian-engine homeostasis to terminal output)
//! Dominant primitive: σ Sequence (monitoring is sequential sensing)

use colored::Colorize;
use nexcore_guardian_engine::homeostasis::LoopIterationResult;
use nexcore_guardian_engine::sensing::ThreatLevel;

/// Format health status from the latest homeostasis tick
pub fn handle_health(tick_result: &LoopIterationResult) -> String {
    let status = health_status(tick_result);
    let status_colored = match status {
        "GREEN" => status.green().bold(),
        "YELLOW" => status.yellow().bold(),
        _ => status.red().bold(),
    };

    format!(
        "\n{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}ms\n",
        section_header("SYSTEM HEALTH"),
        "Status".bold(),
        status_colored,
        "Signals".bold(),
        tick_result.signals_detected,
        "Actions".bold(),
        tick_result.actions_taken,
        "Latency".bold(),
        tick_result.duration_ms,
    )
}

/// Format alerts from a tick, optionally filtered by severity string
pub fn handle_alerts(tick_result: &LoopIterationResult, severity_filter: Option<&str>) -> String {
    let mut out = format!("\n{}\n", section_header("ALERTS"));

    if tick_result.signals_detected == 0 {
        out.push_str(&format!("{}\n", "No alerts detected.".green()));
        return out;
    }

    // Show actuator results as alert summaries
    let mut shown = 0usize;
    for result in &tick_result.results {
        let matches = match severity_filter {
            Some(filter) => result
                .actuator
                .to_lowercase()
                .contains(&filter.to_lowercase()),
            None => true,
        };
        if matches {
            out.push_str(&format!(
                "  {} {}: {}\n",
                bullet_for_success(result.success),
                result.actuator.bold(),
                if result.success {
                    "resolved"
                } else {
                    "pending"
                },
            ));
            shown += 1;
        }
    }

    if shown == 0 {
        out.push_str(&format!("  {}\n", "No alerts matching filter.".dimmed()));
    }

    out
}

/// Format sensor inventory
pub fn handle_sensors(sensor_count: usize, actuator_count: usize) -> String {
    format!(
        "\n{}\n{}: {}\n{}: {}\n",
        section_header("SENSOR INVENTORY"),
        "Active sensors".bold(),
        sensor_count,
        "Active actuators".bold(),
        actuator_count,
    )
}

/// Format monitoring tick result
pub fn handle_montick(tick_result: &LoopIterationResult) -> String {
    format!(
        "\n{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}ms\n",
        section_header("MONITORING TICK"),
        "Iteration".bold(),
        tick_result.iteration_id,
        "Signals".bold(),
        tick_result.signals_detected,
        "Actions".bold(),
        tick_result.actions_taken,
        "Latency".bold(),
        tick_result.duration_ms,
    )
}

/// Classify severity string for display
pub fn severity_color(sev: &ThreatLevel) -> colored::ColoredString {
    match sev {
        ThreatLevel::Critical => "CRITICAL".red().bold(),
        ThreatLevel::High => "HIGH".yellow().bold(),
        ThreatLevel::Medium => "MEDIUM".blue(),
        ThreatLevel::Low => "LOW".green(),
        ThreatLevel::Info => "INFO".dimmed(),
    }
}

fn health_status(tick: &LoopIterationResult) -> &'static str {
    if tick.signals_detected == 0 {
        "GREEN"
    } else if tick.actions_taken >= tick.signals_detected {
        "YELLOW"
    } else {
        "RED"
    }
}

fn bullet_for_success(success: bool) -> colored::ColoredString {
    if success {
        "  [OK]".green()
    } else {
        " [!!]".red().bold()
    }
}

fn section_header(title: &str) -> colored::ColoredString {
    format!("=== {title} ===").cyan().bold()
}
