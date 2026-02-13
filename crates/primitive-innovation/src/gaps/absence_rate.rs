// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # AbsenceRateDetector
//!
//! **Tier**: T2-C (void + nu + kappa + N)
//! **Fills pair gap**: Void x Frequency (previously unexplored)
//!
//! Detects periodic absence of expected data. Answers the question:
//! "How often does data NOT arrive when it should?"
//!
//! This fills the conceptual gap where "absence has no frequency" —
//! it turns out absence absolutely has a measurable rate.

use core::fmt;

/// A time window for absence tracking.
#[derive(Debug, Clone, Copy)]
pub struct Window {
    /// Start of window (monotonic timestamp).
    pub start: u64,
    /// End of window.
    pub end: u64,
}

impl Window {
    /// Duration of the window in ms.
    #[must_use]
    pub fn duration(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

/// Absence severity classification.
///
/// ## Tier: T2-P (void + kappa)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbsenceSeverity {
    /// Occasional misses, within tolerance.
    Sporadic,
    /// Regular pattern of absence detected.
    Periodic,
    /// Majority of expected data is missing.
    Chronic,
    /// Complete absence — no data at all.
    Total,
}

/// Detects periodic absence of expected data arrivals.
///
/// ## Tier: T2-C (void + nu + kappa + N)
/// Dominant: void (this is fundamentally about absence)
///
/// Innovation: fills the previously unexplored Void x Frequency pair.
/// Absence has a measurable rate when you know what SHOULD arrive.
#[derive(Debug, Clone)]
pub struct AbsenceRateDetector {
    /// Expected arrival interval (ms).
    expected_interval_ms: u64,
    /// Tolerance factor (1.5 = 50% grace before declaring absent).
    tolerance: f64,
    /// Timestamps of actual arrivals.
    arrivals: Vec<u64>,
    /// Detected absence windows.
    absences: Vec<Window>,
    /// Maximum tracked arrivals.
    max_history: usize,
}

impl AbsenceRateDetector {
    /// Create a new absence rate detector.
    ///
    /// `expected_interval_ms`: how often data SHOULD arrive.
    #[must_use]
    pub fn new(expected_interval_ms: u64) -> Self {
        Self {
            expected_interval_ms,
            tolerance: 1.5,
            arrivals: Vec::new(),
            absences: Vec::new(),
            max_history: 1000,
        }
    }

    /// Set tolerance factor.
    #[must_use]
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance.max(1.0);
        self
    }

    /// Record a data arrival.
    pub fn record_arrival(&mut self, timestamp: u64) {
        // Check for absence gap before this arrival
        if let Some(&last) = self.arrivals.last() {
            let gap = timestamp.saturating_sub(last);
            let threshold = (self.expected_interval_ms as f64 * self.tolerance) as u64;

            if gap > threshold {
                self.absences.push(Window {
                    start: last,
                    end: timestamp,
                });

                // Trim absence history
                if self.absences.len() > self.max_history {
                    self.absences.remove(0);
                }
            }
        }

        self.arrivals.push(timestamp);

        // Trim arrival history
        if self.arrivals.len() > self.max_history {
            self.arrivals.remove(0);
        }
    }

    /// Check for absence at a given timestamp (no data arrived since expected).
    #[must_use]
    pub fn check_absence(&self, now: u64) -> bool {
        if let Some(&last) = self.arrivals.last() {
            let gap = now.saturating_sub(last);
            let threshold = (self.expected_interval_ms as f64 * self.tolerance) as u64;
            gap > threshold
        } else {
            // No arrivals ever — total absence
            true
        }
    }

    /// Absence rate: fraction of expected intervals that were missed.
    #[must_use]
    pub fn absence_rate(&self) -> f64 {
        if self.arrivals.len() < 2 {
            return 0.0;
        }

        let first = self.arrivals[0];
        let last = self.arrivals[self.arrivals.len() - 1];
        let total_duration = last.saturating_sub(first);

        if total_duration == 0 || self.expected_interval_ms == 0 {
            return 0.0;
        }

        let expected_count = total_duration / self.expected_interval_ms;
        let actual_count = self.arrivals.len().saturating_sub(1) as u64;

        if expected_count == 0 {
            return 0.0;
        }

        let missed = expected_count.saturating_sub(actual_count);
        missed as f64 / expected_count as f64
    }

    /// Classify absence severity.
    #[must_use]
    pub fn severity(&self) -> AbsenceSeverity {
        let rate = self.absence_rate();

        if self.arrivals.is_empty() {
            AbsenceSeverity::Total
        } else if rate > 0.8 {
            AbsenceSeverity::Chronic
        } else if rate > 0.3 {
            AbsenceSeverity::Periodic
        } else {
            AbsenceSeverity::Sporadic
        }
    }

    /// Number of detected absence windows.
    #[must_use]
    pub fn absence_count(&self) -> usize {
        self.absences.len()
    }

    /// Total duration of all absences (ms).
    #[must_use]
    pub fn total_absence_duration(&self) -> u64 {
        self.absences.iter().map(|w| w.duration()).sum()
    }

    /// Mean absence duration (ms).
    #[must_use]
    pub fn mean_absence_duration(&self) -> f64 {
        if self.absences.is_empty() {
            return 0.0;
        }
        self.total_absence_duration() as f64 / self.absences.len() as f64
    }

    /// Absence frequency: absences per expected interval.
    #[must_use]
    pub fn absence_frequency_hz(&self) -> f64 {
        if self.arrivals.len() < 2 || self.expected_interval_ms == 0 {
            return 0.0;
        }

        let first = self.arrivals[0];
        let last = self.arrivals[self.arrivals.len() - 1];
        let duration_sec = (last.saturating_sub(first)) as f64 / 1000.0;

        if duration_sec <= 0.0 {
            return 0.0;
        }

        self.absences.len() as f64 / duration_sec
    }
}

impl fmt::Display for AbsenceRateDetector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AbsenceRateDetector({:.1}% absent, {:?}, {} gaps)",
            self.absence_rate() * 100.0,
            self.severity(),
            self.absence_count(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_absence_regular_arrivals() {
        let mut detector = AbsenceRateDetector::new(1000); // expect every 1s

        for i in 0..10 {
            detector.record_arrival(i * 1000);
        }

        assert_eq!(detector.absence_count(), 0);
        assert!(detector.absence_rate() < 0.01);
        assert_eq!(detector.severity(), AbsenceSeverity::Sporadic);
    }

    #[test]
    fn test_detects_gap() {
        let mut detector = AbsenceRateDetector::new(1000);

        detector.record_arrival(0);
        detector.record_arrival(1000);
        detector.record_arrival(2000);
        // Gap: 3000, 4000 missing
        detector.record_arrival(5000);

        assert_eq!(detector.absence_count(), 1);
        assert!(detector.absence_rate() > 0.0);
    }

    #[test]
    fn test_check_absence_now() {
        let mut detector = AbsenceRateDetector::new(1000).with_tolerance(1.5);

        detector.record_arrival(0);
        detector.record_arrival(1000);

        // At 2000ms — within tolerance (1500ms threshold)
        assert!(!detector.check_absence(2000));

        // At 3000ms — beyond tolerance
        assert!(detector.check_absence(3000));
    }

    #[test]
    fn test_total_absence() {
        let detector = AbsenceRateDetector::new(1000);
        assert_eq!(detector.severity(), AbsenceSeverity::Total);
        assert!(detector.check_absence(5000));
    }

    #[test]
    fn test_chronic_absence() {
        let mut detector = AbsenceRateDetector::new(100); // expect every 100ms

        // Only arrive every 1000ms = 90% absence rate
        for i in 0..10 {
            detector.record_arrival(i * 1000);
        }

        assert!(detector.absence_rate() > 0.8);
        assert_eq!(detector.severity(), AbsenceSeverity::Chronic);
    }
}
