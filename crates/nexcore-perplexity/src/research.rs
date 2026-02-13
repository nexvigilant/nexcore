// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! High-level research query builders for three domain-specific use cases.
//!
//! ## Use Cases
//!
//! 1. **General Research** -- Open-ended web research, no domain filter
//! 2. **Competitive Intelligence** -- Filter to competitor domains + industry sites
//! 3. **Regulatory/Landscape** -- Filter to regulatory domains (FDA, EMA, ICH, WHO)
//!
//! ## Primitive Grounding
//!
//! Research functions compose: mu (query mapping) + boundary (domain filters) + causality (citations).
//! The three use cases differ in their boundary constraints.

use crate::client::PerplexityClient;
use crate::error::PerplexityResult;
use crate::types::{ChatRequest, Model, ResearchResult, SearchRecency};

// ============================================================================
// System Prompts
// ============================================================================

const GENERAL_SYSTEM_PROMPT: &str = "You are a thorough research assistant. Provide comprehensive, well-sourced answers. \
     Always cite specific sources. Organize findings clearly with headings when appropriate.";

const COMPETITIVE_SYSTEM_PROMPT: &str = "You are a competitive intelligence analyst. Focus on: market positioning, product features, \
     pricing strategies, recent announcements, partnerships, and strategic moves. \
     Be specific with dates and data points. Cite all sources.";

const REGULATORY_SYSTEM_PROMPT: &str = "You are a regulatory intelligence specialist focused on pharmaceutical and healthcare regulations. \
     Focus on: guideline updates, regulatory decisions, safety communications, approval actions, \
     and compliance requirements. Cite specific regulatory documents with dates.";

// ============================================================================
// Regulatory Domains
// ============================================================================

/// Standard regulatory domains for pharmaceutical intelligence.
const REGULATORY_DOMAINS: &[&str] = &[
    "fda.gov",
    "ema.europa.eu",
    "ich.org",
    "who.int",
    "accessdata.fda.gov",
    "clinicaltrials.gov",
    "drugs.com",
    "drugbank.com",
    "pmda.go.jp",
    "gov.uk",
];

// ============================================================================
// Research Functions
// ============================================================================

/// General web research -- no domain filter, broad search.
///
/// Best for: open-ended questions, technology research, general knowledge.
pub async fn research_general(
    client: &PerplexityClient,
    query: &str,
    model: Option<Model>,
) -> PerplexityResult<ResearchResult> {
    let model = model.unwrap_or(client.default_model());
    let request = ChatRequest::simple(model, query).with_system_prompt(GENERAL_SYSTEM_PROMPT);

    let response = client.chat(request).await?;
    Ok(ResearchResult::from_response(&response, model))
}

/// Competitive intelligence -- filter to competitor domains + industry sites.
///
/// Best for: competitor analysis, market research, strategic intelligence.
pub async fn research_competitive(
    client: &PerplexityClient,
    query: &str,
    competitor_domains: &[String],
    model: Option<Model>,
) -> PerplexityResult<ResearchResult> {
    let model = model.unwrap_or(Model::SonarPro); // Default to Pro for competitive intel

    let mut request =
        ChatRequest::simple(model, query).with_system_prompt(COMPETITIVE_SYSTEM_PROMPT);

    if !competitor_domains.is_empty() {
        request = request.with_domain_filter(competitor_domains.to_vec());
    }

    // Competitive intel benefits from recent data
    request = request.with_recency(SearchRecency::Month);

    let response = client.chat(request).await?;
    Ok(ResearchResult::from_response(&response, model))
}

/// Regulatory/landscape intelligence -- filter to regulatory domains.
///
/// Best for: FDA actions, EMA guidelines, ICH harmonization, WHO reports.
pub async fn research_regulatory(
    client: &PerplexityClient,
    query: &str,
    recency: Option<SearchRecency>,
    model: Option<Model>,
) -> PerplexityResult<ResearchResult> {
    let model = model.unwrap_or(Model::SonarPro); // Default to Pro for regulatory depth

    let domains: Vec<String> = REGULATORY_DOMAINS.iter().map(|d| d.to_string()).collect();

    let mut request = ChatRequest::simple(model, query)
        .with_system_prompt(REGULATORY_SYSTEM_PROMPT)
        .with_domain_filter(domains);

    if let Some(r) = recency {
        request = request.with_recency(r);
    } else {
        // Default to month for regulatory to catch recent updates
        request = request.with_recency(SearchRecency::Month);
    }

    let response = client.chat(request).await?;
    Ok(ResearchResult::from_response(&response, model))
}

// ============================================================================
// Research Use Case Enum (for MCP routing)
// ============================================================================

/// Research use case selector for MCP tool routing.
///
/// Tier: T2-P (kappa Comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchUseCase {
    /// General web research.
    General,
    /// Competitive intelligence.
    Competitive,
    /// Regulatory/landscape intelligence.
    Regulatory,
}

impl ResearchUseCase {
    /// Parse from string input.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "general" | "research" | "web" => Some(Self::General),
            "competitive" | "competitor" | "market" => Some(Self::Competitive),
            "regulatory" | "regulation" | "landscape" | "pharma" => Some(Self::Regulatory),
            _ => None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_use_case_parsing() {
        assert_eq!(
            ResearchUseCase::from_str_opt("general"),
            Some(ResearchUseCase::General)
        );
        assert_eq!(
            ResearchUseCase::from_str_opt("competitive"),
            Some(ResearchUseCase::Competitive)
        );
        assert_eq!(
            ResearchUseCase::from_str_opt("regulatory"),
            Some(ResearchUseCase::Regulatory)
        );
        assert_eq!(
            ResearchUseCase::from_str_opt("pharma"),
            Some(ResearchUseCase::Regulatory)
        );
        assert_eq!(ResearchUseCase::from_str_opt("invalid"), None);
    }

    #[test]
    fn regulatory_domains_are_nonempty() {
        assert!(
            !REGULATORY_DOMAINS.is_empty(),
            "regulatory domains list must not be empty"
        );
        assert!(
            REGULATORY_DOMAINS.contains(&"fda.gov"),
            "fda.gov must be in regulatory domains"
        );
    }

    #[test]
    fn system_prompts_are_nonempty() {
        assert!(!GENERAL_SYSTEM_PROMPT.is_empty());
        assert!(!COMPETITIVE_SYSTEM_PROMPT.is_empty());
        assert!(!REGULATORY_SYSTEM_PROMPT.is_empty());
    }
}
