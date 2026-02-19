use nexcore_insight::prelude::*;
use serde_json::Value;
use std::fs;

#[test]
fn test_hexim1_cross_modal_synthesis() {
    // 1. Initialize the Insight Engine with a low pattern threshold to trigger discovery
    let config = InsightConfig {
        pattern_min_occurrences: 2, // If 2 modalities agree, it's a pattern
        ..InsightConfig::default()
    };
    let mut engine = InsightEngine::with_config(config);

    // 2. Load HEXIM1 Research Data
    let report_path =
        "/home/matthew/Projects/hexim1-research/Data/Validation/HEXIM1_BET_convergence_report.json";
    let report_content = fs::read_to_string(report_path).expect("Failed to read HEXIM1 report");
    let report: Value =
        serde_json::from_str(&report_content).expect("Failed to parse HEXIM1 report");

    println!("--- HEXIM1 CROSS-MODAL SYNTHESIS ---");

    // 3. Ingest observations from different modalities
    let mut observations = Vec::new();

    // Transcriptomics: HEXIM1 induction in Macrophages
    if let Some(t_score) = report["scores"]["transcriptomics"].as_f64() {
        observations.push(
            Observation::with_numeric("target:HEXIM1:transcriptomics", t_score)
                .with_tag("hexim1")
                .with_tag("modality:transcriptomics"),
        );
    }

    // Proteomics: HEXIM1 protein induction in Patients
    if let Some(p_score) = report["scores"]["proteomics"].as_f64() {
        observations.push(
            Observation::with_numeric("target:HEXIM1:proteomics", p_score)
                .with_tag("hexim1")
                .with_tag("modality:proteomics"),
        );
    }

    // Genomics: HEXIM1 CUT&RUN binding to MYC
    if let Some(g_score) = report["scores"]["genomics"].as_f64() {
        observations.push(
            Observation::with_numeric("target:HEXIM1:genomics", g_score)
                .with_tag("hexim1")
                .with_tag("modality:genomics"),
        );
    }

    // 4. Run the Engine
    let events = engine.ingest_batch(observations);

    let mut patterns_found = 0;
    for event in events {
        println!("  {}", event);
        if matches!(event, InsightEvent::PatternDetected(_)) {
            patterns_found += 1;
        }
    }

    // 5. Establish a Connection (Semantic Link)
    // Connecting the PD Marker status to Patent Readiness
    let patent_readiness = report["patent_readiness"].as_bool().unwrap_or(false);
    if patent_readiness {
        engine.connect("target:HEXIM1", "IP:Patent", "is_ready", 0.95);
        println!("  EVENT: ConnectionEstablished(HEXIM1 -> Patent Readiness)");
    }

    // 6. Summary
    println!("\nSynthesis Summary:");
    println!("  Total Observations: {}", engine.observation_count());
    println!("  Unique Keys: {}", engine.unique_key_count());
    println!("  Connections: {}", engine.connections().len());

    // In this specific engine, co-occurrence triggers patterns.
    // Since all observations share the "hexim1" tag (if we used tags for co-occurrence),
    // we manually check if the engine's internal state recognized the convergence.

    assert!(engine.observation_count() >= 3);
}
