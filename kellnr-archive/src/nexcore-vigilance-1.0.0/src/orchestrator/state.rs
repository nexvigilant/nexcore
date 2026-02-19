//! Session storage for persisting successful DAG patterns.
//!
//! Allows agents to remember high-performing skill chains across sessions.

use super::models::Chain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Persistent storage for session patterns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStore {
    /// Mapping from intent/request patterns to successful chains
    pub successful_patterns: HashMap<String, Chain>,
    /// Path to the storage file
    #[serde(skip)]
    pub storage_path: PathBuf,
}

impl SessionStore {
    /// Create a new session store.
    #[must_use]
    pub fn new(metrics_dir: &Path) -> Self {
        let storage_path = metrics_dir.join("session_patterns.json");
        let mut store = Self {
            successful_patterns: HashMap::new(),
            storage_path,
        };
        store.load();
        store
    }

    /// Load patterns from disk.
    pub fn load(&mut self) {
        if !self.storage_path.exists() {
            return;
        }

        if let Ok(file) = File::open(&self.storage_path) {
            let reader = BufReader::new(file);
            if let Ok(data) = serde_json::from_reader::<_, HashMap<String, Chain>>(reader) {
                self.successful_patterns = data;
            }
        }
    }

    /// Save patterns to disk.
    pub fn save(&self) {
        if let Ok(file) = File::create(&self.storage_path) {
            let _ = serde_json::to_writer_pretty(file, &self.successful_patterns);
        }
    }

    /// Remember a successful chain for a given intent.
    pub fn remember(&mut self, intent: &str, chain: Chain) {
        self.successful_patterns.insert(intent.to_string(), chain);
        self.save();
    }

    /// Retrieve a successful chain for a similar intent.
    #[must_use]
    pub fn recall(&self, intent: &str) -> Option<&Chain> {
        self.successful_patterns.get(intent)
    }

    /// Clear all remembered patterns.
    pub fn clear(&mut self) {
        self.successful_patterns.clear();
        self.save();
    }
}
