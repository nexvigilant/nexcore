//! Endocrine Loader Hook - SessionStart
//!
//! Loads hormone state, applies decay, reports status.

use nexcore_hormones::{BehavioralModifiers, EndocrineState, HormoneType};

fn main() {
    let mut state = EndocrineState::load();
    state.apply_decay();

    if let Err(e) = state.save() {
        eprintln!("Warning: Could not save hormone state: {e}");
    }

    let modifiers = BehavioralModifiers::from(&state);
    let mood_pct = (state.mood_score() * 100.0) as u32;
    let risk_pct = (modifiers.risk_tolerance * 100.0) as u32;
    let oxytocin_pct = (state.oxytocin.value() * 100.0) as u32;

    let mut status_lines = vec![
        "🧬 **ENDOCRINE STATE**".to_string(),
        "───────────────────────────────────────".to_string(),
    ];

    for hormone in HormoneType::ALL {
        let level = state.get(hormone);
        let pct = (level.value() * 100.0) as u32;
        let bar = level_bar(level.value());
        status_lines.push(format!("{}: {} {}%", hormone.name(), bar, pct));
    }

    status_lines.push("───────────────────────────────────────".to_string());
    status_lines.push(format!(
        "Mood Score: {}% | Risk Tolerance: {}%",
        mood_pct, risk_pct
    ));
    status_lines.push(format!(
        "Partnership Trust: {}% | Sessions: {}",
        oxytocin_pct, state.session_count
    ));

    let mut modes = Vec::new();
    if modifiers.crisis_mode {
        modes.push("⚡ CRISIS MODE");
    }
    if modifiers.partnership_mode {
        modes.push("🤝 PARTNERSHIP MODE");
    }
    if modifiers.rest_recommended {
        modes.push("😴 REST RECOMMENDED");
    }
    if !modes.is_empty() {
        status_lines.push(format!("Active: {}", modes.join(" | ")));
    }
    status_lines.push("───────────────────────────────────────".to_string());

    let message = status_lines.join("\n");
    let output = serde_json::json!({ "sessionContext": message });
    println!("{output}");
}

fn level_bar(value: f64) -> String {
    let filled = (value * 10.0) as usize;
    let empty = 10 - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}
