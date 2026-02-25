//! CLI validator for .true formal axiom files.
//!
//! Usage: core-true [path]
//!
//! If no path given, defaults to `core.true` in the workspace root.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use std::process::ExitCode;

fn run() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().collect();

    let path = if args.len() > 1 {
        args[1].clone()
    } else {
        // Default: look for core.true relative to binary or cwd
        let candidates = [
            "core.true",
            "../../core.true",    // from target/debug/
            "../../../core.true", // from target/debug/deps/
        ];
        let mut found = None;
        for candidate in candidates {
            if std::path::Path::new(candidate).exists() {
                found = Some(candidate.to_string());
                break;
            }
        }
        found.ok_or_else(|| "no .true file found — pass path as argument".to_string())?
    };

    let source = std::fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path, e))?;

    let report = core_true::parse_and_validate(&source);

    // Count constructs
    let axioms = report
        .statements
        .iter()
        .filter(|s| s.construct == core_true::Construct::Axiom)
        .count();
    let defs = report
        .statements
        .iter()
        .filter(|s| s.construct == core_true::Construct::Def)
        .count();
    let theorems = report
        .statements
        .iter()
        .filter(|s| s.construct == core_true::Construct::Theorem)
        .count();

    eprintln!("core-true validate {}", path);
    eprintln!();
    eprintln!("  FILE        {}", path);
    eprintln!(
        "  STATEMENTS  {} axioms  {} defs  {} theorems",
        axioms, defs, theorems
    );
    eprintln!();

    // Theorem breakdown
    if !report.theorem_confs.is_empty() {
        eprintln!("  THEOREM                                  PROOF              CONF");
        for (name, conf) in &report.theorem_confs {
            // Find the proof type for display
            let proof_label = report
                .statements
                .iter()
                .find(|s| s.subject == *name)
                .and_then(|s| s.proof_type.as_ref())
                .map_or("???", |pt| match pt {
                    core_true::ProofType::Computational => "computational",
                    core_true::ProofType::Analytical => "analytical",
                    core_true::ProofType::Mapping => "mapping",
                    core_true::ProofType::Adversarial => "adversarial",
                    core_true::ProofType::Empirical => "empirical",
                });
            eprintln!("  {:<42} {:<18} {:.2}", name, proof_label, conf);
        }
        eprintln!();
    }

    // Passes
    let pass_ok = |p: u8| -> &str {
        if report.errors.iter().any(|e| e.pass == p) {
            "FAIL"
        } else {
            "ok"
        }
    };
    let total_passes: u8 = 8;
    let failed_passes = report
        .errors
        .iter()
        .map(|e| e.pass)
        .collect::<std::collections::HashSet<_>>()
        .len() as u8;

    eprintln!(
        "  PASSES  {}/{}",
        total_passes - failed_passes,
        total_passes
    );
    eprintln!("    1. syntax well-formed              {}", pass_ok(1));
    eprintln!("    2. no duplicate subjects            {}", pass_ok(2));
    eprintln!("    3. all from-refs exist              {}", pass_ok(3));
    eprintln!("    4. DAG acyclic                      {}", pass_ok(4));
    eprintln!("    5. all theorems have [proof]        {}", pass_ok(5));
    eprintln!("    6. file ends with halt              {}", pass_ok(6));
    eprintln!("    7. conf monotonicity                {}", pass_ok(7));
    eprintln!("    8. spec ↔ impl cross-validate       {}", pass_ok(8));

    if !report.is_valid() {
        eprintln!();
        for err in &report.errors {
            eprintln!("  [!] {}", err);
        }
    }

    eprintln!();
    eprintln!("  SYSTEM CONF = {:.2}", report.system_conf);
    eprintln!();

    if report.is_valid() {
        eprintln!(
            "[ok] {} valid — {} theorems, conf={:.2}",
            path, theorems, report.system_conf
        );
    } else {
        eprintln!("[!!] {} — {} errors", path, report.errors.len());
    }

    Ok(report.is_valid())
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("[!!] {}", e);
            ExitCode::from(2)
        }
    }
}
