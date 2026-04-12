//! nexcore-processor CLI — ICSR triage, deadlines, and pipeline demo.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nexcore_processor::processor::Processor;
use nexcore_processor::pv::{self, CaseReport};

#[derive(Parser)]
#[command(
    name = "nexcore-processor",
    version,
    about = "PV case processing pipeline — triage, deadlines, batch"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Triage a case through the full ICSR pipeline
    Triage {
        /// Suspect drug name
        drug: String,
        /// Adverse event description
        event: String,
        /// Patient identifier
        #[arg(long)]
        patient: Option<String>,
        /// Reporter type (e.g. physician, consumer)
        #[arg(long)]
        reporter: Option<String>,
    },
    /// Compute reporting deadline for a drug-event pair
    Deadline {
        /// Suspect drug name
        drug: String,
        /// Adverse event description
        event: String,
        /// Patient identifier (required for valid case)
        #[arg(long, default_value = "PT-001")]
        patient: String,
        /// Reporter type
        #[arg(long, default_value = "physician")]
        reporter: String,
    },
    /// Run the demo pipeline showing all 4 stages
    Demo,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Triage {
            drug,
            event,
            patient,
            reporter,
        } => cmd_triage(&drug, &event, patient.as_deref(), reporter.as_deref()),
        Command::Deadline {
            drug,
            event,
            patient,
            reporter,
        } => cmd_deadline(&drug, &event, &patient, &reporter),
        Command::Demo => cmd_demo(),
    }
}

fn cmd_triage(drug: &str, event: &str, patient: Option<&str>, reporter: Option<&str>) -> ExitCode {
    let mut case = CaseReport::new(drug, event);
    if let Some(p) = patient {
        case = case.with_patient(p);
    }
    if let Some(r) = reporter {
        case = case.with_reporter(r);
    }

    let pipeline = pv::icsr_pipeline();
    match pipeline.process(case) {
        Ok(result) => {
            let expedited = pv::is_expedited(&result);
            let deadline = pv::reporting_deadline_days(&result);

            println!("{{");
            println!("  \"drug\": \"{}\",", result.drug);
            println!("  \"event\": \"{}\",", result.event);
            println!(
                "  \"meddra_pt\": {},",
                result
                    .meddra_pt
                    .as_ref()
                    .map(|s| format!("\"{s}\""))
                    .unwrap_or_else(|| "null".into())
            );
            println!(
                "  \"seriousness\": \"{:?}\",",
                result
                    .seriousness
                    .as_ref()
                    .unwrap_or(&pv::Seriousness::NonSerious)
            );
            println!(
                "  \"causality\": {},",
                result
                    .causality
                    .as_ref()
                    .map(|s| format!("\"{s}\""))
                    .unwrap_or_else(|| "null".into())
            );
            println!(
                "  \"valid_for_submission\": {},",
                result.valid_for_submission
            );
            println!("  \"expedited\": {expedited},");
            println!(
                "  \"deadline_days\": {}",
                deadline
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "null".into())
            );
            println!("}}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            eprintln!(
                "Hint: triage requires --patient and --reporter for ICH E2A minimum criteria"
            );
            ExitCode::from(1)
        }
    }
}

fn cmd_deadline(drug: &str, event: &str, patient: &str, reporter: &str) -> ExitCode {
    let case = CaseReport::new(drug, event)
        .with_patient(patient)
        .with_reporter(reporter);

    let pipeline = pv::icsr_pipeline();
    match pipeline.process(case) {
        Ok(result) => {
            let deadline = pv::reporting_deadline_days(&result);
            let expedited = pv::is_expedited(&result);

            println!("{{");
            println!("  \"drug\": \"{}\",", result.drug);
            println!("  \"event\": \"{}\",", result.event);
            println!("  \"expedited\": {expedited},");
            println!(
                "  \"deadline_days\": {},",
                deadline
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "null".into())
            );
            println!(
                "  \"seriousness\": \"{:?}\"",
                result
                    .seriousness
                    .as_ref()
                    .unwrap_or(&pv::Seriousness::NonSerious)
            );
            println!("}}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_demo() -> ExitCode {
    eprintln!("=== ICSR Pipeline Demo ===\n");

    let cases = vec![
        (
            "Semaglutide",
            "severe pancreatitis requiring hospitalization",
            "PT-001",
            "physician",
        ),
        (
            "Metformin",
            "lactic acidosis with respiratory failure",
            "PT-002",
            "physician",
        ),
        ("Aspirin", "mild headache", "PT-003", "consumer"),
        (
            "Lisinopril",
            "death after cardiac arrest",
            "PT-004",
            "physician",
        ),
    ];

    for (drug, event, patient, reporter) in &cases {
        let case = CaseReport::new(*drug, *event)
            .with_patient(*patient)
            .with_reporter(*reporter);

        let pipeline = pv::icsr_pipeline();
        match pipeline.process(case) {
            Ok(result) => {
                let expedited = pv::is_expedited(&result);
                let deadline = pv::reporting_deadline_days(&result);
                let marker = if expedited { "!" } else { " " };

                eprintln!(
                    "[{marker}] {:<15} + {:<45} → {:?} | deadline: {} days | submit: {}",
                    result.drug,
                    result.event,
                    result
                        .seriousness
                        .as_ref()
                        .unwrap_or(&pv::Seriousness::NonSerious),
                    deadline
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "-".into()),
                    result.valid_for_submission
                );
            }
            Err(e) => eprintln!("  ERROR: {e}"),
        }
    }

    eprintln!("\n[!] = expedited reporting required");
    ExitCode::SUCCESS
}
