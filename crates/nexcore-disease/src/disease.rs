use serde::{Deserialize, Serialize};

use crate::TherapeuticArea;
use crate::biomarker::Biomarker;
use crate::epidemiology::Epidemiology;
use crate::id::DiseaseId;
use crate::safety_burden::SafetyBurden;
use crate::treatment::TreatmentLine;
use crate::unmet_need::UnmetNeed;

/// A disease entity with full clinical and safety context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Disease {
    /// Unique identifier (e.g., "t2dm", "alzheimers").
    pub id: DiseaseId,
    /// Full disease name.
    pub name: String,
    /// ICD-10 codes.
    pub icd10_codes: Vec<String>,
    /// Primary therapeutic area.
    pub therapeutic_area: TherapeuticArea,
    /// Epidemiological profile.
    pub epidemiology: Epidemiology,
    /// Standard of care treatment lines.
    pub standard_of_care: Vec<TreatmentLine>,
    /// Unmet clinical needs.
    pub unmet_needs: Vec<UnmetNeed>,
    /// Aggregate safety burden across approved drugs.
    pub safety_burden: SafetyBurden,
    /// Relevant biomarkers.
    pub biomarkers: Vec<Biomarker>,
}

impl Disease {
    /// Number of approved drug classes in the treatment algorithm.
    pub fn drug_class_count(&self) -> usize {
        self.standard_of_care
            .iter()
            .flat_map(|line| &line.drug_classes)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    /// Number of critical unmet needs.
    pub fn critical_need_count(&self) -> usize {
        self.unmet_needs
            .iter()
            .filter(|n| matches!(n.severity, crate::unmet_need::NeedSeverity::Critical))
            .count()
    }
}
