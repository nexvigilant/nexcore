//! Re-scan skills and agents into the live skills.db.
//!
//! Usage: cargo run -p nexcore-registry --example rescan

fn main() {
    let pool = match nexcore_registry::pool::RegistryPool::open_default() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to open registry: {e}");
            std::process::exit(1);
        }
    };

    let result = pool.with_conn(|conn| {
        // Clear stale rows before full re-scan
        nexcore_registry::skills::delete_all(conn)?;
        nexcore_registry::agents::delete_all(conn)?;

        let scan = nexcore_registry::scanner::scan_all(conn)?;
        println!(
            "Scanned {} skills, {} agents",
            scan.skills_scanned, scan.agents_scanned
        );
        if !scan.errors.is_empty() {
            for err in &scan.errors {
                eprintln!("  Warning: {err}");
            }
        }

        // Compute KPIs after scan
        let kpis = nexcore_registry::kpi::compute_all_kpis(conn)?;
        println!("\nKPIs:");
        for kpi in &kpis {
            println!(
                "  {}: {} {}",
                kpi.name,
                kpi.current_value
                    .map(|v| format!("{v}"))
                    .unwrap_or_else(|| "null".to_string()),
                kpi.unit.as_deref().unwrap_or("")
            );
        }

        Ok(())
    });

    if let Err(e) = result {
        eprintln!("Scan failed: {e}");
        std::process::exit(1);
    }
}
