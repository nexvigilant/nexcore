//! # T1 Primitive Inventory
//!
//! Documents the 13 operational primitives manifested in `nexcore-signal-pipeline`
//! and maps each pipeline stage to its dominant primitive.
//!
//! ## Pipeline Stage Map (10-stage sequence)
//!
//! ```text
//! ingest(sigma) -> normalize(mu) -> detect(N) -> threshold(partial)
//!   -> validate(kappa) -> stats(mu) -> store(pi) -> alert(varsigma)
//!   -> report(mu) -> orchestrate(sigma)
//! ```
//!
//! ## T1 Primitive: Product (x)
//!
//! A manifest is a product type: it combines all primitive presence data
//! into a single conjunctive structure (crate_name x stages x primitives).

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

/// A pipeline stage and its dominant T1 primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineStagePrimitive {
    /// Stage name (e.g., "ingest", "normalize").
    pub stage: &'static str,
    /// Stage order in the pipeline (1-indexed).
    pub order: u8,
    /// Dominant T1 primitive for this stage.
    pub dominant: LexPrimitiva,
    /// Why this primitive dominates this stage.
    pub rationale: &'static str,
}

/// A primitive manifestation within the crate, independent of pipeline stage.
///
/// **Note:** The `examples` field uses `&'static [&'static str]` and does NOT
/// roundtrip through serde. Deserialization always yields an empty slice via
/// `deserialize_static_str_slice`. Use serialization-only workflows or parse
/// via `serde_json::Value` if the examples data is needed after deserialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = ""))]
pub struct PrimitiveManifestation {
    /// The T1 primitive.
    pub primitive: LexPrimitiva,
    /// Symbol (e.g., "sigma", "mu", "N").
    pub symbol: &'static str,
    /// How this primitive manifests in the signal pipeline.
    pub manifestation: &'static str,
    /// Concrete types or modules where this primitive is visible.
    #[serde(serialize_with = "serialize_static_str_slice")]
    #[serde(deserialize_with = "deserialize_static_str_slice")]
    pub examples: &'static [&'static str],
}

fn serialize_static_str_slice<S>(
    val: &&'static [&'static str],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(val.len()))?;
    for item in *val {
        seq.serialize_element(item)?;
    }
    seq.end()
}

fn deserialize_static_str_slice<'de, D>(
    _deserializer: D,
) -> Result<&'static [&'static str], D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Static slices cannot be deserialized from arbitrary input.
    // Return an empty static slice as a safe default for roundtrip.
    Ok(&[])
}

/// Full primitive manifest for the crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CratePrimitiveManifest {
    /// Crate name.
    pub crate_name: &'static str,
    /// Pipeline stages with their dominant primitives.
    pub stages: Vec<PipelineStagePrimitive>,
    /// All primitive manifestations found in the crate.
    pub primitives: Vec<PrimitiveManifestation>,
    /// Total count of distinct primitives present.
    pub primitive_count: usize,
}

// ---- Static data ----

/// The 10 pipeline stages and their dominant primitives.
static PIPELINE_STAGES: &[PipelineStagePrimitive] = &[
    PipelineStagePrimitive {
        stage: "ingest",
        order: 1,
        dominant: LexPrimitiva::Sequence,
        rationale: "Ordered stream of raw reports from external sources",
    },
    PipelineStagePrimitive {
        stage: "normalize",
        order: 2,
        dominant: LexPrimitiva::Mapping,
        rationale: "RawReport -> NormalizedEvent transformation",
    },
    PipelineStagePrimitive {
        stage: "detect",
        order: 3,
        dominant: LexPrimitiva::Quantity,
        rationale: "Contingency table arithmetic (a, b, c, d cell counts)",
    },
    PipelineStagePrimitive {
        stage: "threshold",
        order: 4,
        dominant: LexPrimitiva::Boundary,
        rationale: "Pass/fail boundary evaluation (Evans criteria)",
    },
    PipelineStagePrimitive {
        stage: "validate",
        order: 5,
        dominant: LexPrimitiva::Comparison,
        rationale: "Metric-vs-threshold comparison checks",
    },
    PipelineStagePrimitive {
        stage: "stats",
        order: 6,
        dominant: LexPrimitiva::Mapping,
        rationale: "ContingencyTable -> SignalMetrics pure transformation",
    },
    PipelineStagePrimitive {
        stage: "store",
        order: 7,
        dominant: LexPrimitiva::Persistence,
        rationale: "Durable state for detection results and alerts",
    },
    PipelineStagePrimitive {
        stage: "alert",
        order: 8,
        dominant: LexPrimitiva::State,
        rationale: "Alert lifecycle state machine (New -> UnderReview -> Confirmed -> ...)",
    },
    PipelineStagePrimitive {
        stage: "report",
        order: 9,
        dominant: LexPrimitiva::Mapping,
        rationale: "DetectionResult -> formatted output (JSON, table)",
    },
    PipelineStagePrimitive {
        stage: "orchestrate",
        order: 10,
        dominant: LexPrimitiva::Sequence,
        rationale: "Sequential pipeline coordination across all stages",
    },
];

/// The 13 primitives manifested in this crate.
static PRIMITIVE_MANIFESTATIONS: &[PrimitiveManifestation] = &[
    PrimitiveManifestation {
        primitive: LexPrimitiva::Quantity,
        symbol: "N",
        manifestation: "Numeric measurements: PRR, ROR, IC, EBGM, chi-square, cell counts",
        examples: &["Prr", "Ror", "Ic", "Ebgm", "ChiSquare", "ContingencyTable"],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Sequence,
        symbol: "sigma",
        manifestation: "Pipeline stage ordering and data flow direction",
        examples: &["Pipeline::run", "ingest -> normalize -> detect", "relay chain"],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Mapping,
        symbol: "mu",
        manifestation: "Type transformations between pipeline stages",
        examples: &[
            "RawReport -> NormalizedEvent",
            "ContingencyTable -> SignalMetrics",
            "DetectionResult -> Report",
        ],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Boundary,
        symbol: "partial",
        manifestation: "Pass/fail thresholds separating signal from noise",
        examples: &[
            "ThresholdConfig",
            "EvansThreshold",
            "CompositeThreshold",
            "SignalError",
        ],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Comparison,
        symbol: "kappa",
        manifestation: "Metric evaluation against thresholds and criteria",
        examples: &[
            "ValidationCheck",
            "SignalStrength::from_prr",
            "Threshold::apply",
        ],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::State,
        symbol: "varsigma",
        manifestation: "Lifecycle state machines and encapsulated context",
        examples: &["AlertState", "Alert", "ContingencyTable"],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Persistence,
        symbol: "pi",
        manifestation: "Durable storage of results and alerts",
        examples: &["MemoryStore", "JsonFileStore", "Store trait"],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Product,
        symbol: "x",
        manifestation: "Conjunctive structs combining multiple fields",
        examples: &[
            "DetectionResult",
            "Alert",
            "NormalizedEvent",
            "SignalMetrics",
        ],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Existence,
        symbol: "exists",
        manifestation: "Constructors and factory methods",
        examples: &[
            "DrugEventPair::new",
            "Pipeline::new",
            "NexId generation",
        ],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Location,
        symbol: "lambda",
        manifestation: "Positional context: report sources and MedDRA codes",
        examples: &["ReportSource", "meddra_pt", "meddra_soc"],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Frequency,
        symbol: "nu",
        manifestation: "Reporting rates and disproportionality ratios",
        examples: &["PRR (proportional reporting ratio)", "case counts", "cell frequencies"],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Sum,
        symbol: "Sigma",
        manifestation: "Enum variants as exclusive disjunctions",
        examples: &[
            "SignalStrength (None|Weak|Moderate|Strong|Critical)",
            "AlertState",
            "ReportSource",
            "SignalError",
        ],
    },
    PrimitiveManifestation {
        primitive: LexPrimitiva::Recursion,
        symbol: "rho",
        manifestation: "Hierarchical signal aggregation (SOC -> PT -> report)",
        examples: &["meddra_soc -> meddra_pt hierarchy", "nested pipeline chains"],
    },
];

/// Build the full crate primitive manifest.
#[must_use]
pub fn crate_manifest() -> CratePrimitiveManifest {
    CratePrimitiveManifest {
        crate_name: "nexcore-signal-pipeline",
        stages: PIPELINE_STAGES.to_vec(),
        primitives: PRIMITIVE_MANIFESTATIONS.to_vec(),
        primitive_count: PRIMITIVE_MANIFESTATIONS.len(),
    }
}

/// Return the dominant primitive for a named pipeline stage.
#[must_use]
pub fn stage_primitive(stage_name: &str) -> Option<&'static PipelineStagePrimitive> {
    PIPELINE_STAGES.iter().find(|s| s.stage == stage_name)
}

/// Return all stages dominated by a given primitive.
#[must_use]
pub fn stages_for_primitive(primitive: LexPrimitiva) -> Vec<&'static PipelineStagePrimitive> {
    PIPELINE_STAGES
        .iter()
        .filter(|s| s.dominant == primitive)
        .collect()
}

/// Return the manifestation record for a given primitive.
#[must_use]
pub fn manifestation(primitive: LexPrimitiva) -> Option<&'static PrimitiveManifestation> {
    PRIMITIVE_MANIFESTATIONS
        .iter()
        .find(|m| m.primitive == primitive)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn manifest_has_correct_crate_name() {
        let m = crate_manifest();
        assert_eq!(m.crate_name, "nexcore-signal-pipeline");
    }

    #[test]
    fn manifest_has_10_stages() {
        let m = crate_manifest();
        assert_eq!(m.stages.len(), 10);
    }

    #[test]
    fn manifest_has_13_primitives() {
        let m = crate_manifest();
        assert_eq!(m.primitive_count, 13);
        assert_eq!(m.primitives.len(), 13);
    }

    #[test]
    fn stages_are_ordered_1_through_10() {
        let m = crate_manifest();
        for (i, stage) in m.stages.iter().enumerate() {
            assert_eq!(
                stage.order as usize,
                i + 1,
                "Stage {} should have order {}, got {}",
                stage.stage,
                i + 1,
                stage.order,
            );
        }
    }

    #[test]
    fn ingest_stage_is_sequence() {
        let s = stage_primitive("ingest");
        assert!(s.is_some());
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn normalize_stage_is_mapping() {
        let s = stage_primitive("normalize");
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn detect_stage_is_quantity() {
        let s = stage_primitive("detect");
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn threshold_stage_is_boundary() {
        let s = stage_primitive("threshold");
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn validate_stage_is_comparison() {
        let s = stage_primitive("validate");
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn store_stage_is_persistence() {
        let s = stage_primitive("store");
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::Persistence));
    }

    #[test]
    fn alert_stage_is_state() {
        let s = stage_primitive("alert");
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::State));
    }

    #[test]
    fn orchestrate_stage_is_sequence() {
        let s = stage_primitive("orchestrate");
        assert_eq!(s.map(|s| s.dominant), Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn mapping_dominates_three_stages() {
        let mu_stages = stages_for_primitive(LexPrimitiva::Mapping);
        assert_eq!(mu_stages.len(), 3, "mu should dominate normalize, stats, report");
        let names: Vec<&str> = mu_stages.iter().map(|s| s.stage).collect();
        assert!(names.contains(&"normalize"));
        assert!(names.contains(&"stats"));
        assert!(names.contains(&"report"));
    }

    #[test]
    fn sequence_dominates_two_stages() {
        let sigma_stages = stages_for_primitive(LexPrimitiva::Sequence);
        assert_eq!(sigma_stages.len(), 2, "sigma should dominate ingest and orchestrate");
    }

    #[test]
    fn unknown_stage_returns_none() {
        assert!(stage_primitive("nonexistent").is_none());
    }

    #[test]
    fn quantity_manifestation_includes_prr() {
        let m = manifestation(LexPrimitiva::Quantity);
        assert!(m.is_some());
        if let Some(entry) = m {
            assert!(entry.examples.contains(&"Prr"));
        }
    }

    #[test]
    fn all_primitives_have_non_empty_examples() {
        let m = crate_manifest();
        for p in &m.primitives {
            assert!(
                !p.examples.is_empty(),
                "Primitive {} ({}) has no examples",
                p.symbol,
                p.manifestation,
            );
        }
    }

    #[test]
    fn all_primitives_have_non_empty_manifestation() {
        let m = crate_manifest();
        for p in &m.primitives {
            assert!(
                !p.manifestation.is_empty(),
                "Primitive {} has empty manifestation",
                p.symbol,
            );
        }
    }

    #[test]
    fn manifest_serialization() {
        let m = crate_manifest();
        let json = serde_json::to_string(&m);
        assert!(json.is_ok());
        if let Ok(json_str) = json {
            // Verify JSON structure via serde_json::Value (avoids 'static lifetime)
            let value: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            assert!(value.is_ok());
            if let Ok(v) = value {
                assert_eq!(v["crate_name"], "nexcore-signal-pipeline");
                assert_eq!(v["primitive_count"], 13);
                assert!(v["stages"].is_array());
                assert!(v["primitives"].is_array());
            }
        }
    }

    #[test]
    fn void_primitive_not_present() {
        // Void (empty set) is NOT one of the 13 operational primitives
        // in this crate — signal pipeline always has data.
        let m = manifestation(LexPrimitiva::Void);
        assert!(m.is_none(), "Void should not be manifested in signal pipeline");
    }
}
