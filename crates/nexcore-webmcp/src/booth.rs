use crate::config::WebMcpConfig;
use crate::registry::Registry;

/// The Booth — the public MCP wrapper that agents interact with.
///
/// Agents never see the Rust code. They see the booth: clean tool
/// surfaces that return configs, run lookups, and report station status.
/// The booth is the interface. The registry is the engine.
pub struct Booth {
    registry: Registry,
    station_name: String,
}

/// Result of a booth lookup — what agents receive.
#[derive(Debug, serde::Serialize)]
pub struct LookupResult {
    /// Number of configs found.
    pub config_count: usize,
    /// Number of tools available.
    pub tool_count: usize,
    /// The configs matching the query.
    pub configs: Vec<ConfigSummary>,
}

/// Summary of a config for booth display.
#[derive(Debug, serde::Serialize)]
pub struct ConfigSummary {
    pub id: String,
    pub domain: String,
    pub title: String,
    pub tool_count: usize,
    pub tools: Vec<ToolSummary>,
}

/// Summary of a tool for booth display.
#[derive(Debug, serde::Serialize)]
pub struct ToolSummary {
    pub name: String,
    pub description: String,
    pub read_only: bool,
}

/// Station status — what the health probe returns.
#[derive(Debug, serde::Serialize)]
pub struct StationStatus {
    pub station_name: String,
    pub domains: usize,
    pub configs: usize,
    pub tools: usize,
    pub quality_issues: usize,
    pub domains_list: Vec<String>,
}

impl Booth {
    /// Create a new booth wrapping a registry.
    pub fn new(registry: Registry, station_name: impl Into<String>) -> Self {
        Self {
            registry,
            station_name: station_name.into(),
        }
    }

    /// Look up configs for a domain.
    pub fn lookup(&self, domain: &str) -> LookupResult {
        let configs: Vec<ConfigSummary> = self.registry
            .lookup(domain)
            .into_iter()
            .map(summarize_config)
            .collect();

        let tool_count: usize = configs.iter().map(|c| c.tool_count).sum();

        LookupResult {
            config_count: configs.len(),
            tool_count,
            configs,
        }
    }

    /// Look up configs matching a URL pattern.
    pub fn lookup_url(&self, domain: &str, url: &str) -> LookupResult {
        let configs: Vec<ConfigSummary> = self.registry
            .lookup_pattern(domain, url)
            .into_iter()
            .map(summarize_config)
            .collect();

        let tool_count: usize = configs.iter().map(|c| c.tool_count).sum();

        LookupResult {
            config_count: configs.len(),
            tool_count,
            configs,
        }
    }

    /// Get station status.
    pub fn status(&self) -> StationStatus {
        let issues = self.registry.validate_all();
        let quality_issues: usize = issues.values().map(|v| v.len()).sum();
        let mut domains_list: Vec<String> = self.registry.domains()
            .into_iter()
            .map(String::from)
            .collect();
        domains_list.sort();

        StationStatus {
            station_name: self.station_name.clone(),
            domains: self.registry.domain_count(),
            configs: self.registry.config_count(),
            tools: self.registry.tool_count(),
            quality_issues,
            domains_list,
        }
    }

    /// Get the underlying registry (for sync operations).
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Get a mutable reference to the registry.
    pub fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }
}

fn summarize_config(config: &WebMcpConfig) -> ConfigSummary {
    ConfigSummary {
        id: config.id.clone().unwrap_or_default(),
        domain: config.domain.clone(),
        title: config.title.clone(),
        tool_count: config.tools.len(),
        tools: config.tools.iter().map(|t| ToolSummary {
            name: t.name.clone(),
            description: t.description.clone(),
            read_only: t.annotations.read_only_hint == "true",
        }).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    fn build_station() -> Booth {
        let mut reg = Registry::new();

        // NexVigilant home
        reg.insert(WebMcpConfig {
            id: None,
            domain: "nexvigilant.com".into(),
            url_pattern: "nexvigilant.com".into(),
            title: "NexVigilant Platform".into(),
            description: "PV platform. DISCLAIMER: NexVigilant, LLC.".into(),
            tags: vec!["pv".into(), "nexvigilant".into()],
            tools: vec![
                WebMcpTool {
                    name: "navigate-academy".into(),
                    description: "Navigate to the NexVigilant Academy".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {}}),
                    annotations: ToolAnnotations { read_only_hint: "false".into(), ..Default::default() },
                    execution: ToolExecution {
                        steps: vec![ExecutionStep { action: "navigate".into(), url: Some("https://nexvigilant.com/academy".into()), selector: None, value: None }],
                        selector: "body".into(), autosubmit: false, result_extract: None,
                    },
                },
                WebMcpTool {
                    name: "read-services".into(),
                    description: "Read NexVigilant services listing".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {}}),
                    annotations: ToolAnnotations::default(),
                    execution: ToolExecution {
                        steps: vec![ExecutionStep { action: "navigate".into(), url: Some("https://nexvigilant.com/services".into()), selector: None, value: None }],
                        selector: "body".into(), autosubmit: false, result_extract: Some("list".into()),
                    },
                },
            ],
        });

        // FDA
        reg.insert(WebMcpConfig {
            id: None,
            domain: "fda.gov".into(),
            url_pattern: "fda.gov/safety/medwatch".into(),
            title: "FDA MedWatch by NexVigilant".into(),
            description: "MedWatch. DISCLAIMER: NexVigilant, LLC.".into(),
            tags: vec!["fda".into()],
            tools: vec![WebMcpTool {
                name: "browse-alerts".into(),
                description: "Browse FDA safety alerts and recalls".into(),
                input_schema: serde_json::json!({"type": "object", "properties": {}}),
                annotations: ToolAnnotations::default(),
                execution: ToolExecution {
                    steps: vec![ExecutionStep { action: "navigate".into(), url: Some("https://fda.gov/safety/medwatch".into()), selector: None, value: None }],
                    selector: "body".into(), autosubmit: false, result_extract: Some("list".into()),
                },
            }],
        });

        Booth::new(reg, "NexVigilant Station")
    }

    #[test]
    fn test_booth_lookup() {
        let booth = build_station();
        let result = booth.lookup("nexvigilant.com");
        assert_eq!(result.config_count, 1);
        assert_eq!(result.tool_count, 2);
    }

    #[test]
    fn test_booth_lookup_empty() {
        let booth = build_station();
        let result = booth.lookup("unknown.com");
        assert_eq!(result.config_count, 0);
        assert_eq!(result.tool_count, 0);
    }

    #[test]
    fn test_booth_status() {
        let booth = build_station();
        let status = booth.status();
        assert_eq!(status.station_name, "NexVigilant Station");
        assert_eq!(status.domains, 2);
        assert_eq!(status.configs, 2);
        assert_eq!(status.tools, 3);
        assert_eq!(status.quality_issues, 0);
    }

    #[test]
    fn test_booth_tool_readonly() {
        let booth = build_station();
        let result = booth.lookup("nexvigilant.com");
        let tools = &result.configs[0].tools;
        assert!(!tools[0].read_only); // navigate-academy
        assert!(tools[1].read_only);  // read-services
    }

    #[test]
    fn test_booth_status_serializes() {
        let booth = build_station();
        let status = booth.status();
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("NexVigilant Station"));
        assert!(json.contains("nexvigilant.com"));
    }
}
