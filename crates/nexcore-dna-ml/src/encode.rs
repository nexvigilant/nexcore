//! DNA feature encoding — transforms ML feature vectors into DNA strands.
//!
//! Each 12-element PV feature vector is quantized to bytes, then encoded
//! as a DNA strand via `nexcore_dna::storage`. This creates a biological
//! representation that supports alignment, similarity, and evolutionary ops.

use nexcore_dna::storage;
use nexcore_dna::types::Strand;

/// Quantize a floating-point feature vector into bytes (0-255).
///
/// Uses min-max scaling per feature across the dataset, then maps [0,1] → [0,255].
pub fn quantize_features(features: &[f64], mins: &[f64], maxs: &[f64]) -> Vec<u8> {
    features
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let min = mins.get(i).copied().unwrap_or(0.0);
            let max = maxs.get(i).copied().unwrap_or(1.0);
            let range = max - min;
            if range.abs() < f64::EPSILON {
                128u8
            } else {
                let normalized = ((v - min) / range).clamp(0.0, 1.0);
                (normalized * 255.0) as u8
            }
        })
        .collect()
}

/// Encode a feature vector as a DNA strand.
///
/// Pipeline: features → quantize → bytes → DNA strand.
pub fn encode_features(features: &[f64], mins: &[f64], maxs: &[f64]) -> Strand {
    let bytes = quantize_features(features, mins, maxs);
    storage::encode(&bytes)
}

/// Decode a DNA strand back to a quantized byte vector.
///
/// Returns the raw bytes — call `dequantize` to recover approximate floats.
pub fn decode_strand(strand: &Strand) -> Result<Vec<u8>, nexcore_dna::error::DnaError> {
    storage::decode(strand)
}

/// Recover approximate floating-point features from quantized bytes.
pub fn dequantize(bytes: &[u8], mins: &[f64], maxs: &[f64]) -> Vec<f64> {
    bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            let min = mins.get(i).copied().unwrap_or(0.0);
            let max = maxs.get(i).copied().unwrap_or(1.0);
            let normalized = b as f64 / 255.0;
            min + normalized * (max - min)
        })
        .collect()
}

/// Compute per-feature min/max from a dataset of feature vectors.
pub fn compute_bounds(dataset: &[Vec<f64>]) -> (Vec<f64>, Vec<f64>) {
    if dataset.is_empty() {
        return (vec![], vec![]);
    }
    let dim = dataset[0].len();
    let mut mins = vec![f64::INFINITY; dim];
    let mut maxs = vec![f64::NEG_INFINITY; dim];

    for sample in dataset {
        for (i, &v) in sample.iter().enumerate() {
            if i < dim {
                if v < mins[i] {
                    mins[i] = v;
                }
                if v > maxs[i] {
                    maxs[i] = v;
                }
            }
        }
    }
    (mins, maxs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encoding() {
        let features = vec![1.5, 3.0, 0.0, 10.0];
        let mins = vec![0.0, 0.0, 0.0, 0.0];
        let maxs = vec![5.0, 5.0, 5.0, 20.0];

        let strand = encode_features(&features, &mins, &maxs);
        let bytes = decode_strand(&strand).unwrap();
        let recovered = dequantize(&bytes, &mins, &maxs);

        // Quantization loses precision — within 1% of range
        for (i, (&orig, &rec)) in features.iter().zip(recovered.iter()).enumerate() {
            let range = maxs[i] - mins[i];
            let err = (orig - rec).abs();
            assert!(
                err < range * 0.01,
                "feature {i}: orig={orig}, recovered={rec}, err={err}"
            );
        }
    }

    #[test]
    fn compute_bounds_works() {
        let data = vec![vec![1.0, 5.0], vec![3.0, 2.0], vec![2.0, 8.0]];
        let (mins, maxs) = compute_bounds(&data);
        assert_eq!(mins, vec![1.0, 2.0]);
        assert_eq!(maxs, vec![3.0, 8.0]);
    }
}
