// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima-to-MCP Compiler CLI
//!
//! Compile Prima source files to MCP tool definitions.

use prima_mcp::compile;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        // Demo mode with inline source
        demo();
        return;
    }

    // Compile provided file
    let path = PathBuf::from(&args[1]);
    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            std::process::exit(1);
        }
    };

    let prefix = args.get(2).map(|s| s.as_str()).unwrap_or("prima");
    let catalog = compile(&source, prefix);

    match serde_json::to_string_pretty(&catalog) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error formatting JSON: {}", e);
            std::process::exit(1);
        }
    }
}

fn demo() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  PRIMA-TO-MCP COMPILER");
    println!("  Write Prima functions → Get MCP tools");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    let source = r#"
// Classify a skill by its primitive count
// Returns tier code: 1=T1, 2=T2-P, 3=T2-C, 4=T3
μ classify_tier(primitive_count: N) → N {
    ∂ primitive_count κ= 1 { 1 }
    else { ∂ primitive_count κ< 4 { 2 }
    else { ∂ primitive_count κ< 6 { 3 }
    else { 4 } } }
}

// Transfer confidence based on tier code
μ transfer_confidence(tier_code: N) → N {
    Σ tier_code { 1 → 100, 2 → 90, 3 → 70, _ → 40 }
}

// Sum all numbers in a sequence
μ sum_sequence(nums: σ[N]) → N {
    nums |> Ω(0, |a,b| a+b)
}
"#;

    println!("INPUT (Prima source):");
    println!("─────────────────────");
    println!("{}", source);

    let catalog = compile(source, "prima_skill");

    println!("OUTPUT (MCP tool catalog):");
    println!("──────────────────────────");
    match serde_json::to_string_pretty(&catalog) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error: {}", e),
    }
}
