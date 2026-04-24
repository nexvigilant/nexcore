//! nexcore-vigil-train — CLI for the full-Rust sovereign LoRA pipeline.
//!
//! Mirrors the Python `vigil-lora-train` subcommand surface so callers can
//! swap between the two stacks one-for-one during the Rust transition.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use candle_core::DType;
use clap::{Parser, Subcommand};
use nexcore_error::{NexError, Result};
use nexcore_vigil_train as vt;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "nexcore-vigil-train",
    version,
    about = "Full-Rust sovereign LoRA pipeline (inference + measurement shipped; SFT/DPO/merge scaffolded)."
)]
struct Cli {
    #[arg(long, global = true)]
    tokenizer: Option<PathBuf>,

    #[arg(long, global = true)]
    gguf: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Load the quantized model and generate one completion for a prompt.
    Infer {
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        system: Option<String>,
        #[arg(long, default_value_t = 0.7)]
        temperature: f64,
        #[arg(long, default_value_t = 512)]
        max_new_tokens: usize,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },

    /// Interactive REPL: loads the model ONCE, then reads prompts from stdin
    /// and writes JSONL responses to stdout. Stays alive until EOF (Ctrl-D).
    Repl {
        #[arg(long)]
        model_dir: Option<PathBuf>,
        #[arg(long, default_value = "f32")]
        dtype: String,
        #[arg(long)]
        system: Option<String>,
        #[arg(long, default_value_t = 0.3)]
        temperature: f64,
        #[arg(long, default_value_t = 512)]
        max_new_tokens: usize,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Render output as human-readable text instead of JSONL.
        #[arg(long)]
        pretty: bool,
    },

    /// Load a non-quantized (fp16/bf16) model directory and generate once.
    InferNative {
        /// Path to a HF-shaped model dir containing config.json + model.safetensors.
        #[arg(long)]
        model_dir: PathBuf,
        /// Precision: f32 | f16 | bf16 (default bf16 — 13th-gen avx_vnni).
        #[arg(long, default_value = "bf16")]
        dtype: String,
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        system: Option<String>,
        #[arg(long, default_value_t = 0.7)]
        temperature: f64,
        #[arg(long, default_value_t = 128)]
        max_new_tokens: usize,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },

    /// Filter a SFT JSONL to rows whose full ChatML tokenization fits under `--max-tokens`.
    /// Writes kept rows to `--output-jsonl` and reports kept/dropped counts.
    FilterDataset {
        #[arg(
            long,
            default_value = "/home/matthew/.claude/brain/vigil-training-set.jsonl"
        )]
        source_jsonl: PathBuf,
        #[arg(
            long,
            default_value = "/home/matthew/.claude/brain/vigil-training-set-filtered.jsonl"
        )]
        output_jsonl: PathBuf,
        #[arg(long, default_value_t = 1024)]
        max_tokens: usize,
    },

    /// Report primitive stratification on a JSONL dataset (inverse-freq weights).
    StratifyReport {
        #[arg(
            long,
            default_value = "/home/matthew/.claude/brain/vigil-training-set.jsonl"
        )]
        source_jsonl: PathBuf,
    },

    /// Print a LoRA bundle's parameter count for a given Qwen2 config.
    LoraInfo {
        #[arg(long, default_value_t = 28)]
        num_layers: usize,
        #[arg(long, default_value_t = 1536)]
        hidden_size: usize,
        #[arg(long, default_value_t = 12)]
        num_attention_heads: usize,
        #[arg(long, default_value_t = 2)]
        num_key_value_heads: usize,
        #[arg(long, default_value_t = 8)]
        lora_r: usize,
        #[arg(long, default_value_t = 16.0)]
        lora_alpha: f64,
    },

    /// Sample N SFT rows, generate a completion per row, grade via rsk mcg test.
    Measure {
        #[arg(
            long,
            default_value = "/home/matthew/.claude/brain/vigil-training-set.jsonl"
        )]
        source_jsonl: PathBuf,
        #[arg(short = 'n', long, default_value_t = 10)]
        count: usize,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 0.3)]
        temperature: f64,
        #[arg(long, default_value_t = 2048)]
        max_new_tokens: usize,
        #[arg(long)]
        verbose: bool,
    },

    /// SFT fine-tune via Candle LoRA. Requires base model at `--base-model-dir`.
    Train {
        #[arg(long)]
        base_model_dir: Option<PathBuf>,
        #[arg(long)]
        train_jsonl: Option<PathBuf>,
        #[arg(long)]
        output_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 300)]
        steps: u32,
        #[arg(long, default_value_t = 1e-4)]
        lr: f64,
        #[arg(long, default_value_t = 1)]
        batch_size: usize,
        #[arg(long, default_value_t = 8)]
        grad_accum: u32,
        #[arg(long, default_value_t = 8)]
        lora_r: usize,
        #[arg(long, default_value_t = 16.0)]
        lora_alpha: f64,
        #[arg(long, default_value_t = 2048)]
        max_seq_len: usize,
        #[arg(long, default_value_t = 10)]
        log_every: u32,
        #[arg(long, default_value_t = 100)]
        save_every: u32,
        /// Stratification: uniform | inverse-freq (primitive-weighted)
        #[arg(long, default_value = "inverse-freq")]
        stratification: String,
        #[arg(long, default_value = "bf16")]
        dtype: String,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },

    /// DPO fine-tune (Phase R3 — not implemented, errors cleanly).
    DpoTrain,

    /// Merge LoRA adapter into base weights, producing a standalone model dir.
    Merge {
        #[arg(long)]
        base_model_dir: Option<PathBuf>,
        #[arg(long)]
        adapter: Option<PathBuf>,
        #[arg(long)]
        output_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 16)]
        lora_r: usize,
        #[arg(long, default_value_t = 32.0)]
        lora_alpha: f64,
    },

    /// Run the Claude-compatible HTTP inference server (POST /v1/messages).
    Serve {
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: String,
        #[arg(long)]
        model_dir: Option<PathBuf>,
        #[arg(long, default_value = "f32")]
        dtype: String,
        #[arg(long, default_value = "vigil-qwen-v1")]
        model_id: String,
    },
}

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    let cli = Cli::parse();
    let rc = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    };
    std::process::exit(rc);
}

fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::Infer {
            prompt,
            system,
            temperature,
            max_new_tokens,
            seed,
        } => cmd_infer(
            cli.tokenizer.as_deref(),
            cli.gguf.as_deref(),
            prompt,
            system,
            temperature,
            max_new_tokens,
            seed,
        ),
        Cmd::Repl {
            model_dir,
            dtype,
            system,
            temperature,
            max_new_tokens,
            seed,
            pretty,
        } => cmd_repl(
            cli.tokenizer.as_deref(),
            model_dir.as_deref(),
            &dtype,
            system,
            temperature,
            max_new_tokens,
            seed,
            pretty,
        ),
        Cmd::InferNative {
            model_dir,
            dtype,
            prompt,
            system,
            temperature,
            max_new_tokens,
            seed,
        } => cmd_infer_native(
            cli.tokenizer.as_deref(),
            &model_dir,
            &dtype,
            prompt,
            system,
            temperature,
            max_new_tokens,
            seed,
        ),
        Cmd::FilterDataset {
            source_jsonl,
            output_jsonl,
            max_tokens,
        } => cmd_filter_dataset(
            cli.tokenizer.as_deref(),
            &source_jsonl,
            &output_jsonl,
            max_tokens,
        ),
        Cmd::StratifyReport { source_jsonl } => cmd_stratify(&source_jsonl),
        Cmd::LoraInfo {
            num_layers,
            hidden_size,
            num_attention_heads,
            num_key_value_heads,
            lora_r,
            lora_alpha,
        } => cmd_lora_info(
            num_layers,
            hidden_size,
            num_attention_heads,
            num_key_value_heads,
            lora_r,
            lora_alpha,
        ),
        Cmd::Measure {
            source_jsonl,
            count,
            seed,
            temperature,
            max_new_tokens,
            verbose,
        } => cmd_measure(
            cli.tokenizer.as_deref(),
            cli.gguf.as_deref(),
            &source_jsonl,
            count,
            seed,
            temperature,
            max_new_tokens,
            verbose,
        ),
        Cmd::Train {
            base_model_dir,
            train_jsonl,
            output_dir,
            steps,
            lr,
            batch_size,
            grad_accum,
            lora_r,
            lora_alpha,
            max_seq_len,
            log_every,
            save_every,
            stratification,
            dtype,
            seed,
        } => cmd_train(
            base_model_dir,
            train_jsonl,
            output_dir,
            steps,
            lr,
            batch_size,
            grad_accum,
            lora_r,
            lora_alpha,
            max_seq_len,
            log_every,
            save_every,
            &stratification,
            &dtype,
            seed,
        ),
        Cmd::DpoTrain => vt::dpo::run(&vt::dpo::DpoConfig::default()),
        Cmd::Merge {
            base_model_dir,
            adapter,
            output_dir,
            lora_r,
            lora_alpha,
        } => {
            let mut mcfg = vt::merge::MergeConfig::default();
            if let Some(p) = base_model_dir {
                mcfg.base_model_dir = p;
            }
            if let Some(p) = adapter {
                mcfg.adapter_path = p;
            }
            if let Some(p) = output_dir {
                mcfg.output_dir = p;
            }
            mcfg.lora_r = lora_r;
            mcfg.lora_alpha = lora_alpha;
            vt::merge::run(&mcfg)
        }
        Cmd::Serve {
            bind,
            model_dir,
            dtype,
            model_id,
        } => {
            let mut scfg = vt::serve::ServeConfig::default();
            scfg.bind = bind;
            if let Some(p) = model_dir {
                scfg.model_dir = p;
            }
            scfg.dtype = parse_dtype(&dtype)?;
            scfg.model_id = model_id;
            scfg.tokenizer_path = cli.tokenizer.clone();
            vt::serve::run(&scfg)
        }
    }
}

fn read_stdin() -> Result<String> {
    use std::io::Read;
    let mut s = String::new();
    std::io::stdin()
        .read_to_string(&mut s)
        .map_err(|e| NexError::new(format!("stdin read: {e}")))?;
    Ok(s)
}

#[allow(clippy::too_many_arguments)]
fn cmd_infer(
    tokenizer_path: Option<&std::path::Path>,
    gguf_path: Option<&std::path::Path>,
    prompt: Option<String>,
    system: Option<String>,
    temperature: f64,
    max_new_tokens: usize,
    seed: u64,
) -> Result<()> {
    let user_prompt = match prompt {
        Some(p) => p,
        None => read_stdin()?,
    };
    let system_prompt = system.unwrap_or_default();
    let chatml = vt::tokenizer::format_chatml(&system_prompt, &user_prompt);

    let tok = vt::tokenizer::load(tokenizer_path)?;
    let device = vt::model::pick_device()?;
    let gguf = vt::model::resolve_gguf(gguf_path)?;
    let loaded = vt::model::load_quantized(&gguf, &device)?;
    let mut m = match loaded {
        vt::model::VigilModel::Quantized(w) => w,
        vt::model::VigilModel::Native(_) => {
            return Err(NexError::new(
                "native model path not wired for inference yet — use quantized GGUF",
            ));
        }
    };

    let cfg = vt::inference::SamplingConfig {
        temperature,
        top_p: Some(0.9),
        seed,
        max_new_tokens,
    };
    let t0 = Instant::now();
    let (text, n) = vt::inference::generate(&mut m, &tok, &device, &chatml, &cfg)?;
    let dt_ms = t0.elapsed().as_millis();

    let out = serde_json::json!({
        "model": "qwen2.5:3b-quantized",
        "gguf": gguf.display().to_string(),
        "tokens_generated": n,
        "duration_ms": dt_ms,
        "response": text,
    });
    let stdout = std::io::stdout();
    let mut h = stdout.lock();
    writeln!(
        h,
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| NexError::new(format!("json: {e}")))?
    )
    .map_err(|e| NexError::new(format!("stdout: {e}")))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_measure(
    tokenizer_path: Option<&std::path::Path>,
    gguf_path: Option<&std::path::Path>,
    source: &std::path::Path,
    count: usize,
    seed: u64,
    temperature: f64,
    max_new_tokens: usize,
    verbose: bool,
) -> Result<()> {
    let rows = vt::dataset::read_sft(source)?;
    let n_source = rows.len();
    let mut indexed: Vec<(usize, vt::dataset::SftRow)> = rows.into_iter().enumerate().collect();
    shuffle_indices(&mut indexed, seed);
    indexed.truncate(count);

    let tok = vt::tokenizer::load(tokenizer_path)?;
    let device = vt::model::pick_device()?;
    let gguf = vt::model::resolve_gguf(gguf_path)?;
    let mut m = match vt::model::load_quantized(&gguf, &device)? {
        vt::model::VigilModel::Quantized(w) => w,
        vt::model::VigilModel::Native(_) => {
            return Err(NexError::new(
                "native model path not wired for inference yet — use quantized GGUF",
            ));
        }
    };
    let rsk_bin = vt::eval::rsk_path();

    let cfg = vt::inference::SamplingConfig {
        temperature,
        top_p: Some(0.9),
        seed,
        max_new_tokens,
    };

    let t0 = Instant::now();
    let mut results = Vec::with_capacity(indexed.len());
    for (i, (original_idx, row)) in indexed.iter().enumerate() {
        let (system, user, _gold) = row.split()?;
        let chatml = vt::tokenizer::format_chatml(system, user);

        let step_t0 = Instant::now();
        let completion = vt::inference::generate(&mut m, &tok, &device, &chatml, &cfg);
        let step_dt = step_t0.elapsed().as_millis() as u64;
        let row_result = match completion {
            Ok((text, _n)) => match vt::eval::grade(&text, &rsk_bin) {
                Ok((passed, failed, total)) => vt::eval::EvalRow {
                    index: *original_idx,
                    source: row.source.clone(),
                    score: if total > 0 {
                        passed as f32 / total as f32
                    } else {
                        0.0
                    },
                    passed,
                    failed,
                    total,
                    duration_ms: step_dt,
                    error: None,
                },
                Err(e) => vt::eval::EvalRow {
                    index: *original_idx,
                    source: row.source.clone(),
                    score: 0.0,
                    passed: 0,
                    failed: 0,
                    total: 0,
                    duration_ms: step_dt,
                    error: Some(format!("grade: {e}")),
                },
            },
            Err(e) => vt::eval::EvalRow {
                index: *original_idx,
                source: row.source.clone(),
                score: 0.0,
                passed: 0,
                failed: 0,
                total: 0,
                duration_ms: step_dt,
                error: Some(format!("generate: {e}")),
            },
        };
        if verbose {
            eprintln!(
                "  [{}/{}] src={:?} score={:.2} {}/{} ({}ms)",
                i + 1,
                indexed.len(),
                row_result.source.as_deref().unwrap_or("?"),
                row_result.score,
                row_result.passed,
                row_result.total,
                row_result.duration_ms
            );
        }
        results.push(row_result);
    }
    let elapsed_s = t0.elapsed().as_secs_f64();
    let summary = vt::eval::aggregate(results, elapsed_s);

    let stdout = std::io::stdout();
    let mut h = stdout.lock();
    let mut out_json =
        serde_json::to_value(&summary).map_err(|e| NexError::new(format!("json: {e}")))?;
    if let Some(obj) = out_json.as_object_mut() {
        obj.insert("source_rows".into(), serde_json::json!(n_source));
    }
    writeln!(
        h,
        "{}",
        serde_json::to_string_pretty(&out_json).map_err(|e| NexError::new(format!("json: {e}")))?
    )
    .map_err(|e| NexError::new(format!("stdout: {e}")))?;
    Ok(())
}

fn parse_stratification(s: &str) -> Result<vt::dataset::Stratification> {
    match s.to_ascii_lowercase().replace('_', "-").as_str() {
        "uniform" => Ok(vt::dataset::Stratification::Uniform),
        "inverse-freq" | "inverse-frequency" | "primitive-weighted" => {
            Ok(vt::dataset::Stratification::InverseFrequency)
        }
        other => Err(NexError::new(format!("unknown stratification: {other}"))),
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_train(
    base_model_dir: Option<PathBuf>,
    train_jsonl: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    steps: u32,
    lr: f64,
    batch_size: usize,
    grad_accum: u32,
    lora_r: usize,
    lora_alpha: f64,
    max_seq_len: usize,
    log_every: u32,
    save_every: u32,
    stratification_str: &str,
    dtype_str: &str,
    seed: u64,
) -> Result<()> {
    let mut cfg = vt::sft::SftConfig::default();
    if let Some(p) = base_model_dir {
        cfg.base_model_dir = p;
    }
    if let Some(p) = train_jsonl {
        cfg.train_jsonl = p;
    }
    if let Some(p) = output_dir {
        cfg.output_dir = p;
    }
    cfg.max_steps = steps;
    cfg.lr = lr;
    cfg.batch_size = batch_size;
    cfg.grad_accum = grad_accum;
    cfg.lora_r = lora_r;
    cfg.lora_alpha = lora_alpha;
    cfg.max_seq_len = max_seq_len;
    cfg.log_every = log_every;
    cfg.save_every = save_every;
    cfg.stratification = parse_stratification(stratification_str)?;
    cfg.dtype = parse_dtype(dtype_str)?;
    cfg.seed = seed;

    let summary = vt::sft::run(&cfg)?;
    let out = serde_json::json!({
        "steps_completed": summary.steps_completed,
        "final_loss": summary.final_loss,
        "output_dir": summary.output_dir.display().to_string(),
        "saved": summary.saved,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| NexError::new(format!("json: {e}")))?
    );
    Ok(())
}

fn cmd_filter_dataset(
    tokenizer_path: Option<&std::path::Path>,
    source: &std::path::Path,
    output: &std::path::Path,
    max_tokens: usize,
) -> Result<()> {
    use std::io::Write as _;
    let rows = vt::dataset::read_sft(source)?;
    let tok = vt::tokenizer::load(tokenizer_path)?;

    let out_file = std::fs::File::create(output)
        .map_err(|e| NexError::new(format!("create {}: {e}", output.display())))?;
    let mut writer = std::io::BufWriter::new(out_file);

    let mut kept = 0usize;
    let mut dropped = 0usize;
    let mut drop_reasons: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for row in &rows {
        match vt::sft::build_example(row, &tok, max_tokens) {
            Ok(_) => {
                let line = serde_json::to_string(row)
                    .map_err(|e| NexError::new(format!("serialize: {e}")))?;
                writeln!(writer, "{line}").map_err(|e| NexError::new(format!("write: {e}")))?;
                kept += 1;
            }
            Err(e) => {
                dropped += 1;
                let msg = e.to_string();
                let bucket = if msg.contains("exceeds max_seq_len") {
                    "too_long"
                } else if msg.contains("missing user") {
                    "missing_user"
                } else if msg.contains("missing assistant") {
                    "missing_assistant"
                } else {
                    "other"
                };
                *drop_reasons.entry(bucket.to_string()).or_insert(0) += 1;
            }
        }
    }
    writer
        .flush()
        .map_err(|e| NexError::new(format!("flush: {e}")))?;

    let out = serde_json::json!({
        "source": source.display().to_string(),
        "output": output.display().to_string(),
        "max_tokens": max_tokens,
        "input_rows": rows.len(),
        "kept": kept,
        "dropped": dropped,
        "drop_reasons": drop_reasons,
        "yield_pct": format!("{:.1}%", kept as f64 * 100.0 / rows.len().max(1) as f64),
    });
    let stdout = std::io::stdout();
    let mut h = stdout.lock();
    writeln!(
        h,
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| NexError::new(format!("json: {e}")))?
    )
    .map_err(|e| NexError::new(format!("stdout: {e}")))?;
    Ok(())
}

fn cmd_stratify(source: &std::path::Path) -> Result<()> {
    let rows = vt::dataset::read_sft(source)?;
    let report_uniform = vt::dataset::report(&rows, vt::dataset::Stratification::Uniform);
    let report_inv = vt::dataset::report(&rows, vt::dataset::Stratification::InverseFrequency);

    let counts_json: Vec<serde_json::Value> = report_inv
        .counts
        .iter()
        .map(|(p, c)| serde_json::json!({"primitive": p, "count": c}))
        .collect();

    let out = serde_json::json!({
        "source": source.display().to_string(),
        "total_rows": rows.len(),
        "unlabeled": report_inv.unlabeled,
        "primitive_counts": counts_json,
        "uniform": {
            "min_weight": report_uniform.min_weight,
            "max_weight": report_uniform.max_weight,
            "max_over_min": report_uniform.max_over_min,
        },
        "inverse_frequency": {
            "min_weight": report_inv.min_weight,
            "max_weight": report_inv.max_weight,
            "max_over_min": report_inv.max_over_min,
            "interpretation": format!(
                "rarest primitive gets {:.1}× the sampling weight of most common",
                report_inv.max_over_min
            ),
        },
    });
    let stdout = std::io::stdout();
    let mut h = stdout.lock();
    writeln!(
        h,
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| NexError::new(format!("json: {e}")))?
    )
    .map_err(|e| NexError::new(format!("stdout: {e}")))?;
    Ok(())
}

fn parse_dtype(s: &str) -> Result<vt::model::NativeDType> {
    match s.to_ascii_lowercase().as_str() {
        "f32" | "fp32" => Ok(vt::model::NativeDType::F32),
        "f16" | "fp16" => Ok(vt::model::NativeDType::F16),
        "bf16" => Ok(vt::model::NativeDType::BF16),
        other => Err(NexError::new(format!("unknown dtype: {other}"))),
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_repl(
    tokenizer_path: Option<&std::path::Path>,
    model_dir: Option<&std::path::Path>,
    dtype_str: &str,
    system: Option<String>,
    temperature: f64,
    max_new_tokens: usize,
    seed: u64,
    pretty: bool,
) -> Result<()> {
    use std::io::BufRead;

    // Resolve model dir: prefer --model-dir, fall back to vigil-qwen-v1 merged,
    // final fallback to the base model.
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    let resolved = match model_dir {
        Some(p) => p.to_path_buf(),
        None => {
            let merged = PathBuf::from(format!("{home}/.claude/brain/vigil-qwen-v1"));
            let base = PathBuf::from(format!("{home}/.claude/brain/vigil-base-v1"));
            if merged.join("model.safetensors").is_file() {
                merged
            } else {
                base
            }
        }
    };

    let dtype = parse_dtype(dtype_str)?;
    let tok = vt::tokenizer::load(tokenizer_path)?;
    let device = vt::model::pick_device()?;

    eprintln!("[vigil-repl] loading model from {}", resolved.display());
    let loaded = vt::model::load_native(&resolved, dtype, &device)?;
    let mut m = match loaded {
        vt::model::VigilModel::Native(m) => m,
        vt::model::VigilModel::Quantized(_) => {
            return Err(NexError::new("REPL expects native model, got quantized"));
        }
    };
    eprintln!("[vigil-repl] ready. Type a prompt and press Enter. Ctrl-D to exit.");

    let system_prompt = system.unwrap_or_else(|| String::from(
        "You are Vigil, NexVigilant's sovereign pharmacovigilance AI. Answer concisely and technically."
    ));

    let cfg = vt::inference::SamplingConfig {
        temperature,
        top_p: Some(0.9),
        seed,
        max_new_tokens,
    };

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    // Pretty mode: show a prompt marker on stderr so stdout stays clean.
    if pretty {
        eprint!("\nvigil> ");
    }

    let mut turn: u64 = 0;
    for line_result in stdin.lock().lines() {
        let line = line_result.map_err(|e| NexError::new(format!("stdin read: {e}")))?;
        if line.trim().is_empty() {
            if pretty {
                eprint!("vigil> ");
            }
            continue;
        }
        turn += 1;

        let chatml = vt::tokenizer::format_chatml(&system_prompt, line.trim());
        vt::inference::clear_native_cache(&mut m);

        let t0 = Instant::now();
        match vt::inference::generate_native(&mut m, &tok, &device, &chatml, &cfg) {
            Ok((text, n_tok)) => {
                let dt_ms = t0.elapsed().as_millis();
                if pretty {
                    let mut h = stdout.lock();
                    writeln!(h, "{}", text).map_err(|e| NexError::new(format!("stdout: {e}")))?;
                    eprintln!(
                        "[{} tok in {:.1}s — {:.1} tok/s]",
                        n_tok,
                        dt_ms as f64 / 1000.0,
                        n_tok as f64 / (dt_ms as f64 / 1000.0).max(0.001)
                    );
                    eprint!("vigil> ");
                } else {
                    let out = serde_json::json!({
                        "turn": turn,
                        "response": text,
                        "tokens": n_tok,
                        "duration_ms": dt_ms,
                        "tok_per_sec":
                            n_tok as f64 / (dt_ms as f64 / 1000.0).max(0.001),
                    });
                    let mut h = stdout.lock();
                    writeln!(
                        h,
                        "{}",
                        serde_json::to_string(&out)
                            .map_err(|e| NexError::new(format!("json: {e}")))?
                    )
                    .map_err(|e| NexError::new(format!("stdout: {e}")))?;
                }
            }
            Err(e) => {
                let err = serde_json::json!({
                    "turn": turn,
                    "error": e.to_string(),
                });
                let mut h = stdout.lock();
                writeln!(
                    h,
                    "{}",
                    serde_json::to_string(&err).map_err(|e| NexError::new(format!("json: {e}")))?
                )
                .map_err(|e| NexError::new(format!("stdout: {e}")))?;
                if pretty {
                    eprint!("vigil> ");
                }
            }
        }
    }
    eprintln!("\n[vigil-repl] goodbye.");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_infer_native(
    tokenizer_path: Option<&std::path::Path>,
    model_dir: &std::path::Path,
    dtype_str: &str,
    prompt: Option<String>,
    system: Option<String>,
    temperature: f64,
    max_new_tokens: usize,
    seed: u64,
) -> Result<()> {
    let user_prompt = match prompt {
        Some(p) => p,
        None => read_stdin()?,
    };
    let system_prompt = system.unwrap_or_default();
    let chatml = vt::tokenizer::format_chatml(&system_prompt, &user_prompt);

    let dtype = parse_dtype(dtype_str)?;
    let tok = vt::tokenizer::load(tokenizer_path)?;
    let device = vt::model::pick_device()?;
    let loaded = vt::model::load_native(model_dir, dtype, &device)?;
    let mut m = match loaded {
        vt::model::VigilModel::Native(m) => m,
        vt::model::VigilModel::Quantized(_) => {
            return Err(NexError::new("load_native returned quantized — impossible"));
        }
    };

    let cfg = vt::inference::SamplingConfig {
        temperature,
        top_p: Some(0.9),
        seed,
        max_new_tokens,
    };
    let t0 = Instant::now();
    let (text, n) = vt::inference::generate_native(&mut m, &tok, &device, &chatml, &cfg)?;
    let dt_ms = t0.elapsed().as_millis();

    let out = serde_json::json!({
        "model_dir": model_dir.display().to_string(),
        "dtype": dtype_str,
        "tokens_generated": n,
        "duration_ms": dt_ms,
        "response": text,
    });
    let stdout = std::io::stdout();
    let mut h = stdout.lock();
    writeln!(
        h,
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| NexError::new(format!("json: {e}")))?
    )
    .map_err(|e| NexError::new(format!("stdout: {e}")))?;
    Ok(())
}

fn cmd_lora_info(
    num_layers: usize,
    hidden_size: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    lora_r: usize,
    lora_alpha: f64,
) -> Result<()> {
    let lora_cfg = vt::lora::LoraConfig {
        r: lora_r,
        alpha: lora_alpha,
        dropout: 0.05,
    };
    let device = vt::model::pick_device()?;
    let bundle = vt::lora::LoraBundle::attach_canonical(
        num_layers,
        hidden_size,
        num_attention_heads,
        num_key_value_heads,
        lora_cfg,
        DType::F32,
        &device,
    )?;
    // Compute base model params for comparison — Qwen2 rough estimate:
    // 4 projections per attention block scale with hidden_size^2 (approx).
    let attn_params = num_layers * 4 * hidden_size * hidden_size;
    let mlp_params = num_layers * 3 * hidden_size * hidden_size * 4; // rough Qwen2 SwiGLU
    let base_total = attn_params + mlp_params + hidden_size * 151936; // + embedding
    let lora_total = bundle.trainable_params();
    let ratio = lora_total as f64 / base_total as f64;

    let out = serde_json::json!({
        "num_layers": num_layers,
        "hidden_size": hidden_size,
        "num_attention_heads": num_attention_heads,
        "num_key_value_heads": num_key_value_heads,
        "lora_r": lora_r,
        "lora_alpha": lora_alpha,
        "scaling_alpha_over_r": lora_cfg.scaling(),
        "adapter_count": bundle.adapters.len(),
        "lora_trainable_params": lora_total,
        "base_params_estimate": base_total,
        "trainable_ratio_pct": format!("{:.3}%", ratio * 100.0),
    });
    let stdout = std::io::stdout();
    let mut h = stdout.lock();
    writeln!(
        h,
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| NexError::new(format!("json: {e}")))?
    )
    .map_err(|e| NexError::new(format!("stdout: {e}")))?;
    Ok(())
}

/// Deterministic Fisher-Yates shuffle using splitmix64 for RNG (no external deps).
fn shuffle_indices<T>(items: &mut Vec<T>, seed: u64) {
    let mut state = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    for i in (1..items.len()).rev() {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let j = (z as usize) % (i + 1);
        items.swap(i, j);
    }
}
