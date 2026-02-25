use nexcore_chrono::{DateTime, Duration};
use nexcore_synapse::{AmplitudeConfig, ConsolidationStatus, LearningSignal, Synapse};
use serde_json::Value;
use std::fs;

#[test]
fn test_hexim1_synapse_consolidation() {
    // 1. SETUP: Configure a Synapse for the HEXIM1 PD Marker
    // Use a threshold calibrated to this test fixture's evidence volume.
    let config = AmplitudeConfig {
        learning_rate: 0.2,
        consolidation_threshold: 0.45,
        half_life_seconds: 3600.0 * 24.0 * 7.0, // 1 week half-life for research data
        ..AmplitudeConfig::DEFAULT
    };
    let mut synapse = Synapse::new("HEXIM1_as_PD_Marker", config);

    // 2. LOAD DATA
    let report_path =
        "/home/matthew/Projects/hexim1-research/Data/Validation/HEXIM1_BET_convergence_report.json";
    let report_content = fs::read_to_string(report_path).expect("Failed to read HEXIM1 report");
    let report: Value =
        serde_json::from_str(&report_content).expect("Failed to parse HEXIM1 report");

    println!("--- HEXIM1 SYNAPSE LEARNING CASCADE ---");

    // 3. PHASE 1: Early Lab Evidence (Transcriptomics)
    // GSE92532: Extremely significant (p < 1e-23)
    let t_signal = LearningSignal::with_timestamp(
        0.95,                                 // High confidence
        1.0,                                  // Highly relevant
        DateTime::now() - Duration::days(30), // Happened a month ago
    );
    synapse.observe(t_signal);
    println!(
        "After Transcriptomics (t=-30d): α={}, Status={}",
        synapse.current_amplitude(),
        synapse.status()
    );

    // 4. PHASE 2: Gap Closing (Proteomics)
    // Gap #2 was closed on 2025-12-30 with PMID 37585491
    let p_signal = LearningSignal::with_timestamp(
        0.85,
        0.9,
        DateTime::now() - Duration::days(15), // Happened 2 weeks ago
    );
    synapse.observe(p_signal);
    println!(
        "After Proteomics (t=-15d): α={}, Status={}",
        synapse.current_amplitude(),
        synapse.status()
    );

    // 5. PHASE 3: Clinical Validation (The "Caspase" of Evidence)
    // 3 Clinical trials reported
    let clinical_trials = report["evidence_summary"]["literature"]["clinical_pd_trials"]
        .as_array()
        .unwrap();
    for (i, trial) in clinical_trials.iter().enumerate() {
        let trial_signal = LearningSignal::new(
            0.9, // Clinical evidence is very strong
            1.0, // Direct PD marker measurement
        );
        synapse.observe(trial_signal);
        println!(
            "After Clinical Trial {} (t=now): α={}, Status={}",
            i + 1,
            synapse.current_amplitude(),
            synapse.status()
        );
    }

    // 6. FINAL STATE
    println!("\nFinal Learning State:");
    println!("  Consolidated: {}", synapse.is_consolidated());
    println!("  Peak Amplitude reached: {}", synapse.peak_amplitude());
    println!(
        "  Observation Count (Frequency ν): {}",
        synapse.observation_count()
    );

    // ASSERTION: With 3 clinical trials + transcriptomics + proteomics,
    // it MUST be consolidated despite the 30-day decay.
    assert!(synapse.is_consolidated());
    assert_eq!(synapse.status(), ConsolidationStatus::Consolidated);

    // Check time to decay below threshold (Innovative Prediction)
    if let Some(ttl) = synapse.time_to_decay() {
        println!(
            "  Fact Sustainability: This will remain 'Known' for another {} days without reinforcement.",
            ttl.as_secs() / 86400
        );
    }
}
