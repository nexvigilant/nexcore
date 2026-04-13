//! Source registry — tracks known data sources, their configuration,
//! reliability weights, and health timestamps.

use std::collections::HashMap;

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::error::{PerceptionError, Result};
use crate::types::SourceId;

/// Configuration for a single data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Stable source identifier.
    pub id: SourceId,
    /// Human-readable display name.
    pub name: String,
    /// Base URL for the source API.
    pub base_url: String,
    /// Expected ingestion latency in hours. Sources silent for
    /// `2 × expected_latency_hours` are considered stale.
    pub expected_latency_hours: f64,
    /// Reliability weight (0.0–1.0) used in confidence-weighted fusion.
    /// Higher = more trusted.
    pub reliability_weight: f64,
    /// Whether this source is currently enabled.
    pub enabled: bool,
}

/// Runtime entry in the source registry — config plus liveness state.
#[derive(Debug, Clone)]
struct SourceEntry {
    config: SourceConfig,
    /// Last time a record was successfully received from this source.
    last_seen: Option<DateTime>,
}

/// Registry of all known data sources.
///
/// Loaded from YAML configuration at startup; updated at runtime as
/// connectors report health.
#[derive(Debug, Default)]
pub struct SourceRegistry {
    entries: HashMap<SourceId, SourceEntry>,
}

impl SourceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a registry from a slice of [`SourceConfig`] values.
    pub fn from_configs(configs: Vec<SourceConfig>) -> Self {
        let mut registry = Self::new();
        for cfg in configs {
            registry.entries.insert(
                cfg.id.clone(),
                SourceEntry {
                    config: cfg,
                    last_seen: None,
                },
            );
        }
        registry
    }

    /// Parse a YAML string and build a registry.
    ///
    /// Expected YAML shape:
    /// ```yaml
    /// sources:
    ///   - id: faers
    ///     name: FDA FAERS
    ///     base_url: https://api.fda.gov/drug/event.json
    ///     expected_latency_hours: 1.0
    ///     reliability_weight: 0.85
    ///     enabled: true
    /// ```
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        #[derive(Deserialize)]
        struct RegistryFile {
            sources: Vec<SourceConfig>,
        }

        let file: RegistryFile =
            serde_yaml::from_str(yaml).map_err(|e| PerceptionError::ConfigParse(e.to_string()))?;

        Ok(Self::from_configs(file.sources))
    }

    /// Return the reliability weight for a source.
    ///
    /// Returns `0.5` for unknown sources (neutral weight).
    pub fn reliability_weight(&self, source_id: &SourceId) -> f64 {
        self.entries
            .get(source_id)
            .map(|e| e.config.reliability_weight)
            .unwrap_or(0.5)
    }

    /// Return all source IDs whose last-seen timestamp is absent or older
    /// than `2 × expected_latency_hours` relative to `now`.
    pub fn stale_sources(&self, now: DateTime) -> Vec<SourceId> {
        self.entries
            .values()
            .filter(|entry| {
                if !entry.config.enabled {
                    return false;
                }
                let threshold_secs = (entry.config.expected_latency_hours * 2.0 * 3600.0) as i64;
                match entry.last_seen {
                    None => true,
                    Some(ts) => {
                        let age = now.signed_duration_since(ts).num_seconds();
                        age > threshold_secs
                    }
                }
            })
            .map(|e| e.config.id.clone())
            .collect()
    }

    /// Record a successful ingestion event for `source_id` at `when`.
    pub fn mark_seen(&mut self, source_id: &SourceId, when: DateTime) {
        if let Some(entry) = self.entries.get_mut(source_id) {
            entry.last_seen = Some(when);
        }
    }

    /// Return the config for a source, if registered.
    pub fn get(&self, source_id: &SourceId) -> Option<&SourceConfig> {
        self.entries.get(source_id).map(|e| &e.config)
    }

    /// Return all enabled source configs.
    pub fn enabled_sources(&self) -> Vec<&SourceConfig> {
        self.entries
            .values()
            .filter(|e| e.config.enabled)
            .map(|e| &e.config)
            .collect()
    }

    /// Number of registered sources (enabled + disabled).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if no sources are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_chrono::Duration;

    fn make_config(id: &str, latency_hours: f64, weight: f64) -> SourceConfig {
        SourceConfig {
            id: SourceId::new(id),
            name: id.to_string(),
            base_url: format!("https://example.com/{id}"),
            expected_latency_hours: latency_hours,
            reliability_weight: weight,
            enabled: true,
        }
    }

    #[test]
    fn unknown_source_returns_neutral_weight() {
        let reg = SourceRegistry::new();
        assert!((reg.reliability_weight(&SourceId::new("unknown")) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn known_source_returns_configured_weight() {
        let reg = SourceRegistry::from_configs(vec![make_config("faers", 1.0, 0.85)]);
        assert!((reg.reliability_weight(&SourceId::new("faers")) - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn never_seen_source_is_stale() {
        let reg = SourceRegistry::from_configs(vec![make_config("faers", 1.0, 0.85)]);
        let now = DateTime::now();
        assert!(reg.stale_sources(now).contains(&SourceId::new("faers")));
    }

    #[test]
    fn recently_seen_source_is_not_stale() {
        let mut reg = SourceRegistry::from_configs(vec![make_config("faers", 1.0, 0.85)]);
        let now = DateTime::now();
        reg.mark_seen(&SourceId::new("faers"), now);
        assert!(reg.stale_sources(now).is_empty());
    }

    #[test]
    fn old_source_is_stale() {
        let mut reg = SourceRegistry::from_configs(vec![make_config("faers", 1.0, 0.85)]);
        let now = DateTime::now();
        // last seen 3 hours ago; threshold = 2 × 1.0 h = 2 h → stale
        let old = now - Duration::hours(3);
        reg.mark_seen(&SourceId::new("faers"), old);
        assert!(reg.stale_sources(now).contains(&SourceId::new("faers")));
    }

    #[test]
    fn from_yaml_parses_correctly() {
        let yaml = r#"
sources:
  - id: faers
    name: FDA FAERS
    base_url: https://api.fda.gov/drug/event.json
    expected_latency_hours: 1.0
    reliability_weight: 0.85
    enabled: true
  - id: pubmed
    name: PubMed
    base_url: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/
    expected_latency_hours: 2.0
    reliability_weight: 0.75
    enabled: true
"#;
        let reg = SourceRegistry::from_yaml(yaml).expect("parse failed");
        assert_eq!(reg.len(), 2);
        assert!((reg.reliability_weight(&SourceId::new("faers")) - 0.85).abs() < f64::EPSILON);
    }
}
