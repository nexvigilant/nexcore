//! Cardiovascular pathology detection and diagnosis.
//!
//! ## Disease Patterns
//!
//! | Pathology | Symptom | Computational Analog |
//! |-----------|---------|---------------------|
//! | Atherosclerosis | Vessel occlusion | Pipeline latency accumulation |
//! | Infarction | Complete blockage | Critical path failure |
//! | Hypertension | Sustained high BP | Chronic backpressure |
//! | Anemia | Low carriers | Insufficient transport capacity |
//! | Hypotension | Low BP | Insufficient throughput |
//!
//! ## T1 Grounding: ∝ (Irreversibility) dominant
//!
//! Pathology IS irreversibility — damage that cannot be simply undone.
//! Atherosclerosis accumulates. Infarction kills tissue. These are
//! one-way transitions requiring active intervention to address.

use serde::{Deserialize, Serialize};

use crate::blood::BloodHealth;
use crate::pressure::Baroreceptor;
use crate::types::Pathology;
use crate::vessels::VascularBed;

/// Severity of a detected pathology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Monitoring only
    Mild,
    /// Requires attention
    Moderate,
    /// Requires intervention
    Severe,
    /// Life-threatening
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mild => write!(f, "Mild"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Severe => write!(f, "Severe"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// A diagnosed condition with severity and recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    /// The pathology
    pub pathology: Pathology,
    /// Severity classification
    pub severity: Severity,
    /// Recommended action
    pub recommendation: String,
}

/// Run a full cardiovascular diagnostic scan.
///
/// Checks vessels, pressure, and blood health for pathologies.
pub fn diagnose(
    beds: &[VascularBed],
    baroreceptor: &Baroreceptor,
    blood_health: BloodHealth,
) -> Vec<Diagnosis> {
    let mut diagnoses = Vec::new();

    // Check for atherosclerosis (vessel occlusion)
    for bed in beds {
        for vessel in &bed.vessels {
            if vessel.occlusion > 0.3 {
                let severity = if vessel.occlusion > 0.9 {
                    Severity::Critical
                } else if vessel.occlusion > 0.7 {
                    Severity::Severe
                } else if vessel.occlusion > 0.5 {
                    Severity::Moderate
                } else {
                    Severity::Mild
                };

                let is_complete = vessel.occlusion > 0.95;
                let pathology = if is_complete {
                    Pathology::Infarction {
                        vessel: vessel.id.clone(),
                        complete: true,
                    }
                } else {
                    Pathology::Atherosclerosis {
                        vessel: vessel.id.clone(),
                        occlusion: vessel.occlusion,
                    }
                };

                let recommendation = match severity {
                    Severity::Critical => {
                        "EMERGENCY: Restore flow immediately. Clear pipeline obstruction."
                            .to_string()
                    }
                    Severity::Severe => {
                        "Urgent: Reduce load on affected pathway. Schedule maintenance.".to_string()
                    }
                    Severity::Moderate => {
                        "Schedule debt reduction on affected pipeline.".to_string()
                    }
                    Severity::Mild => "Monitor. Debt accumulating in pipeline.".to_string(),
                };

                diagnoses.push(Diagnosis {
                    pathology,
                    severity,
                    recommendation,
                });
            }
        }
    }

    // Check for pressure pathology
    if let Some(pathology) = baroreceptor.detect_pathology() {
        let severity = match &pathology {
            Pathology::Hypertension { duration, .. } => {
                if *duration > 50 {
                    Severity::Severe
                } else if *duration > 20 {
                    Severity::Moderate
                } else {
                    Severity::Mild
                }
            }
            Pathology::Hypotension { .. } => Severity::Moderate,
            _ => Severity::Mild,
        };

        let recommendation = match &pathology {
            Pathology::Hypertension { .. } => {
                "Reduce system load. Scale down non-essential consumers.".to_string()
            }
            Pathology::Hypotension { .. } => {
                "Increase throughput. Check for resource starvation.".to_string()
            }
            _ => "Monitor and reassess.".to_string(),
        };

        diagnoses.push(Diagnosis {
            pathology,
            severity,
            recommendation,
        });
    }

    // Check blood health
    match blood_health {
        BloodHealth::Healthy => {}
        BloodHealth::Anemic => {
            diagnoses.push(Diagnosis {
                pathology: Pathology::Anemia {
                    carrier_count: 0,
                    required: 100,
                },
                severity: Severity::Moderate,
                recommendation:
                    "Insufficient data carriers. Increase worker pool or buffer capacity."
                        .to_string(),
            });
        }
        BloodHealth::Leukopenic => {
            diagnoses.push(Diagnosis {
                pathology: Pathology::Unknown(
                    "Leukopenia — insufficient threat detection capacity".to_string(),
                ),
                severity: Severity::Severe,
                recommendation: "Error detection compromised. Restore immunity handlers."
                    .to_string(),
            });
        }
        BloodHealth::Thrombocytopenic => {
            diagnoses.push(Diagnosis {
                pathology: Pathology::Unknown(
                    "Thrombocytopenia — insufficient repair capacity".to_string(),
                ),
                severity: Severity::Moderate,
                recommendation: "Self-repair degraded. Restore repair/reconnect handlers."
                    .to_string(),
            });
        }
        BloodHealth::Hypovolemic => {
            diagnoses.push(Diagnosis {
                pathology: Pathology::Hypotension {
                    pressure: crate::types::Pressure::new(0.1),
                },
                severity: Severity::Severe,
                recommendation: "Transport medium depleted. Restore buffer/queue capacity."
                    .to_string(),
            });
        }
    }

    diagnoses
}
