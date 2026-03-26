//! Core PHAROS pipeline — the full surveillance loop.
//!
//! Primitive composition: σ(Sequence) + →(Causality) + ∂(Boundary) + ν(Frequency)
//!
//! Pipeline stages:
//! 1. INGEST  — Load FAERS quarterly ASCII via nexcore-faers-etl
//! 2. DETECT  — Run all 4 signal detection algorithms (PRR/ROR/IC/EBGM)
//! 3. FILTER  — Apply configurable thresholds (∂ boundary gate)
//! 4. INJECT  — Feed actionable signals into Guardian homeostasis loop
//! 5. EMIT    — Fire cytokine events (IFN for new signals, CSF on completion)
//! 6. PERSIST — Write Parquet + JSON report

use std::time::Instant;

use nexcore_error::{Context, Result};
use tracing;

use nexcore_faers_etl::{SignalDetectionResult, filter_signals, run_full_pipeline, sink_signals};

use crate::config::PharosConfig;
use crate::report::{SignalEntry, SurveillanceReport};
use crate::thresholds::SignalThresholds;

/// Output from a single PHAROS pipeline execution.
#[derive(Debug)]
pub struct PharosOutput {
    /// The surveillance report for this run.
    pub report: SurveillanceReport,

    /// Actionable signals that passed threshold filtering.
    pub actionable_signals: Vec<ActionableSignal>,
}

/// A signal that passed all threshold gates — ready for Guardian injection.
#[derive(Debug, Clone)]
pub struct ActionableSignal {
    pub drug: String,
    pub event: String,
    pub case_count: u64,
    pub prr: f64,
    pub prr_lower_ci: f64,
    pub ror: f64,
    pub ror_lower_ci: f64,
    pub ic: f64,
    pub ic025: f64,
    pub ebgm: f64,
    pub eb05: f64,
    pub algorithms_flagged: u32,
    /// Unified boundary sharpness score (∂-score).
    ///
    /// Encodes the conservation law: all four algorithms measure the same
    /// boundary from different angles. This is the Rosetta number —
    /// a single continuous value encoding how crystallized the signal is.
    pub boundary_score: f64,
}

impl ActionableSignal {
    /// Classify threat level based on boundary sharpness (∂-score).
    ///
    /// Maps to the visual encoding:
    /// - Low: faint glow — boundary barely visible
    /// - Medium: ring forming — boundary detectable
    /// - High: ring crystallized — boundary confirmed
    /// - Critical: ring blazing — existence undeniable
    pub fn threat_level(&self) -> &'static str {
        match self.boundary_score {
            s if s >= 1.5 => "Critical",
            s if s >= 0.5 => "High",
            s if s >= 0.1 => "Medium",
            _ => "Low",
        }
    }

    /// Convert to a report entry.
    pub fn to_entry(&self) -> SignalEntry {
        SignalEntry {
            drug: self.drug.clone(),
            event: self.event.clone(),
            case_count: self.case_count,
            prr: self.prr,
            prr_lower_ci: self.prr_lower_ci,
            ror: self.ror,
            ror_lower_ci: self.ror_lower_ci,
            ic: self.ic,
            ic025: self.ic025,
            ebgm: self.ebgm,
            eb05: self.eb05,
            algorithms_flagged: self.algorithms_flagged,
            threat_level: self.threat_level().to_string(),
            boundary_score: self.boundary_score,
        }
    }
}

/// The PHAROS pipeline executor.
pub struct PharosPipeline {
    config: PharosConfig,
}

impl PharosPipeline {
    /// Create a new pipeline with the given configuration.
    pub fn new(config: PharosConfig) -> Self {
        Self { config }
    }

    /// Execute the full PHAROS surveillance pipeline.
    ///
    /// This is the main entry point. It runs all 6 stages synchronously
    /// and returns the complete output including the report.
    pub fn execute(&self) -> Result<PharosOutput> {
        let start = Instant::now();
        let mut report = SurveillanceReport::new(&self.config.faers_dir.display().to_string());

        // Stage 1: INGEST + DETECT (via nexcore-faers-etl)
        tracing::info!(
            faers_dir = %self.config.faers_dir.display(),
            min_cases = self.config.min_cases,
            "PHAROS Stage 1-2: Ingesting FAERS data and running signal detection"
        );

        let pipeline_output = run_full_pipeline(
            &self.config.faers_dir,
            self.config.include_all_roles,
            self.config.min_cases,
        )
        .context("FAERS ETL pipeline failed")?;

        report.total_pairs = pipeline_output.total_pairs;

        // Count raw signals (any algorithm flagged)
        let raw_signal_refs = filter_signals(&pipeline_output.results);
        report.raw_signals = raw_signal_refs.len();

        tracing::info!(
            total_pairs = report.total_pairs,
            raw_signals = report.raw_signals,
            "PHAROS Stage 2 complete: Signal detection finished"
        );

        // Stage 3: FILTER — Apply threshold gates
        let actionable = self.apply_thresholds(&pipeline_output.results);
        report.actionable_signals = actionable.len();

        tracing::info!(
            actionable = report.actionable_signals,
            thresholds = ?self.config.thresholds,
            "PHAROS Stage 3 complete: Threshold filtering"
        );

        // Stage 4: INJECT — Guardian signals (counted but actual injection
        // requires the Guardian loop instance, done by the caller)
        report.guardian_injections = if self.config.inject_guardian {
            actionable.len()
        } else {
            0
        };

        // Stage 5: EMIT — Cytokine events (counted, actual emission by caller)
        report.cytokines_emitted = if self.config.emit_cytokines {
            // 1 IFN per actionable signal + 1 CSF for pipeline completion
            actionable.len() + 1
        } else {
            0
        };

        // Stage 6: PERSIST — Write outputs
        self.persist_outputs(&pipeline_output.results, &actionable, &mut report)?;

        // Finalize report
        report.duration_ms = start.elapsed().as_millis();
        report.thresholds_used = format!("{:?}", self.config.thresholds);

        // Build top signals for report (sorted by ∂-score descending —
        // the unified boundary sharpness metric, not a single algorithm)
        let mut top: Vec<SignalEntry> = actionable.iter().map(|s| s.to_entry()).collect();
        top.sort_by(|a, b| {
            b.boundary_score
                .partial_cmp(&a.boundary_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        top.truncate(self.config.top_n_report);
        report.top_signals = top;

        report.save(&self.config.output_dir)?;
        tracing::info!(summary = %report.summary(), "PHAROS pipeline complete");

        Ok(PharosOutput {
            report,
            actionable_signals: actionable,
        })
    }

    /// Stage 3: Apply threshold filtering to raw signal results.
    ///
    /// Each signal that passes the binary gate also receives a continuous
    /// ∂-score measuring boundary sharpness. The binary gate answers "is
    /// there a boundary?" The ∂-score answers "how sharp is it?"
    fn apply_thresholds(&self, results: &[SignalDetectionResult]) -> Vec<ActionableSignal> {
        let thresholds = &self.config.thresholds;

        results
            .iter()
            .filter_map(|r| {
                let flagged = count_algorithms_flagged(r);
                let prr = r.prr.point.value();
                let ror_lower = r.ror.lower_ci;
                let ic025 = r.ic.lower_ci;
                let eb05 = r.ebgm.lower_ci;
                let cases = r.case_count.value();

                if thresholds.passes(prr, ror_lower, ic025, eb05, cases, flagged) {
                    let boundary_score =
                        thresholds.boundary_score(prr, ror_lower, ic025, eb05, cases, flagged);

                    Some(ActionableSignal {
                        drug: r.drug.as_str().to_string(),
                        event: r.event.as_str().to_string(),
                        case_count: cases,
                        prr,
                        prr_lower_ci: r.prr.lower_ci,
                        ror: r.ror.point.value(),
                        ror_lower_ci: ror_lower,
                        ic: r.ic.point.value(),
                        ic025,
                        ebgm: r.ebgm.point.value(),
                        eb05,
                        algorithms_flagged: flagged,
                        boundary_score,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Stage 6: Persist Parquet + report.
    fn persist_outputs(
        &self,
        all_results: &[SignalDetectionResult],
        _actionable: &[ActionableSignal],
        report: &mut SurveillanceReport,
    ) -> Result<()> {
        std::fs::create_dir_all(&self.config.output_dir)?;

        // Write all signals to JSON
        let signals_path = self.config.output_dir.join("signals.json");
        let row_count =
            sink_signals(all_results, &signals_path).context("Failed to write signals JSON")?;

        tracing::info!(
            rows = row_count.value(),
            path = %signals_path.display(),
            "PHAROS Stage 6: Signals persisted"
        );

        // Report will be saved by the caller after finalization
        let _ = report;
        Ok(())
    }
}

/// Count how many of the 4 algorithms flagged a signal.
fn count_algorithms_flagged(r: &SignalDetectionResult) -> u32 {
    u32::from(r.prr.is_signal)
        + u32::from(r.ror.is_signal)
        + u32::from(r.ic.is_signal)
        + u32::from(r.ebgm.is_signal)
}
