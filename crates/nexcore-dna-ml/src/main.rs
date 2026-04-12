//! DNA-ML: DNA-encoded machine learning for PV signal detection.
//!
//! Usage:
//!   dna-ml demo
//!   dna-ml run <json_file>

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use nexcore_dna_ml::pipeline::{self, DnaMlConfig, DnaMlResult};
use nexcore_ml_pipeline::types::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    match command {
        "demo" => run_demo(),
        "run" => {
            let path = args.get(3).or(args.get(2));
            match path {
                Some(p) => run_from_file(p),
                None => {
                    eprintln!("Usage: dna-ml run <json_file>");
                    std::process::exit(1);
                }
            }
        }
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("Unknown command: {other}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("dna-ml — DNA-encoded ML for PV signal detection");
    println!();
    println!("Commands:");
    println!("  demo              Run with built-in sample data");
    println!("  run <json_file>   Run with FAERS data from JSON file");
    println!("  help              Show this help");
    println!();
    println!(
        "Pipeline: FAERS features -> DNA encoding -> similarity augmentation -> random forest"
    );
}

fn run_demo() {
    println!("=== DNA-ML Pipeline Demo ===\n");

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
    let labels: Vec<String> = vec![
        "signal", "signal", "signal", "signal", "signal", "signal", "noise", "noise", "noise",
        "noise",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    // Baseline (PV features only)
    let config_baseline = DnaMlConfig {
        n_trees: 30,
        max_depth: 6,
        use_dna_features: false,
        ..Default::default()
    };
    println!("Training baseline model (12 PV features)...");
    match pipeline::run(&data, &labels, &config_baseline) {
        Ok(r) => print_result("Baseline (PV only)", &r),
        Err(e) => eprintln!("Baseline error: {e}"),
    }

    // DNA-augmented
    let config_dna = DnaMlConfig {
        n_trees: 30,
        max_depth: 6,
        use_dna_features: true,
        ..Default::default()
    };
    println!("\nTraining DNA-augmented model (17 features)...");
    match pipeline::run(&data, &labels, &config_dna) {
        Ok(r) => print_result("DNA-Augmented", &r),
        Err(e) => eprintln!("DNA-ML error: {e}"),
    }
}

fn print_result(label: &str, result: &DnaMlResult) {
    println!("\n--- {label} ---");
    println!(
        "  Features:  {} PV + {} DNA = {} total",
        result.pv_feature_count, result.dna_feature_count, result.total_features
    );
    println!("  Samples:   {}", result.n_samples);
    println!("  AUC:       {:.4}", result.metrics.auc);
    println!("  Accuracy:  {:.4}", result.metrics.accuracy);
    println!("  Precision: {:.4}", result.metrics.precision);
    println!("  Recall:    {:.4}", result.metrics.recall);
    println!("  F1:        {:.4}", result.metrics.f1);
}

fn run_from_file(path: &str) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {path}: {e}");
            std::process::exit(1);
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
            std::process::exit(1);
        }
    };

    let config = input.config.unwrap_or_default();

    match pipeline::run(&input.data, &input.labels, &config) {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("Failed to serialize result: {e}"),
        },
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            std::process::exit(1);
        }
    }
}
