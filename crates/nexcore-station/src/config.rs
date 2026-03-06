//! WebMCP Hub config types — Rust-native rail definitions.
//!
//! Each `StationConfig` represents a config on WebMCP Hub.
//! Each `StationTool` represents a tool within that config.
//! Together they form the discovery layer for AI agents.

use serde::{Deserialize, Serialize};

/// Access tier for a tool — determines auth requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTier {
    /// Public — no auth required. Marketing/discovery lane.
    Public,
    /// Gated — requires NexVigilant membership.
    Gated,
    /// Premium — requires paid tier.
    Premium,
}

/// Execution type matching WebMCP Hub's tool execution model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionType {
    /// Navigate to a URL.
    Navigate,
    /// Fill a form with input parameters.
    Fill,
    /// Click an element to read results.
    Click,
    /// Extract data from the page.
    Extract,
}

/// A single tool within a station config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationTool {
    /// Tool name (kebab-case, unique within config).
    pub name: String,
    /// Human-readable description for agent discovery.
    pub description: String,
    /// The route path this tool targets (e.g., "/nucleus/vigilance/signals").
    pub route: String,
    /// How the tool executes on the target page.
    pub execution_type: ExecutionType,
    /// Access tier — public, gated, or premium.
    pub access_tier: AccessTier,
    /// Input parameter schema (JSON Schema format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// Tags for discovery.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Domain vertical — which PV data domain this config serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PvVertical {
    /// NexVigilant platform tools.
    Platform,
    /// FDA adverse event reporting (openFDA / FAERS).
    Faers,
    /// Drug labeling and SPL data (DailyMed).
    DailyMed,
    /// European pharmacovigilance (EudraVigilance).
    EudraVigilance,
    /// WHO global ICSR database (VigiBase/VigiAccess).
    VigiBase,
    /// Medical terminology (MedDRA).
    MedDra,
    /// Clinical trials safety data.
    ClinicalTrials,
    /// PV literature (PubMed).
    PubMed,
    /// Regulatory harmonization (ICH).
    Ich,
    /// European Medicines Agency.
    Ema,
    /// FDA regulatory (beyond FAERS).
    Fda,
}

impl PvVertical {
    /// The canonical domain for this vertical.
    pub fn domain(&self) -> &'static str {
        match self {
            Self::Platform => "nexvigilant.com",
            Self::Faers => "api.fda.gov",
            Self::DailyMed => "dailymed.nlm.nih.gov",
            Self::EudraVigilance => "www.adrreports.eu",
            Self::VigiBase => "vigiaccess.org",
            Self::MedDra => "www.meddra.org",
            Self::ClinicalTrials => "clinicaltrials.gov",
            Self::PubMed => "pubmed.ncbi.nlm.nih.gov",
            Self::Ich => "www.ich.org",
            Self::Ema => "www.ema.europa.eu",
            Self::Fda => "www.fda.gov",
        }
    }

    /// All verticals in the PV domain.
    pub fn all() -> &'static [Self] {
        &[
            Self::Platform,
            Self::Faers,
            Self::DailyMed,
            Self::EudraVigilance,
            Self::VigiBase,
            Self::MedDra,
            Self::ClinicalTrials,
            Self::PubMed,
            Self::Ich,
            Self::Ema,
            Self::Fda,
        ]
    }
}

/// A station config — one entry on WebMCP Hub representing a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationConfig {
    /// WebMCP Hub config ID (UUID, assigned after creation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The domain this config targets.
    pub domain: String,
    /// Which PV vertical this config serves.
    pub vertical: PvVertical,
    /// Config title for the hub listing.
    pub title: String,
    /// Config description with NexVigilant disclaimer.
    pub description: String,
    /// All tools in this config.
    pub tools: Vec<StationTool>,
    /// Tags for the config.
    pub tags: Vec<String>,
    /// Contributor name on the hub.
    pub contributor: String,
}

impl StationConfig {
    /// Count of public (unauth) tools.
    pub fn public_tool_count(&self) -> usize {
        self.tools
            .iter()
            .filter(|t| t.access_tier == AccessTier::Public)
            .count()
    }

    /// Count of gated (auth-required) tools.
    pub fn gated_tool_count(&self) -> usize {
        self.tools
            .iter()
            .filter(|t| t.access_tier == AccessTier::Gated)
            .count()
    }

    /// Count of premium tools.
    pub fn premium_tool_count(&self) -> usize {
        self.tools
            .iter()
            .filter(|t| t.access_tier == AccessTier::Premium)
            .count()
    }

    /// Total tools.
    pub fn total_tools(&self) -> usize {
        self.tools.len()
    }
}
