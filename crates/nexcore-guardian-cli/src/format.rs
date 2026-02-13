//! Response formatting for NVREPL

use colored::Colorize;
use nexcore_vigilance::guardian::{
    OriginatorType, RiskContext, calculate_risk_score,
    homeostasis::{LoopIterationResult, evaluate_pv_risk},
};

pub fn risk_response(context: &RiskContext) -> String {
    let score = calculate_risk_score(context);
    let (_, actions) = evaluate_pv_risk(context);

    let level_color = color_for_level(&score.level);
    let header = "═══ GUARDIAN RISK ASSESSMENT ═══".cyan().bold();

    let mut output = format!(
        "\n{header}\n{}: {} + {}\n{}: {:.1}\n{}: {level_color}\n",
        "Signal".bold(),
        context.drug,
        context.event,
        "Score".bold(),
        score.score.value,
        "Level".bold(),
    );

    append_factors(&mut output, &score.factors);
    append_actions(&mut output, &actions);

    output
}

fn color_for_level(level: &str) -> colored::ColoredString {
    match level {
        "Critical" => level.red().bold(),
        "High" => level.yellow().bold(),
        "Medium" => level.blue(),
        _ => level.green(),
    }
}

fn append_factors(output: &mut String, factors: &[String]) {
    if factors.is_empty() {
        return;
    }

    output.push_str(&format!("\n{}\n", "Contributing Factors:".bold()));
    for factor in factors {
        output.push_str(&format!("  • {factor}\n"));
    }
}

fn append_actions<T: std::fmt::Debug>(output: &mut String, actions: &[T]) {
    if actions.is_empty() {
        return;
    }

    output.push_str(&format!("\n{}\n", "Recommended Actions:".bold()));
    for action in actions {
        output.push_str(&format!("  → {action:?}\n"));
    }
}

pub fn tick_response(result: &LoopIterationResult) -> String {
    format!(
        "\n{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}ms\n",
        "═══ HOMEOSTASIS TICK ═══".cyan().bold(),
        "Iteration".bold(),
        result.iteration_id,
        "Signals".bold(),
        result.signals_detected,
        "Actions".bold(),
        result.actions_taken,
        "Latency".bold(),
        result.duration_ms
    )
}

pub fn status_response(
    sensors: usize,
    actuators: usize,
    iterations: u64,
    commands: usize,
) -> String {
    format!(
        "\n{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n",
        "═══ GUARDIAN STATUS ═══".cyan().bold(),
        "Sensors".bold(),
        sensors,
        "Actuators".bold(),
        actuators,
        "Iterations".bold(),
        iterations,
        "Commands".bold(),
        commands
    )
}

pub fn originator_response(type_str: &str) -> String {
    let originator = parse_originator_type(type_str);
    let Some(o) = originator else {
        return format!("Unknown originator type: {type_str}");
    };

    format_originator(&o)
}

fn parse_originator_type(s: &str) -> Option<OriginatorType> {
    match s.to_lowercase().as_str() {
        "tool" | "t" => Some(OriginatorType::Tool),
        "r" | "agentr" => Some(OriginatorType::AgentWithR),
        "vr" | "agentvr" => Some(OriginatorType::AgentWithVR),
        "gr" | "agentgr" => Some(OriginatorType::AgentWithGR),
        "gvr" | "agentgvr" => Some(OriginatorType::AgentWithGVR),
        _ => None,
    }
}

fn format_originator(o: &OriginatorType) -> String {
    let g = bool_icon(o.has_goal_selection());
    let v = bool_icon(o.has_value_evaluation());
    let r = bool_icon(o.has_refusal_capacity());

    format!(
        "\n{}\n{}: {:?}\n{}: {}\n{}: {g}\n{}: {v}\n{}: {r}\n{}: {:.1}\n",
        "═══ GVR FRAMEWORK ═══".cyan().bold(),
        "Type".bold(),
        o,
        "Description".bold(),
        o.description(),
        "Goal-Selection (G)".bold(),
        "Value-Evaluation (V)".bold(),
        "Refusal-Capacity (R)".bold(),
        "Ceiling Multiplier".bold(),
        o.ceiling_multiplier()
    )
}

fn bool_icon(v: bool) -> colored::ColoredString {
    if v { "✓".green() } else { "✗".red() }
}

pub fn help() -> String {
    format!(
        r#"
{}

{}
  {}  Evaluate PV signal risk
           Example: risk Aspirin Bleeding 3.5 2.1 0.5 2.5 15
  {}              Run homeostasis control loop tick
  {}           Show Guardian status
  {}            Reset Guardian state
  {} Classify entity by GVR framework

{}
  {} Complete signal analysis (PRR/ROR/IC/EBGM/Chi2)
           Example: signal 15 100 20 10000
  {}         Individual PRR calculation
  {}         Individual ROR calculation
  {}          Individual IC calculation
  {}        Individual EBGM calculation

{}
  {}            System health status (GREEN/YELLOW/RED)
  {}  Filter alerts by severity
  {}          List active sensors and actuators
  {}          Run monitoring tick

{}
  {}  Triage by ICH E2A seriousness
           Example: triage fatal
  {}  Resolve priority conflict (P0-P5)
           Example: priority p0 p3
  {}  Show escalation timeline

{}
  {}       Energy status and regime
           Example: energy 10000
  {}  Recommend strategy for operation
           Example: decide 10000 500 3.5

{}
  {}             Show this help
  {}             Exit REPL

{}  All responses deterministic (<10ms). Tab completion enabled.
"#,
        "═══ NVREPL — NexVigilant Terminal ═══".cyan().bold(),
        "Guardian:".bold(),
        "risk <drug> <event> <prr> <ror> <ic025> <eb05> <n>".yellow(),
        "tick".yellow(),
        "status".yellow(),
        "reset".yellow(),
        "originator <type>".yellow(),
        "Signal Detection:".bold(),
        "signal <a> <b> <c> <d>".yellow(),
        "prr <a> <b> <c> <d>".yellow(),
        "ror <a> <b> <c> <d>".yellow(),
        "ic <a> <b> <c> <d>".yellow(),
        "ebgm <a> <b> <c> <d>".yellow(),
        "Monitoring:".bold(),
        "health".yellow(),
        "alerts [severity]".yellow(),
        "sensors".yellow(),
        "montick".yellow(),
        "Patient Safety:".bold(),
        "triage <seriousness>".yellow(),
        "priority <a> <b>".yellow(),
        "escalation <seriousness>".yellow(),
        "Energy:".bold(),
        "energy <budget>".yellow(),
        "decide <budget> <cost> <value>".yellow(),
        "Meta:".bold(),
        "help".yellow(),
        "exit".yellow(),
        "Note:".bold()
    )
}
