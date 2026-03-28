//! Epidemiological data for a disease — prevalence, incidence, demographics, trend.

use serde::{Deserialize, Serialize};

/// Epidemiological profile of a disease.
///
/// All prevalence and incidence values are expressed as percentages of the
/// relevant population (e.g., `10.0` means 10 %). `None` indicates the value
/// is not established or not applicable for this disease.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Epidemiology {
    /// Global prevalence as a percentage of the world population.
    pub global_prevalence: Option<f64>,
    /// US prevalence as a percentage of the US adult population.
    pub us_prevalence: Option<f64>,
    /// Annual incidence per 100 000 persons.
    pub annual_incidence: Option<f64>,
    /// Demographic risk profile.
    pub demographics: Demographics,
    /// Observed secular trend in disease burden.
    pub trend: Trend,
}

/// Demographic characteristics of a disease population.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Demographics {
    /// Median age of symptom onset in years.
    pub median_age_onset: Option<u8>,
    /// Sex ratio string, e.g. `"1.2:1 F:M"` or `"3:1 F:M"`.
    pub sex_ratio: Option<String>,
    /// Major modifiable and non-modifiable risk factors.
    pub risk_factors: Vec<String>,
}

/// Secular trend in disease burden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trend {
    /// Disease burden is rising over time.
    Increasing,
    /// Disease burden is broadly stable.
    Stable,
    /// Disease burden is falling over time.
    Decreasing,
}

impl std::fmt::Display for Trend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Increasing => f.write_str("Increasing"),
            Self::Stable => f.write_str("Stable"),
            Self::Decreasing => f.write_str("Decreasing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trend_display() {
        assert_eq!(Trend::Increasing.to_string(), "Increasing");
        assert_eq!(Trend::Stable.to_string(), "Stable");
        assert_eq!(Trend::Decreasing.to_string(), "Decreasing");
    }

    #[test]
    fn epidemiology_round_trip_serde() {
        let epi = Epidemiology {
            global_prevalence: Some(10.0),
            us_prevalence: Some(11.3),
            annual_incidence: None,
            demographics: Demographics {
                median_age_onset: Some(55),
                sex_ratio: Some("1.1:1 M:F".to_string()),
                risk_factors: vec!["obesity".to_string(), "sedentary lifestyle".to_string()],
            },
            trend: Trend::Increasing,
        };
        let json = serde_json::to_string(&epi).expect("serialise");
        let parsed: Epidemiology = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(epi, parsed);
    }

    #[test]
    fn optional_fields_accept_none() {
        let epi = Epidemiology {
            global_prevalence: None,
            us_prevalence: None,
            annual_incidence: None,
            demographics: Demographics {
                median_age_onset: None,
                sex_ratio: None,
                risk_factors: vec![],
            },
            trend: Trend::Stable,
        };
        let json = serde_json::to_string(&epi).expect("serialise");
        assert!(json.contains("\"global_prevalence\":null"));
    }
}
