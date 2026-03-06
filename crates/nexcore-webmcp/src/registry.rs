use std::collections::HashMap;

use crate::config::WebMcpConfig;

/// The NexVigilant WebMCP Registry — source of truth for all configs.
///
/// Configs are indexed by domain for O(1) lookup. Multiple configs
/// can exist per domain (different URL patterns).
#[derive(Debug, Default)]
pub struct Registry {
    /// All configs, indexed by ID.
    configs: HashMap<String, WebMcpConfig>,
    /// Domain → list of config IDs for fast lookup.
    domain_index: HashMap<String, Vec<String>>,
}

impl Registry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Total number of configs in the registry.
    pub fn config_count(&self) -> usize {
        self.configs.len()
    }

    /// Total number of tools across all configs.
    pub fn tool_count(&self) -> usize {
        self.configs.values().map(|c| c.tools.len()).sum()
    }

    /// Number of unique domains.
    pub fn domain_count(&self) -> usize {
        self.domain_index.len()
    }

    /// Insert a config into the registry. Generates an ID if none exists.
    pub fn insert(&mut self, mut config: WebMcpConfig) -> String {
        let id = config.id.clone().unwrap_or_else(|| generate_id());
        config.id = Some(id.clone());

        self.domain_index
            .entry(config.domain.clone())
            .or_default()
            .push(id.clone());

        self.configs.insert(id.clone(), config);
        id
    }

    /// Look up all configs for a domain.
    pub fn lookup(&self, domain: &str) -> Vec<&WebMcpConfig> {
        self.domain_index
            .get(domain)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.configs.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Look up configs matching a domain and URL pattern.
    pub fn lookup_pattern(&self, domain: &str, url: &str) -> Vec<&WebMcpConfig> {
        self.lookup(domain)
            .into_iter()
            .filter(|c| url.contains(&c.url_pattern) || c.url_pattern.contains(domain))
            .collect()
    }

    /// Get a config by ID.
    pub fn get(&self, id: &str) -> Option<&WebMcpConfig> {
        self.configs.get(id)
    }

    /// Remove a config by ID. Returns the removed config if it existed.
    pub fn remove(&mut self, id: &str) -> Option<WebMcpConfig> {
        if let Some(config) = self.configs.remove(id) {
            if let Some(ids) = self.domain_index.get_mut(&config.domain) {
                ids.retain(|i| i != id);
                if ids.is_empty() {
                    self.domain_index.remove(&config.domain);
                }
            }
            Some(config)
        } else {
            None
        }
    }

    /// Validate all configs and return issues per config ID.
    pub fn validate_all(&self) -> HashMap<String, Vec<String>> {
        let mut results = HashMap::new();
        for (id, config) in &self.configs {
            let issues = config.validate();
            if !issues.is_empty() {
                results.insert(id.clone(), issues);
            }
        }
        results
    }

    /// List all domains in the registry.
    pub fn domains(&self) -> Vec<&str> {
        self.domain_index.keys().map(|s| s.as_str()).collect()
    }

    /// Iterate all configs.
    pub fn iter(&self) -> impl Iterator<Item = &WebMcpConfig> {
        self.configs.values()
    }

    /// Load configs from a JSON array.
    pub fn load_json(&mut self, json: &str) -> Result<usize, serde_json::Error> {
        let configs: Vec<WebMcpConfig> = serde_json::from_str(json)?;
        let count = configs.len();
        for config in configs {
            self.insert(config);
        }
        Ok(count)
    }

    /// Export all configs as a JSON array.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let configs: Vec<&WebMcpConfig> = self.configs.values().collect();
        serde_json::to_string_pretty(&configs)
    }
}

/// Generate a simple unique ID (not UUID — sovereignty).
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("nv-{ts:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    fn make_config(domain: &str, pattern: &str, tool_name: &str) -> WebMcpConfig {
        WebMcpConfig {
            id: None,
            domain: domain.into(),
            url_pattern: pattern.into(),
            title: format!("Test — {pattern}"),
            description: "Test config. DISCLAIMER: NexVigilant, LLC test.".into(),
            tags: vec!["test".into()],
            tools: vec![WebMcpTool {
                name: tool_name.into(),
                description: "A test tool for validation purposes".into(),
                input_schema: serde_json::json!({"type": "object", "properties": {}}),
                annotations: ToolAnnotations::default(),
                execution: ToolExecution {
                    steps: vec![ExecutionStep {
                        action: "navigate".into(),
                        url: Some(format!("https://{pattern}")),
                        selector: None,
                        value: None,
                    }],
                    selector: "body".into(),
                    autosubmit: false,
                    result_extract: Some("list".into()),
                },
            }],
        }
    }

    #[test]
    fn test_insert_and_lookup() {
        let mut reg = Registry::new();
        reg.insert(make_config("fda.gov", "fda.gov/safety", "browse-safety"));
        reg.insert(make_config("fda.gov", "fda.gov/drugs", "browse-drugs"));
        reg.insert(make_config("ema.europa.eu", "ema.europa.eu/pv", "browse-pv"));

        assert_eq!(reg.config_count(), 3);
        assert_eq!(reg.tool_count(), 3);
        assert_eq!(reg.domain_count(), 2);
        assert_eq!(reg.lookup("fda.gov").len(), 2);
        assert_eq!(reg.lookup("ema.europa.eu").len(), 1);
        assert_eq!(reg.lookup("who.int").len(), 0);
    }

    #[test]
    fn test_lookup_pattern() {
        let mut reg = Registry::new();
        reg.insert(make_config("fda.gov", "fda.gov/safety", "safety"));
        reg.insert(make_config("fda.gov", "fda.gov/drugs", "drugs"));

        let results = reg.lookup_pattern("fda.gov", "fda.gov/safety/medwatch");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tools[0].name, "safety");
    }

    #[test]
    fn test_remove() {
        let mut reg = Registry::new();
        let id = reg.insert(make_config("fda.gov", "fda.gov/safety", "safety"));
        assert_eq!(reg.config_count(), 1);

        let removed = reg.remove(&id);
        assert!(removed.is_some());
        assert_eq!(reg.config_count(), 0);
        assert_eq!(reg.domain_count(), 0);
    }

    #[test]
    fn test_validate_all_clean() {
        let mut reg = Registry::new();
        reg.insert(make_config("fda.gov", "fda.gov/safety", "browse-safety"));
        let issues = reg.validate_all();
        assert!(issues.is_empty(), "Got issues: {issues:?}");
    }

    #[test]
    fn test_json_roundtrip() {
        let mut reg = Registry::new();
        reg.insert(make_config("fda.gov", "fda.gov/safety", "browse-safety"));
        reg.insert(make_config("ema.europa.eu", "ema.europa.eu/pv", "browse-pv"));

        let json = reg.export_json().expect("export");
        let mut reg2 = Registry::new();
        let count = reg2.load_json(&json).expect("load");
        assert_eq!(count, 2);
        assert_eq!(reg2.config_count(), 2);
    }

    #[test]
    fn test_domains_list() {
        let mut reg = Registry::new();
        reg.insert(make_config("fda.gov", "fda.gov/safety", "safety"));
        reg.insert(make_config("ema.europa.eu", "ema.europa.eu/pv", "pv"));
        let mut domains = reg.domains();
        domains.sort();
        assert_eq!(domains, vec!["ema.europa.eu", "fda.gov"]);
    }

    #[test]
    fn test_generated_ids_unique() {
        let mut reg = Registry::new();
        let id1 = reg.insert(make_config("a.com", "a.com", "t1"));
        let id2 = reg.insert(make_config("b.com", "b.com", "t2"));
        assert_ne!(id1, id2);
    }
}
