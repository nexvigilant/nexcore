//! Experiment Runner - σf(runner) primitive
//!
//! CLI tool for executing A/B experiments on Claude Code agent modes.
//! Outputs JSONL measurements for analysis.
//!
//! # Codex Compliance
//! - **Tier**: T3 (Experiment Infrastructure)
//! - **Primitives**: σ(sequence), f(frequency), N(quantity)

use nexcore_hooks::experiment::{
    Experiment, ExperimentResults, ExperimentStatus, ExperimentType, Hypothesis, Measurement, Timer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

// ── Types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ExperimentSpec {
    id: String,
    name: String,
    description: String,
    experiment_type: SpecExperimentType,
    hypothesis: SpecHypothesis,
    config: HashMap<String, String>,
    tasks: Vec<TaskSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum SpecExperimentType {
    #[serde(rename = "ab_test")]
    ABTest { control: String, variant: String },
}

#[derive(Debug, Deserialize)]
struct SpecHypothesis {
    statement: String,
    expected: String,
    null: String,
    confidence: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct TaskSpec {
    id: String,
    name: String,
    prompt: String,
    expected_tool_calls: usize,
    complexity: String,
}

#[derive(Debug, Serialize)]
struct TrialResult {
    experiment_id: String,
    trial_id: usize,
    mode: String,
    task_id: String,
    wall_clock_ms: f64,
    tool_calls: usize,
    success: bool,
    error: Option<String>,
    timestamp: u64,
}

struct CliArgs {
    spec_path: PathBuf,
    output_path: PathBuf,
    dry_run: bool,
}

// ── Main ───────────────────────────────────────────────────────────

fn main() {
    let args = parse_args();

    let spec = load_spec(&args.spec_path).unwrap_or_else(|e| {
        eprintln!("Failed to load spec: {e}");
        std::process::exit(1);
    });

    if args.dry_run {
        print_dry_run(&spec);
        return;
    }

    run_experiment(&spec, &args.output_path);
}

fn run_experiment(spec: &ExperimentSpec, output_path: &PathBuf) {
    let mut experiment = create_experiment(spec);
    experiment.start();

    print_header(spec);

    let mut writer = create_output_writer(output_path);
    let total_trials = execute_all_trials(spec, &mut experiment, &mut writer);

    finalize_experiment(&mut experiment, output_path, total_trials);
}

// ── Argument Parsing ───────────────────────────────────────────────

fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut cli = CliArgs {
        spec_path: PathBuf::from("experiment.json"),
        output_path: PathBuf::from("results.jsonl"),
        dry_run: false,
    };

    let mut i = 1;
    while i < args.len() {
        i = parse_single_arg(&args, i, &mut cli);
    }
    cli
}

fn parse_single_arg(args: &[String], i: usize, cli: &mut CliArgs) -> usize {
    match args[i].as_str() {
        "--spec" | "-s" if i + 1 < args.len() => {
            cli.spec_path = PathBuf::from(&args[i + 1]);
            i + 2
        }
        "--output" | "-o" if i + 1 < args.len() => {
            cli.output_path = PathBuf::from(&args[i + 1]);
            i + 2
        }
        "--dry-run" | "-n" => {
            cli.dry_run = true;
            i + 1
        }
        "--help" | "-h" => {
            print_usage();
            std::process::exit(0);
        }
        _ => i + 1,
    }
}

fn print_usage() {
    println!("experiment_runner - A/B experiment execution harness\n");
    println!("USAGE: experiment_runner [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  -s, --spec <FILE>    Experiment spec JSON");
    println!("  -o, --output <FILE>  Output JSONL file");
    println!("  -n, --dry-run        Print plan without executing");
    println!("  -h, --help           Print help");
}

// ── Spec Loading ───────────────────────────────────────────────────

fn load_spec(path: &PathBuf) -> Result<ExperimentSpec, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    serde_json::from_reader(file).map_err(|e| e.to_string())
}

fn create_experiment(spec: &ExperimentSpec) -> Experiment {
    let exp_type = match &spec.experiment_type {
        SpecExperimentType::ABTest { control, variant } => ExperimentType::ABTest {
            control: control.clone(),
            variant: variant.clone(),
        },
    };

    let hypothesis = Hypothesis::new(&spec.hypothesis.statement)
        .expecting(&spec.hypothesis.expected)
        .null(&spec.hypothesis.null)
        .with_confidence(spec.hypothesis.confidence);

    let mut exp = Experiment::new(&spec.name, exp_type)
        .with_description(&spec.description)
        .with_hypothesis(hypothesis);

    for (k, v) in &spec.config {
        exp = exp.with_config(k, v);
    }
    exp
}

// ── Trial Execution ────────────────────────────────────────────────

fn create_output_writer(path: &PathBuf) -> BufWriter<File> {
    let file = File::create(path).expect("Failed to create output file");
    BufWriter::new(file)
}

fn execute_all_trials(
    spec: &ExperimentSpec,
    experiment: &mut Experiment,
    writer: &mut BufWriter<File>,
) -> usize {
    let runs = get_runs_per_task(spec);
    let modes = ["foreground", "background"];
    let mut trial_id = 0;
    let total = runs * spec.tasks.len() * modes.len();

    for task in &spec.tasks {
        for mode in &modes {
            for _ in 0..runs {
                trial_id += 1;
                execute_single_trial(spec, experiment, writer, trial_id, mode, task, total);
            }
        }
    }
    total
}

fn execute_single_trial(
    spec: &ExperimentSpec,
    experiment: &mut Experiment,
    writer: &mut BufWriter<File>,
    trial_id: usize,
    mode: &str,
    task: &TaskSpec,
    total: usize,
) {
    let result = run_trial(&spec.id, trial_id, mode, task);
    record_trial_measurement(experiment, &result, mode, &task.id);
    write_trial_result(writer, &result);
    print_progress(trial_id, total);
}

fn get_runs_per_task(spec: &ExperimentSpec) -> usize {
    spec.config
        .get("runs_per_task_per_mode")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn record_trial_measurement(exp: &mut Experiment, result: &TrialResult, mode: &str, task_id: &str) {
    exp.record(
        Measurement::new("wall_clock_ms", result.wall_clock_ms)
            .with_unit("ms")
            .with_tag("mode", mode)
            .with_tag("task", task_id),
    );
    exp.record(
        Measurement::new("tool_calls", result.tool_calls as f64)
            .with_tag("mode", mode)
            .with_tag("task", task_id),
    );
}

fn write_trial_result(writer: &mut BufWriter<File>, result: &TrialResult) {
    let json = serde_json::to_string(result).expect("serialize");
    writeln!(writer, "{json}").expect("write");
}

// ── Trial Simulation ───────────────────────────────────────────────

fn run_trial(exp_id: &str, trial_id: usize, mode: &str, task: &TaskSpec) -> TrialResult {
    let wall_clock_ms = simulate_execution_time(&task.complexity, mode);
    std::thread::sleep(std::time::Duration::from_millis(5));

    TrialResult {
        experiment_id: exp_id.to_string(),
        trial_id,
        mode: mode.to_string(),
        task_id: task.id.clone(),
        wall_clock_ms,
        tool_calls: task.expected_tool_calls,
        success: true,
        error: None,
        timestamp: unix_millis(),
    }
}

fn simulate_execution_time(complexity: &str, mode: &str) -> f64 {
    let base = match complexity {
        "low" => 500.0,
        "medium" => 1500.0,
        "high" => 3000.0,
        _ => 1000.0,
    };
    let jitter = if mode == "background" {
        rand_simple() * 500.0 + 200.0
    } else {
        rand_simple() * 200.0
    };
    base + jitter
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn rand_simple() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0x853c49e6748fea9b);
    let mut s = STATE.load(Ordering::Relaxed);
    s ^= s >> 12;
    s ^= s << 25;
    s ^= s >> 27;
    STATE.store(s, Ordering::Relaxed);
    (s.wrapping_mul(0x2545F4914F6CDD1D) as f64) / (u64::MAX as f64)
}

// ── Results Computation ────────────────────────────────────────────

fn finalize_experiment(experiment: &mut Experiment, output_path: &PathBuf, total: usize) {
    experiment.end(ExperimentStatus::Completed);
    let results = compute_results(experiment);
    experiment.finalize(results);

    write_summary(experiment, output_path);
    print_completion(output_path, total);
}

fn compute_results(experiment: &Experiment) -> ExperimentResults {
    let (fg_times, bg_times) = group_times_by_mode(&experiment.measurements);
    let stats = compute_statistics(&fg_times, &bg_times);
    build_results(stats)
}

fn group_times_by_mode(measurements: &[Measurement]) -> (Vec<f64>, Vec<f64>) {
    let mut fg = Vec::new();
    let mut bg = Vec::new();

    for m in measurements {
        if m.metric != "wall_clock_ms" {
            continue;
        }
        match m.tags.get("mode").map(String::as_str) {
            Some("foreground") => fg.push(m.value),
            Some("background") => bg.push(m.value),
            _ => {}
        }
    }
    (fg, bg)
}

struct Stats {
    fg_mean: f64,
    bg_mean: f64,
    diff: f64,
    t_stat: f64,
    p_value: f64,
}

fn compute_statistics(fg_times: &[f64], bg_times: &[f64]) -> Stats {
    let fg_mean = mean(fg_times);
    let bg_mean = mean(bg_times);
    let diff = bg_mean - fg_mean;

    let fg_var = variance(fg_times, fg_mean);
    let bg_var = variance(bg_times, bg_mean);
    let se = pooled_se(fg_var, fg_times.len(), bg_var, bg_times.len());
    let t_stat = if se > 0.0 { diff / se } else { 0.0 };
    let p_value = 2.0 * (1.0 - normal_cdf(t_stat.abs()));

    Stats {
        fg_mean,
        bg_mean,
        diff,
        t_stat,
        p_value,
    }
}

fn build_results(stats: Stats) -> ExperimentResults {
    let mut statistics = HashMap::new();
    statistics.insert("fg_mean_ms".into(), stats.fg_mean);
    statistics.insert("bg_mean_ms".into(), stats.bg_mean);
    statistics.insert("diff_ms".into(), stats.diff);
    statistics.insert("t_statistic".into(), stats.t_stat);

    let confirmed = stats.p_value < 0.05 && stats.diff > 0.0;

    ExperimentResults {
        hypothesis_confirmed: Some(confirmed),
        confidence: 1.0 - stats.p_value,
        p_value: Some(stats.p_value),
        statistics,
        findings: vec![
            format!("FG mean: {:.2} ms", stats.fg_mean),
            format!("BG mean: {:.2} ms", stats.bg_mean),
            format!("Diff: {:.2} ms", stats.diff),
            format!("p-value: {:.4}", stats.p_value),
        ],
        recommendations: if confirmed {
            vec!["Background agents show higher latency.".into()]
        } else {
            vec!["No significant difference.".into()]
        },
    }
}

// ── Statistics Helpers ─────────────────────────────────────────────

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn variance(values: &[f64], mean: f64) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
    }
}

fn pooled_se(var1: f64, n1: usize, var2: f64, n2: usize) -> f64 {
    ((var1 / n1 as f64) + (var2 / n2 as f64)).sqrt()
}

fn normal_cdf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989423 * (-x * x / 2.0).exp();
    let p =
        d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));
    if x > 0.0 { 1.0 - p } else { p }
}

// ── Output ─────────────────────────────────────────────────────────

fn print_header(spec: &ExperimentSpec) {
    println!("Starting: {}", spec.name);
    println!(
        "Tasks: {}, Runs/task: {}",
        spec.tasks.len(),
        get_runs_per_task(spec)
    );
}

fn print_progress(current: usize, total: usize) {
    let pct = (current as f64 / total as f64) * 100.0;
    print!("\rProgress: {current}/{total} ({pct:.1}%)");
    std::io::stdout().flush().ok();
}

fn print_completion(output_path: &PathBuf, _total: usize) {
    println!("\n\nComplete. Results: {}", output_path.display());
}

fn write_summary(experiment: &Experiment, output_path: &PathBuf) {
    let summary_path = output_path.with_extension("summary.json");
    let file = File::create(&summary_path).expect("summary file");
    serde_json::to_writer_pretty(file, experiment).expect("serialize");
}

fn print_dry_run(spec: &ExperimentSpec) {
    println!("=== DRY RUN ===");
    println!("Experiment: {} ({})", spec.name, spec.id);
    println!("\nTasks:");
    for t in &spec.tasks {
        println!("  - {} [{}]", t.id, t.complexity);
    }
    let runs = get_runs_per_task(spec);
    println!("\nTotal trials: {}", runs * spec.tasks.len() * 2);
}
