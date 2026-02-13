// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima MCP Server CLI
//!
//! Run Prima functions as MCP tools.
//!
//! ## Usage
//!
//! ```bash
//! prima-mcp-server path/to/skills.true [prefix]
//! ```

use prima_mcp_server::Server;
use std::fs;
use std::io::{BufReader, stdin, stdout};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: prima-mcp-server <source.true> [prefix]");
        eprintln!();
        eprintln!("Runs Prima functions as MCP tools via stdio.");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  prima-mcp-server skills.true prima_skill");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            std::process::exit(1);
        }
    };

    let prefix = args.get(2).map(|s| s.as_str()).unwrap_or("prima");
    let server = Server::new(&source, prefix);

    let reader = BufReader::new(stdin());
    let writer = stdout();

    if let Err(e) = server.run(reader, writer) {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
