//! nexcore-processor CLI — ICSR triage, labeling checks, and analysis pipeline.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nexcore_processor::processor::Processor;
use nexcore_processor::pv::{self, CaseReport, Seriousness};

const STATION: &str = "https://mcp.nexvigilant.com";

#[derive(Parser)]
#[command(
    name = "nexcore-processor",
    version,
    about = "PV case processing — triage, labeling, analysis"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Triage a case through the ICSR pipeline
    Triage {
        /// Suspect drug name
        drug: String,
        /// Adverse event description
        event: String,
        #[arg(long)]
        patient: Option<String>,
        #[arg(long)]
        reporter: Option<String>,
    },
    /// Check if an event is listed in the drug label (expected vs unexpected)
    LabelCheck {
        /// Drug name
        drug: String,
        /// Adverse event to check
        event: String,
    },
    /// Full analysis: triage + label check + signal scores + verdict
    Analyze {
        /// Drug name
        drug: String,
        /// Adverse event
        event: String,
        #[arg(long, default_value = "PT-001")]
        patient: String,
        #[arg(long, default_value = "physician")]
        reporter: String,
    },
    /// Compute reporting deadline
    Deadline {
        drug: String,
        event: String,
        #[arg(long, default_value = "PT-001")]
        patient: String,
        #[arg(long, default_value = "physician")]
        reporter: String,
    },
    /// Demo pipeline with sample cases
    Demo,
}

// ─── Station Client ──────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
struct StationContent {
    #[serde(default)]
    text: String,
}

#[derive(serde::Deserialize)]
struct StationResult {
    #[serde(default)]
    content: Vec<StationContent>,
}

#[derive(serde::Deserialize)]
struct StationResponse {
    result: Option<StationResult>,
}

async fn call_station(tool: &str, args: &serde_json::Value) -> Option<serde_json::Value> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": { "name": tool, "arguments": args },
        "id": format!("proc-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0))
    });

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{STATION}/mcp"))
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .ok()?;

    let resp: StationResponse = res.json().await.ok()?;
    let result = resp.result?;
    let content = result.content;
    let text = &content.first()?.text;
    serde_json::from_str(text).ok()
}

// ─── Label Check ──���──────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct LabelCheckResult {
    drug: String,
    event: String,
    found_in_label: bool,
    expectedness: String,
    sections_checked: Vec<LabelSection>,
}

#[derive(serde::Serialize)]
struct LabelSection {
    section: String,
    contains_event: bool,
    excerpt: String,
}

async fn check_label(drug: &str, event: &str) -> LabelCheckResult {
    let mut sections = Vec::new();
    let mut found = false;
    let event_lower = event.to_lowercase();

    // Check adverse reactions section
    if let Some(adr) = call_station(
        "dailymed_nlm_nih_gov_get_adverse_reactions",
        &serde_json::json!({"drug": drug}),
    )
    .await
    {
        let text = adr
            .get("adverse_reactions")
            .or_else(|| adr.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let contains = text.to_lowercase().contains(&event_lower);
        if contains {
            found = true;
        }
        let excerpt = extract_excerpt(text, &event_lower);
        sections.push(LabelSection {
            section: "Adverse Reactions".into(),
            contains_event: contains,
            excerpt,
        });
    }

    // Check boxed warning
    if let Some(boxed) = call_station(
        "dailymed_nlm_nih_gov_get_boxed_warning",
        &serde_json::json!({"drug": drug}),
    )
    .await
    {
        let text = boxed
            .get("boxed_warning")
            .or_else(|| boxed.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let contains = text.to_lowercase().contains(&event_lower);
        if contains {
            found = true;
        }
        if !text.is_empty() {
            let excerpt = extract_excerpt(text, &event_lower);
            sections.push(LabelSection {
                section: "Boxed Warning".into(),
                contains_event: contains,
                excerpt,
            });
        }
    }

    // Check contraindications
    if let Some(contra) = call_station(
        "dailymed_nlm_nih_gov_get_contraindications",
        &serde_json::json!({"drug": drug}),
    )
    .await
    {
        let text = contra
            .get("contraindications")
            .or_else(|| contra.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let contains = text.to_lowercase().contains(&event_lower);
        if contains {
            found = true;
        }
        if !text.is_empty() {
            let excerpt = extract_excerpt(text, &event_lower);
            sections.push(LabelSection {
                section: "Contraindications".into(),
                contains_event: contains,
                excerpt,
            });
        }
    }

    LabelCheckResult {
        drug: drug.into(),
        event: event.into(),
        found_in_label: found,
        expectedness: if found {
            "Expected (listed in label)".into()
        } else {
            "Unexpected (not in label)".into()
        },
        sections_checked: sections,
    }
}

fn extract_excerpt(text: &str, query: &str) -> String {
    let lower = text.to_lowercase();
    if let Some(pos) = lower.find(query) {
        let start = pos.saturating_sub(80);
        let end = (pos + query.len() + 80).min(text.len());
        // Find safe char boundaries
        let start = text[..start]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        let end = text[end..]
            .char_indices()
            .next()
            .map(|(i, _)| end + i)
            .unwrap_or(text.len());
        format!("...{}...", &text[start..end])
    } else if text.len() > 200 {
        format!("{}...", &text[..200])
    } else {
        text.to_string()
    }
}

// ─── Signal Scores ───────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct SignalScores {
    prr: Option<f64>,
    ror: Option<f64>,
    ic: Option<f64>,
    ebgm: Option<f64>,
    signal_detected: bool,
}

async fn compute_signals(drug: &str, event: &str) -> Option<SignalScores> {
    let args = serde_json::json!({"drug": drug, "event": event});

    let prr = call_station("calculate_nexvigilant_com_compute_prr", &args).await;
    let ror = call_station("calculate_nexvigilant_com_compute_ror", &args).await;

    let prr_val = prr.as_ref().and_then(|r| {
        r.get("prr")
            .or_else(|| r.get("value"))
            .and_then(|v| v.as_f64())
    });
    let ror_val = ror.as_ref().and_then(|r| {
        r.get("ror")
            .or_else(|| r.get("value"))
            .and_then(|v| v.as_f64())
    });

    if prr_val.is_none() && ror_val.is_none() {
        return None;
    }

    let signal =
        prr_val.map(|v| v > 2.0).unwrap_or(false) || ror_val.map(|v| v > 2.0).unwrap_or(false);

    Some(SignalScores {
        prr: prr_val,
        ror: ror_val,
        ic: None,
        ebgm: None,
        signal_detected: signal,
    })
}

// ─── Commands ─────────────────���──────────────────────────────────────────────

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

            let json = serde_json::json!({
                "drug": result.drug,
                "event": result.event,
                "meddra_pt": result.meddra_pt,
                "seriousness": format!("{:?}", result.seriousness.as_ref().unwrap_or(&Seriousness::NonSerious)),
                "causality": result.causality,
                "valid_for_submission": result.valid_for_submission,
                "expedited": expedited,
                "deadline_days": deadline,
            });
            match serde_json::to_string_pretty(&json) {
                Ok(s) => println!("{s}"),
                Err(e) => eprintln!("Serialize error: {e}"),
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            eprintln!("Hint: use --patient and --reporter for ICH E2A minimum criteria");
            ExitCode::from(1)
        }
    }
}

async fn cmd_label_check(drug: &str, event: &str) -> ExitCode {
    eprintln!("Checking label for {drug} + {event}...");
    let result = check_label(drug, event).await;

    match serde_json::to_string_pretty(&result) {
        Ok(s) => {
            println!("{s}");
            eprintln!(
                "→ {}",
                if result.found_in_label {
                    "EXPECTED — event found in drug label"
                } else {
                    "UNEXPECTED — event NOT in drug label (potential new signal)"
                }
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Serialize error: {e}");
            ExitCode::from(1)
        }
    }
}

async fn cmd_analyze(drug: &str, event: &str, patient: &str, reporter: &str) -> ExitCode {
    eprintln!("=== Full Analysis: {drug} + {event} ===\n");

    // Step 1: Triage
    eprintln!("[1/3] Running ICSR triage...");
    let case = CaseReport::new(drug, event)
        .with_patient(patient)
        .with_reporter(reporter);

    let pipeline = pv::icsr_pipeline();
    let triage = match pipeline.process(case) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Triage failed: {e}");
            return ExitCode::from(1);
        }
    };

    let expedited = pv::is_expedited(&triage);
    let deadline = pv::reporting_deadline_days(&triage);

    // Step 2: Label check
    eprintln!("[2/3] Checking drug label...");
    let label = check_label(drug, event).await;

    // Step 3: Signal scores
    eprintln!("[3/3] Computing signal scores...");
    let signals = compute_signals(drug, event).await;

    // Composite output
    let output = serde_json::json!({
        "drug": drug,
        "event": event,
        "triage": {
            "meddra_pt": triage.meddra_pt,
            "seriousness": format!("{:?}", triage.seriousness.as_ref().unwrap_or(&Seriousness::NonSerious)),
            "causality": triage.causality,
            "expedited": expedited,
            "deadline_days": deadline,
            "valid": triage.valid_for_submission,
        },
        "labeling": {
            "found_in_label": label.found_in_label,
            "expectedness": label.expectedness,
            "sections_checked": label.sections_checked.len(),
        },
        "signals": signals.as_ref().map(|s| serde_json::json!({
            "prr": s.prr,
            "ror": s.ror,
            "signal_detected": s.signal_detected,
        })),
        "verdict": {
            "action": if expedited && !label.found_in_label {
                "URGENT: Serious unexpected event — expedited reporting required"
            } else if expedited {
                "Serious expected event — expedited reporting required"
            } else if !label.found_in_label {
                "Non-serious unexpected event — routine monitoring"
            } else {
                "Non-serious expected event — no action required"
            },
        }
    });

    match serde_json::to_string_pretty(&output) {
        Ok(s) => {
            println!("{s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Serialize error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_demo() -> ExitCode {
    eprintln!("=== ICSR Pipeline Demo ===\n");

    let cases = vec![
        ("Semaglutide", "severe pancreatitis", "PT-001", "physician"),
        ("Metformin", "lactic acidosis", "PT-002", "physician"),
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
                    "[{marker}] {:<15} + {:<35} → {:?} | deadline: {} days",
                    result.drug,
                    result.event,
                    result
                        .seriousness
                        .as_ref()
                        .unwrap_or(&Seriousness::NonSerious),
                    deadline
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "-".into()),
                );
            }
            Err(e) => eprintln!("  ERROR: {e}"),
        }
    }

    eprintln!("\n[!] = expedited reporting required");
    eprintln!("Use 'analyze <drug> <event>' for full analysis with label check + signals");
    ExitCode::SUCCESS
}

// ─── Main ────────────────────────────────────��───────────────────────────────

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Triage {
            drug,
            event,
            patient,
            reporter,
        } => cmd_triage(&drug, &event, patient.as_deref(), reporter.as_deref()),
        Command::LabelCheck { drug, event } => cmd_label_check(&drug, &event).await,
        Command::Analyze {
            drug,
            event,
            patient,
            reporter,
        } => cmd_analyze(&drug, &event, &patient, &reporter).await,
        Command::Deadline {
            drug,
            event,
            patient,
            reporter,
        } => {
            // Reuse triage for deadline
            cmd_triage(&drug, &event, Some(&patient), Some(&reporter))
        }
        Command::Demo => cmd_demo(),
    }
}
