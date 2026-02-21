//! Game state management
//!
//! Tier: T2-C (composed state container)

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Mutex;

pub static GAME_STATE: Mutex<Option<GameState>> = Mutex::new(None);

#[derive(Clone, Serialize)]
pub struct GameState {
    pub score: u64,
    pub combo: u32,
    pub depth: f64,
    pub inventory: VecDeque<String>,
    pub dropped: u32,
    pub last_action: String,
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

pub fn init_game() {
    *GAME_STATE.lock().unwrap() = Some(GameState::default());
}
