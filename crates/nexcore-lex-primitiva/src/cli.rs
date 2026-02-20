//! # Command-Line Interface
//!
//! Binary interface for Lex Primitiva operations.
//!
//! ## Tier: T3-DomainSpecific (Application layer)

use crate::composition::{CompositionAlgebra, CompositionBuilder};
use crate::dossier::DossierGenerator;
use crate::extraction::PrimitiveExtractor;
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::symbols::*;
use crate::tier::Tier;
use crate::transfer::{Domain, TransferCalculator};
use crate::validate::{PrimitivaValidator, validate_system};
use clap::{Parser, Subcommand};

/// Lex Primitiva CLI - Computational primitive analysis toolkit.
#[derive(Parser, Debug)]
#[command(name = "lex-primitiva")]
#[command(author = "Matthew Campion, PharmD; NexVigilant")]
#[command(version = "0.1.0")]
#[command(about = "Analyze and manipulate computational primitives")]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all primitives.
    List {
        /// Show detailed information.
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show primitive details.
    Show {
        /// Primitive symbol (e.g., "σ", "μ") or name (e.g., "Sequence", "Mapping").
        primitive: String,
    },
    /// Extract primitives from text.
    Extract {
        /// Text to analyze.
        text: String,
        /// Minimum confidence threshold (0.0-1.0).
        #[arg(short, long, default_value = "0.1")]
        threshold: f64,
    },
    /// Generate a dossier for a primitive.
    Dossier {
        /// Primitive name or symbol.
        primitive: String,
        /// Output format (markdown, json).
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
    /// Calculate transfer confidence.
    Transfer {
        /// Source domain.
        #[arg(short, long, default_value = "Computation")]
        source: String,
        /// Target domain.
        #[arg(short, long)]
        target: String,
        /// Primitives to transfer (comma-separated symbols).
        primitives: String,
    },
    /// Validate system integrity.
    Validate {
        /// Include info-level messages.
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show dependency graph.
    Graph {
        /// Output as DOT format.
        #[arg(short, long)]
        dot: bool,
    },
    /// Compose primitives.
    Compose {
        /// Primitives to compose (comma-separated).
        primitives: String,
        /// Set dominant primitive.
        #[arg(short, long)]
        dominant: Option<String>,
    },
}

/// Run the CLI.
pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::List { verbose } => cmd_list(verbose),
        Commands::Show { primitive } => cmd_show(&primitive),
        Commands::Extract { text, threshold } => cmd_extract(&text, threshold),
        Commands::Dossier { primitive, format } => cmd_dossier(&primitive, &format),
        Commands::Transfer {
            source,
            target,
            primitives,
        } => cmd_transfer(&source, &target, &primitives),
        Commands::Validate { verbose } => cmd_validate(verbose),
        Commands::Graph { dot } => cmd_graph(dot),
        Commands::Compose {
            primitives,
            dominant,
        } => cmd_compose(&primitives, dominant.as_deref()),
    }
}

fn cmd_list(verbose: bool) -> Result<(), String> {
    println!("Lex Primitiva - 16 Computational Primitives\n");
    for p in LexPrimitiva::all() {
        print_primitive_line(p, verbose);
    }
    if verbose {
        println!("\nRoots: {} (N), {} (→)", N_QUANT, ARROW_CAUSAL);
        println!("Total: 16 primitives, 80 bedrock atoms, 10 constants");
    }
    Ok(())
}

fn print_primitive_line(p: LexPrimitiva, verbose: bool) {
    if verbose {
        let deps = p.derives_from();
        let dep_str = if deps.is_empty() {
            "ROOT".to_string()
        } else {
            deps.iter()
                .map(|d| d.symbol())
                .collect::<Vec<_>>()
                .join(", ")
        };
        println!(
            "  {} ({:12}) - {} [deps: {}]",
            p.symbol(),
            p.name(),
            p.description(),
            dep_str
        );
    } else {
        println!("  {} - {}", p.symbol(), p.name());
    }
}

fn cmd_show(input: &str) -> Result<(), String> {
    let primitive = parse_primitive(input)?;
    let generator = DossierGenerator::new();
    let dossier = generator.for_primitive(primitive);
    println!("{}", dossier.to_markdown());
    Ok(())
}

fn cmd_extract(text: &str, threshold: f64) -> Result<(), String> {
    let extractor = PrimitiveExtractor::with_threshold(threshold);
    let result = extractor.extract(text);
    println!("Extraction Results\n==================\n");
    println!("Input: \"{}\"\n", result.input);
    println!(
        "Tier: {} (confidence: {:.2})\n",
        result.tier.code(),
        result.confidence
    );
    if result.primitives.is_empty() {
        println!("No primitives detected above threshold {:.2}", threshold);
        return Ok(());
    }
    println!("Primitives:");
    for ep in &result.primitives {
        let dominant = if ep.is_dominant { " [DOMINANT]" } else { "" };
        println!(
            "  {} ({}) - {:.2}{}",
            ep.primitive.symbol(),
            ep.primitive.name(),
            ep.confidence,
            dominant
        );
        if !ep.evidence.is_empty() {
            println!("    Evidence: {}", ep.evidence.join(", "));
        }
    }
    Ok(())
}

fn cmd_dossier(input: &str, format: &str) -> Result<(), String> {
    let primitive = parse_primitive(input)?;
    let generator = DossierGenerator::new();
    let dossier = generator.for_primitive(primitive);
    let output = match format {
        "json" => dossier.to_json(),
        _ => dossier.to_markdown(),
    };
    println!("{}", output);
    Ok(())
}

fn cmd_transfer(source: &str, target: &str, primitives: &str) -> Result<(), String> {
    let source_domain = parse_domain(source)?;
    let target_domain = parse_domain(target)?;
    let prims = parse_primitive_list(primitives)?;
    let comp = PrimitiveComposition::new(prims);
    let calc = TransferCalculator::new();
    let result = calc.calculate(&source_domain, &target_domain, &comp);
    println!("Transfer Analysis: {} → {}\n", result.source, result.target);
    println!("Structural:  {:.2}", result.structural);
    println!("Functional:  {:.2}", result.functional);
    println!("Contextual:  {:.2}", result.contextual);
    println!("Aggregate:   {:.2}", result.aggregate);
    println!(
        "Tier:        {} (×{:.1})",
        result.tier.code(),
        result.tier.transfer_multiplier()
    );
    println!("Final:       {:.2}", result.final_confidence);
    if !result.shared_primitives.is_empty() {
        let shared: Vec<_> = result
            .shared_primitives
            .iter()
            .map(|p| p.symbol())
            .collect();
        println!("\nShared: {}", shared.join(", "));
    }
    if !result.limiting_factors.is_empty() {
        println!("\nLimiting factors:");
        for f in &result.limiting_factors {
            println!("  - {}", f);
        }
    }
    Ok(())
}

fn cmd_validate(verbose: bool) -> Result<(), String> {
    let validator = if verbose {
        PrimitivaValidator::new()
    } else {
        PrimitivaValidator::strict()
    };
    let report = validator.validate_system();
    println!("{}", report.to_text());
    if report.passed {
        Ok(())
    } else {
        Err("Validation failed".to_string())
    }
}

fn cmd_graph(dot: bool) -> Result<(), String> {
    if dot {
        print_dot_graph();
    } else {
        print_ascii_graph();
    }
    Ok(())
}

fn print_dot_graph() {
    println!("digraph LexPrimitiva {{");
    println!("  rankdir=BT;");
    println!("  node [shape=box];");
    for p in LexPrimitiva::all() {
        for dep in p.derives_from() {
            println!("  \"{}\" -> \"{}\";", p.symbol(), dep.symbol());
        }
    }
    println!("}}");
}

fn print_ascii_graph() {
    println!("Lex Primitiva Dependency Graph\n");
    let mut levels: Vec<Vec<LexPrimitiva>> = vec![Vec::new(); 7];
    for p in LexPrimitiva::all() {
        let depth = compute_depth(p);
        levels[depth].push(p);
    }
    for (i, level) in levels.iter().enumerate() {
        if level.is_empty() {
            continue;
        }
        let names: Vec<_> = level
            .iter()
            .map(|p| format!("{} ({})", p.symbol(), p.name()))
            .collect();
        println!("Level {}: {}", i, names.join(", "));
    }
}

fn compute_depth(p: LexPrimitiva) -> usize {
    if p.is_root() {
        return 0;
    }
    1 + p
        .derives_from()
        .iter()
        .map(|d| compute_depth(*d))
        .max()
        .unwrap_or(0)
}

fn cmd_compose(primitives: &str, dominant: Option<&str>) -> Result<(), String> {
    let prims = parse_primitive_list(primitives)?;
    let mut builder = CompositionBuilder::new().add_all(&prims);
    if let Some(d) = dominant {
        let dom = parse_primitive(d)?;
        builder = builder.dominant(dom);
    }
    let comp = builder.build();
    let tier = Tier::classify(&comp);
    let algebra = CompositionAlgebra::new();
    let valid = algebra.is_valid(&comp);
    println!("Composition Analysis\n");
    let syms: Vec<_> = comp.primitives.iter().map(|p| p.symbol()).collect();
    println!("Primitives: {}", syms.join(" + "));
    println!("Tier: {}", tier.code());
    println!("Transfer: ×{:.1}", tier.transfer_multiplier());
    println!("Confidence: {:.2}", comp.confidence);
    if let Some(dom) = comp.dominant {
        println!("Dominant: {} ({})", dom.symbol(), dom.name());
    }
    println!("Valid: {}", if valid.valid { "Yes" } else { "No" });
    if !valid.issues.is_empty() {
        println!("Issues:");
        for issue in &valid.issues {
            println!("  - {}", issue);
        }
    }
    Ok(())
}

fn parse_primitive(input: &str) -> Result<LexPrimitiva, String> {
    for p in LexPrimitiva::all() {
        if p.symbol() == input || p.name().eq_ignore_ascii_case(input) {
            return Ok(p);
        }
    }
    Err(format!("Unknown primitive: {}", input))
}

fn parse_primitive_list(input: &str) -> Result<Vec<LexPrimitiva>, String> {
    input
        .split(',')
        .map(|s| parse_primitive(s.trim()))
        .collect()
}

fn parse_domain(input: &str) -> Result<Domain, String> {
    let lower = input.to_lowercase();
    match lower.as_str() {
        "computation" | "comp" => Ok(Domain::computation()),
        "mathematics" | "math" => Ok(Domain::mathematics()),
        "physics" | "phys" => Ok(Domain::physics()),
        "chemistry" | "chem" => Ok(Domain::chemistry()),
        "biology" | "bio" => Ok(Domain::biology()),
        "economics" | "econ" => Ok(Domain::economics()),
        "law" => Ok(Domain::law()),
        _ => Err(format!(
            "Unknown domain: {}. Use: computation, mathematics, physics, chemistry, biology, economics, law",
            input
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primitive_by_symbol() {
        assert_eq!(
            parse_primitive(SIGMA_SEQ).ok(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(parse_primitive(N_QUANT).ok(), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn test_parse_primitive_by_name() {
        assert_eq!(
            parse_primitive("Sequence").ok(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(parse_primitive("mapping").ok(), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn test_parse_primitive_list() {
        let result =
            parse_primitive_list(&format!("{}, {}, {}", SIGMA_SEQ, MU_MAP, VARSIGMA_STATE))
                .expect("Should parse list");
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_domain() {
        assert!(parse_domain("computation").is_ok());
        assert!(parse_domain("Math").is_ok());
        assert!(parse_domain("unknown").is_err());
    }

    #[test]
    fn test_compute_depth() {
        assert_eq!(compute_depth(LexPrimitiva::Quantity), 0);
        assert_eq!(compute_depth(LexPrimitiva::Causality), 0);
        assert!(compute_depth(LexPrimitiva::Frequency) > 0);
    }
}
