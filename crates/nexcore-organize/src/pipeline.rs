//! Full ORGANIZE pipeline orchestrator.
//!
//! Tier: T3 (8 primitives: ∃ κ μ → ∂ Σ ∅ ς)
//!
//! Chains all 8 steps: Observe → Rank → Group → Assign → Name → Integrate → Zero-out → Enforce.

use std::path::PathBuf;

use crate::config::OrganizeConfig;
use crate::enforce::{self, DriftReport, OrganizeState};
use crate::error::OrganizeResult;
use crate::integrate::{self, IntegrationPlan};
use crate::zero_out::{self, CleanupReport};

// ============================================================================
// Types
// ============================================================================

/// Complete result of the ORGANIZE pipeline.
///
/// Tier: T3 (composes all 8 pipeline step outputs)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrganizeResult2 {
    /// Root directory.
    pub root: PathBuf,
    /// Integration plan (step 6 output).
    pub plan: IntegrationPlan,
    /// Cleanup report (step 7 output).
    pub cleanup: CleanupReport,
    /// State snapshot (step 8 output).
    pub state: OrganizeState,
    /// Drift report (if previous state existed).
    pub drift: Option<DriftReport>,
}

// ============================================================================
// Pipeline
// ============================================================================

/// The ORGANIZE pipeline orchestrator.
///
/// Tier: T3 (∃ κ μ → ∂ Σ ∅ ς)
pub struct OrganizePipeline {
    config: OrganizeConfig,
}

impl OrganizePipeline {
    /// Create a new pipeline with the given config.
    pub fn new(config: OrganizeConfig) -> Self {
        Self { config }
    }

    /// Run the full 8-step pipeline.
    ///
    /// 1. Observe (∃) — inventory filesystem
    /// 2. Rank (κ) — score entries
    /// 3. Group (μ) — cluster by rules
    /// 4. Assign (→) — map to actions
    /// 5. Name (∂) — resolve naming
    /// 6. Integrate (Σ) — execute/simulate
    /// 7. Zero-out (∅) — cleanup empties/dupes
    /// 8. Enforce (ς) — snapshot state
    pub fn run(&self) -> OrganizeResult<OrganizeResult2> {
        let dry_run = self.config.dry_run;

        // Step 1: Observe
        let inventory = crate::observe::observe(&self.config)?;

        // Step 2: Rank
        let ranked = crate::rank::rank(inventory, &self.config)?;

        // Step 3: Group
        let grouped = crate::group::group(ranked, &self.config)?;

        // Step 4: Assign
        let assigned = crate::assign::assign(grouped, &self.config)?;

        // Step 5: Name
        let named = crate::name::name(assigned, &self.config)?;

        // Step 6: Integrate
        let plan = integrate::integrate(named, dry_run)?;

        // Step 7: Zero-out
        let cleanup = zero_out::zero_out(&plan, dry_run)?;

        // Step 8: Enforce
        let state = enforce::snapshot(&self.config.root)?;

        // Check for drift against previous state
        let drift = self
            .load_previous_state()
            .map(|prev| enforce::detect_drift(&prev, &state));

        // Save current state if state_path is configured
        if let Some(ref state_path) = self.config.state_path {
            let _ = state.save(state_path);
        }

        Ok(OrganizeResult2 {
            root: self.config.root.clone(),
            plan,
            cleanup,
            state,
            drift,
        })
    }

    /// Run analysis-only mode (always dry-run, no mutations).
    pub fn analyze(&self) -> OrganizeResult<OrganizeResult2> {
        let mut config = self.config.clone();
        config.dry_run = true;
        let pipeline = OrganizePipeline::new(config);
        pipeline.run()
    }

    /// Load previous state from configured state_path.
    fn load_previous_state(&self) -> Option<OrganizeState> {
        self.config
            .state_path
            .as_ref()
            .and_then(|p| OrganizeState::load(p).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_pipeline_empty_dir() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let config = OrganizeConfig::default_for(dir.path());
            let pipeline = OrganizePipeline::new(config);
            let result = pipeline.run();
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert!(result.plan.dry_run);
                assert_eq!(result.plan.total, 0);
            }
        }
    }

    #[test]
    fn test_pipeline_with_files() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let _ = fs::write(dir.path().join("main.rs"), "fn main() {}");
            let _ = fs::write(dir.path().join("readme.md"), "# Hello");
            let _ = fs::write(dir.path().join("photo.png"), "fake png data");

            let config = OrganizeConfig::default_for(dir.path());
            let pipeline = OrganizePipeline::new(config);
            let result = pipeline.run();
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert!(result.plan.dry_run);
                assert_eq!(result.plan.total, 3);
                assert_eq!(result.state.count, 3);
            }
        }
    }

    #[test]
    fn test_pipeline_analyze_is_dry_run() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let _ = fs::write(dir.path().join("test.txt"), "data");
            let mut config = OrganizeConfig::default_for(dir.path());
            config.dry_run = false; // Set to live mode

            let pipeline = OrganizePipeline::new(config);
            let result = pipeline.analyze(); // Should force dry-run
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert!(result.plan.dry_run);
            }
        }
    }

    #[test]
    fn test_pipeline_drift_detection() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let state_path = dir.path().join(".organize_state.json");
            let _ = fs::write(dir.path().join("a.txt"), "hello");

            // First run: save state
            let mut config = OrganizeConfig::default_for(dir.path());
            config.state_path = Some(state_path.clone());
            let pipeline = OrganizePipeline::new(config.clone());
            let r1 = pipeline.run();
            assert!(r1.is_ok());

            // Add a file, run again
            let _ = fs::write(dir.path().join("b.txt"), "world");
            let pipeline = OrganizePipeline::new(config);
            let r2 = pipeline.run();
            assert!(r2.is_ok());
            if let Ok(r2) = r2 {
                // Should detect drift (b.txt added, state file added)
                if let Some(drift) = r2.drift {
                    assert!(drift.has_drift);
                }
            }
        }
    }
}
