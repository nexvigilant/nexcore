//! Temporal Analysis Methods for Pharmacovigilance
//!
//! This module contains time-based analysis methods for understanding
//! patterns in adverse event timing, trend detection, and forecasting.
//!
//! ## Methods
//!
//! - **Weibull TTO** - Time-to-onset pattern analysis for causality assessment
//! - **ARIMA** - Time series analysis for anomaly detection and forecasting
//!
//! ## Use Cases
//!
//! - Causality assessment via time-to-onset patterns
//! - Trend detection in adverse event reporting
//! - Seasonal effect identification
//! - Anomaly detection (unexpected spikes in reporting)
//!
//! ## Example: Weibull TTO
//!
//! ```rust
//! use nexcore_vigilance::pv::signals::temporal::weibull::{fit_weibull_tto, WeibullTTOConfig, WeibullShape};
//!
//! // Analyze time-to-onset pattern for causality evidence
//! let times = vec![1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0];
//! let config = WeibullTTOConfig::default();
//! let result = fit_weibull_tto(&times, &config).unwrap();
//!
//! match result.pattern {
//!     WeibullShape::EarlyOnset => println!("Supports causal relationship"),
//!     WeibullShape::Random => println!("Neutral for causality"),
//!     WeibullShape::LateOnset => println!("Possible cumulative effect"),
//! }
//! ```
//!
//! ## References
//!
//! - van Puijenbroek EP, et al. (2002). "A comparison of measures of
//!   disproportionality for signal detection in spontaneous reporting
//!   systems for adverse drug reactions." Pharmacoepidemiology and
//!   Drug Safety 11(1):3-10.
//!
//! - Box GEP, Jenkins GM (1970). "Time Series Analysis: Forecasting
//!   and Control." Holden-Day, San Francisco.

pub mod arima;
pub mod weibull;

// Re-export Weibull types for convenience
pub use weibull::{
    WeibullShape, WeibullTTOConfig, WeibullTTOResult, batch_weibull_parallel, fit_weibull_tto,
};

// Re-export ARIMA types
pub use arima::{
    ArimaAnomalyResult, ArimaConfig, ArimaForecast, ArimaResult, batch_arima_parallel,
    detect_anomalies, fit_arima, forecast_arima,
};
