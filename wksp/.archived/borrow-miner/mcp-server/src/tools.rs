//! MCP tool implementations
//!
//! Each tool is a pure function operating on game state

use crate::ore::{calculate_points, ore_symbol, ore_value, roll_ore};
use crate::state::{GameState, GAME_STATE};
use serde_json::Value;

const MAX_INVENTORY: usize = 5;
const MAX_COMBO: u32 = 10;

pub fn mine() -> String {
    let mut lock = GAME_STATE.lock().unwrap();
    let game = lock.as_mut().unwrap();

    if game.inventory.len() >= MAX_INVENTORY {
        return "❌ Inventory full! Use drop_ore first.".into();
    }

    let ore = roll_ore();
    let points = calculate_points(&ore, game.combo, game.depth);
    apply_mine(game, &ore, points);
    format_mine_result(game, &ore, points)
}

fn apply_mine(game: &mut GameState, ore: &str, points: u64) {
    game.score += points;
    game.combo = (game.combo + 1).min(MAX_COMBO);
    game.depth += 0.01;
    game.inventory.push_back(ore.to_string());
    game.last_action = format!("Mined {}", ore);
}

fn format_mine_result(game: &GameState, ore: &str, points: u64) -> String {
    format!(
        "⛏️ Mined: {} (+{} pts)\n📊 Score: {} | Combo: x{}.{} | Depth: {:.2}m\n🎒 Inventory: {}/5",
        ore, points, game.score,
        1 + game.combo / 10, game.combo % 10,
        game.depth, game.inventory.len()
    )
}

pub fn drop_ore() -> String {
    let mut lock = GAME_STATE.lock().unwrap();
    let game = lock.as_mut().unwrap();

    if game.inventory.is_empty() {
        return "❌ No ore to drop!".into();
    }

    let ore = game.inventory.pop_front().unwrap();
    let bonus = ore_value(&ore) / 2;
    apply_drop(game, &ore, bonus);
    format_drop_result(game, &ore, bonus)
}

fn apply_drop(game: &mut GameState, ore: &str, bonus: u64) {
    game.score += bonus;
    game.dropped += 1;
    game.last_action = format!("Dropped {}", ore);
}

fn format_drop_result(game: &GameState, ore: &str, bonus: u64) -> String {
    format!(
        "🗑️ Dropped: {} (+{} bonus)\n📊 Score: {} | Dropped: {}\n🎒 Inventory: {}/5",
        ore, bonus, game.score, game.dropped, game.inventory.len()
    )
}

pub fn get_state() -> String {
    let lock = GAME_STATE.lock().unwrap();
    let game = lock.as_ref().unwrap();
    format_state(game)
}

fn format_state(game: &GameState) -> String {
    let inv: Vec<_> = game.inventory.iter().map(|o| ore_symbol(o)).collect();
    format!(
        "🎮 BORROW MINER\n━━━━━━━━━━━━━━\n📊 Score: {}\n🔥 Combo: x{}.{}\n⬇️ Depth: {:.2}m\n🎒 [{}] ({}/5)\n🗑️ Dropped: {}",
        game.score,
        1 + game.combo / 10, game.combo % 10,
        game.depth,
        inv.join(" "),
        game.inventory.len(),
        game.dropped
    )
}

pub fn signal_check(args: &Value) -> String {
    let drug = args["drug"].as_str().unwrap_or("unknown");
    let event = args["event"].as_str().unwrap_or("unknown");

    let (prr, strength, points) = lookup_signal(drug, event);
    award_signal_points(points);
    format_signal_result(drug, event, prr, strength, points)
}

fn lookup_signal(drug: &str, event: &str) -> (f64, &'static str, u64) {
    let d = drug.to_lowercase();
    let e = event.to_lowercase();

    match (d.as_str(), e.contains("bleed") || e.contains("hemorrhage")) {
        ("aspirin", true) => (3.2, "🟠 MODERATE", 40),
        ("warfarin", true) => (8.7, "🔴 STRONG", 100),
        _ if d == "metformin" && e.contains("lactic") => (5.1, "🔴 STRONG", 100),
        _ if d == "lisinopril" && e.contains("cough") => (2.1, "🟡 WEAK", 15),
        _ => (1.2, "⚪ NONE", 5),
    }
}

fn award_signal_points(points: u64) {
    let mut lock = GAME_STATE.lock().unwrap();
    let game = lock.as_mut().unwrap();
    game.score += points;
}

fn format_signal_result(drug: &str, event: &str, prr: f64, strength: &str, pts: u64) -> String {
    let lock = GAME_STATE.lock().unwrap();
    let score = lock.as_ref().unwrap().score;
    format!(
        "🔬 {} → {}\n{}\nPRR: {:.1} | +{} pts\n📊 Total: {}",
        drug, event, strength, prr, pts, score
    )
}
