//! Main orchestrator agent.

use super::chain_planner::ChainPlanner;
use super::executor::ExecutorEngine;
use super::feedback::FeedbackLoop;
use super::models::{ExecutionResult, Skill};
use super::perception::PerceptionLayer;
use super::skill_matcher::SkillMatcher;
use super::state::SessionStore;
use nexcore_error::Result;
use std::path::Path;

/// Chain orchestrator agent with 5-phase pipeline.
///
/// Pipeline: Perceive → Match → Plan → Execute → Learn
pub struct ChainOrchestratorAgent {
    /// Perception layer for task analysis
    pub perception: PerceptionLayer,
    /// Skill matcher
    pub matcher: SkillMatcher,
    /// Chain planner
    pub planner: ChainPlanner,
    /// Executor engine
    pub executor: ExecutorEngine,
    /// Feedback loop
    pub feedback: FeedbackLoop,
    /// Session pattern storage
    pub session_store: SessionStore,
}

impl ChainOrchestratorAgent {
    /// Create a new orchestrator agent.
    #[must_use]
    pub fn new(skills: Vec<Skill>, metrics_dir: &Path) -> Self {
        Self {
            perception: PerceptionLayer::new(),
            matcher: SkillMatcher::new(skills),
            planner: ChainPlanner::new(),
            executor: ExecutorEngine::new(),
            feedback: FeedbackLoop::new(metrics_dir),
            session_store: SessionStore::new(metrics_dir),
        }
    }

    /// Process a user request through the 5-phase pipeline.
    ///
    /// # Errors
    ///
    /// Returns error if execution fails.
    pub async fn process(&mut self, request: &str) -> Result<ExecutionResult> {
        // 1. Perceive
        let analysis = self.perception.analyze(request);

        // 2-3. Match and Plan (or recall from cache)
        let chain = if let Some(cached_chain) = self.session_store.recall(&analysis.intent) {
            cached_chain.clone()
        } else {
            let matched_skills = self.matcher.match_skills(&analysis, 5);
            self.planner.plan(matched_skills, analysis.clone())
        };

        // 3.5 Verify Safety (Axiomatic Guardrail)
        if let Some(manifold) = &chain.safety_manifold {
            let state = vec![chain.confidence];
            if manifold.distance_to_boundary(&state) < 0.0 {
                nexcore_error::bail!(
                    "AGENT SAFETY VIOLATION: Execution chain is outside the safety manifold boundary."
                );
            }
        }

        // 4. Execute
        let result = self.executor.execute(&chain).await;

        // 5. Learn
        self.feedback.record(&chain, &result);

        // If successful, remember the pattern
        if result.status == super::models::ExecutionStatus::Completed {
            let analysis = self.perception.analyze(request);
            self.session_store.remember(&analysis.intent, chain);
        }

        Ok(result)
    }
}
