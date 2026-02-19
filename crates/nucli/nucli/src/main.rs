//! nucli CLI: encode text as DNA, decode DNA back to text.

use nucli::{codec, complement};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let result = match args[1].as_str() {
        "encode" => cmd_encode(&args[2..]),
        "decode" => cmd_decode(&args[2..]),
        "complement" => cmd_complement(&args[2..]),
        "stats" => cmd_stats(&args[2..]),
        "--help" | "-h" | "help" => {
            print_usage();
            Ok(())
        }
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "nucli — nucleotide text encoder

USAGE:
    nucli <command> <input>

COMMANDS:
    encode <text>       Encode text as a DNA strand
    decode <strand>     Decode a DNA strand back to text
    complement <strand> Reverse complement of a DNA strand
    stats <strand>      Show strand statistics (length, GC content)
    help                Show this help"
    );
}

fn cmd_encode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("encode requires text argument".into());
    }
    let text = args.join(" ");
    let strand = codec::encode(text.as_bytes());
    println!("{strand}");
    eprintln!(
        "{} bytes → {} nucleotides",
        text.len(),
        strand.len()
    );
    Ok(())
}

fn cmd_decode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("decode requires a strand argument".into());
    }
    let bytes = codec::decode(&args[0])?;
    let text = String::from_utf8(bytes).map_err(|e| format!("invalid UTF-8: {e}"))?;
    println!("{text}");
    Ok(())
}

fn cmd_complement(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("complement requires a strand argument".into());
    }
    // Validate input first
    codec::validate(&args[0])?;
    let rc = complement(&args[0]);
    println!("{rc}");
    Ok(())
}

fn cmd_stats(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("stats requires a strand argument".into());
    }
    codec::validate(&args[0])?;
    let strand = &args[0];
    let len = strand.len();
    let gc: usize = strand.chars().filter(|&c| c == 'G' || c == 'C').count();
    let gc_pct = if len > 0 {
        gc as f64 / len as f64 * 100.0
    } else {
        0.0
    };

    println!("Length:      {len} nucleotides ({} bytes)", len / 4);
    println!("GC content:  {gc}/{len} ({gc_pct:.1}%)");
    println!(
        "Valid:       {}",
        if len % 4 == 0 { "yes" } else { "no (incomplete tetrad)" }
    );
    Ok(())
}
