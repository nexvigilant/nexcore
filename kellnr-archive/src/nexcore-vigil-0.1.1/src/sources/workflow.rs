//! WorkflowSource: Watches for workflow trigger phrases and activates skill chains.
//!
//! Loads workflow specs from ~/.claude/molecules/ and monitors incoming events
//! for activation triggers. When a trigger matches, emits a `workflow_activate` event.

use crate::events::EventBus;
use crate::models::{Event, Urgency};
use chrono::Utc;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowSpec {
    pub name: String,
    pub description: String,
    pub bond_type: String,
    pub stability: f64,
    pub activation_triggers: Vec<String>,
    pub chain: Vec<ChainLink>,
    #[serde(default)]
    pub dag: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainLink {
    pub order: u32,
    pub skill: String,
    pub role: String,
    pub purpose: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
}

pub struct WorkflowSource {
    bus: Arc<EventBus>,
    molecules: Vec<WorkflowSpec>,
}

impl WorkflowSource {
    pub fn new(bus: Arc<EventBus>) -> Self {
        let molecules = Self::load_molecules();
        info!(count = molecules.len(), "molecule_source_initialized");
        Self { bus, molecules }
    }

    fn load_molecules() -> Vec<WorkflowSpec> {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let molecules_dir = PathBuf::from(home).join(".claude/molecules");

        let mut molecules = Vec::new();

        if let Ok(entries) = fs::read_dir(&molecules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        match serde_json::from_str::<WorkflowSpec>(&content) {
                            Ok(mol) => {
                                info!(name = %mol.name, "molecule_loaded");
                                molecules.push(mol);
                            }
                            Err(e) => {
                                debug!(path = ?path, error = %e, "molecule_parse_failed");
                            }
                        }
                    }
                }
            }
        }

        molecules
    }

    /// Check if input text matches any molecule triggers.
    /// Returns the matching molecule if found.
    pub fn match_trigger(&self, text: &str) -> Option<&WorkflowSpec> {
        let text_lower = text.to_lowercase();

        for molecule in &self.molecules {
            for trigger in &molecule.activation_triggers {
                if text_lower.contains(&trigger.to_lowercase()) {
                    info!(
                        molecule = %molecule.name,
                        trigger = %trigger,
                        "molecule_trigger_matched"
                    );
                    return Some(molecule);
                }
            }
        }

        None
    }

    /// Process an incoming event and check for molecule triggers.
    /// If a trigger matches, emit a molecule_activate event.
    pub async fn process_event(&self, event: &Event) {
        // Extract text from event payload
        let text = event
            .payload
            .get("text")
            .or_else(|| event.payload.get("transcript"))
            .or_else(|| event.payload.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if let Some(molecule) = self.match_trigger(text) {
            let activate_event = Event {
                id: NexId::v4(),
                source: "molecule".to_string(),
                event_type: "molecule_activate".to_string(),
                payload: serde_json::json!({
                    "molecule_name": molecule.name,
                    "chain": molecule.chain,
                    "dag": molecule.dag,
                    "trigger_text": text,
                    "original_event_id": event.id.to_string(),
                }),
                priority: Urgency::High,
                timestamp: Utc::now(),
                correlation_id: Some(event.id.to_string()),
            };

            self.bus.emit(activate_event).await;
        }
    }

    /// Get all loaded molecules.
    pub fn molecules(&self) -> &[WorkflowSpec] {
        &self.molecules
    }

    /// Reload molecules from disk.
    pub fn reload(&mut self) {
        self.molecules = Self::load_molecules();
        info!(count = self.molecules.len(), "molecules_reloaded");
    }
}
