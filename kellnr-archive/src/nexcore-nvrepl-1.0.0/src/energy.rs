//! Energy management commands
//!
//! Tier: T3 (wires nexcore-energy ATP/ADP model to terminal output)
//! Dominant primitive: N Quantity (energy is quantitative resource tracking)

use colored::Colorize;
use nexcore_energy::{Operation, Regime, TokenPool, decide, snapshot};

/// Handle energy status query
pub fn handle_energy(budget: u64) -> String {
    let pool = TokenPool::new(budget);
    let state = snapshot(&pool, 0.0);

    let regime_colored = regime_color(&state.regime);

    format!(
        "\n{}\n{}: {} tATP / {} total\n{}: {:.4}\n{}: {}\n{}: {}\n{}: {:.2}%\n{}: {:.2}%\n",
        section_header("ENERGY STATUS"),
        "Pool".bold(),
        pool.t_atp,
        pool.total(),
        "Energy Charge".bold(),
        state.energy_charge,
        "Regime".bold(),
        regime_colored,
        "Strategy".bold(),
        state.recommended_strategy.label(),
        "Waste ratio".bold(),
        state.waste_ratio * 100.0,
        "Burn rate".bold(),
        state.burn_rate * 100.0,
    )
}

/// Handle energy decision for a specific operation
pub fn handle_decide(budget: u64, cost: u64, value: f64) -> String {
    let pool = TokenPool::new(budget);
    let operation = Operation::builder("repl-query")
        .cost(cost)
        .value(value)
        .build();

    let strategy = decide(&pool, &operation);
    let ec = pool.energy_charge();
    let regime = pool.regime();
    let coupling = operation.coupling_ratio();

    format!(
        "\n{}\n{}: {} tATP (EC={:.4}, {})\n{}: cost={}, value={:.1}, coupling={:.2}\n{}: {}\n",
        section_header("ENERGY DECISION"),
        "Pool".bold(),
        pool.t_atp,
        ec,
        regime_color(&regime),
        "Operation".bold(),
        cost,
        value,
        coupling,
        "Recommended".bold(),
        strategy.label(),
    )
}

fn regime_color(regime: &Regime) -> colored::ColoredString {
    match regime {
        Regime::Anabolic => regime.label().green().bold(),
        Regime::Homeostatic => regime.label().blue(),
        Regime::Catabolic => regime.label().yellow().bold(),
        Regime::Crisis => regime.label().red().bold(),
    }
}

fn section_header(title: &str) -> colored::ColoredString {
    format!("=== {title} ===").cyan().bold()
}
