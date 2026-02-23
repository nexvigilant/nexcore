//! FDA Data Bridge - ETL pipeline for pharmacovigilance intelligence.
//!
//! Provides comprehensive data integration with FDA sources:
//!
//! # Modules
//!
//! - **FAERS ETL** - Quarterly ASCII file ingestion and signal detection
//! - **OpenFDA API** ([`api`]) - Real-time adverse event queries with caching
//! - **NDC Bridge** ([`ndc`]) - National Drug Code directory lookups
//! - **Deduplication** ([`dedup`]) - Report clustering and duplicate removal
//! - **Types** ([`types`]) - T2-P newtypes and T2-C composites (Primitive Codex)
//!
//! # Configuration
//!
//! The ingest stage reads from the `FAERS_DATA_DIR` environment variable.
//! If not set, it will look for FAERS data in the default location:
//! `./data/faers` relative to the current working directory.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod analytics;
pub mod api;
pub mod dedup;
pub mod grounding;
pub mod ndc;
pub mod spatial_bridge;
pub mod types;

use nexcore_error::{Context, Result};
use nexcore_vigilance::pv::faers::parse_quarterly_linked;
use nexcore_vigilance::pv::signals::batch::{
    BatchContingencyTables, CompleteSignalResult, batch_complete_parallel,
};
use nexcore_vigilance::pv::signals::core::newtypes::{Ebgm, Ic, Prr, Ror};
use polars::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

pub use analytics::{
    CascadeConfig,
    CaseSeriousness,
    // A78 — Polypharmacy Interaction Signal
    CountrySignal,
    DrugCharacterization,
    GeographicCase,
    GeographicConfig,
    GeographicDivergence,
    MonthBucket,
    OutcomeCase,
    OutcomeConditionedConfig,
    OutcomeConditionedSignal,
    PolypharmacyCase,
    PolypharmacyConfig,
    PolypharmacySignal,
    ReactionOutcome,
    ReporterCase,
    ReporterQualification,
    ReporterWeightedConfig,
    ReporterWeightedSignal,
    SeriousnessCascade,
    SeriousnessCase,
    SeriousnessFlag,
    SignalVelocity,
    TemporalCase,
    VelocityConfig,
    compute_geographic_divergence,
    compute_outcome_conditioned,
    compute_polypharmacy_signals,
    compute_reporter_weighted,
    compute_seriousness_cascade,
    compute_signal_velocity,
};
pub use types::{
    CaseCount, ContingencyBatch, DrugName, DrugRole, EventName, MetricAssessment, RowCount, columns,
};

const DEFAULT_FAERS_DIR: &str = "./data/faers";
const FAERS_DATA_DIR_ENV: &str = "FAERS_DATA_DIR";

fn get_faers_data_dir() -> PathBuf {
    env::var(FAERS_DATA_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_FAERS_DIR))
}

fn format_vec<T: std::fmt::Display>(v: &[T]) -> Option<String> {
    if v.is_empty() {
        return None;
    }
    Some(
        v.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("|"),
    )
}

// =============================================================================
// INGEST
// =============================================================================

/// Ingest FAERS quarterly files and return a DataFrame of drug-event pairs.
pub fn ingest_faers_quarterly() -> Result<DataFrame> {
    let faers_dir = get_faers_data_dir();
    ingest_faers_quarterly_with_options(&faers_dir, false)
}

/// Ingest FAERS quarterly files with custom options.
pub fn ingest_faers_quarterly_with_options(
    faers_dir: &Path,
    include_all_roles: bool,
) -> Result<DataFrame> {
    tracing::info!(stage = "faers-quarterly", path = %faers_dir.display(), "Starting FAERS ingest");
    let (reports, _parse_errors) = parse_quarterly_linked(faers_dir);
    if reports.is_empty() {
        return Ok(DataFrame::empty());
    }
    let rows = flatten_reports(&reports, include_all_roles);
    build_ingest_dataframe(&reports, &rows)
}

/// A flattened row: (report_index, case_id, drug_name, role, event_name).
type FlatRow = (usize, u64, String, String, String);

/// Flatten all reports into per-(drug, event) rows, filtering by role.
fn flatten_reports(
    reports: &[nexcore_vigilance::pv::faers::LinkedReport],
    include_all_roles: bool,
) -> Vec<FlatRow> {
    let mut rows = Vec::new();
    for (idx, report) in reports.iter().enumerate() {
        let Ok(case_id) = report.primary_id.parse::<u64>() else {
            tracing::warn!(primary_id = %report.primary_id, "Skipping non-numeric ID");
            continue;
        };
        flatten_single_report(&mut rows, idx, case_id, report, include_all_roles);
    }
    rows
}

/// Flatten a single report's drugs × reactions into rows.
fn flatten_single_report(
    rows: &mut Vec<FlatRow>,
    idx: usize,
    case_id: u64,
    report: &nexcore_vigilance::pv::faers::LinkedReport,
    include_all_roles: bool,
) {
    let selected = select_drugs(report, include_all_roles);
    for (drug_name, role, _seq) in selected {
        append_drug_events(rows, idx, case_id, &drug_name, &role, &report.reactions);
    }
}

/// Append one drug × all reactions to the rows vec.
fn append_drug_events(
    rows: &mut Vec<FlatRow>,
    idx: usize,
    case_id: u64,
    drug: &str,
    role: &str,
    reactions: &[String],
) {
    for event in reactions {
        rows.push((
            idx,
            case_id,
            drug.to_uppercase(),
            role.to_string(),
            event.to_uppercase(),
        ));
    }
}

/// Build the full DataFrame from flattened rows.
fn build_ingest_dataframe(
    reports: &[nexcore_vigilance::pv::faers::LinkedReport],
    rows: &[FlatRow],
) -> Result<DataFrame> {
    let n = rows.len();
    let mut acc = IngestColumns::with_capacity(n);

    for (idx, case_id, drug, role, event) in rows {
        let report = &reports[*idx];
        acc.push(report, *case_id, drug.clone(), role.clone(), event.clone());
    }

    acc.into_dataframe()
}

/// Column vectors accumulated during ingest.
struct IngestColumns {
    case_ids: Vec<u64>,
    drugs: Vec<String>,
    events: Vec<String>,
    age_years: Vec<Option<f64>>,
    age_groups: Vec<Option<String>>,
    sexes: Vec<Option<String>>,
    weights: Vec<Option<f64>>,
    reporter_countries: Vec<Option<String>>,
    occr_countries: Vec<Option<String>>,
    mfr_sndrs: Vec<Option<String>>,
    occp_cods: Vec<Option<String>>,
    mfr_nums: Vec<Option<String>>,
    fda_dts: Vec<Option<String>>,
    event_dts: Vec<Option<String>>,
    role_codes: Vec<Option<String>>,
    report_sources: Vec<Option<String>>,
    outcomes: Vec<Option<String>>,
    therapy: Vec<Option<String>>,
}

impl IngestColumns {
    fn with_capacity(n: usize) -> Self {
        Self {
            case_ids: Vec::with_capacity(n),
            drugs: Vec::with_capacity(n),
            events: Vec::with_capacity(n),
            age_years: Vec::with_capacity(n),
            age_groups: Vec::with_capacity(n),
            sexes: Vec::with_capacity(n),
            weights: Vec::with_capacity(n),
            reporter_countries: Vec::with_capacity(n),
            occr_countries: Vec::with_capacity(n),
            mfr_sndrs: Vec::with_capacity(n),
            occp_cods: Vec::with_capacity(n),
            mfr_nums: Vec::with_capacity(n),
            fda_dts: Vec::with_capacity(n),
            event_dts: Vec::with_capacity(n),
            role_codes: Vec::with_capacity(n),
            report_sources: Vec::with_capacity(n),
            outcomes: Vec::with_capacity(n),
            therapy: Vec::with_capacity(n),
        }
    }

    fn push(
        &mut self,
        r: &nexcore_vigilance::pv::faers::LinkedReport,
        case_id: u64,
        drug: String,
        role: String,
        event: String,
    ) {
        self.case_ids.push(case_id);
        self.drugs.push(drug);
        self.events.push(event);
        self.age_years.push(r.age_years);
        self.age_groups.push(r.age_group.clone());
        self.sexes.push(r.sex.clone());
        self.weights.push(r.weight_kg);
        self.reporter_countries.push(r.reporter_country.clone());
        self.occr_countries.push(r.occr_country.clone());
        self.mfr_sndrs.push(r.mfr_sndr.clone());
        self.occp_cods.push(r.occp_cod.clone());
        self.mfr_nums.push(r.mfr_num.clone());
        self.fda_dts.push(r.fda_dt.clone());
        self.event_dts.push(r.event_dt.clone());
        self.role_codes.push(Some(role));
        self.report_sources.push(format_vec(&r.report_sources));
        self.outcomes.push(format_vec(&r.outcomes));
        self.therapy.push(format_therapy(&r.therapy));
    }

    fn into_dataframe(self) -> Result<DataFrame> {
        DataFrame::new(vec![
            Series::new(columns::CASE_ID.into(), self.case_ids).into(),
            Series::new(columns::DRUG.into(), self.drugs).into(),
            Series::new(columns::EVENT.into(), self.events).into(),
            Series::new(columns::AGE_YEARS.into(), self.age_years).into(),
            Series::new(columns::AGE_GROUP.into(), self.age_groups).into(),
            Series::new(columns::SEX.into(), self.sexes).into(),
            Series::new(columns::WEIGHT_KG.into(), self.weights).into(),
            Series::new(columns::REPORTER_COUNTRY.into(), self.reporter_countries).into(),
            Series::new(columns::OCCR_COUNTRY.into(), self.occr_countries).into(),
            Series::new(columns::MFR_SNDR.into(), self.mfr_sndrs).into(),
            Series::new(columns::OCCP_COD.into(), self.occp_cods).into(),
            Series::new(columns::MFR_NUM.into(), self.mfr_nums).into(),
            Series::new(columns::FDA_DT.into(), self.fda_dts).into(),
            Series::new(columns::EVENT_DT.into(), self.event_dts).into(),
            Series::new(columns::ROLE_CODE.into(), self.role_codes).into(),
            Series::new(columns::REPORT_SOURCES.into(), self.report_sources).into(),
            Series::new(columns::OUTCOMES.into(), self.outcomes).into(),
            Series::new(columns::THERAPY_SUMMARY.into(), self.therapy).into(),
        ])
        .context("Failed to create high-resolution DataFrame")
    }
}

/// Format therapy entries into a pipe-separated string.
fn format_therapy(therapy: &[(Option<String>, Option<String>, u32)]) -> Option<String> {
    if therapy.is_empty() {
        return None;
    }
    let parts: Vec<String> = therapy
        .iter()
        .map(|(s, e, seq)| format_therapy_entry(s, e, *seq))
        .collect();
    Some(parts.join("||"))
}

/// Format a single therapy entry.
fn format_therapy_entry(start: &Option<String>, end: &Option<String>, seq: u32) -> String {
    format!(
        "{}|{}|{}",
        start.as_deref().unwrap_or(""),
        end.as_deref().unwrap_or(""),
        seq
    )
}

/// Select drugs from a report based on role filter.
fn select_drugs(
    report: &nexcore_vigilance::pv::faers::LinkedReport,
    include_all_roles: bool,
) -> Vec<(String, String, u32)> {
    if include_all_roles {
        return report.drugs.clone();
    }
    report
        .drugs
        .iter()
        .filter(|(_, role, _)| DrugRole::from(role.as_str()).is_suspect())
        .cloned()
        .collect()
}

// =============================================================================
// TRANSFORMS
// =============================================================================

/// Transform: normalize drug and event names (no-op, already uppercased).
pub fn transform_normalize_names(df: LazyFrame) -> Result<LazyFrame> {
    Ok(df)
}

/// Transform: aggregate drug-event pairs into counts.
pub fn transform_count_drug_events(df: LazyFrame) -> Result<LazyFrame> {
    transform_count_drug_events_stratified(df, vec![])
}

/// Transform: aggregate with stratification.
pub fn transform_count_drug_events_stratified(
    df: LazyFrame,
    strata: Vec<&str>,
) -> Result<LazyFrame> {
    let mut group_by = vec![col(columns::DRUG), col(columns::EVENT)];
    for s in strata {
        group_by.push(col(s));
    }
    Ok(df
        .group_by(group_by)
        .agg([col(columns::CASE_ID).count().alias(columns::N)]))
}

/// Transform: filter to minimum case count (default 3).
pub fn transform_filter_minimum(df: LazyFrame) -> Result<LazyFrame> {
    transform_filter_minimum_n(df, 3)
}

/// Transform: filter to minimum case count with custom threshold.
pub fn transform_filter_minimum_n(df: LazyFrame, min_cases: i64) -> Result<LazyFrame> {
    Ok(df.filter(col(columns::N).gt_eq(lit(min_cases))))
}

// =============================================================================
// SINKS
// =============================================================================

/// Sink: Write DataFrame to Parquet file (default path).
pub fn sink_parquet_output(df: LazyFrame) -> Result<RowCount> {
    sink_parquet_output_to(df, "output/drug_event_counts.parquet")
}

/// Sink: Write DataFrame to specified Parquet file path.
pub fn sink_parquet_output_to(df: LazyFrame, path_template: &str) -> Result<RowCount> {
    let mut output_df = df.collect().context("Failed to collect DataFrame")?;
    let row_count = RowCount(output_df.height() as u64);
    if row_count.value() == 0 {
        return Ok(RowCount(0));
    }
    let path = chrono::Utc::now().format(path_template).to_string();
    write_parquet(&mut output_df, &path, Some(1_000_000))?;
    Ok(row_count)
}

/// Sink: Write signal detection results to Parquet.
pub fn sink_signals_parquet(results: &[SignalDetectionResult], path: &Path) -> Result<RowCount> {
    let mut df = signals_to_dataframe(results)?;
    let row_count = RowCount(df.height() as u64);
    if row_count.value() == 0 {
        return Ok(RowCount(0));
    }
    write_parquet(&mut df, &path.display().to_string(), None)?;
    Ok(row_count)
}

/// Shared Parquet writer — creates parent dirs, writes with Snappy.
fn write_parquet(df: &mut DataFrame, path: &str, rg_size: Option<usize>) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create dir: {}", parent.display()))?;
    }
    let file =
        std::fs::File::create(path).with_context(|| format!("Failed to create file: {path}"))?;
    let mut w = ParquetWriter::new(file).with_compression(ParquetCompression::Snappy);
    if let Some(size) = rg_size {
        w = w.with_row_group_size(Some(size));
    }
    w.finish(df)
        .with_context(|| format!("Failed to write: {path}"))?;
    Ok(())
}

// =============================================================================
// SIGNAL DETECTION
// =============================================================================

/// Tier: T3 — Signal detection result for a drug-event pair.
#[derive(Debug, Clone)]
pub struct SignalDetectionResult {
    /// Drug name (T2-P)
    pub drug: DrugName,
    /// Event name / MedDRA PT (T2-P)
    pub event: EventName,
    /// Co-occurrence count — cell "a" (T2-P)
    pub case_count: CaseCount,
    /// PRR assessment (T2-C)
    pub prr: MetricAssessment<Prr>,
    /// ROR assessment (T2-C)
    pub ror: MetricAssessment<Ror>,
    /// IC assessment (T2-C)
    pub ic: MetricAssessment<Ic>,
    /// EBGM assessment (T2-C)
    pub ebgm: MetricAssessment<Ebgm>,
}

impl SignalDetectionResult {
    /// Returns true if any algorithm flagged this pair as a signal.
    #[must_use]
    pub fn is_any_signal(&self) -> bool {
        self.prr.is_signal || self.ror.is_signal || self.ic.is_signal || self.ebgm.is_signal
    }
}

/// Build contingency tables from aggregated drug-event counts.
pub fn build_contingency_tables_from_counts(df: &DataFrame) -> Result<ContingencyBatch> {
    let drugs = extract_str_column(df, columns::DRUG)?;
    let events = extract_str_column(df, columns::EVENT)?;
    let counts = extract_counts_column(df)?;
    let (dt, et, total) = compute_marginal_totals(drugs, events, &counts, df.height());
    Ok(build_tables_from_marginals(
        drugs,
        events,
        &counts,
        &dt,
        &et,
        total,
        df.height(),
    ))
}

fn extract_str_column<'a>(df: &'a DataFrame, name: &str) -> Result<&'a StringChunked> {
    df.column(name)
        .with_context(|| format!("Missing '{name}'"))?
        .str()
        .with_context(|| format!("'{name}' not string"))
}

fn extract_counts_column(df: &DataFrame) -> Result<Vec<u64>> {
    let n_col = df.column(columns::N).context("Missing 'n'")?;
    if let Ok(c) = n_col.u32() {
        return Ok(c.iter().map(|v| u64::from(v.unwrap_or(0))).collect());
    }
    if let Ok(c) = n_col.u64() {
        return Ok(c.iter().map(|v| v.unwrap_or(0)).collect());
    }
    if let Ok(c) = n_col.i64() {
        return Ok(c.iter().map(|v| v.unwrap_or(0) as u64).collect());
    }
    nexcore_error::bail!("'n' column must be u32, u64, or i64")
}

fn compute_marginal_totals(
    drugs: &StringChunked,
    events: &StringChunked,
    counts: &[u64],
    n: usize,
) -> (HashMap<String, u64>, HashMap<String, u64>, u64) {
    let mut dt: HashMap<String, u64> = HashMap::new();
    let mut et: HashMap<String, u64> = HashMap::new();
    let mut total: u64 = 0;
    for i in 0..n {
        let d = drugs.get(i).unwrap_or_default().to_string();
        let e = events.get(i).unwrap_or_default().to_string();
        *dt.entry(d).or_insert(0) += counts[i];
        *et.entry(e).or_insert(0) += counts[i];
        total += counts[i];
    }
    (dt, et, total)
}

fn build_tables_from_marginals(
    drugs: &StringChunked,
    events: &StringChunked,
    counts: &[u64],
    dt: &HashMap<String, u64>,
    et: &HashMap<String, u64>,
    total: u64,
    n: usize,
) -> ContingencyBatch {
    let mut dn = Vec::with_capacity(n);
    let mut en = Vec::with_capacity(n);
    let mut av = Vec::with_capacity(n);
    let mut bv = Vec::with_capacity(n);
    let mut cv = Vec::with_capacity(n);
    let mut dv = Vec::with_capacity(n);

    for i in 0..n {
        let d = drugs.get(i).unwrap_or_default().to_string();
        let e = events.get(i).unwrap_or_default().to_string();
        let a = counts[i];
        let d_tot = *dt.get(&d).unwrap_or(&0);
        let e_tot = *et.get(&e).unwrap_or(&0);
        av.push(u64::from(a));
        bv.push(u64::from(d_tot.saturating_sub(a)));
        cv.push(u64::from(e_tot.saturating_sub(a)));
        dv.push(u64::from(total.saturating_sub(d_tot + e_tot - a)));
        dn.push(DrugName(d));
        en.push(EventName(e));
    }

    ContingencyBatch {
        drugs: dn,
        events: en,
        tables: BatchContingencyTables::new(av, bv, cv, dv),
    }
}

/// Run signal detection on a [`ContingencyBatch`].
pub fn run_signal_detection(batch: &ContingencyBatch) -> Result<Vec<SignalDetectionResult>> {
    let complete = batch_complete_parallel(&batch.tables);
    let results: Vec<SignalDetectionResult> = (0..batch.tables.len())
        .into_par_iter()
        .map(|i| map_complete_to_result(batch, &complete[i], i))
        .collect();
    Ok(results)
}

fn map_complete_to_result(
    batch: &ContingencyBatch,
    r: &CompleteSignalResult,
    i: usize,
) -> SignalDetectionResult {
    SignalDetectionResult {
        drug: batch.drugs[i].clone(),
        event: batch.events[i].clone(),
        case_count: CaseCount(batch.tables.a[i]),
        prr: make_assessment(Prr::new_unchecked(r.prr.point_estimate), &r.prr),
        ror: make_assessment(Ror::new_unchecked(r.ror.point_estimate), &r.ror),
        ic: make_assessment(Ic::new_unchecked(r.ic.point_estimate), &r.ic),
        ebgm: make_assessment(Ebgm::new_unchecked(r.ebgm.point_estimate), &r.ebgm),
    }
}

fn make_assessment<M>(
    point: M,
    br: &nexcore_vigilance::pv::signals::batch::BatchResult,
) -> MetricAssessment<M> {
    MetricAssessment {
        point,
        lower_ci: br.lower_ci,
        upper_ci: Some(br.upper_ci),
        is_signal: br.is_signal,
    }
}

/// Filter results to only include detected signals.
pub fn filter_signals(results: &[SignalDetectionResult]) -> Vec<&SignalDetectionResult> {
    results.iter().filter(|r| r.is_any_signal()).collect()
}

/// Convert signal detection results to DataFrame (flat Parquet schema).
pub fn signals_to_dataframe(results: &[SignalDetectionResult]) -> Result<DataFrame> {
    DataFrame::new(vec![
        col_str(columns::DRUG, results, |r| r.drug.as_str()),
        col_str(columns::EVENT, results, |r| r.event.as_str()),
        col_u64(columns::N, results, |r| r.case_count.value()),
        col_f64(columns::PRR, results, |r| r.prr.point.value()),
        col_f64(columns::PRR_LOWER_CI, results, |r| r.prr.lower_ci),
        col_f64(columns::PRR_UPPER_CI, results, |r| {
            r.prr.upper_ci.unwrap_or(0.0)
        }),
        col_bool(columns::PRR_SIGNAL, results, |r| r.prr.is_signal),
        col_f64(columns::ROR, results, |r| r.ror.point.value()),
        col_f64(columns::ROR_LOWER_CI, results, |r| r.ror.lower_ci),
        col_bool(columns::ROR_SIGNAL, results, |r| r.ror.is_signal),
        col_f64(columns::IC, results, |r| r.ic.point.value()),
        col_f64(columns::IC025, results, |r| r.ic.lower_ci),
        col_bool(columns::IC_SIGNAL, results, |r| r.ic.is_signal),
        col_f64(columns::EBGM, results, |r| r.ebgm.point.value()),
        col_f64(columns::EB05, results, |r| r.ebgm.lower_ci),
        col_bool(columns::EBGM_SIGNAL, results, |r| r.ebgm.is_signal),
    ])
    .context("Failed to create signal results DataFrame")
}

// Column builder helpers for signals_to_dataframe
fn col_str<'a>(
    name: &str,
    r: &'a [SignalDetectionResult],
    f: impl Fn(&'a SignalDetectionResult) -> &'a str,
) -> Column {
    Series::new(name.into(), r.iter().map(f).collect::<Vec<_>>()).into()
}
fn col_u64(
    name: &str,
    r: &[SignalDetectionResult],
    f: impl Fn(&SignalDetectionResult) -> u64,
) -> Column {
    Series::new(name.into(), r.iter().map(f).collect::<Vec<_>>()).into()
}
fn col_f64(
    name: &str,
    r: &[SignalDetectionResult],
    f: impl Fn(&SignalDetectionResult) -> f64,
) -> Column {
    Series::new(name.into(), r.iter().map(f).collect::<Vec<_>>()).into()
}
fn col_bool(
    name: &str,
    r: &[SignalDetectionResult],
    f: impl Fn(&SignalDetectionResult) -> bool,
) -> Column {
    Series::new(name.into(), r.iter().map(f).collect::<Vec<_>>()).into()
}

/// Run complete signal detection pipeline on aggregated counts.
pub fn run_signal_detection_pipeline(counts_df: &DataFrame) -> Result<Vec<SignalDetectionResult>> {
    let batch = build_contingency_tables_from_counts(counts_df)?;
    run_signal_detection(&batch)
}

/// End-to-end pipeline result (opaque to callers that don't use Polars).
pub struct PipelineOutput {
    /// All signal detection results
    pub results: Vec<SignalDetectionResult>,
    /// Total drug-event pairs evaluated
    pub total_pairs: usize,
}

/// Run the full ETL pipeline: ingest → normalize → count → filter → detect.
///
/// This is the main entry point for callers that don't want Polars types.
/// All Polars operations are contained within this function.
pub fn run_full_pipeline(
    faers_dir: &Path,
    include_all_roles: bool,
    min_cases: i64,
) -> Result<PipelineOutput> {
    let df = ingest_faers_quarterly_with_options(faers_dir, include_all_roles)?;
    if df.height() == 0 {
        return Ok(PipelineOutput {
            results: Vec::new(),
            total_pairs: 0,
        });
    }

    let lazy = transform_normalize_names(df.lazy())?;
    let counted = transform_count_drug_events(lazy)?;
    let filtered = transform_filter_minimum_n(counted, min_cases)?;
    let counts_df = filtered
        .collect()
        .context("Failed to collect filtered counts")?;
    let total_pairs = counts_df.height();
    let results = run_signal_detection_pipeline(&counts_df)?;

    Ok(PipelineOutput {
        results,
        total_pairs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signal(drug: &str, is_prr: bool) -> SignalDetectionResult {
        let pv = if is_prr { 3.0 } else { 1.0 };
        let lo = if is_prr { 2.0 } else { 0.5 };
        let zero_ror = MetricAssessment {
            point: Ror::new_unchecked(0.0),
            lower_ci: 0.0,
            upper_ci: Some(0.0),
            is_signal: false,
        };
        let zero_ic = MetricAssessment {
            point: Ic::new_unchecked(0.0),
            lower_ci: 0.0,
            upper_ci: Some(0.0),
            is_signal: false,
        };
        let zero_ebgm = MetricAssessment {
            point: Ebgm::new_unchecked(0.0),
            lower_ci: 0.0,
            upper_ci: Some(0.0),
            is_signal: false,
        };
        SignalDetectionResult {
            drug: DrugName(drug.to_string()),
            event: EventName("EVENT".to_string()),
            case_count: CaseCount(10),
            prr: MetricAssessment {
                point: Prr::new_unchecked(pv),
                lower_ci: lo,
                upper_ci: Some(4.0),
                is_signal: is_prr,
            },
            ror: zero_ror,
            ic: zero_ic,
            ebgm: zero_ebgm,
        }
    }

    #[test]
    fn test_count_drug_events() {
        let df = DataFrame::new(vec![
            Series::new(columns::CASE_ID.into(), vec![1u64, 2, 3, 4]).into(),
            Series::new(columns::DRUG.into(), vec!["ASP", "ASP", "ASP", "MET"]).into(),
            Series::new(columns::EVENT.into(), vec!["HA", "HA", "HA", "NA"]).into(),
        ])
        .unwrap_or_else(|e| panic!("{e}"));

        let c = transform_count_drug_events(df.lazy())
            .unwrap_or_else(|e| panic!("{e}"))
            .collect()
            .unwrap_or_else(|e| panic!("{e}"));

        let ah = c
            .clone()
            .lazy()
            .filter(
                col(columns::DRUG)
                    .eq(lit("ASP"))
                    .and(col(columns::EVENT).eq(lit("HA"))),
            )
            .collect()
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(ah.height(), 1);
    }

    #[test]
    fn test_filter_minimum() {
        let df = DataFrame::new(vec![
            Series::new(columns::DRUG.into(), vec!["A", "B"]).into(),
            Series::new(columns::EVENT.into(), vec!["X", "Y"]).into(),
            Series::new(columns::N.into(), vec![5u32, 2]).into(),
        ])
        .unwrap_or_else(|e| panic!("{e}"));
        let c = transform_filter_minimum(df.lazy())
            .unwrap_or_else(|e| panic!("{e}"))
            .collect()
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(c.height(), 1);
    }

    #[test]
    fn test_contingency_tables() {
        let df = DataFrame::new(vec![
            Series::new(columns::DRUG.into(), vec!["DA", "DA", "DB"]).into(),
            Series::new(columns::EVENT.into(), vec!["EX", "EY", "EX"]).into(),
            Series::new(columns::N.into(), vec![10u32, 5, 8]).into(),
        ])
        .unwrap_or_else(|e| panic!("{e}"));
        let b = build_contingency_tables_from_counts(&df).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(b.drugs.len(), 3);
        assert_eq!(b.tables.a[0], 10);
        assert_eq!(b.tables.b[0], 5);
        assert_eq!(b.tables.c[0], 8);
        assert_eq!(b.tables.d[0], 0);
    }

    #[test]
    fn test_pipeline_end_to_end() {
        let df = DataFrame::new(vec![
            Series::new(columns::DRUG.into(), vec!["A", "A", "B", "B", "C", "C"]).into(),
            Series::new(columns::EVENT.into(), vec!["X", "Y", "X", "Y", "X", "Y"]).into(),
            Series::new(columns::N.into(), vec![50u32, 5, 10, 100, 20, 500]).into(),
        ])
        .unwrap_or_else(|e| panic!("{e}"));
        let r = run_signal_detection_pipeline(&df).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(r.len(), 6);
        let ax = r
            .iter()
            .find(|r| r.drug.as_str() == "A" && r.event.as_str() == "X");
        assert!(ax.is_some());
        assert_eq!(
            ax.unwrap_or_else(|| panic!("missing")).case_count.value(),
            50
        );
    }

    #[test]
    fn test_to_dataframe() {
        let r = test_signal("D", true);
        let df = signals_to_dataframe(&[r]).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(df.height(), 1);
        assert_eq!(df.width(), 16);
    }

    #[test]
    fn test_filter_signals() {
        let r = vec![test_signal("SIG", true), test_signal("NO", false)];
        let s = filter_signals(&r);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].drug.as_str(), "SIG");
    }
}
