//! Data-to-Pixel Scales
//!
//! Maps data domains to visual pixel ranges. Inspired by D3's scale
//! abstraction — the most important concept in data visualization.
//!
//! Three scale types:
//! - `LinearScale` — proportional mapping for continuous numeric data
//! - `LogScale` — logarithmic mapping for data spanning orders of magnitude
//! - `OrdinalScale` — discrete category mapping with even spacing
//!
//! Grounded: μ (Mapping) domain→range, N (Quantity) numeric transform,
//!           ∂ (Boundary) domain clamping.

use std::fmt;

// ============================================================================
// Scale trait
// ============================================================================

/// Common interface for all scale types.
pub trait Scale {
    /// Map a data value to a pixel position.
    fn map(&self, value: f64) -> f64;

    /// Map a pixel position back to a data value (inverse transform).
    /// Returns `None` if the scale doesn't support inversion.
    fn inverse(&self, pixel: f64) -> Option<f64>;

    /// Generate "nice" tick values across the domain.
    fn ticks(&self, count: usize) -> Vec<f64>;

    /// The pixel range (start, end).
    fn range(&self) -> (f64, f64);
}

// ============================================================================
// LinearScale
// ============================================================================

/// Proportional mapping from a continuous numeric domain to a pixel range.
///
/// ```text
/// pixel = range_start + (value - domain_min) / (domain_max - domain_min) * (range_end - range_start)
/// ```
#[derive(Debug, Clone)]
pub struct LinearScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    clamp: bool,
}

impl LinearScale {
    /// Create a linear scale with domain `[min, max]` and pixel range `[start, end]`.
    #[must_use]
    pub fn new(domain_min: f64, domain_max: f64, range_start: f64, range_end: f64) -> Self {
        Self {
            domain_min,
            domain_max,
            range_start,
            range_end,
            clamp: false,
        }
    }

    /// Enable clamping: output is constrained to the range even for out-of-domain values.
    #[must_use]
    pub fn with_clamp(mut self) -> Self {
        self.clamp = true;
        self
    }

    /// Set a new domain.
    #[must_use]
    pub fn domain(mut self, min: f64, max: f64) -> Self {
        self.domain_min = min;
        self.domain_max = max;
        self
    }

    /// Set a new range.
    #[must_use]
    pub fn range_to(mut self, start: f64, end: f64) -> Self {
        self.range_start = start;
        self.range_end = end;
        self
    }

    /// "Nice" the domain to round numbers (expand to nearest nice boundary).
    #[must_use]
    pub fn nice(mut self) -> Self {
        let span = self.domain_max - self.domain_min;
        if span.abs() < f64::EPSILON {
            return self;
        }
        let step = nice_step(span, 10);
        self.domain_min = (self.domain_min / step).floor() * step;
        self.domain_max = (self.domain_max / step).ceil() * step;
        self
    }

    fn normalize(&self, value: f64) -> f64 {
        let span = self.domain_max - self.domain_min;
        if span.abs() < f64::EPSILON {
            return 0.5;
        }
        (value - self.domain_min) / span
    }
}

impl Scale for LinearScale {
    fn map(&self, value: f64) -> f64 {
        let t = self.normalize(value);
        let t = if self.clamp { t.clamp(0.0, 1.0) } else { t };
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn inverse(&self, pixel: f64) -> Option<f64> {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return None;
        }
        let t = (pixel - self.range_start) / range_span;
        Some(self.domain_min + t * (self.domain_max - self.domain_min))
    }

    fn ticks(&self, count: usize) -> Vec<f64> {
        if count == 0 {
            return vec![];
        }
        let span = self.domain_max - self.domain_min;
        if span.abs() < f64::EPSILON {
            return vec![self.domain_min];
        }
        let step = nice_step(span, count);
        let start = (self.domain_min / step).ceil() * step;
        let mut ticks = Vec::with_capacity(count + 2);
        let mut v = start;
        while v <= self.domain_max + step * 0.001 {
            // Round to avoid floating-point artifacts
            let rounded = (v / step).round() * step;
            ticks.push(rounded);
            v += step;
        }
        ticks
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }
}

impl fmt::Display for LinearScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LinearScale([{:.2}, {:.2}] -> [{:.1}, {:.1}])",
            self.domain_min, self.domain_max, self.range_start, self.range_end
        )
    }
}

// ============================================================================
// LogScale
// ============================================================================

/// Logarithmic mapping for data spanning orders of magnitude.
///
/// Domain values must be positive. Uses natural log internally,
/// configurable base for tick generation.
#[derive(Debug, Clone)]
pub struct LogScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    base: f64,
    clamp: bool,
}

impl LogScale {
    /// Create a log scale. Domain values must be > 0.
    #[must_use]
    pub fn new(domain_min: f64, domain_max: f64, range_start: f64, range_end: f64) -> Self {
        Self {
            domain_min: domain_min.max(f64::EPSILON),
            domain_max: domain_max.max(f64::EPSILON),
            range_start,
            range_end,
            base: 10.0,
            clamp: false,
        }
    }

    /// Set the log base (default: 10).
    #[must_use]
    pub fn with_base(mut self, base: f64) -> Self {
        self.base = base.max(1.001); // Prevent degenerate bases
        self
    }

    /// Enable clamping.
    #[must_use]
    pub fn with_clamp(mut self) -> Self {
        self.clamp = true;
        self
    }

    fn log_val(&self, v: f64) -> f64 {
        v.max(f64::EPSILON).ln() / self.base.ln()
    }
}

impl Scale for LogScale {
    fn map(&self, value: f64) -> f64 {
        let log_min = self.log_val(self.domain_min);
        let log_max = self.log_val(self.domain_max);
        let log_span = log_max - log_min;
        if log_span.abs() < f64::EPSILON {
            return (self.range_start + self.range_end) / 2.0;
        }
        let t = (self.log_val(value) - log_min) / log_span;
        let t = if self.clamp { t.clamp(0.0, 1.0) } else { t };
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn inverse(&self, pixel: f64) -> Option<f64> {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return None;
        }
        let t = (pixel - self.range_start) / range_span;
        let log_min = self.log_val(self.domain_min);
        let log_max = self.log_val(self.domain_max);
        let log_val = log_min + t * (log_max - log_min);
        Some(self.base.powf(log_val))
    }

    fn ticks(&self, count: usize) -> Vec<f64> {
        if count == 0 {
            return vec![];
        }
        let log_min = self.log_val(self.domain_min).floor() as i32;
        let log_max = self.log_val(self.domain_max).ceil() as i32;

        let mut ticks = Vec::new();
        for exp in log_min..=log_max {
            let base_val = self.base.powi(exp);
            ticks.push(base_val);
            // Add subdivisions if we need more ticks
            if ticks.len() < count {
                for mult in [2.0, 5.0] {
                    let v = base_val * mult;
                    if v >= self.domain_min && v <= self.domain_max {
                        ticks.push(v);
                    }
                }
            }
        }
        ticks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ticks.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
        if ticks.len() > count + 2 {
            // Thin to approximately count ticks
            let step = ticks.len() / count;
            ticks = ticks.into_iter().step_by(step.max(1)).collect();
        }
        ticks
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }
}

// ============================================================================
// OrdinalScale
// ============================================================================

/// Maps discrete categories to evenly-spaced pixel positions.
///
/// Given N categories and a pixel range, each category gets a band of
/// `range_span / N` pixels. `map()` returns the center of each band.
#[derive(Debug, Clone)]
pub struct OrdinalScale {
    categories: Vec<String>,
    range_start: f64,
    range_end: f64,
    padding: f64,
}

impl OrdinalScale {
    /// Create an ordinal scale from a list of category labels.
    #[must_use]
    pub fn new(categories: Vec<String>, range_start: f64, range_end: f64) -> Self {
        Self {
            categories,
            range_start,
            range_end,
            padding: 0.1,
        }
    }

    /// Set padding between bands as a fraction of band width (0.0 - 1.0).
    #[must_use]
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding.clamp(0.0, 0.9);
        self
    }

    /// Map a category name to its center pixel position.
    /// Returns `None` if the category isn't found.
    #[must_use]
    pub fn map_category(&self, category: &str) -> Option<f64> {
        let idx = self.categories.iter().position(|c| c == category)?;
        Some(self.map_index(idx))
    }

    /// Map a category index to its center pixel position.
    #[must_use]
    pub fn map_index(&self, index: usize) -> f64 {
        let n = self.categories.len();
        if n == 0 {
            return (self.range_start + self.range_end) / 2.0;
        }
        let span = self.range_end - self.range_start;
        let band = span / n as f64;
        let inner = band * (1.0 - self.padding);
        let _ = inner; // inner band width available for consumers
        self.range_start + band * (index as f64 + 0.5)
    }

    /// The width of each band (total, including padding).
    #[must_use]
    pub fn bandwidth(&self) -> f64 {
        let n = self.categories.len();
        if n == 0 {
            return 0.0;
        }
        (self.range_end - self.range_start) / n as f64
    }

    /// The usable inner width of each band (excluding padding).
    #[must_use]
    pub fn inner_bandwidth(&self) -> f64 {
        self.bandwidth() * (1.0 - self.padding)
    }

    /// Number of categories.
    #[must_use]
    pub fn len(&self) -> usize {
        self.categories.len()
    }

    /// Whether the scale has no categories.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }

    /// Get category label by index.
    #[must_use]
    pub fn label(&self, index: usize) -> Option<&str> {
        self.categories.get(index).map(|s| s.as_str())
    }
}

// ============================================================================
// Nice tick helpers
// ============================================================================

/// Compute a "nice" step size for tick generation.
///
/// Given a data span and desired tick count, returns a step that is a
/// multiple of 1, 2, or 5 times a power of 10. This is the D3 algorithm.
#[must_use]
fn nice_step(span: f64, count: usize) -> f64 {
    if count == 0 || span.abs() < f64::EPSILON {
        return 1.0;
    }
    let raw_step = span / count as f64;
    let magnitude = 10.0_f64.powf(raw_step.abs().log10().floor());
    let residual = raw_step / magnitude;

    let nice = if residual <= 1.5 {
        1.0
    } else if residual <= 3.5 {
        2.0
    } else if residual <= 7.5 {
        5.0
    } else {
        10.0
    };

    nice * magnitude
}

/// Format a tick value for display, choosing appropriate precision.
#[must_use]
pub fn format_tick(value: f64) -> String {
    let abs = value.abs();
    if abs < f64::EPSILON {
        return "0".to_string();
    }
    if abs >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.1}k", value / 1_000.0)
    } else if abs >= 1.0 {
        // Remove trailing zeros
        let s = format!("{:.2}", value);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    } else if abs >= 0.01 {
        format!("{:.2}", value)
    } else {
        format!("{:.1e}", value)
    }
}

/// Format a tick value as a percentage.
#[must_use]
pub fn format_percent(value: f64) -> String {
    format!("{:.0}%", value * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- LinearScale --

    #[test]
    fn linear_basic_mapping() {
        let s = LinearScale::new(0.0, 100.0, 0.0, 500.0);
        assert!((s.map(0.0) - 0.0).abs() < 0.01);
        assert!((s.map(50.0) - 250.0).abs() < 0.01);
        assert!((s.map(100.0) - 500.0).abs() < 0.01);
    }

    #[test]
    fn linear_inverse() {
        let s = LinearScale::new(0.0, 100.0, 0.0, 500.0);
        let inv = s.inverse(250.0);
        assert!(inv.is_some());
        assert!((inv.unwrap() - 50.0).abs() < 0.01);
    }

    #[test]
    fn linear_clamp() {
        let s = LinearScale::new(0.0, 100.0, 0.0, 500.0).with_clamp();
        assert!((s.map(-50.0) - 0.0).abs() < 0.01);
        assert!((s.map(200.0) - 500.0).abs() < 0.01);
    }

    #[test]
    fn linear_nice_ticks() {
        let s = LinearScale::new(0.0, 97.0, 0.0, 500.0);
        let ticks = s.ticks(5);
        assert!(!ticks.is_empty());
        // Should produce values like 0, 20, 40, 60, 80, 100
        for tick in &ticks {
            assert!(
                *tick >= -0.01 && *tick <= 100.01,
                "tick {} out of range",
                tick
            );
        }
    }

    #[test]
    fn linear_nice_domain() {
        let s = LinearScale::new(3.2, 97.8, 0.0, 500.0).nice();
        assert!((s.domain_min - 0.0).abs() < 0.01);
        assert!((s.domain_max - 100.0).abs() < 0.01);
    }

    #[test]
    fn linear_display() {
        let s = LinearScale::new(0.0, 100.0, 0.0, 500.0);
        let display = format!("{s}");
        assert!(display.contains("LinearScale"));
    }

    // -- LogScale --

    #[test]
    fn log_basic_mapping() {
        let s = LogScale::new(1.0, 1000.0, 0.0, 300.0);
        // log10(1)=0, log10(1000)=3
        assert!((s.map(1.0) - 0.0).abs() < 0.01);
        assert!((s.map(1000.0) - 300.0).abs() < 0.01);
        // log10(10)=1, should be at 1/3 of range = 100
        assert!((s.map(10.0) - 100.0).abs() < 0.1);
    }

    #[test]
    fn log_inverse() {
        let s = LogScale::new(1.0, 1000.0, 0.0, 300.0);
        let inv = s.inverse(100.0);
        assert!(inv.is_some());
        assert!((inv.unwrap() - 10.0).abs() < 0.1);
    }

    #[test]
    fn log_ticks_are_powers() {
        let s = LogScale::new(1.0, 10000.0, 0.0, 400.0);
        let ticks = s.ticks(5);
        assert!(!ticks.is_empty());
        // Should include powers of 10: 1, 10, 100, 1000, 10000
        assert!(ticks.iter().any(|t| (*t - 1.0).abs() < 0.01));
        assert!(ticks.iter().any(|t| (*t - 10.0).abs() < 0.1));
    }

    // -- OrdinalScale --

    #[test]
    fn ordinal_basic() {
        let cats = vec!["A".into(), "B".into(), "C".into()];
        let s = OrdinalScale::new(cats, 0.0, 300.0);
        // 3 categories in 300px → 100px bands, centers at 50, 150, 250
        assert!((s.map_index(0) - 50.0).abs() < 0.01);
        assert!((s.map_index(1) - 150.0).abs() < 0.01);
        assert!((s.map_index(2) - 250.0).abs() < 0.01);
    }

    #[test]
    fn ordinal_by_name() {
        let cats = vec!["Science".into(), "Chemistry".into(), "Physics".into()];
        let s = OrdinalScale::new(cats, 0.0, 300.0);
        assert!(s.map_category("Chemistry").is_some());
        assert!(s.map_category("Unknown").is_none());
    }

    #[test]
    fn ordinal_bandwidth() {
        let cats = vec!["A".into(), "B".into(), "C".into(), "D".into()];
        let s = OrdinalScale::new(cats, 0.0, 400.0);
        assert!((s.bandwidth() - 100.0).abs() < 0.01);
        assert!(s.inner_bandwidth() < s.bandwidth());
    }

    // -- Tick formatting --

    #[test]
    fn format_tick_zero() {
        assert_eq!(format_tick(0.0), "0");
    }

    #[test]
    fn format_tick_integer() {
        assert_eq!(format_tick(50.0), "50");
    }

    #[test]
    fn format_tick_decimal() {
        assert_eq!(format_tick(0.75), "0.75");
    }

    #[test]
    fn format_tick_thousands() {
        assert_eq!(format_tick(1500.0), "1.5k");
    }

    #[test]
    fn format_tick_millions() {
        assert_eq!(format_tick(2_500_000.0), "2.5M");
    }

    #[test]
    fn format_percent_basic() {
        assert_eq!(format_percent(0.75), "75%");
        assert_eq!(format_percent(1.0), "100%");
    }

    // -- nice_step --

    #[test]
    fn nice_step_round_numbers() {
        // span=100, count=5 → step should be 20
        let step = nice_step(100.0, 5);
        assert!((step - 20.0).abs() < 0.01);
    }

    #[test]
    fn nice_step_small_span() {
        // span=0.7, count=5 → step should be ~0.1 or 0.2
        let step = nice_step(0.7, 5);
        assert!(step > 0.09);
        assert!(step < 0.3);
    }
}
