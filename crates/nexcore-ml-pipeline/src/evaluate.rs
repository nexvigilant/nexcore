//! Model evaluation metrics: AUC, precision, recall, F1, confusion matrix.
//!
//! ## Primitive Foundation
//! - T1: Quantity (N) — counts of TP, FP, TN, FN
//! - T1: Comparison (κ) — predicted vs actual
//! - T1: Mapping (μ) — threshold → binary prediction → metric

use crate::types::Metrics;

/// Compute all classification metrics from labels, predictions, and probabilities.
///
/// Positive class is "signal", negative class is "noise".
#[must_use]
pub fn compute_metrics(
    true_labels: &[String],
    predicted_labels: &[String],
    signal_probabilities: &[f64],
) -> Metrics {
    let cm = confusion_matrix(true_labels, predicted_labels);
    let tp = cm[1][1] as f64;
    let fp = cm[0][1] as f64;
    let tn = cm[0][0] as f64;
    let r#fn = cm[1][0] as f64;

    let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };

    let recall = if tp + r#fn > 0.0 {
        tp / (tp + r#fn)
    } else {
        0.0
    };

    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    let accuracy = if tp + fp + tn + r#fn > 0.0 {
        (tp + tn) / (tp + fp + tn + r#fn)
    } else {
        0.0
    };

    let auc = compute_auc(true_labels, signal_probabilities);

    Metrics {
        auc,
        precision,
        recall,
        f1,
        accuracy,
        confusion_matrix: cm,
    }
}

/// Compute confusion matrix [[TN, FP], [FN, TP]].
/// Positive class = "signal".
#[must_use]
pub fn confusion_matrix(true_labels: &[String], predicted_labels: &[String]) -> [[u64; 2]; 2] {
    let mut cm = [[0u64; 2]; 2];

    for (truth, pred) in true_labels.iter().zip(predicted_labels.iter()) {
        let actual_positive = truth == "signal";
        let pred_positive = pred == "signal";

        match (actual_positive, pred_positive) {
            (false, false) => cm[0][0] += 1, // TN
            (false, true) => cm[0][1] += 1,  // FP
            (true, false) => cm[1][0] += 1,  // FN
            (true, true) => cm[1][1] += 1,   // TP
        }
    }

    cm
}

/// Compute AUC-ROC via trapezoidal rule.
///
/// Sorts by descending probability, sweeps threshold, computes TPR/FPR pairs.
#[must_use]
pub fn compute_auc(true_labels: &[String], probabilities: &[f64]) -> f64 {
    if true_labels.is_empty() || probabilities.is_empty() {
        return 0.0;
    }

    // Pair (probability, is_positive) and sort descending by probability
    let mut pairs: Vec<(f64, bool)> = true_labels
        .iter()
        .zip(probabilities.iter())
        .map(|(label, &prob)| (prob, label == "signal"))
        .collect();
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let total_positive = pairs.iter().filter(|(_, is_pos)| *is_pos).count() as f64;
    let total_negative = pairs.len() as f64 - total_positive;

    if total_positive < 1.0 || total_negative < 1.0 {
        return 0.5; // undefined — return chance level
    }

    // Sweep threshold from high to low, accumulating TPR/FPR
    let mut tp = 0.0_f64;
    let mut fp = 0.0_f64;
    let mut auc = 0.0_f64;
    let mut prev_tpr = 0.0_f64;
    let mut prev_fpr = 0.0_f64;

    for (_, is_positive) in &pairs {
        if *is_positive {
            tp += 1.0;
        } else {
            fp += 1.0;
        }

        let tpr = tp / total_positive;
        let fpr = fp / total_negative;

        // Trapezoidal area
        auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2.0;

        prev_tpr = tpr;
        prev_fpr = fpr;
    }

    auc
}

/// Compare ML model metrics against a statistical baseline (PRR threshold).
///
/// Returns (ml_auc, baseline_auc, improvement).
#[must_use]
pub fn compare_baseline(
    true_labels: &[String],
    ml_probabilities: &[f64],
    prr_values: &[f64],
) -> (f64, f64, f64) {
    let ml_auc = compute_auc(true_labels, ml_probabilities);
    let baseline_auc = compute_auc(true_labels, prr_values);
    let improvement = ml_auc - baseline_auc;
    (ml_auc, baseline_auc, improvement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_classifier_metrics() {
        let truth = vec![
            "signal".into(),
            "signal".into(),
            "noise".into(),
            "noise".into(),
        ];
        let preds = vec![
            "signal".into(),
            "signal".into(),
            "noise".into(),
            "noise".into(),
        ];
        let probs = vec![0.9, 0.8, 0.1, 0.2];

        let m = compute_metrics(&truth, &preds, &probs);
        assert!((m.accuracy - 1.0).abs() < f64::EPSILON);
        assert!((m.precision - 1.0).abs() < f64::EPSILON);
        assert!((m.recall - 1.0).abs() < f64::EPSILON);
        assert!((m.f1 - 1.0).abs() < f64::EPSILON);
        assert_eq!(m.confusion_matrix, [[2, 0], [0, 2]]);
    }

    #[test]
    fn auc_perfect_separation() {
        let truth: Vec<String> = vec![
            "signal".into(),
            "signal".into(),
            "noise".into(),
            "noise".into(),
        ];
        let probs = vec![0.9, 0.8, 0.2, 0.1];
        let auc = compute_auc(&truth, &probs);
        assert!(
            (auc - 1.0).abs() < f64::EPSILON,
            "Perfect separation should give AUC=1.0, got {auc}"
        );
    }

    #[test]
    fn auc_random_classifier() {
        // Alternating labels with uniform probabilities → AUC ≈ 0.5
        let truth: Vec<String> = (0..100)
            .map(|i| if i % 2 == 0 { "signal" } else { "noise" }.into())
            .collect();
        let probs: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
        let auc = compute_auc(&truth, &probs);
        assert!(
            (auc - 0.5).abs() < 0.15,
            "Random-ish classifier should have AUC near 0.5, got {auc}"
        );
    }

    #[test]
    fn confusion_matrix_basic() {
        let truth = vec![
            "signal".into(),
            "noise".into(),
            "signal".into(),
            "noise".into(),
        ];
        let preds = vec![
            "signal".into(),
            "signal".into(), // FP
            "noise".into(),  // FN
            "noise".into(),
        ];
        let cm = confusion_matrix(&truth, &preds);
        assert_eq!(cm[0][0], 1); // TN
        assert_eq!(cm[0][1], 1); // FP
        assert_eq!(cm[1][0], 1); // FN
        assert_eq!(cm[1][1], 1); // TP
    }

    #[test]
    fn f1_handles_zero_division() {
        let truth = vec!["noise".into(), "noise".into()];
        let preds = vec!["noise".into(), "noise".into()];
        let probs = vec![0.1, 0.2];
        let m = compute_metrics(&truth, &preds, &probs);
        // No positive predictions or labels → precision/recall/F1 = 0
        assert!((m.precision - 0.0).abs() < f64::EPSILON);
        assert!((m.recall - 0.0).abs() < f64::EPSILON);
        assert!((m.f1 - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn baseline_comparison() {
        let truth: Vec<String> = vec![
            "signal".into(),
            "signal".into(),
            "noise".into(),
            "noise".into(),
        ];
        let ml_probs = vec![0.95, 0.85, 0.1, 0.05];
        let prr_vals = vec![5.0, 4.0, 0.8, 0.5]; // Using PRR as probability proxy
        let (ml_auc, _, _) = compare_baseline(&truth, &ml_probs, &prr_vals);
        assert!(ml_auc > 0.9, "ML AUC should be high: {ml_auc}");
    }
}
