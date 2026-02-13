//! CTVP Validator CLI
//!
//! Command-line tool for running Clinical Trial Validation Paradigm analysis.
//!
//! ## Usage
//!
//! ```bash
//! # Full validation
//! ctvp-cli validate ./my-project
//!
//! # Five Problems analysis
//! ctvp-cli five-problems ./my-project
//!
//! # Reality Gradient score
//! ctvp-cli score ./my-project
//!
//! # Specific phase validation
//! ctvp-cli phase 1 ./my-project
//! ```

use nexcore_ctvp::prelude::*;
use std::path::PathBuf;

/// CLI configuration
struct CliConfig {
    /// Path to deliverable
    path: PathBuf,

    /// Output format
    format: OutputFormat,

    /// Verbose output
    verbose: bool,
}

/// Output format options
#[derive(Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
    Markdown,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let result = match command.as_str() {
        "validate" | "v" => cmd_validate(&args[2..]),
        "five-problems" | "5p" => cmd_five_problems(&args[2..]),
        "score" | "s" => cmd_score(&args[2..]),
        "phase" | "p" => cmd_phase(&args[2..]),
        "evidence" | "e" => cmd_evidence(&args[2..]),
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        "version" | "-V" | "--version" => {
            println!("ctvp-cli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_usage() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════════╗
║                    CTVP Validator CLI                              ║
║         Clinical Trial Validation Paradigm for Software           ║
╚═══════════════════════════════════════════════════════════════════╝

USAGE:
    ctvp-cli <COMMAND> [OPTIONS] <PATH>

COMMANDS:
    validate, v       Run full 5-phase validation
    five-problems, 5p Execute Five Problems Protocol
    score, s          Calculate Reality Gradient score
    phase, p          Validate specific phase (0-4)
    evidence, e       Extract evidence inventory
    help              Show this help message
    version           Show version information

OPTIONS:
    --format, -f      Output format: text (default), json, markdown
    --verbose, -v     Enable verbose output
    --output, -o      Write output to file

EXAMPLES:
    # Full validation with markdown report
    ctvp-cli validate ./my-project -f markdown -o report.md

    # Quick Five Problems analysis
    ctvp-cli five-problems ./src

    # Check Phase 1 (Safety) specifically
    ctvp-cli phase 1 ./my-project

    # Extract evidence inventory as JSON
    ctvp-cli evidence ./my-project -f json

PHASE REFERENCE:
    0  Preclinical   - Unit tests, mocks, property tests
    1  Safety        - Chaos engineering, fault injection
    2  Efficacy      - Real data, SLO measurement
    3  Confirmation  - Shadow/canary deployment
    4  Surveillance  - Drift detection, continuous validation

REALITY GRADIENT INTERPRETATION:
    < 0.20   Testing Theater      (Phase 0 only)
    0.20-0.50 Safety Validated    (through Phase 1)
    0.50-0.80 Efficacy Demonstrated (through Phase 2)
    0.80-0.95 Scale Confirmed     (through Phase 3)
    > 0.95   Production Ready    (all phases)

"#
    );
}

fn parse_args(args: &[String]) -> Result<CliConfig, CtvpError> {
    let mut path = None;
    let mut format = OutputFormat::Text;
    let mut verbose = false;
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-f" | "--format" => {
                i += 1;
                format = parse_format(args, i)?;
            }
            "-v" | "--verbose" => verbose = true,
            "-o" | "--output" => {
                i += 1;
            }
            _ if !arg.starts_with('-') => path = Some(PathBuf::from(arg)),
            _ => {}
        }
        i += 1;
    }

    let path = path.ok_or_else(|| CtvpError::Config("Path required".into()))?;
    Ok(CliConfig {
        path,
        format,
        verbose,
    })
}

fn parse_format(args: &[String], i: usize) -> Result<OutputFormat, CtvpError> {
    if i >= args.len() {
        return Err(CtvpError::Config("Missing format value".into()));
    }
    match args[i].as_str() {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        "markdown" | "md" => Ok(OutputFormat::Markdown),
        other => Err(CtvpError::Config(format!("Unknown format: {}", other))),
    }
}

fn cmd_validate(args: &[String]) -> Result<(), CtvpError> {
    let config = parse_args(args)?;

    println!("🔬 CTVP Validation: {}\n", config.path.display());

    let extractor = EvidenceExtractor::new();
    let inventory = extractor.extract(&config.path)?;

    if config.verbose {
        println!("{}\n", inventory.generate_summary());
    }

    let results = build_validation_results(&inventory);
    let gradient = RealityGradient::calculate(&results);

    let analyzer = FiveProblemsAnalyzer::new();
    let problems = analyzer.analyze(&config.path)?;

    render_validate_report(&config, &gradient, &problems, &inventory)
}

fn build_validation_results(inventory: &EvidenceInventory) -> Vec<ValidationResult> {
    ValidationPhase::get_all()
        .into_iter()
        .map(|p| map_phase_to_result(inventory, p))
        .collect()
}

fn map_phase_to_result(inv: &EvidenceInventory, p: ValidationPhase) -> ValidationResult {
    let q = inv.get_quality(p);
    let outcome = if q >= EvidenceQuality::Weak {
        ValidationOutcome::Validated
    } else {
        ValidationOutcome::Inconclusive {
            reason: "Insufficient evidence".into(),
        }
    };
    ValidationResult::new("deliverable", p, outcome, q)
}

fn render_validate_report(
    config: &CliConfig,
    gradient: &RealityGradient,
    problems: &FiveProblemsAnalysis,
    inventory: &EvidenceInventory,
) -> Result<(), CtvpError> {
    match config.format {
        OutputFormat::Text => print_text_report(gradient, problems),
        OutputFormat::Json => print_validate_json(gradient, problems, inventory),
        OutputFormat::Markdown => {
            print_markdown_report(&config.path, gradient, problems, inventory)
        }
    }
    Ok(())
}

fn print_validate_json(g: &RealityGradient, p: &FiveProblemsAnalysis, i: &EvidenceInventory) {
    let output = serde_json::json!({
        "reality_gradient": {
            "score": g.value,
            "interpretation": format!("{:?}", g.interpretation),
            "limiting_factor": g.limiting_factor.map(|p| p.to_string()),
        },
        "five_problems": p,
        "evidence_inventory": i,
    });
    // INVARIANT: serialization of known types should not fail
    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );
}

fn cmd_five_problems(args: &[String]) -> Result<(), CtvpError> {
    let config = parse_args(args)?;

    println!("🔍 Five Problems Analysis: {}\n", config.path.display());

    let analyzer = FiveProblemsAnalyzer::new();
    let analysis = analyzer.analyze(&config.path)?;

    match config.format {
        OutputFormat::Text => render_five_problems_text(&analysis),
        OutputFormat::Json => {
            // INVARIANT: serialization of known types should not fail
            println!(
                "{}",
                serde_json::to_string_pretty(&analysis).unwrap_or_default()
            );
        }
        OutputFormat::Markdown => render_five_problems_markdown(&config.path, &analysis),
    }

    Ok(())
}

fn render_five_problems_text(analysis: &FiveProblemsAnalysis) {
    println!("Overall Severity: {:?}\n", analysis.overall_severity);
    println!("Problems Discovered:");
    println!("{}", "─".repeat(60));

    for problem in &analysis.problems {
        println!(
            "\n[{}] {} - {} ({})",
            problem.number, problem.category, problem.description, problem.severity
        );
        println!("  Evidence: {}", problem.evidence);
        println!("  Remediation: {}", problem.remediation);
    }

    println!("\n{}", "─".repeat(60));
    println!("\nRemediation Roadmap:");
    for step in &analysis.remediation_roadmap {
        println!("  {}. {}", step.priority, step.action);
    }
}

fn render_five_problems_markdown(path: &std::path::Path, analysis: &FiveProblemsAnalysis) {
    println!("# Five Problems Analysis\n");
    println!("**Deliverable:** `{}`\n", path.display());
    println!("**Overall Severity:** {}\n", analysis.overall_severity);
    println!("## Problems\n");

    for problem in &analysis.problems {
        println!("### {} - {}\n", problem.number, problem.category);
        println!("**Severity:** {}\n", problem.severity);
        println!("{}\n", problem.description);
        println!("**Evidence:** {}\n", problem.evidence);
        println!("**Remediation:** {}\n", problem.remediation);
        println!(
            "**Test Required:**\n```rust\n{}\n```\n",
            problem.test_required
        );
    }
}

fn cmd_score(args: &[String]) -> Result<(), CtvpError> {
    let config = parse_args(args)?;

    println!("📊 Reality Gradient: {}\n", config.path.display());

    let extractor = EvidenceExtractor::new();
    let inventory = extractor.extract(&config.path)?;

    let results: Vec<_> = ValidationPhase::get_all()
        .into_iter()
        .map(|phase| {
            ValidationResult::new(
                "deliverable",
                phase,
                ValidationOutcome::Validated,
                inventory.get_quality(phase),
            )
        })
        .collect();

    let gradient = RealityGradient::calculate(&results);

    match config.format {
        OutputFormat::Text => println!("{}", gradient.generate_summary()),
        OutputFormat::Json => {
            // INVARIANT: serialization of known types should not fail
            println!(
                "{}",
                serde_json::to_string_pretty(&gradient).unwrap_or_default()
            );
        }
        OutputFormat::Markdown => render_score_markdown(&gradient),
    }

    Ok(())
}

fn render_score_markdown(gradient: &RealityGradient) {
    println!("# Reality Gradient Report\n");
    println!("**Score:** {:.2}\n", gradient.value);
    println!("**Interpretation:** {}\n", gradient.interpretation);
    println!("## Phase Breakdown\n");
    println!("| Phase | Evidence | Contribution |");
    println!("|-------|----------|--------------|");
    for pc in &gradient.phase_contributions {
        println!(
            "| {} | {:?} | {:.0}% |",
            pc.phase,
            pc.evidence_quality,
            (pc.contribution / pc.max_contribution) * 100.0
        );
    }
}

fn cmd_phase(args: &[String]) -> Result<(), CtvpError> {
    if args.is_empty() {
        return Err(CtvpError::Config("Phase number required (0-4)".into()));
    }

    let phase_num: u8 = args[0]
        .parse()
        .map_err(|_| CtvpError::Config("Invalid phase number".into()))?;

    let phase = match phase_num {
        0 => ValidationPhase::Preclinical,
        1 => ValidationPhase::Phase1Safety,
        2 => ValidationPhase::Phase2Efficacy,
        3 => ValidationPhase::Phase3Confirmation,
        4 => ValidationPhase::Phase4Surveillance,
        _ => return Err(CtvpError::Config("Phase must be 0-4".into())),
    };

    let config = parse_args(&args[1..])?;

    println!("🔬 {} Validation: {}\n", phase, config.path.display());

    let extractor = EvidenceExtractor::new();
    let inventory = extractor.extract(&config.path)?;

    let evidence = inventory.get_evidence(phase);
    let quality = inventory.get_quality(phase);

    println!("Evidence Quality: {:?}", quality);
    println!("Evidence Items: {}\n", evidence.len());

    if config.verbose {
        for e in evidence {
            println!("  - {} ({})", e.description, e.source);
        }
    }

    println!("\nKey Question: {}", phase.get_key_question());
    println!(
        "Pharmaceutical Equivalent: {}",
        phase.get_pharma_equivalent()
    );

    Ok(())
}

fn cmd_evidence(args: &[String]) -> Result<(), CtvpError> {
    let config = parse_args(args)?;

    println!("📋 Evidence Inventory: {}\n", config.path.display());

    let extractor = EvidenceExtractor::new();
    let inventory = extractor.extract(&config.path)?;

    match config.format {
        OutputFormat::Text => println!("{}", inventory.generate_summary()),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&inventory).unwrap_or_default()
        ),
        OutputFormat::Markdown => render_evidence_markdown(&inventory),
    }

    Ok(())
}

fn render_evidence_markdown(inventory: &EvidenceInventory) {
    println!("# Evidence Inventory\n");
    println!("**Path:** `{}`\n", inventory.deliverable_path.display());
    println!("**Files Scanned:** {}\n", inventory.files_scanned);

    for phase in ValidationPhase::get_all().into_iter() {
        render_phase_evidence_markdown(inventory, phase);
    }
}

fn render_phase_evidence_markdown(inventory: &EvidenceInventory, phase: ValidationPhase) {
    let evidence = inventory.get_evidence(phase);
    let quality = inventory.get_quality(phase);

    println!("## {} ({:?})\n", phase, quality);

    if evidence.is_empty() {
        println!("*No evidence found*\n");
    } else {
        for e in evidence {
            println!("- {} (`{}`)", e.description, e.source);
        }
        println!();
    }
}

fn print_text_report(gradient: &RealityGradient, problems: &FiveProblemsAnalysis) {
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║                      CTVP VALIDATION REPORT                       ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝\n");

    // Reality Gradient
    println!("REALITY GRADIENT");
    println!("{}", "─".repeat(60));
    println!("Score: {:.2} ({})", gradient.value, gradient.interpretation);
    println!();

    for pc in &gradient.phase_contributions {
        let bar_len = (pc.contribution / pc.max_contribution * 30.0) as usize;
        let bar = format!("{}{}", "█".repeat(bar_len), "░".repeat(30 - bar_len));
        println!("  {} [{}] {:?}", pc.phase, bar, pc.evidence_quality);
    }

    println!();

    // Five Problems
    println!("FIVE PROBLEMS ANALYSIS");
    println!("{}", "─".repeat(60));
    println!("Overall Severity: {:?}\n", problems.overall_severity);

    for problem in &problems.problems {
        println!(
            "[{}] {} - {}",
            problem.number, problem.category, problem.severity
        );
        println!("    {}", problem.description);
    }

    println!();

    // Recommendations
    println!("RECOMMENDATIONS");
    println!("{}", "─".repeat(60));
    println!("1. {}", gradient.interpretation.get_next_action());

    if let Some(limiting) = gradient.limiting_factor {
        println!("2. Focus on improving {} validation", limiting);
    }

    for (i, step) in problems.remediation_roadmap.iter().take(3).enumerate() {
        println!("{}. {}", i + 3, step.action);
    }
}

fn print_markdown_report(
    path: &std::path::Path,
    gradient: &RealityGradient,
    problems: &FiveProblemsAnalysis,
    inventory: &EvidenceInventory,
) {
    println!("# CTVP Validation Report\n");
    println!("**Deliverable:** `{}`\n", path.display());
    println!(
        "**Reality Score:** {:.2} ({})\n",
        gradient.value, gradient.interpretation
    );

    println!("## Phase Evidence Summary\n");
    println!("| Phase | Evidence Quality | Key Findings |");
    println!("|-------|-----------------|--------------|");

    for phase in ValidationPhase::get_all().into_iter() {
        let quality = inventory.get_quality(phase);
        let count = inventory.get_evidence(phase).len();
        println!("| {} | {:?} | {} evidence items |", phase, quality, count);
    }

    println!("\n## Five Problems Analysis\n");
    for problem in &problems.problems {
        println!(
            "{}. **[{}]** [{}]: {}",
            problem.number, problem.category, problem.severity, problem.description
        );
    }

    println!("\n## Remediation Roadmap\n");
    for step in &problems.remediation_roadmap {
        println!("{}. {}", step.priority, step.action);
    }

    println!("\n---");
    println!("*Generated by CTVP Validator*");
}
