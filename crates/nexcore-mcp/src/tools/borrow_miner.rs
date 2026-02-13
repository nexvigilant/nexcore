//! Borrow Miner — ore mining game with FDA signal detection.
//!
//! Consolidated from `borrow-miner-mcp` satellite MCP server.
//! 4 tools: mine, drop_ore, get_state, signal_check.
//!
//! Tier: T3 (ς State + N Quantity + κ Comparison + ∂ Boundary)

use crate::params::SignalCheckParams;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// State
// ============================================================================

static GAME_STATE: Mutex<Option<GameState>> = Mutex::new(None);

const MAX_INVENTORY: usize = 5;
const MAX_COMBO: u32 = 10;

struct GameState {
    score: u64,
    combo: u32,
    depth: f64,
    inventory: VecDeque<String>,
    dropped: u32,
    last_action: String,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            combo: 0,
            depth: 1.0,
            inventory: VecDeque::new(),
            dropped: 0,
            last_action: "Game started".into(),
        }
    }
}

fn ensure_game() -> Result<(), McpError> {
    let mut lock = GAME_STATE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    if lock.is_none() {
        *lock = Some(GameState::default());
    }
    Ok(())
}

// ============================================================================
// Ore
// ============================================================================

fn roll_ore() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let roll = (nanos % 100) as f64;
    match roll {
        r if r < 2.0 => "Platinum".into(),
        r if r < 12.0 => "Gold".into(),
        r if r < 30.0 => "Silver".into(),
        r if r < 60.0 => "Copper".into(),
        _ => "Iron".into(),
    }
}

fn ore_value(ore: &str) -> u64 {
    if ore.contains("Platinum") {
        250
    } else if ore.contains("Gold") {
        100
    } else if ore.contains("Silver") {
        50
    } else if ore.contains("Copper") {
        25
    } else {
        10
    }
}

fn ore_symbol(ore: &str) -> &'static str {
    if ore.contains("Platinum") {
        "Pt"
    } else if ore.contains("Gold") {
        "Au"
    } else if ore.contains("Silver") {
        "Ag"
    } else if ore.contains("Copper") {
        "Cu"
    } else {
        "Fe"
    }
}

fn calculate_points(ore: &str, combo: u32, depth: f64) -> u64 {
    let base = ore_value(ore);
    let combo_mult = 1.0 + (combo as f64 * 0.1);
    (base as f64 * combo_mult * depth) as u64
}

// ============================================================================
// Tools
// ============================================================================

/// Mine for ore. Get points based on combo and depth.
pub fn mine() -> Result<CallToolResult, McpError> {
    ensure_game()?;
    let mut lock = GAME_STATE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    let game = lock
        .as_mut()
        .ok_or_else(|| McpError::internal_error("No game state", None))?;

    if game.inventory.len() >= MAX_INVENTORY {
        return Ok(CallToolResult::success(vec![Content::text(
            "Inventory full! Use drop_ore first.",
        )]));
    }

    let ore = roll_ore();
    let points = calculate_points(&ore, game.combo, game.depth);
    game.score += points;
    game.combo = (game.combo + 1).min(MAX_COMBO);
    game.depth += 0.01;
    game.inventory.push_back(ore.clone());
    game.last_action = format!("Mined {ore}");

    let msg = format!(
        "Mined: {} (+{} pts)\nScore: {} | Combo: x{}.{} | Depth: {:.2}m\nInventory: {}/5",
        ore,
        points,
        game.score,
        1 + game.combo / 10,
        game.combo % 10,
        game.depth,
        game.inventory.len()
    );
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Drop oldest ore for bonus points.
pub fn drop_ore() -> Result<CallToolResult, McpError> {
    ensure_game()?;
    let mut lock = GAME_STATE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    let game = lock
        .as_mut()
        .ok_or_else(|| McpError::internal_error("No game state", None))?;

    if game.inventory.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No ore to drop!",
        )]));
    }

    let ore = game.inventory.pop_front().unwrap_or_default();
    let bonus = ore_value(&ore) / 2;
    game.score += bonus;
    game.dropped += 1;
    game.last_action = format!("Dropped {ore}");

    let msg = format!(
        "Dropped: {} (+{} bonus)\nScore: {} | Dropped: {}\nInventory: {}/5",
        ore,
        bonus,
        game.score,
        game.dropped,
        game.inventory.len()
    );
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Get current game state.
pub fn get_state() -> Result<CallToolResult, McpError> {
    ensure_game()?;
    let lock = GAME_STATE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    let game = lock
        .as_ref()
        .ok_or_else(|| McpError::internal_error("No game state", None))?;

    let inv: Vec<_> = game.inventory.iter().map(|o| ore_symbol(o)).collect();
    let msg = format!(
        "BORROW MINER\nScore: {}\nCombo: x{}.{}\nDepth: {:.2}m\nInventory: [{}] ({}/5)\nDropped: {}",
        game.score,
        1 + game.combo / 10,
        game.combo % 10,
        game.depth,
        inv.join(" "),
        game.inventory.len(),
        game.dropped
    );
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Check FDA signal for drug-event pair.
pub fn signal_check(params: SignalCheckParams) -> Result<CallToolResult, McpError> {
    ensure_game()?;

    let d = params.drug.to_lowercase();
    let e = params.event.to_lowercase();

    let (prr, strength, points) =
        match (d.as_str(), e.contains("bleed") || e.contains("hemorrhage")) {
            ("aspirin", true) => (3.2, "MODERATE", 40_u64),
            ("warfarin", true) => (8.7, "STRONG", 100),
            _ if d == "metformin" && e.contains("lactic") => (5.1, "STRONG", 100),
            _ if d == "lisinopril" && e.contains("cough") => (2.1, "WEAK", 15),
            _ => (1.2, "NONE", 5),
        };

    // Award points
    let score = {
        let mut lock = GAME_STATE
            .lock()
            .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
        let game = lock
            .as_mut()
            .ok_or_else(|| McpError::internal_error("No game state", None))?;
        game.score += points;
        game.score
    };

    let msg = format!(
        "{} -> {}\n{}\nPRR: {:.1} | +{} pts\nTotal: {}",
        params.drug, params.event, strength, prr, points, score
    );
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}
