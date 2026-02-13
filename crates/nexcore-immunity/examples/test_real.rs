// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Test loading real antibodies.yaml

use nexcore_immunity::{ImmunityScanner, load_default_registry};

fn main() {
    // Load real registry
    match load_default_registry() {
        Ok(registry) => {
            println!("✓ Loaded {} antibodies from registry", registry.len());

            // Create scanner
            match ImmunityScanner::new(&registry) {
                Ok(scanner) => {
                    println!("✓ Scanner created successfully");

                    // Print stats
                    let stats = scanner.stats();
                    println!("\nAntibody Statistics:");
                    println!("  Total: {}", stats.get("total").copied().unwrap_or(0));
                    println!("  PAMPs: {}", stats.get("pamp").copied().unwrap_or(0));
                    println!("  DAMPs: {}", stats.get("damp").copied().unwrap_or(0));
                    println!(
                        "  Critical: {}",
                        stats.get("critical").copied().unwrap_or(0)
                    );
                    println!("  High: {}", stats.get("high").copied().unwrap_or(0));

                    // Test scanning infected code
                    let infected_code = r#"
fn main() {
    let x = something.unwrap();
    let y = other.expect("should work");
    panic!("oops");
}
"#;
                    let result = scanner.scan(infected_code, Some("test.rs"));
                    println!("\nScan infected code:");
                    println!("  Clean: {}", result.clean);
                    println!("  Threats: {}", result.threats.len());
                    for threat in &result.threats {
                        println!(
                            "    [{:?}] {} at line {:?}: {}",
                            threat.severity,
                            threat.antibody_name,
                            threat.location,
                            threat.matched_content
                        );
                    }

                    // Test scanning clean code
                    let clean_code = r#"
fn main() -> Result<(), Error> {
    let x = something.ok_or(Error::NotFound)?;
    let y = other.context("should work")?;
    Ok(())
}
"#;
                    let result = scanner.scan(clean_code, Some("test.rs"));
                    println!("\nScan clean code:");
                    println!("  Clean: {}", result.clean);
                    println!("  Threats: {}", result.threats.len());
                }
                Err(e) => eprintln!("✗ Failed to create scanner: {}", e),
            }
        }
        Err(e) => eprintln!("✗ Failed to load registry: {}", e),
    }
}
