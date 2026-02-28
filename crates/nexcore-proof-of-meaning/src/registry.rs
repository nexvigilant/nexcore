//! Atom Registry — canonical definitions with MedDRA seeding.
//!
//! Chemistry analogue: Crystallization + the Periodic Table.

use nexcore_id::NexId;
use std::collections::HashMap;

use crate::element::{Atom, ElementClass};
use crate::spectrum::Spectrum;

/// The canonical registry of all defined semantic atoms.
pub struct AtomRegistry {
    /// All registered atoms, indexed by UUID.
    atoms: HashMap<NexId, RegisteredAtom>,
    /// Label -> UUID index for lookup by name.
    label_index: HashMap<String, NexId>,
    /// MedDRA code -> UUID index for interop.
    meddra_index: HashMap<String, NexId>,
    /// Class -> Vec<UUID> index for browsing.
    class_index: HashMap<ElementClass, Vec<NexId>>,
}

/// A registered atom includes the atom itself plus its crystallization metadata.
#[derive(Debug, Clone)]
pub struct RegisteredAtom {
    pub atom: Atom,
    /// The reference spectrum — the "gold standard" fingerprint.
    pub reference_spectrum: Option<Spectrum>,
    /// Crystallization provenance — where did this definition come from?
    pub provenance: AtomProvenance,
    /// Is this atom frozen (no further modification) or mutable?
    pub status: AtomStatus,
}

/// Where a canonical atom definition originated.
#[derive(Debug, Clone)]
pub enum AtomProvenance {
    /// Seeded from MedDRA hierarchy.
    MedDRA { code: String, level: MedDRALevel },
    /// Derived from ICH guideline text.
    ICHGuideline { document: String, section: String },
    /// Defined by human expert adjudication.
    ExpertDefined {
        adjudicator: String,
        rationale: String,
    },
    /// Emergent — discovered through spectral analysis of corpora.
    Emergent { corpus: String, confidence: f64 },
}

#[derive(Debug, Clone)]
pub enum MedDRALevel {
    /// System Organ Class (top)
    SOC,
    /// High Level Group Term
    HLGT,
    /// High Level Term
    HLT,
    /// Preferred Term
    PT,
    /// Lowest Level Term
    LLT,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtomStatus {
    /// Active and immutable — production-grade definition.
    Crystallized,
    /// Active but still being refined — may change.
    Supersaturated,
    /// Deprecated — replaced by another atom.
    Dissolved { replacement: Option<NexId> },
}

impl AtomRegistry {
    pub fn new() -> Self {
        Self {
            atoms: HashMap::new(),
            label_index: HashMap::new(),
            meddra_index: HashMap::new(),
            class_index: HashMap::new(),
        }
    }

    /// Crystallize a new atom into the registry.
    ///
    /// If a `reference_spectrum` is provided, the atom is registered with its
    /// spectral fingerprint. Atoms with a reference spectrum can be frozen
    /// to production status via [`Self::freeze`].
    pub fn crystallize(
        &mut self,
        atom: Atom,
        provenance: AtomProvenance,
        reference_spectrum: Option<Spectrum>,
    ) -> CrystallizationResult {
        if let Some(&existing_id) = self.label_index.get(&atom.label) {
            return CrystallizationResult::AlreadyExists {
                existing_id,
                label: atom.label.clone(),
            };
        }

        let id = atom.id;
        let label = atom.label.clone();
        let class = atom.class.clone();
        let meddra = atom.meddra_code.clone();

        let registered = RegisteredAtom {
            atom,
            reference_spectrum,
            provenance,
            status: AtomStatus::Supersaturated,
        };

        self.atoms.insert(id, registered);
        self.label_index.insert(label.clone(), id);
        self.class_index.entry(class).or_default().push(id);

        if let Some(code) = meddra {
            self.meddra_index.insert(code, id);
        }

        CrystallizationResult::Crystallized {
            id,
            label,
            status: AtomStatus::Supersaturated,
        }
    }

    /// Freeze an atom — make it immutable / production-grade.
    pub fn freeze(&mut self, id: &NexId) -> Result<(), RegistryError> {
        let entry = self
            .atoms
            .get_mut(id)
            .ok_or(RegistryError::AtomNotFound(*id))?;

        if entry.reference_spectrum.is_none() {
            return Err(RegistryError::NoReferenceSpectrum(*id));
        }

        entry.status = AtomStatus::Crystallized;
        Ok(())
    }

    /// Look up an atom by label.
    pub fn lookup(&self, label: &str) -> Option<&RegisteredAtom> {
        self.label_index
            .get(label)
            .and_then(|id| self.atoms.get(id))
    }

    /// Get all atoms in a given element class.
    pub fn by_class(&self, class: &ElementClass) -> Vec<&RegisteredAtom> {
        self.class_index
            .get(class)
            .map(|ids| ids.iter().filter_map(|id| self.atoms.get(id)).collect())
            .unwrap_or_default()
    }

    /// Seed the registry from MedDRA SOC level (top of hierarchy).
    pub fn seed_meddra_soc(&mut self) {
        let socs = [
            ("Blood and lymphatic system disorders", "10005329"),
            ("Cardiac disorders", "10007541"),
            ("Congenital, familial and genetic disorders", "10010331"),
            ("Ear and labyrinth disorders", "10013993"),
            ("Endocrine disorders", "10014698"),
            ("Eye disorders", "10015919"),
            ("Gastrointestinal disorders", "10017947"),
            (
                "General disorders and administration site conditions",
                "10018065",
            ),
            ("Hepatobiliary disorders", "10019805"),
            ("Immune system disorders", "10021428"),
            ("Infections and infestations", "10021881"),
            ("Injury, poisoning and procedural complications", "10022117"),
            ("Investigations", "10022891"),
            ("Metabolism and nutrition disorders", "10027433"),
            (
                "Musculoskeletal and connective tissue disorders",
                "10028395",
            ),
            ("Neoplasms benign, malignant and unspecified", "10029104"),
            ("Nervous system disorders", "10029205"),
            ("Pregnancy, puerperium and perinatal conditions", "10036585"),
            ("Psychiatric disorders", "10037175"),
            ("Renal and urinary disorders", "10038359"),
            ("Reproductive system and breast disorders", "10038604"),
            (
                "Respiratory, thoracic and mediastinal disorders",
                "10038738",
            ),
            ("Skin and subcutaneous tissue disorders", "10040785"),
            ("Social circumstances", "10041244"),
            ("Surgical and medical procedures", "10042613"),
            ("Vascular disorders", "10047065"),
        ];

        for (label, code) in socs {
            let mut atom = Atom::new(label, ElementClass::OrganSystem, 0.05);
            atom.meddra_code = Some(code.to_string());

            self.crystallize(
                atom,
                AtomProvenance::MedDRA {
                    code: code.to_string(),
                    level: MedDRALevel::SOC,
                },
                None,
            );
        }
    }

    /// Seed canonical causality atoms (ICH E2A / WHO-UMC categories).
    pub fn seed_causality(&mut self) {
        let atoms = [
            ("certain", 0.10),
            ("probable", 0.20),
            ("possible", 0.35),
            ("unlikely", 0.30),
            ("conditional", 0.50),
            ("unassessable", 0.60),
            ("unclassifiable", 0.55),
            ("related", 0.25),
            ("unrelated", 0.20),
            ("not related", 0.20),
        ];
        for (label, volatility) in atoms {
            self.crystallize(
                Atom::new(label, ElementClass::Causality, volatility),
                AtomProvenance::ICHGuideline {
                    document: "ICH E2A / WHO-UMC".into(),
                    section: "Causality categories".into(),
                },
                None,
            );
        }
    }

    /// Seed canonical temporality atoms.
    pub fn seed_temporality(&mut self) {
        let atoms = [
            ("acute", 0.15),
            ("chronic", 0.15),
            ("delayed", 0.20),
            ("immediate", 0.10),
            ("onset", 0.15),
            ("intermittent", 0.25),
            ("persistent", 0.20),
            ("rapid", 0.20),
            ("gradual", 0.25),
            ("recurrent", 0.20),
        ];
        for (label, volatility) in atoms {
            self.crystallize(
                Atom::new(label, ElementClass::Temporality, volatility),
                AtomProvenance::ICHGuideline {
                    document: "ICH E2B(R3)".into(),
                    section: "Temporal descriptors".into(),
                },
                None,
            );
        }
    }

    /// Seed canonical severity atoms (CTCAE / ICH E2A).
    pub fn seed_severity(&mut self) {
        let atoms = [
            ("mild", 0.15),
            ("moderate", 0.20),
            ("severe", 0.15),
            ("life-threatening", 0.10),
            ("fatal", 0.05),
            ("serious", 0.55),
            ("non-serious", 0.50),
        ];
        for (label, volatility) in atoms {
            self.crystallize(
                Atom::new(label, ElementClass::Severity, volatility),
                AtomProvenance::ICHGuideline {
                    document: "ICH E2A / CTCAE".into(),
                    section: "Severity grading".into(),
                },
                None,
            );
        }
    }

    /// Seed canonical observation type atoms.
    pub fn seed_observation_types(&mut self) {
        let atoms = [
            ("adverse event", 0.25),
            ("adverse reaction", 0.25),
            ("adverse experience", 0.30),
            ("adverse effect", 0.25),
            ("adverse drug reaction", 0.20),
            ("side effect", 0.35),
            ("symptom", 0.20),
            ("sign", 0.15),
            ("diagnosis", 0.15),
            ("finding", 0.20),
            ("lab abnormality", 0.15),
            ("condition", 0.30),
        ];
        for (label, volatility) in atoms {
            self.crystallize(
                Atom::new(label, ElementClass::ObservationType, volatility),
                AtomProvenance::ICHGuideline {
                    document: "ICH E2D".into(),
                    section: "Observation classification".into(),
                },
                None,
            );
        }
    }

    /// Seed canonical modifier atoms.
    pub fn seed_modifiers(&mut self) {
        let atoms = [
            ("expected", 0.55),
            ("unexpected", 0.80),
            ("listed", 0.50),
            ("unlisted", 0.75),
            ("drug-related", 0.35),
            ("treatment-emergent", 0.45),
            ("pre-existing", 0.30),
            ("new", 0.40),
            ("worsening", 0.35),
            ("suspected", 0.40),
        ];
        for (label, volatility) in atoms {
            self.crystallize(
                Atom::new(label, ElementClass::Modifier, volatility),
                AtomProvenance::ICHGuideline {
                    document: "ICH E2B(R3)".into(),
                    section: "Event modifiers".into(),
                },
                None,
            );
        }
    }

    /// Seed canonical action atoms (ICH E2B action taken with drug).
    pub fn seed_actions(&mut self) {
        let atoms = [
            ("drug withdrawn", 0.05),
            ("dose reduced", 0.10),
            ("dose increased", 0.15),
            ("dose not changed", 0.10),
            ("drug interrupted", 0.10),
            ("not applicable", 0.20),
            ("unknown", 0.60),
            ("hospitalized", 0.05),
            ("congenital anomaly", 0.05),
        ];
        for (label, volatility) in atoms {
            self.crystallize(
                Atom::new(label, ElementClass::Action, volatility),
                AtomProvenance::ICHGuideline {
                    document: "ICH E2B(R3)".into(),
                    section: "Action taken".into(),
                },
                None,
            );
        }
    }

    /// Seed canonical outcome atoms (ICH E2B outcome of reaction).
    pub fn seed_outcomes(&mut self) {
        let atoms = [
            ("recovered", 0.05),
            ("recovering", 0.10),
            ("not recovered", 0.10),
            ("recovered with sequelae", 0.10),
            ("fatal", 0.05),
            ("unknown", 0.60),
        ];
        for (label, volatility) in atoms {
            self.crystallize(
                Atom::new(label, ElementClass::Outcome, volatility),
                AtomProvenance::ICHGuideline {
                    document: "ICH E2B(R3)".into(),
                    section: "Outcome of reaction".into(),
                },
                None,
            );
        }
    }

    /// Seed ALL element classes — the full periodic table.
    pub fn seed_all(&mut self) {
        self.seed_meddra_soc();
        self.seed_causality();
        self.seed_temporality();
        self.seed_severity();
        self.seed_observation_types();
        self.seed_modifiers();
        self.seed_actions();
        self.seed_outcomes();
    }

    /// Total registered atoms.
    pub fn count(&self) -> usize {
        self.atoms.len()
    }

    /// Count atoms by element class.
    pub fn count_by_class(&self, class: &ElementClass) -> usize {
        self.class_index.get(class).map_or(0, |v| v.len())
    }

    /// List all registered labels.
    pub fn labels(&self) -> Vec<&str> {
        self.label_index.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for AtomRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum CrystallizationResult {
    Crystallized {
        id: NexId,
        label: String,
        status: AtomStatus,
    },
    AlreadyExists {
        existing_id: NexId,
        label: String,
    },
}

#[derive(Debug, nexcore_error::Error)]
pub enum RegistryError {
    #[error("Atom {0} not found in registry")]
    AtomNotFound(NexId),
    #[error("Atom {0} has no reference spectrum — cannot freeze")]
    NoReferenceSpectrum(NexId),
}
