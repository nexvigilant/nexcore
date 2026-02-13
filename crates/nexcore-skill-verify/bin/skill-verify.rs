//! CLI for skill verification.

use clap::Parser;
use nexcore_skill_verify::{Report, ReportFormat, Verifier, VerifyContext};

#[derive(Parser)]
#[command(name = "skill-verify", about = "Verify skill compliance")]
struct Args {
    /// Path to skill directory
    skill_path: std::path::PathBuf,
    /// Output as JSON
    #[arg(long)]
    json: bool,
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    let format = if args.json {
        ReportFormat::Json
    } else {
        ReportFormat::Text
    };

    let mut ctx = VerifyContext::new(&args.skill_path).with_verbose(args.verbose);
    let verifier = Verifier::with_standard_checks();
    let (outcomes, exit_code) = verifier.run_and_exit_code(&mut ctx);

    println!("{}", Report::new(outcomes, format).render());
    std::process::exit(exit_code);
}
