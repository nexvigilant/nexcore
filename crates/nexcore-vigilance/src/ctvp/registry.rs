//! CTVP Capability Registry

use super::capability::{Capability, CapabilityTracker};
use super::config::CtvpConfig;
use super::phases::ValidationSummary;
use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Registry errors
#[derive(Debug, Error)]
pub enum RegistryError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parse error
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    /// Capability not found
    #[error("Capability not found: {0}")]
    NotFound(String),
}

/// Persisted event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtvpEvent {
    /// Event type (e.g., "observation", "alert", "validation")
    pub event_type: String,
    /// Capability ID
    pub capability_id: String,
    /// Whether capability was achieved
    pub achieved: bool,
    /// Effect value
    pub value: f64,
    /// Timestamp (Unix seconds)
    pub timestamp: f64,
    /// Session ID for correlation
    pub session_id: Option<String>,
}

/// Central registry for CTVP capabilities
#[derive(Debug)]
pub struct CtvpRegistry {
    config: CtvpConfig,
    capabilities: HashMap<String, CapabilityTracker>,
    summaries: HashMap<String, ValidationSummary>,
}

impl CtvpRegistry {
    /// Creates new registry with default config
    ///
    /// # Returns
    /// New CtvpRegistry instance
    pub fn new() -> Self {
        Self::with_config(CtvpConfig::load())
    }

    /// Creates registry with specific config
    ///
    /// # Arguments
    /// * `config` - Configuration to use
    ///
    /// # Returns
    /// New CtvpRegistry instance
    pub fn with_config(config: CtvpConfig) -> Self {
        Self {
            config,
            capabilities: HashMap::new(),
            summaries: HashMap::new(),
        }
    }

    /// Registers a capability for tracking
    ///
    /// # Arguments
    /// * `capability` - The capability to register
    pub fn register(&mut self, capability: Capability) {
        let id = capability.id.clone();
        let tracker = CapabilityTracker::new(capability);
        self.capabilities.insert(id.clone(), tracker);
        self.summaries
            .insert(id.clone(), ValidationSummary::new(&id));
    }

    /// Gets mutable reference to a capability tracker
    ///
    /// # Arguments
    /// * `id` - Capability ID
    ///
    /// # Returns
    /// Option with mutable reference to tracker
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CapabilityTracker> {
        self.capabilities.get_mut(id)
    }

    /// Gets reference to a capability tracker
    ///
    /// # Arguments
    /// * `id` - Capability ID
    ///
    /// # Returns
    /// Option with reference to tracker
    pub fn get(&self, id: &str) -> Option<&CapabilityTracker> {
        self.capabilities.get(id)
    }

    /// Records an observation for a capability
    ///
    /// # Arguments
    /// * `id` - Capability ID
    /// * `achieved` - Whether capability was achieved
    /// * `value` - Effect value
    ///
    /// # Returns
    /// Result indicating success or error if not found
    pub fn record(&mut self, id: &str, achieved: bool, value: f64) -> Result<(), RegistryError> {
        let tracker = self
            .capabilities
            .get_mut(id)
            .ok_or_else(|| RegistryError::NotFound(id.to_string()))?;
        tracker.record(achieved, value);
        Ok(())
    }

    /// Lists all capability IDs
    ///
    /// # Returns
    /// Iterator over capability IDs
    pub fn capability_ids(&self) -> impl Iterator<Item = &String> {
        self.capabilities.keys()
    }

    /// Returns number of registered capabilities
    ///
    /// # Returns
    /// Count of capabilities
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Returns true if no capabilities registered
    ///
    /// # Returns
    /// True if empty
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Gets the configuration
    ///
    /// # Returns
    /// Reference to config
    pub fn config(&self) -> &CtvpConfig {
        &self.config
    }

    /// Persists an event to the events file
    ///
    /// # Arguments
    /// * `event` - The event to persist
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn persist_event(&self, event: &CtvpEvent) -> Result<(), RegistryError> {
        let path = CtvpConfig::events_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let line = serde_json::to_string(event)? + "\n";
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        file.write_all(line.as_bytes())?;
        Ok(())
    }

    /// Loads events from file
    ///
    /// # Arguments
    /// * `since` - Only events after this timestamp (optional)
    ///
    /// # Returns
    /// Vector of events
    pub fn load_events(&self, since: Option<f64>) -> Result<Vec<CtvpEvent>, RegistryError> {
        let path = CtvpConfig::events_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&path)?;
        let mut events = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(event) = serde_json::from_str::<CtvpEvent>(line) {
                if let Some(s) = since {
                    if event.timestamp < s {
                        continue;
                    }
                }
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Returns data directory path
    ///
    /// # Returns
    /// PathBuf to data directory
    pub fn data_dir(&self) -> PathBuf {
        CtvpConfig::data_dir()
    }

    /// Generates full status report
    ///
    /// # Returns
    /// Formatted report string
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str("\n╔══════════════════════════════════════════════════════════════╗\n");
        r.push_str("║  🔬 CTVP Registry Status                                     ║\n");
        r.push_str("╠══════════════════════════════════════════════════════════════╣\n");

        if self.capabilities.is_empty() {
            r.push_str("║  No capabilities registered                                  ║\n");
        } else {
            for (id, tracker) in &self.capabilities {
                let car = tracker.car();
                let status = if tracker.is_alerting() {
                    "🚨"
                } else if tracker.meets_threshold() {
                    "✅"
                } else if tracker.has_sufficient_data() {
                    "🟡"
                } else {
                    "⏳"
                };
                let display_id = if id.len() > 20 { &id[..20] } else { id };
                r.push_str(&format!(
                    "║  {} {:<20} CAR={:>5.1}% (n={:<4})        ║\n",
                    status,
                    display_id,
                    car * 100.0,
                    tracker.observation_count()
                ));
            }
        }

        r.push_str("╚══════════════════════════════════════════════════════════════╝\n");
        r
    }
}

impl Default for CtvpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let reg = CtvpRegistry::new();
        assert!(reg.is_empty());
    }

    #[test]
    fn test_register_capability() {
        let mut reg = CtvpRegistry::new();
        let cap = Capability::new("Test Capability");
        reg.register(cap);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_record_observation() {
        let mut reg = CtvpRegistry::new();
        reg.register(Capability::new("Test"));
        reg.record("test", true, 1.0).expect("record");
        assert_eq!(reg.get("test").expect("get").observation_count(), 1);
    }

    #[test]
    fn test_not_found_error() {
        let mut reg = CtvpRegistry::new();
        let result = reg.record("nonexistent", true, 1.0);
        assert!(result.is_err());
    }
}
