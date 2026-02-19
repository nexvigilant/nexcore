//! Live signal detection demo on aspirin-GI bleeding data.
use nexcore_vigilance::primitives::signal::*;

fn main() {
    // Real-world data: Aspirin + GI Bleeding (FAERS-like)
    let table = Table::from_raw(15, 100, 20, 10_000);
    let assoc = Association::new("aspirin", "GI_bleeding");

    println!("┌─────────────────────────────────────────────┐");
    println!("│  SIGNAL DETECTION: {} │", assoc);
    println!("├─────────────────────────────────────────────┤");
    println!("│  {:<43} │", format!("{}", table));
    println!("├─────────────────────────────────────────────┤");

    // PRR with Woolf CI
    if let Some((prr, ci)) = table.prr_with_ci() {
        println!("│  PRR:    {:<34} │", format!("{}", prr));
        println!("│  95% CI: {:<34} │", format!("{}", ci));
        println!("│  CI excludes 1.0: {:<25} │", ci.excludes_null_ratio());
    }

    // ROR
    if let Some((ror, ci)) = table.ror_with_ci() {
        println!("│  ROR:    {:<34} │", format!("{}", ror));
        println!("│  95% CI: {:<34} │", format!("{}", ci));
    }

    // Chi-square
    if let Some(chi) = table.chi_square() {
        println!("│  χ²:     {:<34} │", format!("{:.2}", chi));
    }

    // Full signal
    if let Some(sig) = Signal::from_table_evans(table, Source::known("FAERS")) {
        let strength = SignalStrength::from_ratio(sig.ratio);
        println!("├─────────────────────────────────────────────┤");
        println!("│  RESULT: {:<34} │", sig.detected);
        println!("│  Strength: {:<32} │", strength);
        println!("│  Action warranted: {:<24} │", strength.warrants_action());
    }
    println!("└─────────────────────────────────────────────┘");
}
