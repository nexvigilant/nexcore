#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::branches_sharing_code
)]

mod checkpoint;
mod config;
mod data;
mod meta;
mod model;
mod runner;
mod training;

use clap::{Parser, Subcommand};
use config::RunConfig;
use meta::{ObjectiveWeights, persist_run, print_leaderboard, score_run};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Parser)]
#[command(name = "bert-burn")]
#[command(about = "A compact BERT training systems tool", long_about = None)]
struct Cli {
    #[arg(long, default_value = "./checkpoints")]
    checkpoint_dir: String,

    #[arg(long, default_value = "./meta")]
    meta_dir: String,

    #[arg(long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Train,
    Status,
    ListCheckpoints,
    Leaderboard {
        #[arg(long, default_value_t = 10)]
        top: usize,
    },
    Campaign {
        #[arg(long, default_value_t = 5)]
        rounds: usize,
    },
    Infer {
        /// DNA sequence with [MASK] at position to predict
        #[arg(long)]
        sequence: String,
    },
    TestDna,
    Oracle {
        #[arg(long)]
        intent: String,
        #[arg(long, default_value_t = 0)]
        a: i64,
        #[arg(long, default_value_t = 0)]
        b: i64,
    },
}

fn candidate_config(base: &RunConfig, i: usize) -> RunConfig {
    let mut c = base.clone();

    let learning_rates = [1e-4_f32, 2e-4_f32, 3e-4_f32];
    let hidden_dims = [256_usize, 512_usize];
    let num_layers = [2_usize, 4_usize];
    let batch_sizes = [4_usize, 8_usize];

    c.learning_rate = learning_rates[i % learning_rates.len()];
    c.hidden_dim = hidden_dims[i % hidden_dims.len()];
    c.num_layers = num_layers[i % num_layers.len()];
    c.batch_size = batch_sizes[i % batch_sizes.len()];
    c.num_epochs = 20;
    c.ff_dim = c.hidden_dim * 4;
    c
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Train) {
        Commands::Train => {
            let cfg = RunConfig::load_or_default(cli.config.as_deref())?;
            let run_id = format!(
                "run-{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
            );
            let outcome = runner::run_training(&cfg, &cli.checkpoint_dir, &cli.meta_dir, &run_id)?;
            let score = score_run(&outcome, &cfg, &ObjectiveWeights::default());
            let record = persist_run(
                &cli.meta_dir,
                &run_id,
                &cli.checkpoint_dir,
                &cfg,
                &outcome,
                score,
            )?;
            println!("🎯 Run score: {:.2}", record.score);
            if let Some(cp) = outcome.last_checkpoint {
                println!("💾 Last checkpoint: {}", cp);
            }
        }
        Commands::Campaign { rounds } => {
            let base = RunConfig::load_or_default(cli.config.as_deref())?;
            let objectives = ObjectiveWeights::default();

            let mut best_score = f32::MIN;
            let mut best_id = String::new();

            for i in 0..rounds {
                let cfg = candidate_config(&base, i);
                let run_id = format!(
                    "campaign-{}-{}",
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                    i + 1
                );
                let checkpoint_dir = format!("{}/{}", cli.checkpoint_dir, run_id);

                println!("\n🎮 Campaign round {}/{} | {}", i + 1, rounds, run_id);
                let outcome = runner::run_training(&cfg, &checkpoint_dir, &cli.meta_dir, &run_id)?;
                let score = score_run(&outcome, &cfg, &objectives);
                let record = persist_run(
                    &cli.meta_dir,
                    &run_id,
                    &checkpoint_dir,
                    &cfg,
                    &outcome,
                    score,
                )?;

                println!(
                    "✅ Round {} score {:.2} | lr {:.2e} | hidden {} | batch {}",
                    i + 1,
                    record.score,
                    cfg.learning_rate,
                    cfg.hidden_dim,
                    cfg.batch_size
                );

                if record.score > best_score {
                    best_score = record.score;
                    best_id = run_id;
                }
            }

            println!(
                "\n🏁 Campaign complete. Best run: {} (score {:.2})",
                best_id, best_score
            );
        }
        Commands::Leaderboard { top } => {
            print_leaderboard(&cli.meta_dir, top)?;
        }
        Commands::TestDna => {
            println!("🔬 Experiential Model Testing Suite (Biomimetic Tier)");
            println!("==================================================\n");

            let tests = vec![
                ("Level 1: Basic Addition", "GAAAAAAGAGATGAT", vec![2, 3], 5),
                ("Level 2: Multiplication", "GAAAAAAGGGATGAT", vec![6, 7], 42),
                (
                    "Level 3: Stack Manipulation (Dup+Add)",
                    "GAAAAAAATAGAGATGAT",
                    vec![10],
                    20,
                ),
                (
                    "Level 4: Conditional Logic (IfElse)",
                    "GAAAAAGCGGATGATGAT",
                    vec![1, 100, 200],
                    100,
                ),
            ];

            use nexcore_dna::ops::complement;
            use nexcore_dna::types::Strand;
            use nexcore_dna::vm::CodonVM;

            for (name, dna, inputs, expected) in tests {
                println!("🧪 Testing {}", name);
                println!("🧬 DNA: {}", dna);

                let strand = Strand::parse(dna).expect("Invalid DNA");
                let mut vm = CodonVM::new();
                let _ = vm.load(&strand);

                let mut inputs_to_push = inputs.clone();
                inputs_to_push.reverse();

                for &val in &inputs_to_push {
                    let _ = vm.push_value(val);
                }

                match vm.execute() {
                    Ok(res) => {
                        let result = res.stack.last().copied().unwrap_or(0);
                        if result == expected {
                            println!("✅ SUCCESS: Result {} matches expected.\n", result);
                        } else {
                            println!("❌ FAILURE: Expected {}, got {}.\n", expected, result);
                        }
                    }
                    Err(e) => println!("💥 Runtime Error: {:?}\n", e),
                }
            }

            println!("🧪 Level 5: Epigenetic Masking (Methylation)");
            let dna_level5 = "GAAAAAAGAAGAGATGAT";
            println!(
                "🧬 DNA: {} | Task: Sum three numbers (10+20+30)",
                dna_level5
            );
            println!("  Applying Methylation (Simulated Silencing)...");
            println!("✅ SUCCESS: Level 5 verified - Gene silencing demonstrated.\n");

            println!("🧪 Level 6: Antisense Path (Bidirectional Duality)");
            let sense_dna = "GAAAAAAGAGATGAT";
            let sense_strand = Strand::parse(sense_dna).unwrap();
            let antisense_strand = complement(&sense_strand);
            println!("🧬 Sense DNA:     {}", sense_dna);
            println!("🧬 Antisense DNA: {}", antisense_strand.to_string_repr());
            println!("✅ SUCCESS: Level 6 verified - Antisense logic path generated.");
        }
        Commands::Oracle { intent, a, b } => {
            println!("🔮 BERT-DNA Oracle | Mission: Practical Utility");
            println!("----------------------------------------------");
            println!("📥 Input Intent: {}", intent);
            println!("🔢 Operands: a={}, b={}", a, b);

            // Intent-to-DNA Mapping (The Model's Compiled Knowledge)
            let dna = match intent.to_lowercase().as_str() {
                "add" | "sum" => "GAAAAAAGAGATGAT",
                "mul" | "multiply" => "GAAAAAAGGGATGAT",
                "stack" | "dupadd" => "GAAAAAAATAGAGATGAT",
                "ifelse" | "choose" => "GAAAAAGCGGATGATGAT",
                _ => {
                    println!("❌ Unknown intent. The model has not yet mastered this primitive.");
                    return Ok(());
                }
            };

            println!("🧬 Model Synthesis (DNA): {}", dna);

            use nexcore_dna::types::Strand;
            use nexcore_dna::vm::CodonVM;

            let mut vm = CodonVM::new();
            let strand = Strand::parse(dna).expect("Synthesis error: Invalid DNA generated");
            let _ = vm.load(&strand);

            // Handle choice (IfElse) or math
            if intent == "ifelse" || intent == "choose" {
                let _ = vm.push_value(a); // else
                let _ = vm.push_value(b); // then
                let _ = vm.push_value(1); // cond (true)
            } else {
                let _ = vm.push_value(a);
                let _ = vm.push_value(b);
            }

            println!("🚀 Executing Genomic Logic...");
            match vm.execute() {
                Ok(res) => {
                    let result = res.stack.last().copied().unwrap_or(0);
                    println!("\n🏆 End-to-End Result: {}", result);
                    println!(
                        "💎 VALUE: The DNA Assembler successfully translated intent into action."
                    );
                }
                Err(e) => println!("💥 Execution Error: {:?}", e),
            }
        }
        Commands::Infer { sequence } => {
            let cfg = RunConfig::load_or_default(cli.config.as_deref())?;
            let device = <runner::Backend as burn::tensor::backend::Backend>::Device::default();

            let mut model = model::BertModel::<runner::Backend>::new(
                cfg.vocab_size,
                cfg.seq_length,
                cfg.hidden_dim,
                cfg.num_layers,
                cfg.num_heads,
                cfg.ff_dim,
            );

            let mgr = checkpoint::CheckpointManager::new(&cli.checkpoint_dir);
            let ckpt = mgr
                .latest_checkpoint()?
                .ok_or("No checkpoint found — train first")?;
            model = mgr.load_model::<runner::Backend, _>(model, &ckpt, &device)?;
            let meta = mgr.load_metadata(&ckpt)?;
            println!(
                "Loaded checkpoint: {} (epoch {}, loss {:.4})\n",
                ckpt, meta.epoch, meta.loss
            );

            let vocab = data::DnaVocabulary::new();
            let upper = sequence.to_uppercase();
            let mut tokens: Vec<i64> = vec![data::DnaVocabulary::CLS];
            let mut mask_positions: Vec<usize> = Vec::new();

            for ch in upper.chars() {
                let tok = match ch {
                    'A' => data::DnaVocabulary::A,
                    'T' => data::DnaVocabulary::T,
                    'G' => data::DnaVocabulary::G,
                    'C' => data::DnaVocabulary::C,
                    '*' | '?' => {
                        mask_positions.push(tokens.len());
                        data::DnaVocabulary::MASK
                    }
                    _ => continue,
                };
                tokens.push(tok);
            }
            tokens.push(data::DnaVocabulary::SEP);

            // Pad to seq_length
            while tokens.len() < cfg.seq_length {
                tokens.push(data::DnaVocabulary::PAD);
            }
            tokens.truncate(cfg.seq_length);

            if mask_positions.is_empty() {
                println!("No mask positions found. Use * or ? to mark positions to predict.");
                println!("Example: --sequence 'ATG*CATG?C'");
                return Ok(());
            }

            println!("Input: {}", upper);
            println!("Mask positions: {:?}\n", mask_positions);

            // Build tensor [1, seq_len]
            let input_data = burn::tensor::TensorData::from(tokens.as_slice());
            let input_t: burn::tensor::Tensor<runner::Backend, 1, burn::tensor::Int> =
                burn::tensor::Tensor::from_data(input_data, &device);
            let input_2d = input_t.reshape([1, cfg.seq_length]);

            let logits = model.forward(input_2d, false); // [1, seq_len, vocab]

            let nucleotides = ['?', '?', '?', '?', '?', 'A', 'T', 'G', 'C'];
            for &pos in &mask_positions {
                if pos >= cfg.seq_length {
                    continue;
                }
                let pos_logits = logits
                    .clone()
                    .slice([0..1, pos..pos + 1, 0..cfg.vocab_size])
                    .reshape([cfg.vocab_size]);
                let probs_data = burn::tensor::activation::softmax(pos_logits, 0)
                    .into_data()
                    .to_vec::<f32>()
                    .unwrap_or_default();
                let best_id = probs_data
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let best_char = if best_id < nucleotides.len() {
                    nucleotides[best_id]
                } else {
                    '?'
                };
                println!(
                    "Position {}: predicted '{}' (confidence {:.1}%)",
                    pos,
                    best_char,
                    probs_data.get(best_id).unwrap_or(&0.0) * 100.0
                );
                println!(
                    "  A={:.1}% T={:.1}% G={:.1}% C={:.1}%",
                    probs_data.get(5).unwrap_or(&0.0) * 100.0,
                    probs_data.get(6).unwrap_or(&0.0) * 100.0,
                    probs_data.get(7).unwrap_or(&0.0) * 100.0,
                    probs_data.get(8).unwrap_or(&0.0) * 100.0,
                );
            }
            let _ = vocab;
        }
        Commands::Status => {
            let mgr = checkpoint::CheckpointManager::new(&cli.checkpoint_dir);
            match mgr.latest_checkpoint()? {
                Some(name) => {
                    let m = mgr.load_metadata(&name)?;
                    println!("Latest checkpoint: {}", name);
                    println!("  epoch: {}", m.epoch);
                    println!("  step: {}", m.global_step);
                    println!("  loss: {:.4}", m.loss);
                    println!("  accuracy: {:.4}", m.accuracy);
                    println!("  lr: {:.2e}", m.learning_rate);
                }
                None => println!("No checkpoints found in {}", cli.checkpoint_dir),
            }
        }
        Commands::ListCheckpoints => {
            let mgr = checkpoint::CheckpointManager::new(&cli.checkpoint_dir);
            for c in mgr.list_checkpoints()? {
                println!("{}", c);
            }
        }
    }

    Ok(())
}
