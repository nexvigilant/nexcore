//! AE Signal Overlay — Phase 4 "Nervous System"
//!
//! Provides adverse-event signal overlay computation for VDAG drug-class graphs.
//! The module transforms raw pharmacovigilance `SignalScore` data attached to
//! `VdagNode` instances into ready-to-render visualisation primitives:
//!
//! - **Heatmap generation** — drug x AE grid with per-cell normalised scores,
//!   hex colors, and opacity values derived from configurable color scales.
//! - **Signal normalisation** — four methods: min-max, z-score, percentile, log.
//! - **Color scale mapping** — linear interpolation across multi-stop diverging
//!   color scales (green -> yellow -> red by default).
//! - **Timeline data** — longitudinal signal evolution for a drug-AE pair with
//!   automatic trend detection via linear regression.
//! - **Ranking** — per-drug AE ranking by any supported score field (PRR, ROR,
//!   IC025, EBGM).
//!
//! ## NexVigilant Signal Thresholds
//!
//! Standard thresholds applied when marking cells as `is_signal`:
//!
//! | Statistic | Threshold |
//! |-----------|-----------|
//! | PRR       | >= 2.0    |
//! | ROR       | > 1.0     |
//! | IC025     | > 0.0     |
//! | EBGM      | >= 2.0    |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::ae_overlay::{
//!     compute_heatmap, default_config, ScoreField,
//! };
//! use nexcore_viz::vdag::{SignalScore, VdagNode, VdagNodeType};
//! use std::collections::HashMap;
//!
//! let nodes = vec![VdagNode {
//!     id: "aspirin".into(),
//!     label: "Aspirin".into(),
//!     node_type: VdagNodeType::Drug,
//!     atc_level: None,
//!     signals: vec![SignalScore {
//!         ae_name: "GI Bleeding".into(),
//!         prr: Some(3.2),
//!         ror: Some(3.5),
//!         ic025: Some(0.8),
//!         ebgm: Some(2.9),
//!         case_count: Some(412),
//!         timestamp: Some("2024-01-01".into()),
//!     }],
//!     color: None,
//!     metadata: HashMap::new(),
//! }];
//!
//! let config = default_config();
//! let heatmap = compute_heatmap(&nodes, &config);
//! assert!(heatmap.is_ok());
//! if let Ok(hm) = heatmap {
//!     assert_eq!(hm.drug_ids, vec!["aspirin"]);
//!     assert_eq!(hm.ae_names, vec!["GI Bleeding"]);
//! }
//! ```

use crate::vdag::{SignalScore, VdagNode, VdagNodeType};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Error type
// ============================================================================

/// Error type for AE overlay operations.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayError {
    /// No signals were found across all supplied nodes.
    NoSignals,
    /// A referenced node ID was not found in the graph.
    InvalidNodeId(String),
    /// The input collection was empty.
    EmptyInput,
}

impl fmt::Display for OverlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverlayError::NoSignals => write!(f, "no signals found across supplied nodes"),
            OverlayError::InvalidNodeId(id) => write!(f, "node id not found: {id}"),
            OverlayError::EmptyInput => write!(f, "input collection is empty"),
        }
    }
}

// ============================================================================
// Core data types
// ============================================================================

/// A single cell in an AE heatmap — one drug x one AE intersection.
///
/// `raw_score` is the value extracted directly from `SignalScore` for the
/// configured `ScoreField`. `normalized` is the 0.0..=1.0 form after applying
/// the chosen `NormalizationMethod`. `color` and `opacity` are derived from
/// `normalized` via the `ColorScale`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeatmapCell {
    /// Drug node identifier.
    pub drug_id: String,
    /// Adverse event name.
    pub ae_name: String,
    /// Raw score value (e.g., PRR) for this drug-AE pair.
    pub raw_score: f64,
    /// Normalised score in `[0.0, 1.0]`.
    pub normalized: f64,
    /// Hex color string derived from the color scale (e.g., `"#ff4444"`).
    pub color: String,
    /// Opacity in `[0.0, 1.0]` — driven by the normalised score.
    pub opacity: f64,
    /// Whether this cell exceeds NexVigilant signal thresholds.
    pub is_signal: bool,
}

/// Complete heatmap data for a set of drugs x AEs.
///
/// `cells` contains one entry per (drug, AE) combination where a raw score
/// was available. Missing combinations (drug has no signal for that AE) are
/// omitted from `cells`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AeHeatmap {
    /// Ordered list of drug node IDs forming the column axis.
    pub drug_ids: Vec<String>,
    /// Ordered list of AE names forming the row axis.
    pub ae_names: Vec<String>,
    /// All populated drug x AE cells.
    pub cells: Vec<HeatmapCell>,
    /// Minimum raw score across all cells (before normalisation).
    pub min_score: f64,
    /// Maximum raw score across all cells (before normalisation).
    pub max_score: f64,
}

/// Normalisation method for signal scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizationMethod {
    /// `(v - min) / (max - min)` — maps to `[0.0, 1.0]`.
    MinMax,
    /// `(v - mean) / std_dev` — unbounded; useful for outlier detection.
    ZScore,
    /// Rank-based: `rank / (count - 1)` — uniform `[0.0, 1.0]`.
    Percentile,
    /// `ln(1 + v) / ln(1 + max)` — compresses large-range scores.
    Log,
}

/// A color scale specification built from ordered color stops.
///
/// Stops must be provided in ascending `position` order. At least one stop
/// is required for `interpolate_color` to return a valid color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScale {
    /// Ordered color stops, each at a `position` in `[0.0, 1.0]`.
    pub stops: Vec<ColorStop>,
}

/// A single stop on a `ColorScale`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorStop {
    /// Position on the scale in `[0.0, 1.0]`.
    pub position: f64,
    /// Hex color string (e.g., `"#22c55e"` for green).
    pub color: String,
}

/// A ranked AE entry for a specific drug.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedAe {
    /// Adverse event name.
    pub ae_name: String,
    /// Raw score value for the chosen `ScoreField`.
    pub score: f64,
    /// 1-based rank (1 = highest score).
    pub rank: usize,
    /// Whether this AE exceeds NexVigilant signal thresholds.
    pub is_signal: bool,
    /// Number of spontaneous reports underlying the signal, if available.
    pub case_count: Option<u32>,
}

/// A single point in a signal timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelinePoint {
    /// ISO 8601 timestamp string (e.g., `"2024-Q1"` or `"2024-01-15"`).
    pub timestamp: String,
    /// Score value at this point.
    pub score: f64,
    /// Whether this point exceeds NexVigilant signal thresholds.
    pub is_signal: bool,
    /// Case count at this point, if available.
    pub case_count: Option<u32>,
}

/// Signal timeline for a drug-AE pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalTimeline {
    /// Drug node identifier.
    pub drug_id: String,
    /// Adverse event name.
    pub ae_name: String,
    /// Time-ordered sequence of score points.
    pub points: Vec<TimelinePoint>,
    /// Overall trend direction derived from the points sequence.
    pub trend: Trend,
}

/// Trend direction derived from a sequence of `TimelinePoint` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    /// Linear regression slope > 0.1 — signal is strengthening.
    Rising,
    /// Linear regression slope in `[-0.1, 0.1]` — signal is stable.
    Stable,
    /// Linear regression slope < -0.1 — signal is weakening.
    Declining,
    /// Fewer than 2 data points — trend cannot be determined.
    Insufficient,
}

/// Which score field to use for overlay visualisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreField {
    /// Proportional Reporting Ratio.
    Prr,
    /// Reporting Odds Ratio.
    Ror,
    /// IC lower 95% credible interval.
    Ic025,
    /// Empirical Bayes Geometric Mean.
    Ebgm,
}

/// Overlay configuration controlling how signals are visualised.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    /// Which disproportionality statistic to use as the primary score.
    pub score_field: ScoreField,
    /// How raw scores are normalised before color mapping.
    pub normalization: NormalizationMethod,
    /// Color scale used to map normalised scores to hex colors.
    pub color_scale: ColorScale,
    /// Score value above which a cell is considered a signal.
    /// Applied to the raw (pre-normalisation) score.
    pub signal_threshold: f64,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        default_config()
    }
}

// ============================================================================
// Default constructors
// ============================================================================

/// Build the default overlay configuration.
///
/// Defaults:
/// - Score field: PRR
/// - Normalisation: min-max
/// - Color scale: green (safe) -> yellow (warning) -> red (signal)
/// - Signal threshold: 2.0 (NexVigilant PRR threshold)
///
/// # Example
///
/// ```rust
/// use nexcore_viz::ae_overlay::{default_config, NormalizationMethod, ScoreField};
///
/// let config = default_config();
/// assert_eq!(config.score_field, ScoreField::Prr);
/// assert_eq!(config.normalization, NormalizationMethod::MinMax);
/// assert!((config.signal_threshold - 2.0).abs() < f64::EPSILON);
/// ```
#[must_use]
pub fn default_config() -> OverlayConfig {
    OverlayConfig {
        score_field: ScoreField::Prr,
        normalization: NormalizationMethod::MinMax,
        color_scale: default_color_scale(),
        signal_threshold: 2.0,
    }
}

/// Build the default diverging color scale.
///
/// Three stops: green at 0.0, yellow at 0.5, red at 1.0.
/// This produces a traffic-light style palette where low scores are
/// visually safe (green) and high scores flag danger (red).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::ae_overlay::default_color_scale;
///
/// let scale = default_color_scale();
/// assert_eq!(scale.stops.len(), 3);
/// assert!((scale.stops[0].position - 0.0).abs() < f64::EPSILON);
/// assert!((scale.stops[1].position - 0.5).abs() < f64::EPSILON);
/// assert!((scale.stops[2].position - 1.0).abs() < f64::EPSILON);
/// ```
#[must_use]
pub fn default_color_scale() -> ColorScale {
    ColorScale {
        stops: vec![
            ColorStop {
                position: 0.0,
                color: "#22c55e".into(),
            }, // green-500
            ColorStop {
                position: 0.5,
                color: "#eab308".into(),
            }, // yellow-500
            ColorStop {
                position: 1.0,
                color: "#ef4444".into(),
            }, // red-500
        ],
    }
}

// ============================================================================
// Score extraction helper
// ============================================================================

/// Extract the configured score field from a `SignalScore`.
///
/// Returns `None` when the chosen field is absent on the record.
fn extract_score(signal: &SignalScore, field: ScoreField) -> Option<f64> {
    match field {
        ScoreField::Prr => signal.prr,
        ScoreField::Ror => signal.ror,
        ScoreField::Ic025 => signal.ic025,
        ScoreField::Ebgm => signal.ebgm,
    }
}

// ============================================================================
// Normalisation
// ============================================================================

/// Normalise a slice of scores using the specified method.
///
/// All edge cases are handled without panicking:
/// - Empty input returns an empty `Vec`.
/// - All values identical returns all-zero outputs (MinMax, Log, Percentile)
///   or all-zero outputs (ZScore, since std dev is zero).
/// - Single value returns a single `0.0`.
///
/// # Examples
///
/// ```rust
/// use nexcore_viz::ae_overlay::{normalize_scores, NormalizationMethod};
///
/// let values = vec![1.0, 2.0, 3.0, 4.0];
/// let normed = normalize_scores(&values, NormalizationMethod::MinMax);
/// assert!((normed[0] - 0.0).abs() < 1e-9);
/// assert!((normed[3] - 1.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn normalize_scores(values: &[f64], method: NormalizationMethod) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }

    match method {
        NormalizationMethod::MinMax => normalize_min_max(values),
        NormalizationMethod::ZScore => normalize_zscore(values),
        NormalizationMethod::Percentile => normalize_percentile(values),
        NormalizationMethod::Log => normalize_log(values),
    }
}

fn normalize_min_max(values: &[f64]) -> Vec<f64> {
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    if range < f64::EPSILON {
        return vec![0.0; values.len()];
    }
    values.iter().map(|&v| (v - min) / range).collect()
}

fn normalize_zscore(values: &[f64]) -> Vec<f64> {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();
    if std_dev < f64::EPSILON {
        return vec![0.0; values.len()];
    }
    values.iter().map(|&v| (v - mean) / std_dev).collect()
}

fn normalize_percentile(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    if n == 1 {
        return vec![0.0];
    }
    // Build a sorted index: pair (value, original_idx)
    let mut indexed: Vec<(f64, usize)> = values
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut result = vec![0.0_f64; n];
    let denom = (n - 1) as f64;
    for (rank, &(_, orig_idx)) in indexed.iter().enumerate() {
        result[orig_idx] = rank as f64 / denom;
    }
    result
}

fn normalize_log(values: &[f64]) -> Vec<f64> {
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let log_max = (1.0_f64 + max).ln();
    if log_max < f64::EPSILON {
        return vec![0.0; values.len()];
    }
    values
        .iter()
        .map(|&v| (1.0_f64 + v).ln() / log_max)
        .collect()
}

// ============================================================================
// Color interpolation
// ============================================================================

/// Interpolate a hex color from the scale at position `t`.
///
/// `t` is clamped to `[0.0, 1.0]`. The function performs linear RGB
/// interpolation between the two stops that bracket `t`. If the scale has no
/// stops, `"#000000"` is returned as a safe fallback.
///
/// # Examples
///
/// ```rust
/// use nexcore_viz::ae_overlay::{default_color_scale, interpolate_color};
///
/// let scale = default_color_scale();
/// // At t=0.0, returns the first stop color (green)
/// assert_eq!(interpolate_color(&scale, 0.0), "#22c55e");
/// // At t=1.0, returns the last stop color (red)
/// assert_eq!(interpolate_color(&scale, 1.0), "#ef4444");
/// ```
#[must_use]
pub fn interpolate_color(scale: &ColorScale, t: f64) -> String {
    let t = t.clamp(0.0, 1.0);

    if scale.stops.is_empty() {
        return "#000000".into();
    }
    if scale.stops.len() == 1 {
        return scale.stops[0].color.clone();
    }

    let last = scale.stops.len() - 1;

    // Clamp to boundary stops
    if t <= scale.stops[0].position {
        return scale.stops[0].color.clone();
    }
    if t >= scale.stops[last].position {
        return scale.stops[last].color.clone();
    }

    // Find the pair of stops that bracket t
    let mut lower_idx = 0usize;
    for (i, stop) in scale.stops.iter().enumerate() {
        if stop.position <= t {
            lower_idx = i;
        }
    }
    let upper_idx = (lower_idx + 1).min(last);

    let lo = &scale.stops[lower_idx];
    let hi = &scale.stops[upper_idx];

    let span = hi.position - lo.position;
    let local_t = if span < f64::EPSILON {
        0.0
    } else {
        (t - lo.position) / span
    };

    let (lr, lg, lb) = parse_hex_color(&lo.color);
    let (hr, hg, hb) = parse_hex_color(&hi.color);

    let r = lerp_channel(lr, hr, local_t);
    let g = lerp_channel(lg, hg, local_t);
    let b = lerp_channel(lb, hb, local_t);

    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Parse a 6-digit hex color string into (R, G, B) `u8` components.
///
/// Falls back to `(0, 0, 0)` on any parse failure (malformed input). The
/// leading `#` is optional.
fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return (0, 0, 0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}

/// Linearly interpolate between two `u8` channel values.
fn lerp_channel(a: u8, b: u8, t: f64) -> u8 {
    let af = a as f64;
    let bf = b as f64;
    (af + (bf - af) * t).round().clamp(0.0, 255.0) as u8
}

// ============================================================================
// Heatmap computation
// ============================================================================

/// Compute a full AE heatmap from a set of VDAG nodes.
///
/// Extracts all `Drug` and `DrugClass` nodes from `nodes`, collects the
/// union of all AE names, then builds one `HeatmapCell` per (drug, AE) pair
/// where a raw score is available. Cells with no data for a given AE are
/// omitted.
///
/// Scores are normalised across all cells (not per-drug), so colors are
/// comparable across the entire heatmap. Opacity tracks the normalised score
/// directly: fully transparent at 0.0, fully opaque at 1.0.
///
/// # Errors
///
/// - `OverlayError::EmptyInput` — `nodes` is empty.
/// - `OverlayError::NoSignals` — no drug/drugclass node has any signal with
///   a value for the configured `ScoreField`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::ae_overlay::{compute_heatmap, default_config};
/// use nexcore_viz::vdag::{SignalScore, VdagNode, VdagNodeType};
/// use std::collections::HashMap;
///
/// let nodes = vec![VdagNode {
///     id: "metformin".into(), label: "Metformin".into(),
///     node_type: VdagNodeType::Drug, atc_level: None,
///     signals: vec![SignalScore {
///         ae_name: "Lactic Acidosis".into(), prr: Some(4.1),
///         ror: None, ic025: None, ebgm: None,
///         case_count: None, timestamp: None,
///     }],
///     color: None, metadata: HashMap::new(),
/// }];
///
/// let heatmap = compute_heatmap(&nodes, &default_config());
/// assert!(heatmap.is_ok());
/// ```
pub fn compute_heatmap(
    nodes: &[VdagNode],
    config: &OverlayConfig,
) -> Result<AeHeatmap, OverlayError> {
    if nodes.is_empty() {
        return Err(OverlayError::EmptyInput);
    }

    // Collect drug and drug-class nodes only
    let drug_nodes: Vec<&VdagNode> = nodes
        .iter()
        .filter(|n| matches!(n.node_type, VdagNodeType::Drug | VdagNodeType::DrugClass))
        .collect();

    if drug_nodes.is_empty() {
        return Err(OverlayError::NoSignals);
    }

    // Collect unique AE names in stable insertion order
    let mut ae_names: Vec<String> = Vec::new();
    for node in &drug_nodes {
        for signal in &node.signals {
            if !ae_names.contains(&signal.ae_name) {
                ae_names.push(signal.ae_name.clone());
            }
        }
    }

    if ae_names.is_empty() {
        return Err(OverlayError::NoSignals);
    }

    let drug_ids: Vec<String> = drug_nodes.iter().map(|n| n.id.clone()).collect();

    // Extract raw scores for every (drug, ae) combination that has data.
    // Layout: (drug_id, ae_name, raw_score, is_signal, case_count)
    let mut raw_entries: Vec<(String, String, f64, bool, Option<u32>)> = Vec::new();

    for node in &drug_nodes {
        for signal in &node.signals {
            if let Some(score) = extract_score(signal, config.score_field) {
                let is_sig =
                    signal_exceeds_threshold(signal, config.score_field, config.signal_threshold);
                raw_entries.push((
                    node.id.clone(),
                    signal.ae_name.clone(),
                    score,
                    is_sig,
                    signal.case_count,
                ));
            }
        }
    }

    if raw_entries.is_empty() {
        return Err(OverlayError::NoSignals);
    }

    // Compute min / max for reporting
    let min_score = raw_entries
        .iter()
        .map(|&(_, _, s, _, _)| s)
        .fold(f64::INFINITY, f64::min);
    let max_score = raw_entries
        .iter()
        .map(|&(_, _, s, _, _)| s)
        .fold(f64::NEG_INFINITY, f64::max);

    // Normalise all raw scores together
    let raw_scores: Vec<f64> = raw_entries.iter().map(|&(_, _, s, _, _)| s).collect();
    let normalised = normalize_scores(&raw_scores, config.normalization);

    // Build cells
    let cells: Vec<HeatmapCell> = raw_entries
        .iter()
        .zip(normalised.iter())
        .map(
            |((drug_id, ae_name, raw_score, is_signal, _case_count), &norm)| {
                // Clamp to [0,1] for color/opacity (ZScore can exceed this range)
                let display_norm = norm.clamp(0.0, 1.0);
                let color = interpolate_color(&config.color_scale, display_norm);
                HeatmapCell {
                    drug_id: drug_id.clone(),
                    ae_name: ae_name.clone(),
                    raw_score: *raw_score,
                    normalized: norm,
                    color,
                    opacity: display_norm,
                    is_signal: *is_signal,
                }
            },
        )
        .collect();

    Ok(AeHeatmap {
        drug_ids,
        ae_names,
        cells,
        min_score,
        max_score,
    })
}

/// Return `true` when `signal` exceeds `threshold` for the chosen `field`.
///
/// When the field is absent on the signal, the cell is not considered a signal.
fn signal_exceeds_threshold(signal: &SignalScore, field: ScoreField, threshold: f64) -> bool {
    extract_score(signal, field).is_some_and(|v| v >= threshold)
}

// ============================================================================
// AE ranking
// ============================================================================

/// Rank all AEs for a drug by descending score for the chosen field.
///
/// AEs whose chosen field is `None` are omitted from the result. Ranks are
/// 1-based with 1 being the highest score.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::ae_overlay::{rank_ae_for_drug, ScoreField};
/// use nexcore_viz::vdag::SignalScore;
///
/// let signals = vec![
///     SignalScore { ae_name: "Nausea".into(), prr: Some(1.8), ror: None,
///         ic025: None, ebgm: None, case_count: Some(50), timestamp: None },
///     SignalScore { ae_name: "Hepatotoxicity".into(), prr: Some(4.2), ror: None,
///         ic025: None, ebgm: None, case_count: Some(12), timestamp: None },
/// ];
/// let ranked = rank_ae_for_drug(&signals, ScoreField::Prr);
/// assert_eq!(ranked[0].ae_name, "Hepatotoxicity");
/// assert_eq!(ranked[0].rank, 1);
/// assert_eq!(ranked[1].rank, 2);
/// ```
#[must_use]
pub fn rank_ae_for_drug(signals: &[SignalScore], field: ScoreField) -> Vec<RankedAe> {
    // Collect entries that have a value for the chosen field
    let mut scored: Vec<(&SignalScore, f64)> = signals
        .iter()
        .filter_map(|s| extract_score(s, field).map(|v| (s, v)))
        .collect();

    // Sort descending by score (NaN-safe)
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    scored
        .into_iter()
        .enumerate()
        .map(|(idx, (signal, score))| RankedAe {
            ae_name: signal.ae_name.clone(),
            score,
            rank: idx + 1,
            is_signal: signal.is_signal(),
            case_count: signal.case_count,
        })
        .collect()
}

// ============================================================================
// Signal timeline
// ============================================================================

/// Compute a longitudinal signal timeline for a specific drug-AE pair.
///
/// Filters `signals` to those matching `ae_name` (case-insensitive) and
/// having both a timestamp and a value for `field`. Points are sorted
/// lexicographically by timestamp (ISO 8601 strings sort correctly this way).
/// The `trend` field is computed via linear regression over the score sequence.
///
/// # Errors
///
/// - `OverlayError::EmptyInput` — `signals` slice is empty.
/// - `OverlayError::NoSignals` — no signals matched `ae_name` with valid
///   timestamps and a value for the chosen field.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::ae_overlay::{compute_signal_timeline, ScoreField, Trend};
/// use nexcore_viz::vdag::SignalScore;
///
/// let signals = vec![
///     SignalScore { ae_name: "GI Bleeding".into(), prr: Some(2.1),
///         ror: None, ic025: None, ebgm: None,
///         case_count: Some(10), timestamp: Some("2023-01-01".into()) },
///     SignalScore { ae_name: "GI Bleeding".into(), prr: Some(3.5),
///         ror: None, ic025: None, ebgm: None,
///         case_count: Some(22), timestamp: Some("2024-01-01".into()) },
/// ];
/// let result = compute_signal_timeline(&signals, "drug_x", "GI Bleeding", ScoreField::Prr);
/// assert!(result.is_ok());
/// if let Ok(tl) = result {
///     assert_eq!(tl.points.len(), 2);
///     assert_eq!(tl.trend, Trend::Rising);
/// }
/// ```
pub fn compute_signal_timeline(
    signals: &[SignalScore],
    drug_id: &str,
    ae_name: &str,
    field: ScoreField,
) -> Result<SignalTimeline, OverlayError> {
    if signals.is_empty() {
        return Err(OverlayError::EmptyInput);
    }

    let lower_ae = ae_name.to_lowercase();

    // Filter: matching AE name, has timestamp, has a value for field
    let mut matched: Vec<(&SignalScore, String, f64)> = signals
        .iter()
        .filter(|s| s.ae_name.to_lowercase() == lower_ae)
        .filter_map(|s| {
            let ts = s.timestamp.as_ref()?;
            let score = extract_score(s, field)?;
            Some((s, ts.clone(), score))
        })
        .collect();

    if matched.is_empty() {
        return Err(OverlayError::NoSignals);
    }

    // Sort by timestamp lexicographically (ISO 8601 is lexically ordered)
    matched.sort_by(|a, b| a.1.cmp(&b.1));

    let points: Vec<TimelinePoint> = matched
        .iter()
        .map(|(signal, ts, score)| TimelinePoint {
            timestamp: ts.clone(),
            score: *score,
            is_signal: signal.is_signal(),
            case_count: signal.case_count,
        })
        .collect();

    let trend = compute_trend(&points);

    Ok(SignalTimeline {
        drug_id: drug_id.to_string(),
        ae_name: ae_name.to_string(),
        points,
        trend,
    })
}

// ============================================================================
// Trend computation
// ============================================================================

/// Compute the trend direction from an ordered sequence of timeline points.
///
/// Uses ordinary least squares linear regression over the (index, score) pairs.
/// The slope thresholds are:
/// - `> 0.1` -> `Rising`
/// - `< -0.1` -> `Declining`
/// - Otherwise -> `Stable`
/// - Fewer than 2 points -> `Insufficient`
///
/// # Example
///
/// ```rust
/// use nexcore_viz::ae_overlay::{TimelinePoint, Trend, compute_trend};
///
/// let rising = vec![
///     TimelinePoint { timestamp: "2023".into(), score: 1.0, is_signal: false, case_count: None },
///     TimelinePoint { timestamp: "2024".into(), score: 3.0, is_signal: true,  case_count: None },
/// ];
/// assert_eq!(compute_trend(&rising), Trend::Rising);
///
/// let single = vec![
///     TimelinePoint { timestamp: "2024".into(), score: 2.5, is_signal: true, case_count: None },
/// ];
/// assert_eq!(compute_trend(&single), Trend::Insufficient);
/// ```
#[must_use]
pub fn compute_trend(points: &[TimelinePoint]) -> Trend {
    if points.len() < 2 {
        return Trend::Insufficient;
    }

    let n = points.len() as f64;
    let xs: Vec<f64> = (0..points.len()).map(|i| i as f64).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.score).collect();

    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;

    let numerator: f64 = xs
        .iter()
        .zip(ys.iter())
        .map(|(&x, &y)| (x - mean_x) * (y - mean_y))
        .sum();

    let denominator: f64 = xs.iter().map(|&x| (x - mean_x).powi(2)).sum();

    if denominator < f64::EPSILON {
        // All x values are identical — slope is undefined; report stable
        return Trend::Stable;
    }

    let slope = numerator / denominator;

    if slope > 0.1 {
        Trend::Rising
    } else if slope < -0.1 {
        Trend::Declining
    } else {
        Trend::Stable
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdag::{AtcLevel, SignalScore, VdagNode, VdagNodeType};
    use std::collections::HashMap;

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn make_drug(id: &str, signals: Vec<SignalScore>) -> VdagNode {
        VdagNode {
            id: id.into(),
            label: id.into(),
            node_type: VdagNodeType::Drug,
            atc_level: Some(AtcLevel::Substance),
            signals,
            color: None,
            metadata: HashMap::new(),
        }
    }

    fn make_drug_class(id: &str, signals: Vec<SignalScore>) -> VdagNode {
        VdagNode {
            id: id.into(),
            label: id.into(),
            node_type: VdagNodeType::DrugClass,
            atc_level: Some(AtcLevel::Therapeutic),
            signals,
            color: None,
            metadata: HashMap::new(),
        }
    }

    fn make_ae_node(id: &str) -> VdagNode {
        VdagNode {
            id: id.into(),
            label: id.into(),
            node_type: VdagNodeType::AdverseEvent,
            atc_level: None,
            signals: vec![],
            color: None,
            metadata: HashMap::new(),
        }
    }

    fn signal(ae: &str, prr: f64) -> SignalScore {
        SignalScore {
            ae_name: ae.into(),
            prr: Some(prr),
            ror: Some(prr + 0.3),
            ic025: Some(0.5),
            ebgm: Some(prr - 0.1),
            case_count: Some(50),
            timestamp: None,
        }
    }

    fn signal_with_ts(ae: &str, prr: f64, ts: &str) -> SignalScore {
        SignalScore {
            ae_name: ae.into(),
            prr: Some(prr),
            ror: Some(prr + 0.3),
            ic025: Some(0.5),
            ebgm: Some(prr - 0.1),
            case_count: Some(20),
            timestamp: Some(ts.into()),
        }
    }

    // -----------------------------------------------------------------------
    // default_config / default_color_scale
    // -----------------------------------------------------------------------

    #[test]
    fn default_config_fields_are_correct() {
        let cfg = default_config();
        assert_eq!(cfg.score_field, ScoreField::Prr);
        assert_eq!(cfg.normalization, NormalizationMethod::MinMax);
        assert!((cfg.signal_threshold - 2.0).abs() < f64::EPSILON);
        assert_eq!(cfg.color_scale.stops.len(), 3);
    }

    #[test]
    fn default_color_scale_has_three_stops() {
        let scale = default_color_scale();
        assert_eq!(scale.stops.len(), 3);
    }

    #[test]
    fn default_color_scale_stop_positions() {
        let scale = default_color_scale();
        assert!((scale.stops[0].position - 0.0).abs() < f64::EPSILON);
        assert!((scale.stops[1].position - 0.5).abs() < f64::EPSILON);
        assert!((scale.stops[2].position - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overlay_config_default_trait_matches_default_config() {
        let a = OverlayConfig::default();
        let b = default_config();
        assert_eq!(a.score_field, b.score_field);
        assert_eq!(a.normalization, b.normalization);
        assert!((a.signal_threshold - b.signal_threshold).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // normalize_scores — MinMax
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_minmax_basic() {
        let vals = vec![1.0, 2.0, 3.0, 4.0];
        let normed = normalize_scores(&vals, NormalizationMethod::MinMax);
        assert_eq!(normed.len(), 4);
        assert!((normed[0] - 0.0).abs() < 1e-9);
        assert!((normed[3] - 1.0).abs() < 1e-9);
        assert!((normed[1] - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn normalize_minmax_all_same_returns_zeros() {
        let vals = vec![5.0, 5.0, 5.0];
        let normed = normalize_scores(&vals, NormalizationMethod::MinMax);
        assert!(normed.iter().all(|&v| v.abs() < 1e-9));
    }

    #[test]
    fn normalize_minmax_single_value() {
        let vals = vec![42.0];
        let normed = normalize_scores(&vals, NormalizationMethod::MinMax);
        assert_eq!(normed.len(), 1);
        assert!((normed[0]).abs() < 1e-9);
    }

    #[test]
    fn normalize_minmax_empty_input() {
        let normed = normalize_scores(&[], NormalizationMethod::MinMax);
        assert!(normed.is_empty());
    }

    // -----------------------------------------------------------------------
    // normalize_scores — ZScore
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_zscore_basic() {
        let vals = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let normed = normalize_scores(&vals, NormalizationMethod::ZScore);
        assert_eq!(normed.len(), 8);
        // mean is 5.0; the value at index 4 (5.0) should normalise to ~0.0
        assert!(normed[4].abs() < 1e-9, "z-score of mean should be ~0");
    }

    #[test]
    fn normalize_zscore_all_same() {
        let vals = vec![3.0, 3.0, 3.0];
        let normed = normalize_scores(&vals, NormalizationMethod::ZScore);
        assert!(normed.iter().all(|&v| v.abs() < 1e-9));
    }

    #[test]
    fn normalize_zscore_single_value() {
        let normed = normalize_scores(&[7.0], NormalizationMethod::ZScore);
        assert_eq!(normed.len(), 1);
        assert!((normed[0]).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // normalize_scores — Percentile
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_percentile_ordered_sequence() {
        let vals = vec![10.0, 20.0, 30.0, 40.0];
        let normed = normalize_scores(&vals, NormalizationMethod::Percentile);
        // Ranks 0,1,2,3 -> positions 0/3, 1/3, 2/3, 3/3
        assert!((normed[0] - 0.0).abs() < 1e-9);
        assert!((normed[3] - 1.0).abs() < 1e-9);
        assert!((normed[1] - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn normalize_percentile_single_value() {
        let normed = normalize_scores(&[99.0], NormalizationMethod::Percentile);
        assert_eq!(normed.len(), 1);
        assert!((normed[0]).abs() < 1e-9);
    }

    #[test]
    fn normalize_percentile_preserves_length_and_bounds() {
        let vals = vec![5.0, 1.0, 3.0, 2.0, 4.0];
        let normed = normalize_scores(&vals, NormalizationMethod::Percentile);
        assert_eq!(normed.len(), 5);
        assert!(normed.iter().all(|&v| v >= 0.0 && v <= 1.0 + 1e-9));
    }

    // -----------------------------------------------------------------------
    // normalize_scores — Log
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_log_basic() {
        let vals = vec![0.0, 1.0, 9.0];
        let normed = normalize_scores(&vals, NormalizationMethod::Log);
        assert_eq!(normed.len(), 3);
        // ln(1+0)/ln(1+9) = 0
        assert!((normed[0] - 0.0).abs() < 1e-9);
        // ln(1+9)/ln(1+9) = 1
        assert!((normed[2] - 1.0).abs() < 1e-9);
        // intermediate value is strictly between 0 and 1
        assert!(normed[1] > 0.0 && normed[1] < 1.0);
    }

    #[test]
    fn normalize_log_all_zeros() {
        let vals = vec![0.0, 0.0, 0.0];
        let normed = normalize_scores(&vals, NormalizationMethod::Log);
        assert!(normed.iter().all(|&v| v.abs() < 1e-9));
    }

    // -----------------------------------------------------------------------
    // interpolate_color
    // -----------------------------------------------------------------------

    #[test]
    fn interpolate_color_at_zero_returns_first_stop() {
        let scale = default_color_scale();
        assert_eq!(interpolate_color(&scale, 0.0), "#22c55e");
    }

    #[test]
    fn interpolate_color_at_one_returns_last_stop() {
        let scale = default_color_scale();
        assert_eq!(interpolate_color(&scale, 1.0), "#ef4444");
    }

    #[test]
    fn interpolate_color_at_midpoint_returns_yellow() {
        let scale = default_color_scale();
        assert_eq!(interpolate_color(&scale, 0.5), "#eab308");
    }

    #[test]
    fn interpolate_color_clamps_below_zero() {
        let scale = default_color_scale();
        assert_eq!(
            interpolate_color(&scale, -1.0),
            interpolate_color(&scale, 0.0)
        );
    }

    #[test]
    fn interpolate_color_clamps_above_one() {
        let scale = default_color_scale();
        assert_eq!(
            interpolate_color(&scale, 2.0),
            interpolate_color(&scale, 1.0)
        );
    }

    #[test]
    fn interpolate_color_empty_scale_returns_black() {
        let empty_scale = ColorScale { stops: vec![] };
        assert_eq!(interpolate_color(&empty_scale, 0.5), "#000000");
    }

    #[test]
    fn interpolate_color_single_stop_returns_stop_color() {
        let scale = ColorScale {
            stops: vec![ColorStop {
                position: 0.0,
                color: "#ff0000".into(),
            }],
        };
        assert_eq!(interpolate_color(&scale, 0.5), "#ff0000");
    }

    #[test]
    fn interpolate_color_quarter_is_intermediate() {
        let scale = default_color_scale();
        // t=0.25 -> halfway between green(0.0) and yellow(0.5)
        let color = interpolate_color(&scale, 0.25);
        assert_ne!(color, "#22c55e");
        assert_ne!(color, "#ef4444");
        assert_ne!(color, "#eab308");
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }

    // -----------------------------------------------------------------------
    // compute_heatmap
    // -----------------------------------------------------------------------

    #[test]
    fn compute_heatmap_empty_nodes_returns_error() {
        assert_eq!(
            compute_heatmap(&[], &default_config()),
            Err(OverlayError::EmptyInput)
        );
    }

    #[test]
    fn compute_heatmap_only_ae_nodes_returns_no_signals() {
        let nodes = vec![make_ae_node("ae1"), make_ae_node("ae2")];
        assert_eq!(
            compute_heatmap(&nodes, &default_config()),
            Err(OverlayError::NoSignals)
        );
    }

    #[test]
    fn compute_heatmap_drug_with_no_signals_returns_no_signals() {
        let nodes = vec![make_drug("aspirin", vec![])];
        assert_eq!(
            compute_heatmap(&nodes, &default_config()),
            Err(OverlayError::NoSignals)
        );
    }

    #[test]
    fn compute_heatmap_single_drug_single_ae() {
        let nodes = vec![make_drug("aspirin", vec![signal("GI Bleeding", 3.2)])];
        let result = compute_heatmap(&nodes, &default_config());
        assert!(result.is_ok(), "expected Ok from heatmap computation");
        if let Ok(hm) = result {
            assert_eq!(hm.drug_ids, vec!["aspirin"]);
            assert_eq!(hm.ae_names, vec!["GI Bleeding"]);
            assert_eq!(hm.cells.len(), 1);
        }
    }

    #[test]
    fn compute_heatmap_min_max_scores() {
        let nodes = vec![
            make_drug(
                "drugA",
                vec![signal("Nausea", 1.5), signal("Headache", 4.0)],
            ),
            make_drug("drugB", vec![signal("Nausea", 2.5)]),
        ];
        let result = compute_heatmap(&nodes, &default_config());
        assert!(result.is_ok());
        if let Ok(hm) = result {
            assert!((hm.min_score - 1.5).abs() < 1e-9);
            assert!((hm.max_score - 4.0).abs() < 1e-9);
        }
    }

    #[test]
    fn compute_heatmap_is_signal_flag() {
        let mut high_signal = signal("GI Bleeding", 3.5);
        high_signal.ic025 = Some(0.8);
        high_signal.ebgm = Some(3.0);

        let mut low_signal = signal("Tinnitus", 1.2);
        low_signal.prr = Some(1.2); // below PRR threshold of 2.0

        let nodes = vec![make_drug("aspirin", vec![high_signal, low_signal])];
        let result = compute_heatmap(&nodes, &default_config());
        assert!(result.is_ok());
        if let Ok(hm) = result {
            let gi = hm.cells.iter().find(|c| c.ae_name == "GI Bleeding");
            let tinnitus = hm.cells.iter().find(|c| c.ae_name == "Tinnitus");
            assert!(gi.is_some() && gi.map_or(false, |c| c.is_signal));
            assert!(tinnitus.is_some() && tinnitus.map_or(false, |c| !c.is_signal));
        }
    }

    #[test]
    fn compute_heatmap_drugclass_nodes_included() {
        let nodes = vec![
            make_drug_class("analgesics", vec![signal("GI Bleeding", 2.5)]),
            make_ae_node("ae_gi"),
        ];
        let result = compute_heatmap(&nodes, &default_config());
        assert!(result.is_ok());
        if let Ok(hm) = result {
            assert_eq!(hm.drug_ids, vec!["analgesics"]);
        }
    }

    #[test]
    fn compute_heatmap_cell_colors_are_valid_hex() {
        let nodes = vec![make_drug(
            "d1",
            vec![signal("Nausea", 2.0), signal("Headache", 4.5)],
        )];
        let result = compute_heatmap(&nodes, &default_config());
        assert!(result.is_ok());
        if let Ok(hm) = result {
            for cell in &hm.cells {
                assert!(cell.color.starts_with('#'), "color must start with #");
                assert_eq!(cell.color.len(), 7, "color must be 7 chars");
            }
        }
    }

    #[test]
    fn compute_heatmap_ror_field() {
        let mut s = signal("Nausea", 2.0);
        s.ror = Some(3.5);
        let nodes = vec![make_drug("d1", vec![s])];
        let mut config = default_config();
        config.score_field = ScoreField::Ror;
        let result = compute_heatmap(&nodes, &config);
        assert!(result.is_ok());
        if let Ok(hm) = result {
            assert!((hm.cells[0].raw_score - 3.5).abs() < 1e-9);
        }
    }

    // -----------------------------------------------------------------------
    // rank_ae_for_drug
    // -----------------------------------------------------------------------

    #[test]
    fn rank_ae_for_drug_descending_order() {
        let signals = vec![
            signal("Nausea", 1.8),
            signal("Hepatotoxicity", 4.2),
            signal("Rash", 2.5),
        ];
        let ranked = rank_ae_for_drug(&signals, ScoreField::Prr);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].ae_name, "Hepatotoxicity");
        assert_eq!(ranked[0].rank, 1);
        assert_eq!(ranked[1].ae_name, "Rash");
        assert_eq!(ranked[1].rank, 2);
        assert_eq!(ranked[2].ae_name, "Nausea");
        assert_eq!(ranked[2].rank, 3);
    }

    #[test]
    fn rank_ae_for_drug_omits_missing_field() {
        let signals = vec![
            SignalScore {
                ae_name: "Nausea".into(),
                prr: None, // missing PRR
                ror: Some(2.0),
                ic025: None,
                ebgm: None,
                case_count: None,
                timestamp: None,
            },
            signal("Hepatotoxicity", 3.0),
        ];
        let ranked = rank_ae_for_drug(&signals, ScoreField::Prr);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].ae_name, "Hepatotoxicity");
    }

    #[test]
    fn rank_ae_for_drug_empty_signals() {
        let ranked = rank_ae_for_drug(&[], ScoreField::Prr);
        assert!(ranked.is_empty());
    }

    #[test]
    fn rank_ae_for_drug_is_signal_flag_correct() {
        let mut below = signal("Tinnitus", 1.0);
        below.prr = Some(1.0); // below threshold
        let signals = vec![signal("GI Bleeding", 3.5), below];
        let ranked = rank_ae_for_drug(&signals, ScoreField::Prr);

        let gi = ranked.iter().find(|r| r.ae_name == "GI Bleeding");
        let tinnitus = ranked.iter().find(|r| r.ae_name == "Tinnitus");
        assert!(gi.map_or(false, |r| r.is_signal));
        assert!(tinnitus.map_or(false, |r| !r.is_signal));
    }

    // -----------------------------------------------------------------------
    // compute_trend
    // -----------------------------------------------------------------------

    fn tp(score: f64) -> TimelinePoint {
        TimelinePoint {
            timestamp: String::new(),
            score,
            is_signal: score >= 2.0,
            case_count: None,
        }
    }

    #[test]
    fn compute_trend_insufficient_for_zero_points() {
        assert_eq!(compute_trend(&[]), Trend::Insufficient);
    }

    #[test]
    fn compute_trend_insufficient_for_one_point() {
        assert_eq!(compute_trend(&[tp(2.5)]), Trend::Insufficient);
    }

    #[test]
    fn compute_trend_rising() {
        let pts = vec![tp(1.0), tp(2.0), tp(4.0), tp(8.0)];
        assert_eq!(compute_trend(&pts), Trend::Rising);
    }

    #[test]
    fn compute_trend_declining() {
        let pts = vec![tp(8.0), tp(4.0), tp(2.0), tp(1.0)];
        assert_eq!(compute_trend(&pts), Trend::Declining);
    }

    #[test]
    fn compute_trend_stable_flat() {
        let pts = vec![tp(3.0), tp(3.0), tp(3.0), tp(3.0)];
        assert_eq!(compute_trend(&pts), Trend::Stable);
    }

    #[test]
    fn compute_trend_stable_slight_increase() {
        // slope well below 0.1 threshold
        let pts = vec![tp(2.0), tp(2.05), tp(2.08), tp(2.09)];
        assert_eq!(compute_trend(&pts), Trend::Stable);
    }

    // -----------------------------------------------------------------------
    // compute_signal_timeline
    // -----------------------------------------------------------------------

    #[test]
    fn compute_signal_timeline_empty_input_error() {
        let result = compute_signal_timeline(&[], "drug_x", "Nausea", ScoreField::Prr);
        assert_eq!(result, Err(OverlayError::EmptyInput));
    }

    #[test]
    fn compute_signal_timeline_no_matching_ae() {
        let signals = vec![signal_with_ts("Headache", 2.0, "2024-01-01")];
        let result = compute_signal_timeline(&signals, "drug_x", "Nausea", ScoreField::Prr);
        assert_eq!(result, Err(OverlayError::NoSignals));
    }

    #[test]
    fn compute_signal_timeline_ae_without_timestamp_excluded() {
        // signal() has no timestamp, so it must be excluded
        let signals = vec![signal("Nausea", 2.5)];
        let result = compute_signal_timeline(&signals, "drug_x", "Nausea", ScoreField::Prr);
        assert_eq!(result, Err(OverlayError::NoSignals));
    }

    #[test]
    fn compute_signal_timeline_success_rising() {
        let signals = vec![
            signal_with_ts("GI Bleeding", 2.1, "2023-01-01"),
            signal_with_ts("GI Bleeding", 3.5, "2024-01-01"),
        ];
        let result = compute_signal_timeline(&signals, "drug_x", "GI Bleeding", ScoreField::Prr);
        assert!(result.is_ok(), "expected Ok from signal timeline");
        if let Ok(tl) = result {
            assert_eq!(tl.drug_id, "drug_x");
            assert_eq!(tl.ae_name, "GI Bleeding");
            assert_eq!(tl.points.len(), 2);
            assert_eq!(tl.trend, Trend::Rising);
        }
    }

    #[test]
    fn compute_signal_timeline_sorts_by_timestamp() {
        let signals = vec![
            signal_with_ts("Nausea", 3.0, "2024-01-01"),
            signal_with_ts("Nausea", 1.5, "2023-01-01"),
            signal_with_ts("Nausea", 4.0, "2025-01-01"),
        ];
        let result = compute_signal_timeline(&signals, "drug_x", "Nausea", ScoreField::Prr);
        assert!(result.is_ok());
        if let Ok(tl) = result {
            assert_eq!(tl.points[0].timestamp, "2023-01-01");
            assert_eq!(tl.points[1].timestamp, "2024-01-01");
            assert_eq!(tl.points[2].timestamp, "2025-01-01");
        }
    }

    #[test]
    fn compute_signal_timeline_case_insensitive_ae_name() {
        let signals = vec![signal_with_ts("GI Bleeding", 2.5, "2024-01-01")];
        let result = compute_signal_timeline(&signals, "drug_x", "gi bleeding", ScoreField::Prr);
        assert!(result.is_ok());
        if let Ok(tl) = result {
            assert_eq!(tl.points.len(), 1);
        }
    }

    #[test]
    fn compute_signal_timeline_drug_id_preserved() {
        let signals = vec![signal_with_ts("Nausea", 2.0, "2024-01-01")];
        let result = compute_signal_timeline(&signals, "my_drug", "Nausea", ScoreField::Prr);
        assert!(result.is_ok());
        if let Ok(tl) = result {
            assert_eq!(tl.drug_id, "my_drug");
        }
    }

    // -----------------------------------------------------------------------
    // OverlayError Display
    // -----------------------------------------------------------------------

    #[test]
    fn overlay_error_display_no_signals() {
        let msg = format!("{}", OverlayError::NoSignals);
        assert!(!msg.is_empty());
    }

    #[test]
    fn overlay_error_display_invalid_node_id() {
        let msg = format!("{}", OverlayError::InvalidNodeId("abc".into()));
        assert!(msg.contains("abc"));
    }

    #[test]
    fn overlay_error_display_empty_input() {
        let msg = format!("{}", OverlayError::EmptyInput);
        assert!(!msg.is_empty());
    }

    // -----------------------------------------------------------------------
    // extract_score helper
    // -----------------------------------------------------------------------

    #[test]
    fn extract_score_prr() {
        let s = signal("Nausea", 3.0);
        assert_eq!(extract_score(&s, ScoreField::Prr), Some(3.0));
    }

    #[test]
    fn extract_score_ror() {
        let s = signal("Nausea", 3.0);
        // signal() sets ror = prr + 0.3
        let ror = extract_score(&s, ScoreField::Ror);
        assert!(ror.map_or(false, |v| (v - 3.3).abs() < 1e-9));
    }

    #[test]
    fn extract_score_ic025() {
        let s = signal("Nausea", 3.0);
        assert_eq!(extract_score(&s, ScoreField::Ic025), Some(0.5));
    }

    #[test]
    fn extract_score_ebgm() {
        let s = signal("Nausea", 3.0);
        // signal() sets ebgm = prr - 0.1
        let ebgm = extract_score(&s, ScoreField::Ebgm);
        assert!(ebgm.map_or(false, |v| (v - 2.9).abs() < 1e-9));
    }

    #[test]
    fn extract_score_none_when_absent() {
        let s = SignalScore {
            ae_name: "Rash".into(),
            prr: None,
            ror: None,
            ic025: None,
            ebgm: None,
            case_count: None,
            timestamp: None,
        };
        assert_eq!(extract_score(&s, ScoreField::Prr), None);
        assert_eq!(extract_score(&s, ScoreField::Ebgm), None);
    }

    // -----------------------------------------------------------------------
    // Serde roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn serde_roundtrip_ae_heatmap() {
        let nodes = vec![make_drug("drug1", vec![signal("Nausea", 2.5)])];
        let result = compute_heatmap(&nodes, &default_config());
        assert!(result.is_ok());
        if let Ok(hm) = result {
            let json = serde_json::to_string(&hm).ok();
            assert!(json.is_some(), "serialisation failed");
            if let Some(j) = json {
                let restored: Result<AeHeatmap, _> = serde_json::from_str(&j);
                assert!(restored.is_ok(), "deserialisation failed");
                if let Ok(r) = restored {
                    assert_eq!(r.drug_ids, hm.drug_ids);
                    assert_eq!(r.ae_names, hm.ae_names);
                    assert_eq!(r.cells.len(), hm.cells.len());
                }
            }
        }
    }

    #[test]
    fn serde_roundtrip_overlay_config() {
        let cfg = default_config();
        let json = serde_json::to_string(&cfg).ok();
        assert!(json.is_some(), "serialisation failed");
        if let Some(j) = json {
            let restored: Result<OverlayConfig, _> = serde_json::from_str(&j);
            assert!(restored.is_ok(), "deserialisation failed");
            if let Ok(r) = restored {
                assert_eq!(r.score_field, cfg.score_field);
                assert_eq!(r.normalization, cfg.normalization);
                assert!((r.signal_threshold - cfg.signal_threshold).abs() < f64::EPSILON);
            }
        }
    }
}
