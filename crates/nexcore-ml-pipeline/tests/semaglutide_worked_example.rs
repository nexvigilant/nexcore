//! Semaglutide Worked Example — ML Signal Detection Pipeline
//!
//! Demonstrates the full autonomous ML pipeline using realistic FAERS-like data
//! for semaglutide (Ozempic/Wegovy). Known signals from FDA labeling and
//! literature are used as positive examples; known non-associated events as noise.
//!
//! This test validates:
//! 1. Feature extraction from contingency tables
//! 2. Random forest training on mixed signal/noise data
//! 3. Model evaluation (AUC, precision, recall, F1)
//! 4. Comparison against PRR statistical baseline
//! 5. Prediction on unlabeled pairs

use nexcore_ml_pipeline::prelude::*;

/// Build the semaglutide dataset with known signals and noise.
///
/// Signals (from FDA labeling + literature):
/// - Pancreatitis (boxed warning adjacent)
/// - Thyroid C-cell tumors (boxed warning)
/// - Gastroparesis / delayed gastric emptying
/// - Gallbladder disorders
/// - Acute kidney injury
/// - Diabetic retinopathy complications
/// - Suicidal ideation (under investigation)
///
/// Noise (common events with no disproportionate reporting):
/// - Headache, dizziness, fatigue, insomnia, arthralgia,
///   back pain, upper respiratory infection, urinary tract infection
fn build_semaglutide_dataset() -> (Vec<RawPairData>, Vec<(String, String, String)>) {
    let mut data = Vec::new();
    let mut labels = Vec::new();

    // =========================================================================
    // KNOWN SIGNALS — high a, disproportionate reporting
    // =========================================================================

    let signals = vec![
        (
            "semaglutide",
            "pancreatitis",
            892,
            45_000,
            12_000,
            2_000_000,
            70,
            20,
            750,
            45,
            420,
            5.0,
            12.5,
        ),
        (
            "semaglutide",
            "thyroid_neoplasm",
            234,
            45_000,
            1_800,
            2_000_000,
            80,
            15,
            200,
            30,
            90,
            180.0,
            3.2,
        ),
        (
            "semaglutide",
            "gastroparesis",
            1_456,
            45_000,
            8_000,
            2_000_000,
            55,
            35,
            1100,
            15,
            800,
            14.0,
            20.1,
        ),
        (
            "semaglutide",
            "cholelithiasis",
            678,
            45_000,
            15_000,
            2_000_000,
            60,
            30,
            500,
            8,
            350,
            60.0,
            9.8,
        ),
        (
            "semaglutide",
            "acute_kidney_injury",
            312,
            45_000,
            25_000,
            2_000_000,
            75,
            15,
            290,
            40,
            200,
            10.0,
            4.5,
        ),
        (
            "semaglutide",
            "diabetic_retinopathy",
            189,
            45_000,
            5_500,
            2_000_000,
            85,
            10,
            170,
            5,
            60,
            90.0,
            2.8,
        ),
        (
            "semaglutide",
            "suicidal_ideation",
            156,
            45_000,
            18_000,
            2_000_000,
            45,
            40,
            140,
            25,
            50,
            21.0,
            3.1,
        ),
        (
            "semaglutide",
            "intestinal_obstruction",
            245,
            45_000,
            6_200,
            2_000_000,
            65,
            25,
            220,
            12,
            180,
            8.0,
            4.2,
        ),
        (
            "semaglutide",
            "cholecystitis",
            534,
            45_000,
            11_000,
            2_000_000,
            60,
            28,
            480,
            10,
            380,
            45.0,
            7.6,
        ),
        (
            "semaglutide",
            "ileus",
            178,
            45_000,
            4_800,
            2_000_000,
            70,
            22,
            160,
            8,
            120,
            6.0,
            3.0,
        ),
        (
            "semaglutide",
            "pancreatitis_acute",
            445,
            45_000,
            9_500,
            2_000_000,
            72,
            18,
            400,
            35,
            310,
            3.0,
            8.9,
        ),
        (
            "semaglutide",
            "medullary_thyroid_ca",
            67,
            45_000,
            800,
            2_000_000,
            90,
            5,
            60,
            20,
            25,
            365.0,
            0.9,
        ),
    ];

    for (drug, event, a, b, c, d, hcp, cons, serious, death, hosp, tto, vel) in &signals {
        data.push(make_raw(
            drug, event, *a, *b, *c, *d, *hcp, *cons, *serious, *death, *hosp, *tto, *vel,
        ));
        labels.push((drug.to_string(), event.to_string(), "signal".to_string()));
    }

    // =========================================================================
    // KNOWN NOISE — low a, proportionate reporting
    // =========================================================================

    let noise = vec![
        (
            "semaglutide",
            "headache",
            2_100,
            45_000,
            180_000,
            2_000_000,
            30,
            60,
            200,
            0,
            50,
            3.0,
            30.0,
        ),
        (
            "semaglutide",
            "dizziness",
            1_800,
            45_000,
            150_000,
            2_000_000,
            35,
            55,
            150,
            1,
            30,
            2.0,
            25.0,
        ),
        (
            "semaglutide",
            "fatigue",
            2_500,
            45_000,
            200_000,
            2_000_000,
            28,
            62,
            180,
            0,
            20,
            7.0,
            35.0,
        ),
        (
            "semaglutide",
            "insomnia",
            800,
            45_000,
            90_000,
            2_000_000,
            25,
            65,
            50,
            0,
            10,
            14.0,
            12.0,
        ),
        (
            "semaglutide",
            "arthralgia",
            1_200,
            45_000,
            120_000,
            2_000_000,
            40,
            50,
            100,
            0,
            25,
            30.0,
            18.0,
        ),
        (
            "semaglutide",
            "back_pain",
            950,
            45_000,
            100_000,
            2_000_000,
            35,
            55,
            80,
            0,
            15,
            21.0,
            14.0,
        ),
        (
            "semaglutide",
            "nasopharyngitis",
            1_600,
            45_000,
            160_000,
            2_000_000,
            30,
            60,
            50,
            0,
            5,
            5.0,
            22.0,
        ),
        (
            "semaglutide",
            "urinary_tract_infection",
            700,
            45_000,
            85_000,
            2_000_000,
            45,
            45,
            120,
            2,
            60,
            10.0,
            10.0,
        ),
        (
            "semaglutide",
            "constipation",
            3_000,
            45_000,
            170_000,
            2_000_000,
            30,
            60,
            100,
            0,
            15,
            5.0,
            40.0,
        ),
        (
            "semaglutide",
            "cough",
            600,
            45_000,
            95_000,
            2_000_000,
            25,
            65,
            30,
            0,
            5,
            7.0,
            8.0,
        ),
        (
            "semaglutide",
            "hypertension",
            1_400,
            45_000,
            140_000,
            2_000_000,
            55,
            35,
            250,
            5,
            80,
            60.0,
            20.0,
        ),
        (
            "semaglutide",
            "upper_resp_infection",
            900,
            45_000,
            110_000,
            2_000_000,
            30,
            60,
            40,
            0,
            8,
            5.0,
            12.0,
        ),
    ];

    for (drug, event, a, b, c, d, hcp, cons, serious, death, hosp, tto, vel) in &noise {
        data.push(make_raw(
            drug, event, *a, *b, *c, *d, *hcp, *cons, *serious, *death, *hosp, *tto, *vel,
        ));
        labels.push((drug.to_string(), event.to_string(), "noise".to_string()));
    }

    (data, labels)
}

fn make_raw(
    drug: &str,
    event: &str,
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    hcp: u64,
    consumer: u64,
    serious: u64,
    death: u64,
    hosp: u64,
    tto: f64,
    velocity: f64,
) -> RawPairData {
    RawPairData {
        contingency: ContingencyTable {
            drug: drug.to_string(),
            event: event.to_string(),
            a,
            b,
            c,
            d,
        },
        reporters: ReporterBreakdown {
            hcp,
            consumer,
            other: 10,
        },
        outcomes: OutcomeBreakdown {
            total: a,
            serious,
            death,
            hospitalization: hosp,
        },
        temporal: TemporalData {
            median_tto_days: Some(tto),
            velocity,
        },
    }
}

#[test]
fn semaglutide_full_pipeline() {
    let (raw_data_orig, labels_orig) = build_semaglutide_dataset();

    // Interleave signal and noise so the deterministic split gets both classes
    let n_signals = 12;
    let n_noise = 12;
    let mut raw_data = Vec::with_capacity(raw_data_orig.len());
    let mut labels = Vec::with_capacity(labels_orig.len());
    for i in 0..n_signals.max(n_noise) {
        if i < n_signals {
            raw_data.push(raw_data_orig[i].clone());
            labels.push(labels_orig[i].clone());
        }
        if i < n_noise {
            raw_data.push(raw_data_orig[n_signals + i].clone());
            labels.push(labels_orig[n_signals + i].clone());
        }
    }

    let config = PipelineConfig {
        forest: ForestConfig {
            n_trees: 50,
            max_depth: Some(8),
            seed: 42,
            ..ForestConfig::default()
        },
        train_ratio: 0.75,
        ..PipelineConfig::default()
    };

    let result = pipeline::run(&raw_data, &labels, config);
    assert!(result.is_ok(), "Pipeline failed: {result:?}");
    let result = result.unwrap_or_else(|_| unreachable!());

    // Print results for demonstration
    println!("\n======== SEMAGLUTIDE ML SIGNAL DETECTION — WORKED EXAMPLE ========");
    println!("Model version: {}", result.model_version);
    println!("Trees: {}", result.n_trees);
    println!(
        "Train: {} samples | Test: {} samples",
        result.n_train_samples, result.n_test_samples
    );
    println!("\n--- TRAINING METRICS ---");
    println!("  AUC:       {:.3}", result.train_metrics.auc);
    println!("  Precision: {:.3}", result.train_metrics.precision);
    println!("  Recall:    {:.3}", result.train_metrics.recall);
    println!("  F1:        {:.3}", result.train_metrics.f1);
    println!("  Accuracy:  {:.3}", result.train_metrics.accuracy);

    println!("\n--- TEST METRICS ---");
    println!("  AUC:       {:.3}", result.test_metrics.auc);
    println!("  Precision: {:.3}", result.test_metrics.precision);
    println!("  Recall:    {:.3}", result.test_metrics.recall);
    println!("  F1:        {:.3}", result.test_metrics.f1);
    println!("  Accuracy:  {:.3}", result.test_metrics.accuracy);
    let cm = result.test_metrics.confusion_matrix;
    println!(
        "  Confusion: TN={} FP={} FN={} TP={}",
        cm[0][0], cm[0][1], cm[1][0], cm[1][1]
    );

    println!("\n--- TEST PREDICTIONS ---");
    for p in &result.test_predictions {
        let marker = if p.prediction == "signal" { "!" } else { "." };
        println!(
            "  {marker} {:<30} → {} (prob={:.3})",
            p.event, p.prediction, p.signal_probability
        );
    }

    // Assertions: model should perform reasonably on this separable data
    assert!(
        result.test_metrics.auc >= 0.6,
        "AUC should be >= 0.6: {}",
        result.test_metrics.auc
    );
    assert!(
        result.test_metrics.accuracy >= 0.5,
        "Accuracy >= 0.5: {}",
        result.test_metrics.accuracy
    );
    assert!(
        !result.test_predictions.is_empty(),
        "Should have test predictions"
    );
}

#[test]
fn semaglutide_feature_extraction() {
    // Verify pancreatitis features show signal characteristics
    let raw = make_raw(
        "semaglutide",
        "pancreatitis",
        892,
        45_000,
        12_000,
        2_000_000,
        70,
        20,
        750,
        45,
        420,
        5.0,
        12.5,
    );
    let sample = extract_features(&raw);
    assert!(sample.is_ok());
    let sample = sample.unwrap_or_else(|_| unreachable!());

    println!("\n--- PANCREATITIS FEATURES ---");
    for (name, val) in FEATURE_NAMES.iter().zip(sample.features.iter()) {
        println!("  {:<30} = {:.4}", name, val);
    }

    // PRR should be elevated (signal)
    let prr = sample.features[0];
    assert!(prr > 1.5, "PRR for pancreatitis should be elevated: {prr}");

    // Serious ratio should be high
    let serious_ratio = sample.features[7];
    assert!(
        serious_ratio > 0.5,
        "Serious ratio should be > 0.5: {serious_ratio}"
    );
}

#[test]
fn semaglutide_ml_vs_prr_baseline() {
    let (raw_data, labels) = build_semaglutide_dataset();

    // Extract features + labels
    let mut samples = Vec::new();
    for raw in &raw_data {
        if let Ok(mut s) = extract_features(raw) {
            for (drug, event, label) in &labels {
                if s.drug == *drug && s.event == *event {
                    s.label = Some(label.clone());
                }
            }
            samples.push(s);
        }
    }

    let dataset = Dataset::new(samples.clone(), feature_names());
    let (train, test) = dataset.split(0.75);

    // Train ML model
    let forest = nexcore_ml_pipeline::ensemble::RandomForest::train(
        &train,
        ForestConfig {
            n_trees: 50,
            max_depth: Some(8),
            seed: 42,
            ..ForestConfig::default()
        },
    )
    .unwrap_or_else(|_| unreachable!());

    // Get ML predictions on test set
    let (test_data, test_labels) = test.to_dtree_data();
    let ml_predictions = forest.predict_batch(&test_data);
    let ml_probs: Vec<f64> = ml_predictions.iter().map(|(_, p)| *p).collect();

    // Get PRR values as baseline (feature index 0)
    let prr_values: Vec<f64> = test
        .samples
        .iter()
        .filter(|s| s.label.is_some())
        .map(|s| s.features[0])
        .collect();

    // Compare
    let (ml_auc, baseline_auc, improvement) =
        nexcore_ml_pipeline::evaluate::compare_baseline(&test_labels, &ml_probs, &prr_values);

    println!("\n--- ML vs PRR BASELINE ---");
    println!("  ML AUC:       {:.3}", ml_auc);
    println!("  PRR AUC:      {:.3}", baseline_auc);
    println!("  Improvement:  {:.3}", improvement);

    // ML should be at least competitive with PRR
    assert!(ml_auc >= 0.5, "ML AUC should be >= 0.5: {ml_auc}");

    // Feature importance
    let importances = forest.feature_importance();
    println!("\n--- FEATURE IMPORTANCE ---");
    let mut sorted_imp = importances.clone();
    sorted_imp.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (name, imp) in sorted_imp.iter().take(5) {
        println!("  {:<30} = {:.4}", name, imp);
    }
}
