//! Station registry — tracks all configs owned by NexVigilant on the hub.

use crate::config::{PvVertical, StationConfig};
use serde::{Deserialize, Serialize};

/// Registry of all NexVigilant station configs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StationRegistry {
    /// All configs in the registry.
    pub configs: Vec<StationConfig>,
}

impl StationRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a config to the registry.
    pub fn add(&mut self, config: StationConfig) {
        self.configs.push(config);
    }

    /// Find configs by vertical.
    pub fn by_vertical(&self, vertical: PvVertical) -> Vec<&StationConfig> {
        self.configs
            .iter()
            .filter(|c| c.vertical == vertical)
            .collect()
    }

    /// Find config by domain.
    pub fn by_domain(&self, domain: &str) -> Option<&StationConfig> {
        self.configs.iter().find(|c| c.domain == domain)
    }

    /// Total configs.
    pub fn config_count(&self) -> usize {
        self.configs.len()
    }

    /// Total tools across all configs.
    pub fn total_tools(&self) -> usize {
        self.configs.iter().map(|c| c.total_tools()).sum()
    }

    /// All verticals that have configs.
    pub fn covered_verticals(&self) -> Vec<PvVertical> {
        let mut verticals: Vec<PvVertical> = self.configs.iter().map(|c| c.vertical).collect();
        verticals.sort_by_key(|v| *v as u8);
        verticals.dedup();
        verticals
    }

    /// Verticals NOT yet covered.
    pub fn uncovered_verticals(&self) -> Vec<PvVertical> {
        let covered = self.covered_verticals();
        PvVertical::all()
            .iter()
            .filter(|v| !covered.contains(v))
            .copied()
            .collect()
    }

    /// Coverage ratio (configs / total verticals).
    pub fn coverage_ratio(&self) -> f64 {
        let total = PvVertical::all().len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        self.covered_verticals().len() as f64 / total
    }

    /// Serialize to JSON for persistence.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StationBuilder;

    #[test]
    fn registry_coverage_tracking() {
        let mut reg = StationRegistry::new();
        assert_eq!(reg.coverage_ratio(), 0.0);
        assert_eq!(reg.uncovered_verticals().len(), PvVertical::all().len());

        reg.add(
            StationBuilder::new(PvVertical::DailyMed, "DailyMed")
                .description("test")
                .build(),
        );
        reg.add(
            StationBuilder::new(PvVertical::Faers, "FAERS")
                .description("test")
                .extract_tool("t1", "d", "/")
                .extract_tool("t2", "d", "/")
                .build(),
        );

        assert_eq!(reg.config_count(), 2);
        assert_eq!(reg.total_tools(), 2);
        assert_eq!(reg.covered_verticals().len(), 2);
        assert!(reg.coverage_ratio() > 0.0);
        assert!(reg.by_domain("dailymed.nlm.nih.gov").is_some());
        assert!(reg.by_domain("nonexistent.com").is_none());
    }

    #[test]
    fn registry_roundtrip_json() {
        let mut reg = StationRegistry::new();
        reg.add(
            StationBuilder::new(PvVertical::PubMed, "PubMed")
                .description("lit search")
                .extract_tool("get-abstract", "Get abstract", "/")
                .build(),
        );

        let json = reg.to_json().expect("serialize");
        let restored = StationRegistry::from_json(&json).expect("deserialize");
        assert_eq!(restored.config_count(), 1);
        assert_eq!(restored.total_tools(), 1);
    }
}
