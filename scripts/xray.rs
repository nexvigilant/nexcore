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
    
    // Set NEXCORE_ROOT if not set
    if env::var("NEXCORE_ROOT").is_err() {
        env::set_var("NEXCORE_ROOT", "/home/matthew/Projects/nexcore");
    }

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
