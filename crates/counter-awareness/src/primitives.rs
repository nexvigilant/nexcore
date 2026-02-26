//! # Sensing & Counter-Awareness Primitives
//!
//! T1 irreducible atoms of detection and counter-detection.
//!
//! ## Lex Primitiva Grounding
//! - `SensingPrimitive` → κ (Comparison) — detection = comparing signal against threshold
//! - `CounterPrimitive` → ∂ (Boundary) — counter-awareness = enforcing boundary between target and sensor
//! - `EnergyMode` → Σ (Sum) — active adds emission, passive does not

use serde::{Deserialize, Serialize};

/// T1 sensing primitives — the irreducible atoms of detection.
///
/// Every sensor system decomposes into operations on these primitives.
/// Detection occurs when a primitive's measured value exceeds its noise floor.
///
/// Tier: T1 (each passes the three primitive tests: no domain dependencies,
/// grounds to external physics, not merely a synonym)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensingPrimitive {
    /// Energy returning from a surface. Grounds to: physics:optics.
    /// Used by: EO cameras, radar, LiDAR
    Reflection,

    /// Energy radiated from a source. Grounds to: physics:thermodynamics.
    /// Used by: thermal/IR sensors, passive RF
    Emission,

    /// Difference between adjacent values. Grounds to: perception:discrimination.
    /// Used by: all sensors (target must differ from background)
    Contrast,

    /// Demarcation between inside/outside. Grounds to: topology:set_theory.
    /// Used by: edge detection, shape recognition, SAR imaging
    Boundary,

    /// Magnitude of energy at a point. Grounds to: physics:amplitude.
    /// Used by: all sensors (signal must exceed noise floor)
    Intensity,

    /// Rate of oscillation per unit time. Grounds to: physics:wave_theory.
    /// Used by: spectral sensors, radar (Doppler), RF detection
    Frequency,

    /// Spatial separation between points. Grounds to: geometry:metric.
    /// Used by: LiDAR (time-of-flight), radar (range), stereoscopic vision
    Distance,

    /// Minimum distinguishable unit. Grounds to: information:sampling.
    /// Used by: all imaging sensors (pixel pitch, angular resolution)
    Resolution,
}

/// T1 counter-primitives — the irreducible atoms of awareness resistance.
///
/// Each counter-primitive negates exactly one sensing primitive's contribution
/// to detection probability. The mapping is not strictly 1:1 (cross-effects
/// exist) but the diagonal of the effectiveness matrix captures primary counters.
///
/// Tier: T1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CounterPrimitive {
    /// Convert incident energy to heat rather than reflecting.
    /// Primary counter to: Reflection
    Absorption,

    /// Match surface emission to ambient background temperature.
    /// Primary counter to: Emission
    ThermalEquilibrium,

    /// Reduce difference between target and background across all bands.
    /// Primary counter to: Contrast
    Homogenization,

    /// Blur edges so boundary detection fails.
    /// Primary counter to: Boundary
    Diffusion,

    /// Reduce returned energy below sensor noise floor.
    /// Primary counter to: Intensity
    Attenuation,

    /// Operate outside the sensor's detectable frequency range.
    /// Primary counter to: Frequency
    BandDenial,

    /// Stay beyond sensor effective range or corrupt time-of-flight.
    /// Primary counter to: Distance
    RangeDenial,

    /// Present signature smaller than sensor's minimum distinguishable unit.
    /// Primary counter to: Resolution
    SubResolution,
}

/// Active vs Passive energy mode classification.
///
/// Active sensors emit energy and measure returns.
/// Passive sensors measure ambient energy.
/// This distinction is critical — active countermeasures create new signatures.
///
/// Tier: T2-P (crosses physics, engineering, biology domains)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnergyMode {
    /// Emits energy, measures return (LiDAR, radar, active sonar)
    Active,
    /// Measures ambient energy (thermal, EO, passive RF)
    Passive,
}

/// Electromagnetic band classification.
///
/// Tier: T2-P (wavelength is cross-domain)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpectralBand {
    /// 380–700 nm (human-visible)
    Visible,
    /// 700 nm – 1 mm (thermal)
    Infrared,
    /// 1 mm – 1 m (radar, SAR)
    Microwave,
    /// < 380 nm
    Ultraviolet,
    /// 1064 nm typical (LiDAR-specific NIR)
    NearInfrared,
    /// Multiple discrete bands
    Multispectral,
}

/// Latency tolerance class for mission classification.
///
/// Tier: T2-P (latency is cross-domain: networking, neuroscience, sensing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LatencyClass {
    /// < 50ms — FPV control, obstacle avoidance
    RealTime,
    /// < 5s — tactical awareness, tracking
    NearRealTime,
    /// Unlimited — survey, mapping, forensic analysis
    PostProcessed,
}

/// A sensor system — T3 domain-specific composition of primitives.
///
/// Tier: T3 (decomposes to T1 sensing primitives + T2 spectral/energy classifications)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSystem {
    pub name: String,
    pub energy_mode: EnergyMode,
    pub spectral_band: SpectralBand,
    pub latency_class: LatencyClass,
    /// Which T1 primitives this sensor relies on for detection
    pub primary_primitives: Vec<SensingPrimitive>,
    /// Effective detection range in meters
    pub max_range_m: f64,
    /// Noise floor — minimum detectable signal (normalized 0..1)
    pub noise_floor: f64,
}

/// A countermeasure — T3 domain-specific composition of counter-primitives.
///
/// Tier: T3
///
/// ## Note on `effectiveness`
/// The `effectiveness` field stores per-counter effectiveness overrides [0.0, 1.0]
/// in the same order as `primary_counters`. However, the current detection model
/// sources all effectiveness values from `EffectivenessMatrix` (a global physics-grounded
/// lookup table), not from this field. This field is preserved as public API so callers
/// can record countermeasure-specific performance data; a future extension should wire it
/// into `optimize_loadout` as a per-instance matrix override.
///
/// # Invariant
/// `effectiveness.len()` should equal `primary_counters.len()` when populated.
/// The field is not validated at construction time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Countermeasure {
    pub name: String,
    pub energy_mode: EnergyMode,
    /// Which T1 counter-primitives this countermeasure implements
    pub primary_counters: Vec<CounterPrimitive>,
    /// Mass cost in kg
    pub weight_kg: f64,
    /// Power consumption in watts (0 for passive)
    pub power_w: f64,
    /// Per-counter effectiveness overrides [0.0, 1.0], parallel to `primary_counters`.
    /// Currently informational — see struct-level doc for wiring status.
    pub effectiveness: Vec<f64>,
}

impl SensingPrimitive {
    /// Parse from a string name (case-insensitive, underscore/hyphen-separated).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().replace('-', "_").as_str() {
            "reflection" => Some(Self::Reflection),
            "emission" => Some(Self::Emission),
            "contrast" => Some(Self::Contrast),
            "boundary" => Some(Self::Boundary),
            "intensity" => Some(Self::Intensity),
            "frequency" => Some(Self::Frequency),
            "distance" => Some(Self::Distance),
            "resolution" => Some(Self::Resolution),
            _ => None,
        }
    }

    /// Returns the primary counter-primitive for this sensing primitive.
    /// The diagonal of the effectiveness matrix.
    pub fn primary_counter(&self) -> CounterPrimitive {
        match self {
            Self::Reflection => CounterPrimitive::Absorption,
            Self::Emission => CounterPrimitive::ThermalEquilibrium,
            Self::Contrast => CounterPrimitive::Homogenization,
            Self::Boundary => CounterPrimitive::Diffusion,
            Self::Intensity => CounterPrimitive::Attenuation,
            Self::Frequency => CounterPrimitive::BandDenial,
            Self::Distance => CounterPrimitive::RangeDenial,
            Self::Resolution => CounterPrimitive::SubResolution,
        }
    }

    /// All 8 sensing primitives in canonical order.
    pub fn all() -> &'static [SensingPrimitive; 8] {
        &[
            Self::Reflection,
            Self::Emission,
            Self::Contrast,
            Self::Boundary,
            Self::Intensity,
            Self::Frequency,
            Self::Distance,
            Self::Resolution,
        ]
    }

    /// Index in the canonical ordering (for matrix addressing).
    pub fn index(&self) -> usize {
        match self {
            Self::Reflection => 0,
            Self::Emission => 1,
            Self::Contrast => 2,
            Self::Boundary => 3,
            Self::Intensity => 4,
            Self::Frequency => 5,
            Self::Distance => 6,
            Self::Resolution => 7,
        }
    }
}

impl CounterPrimitive {
    /// Parse from a string name (case-insensitive, underscore/hyphen-separated).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().replace('-', "_").as_str() {
            "absorption" => Some(Self::Absorption),
            "thermal_equilibrium" | "thermalequilibrium" => Some(Self::ThermalEquilibrium),
            "homogenization" => Some(Self::Homogenization),
            "diffusion" => Some(Self::Diffusion),
            "attenuation" => Some(Self::Attenuation),
            "band_denial" | "banddenial" => Some(Self::BandDenial),
            "range_denial" | "rangedenial" => Some(Self::RangeDenial),
            "sub_resolution" | "subresolution" => Some(Self::SubResolution),
            _ => None,
        }
    }

    /// All 8 counter-primitives in canonical order.
    pub fn all() -> &'static [CounterPrimitive; 8] {
        &[
            Self::Absorption,
            Self::ThermalEquilibrium,
            Self::Homogenization,
            Self::Diffusion,
            Self::Attenuation,
            Self::BandDenial,
            Self::RangeDenial,
            Self::SubResolution,
        ]
    }

    /// Index in the canonical ordering (for matrix addressing).
    pub fn index(&self) -> usize {
        match self {
            Self::Absorption => 0,
            Self::ThermalEquilibrium => 1,
            Self::Homogenization => 2,
            Self::Diffusion => 3,
            Self::Attenuation => 4,
            Self::BandDenial => 5,
            Self::RangeDenial => 6,
            Self::SubResolution => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensing_primitive_from_name_roundtrip() {
        // Every canonical name parses to the correct variant
        assert_eq!(
            SensingPrimitive::from_name("reflection"),
            Some(SensingPrimitive::Reflection)
        );
        assert_eq!(
            SensingPrimitive::from_name("emission"),
            Some(SensingPrimitive::Emission)
        );
        assert_eq!(
            SensingPrimitive::from_name("contrast"),
            Some(SensingPrimitive::Contrast)
        );
        assert_eq!(
            SensingPrimitive::from_name("boundary"),
            Some(SensingPrimitive::Boundary)
        );
        assert_eq!(
            SensingPrimitive::from_name("intensity"),
            Some(SensingPrimitive::Intensity)
        );
        assert_eq!(
            SensingPrimitive::from_name("frequency"),
            Some(SensingPrimitive::Frequency)
        );
        assert_eq!(
            SensingPrimitive::from_name("distance"),
            Some(SensingPrimitive::Distance)
        );
        assert_eq!(
            SensingPrimitive::from_name("resolution"),
            Some(SensingPrimitive::Resolution)
        );
        // Case-insensitive
        assert_eq!(
            SensingPrimitive::from_name("REFLECTION"),
            Some(SensingPrimitive::Reflection)
        );
        assert_eq!(
            SensingPrimitive::from_name("Contrast"),
            Some(SensingPrimitive::Contrast)
        );
        // Unknown name returns None
        assert_eq!(SensingPrimitive::from_name("echolocation"), None);
        assert_eq!(SensingPrimitive::from_name(""), None);
    }

    #[test]
    fn counter_primitive_from_name_roundtrip() {
        assert_eq!(
            CounterPrimitive::from_name("absorption"),
            Some(CounterPrimitive::Absorption)
        );
        assert_eq!(
            CounterPrimitive::from_name("thermal_equilibrium"),
            Some(CounterPrimitive::ThermalEquilibrium)
        );
        assert_eq!(
            CounterPrimitive::from_name("thermalequilibrium"),
            Some(CounterPrimitive::ThermalEquilibrium)
        );
        assert_eq!(
            CounterPrimitive::from_name("homogenization"),
            Some(CounterPrimitive::Homogenization)
        );
        assert_eq!(
            CounterPrimitive::from_name("diffusion"),
            Some(CounterPrimitive::Diffusion)
        );
        assert_eq!(
            CounterPrimitive::from_name("attenuation"),
            Some(CounterPrimitive::Attenuation)
        );
        assert_eq!(
            CounterPrimitive::from_name("band_denial"),
            Some(CounterPrimitive::BandDenial)
        );
        assert_eq!(
            CounterPrimitive::from_name("banddenial"),
            Some(CounterPrimitive::BandDenial)
        );
        assert_eq!(
            CounterPrimitive::from_name("range_denial"),
            Some(CounterPrimitive::RangeDenial)
        );
        assert_eq!(
            CounterPrimitive::from_name("rangedenial"),
            Some(CounterPrimitive::RangeDenial)
        );
        assert_eq!(
            CounterPrimitive::from_name("sub_resolution"),
            Some(CounterPrimitive::SubResolution)
        );
        assert_eq!(
            CounterPrimitive::from_name("subresolution"),
            Some(CounterPrimitive::SubResolution)
        );
        // Case-insensitive
        assert_eq!(
            CounterPrimitive::from_name("ABSORPTION"),
            Some(CounterPrimitive::Absorption)
        );
        // Unknown returns None
        assert_eq!(CounterPrimitive::from_name("jamming"), None);
    }

    #[test]
    fn sensing_primary_counter_diagonal() {
        // primary_counter() must map each sensing primitive to its diagonal counter
        assert_eq!(
            SensingPrimitive::Reflection.primary_counter(),
            CounterPrimitive::Absorption
        );
        assert_eq!(
            SensingPrimitive::Emission.primary_counter(),
            CounterPrimitive::ThermalEquilibrium
        );
        assert_eq!(
            SensingPrimitive::Contrast.primary_counter(),
            CounterPrimitive::Homogenization
        );
        assert_eq!(
            SensingPrimitive::Boundary.primary_counter(),
            CounterPrimitive::Diffusion
        );
        assert_eq!(
            SensingPrimitive::Intensity.primary_counter(),
            CounterPrimitive::Attenuation
        );
        assert_eq!(
            SensingPrimitive::Frequency.primary_counter(),
            CounterPrimitive::BandDenial
        );
        assert_eq!(
            SensingPrimitive::Distance.primary_counter(),
            CounterPrimitive::RangeDenial
        );
        assert_eq!(
            SensingPrimitive::Resolution.primary_counter(),
            CounterPrimitive::SubResolution
        );
    }

    #[test]
    fn sensing_index_unique_and_bounded() {
        // All 8 sensing primitives must have distinct indices in 0..8
        let mut seen = [false; 8];
        for sp in SensingPrimitive::all() {
            let idx = sp.index();
            assert!(idx < 8, "index {idx} out of bounds for {:?}", sp);
            assert!(!seen[idx], "duplicate index {idx} for {:?}", sp);
            seen[idx] = true;
        }
        // All slots filled
        assert!(
            seen.iter().all(|&v| v),
            "not all 8 sensing indices are covered"
        );
    }

    #[test]
    fn counter_index_unique_and_bounded() {
        // All 8 counter-primitives must have distinct indices in 0..8
        let mut seen = [false; 8];
        for cp in CounterPrimitive::all() {
            let idx = cp.index();
            assert!(idx < 8, "index {idx} out of bounds for {:?}", cp);
            assert!(!seen[idx], "duplicate index {idx} for {:?}", cp);
            seen[idx] = true;
        }
        assert!(
            seen.iter().all(|&v| v),
            "not all 8 counter indices are covered"
        );
    }

    #[test]
    fn all_returns_all_eight_variants() {
        assert_eq!(SensingPrimitive::all().len(), 8);
        assert_eq!(CounterPrimitive::all().len(), 8);
    }
}
