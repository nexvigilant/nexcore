//! Fluent builder for `StationConfig` — type-safe rail construction.

use crate::config::{AccessTier, ExecutionType, PvVertical, StationConfig, StationTool};
use crate::disclaimer::with_disclaimer;

/// Builder for constructing a `StationConfig` with fluent API.
#[derive(Debug, Clone)]
pub struct StationBuilder {
    vertical: PvVertical,
    title: String,
    description: String,
    tools: Vec<StationTool>,
    tags: Vec<String>,
    contributor: String,
}

impl StationBuilder {
    /// Start building a config for the given vertical.
    pub fn new(vertical: PvVertical, title: impl Into<String>) -> Self {
        Self {
            vertical,
            title: title.into(),
            description: String::new(),
            tools: Vec::new(),
            tags: Vec::new(),
            contributor: "MatthewCampCorp".to_string(),
        }
    }

    /// Set the config description (disclaimer auto-appended).
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Override the contributor name.
    pub fn contributor(mut self, name: impl Into<String>) -> Self {
        self.contributor = name.into();
        self
    }

    /// Add a tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags.
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Add an extraction tool (read-only data scraping).
    pub fn extract_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        route: impl Into<String>,
    ) -> Self {
        self.tools.push(StationTool {
            name: name.into(),
            description: description.into(),
            route: route.into(),
            execution_type: ExecutionType::Extract,
            access_tier: AccessTier::Public,
            input_schema: None,
            tags: Vec::new(),
        });
        self
    }

    /// Add a navigation tool.
    pub fn navigate_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        route: impl Into<String>,
    ) -> Self {
        self.tools.push(StationTool {
            name: name.into(),
            description: description.into(),
            route: route.into(),
            execution_type: ExecutionType::Navigate,
            access_tier: AccessTier::Public,
            input_schema: None,
            tags: Vec::new(),
        });
        self
    }

    /// Add a fill tool (form input).
    pub fn fill_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        route: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        self.tools.push(StationTool {
            name: name.into(),
            description: description.into(),
            route: route.into(),
            execution_type: ExecutionType::Fill,
            access_tier: AccessTier::Public,
            input_schema: Some(input_schema),
            tags: Vec::new(),
        });
        self
    }

    /// Add a click tool (trigger action and read result).
    pub fn click_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        route: impl Into<String>,
    ) -> Self {
        self.tools.push(StationTool {
            name: name.into(),
            description: description.into(),
            route: route.into(),
            execution_type: ExecutionType::Click,
            access_tier: AccessTier::Public,
            input_schema: None,
            tags: Vec::new(),
        });
        self
    }

    /// Add a raw tool with full control.
    pub fn tool(mut self, tool: StationTool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Build the config.
    pub fn build(self) -> StationConfig {
        StationConfig {
            id: None,
            domain: self.vertical.domain().to_string(),
            vertical: self.vertical,
            title: self.title,
            description: with_disclaimer(&self.description),
            tools: self.tools,
            tags: self.tags,
            contributor: self.contributor,
        }
    }

    /// Build and emit as MoltBrowser `contribute_create-config` JSON payload.
    pub fn to_moltbrowser_create(&self) -> serde_json::Value {
        let config = self.clone().build();
        serde_json::json!({
            "domain": config.domain,
            "urlPattern": format!("{}/*", config.domain),
            "title": config.title,
            "description": config.description,
        })
    }

    /// Emit all tools as MoltBrowser `contribute_add-tool` JSON payloads.
    /// Each payload includes a placeholder `configId` to be replaced after creation.
    pub fn to_moltbrowser_tools(&self, config_id: &str) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .map(|tool| {
                let mut payload = serde_json::json!({
                    "configId": config_id,
                    "name": tool.name,
                    "description": tool.description,
                });
                if let (Some(schema), Some(obj)) = (&tool.input_schema, payload.as_object_mut()) {
                    obj.insert("inputSchema".to_string(), schema.clone());
                }
                payload
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_dailymed_config() {
        let config = StationBuilder::new(PvVertical::DailyMed, "DailyMed Drug Labels")
            .description("Extract drug labeling data from DailyMed")
            .tag("pharmacovigilance")
            .tag("drug-labels")
            .extract_tool(
                "get-adverse-reactions",
                "Extract adverse reactions section from a drug label",
                "/drugInfo.cfm",
            )
            .navigate_tool(
                "search-drugs",
                "Search DailyMed for a drug by name",
                "/search.cfm",
            )
            .build();

        assert_eq!(config.domain, "dailymed.nlm.nih.gov");
        assert_eq!(config.vertical, PvVertical::DailyMed);
        assert_eq!(config.total_tools(), 2);
        assert_eq!(config.public_tool_count(), 2);
        assert!(config.description.contains("DISCLAIMER"));
        assert!(config.description.contains("NexVigilant"));
    }

    #[test]
    fn moltbrowser_create_payload() {
        let builder = StationBuilder::new(PvVertical::Faers, "FDA FAERS")
            .description("Adverse event reports from FDA FAERS");

        let payload = builder.to_moltbrowser_create();
        assert_eq!(payload["domain"], "api.fda.gov");
        assert!(payload["title"].as_str().is_some());
    }

    #[test]
    fn moltbrowser_tool_payloads() {
        let builder = StationBuilder::new(PvVertical::PubMed, "PubMed Literature")
            .description("Search PV literature")
            .extract_tool("get-abstract", "Get article abstract", "/")
            .navigate_tool("search-articles", "Search PubMed", "/");

        let tools = builder.to_moltbrowser_tools("config-123");
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0]["configId"], "config-123");
        assert_eq!(tools[0]["name"], "get-abstract");
    }

    #[test]
    fn all_verticals_have_domains() {
        for v in PvVertical::all() {
            assert!(!v.domain().is_empty(), "vertical {v:?} has empty domain");
        }
    }
}
