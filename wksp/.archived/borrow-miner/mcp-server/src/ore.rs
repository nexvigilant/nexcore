//! Ore types and utilities
//!
//! Tier: T3 (domain-specific)

use std::time::{SystemTime, UNIX_EPOCH};

pub fn roll_ore() -> String {
    let roll = random_percent();
    match roll {
        r if r < 2.0 => "💎 Platinum".into(),
        r if r < 12.0 => "🟡 Gold".into(),
        r if r < 30.0 => "⚪ Silver".into(),
        r if r < 60.0 => "🟠 Copper".into(),
        _ => "⚫ Iron".into(),
    }
}

fn random_percent() -> f64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 100) as f64
}

pub fn ore_value(ore: &str) -> u64 {
    if ore.contains("Platinum") { 250 }
    else if ore.contains("Gold") { 100 }
    else if ore.contains("Silver") { 50 }
    else if ore.contains("Copper") { 25 }
    else { 10 }
}

pub fn ore_symbol(ore: &str) -> &'static str {
    if ore.contains("Platinum") { "💎" }
    else if ore.contains("Gold") { "🟡" }
    else if ore.contains("Silver") { "⚪" }
    else if ore.contains("Copper") { "🟠" }
    else { "⚫" }
}

pub fn calculate_points(ore: &str, combo: u32, depth: f64) -> u64 {
    let base = ore_value(ore);
    let combo_mult = 1.0 + (combo as f64 * 0.1);
    (base as f64 * combo_mult * depth) as u64
}
