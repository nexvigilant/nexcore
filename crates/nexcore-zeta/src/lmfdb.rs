//! # LMFDB Connector
//!
//! Parses L-function zero data from the LMFDB (L-functions and Modular Forms
//! DataBase) into [`ZetaZero`] structs for telescope pipeline consumption.
//!
//! Supports multiple LMFDB data formats:
//! - Raw float arrays: `[14.134, 21.022, ...]`
//! - Labeled zero sets: `{"label": "...", "zeros": [...]}`
//! - API response format: `{"data": [{"height": ..., "rank": ...}, ...]}`
//!
//! ## Design
//!
//! This module handles **parsing only** — no HTTP. Network fetching is
//! delegated to the MCP tool layer, keeping this crate dependency-free
//! from async runtimes and HTTP clients.

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::zeros::ZetaZero;

// ── LMFDB Data Types ─────────────────────────────────────────────────────────

/// Metadata for an L-function from LMFDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmfdbLfunction {
    /// LMFDB label (e.g., "1-1-1.1-r0-0-0" for Riemann zeta).
    pub label: String,
    /// Degree of the L-function.
    pub degree: u32,
    /// Conductor.
    pub conductor: u64,
    /// Brief description.
    pub description: String,
}

/// A parsed set of zeros from LMFDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmfdbZeroSet {
    /// Source L-function metadata.
    pub l_function: LmfdbLfunction,
    /// Parsed zeros ready for telescope consumption.
    pub zeros: Vec<ZetaZero>,
    /// Number of zeros that failed to parse (data loss metric).
    pub parse_failures: usize,
    /// Mapping fidelity: zeros.len() / (zeros.len() + parse_failures).
    pub fidelity: f64,
}

/// Catalog of available L-functions for batch processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmfdbCatalog {
    /// Available L-functions with metadata.
    pub entries: Vec<LmfdbLfunction>,
    /// Total number of entries.
    pub count: usize,
}

// ── LMFDB JSON Formats ──────────────────────────────────────────────────────

/// LMFDB API response with array of zero records.
#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<ApiZeroRecord>,
}

/// Single zero record from LMFDB API.
#[derive(Debug, Deserialize)]
struct ApiZeroRecord {
    height: f64,
    #[serde(default)]
    rank: Option<u64>,
}

/// LMFDB labeled zero set format.
#[derive(Debug, Deserialize)]
struct LabeledZeroSet {
    label: String,
    zeros: Vec<f64>,
    #[serde(default)]
    degree: Option<u32>,
    #[serde(default)]
    conductor: Option<u64>,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Parse a JSON string of zero heights into `Vec<ZetaZero>`.
///
/// Accepts multiple formats:
/// 1. Raw array: `[14.134, 21.022, ...]`
/// 2. Labeled: `{"label": "...", "zeros": [...]}`
/// 3. API response: `{"data": [{"height": ..., "rank": ...}, ...]}`
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if the JSON cannot be parsed
/// in any recognized format.
pub fn parse_lmfdb_zeros(json: &str) -> Result<LmfdbZeroSet, ZetaError> {
    // Try format 1: raw float array
    if let Ok(heights) = serde_json::from_str::<Vec<f64>>(json) {
        return Ok(heights_to_zero_set(heights, "unknown", 1, 1));
    }

    // Try format 2: labeled zero set
    if let Ok(labeled) = serde_json::from_str::<LabeledZeroSet>(json) {
        return Ok(heights_to_zero_set(
            labeled.zeros,
            &labeled.label,
            labeled.degree.unwrap_or(1),
            labeled.conductor.unwrap_or(1),
        ));
    }

    // Try format 3: API response
    if let Ok(api) = serde_json::from_str::<ApiResponse>(json) {
        let heights: Vec<f64> = api.data.iter().map(|r| r.height).collect();
        return Ok(heights_to_zero_set(heights, "api-response", 1, 1));
    }

    Err(ZetaError::InvalidParameter(
        "unrecognized LMFDB JSON format".to_string(),
    ))
}

/// Parse LMFDB API response format specifically.
///
/// Expects: `{"data": [{"height": ..., "rank": ...}, ...]}`
pub fn parse_lmfdb_api_response(json: &str) -> Result<LmfdbZeroSet, ZetaError> {
    let api: ApiResponse = serde_json::from_str(json)
        .map_err(|e| ZetaError::InvalidParameter(format!("invalid LMFDB API response: {e}")))?;

    let mut zeros = Vec::with_capacity(api.data.len());
    let mut failures = 0_usize;

    for (i, record) in api.data.iter().enumerate() {
        if record.height.is_finite() && record.height > 0.0 {
            zeros.push(ZetaZero {
                ordinal: record.rank.unwrap_or((i + 1) as u64),
                t: record.height,
                z_value: 0.0,
                on_critical_line: true,
            });
        } else {
            failures += 1;
        }
    }

    let total = zeros.len() + failures;
    let fidelity = if total > 0 {
        zeros.len() as f64 / total as f64
    } else {
        0.0
    };

    Ok(LmfdbZeroSet {
        l_function: LmfdbLfunction {
            label: "api-response".to_string(),
            degree: 1,
            conductor: 1,
            description: "Parsed from LMFDB API response".to_string(),
        },
        zeros,
        parse_failures: failures,
        fidelity,
    })
}

/// Parse a labeled zero set from LMFDB.
///
/// Expects: `{"label": "...", "zeros": [14.134, ...], "degree": 1, "conductor": 1}`
pub fn parse_lmfdb_labeled(json: &str) -> Result<LmfdbZeroSet, ZetaError> {
    let labeled: LabeledZeroSet = serde_json::from_str(json)
        .map_err(|e| ZetaError::InvalidParameter(format!("invalid LMFDB labeled format: {e}")))?;

    Ok(heights_to_zero_set(
        labeled.zeros,
        &labeled.label,
        labeled.degree.unwrap_or(1),
        labeled.conductor.unwrap_or(1),
    ))
}

/// Get embedded Riemann zeta zeros (first 30).
///
/// These are the first 30 non-trivial zeros of the Riemann zeta function,
/// verified to high precision. Useful for testing and validation.
#[must_use]
pub fn embedded_riemann_zeros() -> Vec<ZetaZero> {
    RIEMANN_ZEROS_30
        .iter()
        .enumerate()
        .map(|(i, &t)| ZetaZero {
            ordinal: (i + 1) as u64,
            t,
            z_value: 0.0,
            on_critical_line: true,
        })
        .collect()
}

/// Get embedded Riemann zeta zeros up to a specified count.
///
/// Returns min(n, 30) zeros from the embedded table.
#[must_use]
pub fn embedded_riemann_zeros_n(n: usize) -> Vec<ZetaZero> {
    let count = n.min(RIEMANN_ZEROS_30.len());
    RIEMANN_ZEROS_30[..count]
        .iter()
        .enumerate()
        .map(|(i, &t)| ZetaZero {
            ordinal: (i + 1) as u64,
            t,
            z_value: 0.0,
            on_critical_line: true,
        })
        .collect()
}

/// Build an LMFDB catalog from a JSON array of L-function metadata.
pub fn parse_lmfdb_catalog(json: &str) -> Result<LmfdbCatalog, ZetaError> {
    let entries: Vec<LmfdbLfunction> = serde_json::from_str(json)
        .map_err(|e| ZetaError::InvalidParameter(format!("invalid LMFDB catalog: {e}")))?;
    let count = entries.len();
    Ok(LmfdbCatalog { entries, count })
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Convert a list of heights to an `LmfdbZeroSet`.
fn heights_to_zero_set(
    heights: Vec<f64>,
    label: &str,
    degree: u32,
    conductor: u64,
) -> LmfdbZeroSet {
    let mut zeros = Vec::with_capacity(heights.len());
    let mut failures = 0_usize;

    for (i, &t) in heights.iter().enumerate() {
        if t.is_finite() && t > 0.0 {
            zeros.push(ZetaZero {
                ordinal: (i + 1) as u64,
                t,
                z_value: 0.0,
                on_critical_line: true,
            });
        } else {
            failures += 1;
        }
    }

    let total = zeros.len() + failures;
    let fidelity = if total > 0 {
        zeros.len() as f64 / total as f64
    } else {
        0.0
    };

    LmfdbZeroSet {
        l_function: LmfdbLfunction {
            label: label.to_string(),
            degree,
            conductor,
            description: format!("L-function {label} (degree {degree}, conductor {conductor})"),
        },
        zeros,
        parse_failures: failures,
        fidelity,
    }
}

// ── Embedded Zero Data ───────────────────────────────────────────────────────

/// First 30 non-trivial zeros of the Riemann zeta function (imaginary parts).
///
/// Source: A. Odlyzko's tables, verified to 9+ decimal places.
const RIEMANN_ZEROS_30: [f64; 30] = [
    14.134_725_142,
    21.022_039_639,
    25.010_857_580,
    30.424_876_126,
    32.935_061_588,
    37.586_178_159,
    40.918_719_012,
    43.327_073_281,
    48.005_150_881,
    49.773_832_478,
    52.970_321_478,
    56.446_247_697,
    59.347_044_003,
    60.831_778_525,
    65.112_544_048,
    67.079_810_529,
    69.546_401_711,
    72.067_157_674,
    75.704_690_699,
    77.144_840_069,
    79.337_375_020,
    82.910_380_854,
    84.735_492_981,
    87.425_274_613,
    88.809_111_208,
    92.491_899_271,
    94.651_344_041,
    95.870_634_228,
    98.831_194_218,
    101.317_851_006,
];

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_raw_array() {
        let json = "[14.134725142, 21.022039639, 25.010857580]";
        let result = parse_lmfdb_zeros(json);
        assert!(result.is_ok());
        let zs = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(zs.zeros.len(), 3);
        assert!((zs.zeros[0].t - 14.134_725_142).abs() < 1e-6);
        assert_eq!(zs.fidelity, 1.0);
    }

    #[test]
    fn parse_labeled_format() {
        let json = r#"{
            "label": "1-1-1.1-r0-0-0",
            "zeros": [14.134725142, 21.022039639],
            "degree": 1,
            "conductor": 1
        }"#;
        let result = parse_lmfdb_zeros(json);
        assert!(result.is_ok());
        let zs = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(zs.l_function.label, "1-1-1.1-r0-0-0");
        assert_eq!(zs.l_function.degree, 1);
        assert_eq!(zs.zeros.len(), 2);
    }

    #[test]
    fn parse_api_response_format() {
        let json = r#"{
            "data": [
                {"height": 14.134725142, "rank": 1},
                {"height": 21.022039639, "rank": 2},
                {"height": 25.010857580, "rank": 3}
            ]
        }"#;
        let result = parse_lmfdb_zeros(json);
        assert!(result.is_ok());
        let zs = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(zs.zeros.len(), 3);
        assert_eq!(zs.zeros[0].ordinal, 1);
        assert_eq!(zs.zeros[2].ordinal, 3);
    }

    #[test]
    fn parse_api_response_with_failures() {
        let json = r#"{
            "data": [
                {"height": 14.134725142, "rank": 1},
                {"height": -1.0, "rank": 2},
                {"height": 25.010857580, "rank": 3}
            ]
        }"#;
        let result = parse_lmfdb_api_response(json);
        assert!(result.is_ok());
        let zs = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(zs.zeros.len(), 2);
        assert_eq!(zs.parse_failures, 1);
        assert!((zs.fidelity - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn embedded_zeros_correct_count() {
        let zeros = embedded_riemann_zeros();
        assert_eq!(zeros.len(), 30);
        assert!((zeros[0].t - 14.134_725_142).abs() < 1e-6);
        assert!((zeros[29].t - 101.317_851_006).abs() < 1e-6);
    }

    #[test]
    fn embedded_zeros_are_sorted() {
        let zeros = embedded_riemann_zeros();
        for w in zeros.windows(2) {
            assert!(
                w[0].t < w[1].t,
                "zeros not sorted: {} >= {}",
                w[0].t,
                w[1].t
            );
        }
    }

    #[test]
    fn embedded_zeros_n_caps() {
        let z5 = embedded_riemann_zeros_n(5);
        assert_eq!(z5.len(), 5);
        let z100 = embedded_riemann_zeros_n(100);
        assert_eq!(z100.len(), 30);
    }

    #[test]
    fn fidelity_is_one_for_clean_data() {
        let json =
            serde_json::to_string(&RIEMANN_ZEROS_30.to_vec()).unwrap_or_else(|_| unreachable!());
        let result = parse_lmfdb_zeros(&json);
        assert!(result.is_ok());
        let zs = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(zs.fidelity, 1.0);
        assert_eq!(zs.parse_failures, 0);
    }

    #[test]
    fn invalid_json_returns_error() {
        let result = parse_lmfdb_zeros("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn catalog_parsing() {
        let json = r#"[
            {"label": "1-1-1.1-r0-0-0", "degree": 1, "conductor": 1, "description": "Riemann zeta"},
            {"label": "2-2-4.1-c1-0-0", "degree": 2, "conductor": 4, "description": "Dirichlet chi_4"}
        ]"#;
        let result = parse_lmfdb_catalog(json);
        assert!(result.is_ok());
        let cat = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(cat.count, 2);
        assert_eq!(cat.entries[0].label, "1-1-1.1-r0-0-0");
    }

    #[test]
    fn zeros_feed_telescope() {
        // Verify embedded zeros can run through the telescope pipeline
        let zeros = embedded_riemann_zeros();
        assert!(zeros.len() >= 20);
        let config = crate::pipeline::TelescopeConfig::default();
        let report = crate::pipeline::run_telescope(&zeros, &config);
        assert!(
            report.is_ok(),
            "telescope failed on embedded zeros: {:?}",
            report.err()
        );
        let r = report.unwrap_or_else(|_| unreachable!());
        assert!(r.overall_rh_confidence > 0.0);
    }
}
