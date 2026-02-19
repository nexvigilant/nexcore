//! # CEP Pipeline Orchestration
//!
//! Manages execution of the 8-stage pipeline with state tracking.

use super::feedback::FeedbackSignal;
use super::stages::{
    CompositionResult, DeploymentRecord, ExtractionResult, ImprovementPlan, PerceptionRecord,
    StageId, ValidationReport, VocabularyRecord,
};
use crate::domain_discovery::TranslationRecord;
use serde::{Deserialize, Serialize};

/// Pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Source domain for extraction.
    pub source_domain: String,
    /// Target domain for translation (optional).
    #[serde(default)]
    pub target_domain: Option<String>,
    /// Validation thresholds.
    pub coverage_threshold: f64,
    pub minimality_threshold: f64,
    pub independence_threshold: f64,
    /// Enable feedback loop.
    pub feedback_enabled: bool,
    /// Maximum cycles before stopping.
    pub max_cycles: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            source_domain: String::new(),
            target_domain: None,
            coverage_threshold: super::DEFAULT_COVERAGE_THRESHOLD,
            minimality_threshold: super::DEFAULT_MINIMALITY_THRESHOLD,
            independence_threshold: super::DEFAULT_INDEPENDENCE_THRESHOLD,
            feedback_enabled: true,
            max_cycles: 10,
        }
    }
}

impl PipelineConfig {
    /// Creates a new pipeline config for a domain.
    #[must_use]
    pub fn new(source_domain: impl Into<String>) -> Self {
        Self {
            source_domain: source_domain.into(),
            ..Default::default()
        }
    }

    /// Sets target domain for translation.
    #[must_use]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target_domain = Some(target.into());
        self
    }
}

/// Current state of pipeline execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineState {
    /// Not started.
    NotStarted,
    /// Running a specific stage.
    Running(StageId),
    /// Completed successfully.
    Completed,
    /// Failed at a stage.
    Failed(StageId),
    /// Paused for feedback.
    Paused(StageId),
}

/// Record of a single pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecution {
    /// Execution ID.
    pub id: String,
    /// Cycle number (for feedback loops).
    pub cycle: usize,
    /// Configuration used.
    pub config: PipelineConfig,
    /// Current state.
    pub state: PipelineState,
    /// Stage outputs.
    pub stage_outputs: StageOutputs,
    /// All feedback signals collected.
    pub feedback_buffer: Vec<FeedbackSignal>,
    /// Start time.
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// End time (if completed).
    #[serde(default)]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Outputs from each stage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageOutputs {
    /// Stage 1: SEE output.
    #[serde(default)]
    pub see: Option<PerceptionRecord>,
    /// Stage 2: SPEAK output.
    #[serde(default)]
    pub speak: Option<VocabularyRecord>,
    /// Stage 3: DECOMPOSE output.
    #[serde(default)]
    pub decompose: Option<ExtractionResult>,
    /// Stage 4: COMPOSE output.
    #[serde(default)]
    pub compose: Option<CompositionResult>,
    /// Stage 5: TRANSLATE output.
    #[serde(default)]
    pub translate: Option<TranslationRecord>,
    /// Stage 6: VALIDATE output.
    #[serde(default)]
    pub validate: Option<ValidationReport>,
    /// Stage 7: DEPLOY output.
    #[serde(default)]
    pub deploy: Option<DeploymentRecord>,
    /// Stage 8: IMPROVE output.
    #[serde(default)]
    pub improve: Option<ImprovementPlan>,
}

impl StageOutputs {
    /// Returns the output for a given stage.
    #[must_use]
    pub fn get(&self, stage: StageId) -> Option<serde_json::Value> {
        match stage {
            StageId::See => self.see.as_ref().and_then(|o| serde_json::to_value(o).ok()),
            StageId::Speak => self
                .speak
                .as_ref()
                .and_then(|o| serde_json::to_value(o).ok()),
            StageId::Decompose => self
                .decompose
                .as_ref()
                .and_then(|o| serde_json::to_value(o).ok()),
            StageId::Compose => self
                .compose
                .as_ref()
                .and_then(|o| serde_json::to_value(o).ok()),
            StageId::Translate => self
                .translate
                .as_ref()
                .and_then(|o| serde_json::to_value(o).ok()),
            StageId::Validate => self
                .validate
                .as_ref()
                .and_then(|o| serde_json::to_value(o).ok()),
            StageId::Deploy => self
                .deploy
                .as_ref()
                .and_then(|o| serde_json::to_value(o).ok()),
            StageId::Improve => self
                .improve
                .as_ref()
                .and_then(|o| serde_json::to_value(o).ok()),
        }
    }

    /// Checks if a stage has output.
    #[must_use]
    pub fn has(&self, stage: StageId) -> bool {
        match stage {
            StageId::See => self.see.is_some(),
            StageId::Speak => self.speak.is_some(),
            StageId::Decompose => self.decompose.is_some(),
            StageId::Compose => self.compose.is_some(),
            StageId::Translate => self.translate.is_some(),
            StageId::Validate => self.validate.is_some(),
            StageId::Deploy => self.deploy.is_some(),
            StageId::Improve => self.improve.is_some(),
        }
    }
}

/// Final output from a complete pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    /// Domain processed.
    pub domain: String,
    /// Total cycles executed.
    pub total_cycles: usize,
    /// Final extraction result.
    pub extraction: ExtractionResult,
    /// Final translation (if target specified).
    #[serde(default)]
    pub translation: Option<TranslationRecord>,
    /// Final validation.
    pub validation: ValidationReport,
    /// Deployed artifacts.
    pub deployment: DeploymentRecord,
    /// Improvement plan for next cycle.
    #[serde(default)]
    pub improvement: Option<ImprovementPlan>,
}

/// The CEP Pipeline orchestrator.
#[derive(Debug, Clone)]
pub struct CepPipeline {
    /// Pipeline configuration.
    pub config: PipelineConfig,
    /// Current execution (if running).
    current_execution: Option<PipelineExecution>,
    /// Execution history.
    history: Vec<PipelineExecution>,
}

impl CepPipeline {
    /// Creates a new pipeline with configuration.
    #[must_use]
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            current_execution: None,
            history: Vec::new(),
        }
    }

    /// Creates a pipeline for a source domain.
    #[must_use]
    pub fn for_domain(domain: impl Into<String>) -> Self {
        Self::new(PipelineConfig::new(domain))
    }

    /// Returns the current execution state.
    #[must_use]
    pub fn state(&self) -> PipelineState {
        self.current_execution
            .as_ref()
            .map_or(PipelineState::NotStarted, |e| e.state)
    }

    /// Starts a new pipeline execution.
    /// Returns the execution if started successfully, None if already running.
    pub fn start(&mut self) -> Option<&mut PipelineExecution> {
        if self.current_execution.is_some() {
            return None; // Already running
        }
        let cycle = self.history.len();
        let execution = PipelineExecution {
            id: nexcore_id::NexId::v4().to_string(),
            cycle,
            config: self.config.clone(),
            state: PipelineState::Running(StageId::See),
            stage_outputs: StageOutputs::default(),
            feedback_buffer: Vec::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
        };
        self.current_execution = Some(execution);
        self.current_execution.as_mut()
    }

    /// Advances to the next stage.
    pub fn advance(&mut self) -> Option<StageId> {
        if let Some(ref mut exec) = self.current_execution {
            if let PipelineState::Running(current) = exec.state {
                if let Some(next) = current.next() {
                    exec.state = PipelineState::Running(next);
                    return Some(next);
                }
                // Completed all stages
                exec.state = PipelineState::Completed;
                exec.completed_at = Some(chrono::Utc::now());
            }
        }
        None
    }

    /// Records stage output and advances.
    pub fn complete_stage<T: Serialize>(
        &mut self,
        stage: StageId,
        output: &T,
        feedback: FeedbackSignal,
    ) {
        if let Some(ref mut exec) = self.current_execution {
            // Store feedback
            exec.feedback_buffer.push(feedback);

            // Store output based on stage type
            if let Ok(json) = serde_json::to_value(output) {
                match stage {
                    StageId::See => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.see = Some(o);
                        }
                    }
                    StageId::Speak => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.speak = Some(o);
                        }
                    }
                    StageId::Decompose => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.decompose = Some(o);
                        }
                    }
                    StageId::Compose => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.compose = Some(o);
                        }
                    }
                    StageId::Translate => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.translate = Some(o);
                        }
                    }
                    StageId::Validate => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.validate = Some(o);
                        }
                    }
                    StageId::Deploy => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.deploy = Some(o);
                        }
                    }
                    StageId::Improve => {
                        if let Ok(o) = serde_json::from_value(json) {
                            exec.stage_outputs.improve = Some(o);
                        }
                    }
                }
            }
        }
        self.advance();
    }

    /// Finishes current execution and archives it.
    pub fn finish(&mut self) -> Option<PipelineExecution> {
        if let Some(mut exec) = self.current_execution.take() {
            exec.state = PipelineState::Completed;
            exec.completed_at = Some(chrono::Utc::now());
            self.history.push(exec.clone());
            return Some(exec);
        }
        None
    }

    /// Returns execution history.
    #[must_use]
    pub fn history(&self) -> &[PipelineExecution] {
        &self.history
    }

    /// Initiates feedback loop (IMPROVE → SEE).
    pub fn feedback_loop(&mut self) -> bool {
        if !self.config.feedback_enabled {
            return false;
        }
        if self.history.len() >= self.config.max_cycles {
            return false;
        }

        // Archive current execution and start new cycle
        self.finish();
        self.start();
        true
    }

    /// Returns the current execution (if any).
    #[must_use]
    pub fn current(&self) -> Option<&PipelineExecution> {
        self.current_execution.as_ref()
    }

    /// Returns mutable reference to current execution.
    pub fn current_mut(&mut self) -> Option<&mut PipelineExecution> {
        self.current_execution.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = CepPipeline::for_domain("pharmacovigilance");
        assert_eq!(pipeline.config.source_domain, "pharmacovigilance");
        assert_eq!(pipeline.state(), PipelineState::NotStarted);
    }

    #[test]
    fn test_pipeline_start() {
        let mut pipeline = CepPipeline::for_domain("test");
        let exec = pipeline.start();
        assert!(exec.is_some());
        assert_eq!(pipeline.state(), PipelineState::Running(StageId::See));
    }

    #[test]
    fn test_pipeline_advance() {
        let mut pipeline = CepPipeline::for_domain("test");
        pipeline.start();

        let next = pipeline.advance();
        assert_eq!(next, Some(StageId::Speak));
        assert_eq!(pipeline.state(), PipelineState::Running(StageId::Speak));
    }

    #[test]
    fn test_pipeline_double_start() {
        let mut pipeline = CepPipeline::for_domain("test");
        assert!(pipeline.start().is_some());
        assert!(pipeline.start().is_none()); // Already running
    }
}
