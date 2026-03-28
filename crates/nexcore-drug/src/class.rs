//! Drug class classification enum.
//!
//! `DrugClass` encodes the primary pharmacological mechanism class for
//! a drug entity. Variants cover the major modern drug classes represented
//! in the nexcore drug family. The `Other` variant accommodates classes
//! not yet modelled as first-class variants.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Primary pharmacological mechanism class for a drug.
///
/// # Examples
///
/// ```
/// use nexcore_drug::DrugClass;
///
/// let class = DrugClass::GLP1ReceptorAgonist;
/// assert_eq!(class.to_string(), "GLP-1 Receptor Agonist");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrugClass {
    /// Glucagon-like peptide-1 receptor agonist (e.g. semaglutide, liraglutide)
    GLP1ReceptorAgonist,
    /// GLP-1 / GIP dual agonist (e.g. tirzepatide)
    GLP1GIPDualAgonist,
    /// Anti-amyloid monoclonal antibody (e.g. donanemab, lecanemab)
    AntiAmyloid,
    /// PD-1 / PD-L1 checkpoint inhibitor (e.g. pembrolizumab, nivolumab)
    CheckpointInhibitor,
    /// Janus kinase (JAK) inhibitor (e.g. upadacitinib, tofacitinib)
    JAKInhibitor,
    /// Sodium-glucose cotransporter-2 inhibitor (e.g. dapagliflozin, empagliflozin)
    SGLT2Inhibitor,
    /// CDK4/6 inhibitor (e.g. palbociclib, ribociclib)
    CDK46Inhibitor,
    /// Bruton's tyrosine kinase (BTK) inhibitor (e.g. ibrutinib, zanubrutinib)
    BTKInhibitor,
    /// Tumour necrosis factor alpha inhibitor (e.g. adalimumab, etanercept)
    AntiTNF,
    /// Anti-interleukin-17A antibody (e.g. secukinumab, ixekizumab)
    AntiIL17,
    /// Anti-CD20 monoclonal antibody (e.g. rituximab, ocrelizumab)
    AntiCD20,
    /// Dipeptidyl peptidase-4 inhibitor (e.g. sitagliptin, saxagliptin)
    DPP4Inhibitor,
    /// Poly ADP-ribose polymerase (PARP) inhibitor (e.g. olaparib, niraparib)
    PARP,
    /// Direct oral anticoagulant or vitamin K antagonist (e.g. apixaban, warfarin)
    Anticoagulant,
    /// Vaccine (prophylactic or therapeutic)
    Vaccine,
    /// EGFR tyrosine kinase inhibitor (e.g. osimertinib, erlotinib)
    EGFRTKInhibitor,
    /// Drug class not yet modelled as a first-class variant.
    Other(String),
}

impl fmt::Display for DrugClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::GLP1ReceptorAgonist => "GLP-1 Receptor Agonist",
            Self::GLP1GIPDualAgonist => "GLP-1 / GIP Dual Agonist",
            Self::AntiAmyloid => "Anti-Amyloid Antibody",
            Self::CheckpointInhibitor => "PD-1/PD-L1 Checkpoint Inhibitor",
            Self::JAKInhibitor => "JAK Inhibitor",
            Self::SGLT2Inhibitor => "SGLT2 Inhibitor",
            Self::CDK46Inhibitor => "CDK4/6 Inhibitor",
            Self::BTKInhibitor => "BTK Inhibitor",
            Self::AntiTNF => "Anti-TNF",
            Self::AntiIL17 => "Anti-IL-17A",
            Self::AntiCD20 => "Anti-CD20",
            Self::DPP4Inhibitor => "DPP-4 Inhibitor",
            Self::PARP => "PARP Inhibitor",
            Self::Anticoagulant => "Anticoagulant",
            Self::Vaccine => "Vaccine",
            Self::EGFRTKInhibitor => "EGFR TKI",
            Self::Other(s) => s.as_str(),
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_known_variants() {
        assert_eq!(
            DrugClass::GLP1ReceptorAgonist.to_string(),
            "GLP-1 Receptor Agonist"
        );
        assert_eq!(DrugClass::JAKInhibitor.to_string(), "JAK Inhibitor");
        assert_eq!(DrugClass::Anticoagulant.to_string(), "Anticoagulant");
    }

    #[test]
    fn other_variant_passthrough() {
        let cls = DrugClass::Other("mTOR Inhibitor".to_string());
        assert_eq!(cls.to_string(), "mTOR Inhibitor");
    }

    #[test]
    fn serializes_round_trip() {
        let class = DrugClass::CheckpointInhibitor;
        let json =
            serde_json::to_string(&class).expect("serialization cannot fail on valid variant");
        let parsed: DrugClass =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(class, parsed);
    }

    #[test]
    fn other_serializes_round_trip() {
        let class = DrugClass::Other("PI3K Inhibitor".to_string());
        let json =
            serde_json::to_string(&class).expect("serialization cannot fail on valid variant");
        let parsed: DrugClass =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(class, parsed);
    }

    #[test]
    fn eq_distinguishes_classes() {
        assert_ne!(DrugClass::AntiTNF, DrugClass::AntiIL17);
        assert_eq!(DrugClass::SGLT2Inhibitor, DrugClass::SGLT2Inhibitor);
    }
}
