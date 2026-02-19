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

use anyhow::{Context, Result};
use tracing;

use nexcore_faers_etl::{
    SignalDetectionResult, filter_signals, run_full_pipeline, sink_signals_parquet,
};

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
}

impl ActionableSignal {
    /// Classify threat level based on signal strength.
    pub fn threat_level(&self) -> &'static str {
        if self.eb05 >= 5.0 && self.algorithms_flagged >= 4 {
            "Critical"
        } else if self.eb05 >= 3.0 && self.algorithms_flagged >= 3 {
            "High"
        } else if self.eb05 >= 2.0 && self.algorithms_flagged >= 2 {
            "Medium"
        } else {
            "Low"
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

        // Build top signals for report (sorted by EB05 descending)
        let mut top: Vec<SignalEntry> = actionable.iter().map(|s| s.to_entry()).collect();
        top.sort_by(|a, b| {
            b.eb05
                .partial_cmp(&a.eb05)
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
    fn apply_thresholds(&self, results: &[SignalDetectionResult]) -> Vec<ActionableSignal> {
        let thresholds = &self.config.thresholds;

        results
            .iter()
            .filter_map(|r| {
                let flagged = count_algorithms_flagged(r);
                if thresholds.passes(
                    r.prr.point.value(),
                    r.ror.lower_ci,
                    r.ic.lower_ci,
                    r.ebgm.lower_ci,
                    r.case_count.value(),
                    flagged,
                ) {
                    Some(ActionableSignal {
                        drug: r.drug.as_str().to_string(),
                        event: r.event.as_str().to_string(),
                        case_count: r.case_count.value(),
                        prr: r.prr.point.value(),
                        prr_lower_ci: r.prr.lower_ci,
                        ror: r.ror.point.value(),
                        ror_lower_ci: r.ror.lower_ci,
                        ic: r.ic.point.value(),
                        ic025: r.ic.lower_ci,
                        ebgm: r.ebgm.point.value(),
                        eb05: r.ebgm.lower_ci,
                        algorithms_flagged: flagged,
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

        // Write all signals to Parquet
        let parquet_path = self.config.output_dir.join("signals.parquet");
        let row_count = sink_signals_parquet(all_results, &parquet_path)
            .context("Failed to write signals Parquet")?;

        tracing::info!(
            rows = row_count.value(),
            path = %parquet_path.display(),
            "PHAROS Stage 6: Signals persisted to Parquet"
        );

        // Report will be saved by the caller after finalization
        let _ = report;
        Ok(())
    }
}

/// Count how many of the 4 algorithms flagged a signal.
fn count_algorithms_flagged(r: &SignalDetectionResult) -> u32 {
    let mut count = 0u32;
    if r.prr.is_signal {
        count += 1;
    }
    if r.ror.is_signal {
        count += 1;
    }
    if r.ic.is_signal {
        count += 1;
    }
    if r.ebgm.is_signal {
        count += 1;
    }
    count
}
