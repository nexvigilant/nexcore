//! PV signal detection commands
//!
//! Tier: T3 (wires nexcore-pv-core signal analysis to terminal output)
//! Dominant primitive: ∂ Boundary (signal detection is boundary-guarding)

use colored::Colorize;
use nexcore_pv_core::{
    ContingencyTable, SignalCriteria,
    compat::{calculate_chi_square, calculate_ebgm, calculate_ic, calculate_prr, calculate_ror},
    signals::evaluate_signal_complete,
};

/// Run complete signal analysis on a 2x2 contingency table
pub fn handle_signal(a: u64, b: u64, c: u64, d: u64) -> String {
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let result = evaluate_signal_complete(&table, &criteria);

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n{}: a={}, b={}, c={}, d={} (N={})\n{}: Evans (PRR>=2.0, n>=3, Chi2>=3.841)\n\n",
        section_header("SIGNAL DETECTION — COMPLETE"),
        "Table".bold(),
        a,
        b,
        c,
        d,
        table.total(),
        "Criteria".bold(),
    ));

    // PRR
    append_metric(
        &mut out,
        "PRR",
        result.prr.point_estimate,
        result.prr.is_signal,
    );
    out.push_str(&format!(
        "      CI: [{:.4}, {:.4}]\n",
        result.prr.lower_ci, result.prr.upper_ci
    ));

    // ROR
    append_metric(
        &mut out,
        "ROR",
        result.ror.point_estimate,
        result.ror.is_signal,
    );
    out.push_str(&format!(
        "      CI: [{:.4}, {:.4}]\n",
        result.ror.lower_ci, result.ror.upper_ci
    ));

    // IC
    append_metric(
        &mut out,
        "IC",
        result.ic.point_estimate,
        result.ic.is_signal,
    );
    out.push_str(&format!(
        "      CI: [{:.4}, {:.4}]\n",
        result.ic.lower_ci, result.ic.upper_ci
    ));

    // EBGM
    append_metric(
        &mut out,
        "EBGM",
        result.ebgm.point_estimate,
        result.ebgm.is_signal,
    );
    out.push_str(&format!(
        "      CI: [{:.4}, {:.4}]\n",
        result.ebgm.lower_ci, result.ebgm.upper_ci
    ));

    // Chi-square
    let chi_flag = result.chi_square >= 3.841;
    append_metric(&mut out, "Chi2", result.chi_square, chi_flag);

    // Summary
    let signal_count = [
        result.prr.is_signal,
        result.ror.is_signal,
        result.ic.is_signal,
        result.ebgm.is_signal,
        chi_flag,
    ]
    .iter()
    .filter(|&&s| s)
    .count();

    out.push_str(&format!(
        "\n{}: {}/5 metrics flagged\n",
        "Summary".bold(),
        signal_count
    ));

    if signal_count >= 3 {
        out.push_str(&format!(
            "{}\n",
            "SIGNAL DETECTED — recommend investigation".red().bold()
        ));
    } else if signal_count >= 1 {
        out.push_str(&format!(
            "{}\n",
            "WEAK SIGNAL — monitor and reassess".yellow()
        ));
    } else {
        out.push_str(&format!("{}\n", "NO SIGNAL".green()));
    }

    out
}

/// Calculate individual PRR metric
pub fn handle_prr(a: u64, b: u64, c: u64, d: u64) -> String {
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let result = calculate_prr(&table, &criteria);
    format_single_metric(
        "PRR",
        &result.point_estimate,
        &result.lower_ci,
        &result.upper_ci,
        result.is_signal,
    )
}

/// Calculate individual ROR metric
pub fn handle_ror(a: u64, b: u64, c: u64, d: u64) -> String {
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let result = calculate_ror(&table, &criteria);
    format_single_metric(
        "ROR",
        &result.point_estimate,
        &result.lower_ci,
        &result.upper_ci,
        result.is_signal,
    )
}

/// Calculate individual IC metric
pub fn handle_ic(a: u64, b: u64, c: u64, d: u64) -> String {
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let result = calculate_ic(&table, &criteria);
    format_single_metric(
        "IC",
        &result.point_estimate,
        &result.lower_ci,
        &result.upper_ci,
        result.is_signal,
    )
}

/// Calculate individual EBGM metric
pub fn handle_ebgm(a: u64, b: u64, c: u64, d: u64) -> String {
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let result = calculate_ebgm(&table, &criteria);
    format_single_metric(
        "EBGM",
        &result.point_estimate,
        &result.lower_ci,
        &result.upper_ci,
        result.is_signal,
    )
}

fn format_single_metric(
    name: &str,
    point: &f64,
    lower: &f64,
    upper: &f64,
    is_signal: bool,
) -> String {
    let flag = if is_signal {
        "SIGNAL".red().bold()
    } else {
        "no signal".green().normal()
    };

    format!(
        "\n{}\n{}: {:.4}  CI: [{:.4}, {:.4}]  [{}]\n",
        section_header(name),
        name.bold(),
        point,
        lower,
        upper,
        flag
    )
}

fn append_metric(out: &mut String, name: &str, value: f64, is_signal: bool) {
    let flag = if is_signal {
        "SIGNAL".red().bold()
    } else {
        "---".dimmed().normal()
    };
    out.push_str(&format!(
        "  {:<6} {:.4}  [{}]\n",
        format!("{}:", name).bold(),
        value,
        flag
    ));
}

fn section_header(title: &str) -> colored::ColoredString {
    format!("=== {title} ===").cyan().bold()
}
