// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Request and response types for the Perplexity AI Sonar API.
//!
//! The Perplexity API uses an OpenAI-compatible chat completions format
//! with extensions for search grounding (citations, search results).
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Dominant | Composition |
//! |------|------|----------|-------------|
//! | `ChatRequest` | T2-C | sigma Sequence | sigma + mu + boundary + kappa |
//! | `ChatResponse` | T3 | mu Mapping | mu + causality + sigma + N + existence |
//! | `SearchResult` | T2-C | causality | causality + lambda + nu |
//! | `Model` | T2-P | kappa Comparison | kappa |
//! | `SearchRecency` | T2-P | nu Frequency | nu |

use serde::{Deserialize, Serialize};

// ============================================================================
// Models
// ============================================================================

/// Perplexity AI model selection.
///
/// Tier: T2-P (kappa Comparison)
/// Two models optimize for different speed/depth tradeoffs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Model {
    /// `sonar` -- Fast, cost-effective search model.
    Sonar,
    /// `sonar-pro` -- Advanced search with deeper analysis.
    SonarPro,
    /// `sonar-deep-research` -- Multi-step research agent.
    SonarDeepResearch,
}

impl Model {
    /// Returns the API model string identifier.
    pub fn as_api_str(&self) -> &'static str {
        match self {
            Self::Sonar => "sonar",
            Self::SonarPro => "sonar-pro",
            Self::SonarDeepResearch => "sonar-deep-research",
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::Sonar
    }
}

/// Parse a model name from user input.
impl Model {
    /// Parse model from string, defaulting to Sonar.
    pub fn from_str_or_default(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sonar-pro" | "pro" => Self::SonarPro,
            "sonar-deep-research" | "deep-research" | "deep" => Self::SonarDeepResearch,
            _ => Self::Sonar,
        }
    }
}

// ============================================================================
// Search Recency Filter
// ============================================================================

/// Search recency filter for Perplexity queries.
///
/// Tier: T2-P (nu Frequency)
/// Controls temporal window of search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchRecency {
    /// Results from the last hour.
    Hour,
    /// Results from the last day.
    Day,
    /// Results from the last week.
    Week,
    /// Results from the last month.
    Month,
}

impl SearchRecency {
    /// Returns the API parameter value.
    pub fn as_api_str(&self) -> &'static str {
        match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }

    /// Parse recency from string input.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hour" | "1h" => Some(Self::Hour),
            "day" | "1d" | "24h" => Some(Self::Day),
            "week" | "1w" | "7d" => Some(Self::Week),
            "month" | "1m" | "30d" => Some(Self::Month),
            _ => None,
        }
    }
}

// ============================================================================
// Chat Request (OpenAI-compatible)
// ============================================================================

/// A message in the chat conversation.
///
/// Tier: T2-P (sigma Sequence element)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role: "system", "user", or "assistant".
    pub role: String,
    /// Message content.
    pub content: String,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
}

/// Chat completion request for the Perplexity API.
///
/// Tier: T2-C (sigma Sequence + mu Mapping + boundary + kappa Comparison)
/// Dominant: sigma -- message list is the core sequence.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    /// Model identifier (e.g., "sonar", "sonar-pro").
    pub model: String,
    /// Message history forming the conversation.
    pub messages: Vec<Message>,
    /// Maximum tokens in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Sampling temperature (0.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Domain filter for search results (e.g., ["fda.gov", "ema.europa.eu"]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_domain_filter: Option<Vec<String>>,
    /// Temporal recency filter for search results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_recency_filter: Option<String>,
    /// Whether to return search results with URLs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_related_questions: Option<bool>,
}

impl ChatRequest {
    /// Create a minimal request with a single user query.
    pub fn simple(model: Model, query: impl Into<String>) -> Self {
        Self {
            model: model.as_api_str().to_string(),
            messages: vec![Message::user(query)],
            max_tokens: None,
            temperature: Some(0.2),
            search_domain_filter: None,
            search_recency_filter: None,
            return_related_questions: Some(true),
        }
    }

    /// Add a system prompt to the request.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.messages.insert(0, Message::system(prompt));
        self
    }

    /// Add domain filter.
    pub fn with_domain_filter(mut self, domains: Vec<String>) -> Self {
        self.search_domain_filter = Some(domains);
        self
    }

    /// Add recency filter.
    pub fn with_recency(mut self, recency: SearchRecency) -> Self {
        self.search_recency_filter = Some(recency.as_api_str().to_string());
        self
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }
}

// ============================================================================
// Chat Response (OpenAI-compatible + Perplexity extensions)
// ============================================================================

/// Chat completion response from the Perplexity API.
///
/// Tier: T3 (mu Mapping + causality + sigma + N + existence)
/// Dominant: mu -- query maps to grounded response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// Unique response identifier.
    pub id: String,
    /// Model that generated the response.
    pub model: String,
    /// Response choices (typically one).
    pub choices: Vec<Choice>,
    /// Citation URLs grounding the response (Perplexity extension).
    #[serde(default)]
    pub citations: Vec<String>,
    /// Related questions suggested by the model.
    #[serde(default)]
    pub related_questions: Vec<String>,
    /// Token usage statistics.
    #[serde(default)]
    pub usage: Usage,
}

impl ChatResponse {
    /// Extract the primary response text.
    pub fn text(&self) -> &str {
        self.choices
            .first()
            .map(|c| c.message.content.as_str())
            .unwrap_or("")
    }

    /// Format response with citations for display.
    pub fn formatted(&self) -> String {
        let mut output = self.text().to_string();

        if !self.citations.is_empty() {
            output.push_str("\n\n**Sources:**\n");
            for (i, url) in self.citations.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, url));
            }
        }

        if !self.related_questions.is_empty() {
            output.push_str("\n**Related Questions:**\n");
            for q in &self.related_questions {
                output.push_str(&format!("- {q}\n"));
            }
        }

        output
    }
}

/// A single choice in the response.
///
/// Tier: T2-P (mu Mapping)
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    /// Index of this choice.
    pub index: u32,
    /// The response message.
    pub message: ChoiceMessage,
    /// Finish reason (e.g., "stop", "length").
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// Message content within a choice.
///
/// Tier: T2-P (mu Mapping)
#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceMessage {
    /// Role (always "assistant" in responses).
    pub role: String,
    /// Response content.
    pub content: String,
}

/// Token usage statistics.
///
/// Tier: T2-P (N Quantity)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt.
    #[serde(default)]
    pub prompt_tokens: u32,
    /// Tokens in the completion.
    #[serde(default)]
    pub completion_tokens: u32,
    /// Total tokens used.
    #[serde(default)]
    pub total_tokens: u32,
}

// ============================================================================
// Search Result (from API response)
// ============================================================================

/// A search result returned by the Perplexity API.
///
/// Tier: T2-C (causality + lambda + nu)
/// Dominant: causality -- the result grounds a claim to a source.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    /// Title of the source document.
    #[serde(default)]
    pub title: String,
    /// URL of the source.
    pub url: String,
    /// Publication date (if available).
    #[serde(default)]
    pub date: Option<String>,
}

// ============================================================================
// Citation (high-level wrapper)
// ============================================================================

/// A formatted citation from the Perplexity response.
///
/// Tier: T2-C (causality + lambda + nu)
/// Dominant: causality -- response is grounded to source URL.
#[derive(Debug, Clone)]
pub struct Citation {
    /// Citation index (1-based).
    pub index: usize,
    /// Source URL.
    pub url: String,
}

// ============================================================================
// Research Result (high-level output)
// ============================================================================

/// High-level research result combining answer + citations + metadata.
///
/// Tier: T3 (mu + causality + sigma + N)
/// Dominant: mu -- the core query-to-answer mapping.
#[derive(Debug, Clone)]
pub struct ResearchResult {
    /// The research answer text.
    pub answer: String,
    /// Citations grounding the answer.
    pub citations: Vec<Citation>,
    /// Model that produced the result.
    pub model_used: Model,
    /// Token usage.
    pub tokens_used: u32,
    /// Related follow-up questions.
    pub related_questions: Vec<String>,
}

impl ResearchResult {
    /// Build from a ChatResponse and model.
    pub fn from_response(response: &ChatResponse, model: Model) -> Self {
        let citations = response
            .citations
            .iter()
            .enumerate()
            .map(|(i, url)| Citation {
                index: i + 1,
                url: url.clone(),
            })
            .collect();

        Self {
            answer: response.text().to_string(),
            citations,
            model_used: model,
            tokens_used: response.usage.total_tokens,
            related_questions: response.related_questions.clone(),
        }
    }

    /// Format for display with citations and metadata.
    pub fn formatted(&self) -> String {
        let mut output = self.answer.clone();

        if !self.citations.is_empty() {
            output.push_str("\n\n**Sources:**\n");
            for c in &self.citations {
                output.push_str(&format!("{}. {}\n", c.index, c.url));
            }
        }

        if !self.related_questions.is_empty() {
            output.push_str("\n**Related Questions:**\n");
            for q in &self.related_questions {
                output.push_str(&format!("- {q}\n"));
            }
        }

        output.push_str(&format!(
            "\n_Model: {} | Tokens: {}_",
            self.model_used.as_api_str(),
            self.tokens_used
        ));

        output
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_api_str() {
        assert_eq!(Model::Sonar.as_api_str(), "sonar");
        assert_eq!(Model::SonarPro.as_api_str(), "sonar-pro");
        assert_eq!(Model::SonarDeepResearch.as_api_str(), "sonar-deep-research");
    }

    #[test]
    fn model_from_str_or_default() {
        assert_eq!(Model::from_str_or_default("pro"), Model::SonarPro);
        assert_eq!(Model::from_str_or_default("sonar-pro"), Model::SonarPro);
        assert_eq!(
            Model::from_str_or_default("deep-research"),
            Model::SonarDeepResearch
        );
        assert_eq!(Model::from_str_or_default("anything"), Model::Sonar);
    }

    #[test]
    fn search_recency_parse() {
        assert_eq!(
            SearchRecency::from_str_opt("hour"),
            Some(SearchRecency::Hour)
        );
        assert_eq!(SearchRecency::from_str_opt("1d"), Some(SearchRecency::Day));
        assert_eq!(
            SearchRecency::from_str_opt("week"),
            Some(SearchRecency::Week)
        );
        assert_eq!(
            SearchRecency::from_str_opt("30d"),
            Some(SearchRecency::Month)
        );
        assert_eq!(SearchRecency::from_str_opt("invalid"), None);
    }

    #[test]
    fn chat_request_simple_builder() {
        let req = ChatRequest::simple(Model::Sonar, "test query");
        assert_eq!(req.model, "sonar");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
        assert_eq!(req.messages[0].content, "test query");
    }

    #[test]
    fn chat_request_with_system_prompt() {
        let req = ChatRequest::simple(Model::SonarPro, "query")
            .with_system_prompt("You are a researcher.");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, "system");
        assert_eq!(req.messages[1].role, "user");
    }

    #[test]
    fn chat_request_with_domain_filter() {
        let req = ChatRequest::simple(Model::Sonar, "query")
            .with_domain_filter(vec!["fda.gov".to_string()]);
        assert_eq!(req.search_domain_filter, Some(vec!["fda.gov".to_string()]));
    }

    #[test]
    fn chat_request_with_recency() {
        let req = ChatRequest::simple(Model::Sonar, "query").with_recency(SearchRecency::Week);
        assert_eq!(req.search_recency_filter, Some("week".to_string()));
    }

    #[test]
    fn chat_response_deserialize() {
        let json = r#"{
            "id": "test-id",
            "model": "sonar",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "The answer is 42."
                    },
                    "finish_reason": "stop"
                }
            ],
            "citations": ["https://example.com/source1"],
            "related_questions": ["What about 43?"],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            }
        }"#;

        let response: ChatResponse =
            serde_json::from_str(json).expect("hardcoded valid JSON in test");
        assert_eq!(response.text(), "The answer is 42.");
        assert_eq!(response.citations.len(), 1);
        assert_eq!(response.citations[0], "https://example.com/source1");
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[test]
    fn chat_response_formatted_includes_citations() {
        let json = r#"{
            "id": "test-id",
            "model": "sonar",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Answer here."
                    }
                }
            ],
            "citations": ["https://fda.gov/drug-approvals", "https://ema.europa.eu/guidance"],
            "related_questions": []
        }"#;

        let response: ChatResponse =
            serde_json::from_str(json).expect("hardcoded valid JSON in test");
        let formatted = response.formatted();
        assert!(formatted.contains("Answer here."));
        assert!(formatted.contains("Sources:"));
        assert!(formatted.contains("1. https://fda.gov/drug-approvals"));
        assert!(formatted.contains("2. https://ema.europa.eu/guidance"));
    }

    #[test]
    fn research_result_from_response() {
        let json = r#"{
            "id": "test",
            "model": "sonar-pro",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "Research answer."}}],
            "citations": ["https://example.com"],
            "related_questions": ["Follow up?"],
            "usage": {"prompt_tokens": 5, "completion_tokens": 10, "total_tokens": 15}
        }"#;

        let response: ChatResponse =
            serde_json::from_str(json).expect("hardcoded valid JSON in test");
        let result = ResearchResult::from_response(&response, Model::SonarPro);

        assert_eq!(result.answer, "Research answer.");
        assert_eq!(result.citations.len(), 1);
        assert_eq!(result.citations[0].index, 1);
        assert_eq!(result.model_used, Model::SonarPro);
        assert_eq!(result.tokens_used, 15);
        assert_eq!(result.related_questions.len(), 1);
    }
}
