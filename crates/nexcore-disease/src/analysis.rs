use crate::disease::Disease;
use crate::safety_burden::SafetyBurden;
use crate::treatment::LineOfTherapy;
use crate::treatment::TreatmentLine;
use crate::unmet_need::NeedSeverity;
use crate::unmet_need::UnmetNeed;

/// Trait for disease-level analysis.
pub trait DiseaseAnalysis {
    /// The underlying disease model.
    fn disease(&self) -> &Disease;

    /// Full treatment landscape.
    fn treatment_landscape(&self) -> &[TreatmentLine] {
        &self.disease().standard_of_care
    }

    /// All unmet needs.
    fn unmet_needs(&self) -> &[UnmetNeed] {
        &self.disease().unmet_needs
    }

    /// Aggregate safety burden.
    fn safety_burden(&self) -> &SafetyBurden {
        &self.disease().safety_burden
    }

    /// Drugs used at a specific line of therapy.
    fn drugs_by_line(&self, line: &LineOfTherapy) -> Vec<&str> {
        self.disease()
            .standard_of_care
            .iter()
            .filter(|t| &t.line == line)
            .flat_map(|t| t.representative_drugs.iter().map(|s| s.as_str()))
            .collect()
    }

    /// Only the critical unmet needs.
    fn critical_unmet_needs(&self) -> Vec<&UnmetNeed> {
        self.disease()
            .unmet_needs
            .iter()
            .filter(|n| matches!(n.severity, NeedSeverity::Critical))
            .collect()
    }
}
