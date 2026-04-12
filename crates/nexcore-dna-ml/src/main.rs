//! DNA-ML: DNA-encoded machine learning for PV signal detection.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nexcore_dna_ml::pipeline::{self, DnaMlConfig, DnaMlResult};
use nexcore_ml_pipeline::types::*;

#[derive(Parser)]
#[command(
    name = "dna-ml",
    version,
    about = "DNA-encoded ML for PV signal detection"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run with built-in sample data (semaglutide, metformin, aspirin)
    Demo {
        /// Number of trees in the random forest
        #[arg(long, default_value_t = 30)]
        trees: usize,
        /// Max tree depth
        #[arg(long, default_value_t = 6)]
        depth: usize,
    },
    /// Run pipeline from a JSON data file
    Run {
        /// Path to JSON file with {data, labels, config?}
        file: String,
    },
    /// Encode a feature vector to a DNA strand
    Encode {
        /// Feature values (space-separated floats)
        features: Vec<f64>,
    },
    /// Show pipeline input schema
    Schema,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Demo { trees, depth } => cmd_demo(trees, depth),
        Command::Run { file } => cmd_run(&file),
        Command::Encode { features } => cmd_encode(&features),
        Command::Schema => cmd_schema(),
    }
}

fn cmd_demo(trees: usize, depth: usize) -> ExitCode {
    eprintln!("=== DNA-ML Pipeline Demo ===\n");

    let signal = RawPairData {
        contingency: ContingencyTable {
            drug: "Semaglutide".into(),
            event: "Pancreatitis".into(),
            a: 2068,
            b: 76216,
            c: 75999,
            d: 19852706,
        },
        reporters: ReporterBreakdown {
            hcp: 1200,
            consumer: 868,
            other: 0,
        },
        outcomes: OutcomeBreakdown {
            total: 2068,
            serious: 1500,
            death: 50,
            hospitalization: 1200,
        },
        temporal: TemporalData {
            median_tto_days: Some(30.0),
            velocity: 3.0,
        },
    };

    let moderate = RawPairData {
        contingency: ContingencyTable {
            drug: "Metformin".into(),
            event: "Lactic Acidosis".into(),
            a: 800,
            b: 120000,
            c: 40000,
            d: 19800000,
        },
        reporters: ReporterBreakdown {
            hcp: 600,
            consumer: 200,
            other: 0,
        },
        outcomes: OutcomeBreakdown {
            total: 800,
            serious: 700,
            death: 100,
            hospitalization: 500,
        },
        temporal: TemporalData {
            median_tto_days: Some(90.0),
            velocity: 5.0,
        },
    };

    let noise = RawPairData {
        contingency: ContingencyTable {
            drug: "Aspirin".into(),
            event: "Headache".into(),
            a: 50,
            b: 500000,
            c: 200000,
            d: 19000000,
        },
        reporters: ReporterBreakdown {
            hcp: 10,
            consumer: 40,
            other: 0,
        },
        outcomes: OutcomeBreakdown {
            total: 50,
            serious: 2,
            death: 0,
            hospitalization: 1,
        },
        temporal: TemporalData {
            median_tto_days: None,
            velocity: 1.0,
        },
    };

    let data = vec![
        signal.clone(),
        signal.clone(),
        signal,
        moderate.clone(),
        moderate.clone(),
        moderate,
        noise.clone(),
        noise.clone(),
        noise.clone(),
        noise,
    ];
    let labels: Vec<String> = [
        "signal", "signal", "signal", "signal", "signal", "signal", "noise", "noise", "noise",
        "noise",
    ]
    .iter()
    .map(|s| String::from(*s))
    .collect();

    let config_baseline = DnaMlConfig {
        n_trees: trees,
        max_depth: depth,
        use_dna_features: false,
        ..Default::default()
    };
    eprintln!("Training baseline model (12 PV features)...");
    match pipeline::run(&data, &labels, &config_baseline) {
        Ok(r) => print_result("Baseline (PV only)", &r),
        Err(e) => eprintln!("Baseline error: {e}"),
    }

    let config_dna = DnaMlConfig {
        n_trees: trees,
        max_depth: depth,
        use_dna_features: true,
        ..Default::default()
    };
    eprintln!("\nTraining DNA-augmented model (17 features)...");
    match pipeline::run(&data, &labels, &config_dna) {
        Ok(r) => print_result("DNA-Augmented", &r),
        Err(e) => eprintln!("DNA-ML error: {e}"),
    }

    ExitCode::SUCCESS
}

fn print_result(label: &str, result: &DnaMlResult) {
    eprintln!("\n--- {label} ---");
    eprintln!(
        "  Features:  {} PV + {} DNA = {} total",
        result.pv_feature_count, result.dna_feature_count, result.total_features
    );
    eprintln!("  Samples:   {}", result.n_samples);
    eprintln!("  AUC:       {:.4}", result.metrics.auc);
    eprintln!("  Accuracy:  {:.4}", result.metrics.accuracy);
    eprintln!("  Precision: {:.4}", result.metrics.precision);
    eprintln!("  Recall:    {:.4}", result.metrics.recall);
    eprintln!("  F1:        {:.4}", result.metrics.f1);
}

fn cmd_run(path: &str) -> ExitCode {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {path}: {e}");
            return ExitCode::from(1);
        }
    };

    #[derive(serde::Deserialize)]
    struct InputData {
        data: Vec<RawPairData>,
        labels: Vec<String>,
        #[serde(default)]
        config: Option<DnaMlConfig>,
    }

    let input: InputData = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to parse JSON: {e}");
            return ExitCode::from(1);
        }
    };

    let config = input.config.unwrap_or_default();

    match pipeline::run(&input.data, &input.labels, &config) {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Failed to serialize: {e}");
                ExitCode::from(1)
            }
        },
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_encode(features: &[f64]) -> ExitCode {
    if features.is_empty() {
        eprintln!("Provide at least one feature value");
        return ExitCode::from(1);
    }

    let mins: Vec<f64> = features.iter().map(|_| 0.0).collect();
    let maxs: Vec<f64> = features.iter().map(|_| 100.0).collect();

    let strand = nexcore_dna_ml::encode::encode_features(features, &mins, &maxs);
    let nucleotides: String = strand
        .bases
        .iter()
        .map(|n: &nexcore_dna::types::Nucleotide| n.as_char())
        .collect();

    println!(
        "{{\"strand\":\"{nucleotides}\",\"length\":{}}}",
        strand.len()
    );
    ExitCode::SUCCESS
}

fn cmd_schema() -> ExitCode {
    let schema = r#"{
  "data": [
    {
      "contingency": {"drug": "string", "event": "string", "a": 0, "b": 0, "c": 0, "d": 0},
      "reporters": {"hcp": 0, "consumer": 0, "other": 0},
      "outcomes": {"total": 0, "serious": 0, "death": 0, "hospitalization": 0},
      "temporal": {"median_tto_days": null, "velocity": 0.0}
    }
  ],
  "labels": ["signal", "noise"],
  "config": {
    "n_trees": 50,
    "max_depth": 8,
    "max_features": null,
    "min_samples_split": 5,
    "use_dna_features": true
  }
}"#;
    println!("{schema}");
    ExitCode::SUCCESS
}
