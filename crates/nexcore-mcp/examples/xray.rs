use nexcore_mcp::params::CrateXrayParams;
use nexcore_mcp::tools::crate_xray;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: xray <crate_name>");
        std::process::exit(1);
    }
    let crate_name = &args[1];

    let params = CrateXrayParams {
        crate_name: crate_name.clone(),
        include_stats: Some(true),
    };

    match crate_xray::xray(params) {
        Ok(result) => {
            for content in result.content {
                if let Some(text) = content.as_text() {
                    println!("{}", text.text);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
