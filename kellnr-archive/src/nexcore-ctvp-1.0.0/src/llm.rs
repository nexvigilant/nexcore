//! LLM integration for AI-assisted validation analysis.
//!
//! This module provides utilities for using language models to assist
//! with code analysis, gap detection, and problem discovery.

use crate::error::{CtvpError, CtvpResult};
use crate::five_problems::{Problem, ProblemCategory};
use crate::types::*;
use serde::{Deserialize, Serialize};
use stem_core::{AutonomousLoop, LoopOutcome};

/// Configuration for LLM integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API endpoint (e.g., for Anthropic, OpenAI, or local models)
    pub endpoint: String,

    /// Model identifier
    pub model: String,

    /// Maximum tokens for response
    pub max_tokens: u32,

    /// Temperature for generation
    pub temperature: f32,

    /// API key (from environment by default)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.anthropic.com/v1/messages".into(),
            model: "claude-sonnet-4-20250514".into(),
            max_tokens: 4096,
            temperature: 0.3, // Lower for more deterministic analysis
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
        }
    }
}

impl LlmConfig {
    /// Creates config for Anthropic Claude
    pub fn new_anthropic(model: &str) -> Self {
        Self::new_base_config(
            "https://api.anthropic.com/v1/messages",
            model,
            "ANTHROPIC_API_KEY",
        )
    }

    /// Creates config for local Ollama
    pub fn new_ollama(model: &str) -> Self {
        Self::new_base_config("http://localhost:11434/api/generate", model, "")
    }

    /// Creates config for Google Gemini
    pub fn new_gemini(model: &str) -> Self {
        Self::new_base_config(
            "https://generativelanguage.googleapis.com/v1beta/models",
            model,
            "GEMINI_API_KEY",
        )
    }

    fn new_base_config(endpoint: &str, model: &str, key_env: &str) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
            api_key: Self::get_api_key(key_env),
            ..Default::default()
        }
    }

    fn get_api_key(env_var: &str) -> Option<String> {
        if env_var.is_empty() {
            None
        } else {
            std::env::var(env_var).ok()
        }
    }
}

/// Prompt templates for LLM analysis.
pub struct PromptTemplates;

impl PromptTemplates {
    /// Prompt for Five Problems analysis
    pub fn generate_five_problems_prompt(code_summary: &str, file_list: &[String]) -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            Self::FIVE_PROBLEMS_HEADER,
            Self::format_code_summary(code_summary),
            Self::format_file_list(file_list),
            Self::FIVE_PROBLEMS_TASK
        )
    }

    fn format_code_summary(summary: &str) -> String {
        format!("## Code Summary\n{}", summary)
    }

    fn format_file_list(files: &[String]) -> String {
        format!("## Files\n{}", files.join("\n"))
    }

    const FIVE_PROBLEMS_HEADER: &'static str = "You are a senior software architect performing a Five Problems Analysis using the CTVP (Clinical Trial Validation Paradigm) methodology.";
    const FIVE_PROBLEMS_TASK: &'static str = r#"## Task
Analyze this codebase and identify exactly ONE problem in each of these five categories:
1. Safety (Phase 1 Gap)
2. Efficacy (Phase 2 Gap)
3. Confirmation (Phase 3 Gap)
4. Structural
5. Functional

## Output Format (JSON)
Respond ONLY with valid JSON.
{
  "problems": [
    { "number": 1, "category": "safety", "description": "...", "evidence": "...", "severity": "...", "remediation": "...", "test_required": "..." }
  ],
  "overall_severity": "...",
  "confidence": 0.0
}"#;

    /// Prompt for phase-specific gap analysis
    pub fn generate_phase_gap_prompt(phase: ValidationPhase, evidence_summary: &str) -> String {
        let (name, expected, proves) = Self::get_phase_context(phase);
        format!(
            "Analyzing {} evidence.\nExpected: {}\nProves: {}\n\nEvidence:\n{}\n\n{}",
            name,
            expected,
            proves,
            evidence_summary,
            Self::PHASE_GAP_TASK
        )
    }

    const PHASE_GAP_TASK: &'static str = r#"## Task
1. Assess quality (None/Weak/Moderate/Strong)
2. Identify gaps
3. Recommend actions

## Output Format (JSON)
{
  "quality": "...",
  "quality_rationale": "...",
  "gaps": [{ "description": "...", "impact": "...", "remediation": "..." }],
  "recommendations": ["..."]
}"#;

    /// Prompt for Reality Gradient interpretation
    pub fn generate_reality_gradient_interpretation_prompt(
        score: f64,
        phase_breakdown: &str,
        limiting_factor: Option<ValidationPhase>,
    ) -> String {
        let limiting = limiting_factor
            .map(|p| p.to_string())
            .unwrap_or_else(|| "None".into());
        format!(
            "Score: {:.2}\nBreakdown:\n{}\nLimiting Factor: {}\n\n{}",
            score,
            phase_breakdown,
            limiting,
            Self::REALITY_INTERP_TASK
        )
    }

    const REALITY_INTERP_TASK: &'static str = r#"## Task
1. Plain-language interpretation
2. Risk assessment
3. Prioritized roadmap

## Output Format (JSON)
{
  "interpretation": "...",
  "readiness_level": "...",
  "risks": [{ "description": "...", "likelihood": "...", "impact": "..." }],
  "roadmap": [{ "priority": 1, "action": "...", "effort": "...", "impact_on_score": 0.0 }]
}"#;

    /// Prompt for code-specific analysis
    pub fn generate_code_analysis_prompt(code: &str, category: ProblemCategory) -> String {
        let focus = Self::get_category_focus(category);
        format!(
            "Analyze code for {} issues.\nFocus: {}\n\nCode:\n```\n{}\n```\n\n{}",
            category,
            focus,
            code,
            Self::CODE_ANALYSIS_TASK
        )
    }

    const CODE_ANALYSIS_TASK: &'static str = r#"## Task
Identify specific issues in this category. Be concrete.

## Output Format (JSON)
{
  "issues": [{ "line": "...", "description": "...", "severity": "...", "fix": "..." }],
  "overall_assessment": "..."
}"#;

    fn get_category_focus(category: ProblemCategory) -> &'static str {
        match category {
            ProblemCategory::Safety => "failure modes, resilience patterns",
            ProblemCategory::Efficacy => "capability measurements, logic",
            ProblemCategory::Confirmation => "deployment patterns, scaling",
            ProblemCategory::Structural => "coupling, boundaries",
            ProblemCategory::Functional => "edge cases, input validation",
        }
    }

    fn get_phase_context(phase: ValidationPhase) -> (&'static str, &'static str, &'static str) {
        match phase {
            ValidationPhase::Preclinical => (
                "Preclinical (Phase 0)",
                "unit tests, property tests, static analysis, coverage",
                "mechanism validity under controlled conditions",
            ),
            ValidationPhase::Phase1Safety => (
                "Safety (Phase 1)",
                "fault injection, chaos engineering, circuit breakers, timeouts",
                "graceful failure under stress",
            ),
            ValidationPhase::Phase2Efficacy => (
                "Efficacy (Phase 2)",
                "real data tests, SLO measurements, integration tests",
                "actual capability achievement with production-like data",
            ),
            ValidationPhase::Phase3Confirmation => (
                "Confirmation (Phase 3)",
                "shadow deployment, canary deployment, A/B testing",
                "performance at scale compared to baseline",
            ),
            ValidationPhase::Phase4Surveillance => (
                "Surveillance (Phase 4)",
                "observability, drift detection, continuous validation",
                "ongoing correctness over time",
            ),
        }
    }
}

/// LLM client for CTVP analysis.
pub struct LlmClient {
    config: LlmConfig,
    #[cfg(feature = "llm")]
    client: reqwest::Client,
}

impl LlmClient {
    /// Creates a new LLM client
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "llm")]
            client: reqwest::Client::new(),
        }
    }

    /// Creates client with default Anthropic config
    pub fn anthropic() -> Self {
        Self::new(LlmConfig::default())
    }

    /// Analyzes code for Five Problems with automatic retries
    #[cfg(feature = "llm")]
    pub async fn analyze_five_problems(
        &self,
        code_summary: &str,
        files: &[String],
    ) -> CtvpResult<FiveProblemsResponse> {
        let prompt = PromptTemplates::generate_five_problems_prompt(code_summary, files);
        let mut loop_ctrl = AutonomousLoop::new(3);

        loop_ctrl
            .run(|attempt| self.try_analyze_five_problems(&prompt, attempt))
            .await
    }

    #[cfg(feature = "llm")]
    async fn try_analyze_five_problems(
        &self,
        prompt: &str,
        attempt: u32,
    ) -> Result<FiveProblemsResponse, LoopOutcome<CtvpError>> {
        match self.analyze_generic::<FiveProblemsResponse>(prompt).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::warn!("LLM attempt {} failed: {}", attempt, e);
                Err(LoopOutcome::Retryable(e))
            }
        }
    }

    /// Analyzes a specific phase with automatic retries
    #[cfg(feature = "llm")]
    pub async fn analyze_phase(
        &self,
        phase: ValidationPhase,
        evidence_summary: &str,
    ) -> CtvpResult<PhaseAnalysisResponse> {
        let prompt = PromptTemplates::generate_phase_gap_prompt(phase, evidence_summary);
        let mut loop_ctrl = AutonomousLoop::new(2);

        loop_ctrl
            .run(|attempt| self.try_analyze_phase(&prompt, attempt))
            .await
    }

    #[cfg(feature = "llm")]
    async fn try_analyze_phase(
        &self,
        prompt: &str,
        attempt: u32,
    ) -> Result<PhaseAnalysisResponse, LoopOutcome<CtvpError>> {
        match self.analyze_generic::<PhaseAnalysisResponse>(prompt).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::warn!("LLM phase attempt {} failed: {}", attempt, e);
                Err(LoopOutcome::Retryable(e))
            }
        }
    }

    #[cfg(feature = "llm")]
    async fn analyze_generic<T: serde::de::DeserializeOwned>(&self, prompt: &str) -> CtvpResult<T> {
        let resp = self.execute_completion_request(prompt).await?;
        serde_json::from_str(&resp).map_err(|e| CtvpError::Parse(format!("JSON error: {}", e)))
    }

    /// Sends a completion request
    #[cfg(feature = "llm")]
    async fn execute_completion_request(&self, prompt: &str) -> CtvpResult<String> {
        let key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| CtvpError::Config("No API key".into()))?;
        let body = self.build_anthropic_body(prompt);

        let resp = self
            .client
            .post(&self.config.endpoint)
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        self.parse_anthropic_response(resp.json().await?).await
    }

    #[cfg(feature = "llm")]
    fn build_anthropic_body(&self, prompt: &str) -> serde_json::Value {
        serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": [{"role": "user", "content": prompt}]
        })
    }

    #[cfg(feature = "llm")]
    async fn parse_anthropic_response(&self, json: serde_json::Value) -> CtvpResult<String> {
        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| CtvpError::Parse("Unexpected format".into()))
    }

    /// Stub for non-LLM builds
    #[cfg(not(feature = "llm"))]
    pub fn analyze_five_problems(
        &self,
        _code_summary: &str,
        _files: &[String],
    ) -> CtvpResult<FiveProblemsResponse> {
        Err(CtvpError::Config("LLM feature not enabled".into()))
    }

    /// Stub for non-LLM builds
    #[cfg(not(feature = "llm"))]
    pub fn analyze_phase(
        &self,
        _phase: ValidationPhase,
        _evidence_summary: &str,
    ) -> CtvpResult<PhaseAnalysisResponse> {
        Err(CtvpError::Config("LLM feature not enabled".into()))
    }
}

/// Response from Five Problems analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiveProblemsResponse {
    /// Discovered problems
    pub problems: Vec<LlmProblem>,

    /// Overall severity
    pub overall_severity: String,

    /// Confidence in analysis
    pub confidence: f64,
}

/// Problem from LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProblem {
    /// Problem number
    pub number: u8,

    /// Category
    pub category: String,

    /// Description
    pub description: String,

    /// Evidence
    pub evidence: String,

    /// Severity
    pub severity: String,

    /// Remediation
    pub remediation: String,

    /// Test required
    pub test_required: String,
}

impl TryFrom<LlmProblem> for Problem {
    type Error = CtvpError;

    fn try_from(llm: LlmProblem) -> Result<Self, Self::Error> {
        let category = match llm.category.to_lowercase().as_str() {
            "safety" => ProblemCategory::Safety,
            "efficacy" => ProblemCategory::Efficacy,
            "confirmation" => ProblemCategory::Confirmation,
            "structural" => ProblemCategory::Structural,
            "functional" => ProblemCategory::Functional,
            other => return Err(CtvpError::Parse(format!("Unknown category: {}", other))),
        };

        let severity = match llm.severity.to_lowercase().as_str() {
            "critical" => DiagnosticLevel::Critical,
            "high" => DiagnosticLevel::High,
            "medium" => DiagnosticLevel::Medium,
            "low" => DiagnosticLevel::Low,
            other => return Err(CtvpError::Parse(format!("Unknown severity: {}", other))),
        };

        Ok(
            Problem::new(llm.number, category, llm.description, severity)
                .with_evidence(llm.evidence)
                .with_remediation(llm.remediation)
                .with_test(llm.test_required),
        )
    }
}

/// Response from phase analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseAnalysisResponse {
    /// Evidence quality
    pub quality: String,

    /// Quality rationale
    pub quality_rationale: String,

    /// Gaps identified
    pub gaps: Vec<GapInfo>,

    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Gap information from analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapInfo {
    /// Gap description
    pub description: String,

    /// Impact of gap
    pub impact: String,

    /// How to remediate
    pub remediation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_generation() {
        let prompt = PromptTemplates::generate_five_problems_prompt(
            "A web service that handles user authentication",
            &["src/auth.rs".into(), "src/main.rs".into()],
        );

        assert!(prompt.contains("Five Problems Analysis"));
        assert!(prompt.contains("Safety"));
        assert!(prompt.contains("Efficacy"));
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert!(config.endpoint.contains("anthropic"));
        assert!(config.temperature < 1.0);
    }

    #[test]
    fn test_llm_problem_conversion() -> CtvpResult<()> {
        let llm_problem = LlmProblem {
            number: 1,
            category: "safety".into(),
            description: "Test problem".into(),
            evidence: "Test evidence".into(),
            severity: "high".into(),
            remediation: "Fix it".into(),
            test_required: "Add test".into(),
        };

        let problem: Problem = llm_problem.try_into()?;
        assert_eq!(problem.category, ProblemCategory::Safety);
        assert_eq!(problem.severity, DiagnosticLevel::High);
        Ok(())
    }
}
