use clap::Parser;
use nexcore_error::{Context, Result};
use nexcore_faers_etl::{RowCount, SignalDetectionResult, columns};
use polars::prelude::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "faers-pipeline")]
#[command(about = "Run FAERS ETL -> counts -> signal detection")]
struct Args {
    #[arg(long)]
    faers_dir: PathBuf,
    #[arg(long, default_value = "output/faers_counts.parquet")]
    counts_out: PathBuf,
    #[arg(long, default_value = "output/faers_signals.parquet")]
    signals_out: PathBuf,
    #[arg(long, default_value = "3")]
    min_cases: i64,
    #[arg(long)]
    include_all_roles: bool,
    #[arg(long)]
    no_summary: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    validate_args(&args)?;
    let (counts, counts_written) = run_etl_phase(&args)?;
    if !args.no_summary {
        print_summary(&counts)?;
    }
    run_signal_phase(&args, &counts)?;
    println!("counts_rows: {counts_written}");
    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if !args.faers_dir.exists() {
        nexcore_error::bail!("FAERS directory not found: {}", args.faers_dir.display());
    }
    Ok(())
}

fn run_etl_phase(args: &Args) -> Result<(DataFrame, RowCount)> {
    let raw = nexcore_faers_etl::ingest_faers_quarterly_with_options(
        &args.faers_dir,
        args.include_all_roles,
    )
    .context("Ingest failed")?;

    let lazy = nexcore_faers_etl::transform_normalize_names(raw.lazy())?;
    let lazy = nexcore_faers_etl::transform_count_drug_events(lazy)?;
    let lazy = nexcore_faers_etl::transform_filter_minimum_n(lazy, args.min_cases)?;

    let counts = lazy.clone().collect().context("Failed to collect counts")?;
    let out_str = args.counts_out.display().to_string();
    let written = nexcore_faers_etl::sink_parquet_output_to(lazy, &out_str)?;
    println!("counts_parquet: {}", args.counts_out.display());
    Ok((counts, written))
}

fn run_signal_phase(args: &Args, counts: &DataFrame) -> Result<()> {
    let signals = nexcore_faers_etl::run_signal_detection_pipeline(counts)?;
    let written = nexcore_faers_etl::sink_signals_parquet(&signals, &args.signals_out)?;
    print_signal_stats(&signals, &args.signals_out, written);
    Ok(())
}

fn print_signal_stats(signals: &[SignalDetectionResult], path: &PathBuf, written: RowCount) {
    let prr = signals.iter().filter(|r| r.prr.is_signal).count();
    let ror = signals.iter().filter(|r| r.ror.is_signal).count();
    let ic = signals.iter().filter(|r| r.ic.is_signal).count();
    let ebgm = signals.iter().filter(|r| r.ebgm.is_signal).count();
    println!("signals_parquet: {}", path.display());
    println!("signals_total: {}", signals.len());
    println!("signals_prr: {prr}");
    println!("signals_ror: {ror}");
    println!("signals_ic: {ic}");
    println!("signals_ebgm: {ebgm}");
    println!("signals_rows_written: {written}");
}

fn print_summary(counts: &DataFrame) -> Result<()> {
    print_basic_info(counts);
    print_n_stats(counts)?;
    print_distinct_counts(counts)?;
    print_top_pairs(counts)?;
    print_top_by_column(counts, columns::DRUG, "drugs")?;
    print_top_by_column(counts, columns::EVENT, "events")?;
    Ok(())
}

fn print_basic_info(counts: &DataFrame) {
    println!("counts_columns: {:?}", counts.get_column_names());
    println!("counts_dtypes: {:?}", counts.dtypes());
    println!("\nnull_counts:\n{}", counts.null_count());
}

fn print_n_stats(counts: &DataFrame) -> Result<()> {
    let stats = counts
        .clone()
        .lazy()
        .select([
            col(columns::N).min().alias("min"),
            col(columns::N).max().alias("max"),
            col(columns::N).mean().alias("mean"),
            col(columns::N)
                .quantile(lit(0.50), QuantileMethod::Nearest)
                .alias("p50"),
            col(columns::N)
                .quantile(lit(0.90), QuantileMethod::Nearest)
                .alias("p90"),
            col(columns::N)
                .quantile(lit(0.99), QuantileMethod::Nearest)
                .alias("p99"),
        ])
        .collect()
        .context("n stats")?;
    println!("\nfield:n stats:\n{stats}");
    Ok(())
}

fn print_distinct_counts(counts: &DataFrame) -> Result<()> {
    let dc = counts
        .clone()
        .lazy()
        .select([
            col(columns::DRUG).n_unique().alias("distinct_drugs"),
            col(columns::EVENT).n_unique().alias("distinct_events"),
        ])
        .collect()
        .context("distinct counts")?;
    println!("\ndistinct counts:\n{dc}");
    Ok(())
}

fn print_top_pairs(counts: &DataFrame) -> Result<()> {
    let top = counts
        .clone()
        .lazy()
        .sort(
            [columns::N],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .limit(15)
        .collect()
        .context("top pairs")?;
    println!("\nTop 15 pairs by n:\n{top}");
    Ok(())
}

fn print_top_by_column(counts: &DataFrame, col_name: &str, label: &str) -> Result<()> {
    let top = counts
        .clone()
        .lazy()
        .group_by([col_name])
        .agg([col(columns::N).sum().alias("total_n")])
        .sort(
            ["total_n"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .limit(15)
        .collect()
        .context("top by column")?;
    println!("\nTop 15 {label} by total_n:\n{top}");
    Ok(())
}
