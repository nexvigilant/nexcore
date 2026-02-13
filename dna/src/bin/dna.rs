//! nexcore-dna CLI: assemble, disassemble, and run DNA programs.
//!
//! Zero external dependencies — arg parsing via std::env::args().

use nexcore_dna::asm;
use nexcore_dna::cortex;
use nexcore_dna::data;
use nexcore_dna::disasm;
use nexcore_dna::glyph;
use nexcore_dna::isa;
use nexcore_dna::lang::diagnostic;
use nexcore_dna::lang::json;
use nexcore_dna::lang::templates;
use nexcore_dna::lexicon;
use nexcore_dna::pv_theory;
use nexcore_dna::statemind;
use nexcore_dna::storage;
use nexcore_dna::string_theory;
use nexcore_dna::tile::{Tile, TileInspector};
use nexcore_dna::transcode;
use nexcore_dna::types::Strand;
use nexcore_dna::voxel;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = args[1].as_str();
    let result = match command {
        "asm" => cmd_asm(&args[2..]),
        "dis" => cmd_dis(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "catalog" => cmd_catalog(),
        "eval" => cmd_eval(&args[2..]),
        "repl" => cmd_repl(),
        "genome" => cmd_genome(&args[2..]),
        "tile" => cmd_tile(&args[2..]),
        "voxel" => cmd_voxel(&args[2..]),
        "glyph" => cmd_glyph(&args[2..]),
        "encode" => cmd_encode(&args[2..]),
        "decode" => cmd_decode(&args[2..]),
        "lexicon" => cmd_lexicon(&args[2..]),
        "statemind" => cmd_statemind(&args[2..]),
        "cortex" => cmd_cortex(&args[2..]),
        "strings" => cmd_strings(&args[2..]),
        "pv" => cmd_pv(&args[2..]),
        "ast" => cmd_ast(&args[2..]),
        "from-ast" => cmd_from_ast(&args[2..]),
        "template" => cmd_template(&args[2..]),
        "data" => cmd_data(&args[2..]),
        "diagnose" => cmd_diagnose(&args[2..]),
        "--help" | "-h" | "help" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "nexcore-dna — DNA-based computation

USAGE:
    dna <command> [options] [file]

COMMANDS:
    asm <file.dna>          Assemble source to DNA strand
    asm <file.dna> -o <out> Assemble and write strand to file
    dis <file>              Disassemble a DNA strand file
    run <file.dna>          Assemble and execute
    run -s <strand>         Execute raw DNA strand
    eval <expr>             Evaluate expression (high-level)
    eval -f <file>          Evaluate file (high-level)
    repl                    Interactive REPL
    catalog                 Print instruction set catalog
    genome <file>           Show gene catalog for a source file
    genome <file> --express <gene> [args...]
                            Express (run) a specific gene
    tile <expr>             Build & inspect tile from expression
    tile -f <file>          Build & inspect tile from source file
    tile <expr> --checksum  Include checksum in tile
    voxel <expr>            3D chemical classification of instructions
    voxel -f <file>         3D classification from source file
    glyph <expr>            QFA glyph pair mapping of instructions
    glyph -f <file>         Glyph mapping from source file
    encode <text>           Encode text as DNA
    decode <strand>         Decode DNA to text
    lexicon mine <word>     Mine word properties (entropy, GC, distribution)
    lexicon compare <a> <b> Compare two words (edit distance, similarity, LCS)
    lexicon batch <words..> Build vocabulary, show similarity matrix
    statemind project <words..>
                            Project words into 3D mind-space
    statemind simulate <word> [count]
                            Simulate mutations and track drift
    statemind auto <seeds..>
                            Auto-mine seeds + all single-char mutations
    cortex cluster <k> <words..>
                            K-means clustering in 3D word-space
    cortex gravity <ticks> <words..>
                            N-body gravity simulation
    cortex evolve <target> <generations> <seeds..>
                            Evolve words toward target via GA
    strings tension <words..>
                            Compute string tension from DNA bond energies
    strings spectrum <words..>
                            Frequency spectrum via autocorrelation
    strings resonance <a> <b>
                            Spectral resonance between two words
    strings energy <words..>
                            Tension + information energy analysis
    ast <expr>              Export expression AST as JSON
    ast -f <file>           Export file AST as JSON
    ast <expr> --pretty     Pretty-printed JSON AST
    from-ast <json>         Import JSON AST and eval
    from-ast -f <file>      Import JSON AST file and eval
    template                List available templates
    template <name> <args>  Expand and eval a template
    template <name> <args> --source
                            Show expanded source only
    data encode <type> <value>
                            Encode typed value as DNA strand
    data decode <strand>    Decode TLV DNA back to value
    data record <key=value...>
                            Build record from key=value pairs
    data inspect <strand>   Show TLV structure breakdown
    diagnose <expr>         Structured error diagnostics (JSON)
    diagnose -f <file>      Diagnose from file
    pv profile <drugs..>    Profile drugs as DNA-encoded compounds
    pv signal <drug> <event>
                            Detect signal between drug and event
    pv margin <entity> [baseline]
                            Safety margin (Theory of Vigilance d(s))
    pv causality <drug> <event>
                            Assess causality (proximity + mechanistic)
    pv monitor <drug> <events..>
                            Full vigilance state monitoring

OPTIONS:
    -o <file>    Output file
    -s <strand>  Raw strand input
    -v           Verbose output
    --help       Show this help"
    );
}

fn cmd_asm(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("asm requires an input file".into());
    }

    let input = &args[0];
    let source =
        std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?;

    let program = asm::assemble(&source).map_err(|e| format!("{e}"))?;

    let strand_str = program.strand().to_string_repr();

    // Check for -o flag
    if args.len() >= 3 && args[1] == "-o" {
        let output = &args[2];
        std::fs::write(output, &strand_str).map_err(|e| format!("cannot write '{output}': {e}"))?;
        let codon_count = program.codon_count().unwrap_or(0);
        eprintln!(
            "Assembled: {} codons, {} nucleotides → {output}",
            codon_count,
            strand_str.len()
        );
    } else {
        println!("{strand_str}");
        let codon_count = program.codon_count().unwrap_or(0);
        eprintln!(
            "Assembled: {} codons, {} nucleotides",
            codon_count,
            strand_str.len()
        );
    }

    Ok(())
}

fn cmd_dis(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("dis requires an input file or -s <strand>".into());
    }

    let strand = if args[0] == "-s" {
        if args.len() < 2 {
            return Err("-s requires a strand argument".into());
        }
        Strand::parse(&args[1]).map_err(|e| format!("{e}"))?
    } else {
        let input = &args[0];
        let content =
            std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?;
        Strand::parse(content.trim()).map_err(|e| format!("{e}"))?
    };

    let listing = disasm::disassemble(&strand).map_err(|e| format!("{e}"))?;
    print!("{listing}");

    Ok(())
}

fn cmd_run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("run requires an input file or -s <strand>".into());
    }

    let verbose = args.iter().any(|a| a == "-v");

    if args[0] == "-s" {
        // Run raw strand
        if args.len() < 2 {
            return Err("-s requires a strand argument".into());
        }
        let strand = Strand::parse(&args[1]).map_err(|e| format!("{e}"))?;
        let program = nexcore_dna::program::Program::code_only(strand);
        let result = program.run().map_err(|e| format!("{e}"))?;

        print_result(&result, verbose);
    } else {
        // Assemble and run
        let input = &args[0];
        let source =
            std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?;
        let program = asm::assemble(&source).map_err(|e| format!("{e}"))?;
        let result = program.run().map_err(|e| format!("{e}"))?;

        print_result(&result, verbose);
    }

    Ok(())
}

fn print_result(result: &nexcore_dna::vm::VmResult, verbose: bool) {
    // Output buffer
    if !result.output.is_empty() {
        for val in &result.output {
            println!("{val}");
        }
    }

    if verbose {
        eprintln!("---");
        eprintln!("Halt:   {:?}", result.halt_reason);
        eprintln!("Cycles: {}", result.cycles);
        if !result.stack.is_empty() {
            eprintln!("Stack:  {:?}", result.stack);
        }
    }
}

fn cmd_catalog() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{:<5} {:<12} {:<5} {:<5} STACK EFFECT",
        "IDX", "MNEMONIC", "CODON", "AA"
    );
    println!("{}", "-".repeat(50));

    for entry in isa::catalog() {
        println!(
            "{:<5} {:<12} {:<5} {:<5} {}",
            entry.index, entry.mnemonic, entry.codon_str, entry.amino_acid, entry.stack_effect
        );
    }

    Ok(())
}

fn cmd_eval(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("eval requires an expression or -f <file>".into());
    }

    let verbose = args.iter().any(|a| a == "-v");

    let source = if args[0] == "-f" {
        // Evaluate from file
        if args.len() < 2 {
            return Err("-f requires a file argument".into());
        }
        let input = &args[1];
        std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?
    } else {
        // Evaluate inline expression(s)
        args.iter()
            .filter(|a| a.as_str() != "-v")
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    };

    let result = nexcore_dna::lang::compiler::eval(&source).map_err(|e| format!("{e}"))?;

    for val in &result.output {
        println!("{val}");
    }

    if verbose {
        eprintln!("---");
        eprintln!("Halt:   {:?}", result.halt_reason);
        eprintln!("Cycles: {}", result.cycles);
        if !result.stack.is_empty() {
            eprintln!("Stack:  {:?}", result.stack);
        }
    }

    Ok(())
}

fn cmd_repl() -> Result<(), Box<dyn std::error::Error>> {
    use nexcore_dna::lang::repl::{Repl, ReplAction};

    eprintln!("nexcore-dna REPL v0.2.0 — type :help for commands, :quit to exit");

    let mut repl = Repl::new();
    let stdin = std::io::stdin();
    let mut line_buf = String::new();

    loop {
        // Prompt
        if repl.is_continuation() {
            eprint!("... ");
        } else {
            eprint!("dna> ");
        }
        // Flush stderr prompt (eprint doesn't auto-flush)
        use std::io::Write;
        std::io::stderr().flush().ok();

        line_buf.clear();
        let bytes = stdin.read_line(&mut line_buf)?;
        if bytes == 0 {
            // EOF
            eprintln!();
            break;
        }

        match repl.eval_line(&line_buf) {
            ReplAction::Output(vals) => {
                for val in &vals {
                    println!("{val}");
                }
            }
            ReplAction::Meta(text) => {
                println!("{text}");
            }
            ReplAction::Error(msg) => {
                eprintln!("Error: {msg}");
            }
            ReplAction::Empty => {}
            ReplAction::Continue => {}
            ReplAction::Exit => break,
        }
    }

    Ok(())
}

fn cmd_genome(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("genome requires an input file".into());
    }

    let input = &args[0];
    let source =
        std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?;

    let genome =
        nexcore_dna::lang::compiler::compile_genome(&source).map_err(|e| format!("{e}"))?;

    // Check for --express flag
    if args.len() >= 3 && args[1] == "--express" {
        let gene_name = &args[2];
        let gene_args: Vec<i64> = args[3..]
            .iter()
            .filter_map(|a| a.parse::<i64>().ok())
            .collect();
        let result = genome
            .express(gene_name, &gene_args)
            .map_err(|e| format!("{e}"))?;
        if !result.output.is_empty() {
            for val in &result.output {
                println!("{val}");
            }
        }
        if !result.stack.is_empty() {
            println!("{}", result.stack[result.stack.len() - 1]);
        }
    } else {
        // Display catalog
        println!("{:<20} {:<8} {:<8}", "GENE", "ARITY", "CODONS");
        println!("{}", "-".repeat(36));
        for (name, arity, codons) in genome.catalog() {
            println!("{:<20} {:<8} {:<8}", name, arity, codons);
        }
        println!();
        println!(
            "Total: {} genes, {} codons, GC={:.1}%",
            genome.gene_count(),
            genome.codon_count(),
            genome.gc_content() * 100.0
        );
    }

    Ok(())
}

fn cmd_tile(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("tile requires an expression or -f <file>".into());
    }

    let source = if args[0] == "-f" {
        if args.len() < 2 {
            return Err("-f requires a file argument".into());
        }
        let input = &args[1];
        std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?
    } else {
        args.iter()
            .filter(|a| a.as_str() != "-v" && a.as_str() != "--checksum")
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    };

    let with_checksum = args.iter().any(|a| a == "--checksum");

    // Compile to get instructions
    let program =
        nexcore_dna::lang::compiler::compile(&source).map_err(|e| format!("compile: {e}"))?;
    let strand = program.strand();
    let codons = strand.codons().map_err(|e| format!("{e}"))?;
    let instrs: Vec<nexcore_dna::isa::Instruction> =
        codons.iter().map(nexcore_dna::isa::decode).collect();

    // Transcode analysis
    let rec = transcode::recommend(&instrs);
    let profile = transcode::ProgramProfile::analyze(&instrs);

    eprintln!(
        "Source: {} chars → {} instructions",
        source.trim().len(),
        instrs.len()
    );
    eprintln!("Recommendation: {:?} — {}", rec.encoding, rec.reason);
    eprintln!(
        "Profile: {} Lits, lits_fit_u8={}, families={}",
        profile.lit_count, profile.lits_fit_u8, profile.family_coverage
    );
    eprintln!();

    // Build tile
    let mut tile = Tile::from_instructions(&instrs);

    if with_checksum {
        let hash = tile.compute_checksum();
        tile.set_checksum(&hash);
    }

    // Inspect
    let inspector = TileInspector::new(&tile);
    println!("{}", inspector.report());

    // Also run the program and show output
    let result = program.run().map_err(|e| format!("{e}"))?;
    if !result.output.is_empty() {
        eprintln!("VM Output: {:?}", result.output);
        eprintln!("Cycles:    {}", result.cycles);
    }

    Ok(())
}

fn cmd_encode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("encode requires text argument".into());
    }
    let text = args.join(" ");
    let strand = storage::encode_str(&text);
    println!("{}", strand.to_string_repr());
    Ok(())
}

fn cmd_decode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("decode requires a strand argument".into());
    }
    let strand = Strand::parse(&args[0]).map_err(|e| format!("{e}"))?;
    let text = storage::decode_str(&strand).map_err(|e| format!("{e}"))?;
    println!("{text}");
    Ok(())
}

fn cmd_voxel(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("voxel requires an expression or -f <file>".into());
    }

    let source = if args[0] == "-f" {
        if args.len() < 2 {
            return Err("-f requires a file argument".into());
        }
        let input = &args[1];
        std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?
    } else {
        args.iter()
            .filter(|a| a.as_str() != "-v")
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    };

    // Compile to instructions
    let program =
        nexcore_dna::lang::compiler::compile(&source).map_err(|e| format!("compile: {e}"))?;
    let strand = program.strand();
    let codons = strand.codons().map_err(|e| format!("{e}"))?;
    let instrs: Vec<nexcore_dna::isa::Instruction> =
        codons.iter().map(nexcore_dna::isa::decode).collect();

    eprintln!(
        "Source: {} chars → {} instructions",
        source.trim().len(),
        instrs.len()
    );

    // Build voxel cube
    let cube = voxel::VoxelCube::from_instructions(&instrs);
    let total_a = cube.total_absorbance();

    // Header
    println!("╔══════════════════════════════════╗");
    println!("║       VOXEL CUBE (4×4×4)         ║");
    println!("╚══════════════════════════════════╝");
    println!();

    // Classification table
    println!(
        "{:<5} {:<12} {:<12} {:<10} {:<10}",
        "IDX", "INSTRUCTION", "CHARGE", "ENERGY", "STABILITY"
    );
    println!("{}", "-".repeat(50));
    for (i, instr) in instrs.iter().enumerate() {
        let _pos = voxel::classify(instr);
        println!(
            "{:<5} {:<12} {:<12} {:<10} {:<10}",
            i,
            format!("{instr:?}"),
            format!("{:?}", voxel::charge_of(instr)),
            format!("{:?}", voxel::energy_of(instr)),
            format!("{:?}", voxel::stability_of(instr)),
        );
    }
    println!();

    // Projections
    let labels = ["X (Charge)", "Y (Energy)", "Z (Stability)"];
    let projections = [cube.project_x(), cube.project_y(), cube.project_z()];

    for (label, proj) in labels.iter().zip(projections.iter()) {
        println!("Projection {label}:");
        for row in proj {
            let cells: Vec<String> = row.iter().map(|v| format!("{v:6.2}")).collect();
            println!("  [{}]", cells.join(", "));
        }
        println!();
    }

    println!("Total absorbance: {total_a:.4}");
    println!("Total concentration: {:.4}", cube.total_concentration());

    Ok(())
}

fn cmd_glyph(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("glyph requires an expression or -f <file>".into());
    }

    let source = if args[0] == "-f" {
        if args.len() < 2 {
            return Err("-f requires a file argument".into());
        }
        let input = &args[1];
        std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?
    } else {
        args.iter()
            .filter(|a| a.as_str() != "-v")
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    };

    // Compile to instructions
    let program =
        nexcore_dna::lang::compiler::compile(&source).map_err(|e| format!("compile: {e}"))?;
    let strand = program.strand();
    let codons = strand.codons().map_err(|e| format!("{e}"))?;
    let instrs: Vec<nexcore_dna::isa::Instruction> =
        codons.iter().map(nexcore_dna::isa::decode).collect();

    eprintln!(
        "Source: {} chars → {} instructions",
        source.trim().len(),
        instrs.len()
    );

    // Header
    println!("╔══════════════════════════════════╗");
    println!("║     QFA GLYPH PAIR MAPPING       ║");
    println!("╚══════════════════════════════════╝");
    println!();

    println!(
        "{:<5} {:<12} {:<8} {:<8} {:<6} {:<10}",
        "IDX", "INSTRUCTION", "FAMILY", "VARIANT", "INDEX", "GLYPH"
    );
    println!("{}", "-".repeat(55));

    let mut family_counts = [0u32; 8];

    for (i, instr) in instrs.iter().enumerate() {
        if let Some(gp) = glyph::glyph_for_instruction(instr) {
            let fam = gp.p0.index();
            let var = gp.p1.index();
            let idx = gp.glyph_index();
            if (fam as usize) < family_counts.len() {
                family_counts[fam as usize] += 1;
            }
            println!(
                "{:<5} {:<12} {:<8} {:<8} {:<6} {}",
                i,
                format!("{instr:?}"),
                fam,
                var,
                idx,
                gp,
            );
        } else {
            println!("{:<5} {:<12} (no glyph mapping)", i, format!("{instr:?}"));
        }
    }

    println!();
    println!("Family distribution:");
    let family_names = [
        "∂ Boundary",
        "μ Transform",
        "σ Data",
        "κ Compare",
        "→ Causality",
        "ν Frequency",
        "ς State",
        "N Quantity",
    ];
    for (fam, count) in family_counts.iter().enumerate() {
        if *count > 0 {
            let bar: String = "█".repeat(*count as usize);
            let name = if fam < family_names.len() {
                family_names[fam]
            } else {
                "?"
            };
            println!("  F{fam} {:<12} {bar} ({count})", name);
        }
    }

    // Hamming analysis for adjacent instructions
    if instrs.len() >= 2 {
        println!();
        println!("Hamming distances (adjacent pairs):");
        let mut total_d = 0u32;
        let mut pairs = 0u32;
        for w in instrs.windows(2) {
            let g0 = glyph::glyph_for_instruction(&w[0]);
            let g1 = glyph::glyph_for_instruction(&w[1]);
            if let (Some(a), Some(b)) = (g0, g1) {
                let d = a.hamming_distance(&b);
                total_d += d as u32;
                pairs += 1;
            }
        }
        if pairs > 0 {
            let avg = total_d as f64 / pairs as f64;
            println!("  {pairs} pairs, avg distance: {avg:.2} (range 0-2)");
        }
    }

    Ok(())
}

fn cmd_lexicon(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("lexicon requires a subcommand: mine, compare, batch".into());
    }

    match args[0].as_str() {
        "mine" => {
            if args.len() < 2 {
                return Err("lexicon mine requires a word".into());
            }
            let word = &args[1];
            let ore = lexicon::mine(word);
            println!("{ore}");
            Ok(())
        }
        "compare" => {
            if args.len() < 3 {
                return Err("lexicon compare requires two words".into());
            }
            let a = &args[1];
            let b = &args[2];
            let aff = lexicon::compare(a, b);
            println!("{aff}");
            Ok(())
        }
        "batch" => {
            if args.len() < 2 {
                return Err("lexicon batch requires at least one word".into());
            }
            let words: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
            let mut lex = lexicon::Lexicon::new();
            for w in &words {
                lex.insert(w);
            }

            // Display each word's properties
            println!("=== Word Properties ===");
            for entry in lex.entries() {
                println!("  {entry}");
            }
            println!();

            // Similarity matrix
            let matrix = lex.similarity_matrix();
            println!("=== Similarity Matrix ===");
            // Header
            print!("{:<12}", "");
            for w in &words {
                print!("{:<10}", w);
            }
            println!();

            for (i, row) in matrix.iter().enumerate() {
                print!("{:<12}", words[i]);
                for val in row {
                    print!("{val:<10.2}");
                }
                println!();
            }
            println!();

            // Entropy ranking
            println!("=== Entropy Ranking ===");
            let mut ranked: Vec<&lexicon::WordOre> = lex.entries().iter().collect();
            ranked.sort_by(|a, b| {
                b.entropy
                    .partial_cmp(&a.entropy)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for (i, ore) in ranked.iter().enumerate() {
                println!("  {}. \"{}\" H={:.4}", i + 1, ore.word, ore.entropy);
            }

            Ok(())
        }
        other => Err(format!("unknown lexicon subcommand: {other}").into()),
    }
}

fn cmd_statemind(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("statemind requires a subcommand: project, simulate, auto".into());
    }

    match args[0].as_str() {
        "project" => {
            if args.len() < 2 {
                return Err("statemind project requires at least one word".into());
            }
            let words: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
            let mut mind = statemind::StateMind::new();
            for w in &words {
                mind.ingest(w);
            }

            println!("=== 3D Word-Space Projection ===");
            println!(
                "{:<15} {:<10} {:<10} {:<10}",
                "WORD", "ENTROPY", "GC", "DENSITY"
            );
            println!("{}", "-".repeat(45));
            for w in &words {
                if let Some(pt) = mind.project(w) {
                    println!(
                        "{:<15} {:<10.4} {:<10.4} {:<10.4}",
                        w, pt.entropy_norm, pt.gc_content, pt.density
                    );
                }
            }
            println!();

            let c = mind.centroid();
            let r = mind.radius();
            println!("Centroid: {c}");
            println!("Radius:   {r:.4}");
            println!();

            let stats = mind.dimension_stats();
            let labels = ["Entropy", "GC Content", "Density"];
            println!("=== Dimension Statistics ===");
            for (label, s) in labels.iter().zip(stats.iter()) {
                println!("  {label:<12} {s}");
            }

            Ok(())
        }
        "simulate" => {
            if args.len() < 2 {
                return Err("statemind simulate requires a word".into());
            }
            let word = &args[1];
            let count: usize = if args.len() >= 3 {
                args[2]
                    .parse()
                    .map_err(|_| "count must be a positive integer")?
            } else {
                10
            };

            let mind = statemind::StateMind::new();
            let results = mind.simulate_mutation(word, count);

            println!("=== Mutation Simulation: \"{word}\" ({count} mutations) ===");
            println!("{:<15} {:<15} {:<10}", "MUTANT", "POINT", "DRIFT");
            println!("{}", "-".repeat(40));
            for r in &results {
                println!("{:<15} {} {}", r.mutant, r.mutant_point, r.drift);
            }

            if !results.is_empty() {
                let avg_drift: f64 =
                    results.iter().map(|r| r.drift.magnitude).sum::<f64>() / results.len() as f64;
                let max_drift = results
                    .iter()
                    .map(|r| r.drift.magnitude)
                    .fold(0.0_f64, f64::max);
                println!();
                println!("Avg drift: {avg_drift:.6}");
                println!("Max drift: {max_drift:.6}");
            }

            Ok(())
        }
        "auto" => {
            if args.len() < 2 {
                return Err("statemind auto requires at least one seed word".into());
            }
            let seeds: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
            let mut mind = statemind::StateMind::new();
            let added = mind.auto_mine(&seeds);

            println!("=== Auto-Mine Results ===");
            println!("Seeds: {}", seeds.join(", "));
            println!("Words mined: {added}");
            println!();
            println!("{mind}");
            println!();

            let stats = mind.dimension_stats();
            let labels = ["Entropy", "GC Content", "Density"];
            println!("=== Dimension Statistics ===");
            for (label, s) in labels.iter().zip(stats.iter()) {
                println!("  {label:<12} {s}");
            }
            println!();

            // Show top-5 nearest to centroid
            let c = mind.centroid();
            let nearest = mind.nearest_3d(&c, 5);
            println!("=== Nearest to Centroid ===");
            for (ore, pt, dist) in &nearest {
                println!("  \"{}\"\t{}\td={:.4}", ore.word, pt, dist);
            }

            Ok(())
        }
        other => Err(format!("unknown statemind subcommand: {other}").into()),
    }
}

fn cmd_cortex(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("cortex requires a subcommand: cluster, gravity, evolve".into());
    }

    match args[0].as_str() {
        "cluster" => {
            if args.len() < 3 {
                return Err("cortex cluster requires <k> <words..>".into());
            }
            let k: usize = args[1]
                .parse()
                .map_err(|_| "k must be a positive integer")?;
            let words: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

            let mut mind = statemind::StateMind::new();
            for w in &words {
                mind.ingest(w);
            }

            let result = cortex::kmeans(&mind, k, 1000);
            println!("{result}");
            println!();

            for cluster in &result.clusters {
                println!("{cluster}");
                // Show member words
                let entries = mind.lexicon().entries();
                for &idx in &cluster.members {
                    if idx < entries.len() {
                        let pt = &mind.points()[idx];
                        println!("  \"{}\" {}", entries[idx].word, pt);
                    }
                }
            }

            Ok(())
        }
        "gravity" => {
            if args.len() < 3 {
                return Err("cortex gravity requires <ticks> <words..>".into());
            }
            let max_ticks: usize = args[1]
                .parse()
                .map_err(|_| "ticks must be a positive integer")?;
            let words: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

            let mut mind = statemind::StateMind::new();
            for w in &words {
                mind.ingest(w);
            }

            let config = cortex::GravityConfig {
                max_ticks,
                ..cortex::GravityConfig::default()
            };
            let result = cortex::gravity_sim(&mind, config);
            println!("{result}");
            println!();

            let entries = mind.lexicon().entries();
            println!("{:<15} {:<30} {:<10}", "WORD", "FINAL POSITION", "MASS");
            println!("{}", "-".repeat(55));
            for p in &result.particles {
                let word = if p.word_idx < entries.len() {
                    &entries[p.word_idx].word
                } else {
                    "?"
                };
                println!("{:<15} {:<30} {:.2}", word, p.position, p.mass);
            }

            if !result.snapshots.is_empty() {
                println!();
                println!("Energy snapshots:");
                for snap in &result.snapshots {
                    println!(
                        "  t={:<5} KE={:.6}  PE={:.6}",
                        snap.tick, snap.kinetic_energy, snap.potential_energy
                    );
                }
            }

            Ok(())
        }
        "evolve" => {
            if args.len() < 4 {
                return Err("cortex evolve requires <target> <generations> <seeds..>".into());
            }
            let target = &args[1];
            let generations: usize = args[2]
                .parse()
                .map_err(|_| "generations must be a positive integer")?;
            let seeds: Vec<&str> = args[3..].iter().map(|s| s.as_str()).collect();

            let config = cortex::EvolutionConfig {
                generations,
                ..cortex::EvolutionConfig::default()
            };
            let result = cortex::evolve(&seeds, target, config);
            println!("{result}");
            println!();

            println!("Best: {}", result.best);
            if let Some(conv) = result.converged_at {
                println!("Converged at generation {conv}");
            }
            println!();

            // Show last 5 generations
            let start = result.generations.len().saturating_sub(5);
            println!("Recent generations:");
            for g in &result.generations[start..] {
                println!("  {g}");
            }

            Ok(())
        }
        other => Err(format!("unknown cortex subcommand: {other}").into()),
    }
}

// ── String Theory ────────────────────────────────────────────────────

fn cmd_strings(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("strings requires a subcommand: tension, spectrum, resonance, energy".into());
    }

    let sub = args[0].as_str();
    let args = &args[1..];

    match sub {
        "tension" => {
            if args.is_empty() {
                return Err("strings tension requires <words..>".into());
            }
            for word in args {
                let t = string_theory::word_tension(word);
                println!("{t}");
            }
            Ok(())
        }
        "spectrum" => {
            if args.is_empty() {
                return Err("strings spectrum requires <words..>".into());
            }
            for word in args {
                let s = string_theory::word_spectrum(word);
                println!("{s}");
                if !s.modes.is_empty() {
                    println!("  Modes:");
                    for mode in &s.modes {
                        println!("    {mode}");
                    }
                }
                println!();
            }
            Ok(())
        }
        "resonance" => {
            if args.len() < 2 {
                return Err("strings resonance requires <a> <b>".into());
            }
            let r = string_theory::word_resonance(&args[0], &args[1]);
            println!("{r}");
            Ok(())
        }
        "energy" => {
            if args.is_empty() {
                return Err("strings energy requires <words..>".into());
            }
            for word in args {
                let e = string_theory::string_energy(word);
                println!("{e}");
            }
            Ok(())
        }
        other => Err(format!("unknown strings subcommand: {other}").into()),
    }
}

// ── Data ─────────────────────────────────────────────────────────────

fn cmd_data(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("data requires a subcommand: encode, decode, record, inspect".into());
    }

    match args[0].as_str() {
        "encode" => {
            if args.len() < 3 {
                return Err("data encode requires <type> <value>".into());
            }
            let type_name = args[1].as_str();
            let raw = args[2..].join(" ");
            let value = match type_name {
                "null" => data::DnaValue::null(),
                "bool" => {
                    let b = raw == "true" || raw == "1";
                    data::DnaValue::bool(b)
                }
                "int" => {
                    let n: i64 = raw.parse().map_err(|_| format!("invalid int: {raw}"))?;
                    data::DnaValue::int(n)
                }
                "float" => {
                    let n: f64 = raw.parse().map_err(|_| format!("invalid float: {raw}"))?;
                    data::DnaValue::float(n)
                }
                "text" => data::DnaValue::text(&raw),
                other => return Err(format!("unknown type: {other}").into()),
            };
            let tlv = value.encode_tlv().map_err(|e| format!("{e}"))?;
            println!("{}", tlv.to_string_repr());
            eprintln!(
                "Type: {} | Value: {} | Nucleotides: {}",
                value.dtype,
                value,
                tlv.len()
            );
            Ok(())
        }
        "decode" => {
            if args.len() < 2 {
                return Err("data decode requires a strand".into());
            }
            let strand = Strand::parse(&args[1]).map_err(|e| format!("{e}"))?;
            let (value, consumed) =
                data::DnaValue::decode_tlv(&strand.bases, 0).map_err(|e| format!("{e}"))?;
            println!("{value}");
            eprintln!("Type: {} | Consumed: {} nucs", value.dtype, consumed);
            Ok(())
        }
        "record" => {
            if args.len() < 2 {
                return Err("data record requires key=value pairs".into());
            }
            let mut rec = data::DnaRecord::new();
            for pair in &args[1..] {
                if let Some((key, val)) = pair.split_once('=') {
                    // Try int, then float, then text
                    let value = if let Ok(n) = val.parse::<i64>() {
                        data::DnaValue::int(n)
                    } else if let Ok(n) = val.parse::<f64>() {
                        data::DnaValue::float(n)
                    } else if val == "true" || val == "false" {
                        data::DnaValue::bool(val == "true")
                    } else if val == "null" {
                        data::DnaValue::null()
                    } else {
                        data::DnaValue::text(val)
                    };
                    rec.set(key.to_string(), value);
                } else {
                    return Err(format!("invalid pair (expected key=value): {pair}").into());
                }
            }
            println!("{rec}");
            let encoded = rec.encode().map_err(|e| format!("{e}"))?;
            println!("{}", encoded.to_string_repr());
            eprintln!("Fields: {} | Nucleotides: {}", rec.len(), encoded.len());
            Ok(())
        }
        "inspect" => {
            if args.len() < 2 {
                return Err("data inspect requires a strand".into());
            }
            let strand = Strand::parse(&args[1]).map_err(|e| format!("{e}"))?;
            let (value, consumed) =
                data::DnaValue::decode_tlv(&strand.bases, 0).map_err(|e| format!("{e}"))?;
            println!("TLV Structure:");
            println!(
                "  Type:     {} (codon index {})",
                value.dtype,
                value.dtype.index()
            );
            println!("  Length:   {} nucleotides", value.strand.len());
            println!("  Value:    {value}");
            println!("  Consumed: {consumed} nucleotides total");
            println!(
                "  Overhead: {} nucleotides (header)",
                consumed - value.strand.len()
            );
            Ok(())
        }
        other => Err(format!("unknown data subcommand: {other}").into()),
    }
}

// ── Diagnose ─────────────────────────────────────────────────────────

fn cmd_diagnose(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("diagnose requires an expression or -f <file>".into());
    }

    let source = if args[0] == "-f" {
        if args.len() < 2 {
            return Err("-f requires a file argument".into());
        }
        let input = &args[1];
        std::fs::read_to_string(input).map_err(|e| format!("cannot read '{input}': {e}"))?
    } else {
        args.join(" ")
    };

    let diags = diagnostic::diagnose(&source);

    if diags.is_empty() {
        println!("[]");
        eprintln!("No errors found.");
    } else {
        println!("{}", diagnostic::diagnostics_to_json(&diags));
        eprintln!("{} diagnostic(s)", diags.len());
    }

    Ok(())
}

// ── PV Theory ────────────────────────────────────────────────────────

fn cmd_pv(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("pv requires a subcommand: profile, signal, margin, causality, monitor".into());
    }

    let sub = args[0].as_str();
    let args = &args[1..];

    match sub {
        "profile" => {
            if args.is_empty() {
                return Err("pv profile requires <drugs..>".into());
            }
            for drug in args {
                let p = pv_theory::profile_drug(drug);
                println!("{p}");
            }
            Ok(())
        }
        "signal" => {
            if args.len() < 2 {
                return Err("pv signal requires <drug> <event>".into());
            }
            let s = pv_theory::detect_signal(&args[0], &args[1]);
            println!("{s}");
            Ok(())
        }
        "margin" => {
            if args.is_empty() {
                return Err("pv margin requires <entity> [baseline]".into());
            }
            let baseline = if args.len() > 1 {
                // Use second word as baseline reference
                let ore = lexicon::mine(&args[1]);
                statemind::MindPoint::from_ore(&ore)
            } else {
                statemind::MindPoint::origin()
            };
            let m = pv_theory::safety_margin(&args[0], &baseline);
            println!("{m}");
            Ok(())
        }
        "causality" => {
            if args.len() < 2 {
                return Err("pv causality requires <drug> <event>".into());
            }
            let c = pv_theory::assess_causality(&args[0], &args[1]);
            println!("{c}");
            Ok(())
        }
        "monitor" => {
            if args.len() < 2 {
                return Err("pv monitor requires <drug> <events..>".into());
            }
            let drug = &args[0];
            let events: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
            let v = pv_theory::monitor(drug, &events);
            println!("{v}");
            println!();

            // Show individual signals
            println!("Signals:");
            for sig in &v.signals {
                println!("  {sig}");
            }
            Ok(())
        }
        other => Err(format!("unknown pv subcommand: {other}").into()),
    }
}

// ---------------------------------------------------------------------------
// AST — JSON AST export
// ---------------------------------------------------------------------------

fn cmd_ast(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("ast requires source expression or -f <file>".into());
    }

    let source = if args[0] == "-f" {
        if args.len() < 2 {
            return Err("ast -f requires a file path".into());
        }
        std::fs::read_to_string(&args[1]).map_err(|e| format!("cannot read '{}': {e}", args[1]))?
    } else {
        args.join(" ")
    };

    let pretty = args.iter().any(|a| a == "--pretty" || a == "-p");

    if pretty {
        let stmts = nexcore_dna::lang::parser::parse(&source).map_err(|e| format!("{e}"))?;
        let output = json::ast_to_json_pretty(&stmts);
        println!("{output}");
    } else {
        let json_str = json::source_to_json(&source).map_err(|e| format!("{e}"))?;
        println!("{json_str}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// FROM-AST — JSON AST import and compile/eval
// ---------------------------------------------------------------------------

fn cmd_from_ast(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("from-ast requires JSON string or -f <file>".into());
    }

    let json_str = if args[0] == "-f" {
        if args.len() < 2 {
            return Err("from-ast -f requires a file path".into());
        }
        std::fs::read_to_string(&args[1]).map_err(|e| format!("cannot read '{}': {e}", args[1]))?
    } else {
        args.join(" ")
    };

    let result = json::json_eval(&json_str).map_err(|e| format!("{e}"))?;

    println!("Output: {:?}", result.output);
    println!("Cycles: {}", result.cycles);
    if !result.stack.is_empty() {
        println!("Stack:  {:?}", result.stack);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TEMPLATE — code generation templates
// ---------------------------------------------------------------------------

fn cmd_template(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        println!("Available templates:");
        println!();
        for t in templates::catalog() {
            let params_str = t.params.join(", ");
            println!(
                "  {:<12} {}({})  — {}",
                t.name, t.name, params_str, t.description
            );
        }
        return Ok(());
    }

    let name = &args[0];

    // Check for --source flag (show source instead of eval)
    let show_source = args.iter().any(|a| a == "--source" || a == "-s");

    // Parse numeric arguments (skip flags)
    let num_args: Vec<i64> = args[1..]
        .iter()
        .filter(|a| !a.starts_with('-'))
        .map(|a| {
            a.parse::<i64>()
                .map_err(|_| format!("invalid number: '{a}'"))
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if show_source {
        let source = templates::expand(name, &num_args).map_err(|e| format!("{e}"))?;
        println!("{source}");
    } else {
        let output = templates::expand_eval(name, &num_args).map_err(|e| format!("{e}"))?;
        println!("{output:?}");
    }
    Ok(())
}
