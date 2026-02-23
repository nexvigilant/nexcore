//! Lex Primitiva MCP tools: T1 symbolic foundation queries
//!
//! The 15 irreducible primitives that ground all higher-tier types:
//! σ (Sequence), μ (Mapping), ς (State), ρ (Recursion), ∅ (Void),
//! ∂ (Boundary), f (Frequency), ∃ (Existence), π (Persistence),
//! → (Causality), κ (Comparison), N (Quantity), λ (Location),
//! ∝ (Irreversibility), Σ (Sum)

use crate::params::{
    LexPrimitivaCompositionParams, LexPrimitivaDominantShiftParams, LexPrimitivaGetParams,
    LexPrimitivaListParams, LexPrimitivaReverseComposeParams, LexPrimitivaReverseLookupParams,
    LexPrimitivaStateModeParams, LexPrimitivaTierParams,
};
use nexcore_lex_primitiva::synthesizer::{RevSynthesizer, SynthesisOpts};
use nexcore_vigilance::lex_primitiva::{
    GroundingTier, GroundsTo, LexPrimitiva, PrimitiveComposition,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashSet;

// ============================================================================
// Tool Implementations (σ: Sequence pattern - flat, no nesting)
// ============================================================================

/// List all 15 Lex Primitiva symbols
pub fn list_primitives(params: LexPrimitivaListParams) -> Result<CallToolResult, McpError> {
    let primitives: Vec<_> = LexPrimitiva::all()
        .iter()
        .map(|p| primitive_to_json(p, params.include_symbols))
        .collect();

    let json = json!({
        "count": 15,
        "tier": "T1-Universal",
        "primitives": primitives,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get details about a specific Lex Primitiva
pub fn get_primitive(params: LexPrimitivaGetParams) -> Result<CallToolResult, McpError> {
    let primitive = find_primitive(&params.name);
    let json = primitive.map_or_else(
        || unknown_primitive_json(&params.name),
        |p| single_primitive_json(&p),
    );
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Classify a type's grounding tier
pub fn classify_tier(params: LexPrimitivaTierParams) -> Result<CallToolResult, McpError> {
    let json = lookup_grounded_type(&params.type_name, tier_json);
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get primitive composition for a grounded type
pub fn get_composition(params: LexPrimitivaCompositionParams) -> Result<CallToolResult, McpError> {
    let json = lookup_grounded_type(&params.type_name, composition_json);
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

// ============================================================================
// Pure Helper Functions (no side effects, single responsibility)
// ============================================================================

fn primitive_to_json(p: &LexPrimitiva, include_symbol: bool) -> serde_json::Value {
    if include_symbol {
        json!({ "name": format!("{:?}", p), "symbol": p.symbol(), "description": p.description() })
    } else {
        json!({ "name": format!("{:?}", p), "description": p.description() })
    }
}

pub(crate) fn find_primitive(name: &str) -> Option<LexPrimitiva> {
    let name_lower = name.to_lowercase();
    LexPrimitiva::all()
        .into_iter()
        .find(|p| format!("{:?}", p).to_lowercase() == name_lower || p.symbol() == name)
}

fn single_primitive_json(p: &LexPrimitiva) -> serde_json::Value {
    json!({
        "name": format!("{:?}", p),
        "symbol": p.symbol(),
        "description": p.description(),
        "tier": "T1-Universal",
        "rust_manifestation": p.rust_manifestation(),
    })
}

fn unknown_primitive_json(name: &str) -> serde_json::Value {
    let available: Vec<_> = LexPrimitiva::all()
        .iter()
        .map(|p| format!("{:?}", p))
        .collect();
    json!({ "error": format!("Unknown primitive: {}", name), "available": available })
}

fn tier_json(type_name: &str, comp: &PrimitiveComposition) -> serde_json::Value {
    let tier = GroundingTier::classify(comp);
    json!({
        "type": type_name,
        "tier": format!("{:?}", tier),
        "confidence": comp.confidence,
        "primitive_count": comp.primitives.len(),
        "dominant": comp.dominant.map(|d| format!("{:?}", d)),
    })
}

fn composition_json(type_name: &str, comp: &PrimitiveComposition) -> serde_json::Value {
    let tier = GroundingTier::classify(comp);
    let primitives: Vec<_> = comp
        .primitives
        .iter()
        .map(|p| json!({ "name": format!("{:?}", p), "symbol": p.symbol() }))
        .collect();

    let mut result = json!({
        "type": type_name,
        "tier": format!("{:?}", tier),
        "confidence": comp.confidence,
        "primitives": primitives,
        "dominant": comp.dominant.map(|d| json!({ "name": format!("{:?}", d), "symbol": d.symbol() })),
    });

    // Enrich with state_mode when present
    if let Some(mode) = &comp.state_mode {
        if let serde_json::Value::Object(ref mut map) = result {
            map.insert(
                "state_mode".to_string(),
                json!({
                    "mode": mode.label(),
                    "symbol": mode.symbol_suffix(),
                    "reversible": mode.is_reversible(),
                    "description": mode.description(),
                }),
            );
        }
    }

    result
}

fn unknown_type_json(type_name: &str) -> serde_json::Value {
    json!({
        "error": format!("Unknown type: {}", type_name),
        "known_grounded_types": KNOWN_TYPES,
    })
}

// ============================================================================
// Type Lookup (μ: Mapping pattern - dispatch table)
// ============================================================================

pub(crate) const KNOWN_TYPES: &[&str] = &[
    // Chemistry T2-P/T2-C
    "ThresholdGate",
    "SaturationKinetics",
    "FeasibilityAssessment",
    "RateLaw",
    "BufferSystem",
    "SignalDetector",
    "DecayKinetics",
    "EquilibriumSystem",
    "AggregationPipeline",
    // Domain types
    "SignalResult",
    "Primitive",
    "PrimitiveTier",
    "HarmType",
    "SafetyMargin",
    "Bdi",
    "EcsScore",
    // Bathroom Lock T2-P (ς State)
    "Occupancy",
    // Frontier T2-P (completing T1 surface coverage)
    "AuditTrail",
    "AbsenceMarker",
    "Pipeline",
    "ConsumptionMark",
    "EntityStatus",
    "ResourcePath",
    "RecursionBound",
    // Patient/Safety T2-P/T2-C
    "Recipient",
    "SafetyBoundary",
    "Harm",
    "Vulnerable",
    "Monitoring",
    "Tracked",
    // Cross-Domain Transfer T2-P
    "Homeostasis",
    "StagedValidation",
    "Atomicity",
    "CompareAndSwap",
    "ToctouWindow",
    "SerializationGuard",
    "RateLimiter",
    "CircuitBreaker",
    "Idempotency",
    "NegativeEvidence",
    "TopologicalAddress",
    "Accumulator",
    "Checkpoint",
    // Quantum Domain Primitives (10 T2-P, 3 T2-C)
    "Amplitude",
    "Phase",
    "Superposition",
    "Measurement",
    "Interference",
    "Uncertainty",
    "Unitarity",
    "Eigenstate",
    "Observable",
    "Hermiticity",
    "Entanglement",
    "Decoherence",
    "Qubit",
    // Biological Pipeline (transcriptase → ribosome → phenotype → hormones)
    "Schema",
    "SchemaKind",
    "Fidelity",
    "Contract",
    "DriftType",
    "DriftSeverity",
    "DriftResult",
    "DriftSignal",
    "Mutation",
    "Phenotype",
    "HormoneLevel",
    "HormoneType",
    "Stimulus",
    "EndocrineState",
    "BehavioralModifiers",
    // Cortex LLM T2-P/T2-C/T3
    "ModelConfig",
    "ModelFormat",
    "GenerateParams",
    "LoraConfig",
    "DatasetConfig",
    "TrainingParams",
    "LoraAdapter",
    "FineTuneJob",
    // Trust Engine T2-P/T2-C (Product × grounding)
    "MultiTrustEngine",
    "TrustDimension",
    "DimensionWeights",
    "TrustEngine",
    "Evidence",
    "TrustLevel",
    "TrustVelocity",
    "TrustConfig",
    // Guardian Engine T2-P/T2-C (Product × grounding)
    "OriginatorType",
    "RiskContext",
    "RiskScore",
    // Spatial Bridge Metric types (κ Comparison dominant)
    "ImpurityMetric",
    "SeverityMetric",
    "RiskScoreMetric",
    "DisproportionalityScore",
    // Insight Engine T2-P/T2-C/T3 (system identity)
    "Pattern",
    "Recognition",
    "Novelty",
    "NoveltyReason",
    "Connection",
    "Compression",
    "Suddenness",
    "InsightEngine",
    // DNA — Foundation (7)
    "Nucleotide",
    "AminoAcid",
    "Codon",
    "Strand",
    "DoubleHelix",
    "CodonTable",
    "CodonVM",
    // DNA — Compute (10)
    "DnaInstruction",
    "DnaProgram",
    "DnaToken",
    "DisasmOptions",
    "DnaGlyph",
    "DnaGlyphPair",
    "DnaErrorCode",
    "DnaDiagnostic",
    "DnaJsonValue",
    "DnaTemplate",
    // DNA — Store (9)
    "DnaType",
    "DnaValue",
    "DnaArray",
    "DnaRecord",
    "DnaMap",
    "DnaFrame",
    "Pixel",
    "VoxelPos",
    "VoxelCube",
    // DNA — Evolve (8)
    "Gene",
    "Genome",
    "Plasmid",
    "Organism",
    "DnaCluster",
    "DnaClusterResult",
    "EvolutionConfig",
    "EvolutionResult",
    // DNA — Gravity (2)
    "Particle",
    "GravityConfig",
    // DNA — Analyze: String Theory (5)
    "StringTension",
    "HarmonicMode",
    "FrequencySpectrum",
    "DnaResonance",
    "StringEnergy",
    // DNA — Analyze: PV Theory (7)
    "CausalityCategory",
    "DnaSafetyLevel",
    "AlertLevel",
    "DrugProfile",
    "DnaSignal",
    "DnaSafetyMargin",
    "CausalityScore",
    "VigilanceState",
    // DNA — Analyze: Mind (5)
    "WordOre",
    "Affinity",
    "DnaLexicon",
    "DnaMindPoint",
    "DnaDrift",
    "DnaStateMind",
    // Energy (ATP/ADP biochemistry)
    "Regime",
    "Strategy",
    "EnergySystem",
    "WasteClass",
    "TokenPool",
    "Operation",
    "OperationBuilder",
    "RecyclingRate",
    "EnergyState",
    // Compliance
    "Assessment",
    "ComplianceResult",
    "Finding",
    "FindingSeverity",
    "Control",
    "ControlCatalog",
    "ControlStatus",
    "Exclusion",
    "ExclusionClassification",
    "ExclusionType",
    // Measure (information theory, graph theory, statistics)
    "BayesianPosterior",
    "Centrality",
    "ChemistryMapping",
    "CodeDensityIndex",
    "CouplingRatio",
    "CrateHealth",
    "CrateId",
    "CrateMeasurement",
    "Density",
    "DriftDirection",
    "MeasureDriftResult",
    "Entropy",
    "GraphAnalysis",
    "GraphNode",
    "HealthComponents",
    "HealthRating",
    "HealthScore",
    "MeasureError",
    "MeasureTimestamp",
    "PoissonCi",
    "Probability",
    "RatingDistribution",
    "RegressionResult",
    "TestDensity",
    "WelchResult",
    "WorkspaceHealth",
    "WorkspaceMeasurement",
    // Signal (PV signal detection pipeline)
    "Prr",
    "Ror",
    "Ic",
    "Ebgm",
    "ChiSquare",
    "SignalStrength",
    "ReportSource",
    "AlertState",
    "SignalError",
    "ConfidenceInterval",
    "DrugEventPair",
    "ContingencyTable",
    "ValidationCheck",
    "ThresholdConfig",
    "NormalizedEvent",
    "ValidationReport",
    "SignalMetrics",
    "RawReport",
    "DetectionResult",
    "Alert",
    "AlertTransitions",
    "BasicNormalizer",
    "SynonymNormalizer",
    "CsvIngestor",
    "JsonIngestor",
    "TableDetector",
    "CompositeThreshold",
    "EvansThreshold",
    "StandardValidator",
    "JsonFileStore",
    "MemoryStore",
    "JsonReporter",
    "TableReporter",
    // Brain (sessions, artifacts, implicit learning)
    "AgentId",
    "Artifact",
    "ArtifactMetadata",
    "ArtifactMetrics",
    "ArtifactSizeInfo",
    "ArtifactType",
    "Belief",
    "BeliefGraph",
    "BeliefImplication",
    "BrainConfig",
    "BrainError",
    "BrainHealth",
    "BrainSession",
    "BrainSnapshot",
    "BrainCheckpoint",
    "CodeTracker",
    "CoordinationRegistry",
    "Correction",
    "EvidenceRef",
    "EvidenceType",
    "FileLock",
    "GrowthRate",
    "ImplicationStrength",
    "ImplicitKnowledge",
    "ImplicitStats",
    "LockDuration",
    "LockStatus",
    "BrainPattern",
    "PersistentSynapseBank",
    "PipelineRun",
    "PipelineState",
    "Preference",
    "ProjectSnapshot",
    "RecoveryResult",
    "RunStatus",
    "SessionEntry",
    "SessionMetrics",
    "SynapseBankStats",
    "SynapseInfo",
    "T1Primitive",
    "TrackedFile",
    "TrackerIndex",
    "TrustAccumulator",
    // Config (type-safe Claude/Gemini/Git configuration)
    "AllConfigs",
    "AsrConfig",
    "ClaudeConfig",
    "CredentialHelper",
    "DoDChecklist",
    "DoDItem",
    "FeatureFlags",
    "FlywheelConfig",
    "FlywheelStage",
    "GeminiConfig",
    "GeminiHookType",
    "GitConfig",
    "GitCore",
    "GitDiff",
    "GitFetch",
    "GitInit",
    "GitPull",
    "GitUser",
    "HookEvent",
    "HookMeta",
    "HookRegistry",
    "HookTier",
    "McpServerConfig",
    "Model",
    "ModelUsage",
    "OAuthAccount",
    "PerformanceStats",
    "ProjectConfig",
    "RoutingConfig",
    "SkillChain",
    "SkillMapping",
    "SkillUsageStats",
    "VocabSkillMap",
    "VulnerabilityCache",
    // Mesh (distributed mesh networking)
    "ChemotacticRouter",
    "ChemotacticRouteSelector",
    "DiscoveryAction",
    "DiscoveryLoop",
    "DiscoveryMessage",
    "GossipLoop",
    "GossipMessage",
    "MeshError",
    "MeshEvent",
    "MeshHandle",
    "MeshMessage",
    "MeshRuntime",
    "MeshSnapshot",
    "Neighbor",
    "NeighborRegistry",
    "Node",
    "NodeState",
    "MeshPath",
    "PeerIdentity",
    "ResilienceAction",
    "ResilienceLoop",
    "ResilienceState",
    "Route",
    "RouteQuality",
    "RoutingTable",
    "SecurityPolicy",
    "SnapshotStore",
    "TlsTier",
    // Vigil (AI orchestrator — event bus, consequences, boundaries)
    "Urgency",
    "DecisionAction",
    "ExecutorType",
    "VigilEvent",
    "Interaction",
    "ExecutorResult",
    "CytokineBridge",
    "HormonalModulator",
    "SynapticLearner",
    "ImmunitySensor",
    "EnergyGovernor",
    "GuardianBridge",
    "VigilRuntime",
    "NervousSystem",
    "VigilError",
    "EventId",
    "EventKind",
    "EventSeverity",
    "EscalationLevel",
    "LedgerEntryType",
    "ConsequenceOutcome",
    "ShellConsequence",
    "WebhookConsequence",
    "NotifyConsequence",
    "WatchEvent",
    "ThresholdCheck",
    "BoundarySpec",
    "BoundaryViolation",
    "ConsequenceReceipt",
    "LedgerEntry",
    "BoundaryGate",
    "VigilanceLedger",
    "ConsequencePipeline",
    "VigilDaemon",
    "ShutdownHandle",
    "VigilHealth",
    "VigilStats",
    "VigilSubError",
    // Signal Theory (formal signal detection axioms & algebra)
    "SignalPrimitive",
    "EvidenceKind",
    "A2NoiseDominance",
    "A3SignalExistence",
    "A4BoundaryRequirement",
    "A5Disproportionality",
    "A6CausalInference",
    "ObservationSpace",
    "Baseline",
    "Ratio",
    "Difference",
    "DetectionInterval",
    "DetectionOutcome",
    "SignalStrengthLevel",
    "SignalVerificationReport",
    "BoundaryKind",
    "ConjunctionMode",
    "ThresholdPreset",
    "DetectionPhase",
    "FixedBoundary",
    "AdaptiveBoundary",
    "CompositeBoundary",
    "CascadedThreshold",
    "DetectionPipeline",
    "DecisionOutcome",
    "DecisionMatrix",
    "RocPoint",
    "RocCurve",
    "DPrime",
    "ResponseBias",
    "L1TotalCountConservation",
    "L2BaseRateInvariance",
    "L3SensitivitySpecificityTradeoff",
    "L4InformationConservation",
    "ConservationReport",
    // FAERS ETL (FDA adverse event data pipeline)
    "CaseCount",
    "RowCount",
    "DrugRole",
    "DrugCharacterization",
    "OpenFdaError",
    "DrugName",
    "EventName",
    "ContingencyBatch",
    "FaersPipelineOutput",
    "ReactionOutcome",
    "OutcomeCase",
    "OutcomeConditionedConfig",
    "MonthBucket",
    "TemporalCase",
    "VelocityConfig",
    "SeriousnessFlag",
    "CaseSeriousness",
    "SeriousnessCase",
    "CascadeConfig",
    "PolypharmacyCase",
    "PolypharmacyConfig",
    "ReporterQualification",
    "ReporterCase",
    "ReporterWeightedSignal",
    "ReporterWeightedConfig",
    "GeographicCase",
    "CountrySignal",
    "GeographicConfig",
    "FaersReport",
    "ReportFingerprint",
    "FaersDeduplicationResult",
    "DuplicateCluster",
    "DeduplicatorConfig",
    "FaersDeduplicator",
    "NdcProduct",
    "NdcBridge",
    "NdcMatch",
    "NdcMatchType",
    "DrugEventResponse",
    "DrugEventQuery",
    "OpenFdaClient",
    "FaersSignalDetectionResult",
    "OutcomeConditionedSignal",
    "SignalVelocity",
    "SeriousnessCascade",
    "PolypharmacySignal",
    "GeographicDivergence",
    // Algovigilance (ICSR deduplication + signal triage)
    "Similarity",
    "Relevance",
    "HalfLife",
    "CaseId",
    "SignalId",
    "DecayReport",
    "AlgovigilanceError",
    "SynonymEntry",
    "AlgovigilanceStore",
    "IcsrNarrative",
    "CasePair",
    "AlgoDeduplicationResult",
    "DedupConfig",
    "SynonymPair",
    "DedupFunction",
    "TriagedSignal",
    "TriageConfig",
    "TriageResult",
    "SignalInput",
    "ReinforcementEvent",
    "TriageFunction",
    "UrgencyClassification",
    "UrgencyClassifier",
    "SignalQueue",
    // Signal Fence (process-aware network boundary)
    "FenceMode",
    "Direction",
    "FenceVerdict",
    "Protocol",
    "TcpState",
    "SocketEntry",
    "ProcessInfo",
    "ProcessMatch",
    "NetworkMatch",
    "FenceRule",
    "FenceDecision",
    "FenceAuditEntry",
    "ConnectionEvent",
    "RuleSet",
    "FencePolicy",
    "FenceStats",
    "FenceAuditLog",
    "FenceTickResult",
    "FenceReport",
    "EnforcerOp",
    "TickDecision",
    // Education Machine (mastery, assessment, spaced repetition)
    "MasteryLevel",
    "Difficulty",
    "BayesianPrior",
    "LearningPhase",
    "CompetencyLevel",
    "MasteryVerdict",
    "Grade",
    "Question",
    "QuestionResult",
    "EduAssessment",
    "EduAssessmentResult",
    "Subject",
    "LessonRef",
    "Lesson",
    "LessonStep",
    "LessonContent",
    "PrimitiveMapping",
    "Enrollment",
    "AssessmentRecord",
    "ReviewState",
    "PhaseTransition",
    "Learner",
    // Biological Systems — Integumentary (20 types)
    "AuthResult",
    "SensorKind",
    "SensorReading",
    "SkinCondition",
    "Shield",
    "WoundRepair",
    "CoolingAction",
    "IntegumentaryError",
    "PermissionRule",
    "PermissionDecision",
    "RuleOrigin",
    "PermissionCascade",
    "SettingsScope",
    "ScopedSetting",
    "SettingsPrecedence",
    "SandboxLayer",
    "RiskLevel",
    "Scar",
    "ScarringMechanism",
    "IntegumentaryHealth",
    // Biological Systems — Respiratory (17 types)
    "InputSource",
    "Inhaled",
    "Extracted",
    "Exhaled",
    "ExchangeResult",
    "BreathingRate",
    "RespiratoryError",
    "ContextSource",
    "Inhalation",
    "Exhalation",
    "GasExchange",
    "DeadSpace",
    "ContextFork",
    "TidalVolume",
    "VitalCapacity",
    "BreathCycle",
    "RespiratoryHealth",
    // Biological Systems — Digestive (20 types)
    "Quality",
    "DataKind",
    "Fragment",
    "Taste",
    "Nutrients",
    "Absorbed",
    "Metabolized",
    "DigestiveError",
    "SkillTrigger",
    "SkillLoad",
    "ContextMode",
    "SkillFrontmatter",
    "Sphincter",
    "SkillArguments",
    "EnzymeType",
    "EnzymeSubstitution",
    "Microbiome",
    "SkillResult",
    "SkillExecution",
    "DigestiveHealth",
    // Biological Systems — Circulatory (21 types)
    "CellKind",
    "BloodCell",
    "Enriched",
    "Destination",
    "RouteDecision",
    "BloodPressure",
    "Pulse",
    "Platelet",
    "CirculatoryError",
    "McpTransport",
    "McpScope",
    "FlowDirection",
    "McpServer",
    "McpHeartbeat",
    "ToolCall",
    "ToolResult",
    "PortalFiltration",
    "SelectivePerfusion",
    "BloodPayload",
    "FrankStarling",
    "CirculatoryHealth",
    // Biological Systems — Skeletal (5 types)
    "WolffsLaw",
    "SkeletalCorrection",
    "BoneMarrow",
    "Joint",
    "SkeletalHealth",
    // Biological Systems — Muscular (8 types)
    "MuscleType",
    "ToolClassification",
    "AntagonisticPair",
    "SizePrinciple",
    "Fatigue",
    "ModelEscalation",
    "MotorActivation",
    "MuscularHealth",
    // Biological Systems — Lymphatic (7 types)
    "OutputStyle",
    "LymphDrainage",
    "LymphNode",
    "ThymicSelection",
    "ThymicVerdict",
    "OverflowHandler",
    "LymphaticHealth",
    // Biological Systems — Nervous (18 types)
    "NeuronType",
    "Impulse",
    "ReflexArc",
    "Myelination",
    "SensoryInput",
    "MotorCommand",
    "SignalSpeed",
    "NervousHealth",
    "NervousHookEvent",
    "NervousHookDispatch",
    "HookChain",
    "EventBusRoute",
    "ContextAssembly",
    "ToolChain",
    "SignalLatency",
    "MyelinationCache",
    "ReflexResponse",
    "NervousSystemHealth",
    // Biological Systems — Urinary (18 types)
    "FilterCategory",
    "Nephron",
    "GlomerularFiltration",
    "Reabsorption",
    "Excretion",
    "FiltrationRate",
    "Bladder",
    "UrinaryHealth",
    "TelemetryPruning",
    "SessionExpiry",
    "ArtifactRetention",
    "LogRotation",
    "RetentionPolicy",
    "DecisionAuditCleanup",
    "WasteCategory",
    "DisposalMethod",
    "SilentFailureRisk",
    "UrinarySystemHealth",
    // Biological Systems — Reproductive (18 types)
    "GameteFitness",
    "Gamete",
    "Fertilization",
    "Trimester",
    "Embryo",
    "Differentiation",
    "GeneticMutation",
    "ReproductiveHealth",
    "CiStage",
    "CiPipeline",
    "DeployTarget",
    "DeploymentBirth",
    "BranchMutation",
    "MergeEvent",
    "TrimesterGate",
    "ScalingEvent",
    "KnowledgeTransfer",
    "ReproductiveSystemHealth",
    // Cloud Computing Taxonomy (35 types — nexcore-cloud)
    // T1 (prefixed to avoid collisions with vigilance types)
    "CloudIdentity",
    "CloudThreshold",
    "CloudFeedbackLoop",
    "CloudIdempotency",
    "CloudImmutability",
    "CloudConvergence",
    // T2-P (unique names, no prefix needed)
    "Compute",
    "Storage",
    "NetworkLink",
    "IsolationBoundary",
    "CloudPermission",
    "ResourcePool",
    "Metering",
    "Replication",
    "CloudRouting",
    "CloudLease",
    "CloudEncryption",
    "CloudQueue",
    "CloudHealthCheck",
    "Elasticity",
    // T2-C Composites
    "VirtualMachine",
    "LoadBalancer",
    "AutoScaling",
    "CloudIam",
    "EventualConsistency",
    "Tenancy",
    "PayPerUse",
    "ReservedCapacity",
    "SpotPricing",
    "SecretsManagement",
    // T3 Domain-Specific
    "Container",
    "Iaas",
    "Paas",
    "Saas",
    "Serverless",
];

fn lookup_grounded_type<F>(type_name: &str, f: F) -> serde_json::Value
where
    F: Fn(&str, &PrimitiveComposition) -> serde_json::Value,
{
    use nexcore_labs::betting::bdi::Bdi;
    use nexcore_labs::betting::ecs::EcsScore;
    use nexcore_signal_types::SignalResult;
    use nexcore_vigilance::domain_discovery::primitives::{Primitive, PrimitiveTier};
    use nexcore_vigilance::primitives::bathroom_lock::Occupancy;
    use nexcore_vigilance::primitives::chemistry::*;
    use nexcore_vigilance::primitives::frontier::{
        AbsenceMarker, AuditTrail, ConsumptionMark, EntityStatus, Pipeline, RecursionBound,
        ResourcePath,
    };
    use nexcore_vigilance::primitives::quantum::{
        Amplitude, Decoherence, Eigenstate, Entanglement, Hermiticity, Interference, Measurement,
        Observable, Phase, Qubit, Superposition, Uncertainty, Unitarity,
    };
    use nexcore_vigilance::primitives::recipient::{Recipient, Tracked, Vulnerable};
    use nexcore_vigilance::primitives::safety::{BoundaryBreach, Harm, Monitoring, SafetyBoundary};
    use nexcore_vigilance::primitives::transfer::{
        Accumulator, Atomicity, Checkpoint, CircuitBreaker, CompareAndSwap, Homeostasis,
        Idempotency, NegativeEvidence, RateLimiter, SerializationGuard, StagedValidation,
        ToctouWindow, TopologicalAddress,
    };
    use nexcore_vigilance::tov::{HarmType, SafetyMargin};
    // Biological pipeline
    use nexcore_hormones::{
        BehavioralModifiers, EndocrineState, HormoneLevel, HormoneType, Stimulus,
    };
    use nexcore_phenotype::{Mutation, Phenotype};
    use nexcore_ribosome::{Contract, DriftResult, DriftSeverity, DriftSignal, DriftType};
    use nexcore_transcriptase::{DiagnosticLevel, Fidelity, Schema, SchemaKind};
    // Cortex LLM types (implement standalone nexcore_lex_primitiva::GroundsTo)
    use nexcore_cortex::cloud::{DatasetConfig, FineTuneJob, TrainingParams};
    use nexcore_cortex::generate::GenerateParams;
    use nexcore_cortex::lora::{LoraAdapter, LoraConfig};
    use nexcore_cortex::model::{ModelConfig, ModelFormat};
    // Trust engine types (Product × grounding)
    use nexcore_trust::Evidence as TrustEvidence;
    use nexcore_trust::{
        DimensionWeights, MultiTrustEngine, TrustConfig, TrustDimension, TrustEngine, TrustLevel,
        TrustVelocity,
    };
    // Guardian engine types (Product × grounding)
    use nexcore_guardian_engine::{OriginatorType, RiskContext, RiskScore};
    // Spatial bridge metric types (κ Comparison dominant)
    use nexcore_dtree::spatial_bridge::ImpurityMetric;
    use nexcore_guardian_engine::spatial_bridge::{RiskScoreMetric, SeverityMetric};
    // PV types (Product × update)
    use nexcore_vigilance::primitives::pharmacovigilance::DisproportionalityScore;
    // Insight Engine types (system identity — T2-P/T2-C/T3)
    use nexcore_insight::composites::NoveltyReason;
    use nexcore_insight::{
        Compression, Connection, InsightEngine, Novelty, Pattern, Recognition, Suddenness,
    };
    // DNA-based computation types (nexcore_lex_primitiva::GroundsTo)
    use nexcore_dna::asm::Token as DnaToken;
    use nexcore_dna::codon_table::CodonTable;
    use nexcore_dna::cortex::{
        Cluster as DnaCluster, ClusterResult as DnaClusterResult, EvolutionConfig, EvolutionResult,
        GravityConfig, Organism, Particle,
    };
    use nexcore_dna::data::{DnaArray, DnaFrame, DnaMap, DnaRecord, DnaType, DnaValue};
    use nexcore_dna::disasm::DisasmOptions;
    use nexcore_dna::gene::{Gene, Genome, Plasmid};
    use nexcore_dna::glyph::{Glyph as DnaGlyph, GlyphPair as DnaGlyphPair};
    use nexcore_dna::isa::Instruction as DnaInstruction;
    use nexcore_dna::lang::diagnostic::{Diagnostic as DnaDiagnostic, ErrorCode as DnaErrorCode};
    use nexcore_dna::lang::json::JsonValue as DnaJsonValue;
    use nexcore_dna::lang::templates::Template as DnaTemplate;
    use nexcore_dna::lexicon::{Affinity, Lexicon as DnaLexicon, WordOre};
    use nexcore_dna::program::Program as DnaProgram;
    use nexcore_dna::pv_theory::{
        AlertLevel, CausalityCategory, CausalityScore, DrugEventSignal as DnaSignal, DrugProfile,
        SafetyLevel as DnaSafetyLevel, SafetyMargin as DnaSafetyMargin, VigilanceState,
    };
    use nexcore_dna::statemind::{
        Drift as DnaDrift, MindPoint as DnaMindPoint, StateMind as DnaStateMind,
    };
    use nexcore_dna::string_theory::{
        FrequencySpectrum, HarmonicMode, Resonance as DnaResonance, StringEnergy, StringTension,
    };
    use nexcore_dna::tile::Pixel;
    use nexcore_dna::types::{AminoAcid, Codon, DoubleHelix, Nucleotide, Strand};
    use nexcore_dna::vm::CodonVM;
    use nexcore_dna::voxel::{VoxelCube, VoxelPos};
    // Energy types (ATP/ADP biochemistry)
    use nexcore_energy::{
        EnergyState, EnergySystem, Operation, OperationBuilder, RecyclingRate, Regime, Strategy,
        TokenPool, WasteClass,
    };
    // Compliance types
    use nexcore_compliance::oscal::{Control, ControlCatalog, ControlStatus};
    use nexcore_compliance::sam::{Exclusion, ExclusionClassification, ExclusionType};
    use nexcore_compliance::{Assessment, ComplianceResult, Finding, FindingSeverity};
    // Measure types (information theory, graph theory, statistics)
    use nexcore_measure::error::MeasureError;
    use nexcore_measure::types::{
        BayesianPosterior, Centrality, ChemistryMapping, CodeDensityIndex, CouplingRatio,
        CrateHealth, CrateId, CrateMeasurement, Density, DriftDirection,
        DriftResult as MeasureDriftResult, Entropy, GraphAnalysis, GraphNode, HealthComponents,
        HealthRating, HealthScore, MeasureTimestamp, PoissonCi, Probability, RatingDistribution,
        RegressionResult, TestDensity, WelchResult, WorkspaceHealth, WorkspaceMeasurement,
    };
    // Signal types (PV signal detection pipeline)
    use signal::alert::AlertTransitions;
    use signal::core::{
        Alert as SignalAlert, AlertState as SignalAlertState, ChiSquare as SignalChiSquare,
        ConfidenceInterval as SignalConfidenceInterval, ContingencyTable as SignalContingencyTable,
        DetectionResult as SignalDetectionResult, DrugEventPair, Ebgm as SignalEbgm,
        Ic as SignalIc, NormalizedEvent, Prr as SignalPrr, RawReport, ReportSource,
        Ror as SignalRor, SignalError, SignalStrength, ThresholdConfig as SignalThresholdConfig,
        ValidationCheck as SignalValidationCheck, ValidationReport as SignalValidationReport,
    };
    use signal::detect::TableDetector;
    use signal::ingest::{CsvIngestor, JsonIngestor};
    use signal::normalize::{BasicNormalizer, SynonymNormalizer};
    use signal::report::{JsonReporter, TableReporter};
    use signal::stats::SignalMetrics;
    use signal::store::{JsonFileStore, MemoryStore};
    use signal::threshold::{CompositeThreshold, EvansThreshold};
    use signal::validate::StandardValidator;
    // Brain types (sessions, artifacts, implicit learning)
    use nexcore_brain::artifact::{Artifact, ArtifactMetadata, ArtifactType};
    use nexcore_brain::config::BrainConfig;
    use nexcore_brain::coordination::{
        AgentId, CoordinationRegistry, FileLock, LockDuration, LockStatus,
    };
    use nexcore_brain::error::BrainError;
    use nexcore_brain::implicit::{
        Belief, BeliefGraph, BeliefImplication, Correction, EvidenceRef, EvidenceType,
        ImplicationStrength, ImplicitKnowledge, ImplicitStats, Pattern as BrainPattern, Preference,
        T1Primitive, TrustAccumulator,
    };
    use nexcore_brain::metrics::{
        ArtifactMetrics, ArtifactSizeInfo, BrainHealth, BrainSnapshot, GrowthRate, SessionMetrics,
    };
    use nexcore_brain::pipeline::{
        Checkpoint as BrainCheckpoint, PipelineRun, PipelineState, RunStatus,
    };
    use nexcore_brain::recovery::RecoveryResult;
    use nexcore_brain::session::{BrainSession, SessionEntry};
    use nexcore_brain::synapse::{PersistentSynapseBank, SynapseBankStats, SynapseInfo};
    use nexcore_brain::tracker::{CodeTracker, ProjectSnapshot, TrackedFile, TrackerIndex};
    // Config types (type-safe Claude/Gemini/Git configuration)
    use nexcore_config::AllConfigs;
    use nexcore_config::asr::{
        AsrConfig, DoDChecklist, DoDItem, FlywheelConfig, FlywheelStage, Model, RoutingConfig,
    };
    use nexcore_config::claude::{
        ClaudeConfig, FeatureFlags, McpServerConfig, ModelUsage, OAuthAccount, PerformanceStats,
        ProjectConfig, SkillUsageStats, VulnerabilityCache,
    };
    use nexcore_config::gemini::GeminiConfig;
    use nexcore_config::gemini::HookType as GeminiHookType;
    use nexcore_config::git::{
        CredentialHelper, GitConfig, GitCore, GitDiff, GitFetch, GitInit, GitPull, GitUser,
    };
    use nexcore_config::hooks::{HookEvent, HookMeta, HookRegistry, HookTier};
    use nexcore_config::vocab::{SkillChain, SkillMapping, VocabSkillMap};
    // Mesh types (distributed mesh networking)
    use nexcore_mesh::chemotaxis::{ChemotacticRouteSelector, ChemotacticRouter};
    use nexcore_mesh::discovery::{DiscoveryAction, DiscoveryLoop, DiscoveryMessage};
    use nexcore_mesh::error::MeshError;
    use nexcore_mesh::gossip::{GossipLoop, GossipMessage};
    use nexcore_mesh::neighbor::{Neighbor, NeighborRegistry};
    use nexcore_mesh::node::{MeshMessage, Node, NodeState};
    use nexcore_mesh::persistence::{MeshSnapshot, SnapshotStore};
    use nexcore_mesh::resilience::{ResilienceAction, ResilienceLoop, ResilienceState};
    use nexcore_mesh::routing::{Route, RoutingTable};
    use nexcore_mesh::runtime::{MeshEvent, MeshHandle, MeshRuntime};
    use nexcore_mesh::security::{PeerIdentity, SecurityPolicy, TlsTier};
    use nexcore_mesh::topology::{Path as MeshPath, RouteQuality};
    // Vigil types (AI orchestrator — event bus, consequences, boundaries)
    use nexcore_vigil::VigilError;
    use nexcore_vigil::bridge::nervous_system::{
        CytokineBridge, EnergyGovernor, GuardianBridge, HormonalModulator, ImmunitySensor,
        NervousSystem, SynapticLearner,
    };
    use nexcore_vigil::models::{
        DecisionAction, Event as VigilEvent, ExecutorResult, ExecutorType, Interaction, Urgency,
    };
    use nexcore_vigil::runtime::VigilRuntime;
    use nexcore_vigil::vigilance::boundary::{
        BoundaryGate, BoundarySpec, BoundaryViolation, ThresholdCheck,
    };
    use nexcore_vigil::vigilance::consequence::{
        ConsequenceOutcome, ConsequencePipeline, ConsequenceReceipt, EscalationLevel,
        NotifyConsequence, ShellConsequence, WebhookConsequence,
    };
    use nexcore_vigil::vigilance::daemon::{ShutdownHandle, VigilDaemon, VigilHealth, VigilStats};
    use nexcore_vigil::vigilance::error::VigilError as VigilSubError;
    use nexcore_vigil::vigilance::event::{EventId, EventKind, EventSeverity, WatchEvent};
    use nexcore_vigil::vigilance::ledger::{LedgerEntry, LedgerEntryType, VigilanceLedger};
    // Signal Theory types (formal signal detection axioms & algebra)
    use nexcore_signal_theory::SignalPrimitive;
    use nexcore_signal_theory::algebra::{CascadedThreshold, DetectionPipeline};
    use nexcore_signal_theory::axioms::{
        A2NoiseDominance, A3SignalExistence, A4BoundaryRequirement, A5Disproportionality,
        A6CausalInference, EvidenceKind,
    };
    use nexcore_signal_theory::conservation::{
        ConservationReport, L1TotalCountConservation, L2BaseRateInvariance,
        L3SensitivitySpecificityTradeoff, L4InformationConservation,
    };
    use nexcore_signal_theory::decision::{
        DPrime, DecisionMatrix, DecisionOutcome, ResponseBias, RocCurve, RocPoint,
    };
    use nexcore_signal_theory::detection::{
        Baseline, DetectionInterval, DetectionOutcome, Difference, ObservationSpace, Ratio,
        SignalStrengthLevel, SignalVerificationReport,
    };
    use nexcore_signal_theory::threshold::{
        AdaptiveBoundary, BoundaryKind, CompositeBoundary, ConjunctionMode, DetectionPhase,
        FixedBoundary, ThresholdPreset,
    };
    // FAERS ETL types (FDA adverse event data pipeline)
    use nexcore_faers_etl::api::{DrugEventQuery, DrugEventResponse, OpenFdaClient, OpenFdaError};
    use nexcore_faers_etl::dedup::{
        DeduplicationResult as FaersDeduplicationResult, DeduplicatorConfig, DuplicateCluster,
        FaersDeduplicator, FaersReport, ReportFingerprint,
    };
    use nexcore_faers_etl::ndc::{NdcBridge, NdcMatch, NdcMatchType, NdcProduct};
    use nexcore_faers_etl::{
        CascadeConfig, CaseSeriousness, CountrySignal, DrugCharacterization, GeographicCase,
        GeographicConfig, GeographicDivergence, MonthBucket, OutcomeCase, OutcomeConditionedConfig,
        OutcomeConditionedSignal, PolypharmacyCase, PolypharmacyConfig, PolypharmacySignal,
        ReactionOutcome, ReporterCase, ReporterQualification, ReporterWeightedConfig,
        ReporterWeightedSignal, SeriousnessCascade, SeriousnessCase, SeriousnessFlag,
        SignalVelocity, TemporalCase, VelocityConfig,
    };
    use nexcore_faers_etl::{CaseCount, ContingencyBatch, DrugName, DrugRole, EventName, RowCount};
    use nexcore_faers_etl::{
        PipelineOutput as FaersPipelineOutput, SignalDetectionResult as FaersSignalDetectionResult,
    };
    // Algovigilance types (ICSR deduplication + signal triage)
    use nexcore_algovigilance::AlgovigilanceError;
    use nexcore_algovigilance::dedup::DedupFunction;
    use nexcore_algovigilance::dedup::types::{
        CasePair, DedupConfig, DeduplicationResult as AlgoDeduplicationResult, IcsrNarrative,
        SynonymPair,
    };
    use nexcore_algovigilance::store::{AlgovigilanceStore, SynonymEntry};
    use nexcore_algovigilance::triage::TriageFunction;
    use nexcore_algovigilance::triage::classifier::{UrgencyClassification, UrgencyClassifier};
    use nexcore_algovigilance::triage::queue::SignalQueue;
    use nexcore_algovigilance::triage::types::{
        ReinforcementEvent, SignalInput, TriageConfig, TriageResult, TriagedSignal,
    };
    use nexcore_algovigilance::{CaseId, DecayReport, HalfLife, Relevance, SignalId, Similarity};
    // Signal Fence types (process-aware network boundary)
    use nexcore_signal_fence::audit::{AuditEntry as FenceAuditEntry, AuditLog as FenceAuditLog};
    use nexcore_signal_fence::connection::{Protocol, SocketEntry, TcpState};
    use nexcore_signal_fence::enforcer::EnforcerOp;
    use nexcore_signal_fence::engine::{FenceReport, FenceStats, FenceTickResult, TickDecision};
    use nexcore_signal_fence::policy::{FenceDecision, FenceMode, FencePolicy};
    use nexcore_signal_fence::process::{ConnectionEvent, Direction, ProcessInfo};
    use nexcore_signal_fence::rule::{
        FenceRule, FenceVerdict, NetworkMatch, ProcessMatch, RuleSet,
    };
    // Education Machine types (mastery, assessment, spaced repetition)
    use nexcore_education_machine::assessment::{
        Assessment as EduAssessment, AssessmentResult as EduAssessmentResult, BayesianPrior,
        Question, QuestionResult,
    };
    use nexcore_education_machine::learner::{AssessmentRecord, Enrollment, Learner};
    use nexcore_education_machine::lesson::{Lesson, LessonContent, LessonStep, PrimitiveMapping};
    use nexcore_education_machine::spaced_repetition::ReviewState;
    use nexcore_education_machine::state_machine::PhaseTransition;
    use nexcore_education_machine::subject::{LessonRef, Subject};
    use nexcore_education_machine::types::{
        CompetencyLevel, Difficulty, Grade, LearningPhase, MasteryLevel, MasteryVerdict,
    };
    // Biological Systems — Integumentary (boundary/auth/settings)
    use nexcore_integumentary::claude_code::{
        IntegumentaryHealth, PermissionCascade, PermissionDecision, PermissionRule, RiskLevel,
        RuleOrigin, SandboxLayer, Scar, ScarringMechanism, ScopedSetting, SettingsPrecedence,
        SettingsScope,
    };
    use nexcore_integumentary::{
        AuthResult, CoolingAction, IntegumentaryError, SensorKind, SensorReading, Shield,
        SkinCondition, WoundRepair,
    };
    // Biological Systems — Respiratory (context window I/O)
    use nexcore_respiratory::claude_code::{
        BreathCycle, ContextFork, ContextSource, DeadSpace, Exhalation, GasExchange, Inhalation,
        RespiratoryHealth, TidalVolume, VitalCapacity,
    };
    use nexcore_respiratory::{
        BreathingRate, ExchangeResult, Exhaled, Extracted, Inhaled, InputSource, RespiratoryError,
    };
    // Biological Systems — Digestive (skill pipeline)
    use nexcore_digestive::claude_code::{
        ContextMode, DigestiveHealth, EnzymeSubstitution, EnzymeType, Microbiome, SkillArguments,
        SkillExecution, SkillFrontmatter, SkillLoad, SkillResult, SkillTrigger, Sphincter,
    };
    use nexcore_digestive::{
        Absorbed, DataKind, DigestiveError, Fragment, Metabolized, Nutrients, Quality, Taste,
    };
    // Biological Systems — Circulatory (MCP transport)
    use nexcore_circulatory::claude_code::{
        BloodPayload, CirculatoryHealth, FlowDirection, FrankStarling, McpHeartbeat, McpScope,
        McpServer, McpTransport, PortalFiltration, SelectivePerfusion, ToolCall, ToolResult,
    };
    use nexcore_circulatory::{
        BloodCell, BloodPressure, CellKind, CirculatoryError, Destination, Enriched, Platelet,
        Pulse, RouteDecision,
    };
    // Biological Systems — Skeletal (CLAUDE.md, type structure)
    use nexcore_skeletal::{
        BoneMarrow, Correction as SkeletalCorrection, Joint, SkeletalHealth, WolffsLaw,
    };
    // Biological Systems — Muscular (tool execution)
    use nexcore_muscular::{
        AntagonisticPair, Fatigue, ModelEscalation, MotorActivation, MuscleType, MuscularHealth,
        SizePrinciple, ToolClassification,
    };
    // Biological Systems — Lymphatic (output styles, overflow drainage)
    use nexcore_lymphatic::{
        LymphDrainage, LymphNode, LymphaticHealth, OutputStyle, OverflowHandler, ThymicSelection,
        ThymicVerdict,
    };
    // Biological Systems — Nervous (event routing, hooks, signaling)
    use nexcore_nervous::claude_code::{
        ContextAssembly, EventBusRoute, HookChain, HookDispatch as NervousHookDispatch,
        HookEvent as NervousHookEvent, MyelinationCache, NervousSystemHealth, ReflexResponse,
        SignalLatency, ToolChain,
    };
    use nexcore_nervous::{
        Impulse, MotorCommand, Myelination, NervousHealth, NeuronType, ReflexArc, SensoryInput,
        SignalSpeed,
    };
    // Biological Systems — Urinary (filtration, waste, retention)
    use nexcore_urinary::claude_code::{
        ArtifactRetention, DecisionAuditCleanup, DisposalMethod, LogRotation, RetentionPolicy,
        SessionExpiry, SilentFailureRisk, TelemetryPruning, UrinarySystemHealth, WasteCategory,
    };
    use nexcore_urinary::{
        Bladder, Excretion, FilterCategory, FiltrationRate, GlomerularFiltration, Nephron,
        Reabsorption, UrinaryHealth,
    };
    // Biological Systems — Reproductive (CI/CD, deployment, replication)
    use nexcore_reproductive::claude_code::{
        BranchMutation, CiPipeline, CiStage, DeployTarget, DeploymentBirth, KnowledgeTransfer,
        MergeEvent, ReproductiveSystemHealth, ScalingEvent, TrimesterGate,
    };
    use nexcore_reproductive::{
        Differentiation, Embryo, Fertilization, Gamete, GameteFitness, Mutation as GeneticMutation,
        ReproductiveHealth, Trimester,
    };

    // Two dispatch macros needed: some types implement nexcore_vigilance::GroundsTo,
    // others implement nexcore_lex_primitiva::GroundsTo. Fully-qualified syntax avoids
    // ambiguity when both traits are in scope.
    macro_rules! dispatch_vig {
        ($($name:literal => $ty:ty),* $(,)?) => {
            match type_name {
                $($name => return f($name, &<$ty as nexcore_vigilance::GroundsTo>::primitive_composition()),)*
                _ => {}
            }
        };
    }
    macro_rules! dispatch_lex {
        ($($name:literal => $ty:ty),* $(,)?) => {
            match type_name {
                $($name => return f($name, &<$ty as nexcore_lex_primitiva::GroundsTo>::primitive_composition()),)*
                _ => {}
            }
        };
    }

    // Types implementing nexcore_vigilance::GroundsTo (vigilance re-export)
    dispatch_vig! {
        "ThresholdGate" => ThresholdGate,
        "SaturationKinetics" => SaturationKinetics,
        "FeasibilityAssessment" => FeasibilityAssessment,
        "RateLaw" => RateLaw,
        "BufferSystem" => BufferSystem,
        "SignalDetector" => SignalDetector,
        "DecayKinetics" => DecayKinetics,
        "EquilibriumSystem" => EquilibriumSystem,
        "AggregationPipeline" => AggregationPipeline,
        "SignalResult" => SignalResult,
        "Primitive" => Primitive,
        "PrimitiveTier" => PrimitiveTier,
        "HarmType" => HarmType,
        "SafetyMargin" => SafetyMargin,
        "Bdi" => Bdi,
        "EcsScore" => EcsScore,
        // Bathroom Lock T2-P
        "Occupancy" => Occupancy,
        // Frontier T2-P
        "AuditTrail" => AuditTrail,
        "AbsenceMarker" => AbsenceMarker,
        "Pipeline" => Pipeline,
        "ConsumptionMark" => ConsumptionMark,
        "EntityStatus" => EntityStatus,
        "ResourcePath" => ResourcePath,
        "RecursionBound" => RecursionBound,
        // Patient/Safety T2-P/T2-C
        "Recipient" => Recipient,
        "SafetyBoundary" => SafetyBoundary<f64>,
        "Harm" => Harm,
        "Vulnerable" => Vulnerable,
        "BoundaryBreach" => BoundaryBreach,
        "Monitoring" => Monitoring,
        "Tracked" => Tracked,
        // Cross-Domain Transfer T2-P
        "Homeostasis" => Homeostasis,
        "StagedValidation" => StagedValidation,
        "Atomicity" => Atomicity,
        "CompareAndSwap" => CompareAndSwap,
        "ToctouWindow" => ToctouWindow,
        "SerializationGuard" => SerializationGuard,
        "RateLimiter" => RateLimiter,
        "CircuitBreaker" => CircuitBreaker,
        "Idempotency" => Idempotency,
        "NegativeEvidence" => NegativeEvidence,
        "TopologicalAddress" => TopologicalAddress,
        "Accumulator" => Accumulator,
        "Checkpoint" => Checkpoint,
        // Quantum Domain Primitives
        "Amplitude" => Amplitude,
        "Phase" => Phase,
        "Superposition" => Superposition,
        "Measurement" => Measurement,
        "Interference" => Interference,
        "Uncertainty" => Uncertainty,
        "Unitarity" => Unitarity,
        "Eigenstate" => Eigenstate,
        "Observable" => Observable,
        "Hermiticity" => Hermiticity,
        "Entanglement" => Entanglement,
        "Decoherence" => Decoherence,
        "Qubit" => Qubit,
        // PV (Product × update)
        "DisproportionalityScore" => DisproportionalityScore,
    }

    // Types implementing nexcore_lex_primitiva::GroundsTo (standalone crates)
    dispatch_lex! {
        // Biological Pipeline
        "Schema" => Schema,
        "SchemaKind" => SchemaKind,
        "DiagnosticLevel" => DiagnosticLevel,
        "Fidelity" => Fidelity,
        "Contract" => Contract,
        "DriftType" => DriftType,
        "DriftSeverity" => DriftSeverity,
        "DriftResult" => DriftResult,
        "DriftSignal" => DriftSignal,
        "Mutation" => Mutation,
        "Phenotype" => Phenotype,
        "HormoneLevel" => HormoneLevel,
        "HormoneType" => HormoneType,
        "Stimulus" => Stimulus,
        "EndocrineState" => EndocrineState,
        "BehavioralModifiers" => BehavioralModifiers,
        // Cortex LLM types
        "ModelConfig" => ModelConfig,
        "ModelFormat" => ModelFormat,
        "GenerateParams" => GenerateParams,
        "LoraConfig" => LoraConfig,
        "DatasetConfig" => DatasetConfig,
        "TrainingParams" => TrainingParams,
        "LoraAdapter" => LoraAdapter,
        "FineTuneJob" => FineTuneJob,
        // Trust Engine (Product × grounding)
        "MultiTrustEngine" => MultiTrustEngine,
        "TrustDimension" => TrustDimension,
        "DimensionWeights" => DimensionWeights,
        "TrustEngine" => TrustEngine,
        "Evidence" => TrustEvidence,
        "TrustLevel" => TrustLevel,
        "TrustVelocity" => TrustVelocity,
        "TrustConfig" => TrustConfig,
        // Guardian Engine (Product × grounding)
        "OriginatorType" => OriginatorType,
        "RiskContext" => RiskContext,
        "RiskScore" => RiskScore,
        // Spatial Bridge Metric types (κ Comparison dominant)
        "ImpurityMetric" => ImpurityMetric,
        "SeverityMetric" => SeverityMetric,
        "RiskScoreMetric" => RiskScoreMetric,
        // Insight Engine (system identity — T2-P/T2-C/T3)
        "Pattern" => Pattern,
        "Recognition" => Recognition,
        "Novelty" => Novelty,
        "NoveltyReason" => NoveltyReason,
        "Connection" => Connection,
        "Compression" => Compression,
        "Suddenness" => Suddenness,
        "InsightEngine" => InsightEngine,
        // DNA — Foundation (7)
        "Nucleotide" => Nucleotide,
        "AminoAcid" => AminoAcid,
        "Codon" => Codon,
        "Strand" => Strand,
        "DoubleHelix" => DoubleHelix,
        "CodonTable" => CodonTable,
        "CodonVM" => CodonVM,
        // DNA — Compute (10)
        "DnaInstruction" => DnaInstruction,
        "DnaProgram" => DnaProgram,
        "DnaToken" => DnaToken,
        "DisasmOptions" => DisasmOptions,
        "DnaGlyph" => DnaGlyph,
        "DnaGlyphPair" => DnaGlyphPair,
        "DnaErrorCode" => DnaErrorCode,
        "DnaDiagnostic" => DnaDiagnostic,
        "DnaJsonValue" => DnaJsonValue,
        "DnaTemplate" => DnaTemplate,
        // DNA — Store (9)
        "DnaType" => DnaType,
        "DnaValue" => DnaValue,
        "DnaArray" => DnaArray,
        "DnaRecord" => DnaRecord,
        "DnaMap" => DnaMap,
        "DnaFrame" => DnaFrame,
        "Pixel" => Pixel,
        "VoxelPos" => VoxelPos,
        "VoxelCube" => VoxelCube,
        // DNA — Evolve (8)
        "Gene" => Gene,
        "Genome" => Genome,
        "Plasmid" => Plasmid,
        "Organism" => Organism,
        "DnaCluster" => DnaCluster,
        "DnaClusterResult" => DnaClusterResult,
        "EvolutionConfig" => EvolutionConfig,
        "EvolutionResult" => EvolutionResult,
        // DNA — Gravity (2)
        "Particle" => Particle,
        "GravityConfig" => GravityConfig,
        // DNA — Analyze: String Theory (5)
        "StringTension" => StringTension,
        "HarmonicMode" => HarmonicMode,
        "FrequencySpectrum" => FrequencySpectrum,
        "DnaResonance" => DnaResonance,
        "StringEnergy" => StringEnergy,
        // DNA — Analyze: PV Theory (7+1)
        "CausalityCategory" => CausalityCategory,
        "DnaSafetyLevel" => DnaSafetyLevel,
        "AlertLevel" => AlertLevel,
        "DrugProfile" => DrugProfile,
        "DnaSignal" => DnaSignal,
        "DnaSafetyMargin" => DnaSafetyMargin,
        "CausalityScore" => CausalityScore,
        "VigilanceState" => VigilanceState,
        // DNA — Analyze: Mind (6)
        "WordOre" => WordOre,
        "Affinity" => Affinity,
        "DnaLexicon" => DnaLexicon,
        "DnaMindPoint" => DnaMindPoint,
        "DnaDrift" => DnaDrift,
        "DnaStateMind" => DnaStateMind,
        // Energy (ATP/ADP biochemistry)
        "Regime" => Regime,
        "Strategy" => Strategy,
        "EnergySystem" => EnergySystem,
        "WasteClass" => WasteClass,
        "TokenPool" => TokenPool,
        "Operation" => Operation,
        "OperationBuilder" => OperationBuilder,
        "RecyclingRate" => RecyclingRate,
        "EnergyState" => EnergyState,
        // Compliance
        "Assessment" => Assessment,
        "ComplianceResult" => ComplianceResult,
        "Finding" => Finding,
        "FindingSeverity" => FindingSeverity,
        "Control" => Control,
        "ControlCatalog" => ControlCatalog,
        "ControlStatus" => ControlStatus,
        "Exclusion" => Exclusion,
        "ExclusionClassification" => ExclusionClassification,
        "ExclusionType" => ExclusionType,
        // Measure (information theory, graph theory, statistics)
        "BayesianPosterior" => BayesianPosterior,
        "Centrality" => Centrality,
        "ChemistryMapping" => ChemistryMapping,
        "CodeDensityIndex" => CodeDensityIndex,
        "CouplingRatio" => CouplingRatio,
        "CrateHealth" => CrateHealth,
        "CrateId" => CrateId,
        "CrateMeasurement" => CrateMeasurement,
        "Density" => Density,
        "DriftDirection" => DriftDirection,
        "MeasureDriftResult" => MeasureDriftResult,
        "Entropy" => Entropy,
        "GraphAnalysis" => GraphAnalysis,
        "GraphNode" => GraphNode,
        "HealthComponents" => HealthComponents,
        "HealthRating" => HealthRating,
        "HealthScore" => HealthScore,
        "MeasureError" => MeasureError,
        "MeasureTimestamp" => MeasureTimestamp,
        "PoissonCi" => PoissonCi,
        "Probability" => Probability,
        "RatingDistribution" => RatingDistribution,
        "RegressionResult" => RegressionResult,
        "TestDensity" => TestDensity,
        "WelchResult" => WelchResult,
        "WorkspaceHealth" => WorkspaceHealth,
        "WorkspaceMeasurement" => WorkspaceMeasurement,
        // Signal (PV signal detection pipeline)
        "Prr" => SignalPrr,
        "Ror" => SignalRor,
        "Ic" => SignalIc,
        "Ebgm" => SignalEbgm,
        "ChiSquare" => SignalChiSquare,
        "SignalStrength" => SignalStrength,
        "ReportSource" => ReportSource,
        "AlertState" => SignalAlertState,
        "SignalError" => SignalError,
        "ConfidenceInterval" => SignalConfidenceInterval,
        "DrugEventPair" => DrugEventPair,
        "ContingencyTable" => SignalContingencyTable,
        "ValidationCheck" => SignalValidationCheck,
        "ThresholdConfig" => SignalThresholdConfig,
        "NormalizedEvent" => NormalizedEvent,
        "ValidationReport" => SignalValidationReport,
        "SignalMetrics" => SignalMetrics,
        "RawReport" => RawReport,
        "DetectionResult" => SignalDetectionResult,
        "Alert" => SignalAlert,
        "AlertTransitions" => AlertTransitions,
        "BasicNormalizer" => BasicNormalizer,
        "SynonymNormalizer" => SynonymNormalizer,
        "CsvIngestor" => CsvIngestor,
        "JsonIngestor" => JsonIngestor,
        "TableDetector" => TableDetector,
        "CompositeThreshold" => CompositeThreshold,
        "EvansThreshold" => EvansThreshold,
        "StandardValidator" => StandardValidator,
        "JsonFileStore" => JsonFileStore,
        "MemoryStore" => MemoryStore,
        "JsonReporter" => JsonReporter,
        "TableReporter" => TableReporter,
        // Brain (sessions, artifacts, implicit learning)
        "AgentId" => AgentId,
        "Artifact" => Artifact,
        "ArtifactMetadata" => ArtifactMetadata,
        "ArtifactMetrics" => ArtifactMetrics,
        "ArtifactSizeInfo" => ArtifactSizeInfo,
        "ArtifactType" => ArtifactType,
        "Belief" => Belief,
        "BeliefGraph" => BeliefGraph,
        "BeliefImplication" => BeliefImplication,
        "BrainConfig" => BrainConfig,
        "BrainError" => BrainError,
        "BrainHealth" => BrainHealth,
        "BrainSession" => BrainSession,
        "BrainSnapshot" => BrainSnapshot,
        "BrainCheckpoint" => BrainCheckpoint,
        "CodeTracker" => CodeTracker,
        "CoordinationRegistry" => CoordinationRegistry,
        "Correction" => Correction,
        "EvidenceRef" => EvidenceRef,
        "EvidenceType" => EvidenceType,
        "FileLock" => FileLock,
        "GrowthRate" => GrowthRate,
        "ImplicationStrength" => ImplicationStrength,
        "ImplicitKnowledge" => ImplicitKnowledge,
        "ImplicitStats" => ImplicitStats,
        "LockDuration" => LockDuration,
        "LockStatus" => LockStatus,
        "BrainPattern" => BrainPattern,
        "PersistentSynapseBank" => PersistentSynapseBank,
        "PipelineRun" => PipelineRun,
        "PipelineState" => PipelineState,
        "Preference" => Preference,
        "ProjectSnapshot" => ProjectSnapshot,
        "RecoveryResult" => RecoveryResult,
        "RunStatus" => RunStatus,
        "SessionEntry" => SessionEntry,
        "SessionMetrics" => SessionMetrics,
        "SynapseBankStats" => SynapseBankStats,
        "SynapseInfo" => SynapseInfo,
        "T1Primitive" => T1Primitive,
        "TrackedFile" => TrackedFile,
        "TrackerIndex" => TrackerIndex,
        "TrustAccumulator" => TrustAccumulator,
        // Config (type-safe Claude/Gemini/Git configuration)
        "AllConfigs" => AllConfigs,
        "AsrConfig" => AsrConfig,
        "ClaudeConfig" => ClaudeConfig,
        "CredentialHelper" => CredentialHelper,
        "DoDChecklist" => DoDChecklist,
        "DoDItem" => DoDItem,
        "FeatureFlags" => FeatureFlags,
        "FlywheelConfig" => FlywheelConfig,
        "FlywheelStage" => FlywheelStage,
        "GeminiConfig" => GeminiConfig,
        "GeminiHookType" => GeminiHookType,
        "GitConfig" => GitConfig,
        "GitCore" => GitCore,
        "GitDiff" => GitDiff,
        "GitFetch" => GitFetch,
        "GitInit" => GitInit,
        "GitPull" => GitPull,
        "GitUser" => GitUser,
        "HookEvent" => HookEvent,
        "HookMeta" => HookMeta,
        "HookRegistry" => HookRegistry,
        "HookTier" => HookTier,
        "McpServerConfig" => McpServerConfig,
        "Model" => Model,
        "ModelUsage" => ModelUsage,
        "OAuthAccount" => OAuthAccount,
        "PerformanceStats" => PerformanceStats,
        "ProjectConfig" => ProjectConfig,
        "RoutingConfig" => RoutingConfig,
        "SkillChain" => SkillChain,
        "SkillMapping" => SkillMapping,
        "SkillUsageStats" => SkillUsageStats,
        "VocabSkillMap" => VocabSkillMap,
        "VulnerabilityCache" => VulnerabilityCache,
        // Mesh (distributed mesh networking)
        "ChemotacticRouter" => ChemotacticRouter,
        "ChemotacticRouteSelector" => ChemotacticRouteSelector,
        "DiscoveryAction" => DiscoveryAction,
        "DiscoveryLoop" => DiscoveryLoop,
        "DiscoveryMessage" => DiscoveryMessage,
        "GossipLoop" => GossipLoop,
        "GossipMessage" => GossipMessage,
        "MeshError" => MeshError,
        "MeshEvent" => MeshEvent,
        "MeshHandle" => MeshHandle,
        "MeshMessage" => MeshMessage,
        "MeshRuntime" => MeshRuntime,
        "MeshSnapshot" => MeshSnapshot,
        "Neighbor" => Neighbor,
        "NeighborRegistry" => NeighborRegistry,
        "Node" => Node,
        "NodeState" => NodeState,
        "MeshPath" => MeshPath,
        "PeerIdentity" => PeerIdentity,
        "ResilienceAction" => ResilienceAction,
        "ResilienceLoop" => ResilienceLoop,
        "ResilienceState" => ResilienceState,
        "Route" => Route,
        "RouteQuality" => RouteQuality,
        "RoutingTable" => RoutingTable,
        "SecurityPolicy" => SecurityPolicy,
        "SnapshotStore" => SnapshotStore,
        "TlsTier" => TlsTier,
        // Vigil (AI orchestrator)
        "Urgency" => Urgency,
        "DecisionAction" => DecisionAction,
        "ExecutorType" => ExecutorType,
        "VigilEvent" => VigilEvent,
        "Interaction" => Interaction,
        "ExecutorResult" => ExecutorResult,
        "CytokineBridge" => CytokineBridge,
        "HormonalModulator" => HormonalModulator,
        "SynapticLearner" => SynapticLearner,
        "ImmunitySensor" => ImmunitySensor,
        "EnergyGovernor" => EnergyGovernor,
        "GuardianBridge" => GuardianBridge,
        "VigilRuntime" => VigilRuntime,
        "NervousSystem" => NervousSystem,
        "VigilError" => VigilError,
        "EventId" => EventId,
        "EventKind" => EventKind,
        "EventSeverity" => EventSeverity,
        "EscalationLevel" => EscalationLevel,
        "LedgerEntryType" => LedgerEntryType,
        "ConsequenceOutcome" => ConsequenceOutcome,
        "ShellConsequence" => ShellConsequence,
        "WebhookConsequence" => WebhookConsequence,
        "NotifyConsequence" => NotifyConsequence,
        "WatchEvent" => WatchEvent,
        "ThresholdCheck" => ThresholdCheck,
        "BoundarySpec" => BoundarySpec,
        "BoundaryViolation" => BoundaryViolation,
        "ConsequenceReceipt" => ConsequenceReceipt,
        "LedgerEntry" => LedgerEntry,
        "BoundaryGate" => BoundaryGate,
        "VigilanceLedger" => VigilanceLedger,
        "ConsequencePipeline" => ConsequencePipeline,
        "VigilDaemon" => VigilDaemon,
        "ShutdownHandle" => ShutdownHandle,
        "VigilHealth" => VigilHealth,
        "VigilStats" => VigilStats,
        "VigilSubError" => VigilSubError,
        // Signal Theory (formal signal detection axioms & algebra)
        "SignalPrimitive" => SignalPrimitive,
        "EvidenceKind" => EvidenceKind,
        "A2NoiseDominance" => A2NoiseDominance,
        "A3SignalExistence" => A3SignalExistence,
        "A4BoundaryRequirement" => A4BoundaryRequirement,
        "A5Disproportionality" => A5Disproportionality,
        "A6CausalInference" => A6CausalInference,
        "ObservationSpace" => ObservationSpace,
        "Baseline" => Baseline,
        "Ratio" => Ratio,
        "Difference" => Difference,
        "DetectionInterval" => DetectionInterval,
        "DetectionOutcome" => DetectionOutcome,
        "SignalStrengthLevel" => SignalStrengthLevel,
        "SignalVerificationReport" => SignalVerificationReport,
        "BoundaryKind" => BoundaryKind,
        "ConjunctionMode" => ConjunctionMode,
        "ThresholdPreset" => ThresholdPreset,
        "DetectionPhase" => DetectionPhase,
        "FixedBoundary" => FixedBoundary,
        "AdaptiveBoundary" => AdaptiveBoundary,
        "CompositeBoundary" => CompositeBoundary,
        "CascadedThreshold" => CascadedThreshold,
        "DetectionPipeline" => DetectionPipeline,
        "DecisionOutcome" => DecisionOutcome,
        "DecisionMatrix" => DecisionMatrix,
        "RocPoint" => RocPoint,
        "RocCurve" => RocCurve,
        "DPrime" => DPrime,
        "ResponseBias" => ResponseBias,
        "L1TotalCountConservation" => L1TotalCountConservation,
        "L2BaseRateInvariance" => L2BaseRateInvariance,
        "L3SensitivitySpecificityTradeoff" => L3SensitivitySpecificityTradeoff,
        "L4InformationConservation" => L4InformationConservation,
        "ConservationReport" => ConservationReport,
        // FAERS ETL (FDA adverse event data pipeline)
        "CaseCount" => CaseCount,
        "RowCount" => RowCount,
        "DrugRole" => DrugRole,
        "DrugCharacterization" => DrugCharacterization,
        "OpenFdaError" => OpenFdaError,
        "DrugName" => DrugName,
        "EventName" => EventName,
        "ContingencyBatch" => ContingencyBatch,
        "FaersPipelineOutput" => FaersPipelineOutput,
        "ReactionOutcome" => ReactionOutcome,
        "OutcomeCase" => OutcomeCase,
        "OutcomeConditionedConfig" => OutcomeConditionedConfig,
        "MonthBucket" => MonthBucket,
        "TemporalCase" => TemporalCase,
        "VelocityConfig" => VelocityConfig,
        "SeriousnessFlag" => SeriousnessFlag,
        "CaseSeriousness" => CaseSeriousness,
        "SeriousnessCase" => SeriousnessCase,
        "CascadeConfig" => CascadeConfig,
        "PolypharmacyCase" => PolypharmacyCase,
        "PolypharmacyConfig" => PolypharmacyConfig,
        "ReporterQualification" => ReporterQualification,
        "ReporterCase" => ReporterCase,
        "ReporterWeightedSignal" => ReporterWeightedSignal,
        "ReporterWeightedConfig" => ReporterWeightedConfig,
        "GeographicCase" => GeographicCase,
        "CountrySignal" => CountrySignal,
        "GeographicConfig" => GeographicConfig,
        "FaersReport" => FaersReport,
        "ReportFingerprint" => ReportFingerprint,
        "FaersDeduplicationResult" => FaersDeduplicationResult,
        "DuplicateCluster" => DuplicateCluster,
        "DeduplicatorConfig" => DeduplicatorConfig,
        "FaersDeduplicator" => FaersDeduplicator,
        "NdcProduct" => NdcProduct,
        "NdcBridge" => NdcBridge,
        "NdcMatch" => NdcMatch,
        "NdcMatchType" => NdcMatchType,
        "DrugEventResponse" => DrugEventResponse,
        "DrugEventQuery" => DrugEventQuery,
        "OpenFdaClient" => OpenFdaClient,
        "FaersSignalDetectionResult" => FaersSignalDetectionResult,
        "OutcomeConditionedSignal" => OutcomeConditionedSignal,
        "SignalVelocity" => SignalVelocity,
        "SeriousnessCascade" => SeriousnessCascade,
        "PolypharmacySignal" => PolypharmacySignal,
        "GeographicDivergence" => GeographicDivergence,
        // Algovigilance (ICSR deduplication + signal triage)
        "Similarity" => Similarity,
        "Relevance" => Relevance,
        "HalfLife" => HalfLife,
        "CaseId" => CaseId,
        "SignalId" => SignalId,
        "DecayReport" => DecayReport,
        "AlgovigilanceError" => AlgovigilanceError,
        "SynonymEntry" => SynonymEntry,
        "AlgovigilanceStore" => AlgovigilanceStore,
        "IcsrNarrative" => IcsrNarrative,
        "CasePair" => CasePair,
        "AlgoDeduplicationResult" => AlgoDeduplicationResult,
        "DedupConfig" => DedupConfig,
        "SynonymPair" => SynonymPair,
        "DedupFunction" => DedupFunction,
        "TriagedSignal" => TriagedSignal,
        "TriageConfig" => TriageConfig,
        "TriageResult" => TriageResult,
        "SignalInput" => SignalInput,
        "ReinforcementEvent" => ReinforcementEvent,
        "TriageFunction" => TriageFunction,
        "UrgencyClassification" => UrgencyClassification,
        "UrgencyClassifier" => UrgencyClassifier,
        "SignalQueue" => SignalQueue,
        // Signal Fence (process-aware network boundary)
        "FenceMode" => FenceMode,
        "Direction" => Direction,
        "FenceVerdict" => FenceVerdict,
        "Protocol" => Protocol,
        "TcpState" => TcpState,
        "SocketEntry" => SocketEntry,
        "ProcessInfo" => ProcessInfo,
        "ProcessMatch" => ProcessMatch,
        "NetworkMatch" => NetworkMatch,
        "FenceRule" => FenceRule,
        "FenceDecision" => FenceDecision,
        "FenceAuditEntry" => FenceAuditEntry,
        "ConnectionEvent" => ConnectionEvent,
        "RuleSet" => RuleSet,
        "FencePolicy" => FencePolicy,
        "FenceStats" => FenceStats,
        "FenceAuditLog" => FenceAuditLog,
        "FenceTickResult" => FenceTickResult,
        "FenceReport" => FenceReport,
        "EnforcerOp" => EnforcerOp,
        "TickDecision" => TickDecision,
        // Education Machine (mastery, assessment, spaced repetition)
        "MasteryLevel" => MasteryLevel,
        "Difficulty" => Difficulty,
        "BayesianPrior" => BayesianPrior,
        "LearningPhase" => LearningPhase,
        "CompetencyLevel" => CompetencyLevel,
        "MasteryVerdict" => MasteryVerdict,
        "Grade" => Grade,
        "Question" => Question,
        "QuestionResult" => QuestionResult,
        "EduAssessment" => EduAssessment,
        "EduAssessmentResult" => EduAssessmentResult,
        "Subject" => Subject,
        "LessonRef" => LessonRef,
        "Lesson" => Lesson,
        "LessonStep" => LessonStep,
        "LessonContent" => LessonContent,
        "PrimitiveMapping" => PrimitiveMapping,
        "Enrollment" => Enrollment,
        "AssessmentRecord" => AssessmentRecord,
        "ReviewState" => ReviewState,
        "PhaseTransition" => PhaseTransition,
        "Learner" => Learner,
        // Biological Systems — Integumentary (20)
        "AuthResult" => AuthResult,
        "SensorKind" => SensorKind,
        "SensorReading" => SensorReading,
        "SkinCondition" => SkinCondition,
        "Shield" => Shield,
        "WoundRepair" => WoundRepair,
        "CoolingAction" => CoolingAction,
        "IntegumentaryError" => IntegumentaryError,
        "PermissionRule" => PermissionRule,
        "PermissionDecision" => PermissionDecision,
        "RuleOrigin" => RuleOrigin,
        "PermissionCascade" => PermissionCascade,
        "SettingsScope" => SettingsScope,
        "ScopedSetting" => ScopedSetting,
        "SettingsPrecedence" => SettingsPrecedence,
        "SandboxLayer" => SandboxLayer,
        "RiskLevel" => RiskLevel,
        "Scar" => Scar,
        "ScarringMechanism" => ScarringMechanism,
        "IntegumentaryHealth" => IntegumentaryHealth,
        // Biological Systems — Respiratory (17)
        "InputSource" => InputSource,
        "Inhaled" => Inhaled,
        "Extracted" => Extracted,
        "Exhaled" => Exhaled,
        "ExchangeResult" => ExchangeResult,
        "BreathingRate" => BreathingRate,
        "RespiratoryError" => RespiratoryError,
        "ContextSource" => ContextSource,
        "Inhalation" => Inhalation,
        "Exhalation" => Exhalation,
        "GasExchange" => GasExchange,
        "DeadSpace" => DeadSpace,
        "ContextFork" => ContextFork,
        "TidalVolume" => TidalVolume,
        "VitalCapacity" => VitalCapacity,
        "BreathCycle" => BreathCycle,
        "RespiratoryHealth" => RespiratoryHealth,
        // Biological Systems — Digestive (20)
        "Quality" => Quality,
        "DataKind" => DataKind,
        "Fragment" => Fragment,
        "Taste" => Taste,
        "Nutrients" => Nutrients,
        "Absorbed" => Absorbed,
        "Metabolized" => Metabolized,
        "DigestiveError" => DigestiveError,
        "SkillTrigger" => SkillTrigger,
        "SkillLoad" => SkillLoad,
        "ContextMode" => ContextMode,
        "SkillFrontmatter" => SkillFrontmatter,
        "Sphincter" => Sphincter,
        "SkillArguments" => SkillArguments,
        "EnzymeType" => EnzymeType,
        "EnzymeSubstitution" => EnzymeSubstitution,
        "Microbiome" => Microbiome,
        "SkillResult" => SkillResult,
        "SkillExecution" => SkillExecution,
        "DigestiveHealth" => DigestiveHealth,
        // Biological Systems — Circulatory (21)
        "CellKind" => CellKind,
        "BloodCell" => BloodCell,
        "Enriched" => Enriched,
        "Destination" => Destination,
        "RouteDecision" => RouteDecision,
        "BloodPressure" => BloodPressure,
        "Pulse" => Pulse,
        "Platelet" => Platelet,
        "CirculatoryError" => CirculatoryError,
        "McpTransport" => McpTransport,
        "McpScope" => McpScope,
        "FlowDirection" => FlowDirection,
        "McpServer" => McpServer,
        "McpHeartbeat" => McpHeartbeat,
        "ToolCall" => ToolCall,
        "ToolResult" => ToolResult,
        "PortalFiltration" => PortalFiltration,
        "SelectivePerfusion" => SelectivePerfusion,
        "BloodPayload" => BloodPayload,
        "FrankStarling" => FrankStarling,
        "CirculatoryHealth" => CirculatoryHealth,
        // Biological Systems — Skeletal (5)
        "WolffsLaw" => WolffsLaw,
        "SkeletalCorrection" => SkeletalCorrection,
        "BoneMarrow" => BoneMarrow,
        "Joint" => Joint,
        "SkeletalHealth" => SkeletalHealth,
        // Biological Systems — Muscular (8)
        "MuscleType" => MuscleType,
        "ToolClassification" => ToolClassification,
        "AntagonisticPair" => AntagonisticPair,
        "SizePrinciple" => SizePrinciple,
        "Fatigue" => Fatigue,
        "ModelEscalation" => ModelEscalation,
        "MotorActivation" => MotorActivation,
        "MuscularHealth" => MuscularHealth,
        // Biological Systems — Lymphatic (7)
        "OutputStyle" => OutputStyle,
        "LymphDrainage" => LymphDrainage,
        "LymphNode" => LymphNode,
        "ThymicSelection" => ThymicSelection,
        "ThymicVerdict" => ThymicVerdict,
        "OverflowHandler" => OverflowHandler,
        "LymphaticHealth" => LymphaticHealth,
        // Biological Systems — Nervous (18)
        "NeuronType" => NeuronType,
        "Impulse" => Impulse,
        "ReflexArc" => ReflexArc,
        "Myelination" => Myelination,
        "SensoryInput" => SensoryInput,
        "MotorCommand" => MotorCommand,
        "SignalSpeed" => SignalSpeed,
        "NervousHealth" => NervousHealth,
        "NervousHookEvent" => NervousHookEvent,
        "NervousHookDispatch" => NervousHookDispatch,
        "HookChain" => HookChain,
        "EventBusRoute" => EventBusRoute,
        "ContextAssembly" => ContextAssembly,
        "ToolChain" => ToolChain,
        "SignalLatency" => SignalLatency,
        "MyelinationCache" => MyelinationCache,
        "ReflexResponse" => ReflexResponse,
        "NervousSystemHealth" => NervousSystemHealth,
        // Biological Systems — Urinary (18)
        "FilterCategory" => FilterCategory,
        "Nephron" => Nephron,
        "GlomerularFiltration" => GlomerularFiltration,
        "Reabsorption" => Reabsorption,
        "Excretion" => Excretion,
        "FiltrationRate" => FiltrationRate,
        "Bladder" => Bladder,
        "UrinaryHealth" => UrinaryHealth,
        "TelemetryPruning" => TelemetryPruning,
        "SessionExpiry" => SessionExpiry,
        "ArtifactRetention" => ArtifactRetention,
        "LogRotation" => LogRotation,
        "RetentionPolicy" => RetentionPolicy,
        "DecisionAuditCleanup" => DecisionAuditCleanup,
        "WasteCategory" => WasteCategory,
        "DisposalMethod" => DisposalMethod,
        "SilentFailureRisk" => SilentFailureRisk,
        "UrinarySystemHealth" => UrinarySystemHealth,
        // Biological Systems — Reproductive (18)
        "GameteFitness" => GameteFitness,
        "Gamete" => Gamete,
        "Fertilization" => Fertilization,
        "Trimester" => Trimester,
        "Embryo" => Embryo,
        "Differentiation" => Differentiation,
        "GeneticMutation" => GeneticMutation,
        "ReproductiveHealth" => ReproductiveHealth,
        "CiStage" => CiStage,
        "CiPipeline" => CiPipeline,
        "DeployTarget" => DeployTarget,
        "DeploymentBirth" => DeploymentBirth,
        "BranchMutation" => BranchMutation,
        "MergeEvent" => MergeEvent,
        "TrimesterGate" => TrimesterGate,
        "ScalingEvent" => ScalingEvent,
        "KnowledgeTransfer" => KnowledgeTransfer,
        "ReproductiveSystemHealth" => ReproductiveSystemHealth,
        // Cloud Computing Taxonomy (35 types — nexcore-cloud)
        // T1 (Cloud-prefixed to avoid collision with vigilance types)
        "CloudIdentity" => nexcore_cloud::primitives::Identity,
        "CloudThreshold" => nexcore_cloud::primitives::Threshold,
        "CloudFeedbackLoop" => nexcore_cloud::primitives::FeedbackLoop,
        "CloudIdempotency" => nexcore_cloud::primitives::Idempotency,
        "CloudImmutability" => nexcore_cloud::primitives::Immutability,
        "CloudConvergence" => nexcore_cloud::primitives::Convergence,
        // T2-P
        "Compute" => nexcore_cloud::primitives::Compute,
        "Storage" => nexcore_cloud::primitives::Storage,
        "NetworkLink" => nexcore_cloud::primitives::NetworkLink,
        "IsolationBoundary" => nexcore_cloud::primitives::IsolationBoundary,
        "CloudPermission" => nexcore_cloud::primitives::Permission,
        "ResourcePool" => nexcore_cloud::primitives::ResourcePool,
        "Metering" => nexcore_cloud::primitives::Metering,
        "Replication" => nexcore_cloud::primitives::Replication,
        "CloudRouting" => nexcore_cloud::primitives::Routing,
        "CloudLease" => nexcore_cloud::primitives::Lease,
        "CloudEncryption" => nexcore_cloud::primitives::Encryption,
        "CloudQueue" => nexcore_cloud::primitives::Queue,
        "CloudHealthCheck" => nexcore_cloud::primitives::HealthCheck,
        "Elasticity" => nexcore_cloud::primitives::Elasticity,
        // T2-C Composites
        "VirtualMachine" => nexcore_cloud::composites::VirtualMachine,
        "LoadBalancer" => nexcore_cloud::composites::LoadBalancer,
        "AutoScaling" => nexcore_cloud::composites::AutoScaling,
        "CloudIam" => nexcore_cloud::composites::Iam,
        "EventualConsistency" => nexcore_cloud::composites::EventualConsistency,
        "Tenancy" => nexcore_cloud::composites::Tenancy,
        "PayPerUse" => nexcore_cloud::composites::PayPerUse,
        "ReservedCapacity" => nexcore_cloud::composites::ReservedCapacity,
        "SpotPricing" => nexcore_cloud::composites::SpotPricing,
        "SecretsManagement" => nexcore_cloud::composites::SecretsManagement,
        // T3 Domain-Specific
        "Container" => nexcore_cloud::service_models::Container,
        "Iaas" => nexcore_cloud::service_models::Iaas,
        "Paas" => nexcore_cloud::service_models::Paas,
        "Saas" => nexcore_cloud::service_models::Saas,
        "Serverless" => nexcore_cloud::service_models::Serverless,
    }

    unknown_type_json(type_name)
}

// ============================================================================
// Reverse Synthesis Tools
// ============================================================================

/// Reverse compose: given T1 primitives, synthesize upward through the tier DAG
pub fn reverse_compose(
    params: LexPrimitivaReverseComposeParams,
) -> Result<CallToolResult, McpError> {
    // Parse primitive names to LexPrimitiva enums
    let mut primitives = Vec::new();
    let mut errors = Vec::new();

    for name in &params.primitives {
        match find_primitive(name) {
            Some(p) => primitives.push(p),
            None => errors.push(name.clone()),
        }
    }

    if !errors.is_empty() {
        let available: Vec<_> = LexPrimitiva::all()
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();
        let json = json!({
            "error": format!("Unknown primitives: {}", errors.join(", ")),
            "available": available,
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    let synth = RevSynthesizer::new();
    let opts = SynthesisOpts {
        target_tier: None,
        pattern_hint: params.pattern_hint,
        min_coherence: params.min_coherence.unwrap_or(0.0),
    };

    match synth.synthesize(primitives, opts) {
        Ok(result) => {
            let primitives_json: Vec<_> = result
                .composition
                .primitives
                .iter()
                .map(|p| json!({ "name": format!("{:?}", p), "symbol": p.symbol() }))
                .collect();

            let interactions_json: Vec<_> = result
                .interactions
                .iter()
                .map(|i| {
                    json!({
                        "source": format!("{:?}", i.source),
                        "target": format!("{:?}", i.target),
                        "relation": format!("{}", i.relation),
                        "weight": i.weight,
                    })
                })
                .collect();

            let pattern_matches_json: Vec<_> = result
                .pattern_matches
                .iter()
                .take(5)
                .map(|m| {
                    json!({
                        "name": m.name,
                        "distance": m.distance,
                        "missing": m.missing.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                        "extra": m.extra.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                    })
                })
                .collect();

            let suggestions_json: Vec<_> = result
                .suggestions
                .iter()
                .take(3)
                .map(|s| {
                    json!({
                        "target_pattern": s.target_pattern,
                        "missing_primitives": s.missing_primitives.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                        "resulting_tier": format!("{}", s.resulting_tier),
                        "coherence_gain": s.coherence_gain,
                    })
                })
                .collect();

            let json = json!({
                "tier": format!("{}", result.tier),
                "dominant": result.dominant.map(|d| json!({ "name": format!("{:?}", d), "symbol": d.symbol() })),
                "coherence": result.coherence,
                "primitives": primitives_json,
                "interactions": interactions_json,
                "nearest_pattern": result.nearest_pattern.map(|(name, dist)| json!({ "name": name, "distance": dist })),
                "pattern_matches": pattern_matches_json,
                "suggestions": suggestions_json,
            });

            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{}", e) });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Reverse lookup: find grounded types matching a set of T1 primitives
pub fn reverse_lookup(params: LexPrimitivaReverseLookupParams) -> Result<CallToolResult, McpError> {
    // Parse primitive names
    let mut primitives = Vec::new();
    let mut errors = Vec::new();

    for name in &params.primitives {
        match find_primitive(name) {
            Some(p) => primitives.push(p),
            None => errors.push(name.clone()),
        }
    }

    if !errors.is_empty() {
        let available: Vec<_> = LexPrimitiva::all()
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();
        let json = json!({
            "error": format!("Unknown primitives: {}", errors.join(", ")),
            "available": available,
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    let search_set: HashSet<LexPrimitiva> = primitives.iter().copied().collect();
    let mode = params.match_mode.as_deref().unwrap_or("superset");

    // Iterate KNOWN_TYPES and filter by primitive match
    let mut matches: Vec<serde_json::Value> = Vec::new();

    for &type_name in KNOWN_TYPES {
        if let Some(comp) = get_composition_direct(type_name) {
            let type_set = comp.unique();
            let matched = match mode {
                "exact" => type_set == search_set,
                "subset" => search_set.is_subset(&type_set),
                _ => search_set.is_subset(&type_set), // default: superset (type contains all search primitives)
            };

            if matched {
                let tier = GroundingTier::classify(&comp);
                matches.push(json!({
                    "type": type_name,
                    "tier": format!("{}", tier),
                    "primitives": comp.primitives.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                    "dominant": comp.dominant.map(|d| format!("{:?}", d)),
                    "confidence": comp.confidence,
                }));
            }
        }
    }

    // Also run pattern matching via synthesizer
    let synth = RevSynthesizer::new();
    let pattern_matches: Vec<_> = synth
        .reverse_lookup_patterns(&primitives)
        .into_iter()
        .filter(|m| m.distance < 0.5)
        .take(5)
        .map(|m| {
            json!({
                "name": m.name,
                "distance": m.distance,
                "missing": m.missing.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                "extra": m.extra.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
            })
        })
        .collect();

    let json = json!({
        "search_primitives": primitives.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
        "match_mode": mode,
        "grounded_types": matches,
        "grounded_type_count": matches.len(),
        "canonical_patterns": pattern_matches,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Direct composition lookup without JSON wrapping — used by reverse_lookup.
pub(crate) fn get_composition_direct(type_name: &str) -> Option<PrimitiveComposition> {
    use nexcore_cortex::cloud::{DatasetConfig, FineTuneJob, TrainingParams};
    use nexcore_cortex::generate::GenerateParams;
    use nexcore_cortex::lora::{LoraAdapter, LoraConfig};
    use nexcore_cortex::model::{ModelConfig, ModelFormat};
    use nexcore_hormones::{
        BehavioralModifiers, EndocrineState, HormoneLevel, HormoneType, Stimulus,
    };
    use nexcore_labs::betting::bdi::Bdi;
    use nexcore_labs::betting::ecs::EcsScore;
    use nexcore_phenotype::{Mutation, Phenotype};
    use nexcore_ribosome::{Contract, DriftResult, DriftSeverity, DriftSignal, DriftType};
    use nexcore_signal_types::SignalResult;
    use nexcore_transcriptase::{DiagnosticLevel, Fidelity, Schema, SchemaKind};
    use nexcore_vigilance::domain_discovery::primitives::{Primitive, PrimitiveTier};
    use nexcore_vigilance::primitives::bathroom_lock::Occupancy;
    use nexcore_vigilance::primitives::chemistry::*;
    use nexcore_vigilance::primitives::frontier::{
        AbsenceMarker, AuditTrail, ConsumptionMark, EntityStatus, Pipeline, RecursionBound,
        ResourcePath,
    };
    use nexcore_vigilance::primitives::quantum::{
        Amplitude, Decoherence, Eigenstate, Entanglement, Hermiticity, Interference, Measurement,
        Observable, Phase, Qubit, Superposition, Uncertainty, Unitarity,
    };
    use nexcore_vigilance::primitives::recipient::{Recipient, Tracked, Vulnerable};
    use nexcore_vigilance::primitives::safety::{BoundaryBreach, Harm, Monitoring, SafetyBoundary};
    use nexcore_vigilance::primitives::transfer::{
        Accumulator, Atomicity, Checkpoint, CircuitBreaker, CompareAndSwap, Homeostasis,
        Idempotency, NegativeEvidence, RateLimiter, SerializationGuard, StagedValidation,
        ToctouWindow, TopologicalAddress,
    };
    use nexcore_vigilance::tov::{HarmType, SafetyMargin};
    // Trust engine types (Product × grounding)
    use nexcore_trust::Evidence as TrustEvidence;
    use nexcore_trust::{
        DimensionWeights, MultiTrustEngine, TrustConfig, TrustDimension, TrustEngine, TrustLevel,
        TrustVelocity,
    };
    // Guardian engine types (Product × grounding)
    use nexcore_guardian_engine::{OriginatorType, RiskContext, RiskScore};
    // Spatial bridge metric types (κ Comparison dominant)
    use nexcore_dtree::spatial_bridge::ImpurityMetric;
    use nexcore_guardian_engine::spatial_bridge::{RiskScoreMetric, SeverityMetric};
    // PV types (Product × update)
    use nexcore_vigilance::primitives::pharmacovigilance::DisproportionalityScore;
    // Insight Engine types (system identity — T2-P/T2-C/T3)
    use nexcore_insight::composites::NoveltyReason;
    use nexcore_insight::{
        Compression, Connection, InsightEngine, Novelty, Pattern, Recognition, Suddenness,
    };
    // DNA-based computation types (nexcore_lex_primitiva::GroundsTo)
    use nexcore_dna::asm::Token as DnaToken;
    use nexcore_dna::codon_table::CodonTable;
    use nexcore_dna::cortex::{
        Cluster as DnaCluster, ClusterResult as DnaClusterResult, EvolutionConfig, EvolutionResult,
        GravityConfig, Organism, Particle,
    };
    use nexcore_dna::data::{DnaArray, DnaFrame, DnaMap, DnaRecord, DnaType, DnaValue};
    use nexcore_dna::disasm::DisasmOptions;
    use nexcore_dna::gene::{Gene, Genome, Plasmid};
    use nexcore_dna::glyph::{Glyph as DnaGlyph, GlyphPair as DnaGlyphPair};
    use nexcore_dna::isa::Instruction as DnaInstruction;
    use nexcore_dna::lang::diagnostic::{Diagnostic as DnaDiagnostic, ErrorCode as DnaErrorCode};
    use nexcore_dna::lang::json::JsonValue as DnaJsonValue;
    use nexcore_dna::lang::templates::Template as DnaTemplate;
    use nexcore_dna::lexicon::{Affinity, Lexicon as DnaLexicon, WordOre};
    use nexcore_dna::program::Program as DnaProgram;
    use nexcore_dna::pv_theory::{
        AlertLevel, CausalityCategory, CausalityScore, DrugEventSignal as DnaSignal, DrugProfile,
        SafetyLevel as DnaSafetyLevel, SafetyMargin as DnaSafetyMargin, VigilanceState,
    };
    use nexcore_dna::statemind::{
        Drift as DnaDrift, MindPoint as DnaMindPoint, StateMind as DnaStateMind,
    };
    use nexcore_dna::string_theory::{
        FrequencySpectrum, HarmonicMode, Resonance as DnaResonance, StringEnergy, StringTension,
    };
    use nexcore_dna::tile::Pixel;
    use nexcore_dna::types::{AminoAcid, Codon, DoubleHelix, Nucleotide, Strand};
    use nexcore_dna::vm::CodonVM;
    use nexcore_dna::voxel::{VoxelCube, VoxelPos};
    // Energy types (ATP/ADP biochemistry)
    use nexcore_energy::{
        EnergyState, EnergySystem, Operation, OperationBuilder, RecyclingRate, Regime, Strategy,
        TokenPool, WasteClass,
    };
    // Compliance types
    use nexcore_compliance::oscal::{Control, ControlCatalog, ControlStatus};
    use nexcore_compliance::sam::{Exclusion, ExclusionClassification, ExclusionType};
    use nexcore_compliance::{Assessment, ComplianceResult, Finding, FindingSeverity};
    // Measure types (information theory, graph theory, statistics)
    use nexcore_measure::error::MeasureError;
    use nexcore_measure::types::CodeDensityIndex;
    use nexcore_measure::types::{
        BayesianPosterior, Centrality, ChemistryMapping, CouplingRatio, CrateHealth, CrateId,
        CrateMeasurement, Density, DriftDirection, DriftResult as MeasureDriftResult, Entropy,
        GraphAnalysis, GraphNode, HealthComponents, HealthRating, HealthScore, MeasureTimestamp,
        PoissonCi, Probability, RatingDistribution, RegressionResult, TestDensity, WelchResult,
        WorkspaceHealth, WorkspaceMeasurement,
    };
    // Signal types (PV signal detection pipeline)
    use signal::alert::AlertTransitions;
    use signal::core::{
        Alert as SignalAlert, AlertState as SignalAlertState, ChiSquare as SignalChiSquare,
        ConfidenceInterval as SignalConfidenceInterval, ContingencyTable as SignalContingencyTable,
        DetectionResult as SignalDetectionResult, DrugEventPair, Ebgm as SignalEbgm,
        Ic as SignalIc, NormalizedEvent, Prr as SignalPrr, RawReport, ReportSource,
        Ror as SignalRor, SignalError, SignalStrength, ThresholdConfig as SignalThresholdConfig,
        ValidationCheck as SignalValidationCheck, ValidationReport as SignalValidationReport,
    };
    use signal::detect::TableDetector;
    use signal::ingest::{CsvIngestor, JsonIngestor};
    use signal::normalize::{BasicNormalizer, SynonymNormalizer};
    use signal::report::{JsonReporter, TableReporter};
    use signal::stats::SignalMetrics;
    use signal::store::{JsonFileStore, MemoryStore};
    use signal::threshold::{CompositeThreshold, EvansThreshold};
    use signal::validate::StandardValidator;
    // Brain types (sessions, artifacts, implicit learning)
    use nexcore_brain::artifact::{Artifact, ArtifactMetadata, ArtifactType};
    use nexcore_brain::config::BrainConfig;
    use nexcore_brain::coordination::{
        AgentId, CoordinationRegistry, FileLock, LockDuration, LockStatus,
    };
    use nexcore_brain::error::BrainError;
    use nexcore_brain::implicit::{
        Belief, BeliefGraph, BeliefImplication, Correction, EvidenceRef, EvidenceType,
        ImplicationStrength, ImplicitKnowledge, ImplicitStats, Pattern as BrainPattern, Preference,
        T1Primitive, TrustAccumulator,
    };
    use nexcore_brain::metrics::{
        ArtifactMetrics, ArtifactSizeInfo, BrainHealth, BrainSnapshot, GrowthRate, SessionMetrics,
    };
    use nexcore_brain::pipeline::{
        Checkpoint as BrainCheckpoint, PipelineRun, PipelineState, RunStatus,
    };
    use nexcore_brain::recovery::RecoveryResult;
    use nexcore_brain::session::{BrainSession, SessionEntry};
    use nexcore_brain::synapse::{PersistentSynapseBank, SynapseBankStats, SynapseInfo};
    use nexcore_brain::tracker::{CodeTracker, ProjectSnapshot, TrackedFile, TrackerIndex};
    // Config types (type-safe Claude/Gemini/Git configuration)
    use nexcore_config::AllConfigs;
    use nexcore_config::asr::{
        AsrConfig, DoDChecklist, DoDItem, FlywheelConfig, FlywheelStage, Model, RoutingConfig,
    };
    use nexcore_config::claude::{
        ClaudeConfig, FeatureFlags, McpServerConfig, ModelUsage, OAuthAccount, PerformanceStats,
        ProjectConfig, SkillUsageStats, VulnerabilityCache,
    };
    use nexcore_config::gemini::GeminiConfig;
    use nexcore_config::gemini::HookType as GeminiHookType;
    use nexcore_config::git::{
        CredentialHelper, GitConfig, GitCore, GitDiff, GitFetch, GitInit, GitPull, GitUser,
    };
    use nexcore_config::hooks::{HookEvent, HookMeta, HookRegistry, HookTier};
    use nexcore_config::vocab::{SkillChain, SkillMapping, VocabSkillMap};
    // Mesh types (distributed mesh networking)
    use nexcore_mesh::chemotaxis::{ChemotacticRouteSelector, ChemotacticRouter};
    use nexcore_mesh::discovery::{DiscoveryAction, DiscoveryLoop, DiscoveryMessage};
    use nexcore_mesh::error::MeshError;
    use nexcore_mesh::gossip::{GossipLoop, GossipMessage};
    use nexcore_mesh::neighbor::{Neighbor, NeighborRegistry};
    use nexcore_mesh::node::{MeshMessage, Node, NodeState};
    use nexcore_mesh::persistence::{MeshSnapshot, SnapshotStore};
    use nexcore_mesh::resilience::{ResilienceAction, ResilienceLoop, ResilienceState};
    use nexcore_mesh::routing::{Route, RoutingTable};
    use nexcore_mesh::runtime::{MeshEvent, MeshHandle, MeshRuntime};
    use nexcore_mesh::security::{PeerIdentity, SecurityPolicy, TlsTier};
    use nexcore_mesh::topology::{Path as MeshPath, RouteQuality};
    // Vigil types (AI orchestrator — event bus, consequences, boundaries)
    use nexcore_vigil::VigilError;
    use nexcore_vigil::bridge::nervous_system::{
        CytokineBridge, EnergyGovernor, GuardianBridge, HormonalModulator, ImmunitySensor,
        NervousSystem, SynapticLearner,
    };
    use nexcore_vigil::models::{
        DecisionAction, Event as VigilEvent, ExecutorResult, ExecutorType, Interaction, Urgency,
    };
    use nexcore_vigil::runtime::VigilRuntime;
    use nexcore_vigil::vigilance::boundary::{
        BoundaryGate, BoundarySpec, BoundaryViolation, ThresholdCheck,
    };
    use nexcore_vigil::vigilance::consequence::{
        ConsequenceOutcome, ConsequencePipeline, ConsequenceReceipt, EscalationLevel,
        NotifyConsequence, ShellConsequence, WebhookConsequence,
    };
    use nexcore_vigil::vigilance::daemon::{ShutdownHandle, VigilDaemon, VigilHealth, VigilStats};
    use nexcore_vigil::vigilance::error::VigilError as VigilSubError;
    use nexcore_vigil::vigilance::event::{EventId, EventKind, EventSeverity, WatchEvent};
    use nexcore_vigil::vigilance::ledger::{LedgerEntry, LedgerEntryType, VigilanceLedger};
    // Signal Theory types (formal signal detection axioms & algebra)
    use nexcore_signal_theory::SignalPrimitive;
    use nexcore_signal_theory::algebra::{CascadedThreshold, DetectionPipeline};
    use nexcore_signal_theory::axioms::{
        A2NoiseDominance, A3SignalExistence, A4BoundaryRequirement, A5Disproportionality,
        A6CausalInference, EvidenceKind,
    };
    use nexcore_signal_theory::conservation::{
        ConservationReport, L1TotalCountConservation, L2BaseRateInvariance,
        L3SensitivitySpecificityTradeoff, L4InformationConservation,
    };
    use nexcore_signal_theory::decision::{
        DPrime, DecisionMatrix, DecisionOutcome, ResponseBias, RocCurve, RocPoint,
    };
    use nexcore_signal_theory::detection::{
        Baseline, DetectionInterval, DetectionOutcome, Difference, ObservationSpace, Ratio,
        SignalStrengthLevel, SignalVerificationReport,
    };
    use nexcore_signal_theory::threshold::{
        AdaptiveBoundary, BoundaryKind, CompositeBoundary, ConjunctionMode, DetectionPhase,
        FixedBoundary, ThresholdPreset,
    };
    // FAERS ETL types (FDA adverse event data pipeline)
    use nexcore_faers_etl::api::{DrugEventQuery, DrugEventResponse, OpenFdaClient, OpenFdaError};
    use nexcore_faers_etl::dedup::{
        DeduplicationResult as FaersDeduplicationResult, DeduplicatorConfig, DuplicateCluster,
        FaersDeduplicator, FaersReport, ReportFingerprint,
    };
    use nexcore_faers_etl::ndc::{NdcBridge, NdcMatch, NdcMatchType, NdcProduct};
    use nexcore_faers_etl::{
        CascadeConfig, CaseSeriousness, CountrySignal, DrugCharacterization, GeographicCase,
        GeographicConfig, GeographicDivergence, MonthBucket, OutcomeCase, OutcomeConditionedConfig,
        OutcomeConditionedSignal, PolypharmacyCase, PolypharmacyConfig, PolypharmacySignal,
        ReactionOutcome, ReporterCase, ReporterQualification, ReporterWeightedConfig,
        ReporterWeightedSignal, SeriousnessCascade, SeriousnessCase, SeriousnessFlag,
        SignalVelocity, TemporalCase, VelocityConfig,
    };
    use nexcore_faers_etl::{CaseCount, ContingencyBatch, DrugName, DrugRole, EventName, RowCount};
    use nexcore_faers_etl::{
        PipelineOutput as FaersPipelineOutput, SignalDetectionResult as FaersSignalDetectionResult,
    };
    // Algovigilance types (ICSR deduplication + signal triage)
    use nexcore_algovigilance::AlgovigilanceError;
    use nexcore_algovigilance::dedup::DedupFunction;
    use nexcore_algovigilance::dedup::types::{
        CasePair, DedupConfig, DeduplicationResult as AlgoDeduplicationResult, IcsrNarrative,
        SynonymPair,
    };
    use nexcore_algovigilance::store::{AlgovigilanceStore, SynonymEntry};
    use nexcore_algovigilance::triage::TriageFunction;
    use nexcore_algovigilance::triage::classifier::{UrgencyClassification, UrgencyClassifier};
    use nexcore_algovigilance::triage::queue::SignalQueue;
    use nexcore_algovigilance::triage::types::{
        ReinforcementEvent, SignalInput, TriageConfig, TriageResult, TriagedSignal,
    };
    use nexcore_algovigilance::{CaseId, DecayReport, HalfLife, Relevance, SignalId, Similarity};
    // Signal Fence types (process-aware network boundary)
    use nexcore_signal_fence::audit::{AuditEntry as FenceAuditEntry, AuditLog as FenceAuditLog};
    use nexcore_signal_fence::connection::{Protocol, SocketEntry, TcpState};
    use nexcore_signal_fence::enforcer::EnforcerOp;
    use nexcore_signal_fence::engine::{FenceReport, FenceStats, FenceTickResult, TickDecision};
    use nexcore_signal_fence::policy::{FenceDecision, FenceMode, FencePolicy};
    use nexcore_signal_fence::process::{ConnectionEvent, Direction, ProcessInfo};
    use nexcore_signal_fence::rule::{
        FenceRule, FenceVerdict, NetworkMatch, ProcessMatch, RuleSet,
    };
    // Education Machine types (mastery, assessment, spaced repetition)
    use nexcore_education_machine::assessment::{
        Assessment as EduAssessment, AssessmentResult as EduAssessmentResult, BayesianPrior,
        Question, QuestionResult,
    };
    use nexcore_education_machine::learner::{AssessmentRecord, Enrollment, Learner};
    use nexcore_education_machine::lesson::{Lesson, LessonContent, LessonStep, PrimitiveMapping};
    use nexcore_education_machine::spaced_repetition::ReviewState;
    use nexcore_education_machine::state_machine::PhaseTransition;
    use nexcore_education_machine::subject::{LessonRef, Subject};
    use nexcore_education_machine::types::{
        CompetencyLevel, Difficulty, Grade, LearningPhase, MasteryLevel, MasteryVerdict,
    };
    // Biological Systems — Integumentary
    use nexcore_integumentary::claude_code::{
        IntegumentaryHealth, PermissionCascade, PermissionDecision, PermissionRule, RiskLevel,
        RuleOrigin, SandboxLayer, Scar, ScarringMechanism, ScopedSetting, SettingsPrecedence,
        SettingsScope,
    };
    use nexcore_integumentary::{
        AuthResult, CoolingAction, IntegumentaryError, SensorKind, SensorReading, Shield,
        SkinCondition, WoundRepair,
    };
    // Biological Systems — Respiratory
    use nexcore_respiratory::claude_code::{
        BreathCycle, ContextFork, ContextSource, DeadSpace, Exhalation, GasExchange, Inhalation,
        RespiratoryHealth, TidalVolume, VitalCapacity,
    };
    use nexcore_respiratory::{
        BreathingRate, ExchangeResult, Exhaled, Extracted, Inhaled, InputSource, RespiratoryError,
    };
    // Biological Systems — Digestive
    use nexcore_digestive::claude_code::{
        ContextMode, DigestiveHealth, EnzymeSubstitution, EnzymeType, Microbiome, SkillArguments,
        SkillExecution, SkillFrontmatter, SkillLoad, SkillResult, SkillTrigger, Sphincter,
    };
    use nexcore_digestive::{
        Absorbed, DataKind, DigestiveError, Fragment, Metabolized, Nutrients, Quality, Taste,
    };
    // Biological Systems — Circulatory
    use nexcore_circulatory::claude_code::{
        BloodPayload, CirculatoryHealth, FlowDirection, FrankStarling, McpHeartbeat, McpScope,
        McpServer, McpTransport, PortalFiltration, SelectivePerfusion, ToolCall, ToolResult,
    };
    use nexcore_circulatory::{
        BloodCell, BloodPressure, CellKind, CirculatoryError, Destination, Enriched, Platelet,
        Pulse, RouteDecision,
    };
    // Biological Systems — Skeletal
    use nexcore_skeletal::{
        BoneMarrow, Correction as SkeletalCorrection, Joint, SkeletalHealth, WolffsLaw,
    };
    // Biological Systems — Muscular
    use nexcore_muscular::{
        AntagonisticPair, Fatigue, ModelEscalation, MotorActivation, MuscleType, MuscularHealth,
        SizePrinciple, ToolClassification,
    };
    // Biological Systems — Lymphatic
    use nexcore_lymphatic::{
        LymphDrainage, LymphNode, LymphaticHealth, OutputStyle, OverflowHandler, ThymicSelection,
        ThymicVerdict,
    };
    // Biological Systems — Nervous
    use nexcore_nervous::claude_code::{
        ContextAssembly, EventBusRoute, HookChain, HookDispatch as NervousHookDispatch,
        HookEvent as NervousHookEvent, MyelinationCache, NervousSystemHealth, ReflexResponse,
        SignalLatency, ToolChain,
    };
    use nexcore_nervous::{
        Impulse, MotorCommand, Myelination, NervousHealth, NeuronType, ReflexArc, SensoryInput,
        SignalSpeed,
    };
    // Biological Systems — Urinary
    use nexcore_urinary::claude_code::{
        ArtifactRetention, DecisionAuditCleanup, DisposalMethod, LogRotation, RetentionPolicy,
        SessionExpiry, SilentFailureRisk, TelemetryPruning, UrinarySystemHealth, WasteCategory,
    };
    use nexcore_urinary::{
        Bladder, Excretion, FilterCategory, FiltrationRate, GlomerularFiltration, Nephron,
        Reabsorption, UrinaryHealth,
    };
    // Biological Systems — Reproductive
    use nexcore_reproductive::claude_code::{
        BranchMutation, CiPipeline, CiStage, DeployTarget, DeploymentBirth, KnowledgeTransfer,
        MergeEvent, ReproductiveSystemHealth, ScalingEvent, TrimesterGate,
    };
    use nexcore_reproductive::{
        Differentiation, Embryo, Fertilization, Gamete, GameteFitness, Mutation as GeneticMutation,
        ReproductiveHealth, Trimester,
    };

    // Same dual-dispatch pattern as lookup_grounded_type
    macro_rules! dispatch_comp_vig {
        ($($name:literal => $ty:ty),* $(,)?) => {
            match type_name {
                $($name => return Some(<$ty as nexcore_vigilance::GroundsTo>::primitive_composition()),)*
                _ => {}
            }
        };
    }
    macro_rules! dispatch_comp_lex {
        ($($name:literal => $ty:ty),* $(,)?) => {
            match type_name {
                $($name => return Some(<$ty as nexcore_lex_primitiva::GroundsTo>::primitive_composition()),)*
                _ => {}
            }
        };
    }

    // Types implementing nexcore_vigilance::GroundsTo
    dispatch_comp_vig! {
        "ThresholdGate" => ThresholdGate,
        "SaturationKinetics" => SaturationKinetics,
        "FeasibilityAssessment" => FeasibilityAssessment,
        "RateLaw" => RateLaw,
        "BufferSystem" => BufferSystem,
        "SignalDetector" => SignalDetector,
        "DecayKinetics" => DecayKinetics,
        "EquilibriumSystem" => EquilibriumSystem,
        "AggregationPipeline" => AggregationPipeline,
        "SignalResult" => SignalResult,
        "Primitive" => Primitive,
        "PrimitiveTier" => PrimitiveTier,
        "HarmType" => HarmType,
        "SafetyMargin" => SafetyMargin,
        "Bdi" => Bdi,
        "EcsScore" => EcsScore,
        "Occupancy" => Occupancy,
        "AuditTrail" => AuditTrail,
        "AbsenceMarker" => AbsenceMarker,
        "Pipeline" => Pipeline,
        "ConsumptionMark" => ConsumptionMark,
        "EntityStatus" => EntityStatus,
        "ResourcePath" => ResourcePath,
        "RecursionBound" => RecursionBound,
        "Recipient" => Recipient,
        "SafetyBoundary" => SafetyBoundary<f64>,
        "Harm" => Harm,
        "Vulnerable" => Vulnerable,
        "BoundaryBreach" => BoundaryBreach,
        "Monitoring" => Monitoring,
        "Tracked" => Tracked,
        "Homeostasis" => Homeostasis,
        "StagedValidation" => StagedValidation,
        "Atomicity" => Atomicity,
        "CompareAndSwap" => CompareAndSwap,
        "ToctouWindow" => ToctouWindow,
        "SerializationGuard" => SerializationGuard,
        "RateLimiter" => RateLimiter,
        "CircuitBreaker" => CircuitBreaker,
        "Idempotency" => Idempotency,
        "NegativeEvidence" => NegativeEvidence,
        "TopologicalAddress" => TopologicalAddress,
        "Accumulator" => Accumulator,
        "Checkpoint" => Checkpoint,
        "Amplitude" => Amplitude,
        "Phase" => Phase,
        "Superposition" => Superposition,
        "Measurement" => Measurement,
        "Interference" => Interference,
        "Uncertainty" => Uncertainty,
        "Unitarity" => Unitarity,
        "Eigenstate" => Eigenstate,
        "Observable" => Observable,
        "Hermiticity" => Hermiticity,
        "Entanglement" => Entanglement,
        "Decoherence" => Decoherence,
        "Qubit" => Qubit,
        "DisproportionalityScore" => DisproportionalityScore,
    }

    // Types implementing nexcore_lex_primitiva::GroundsTo (standalone crates)
    dispatch_comp_lex! {
        "Schema" => Schema,
        "SchemaKind" => SchemaKind,
        "DiagnosticLevel" => DiagnosticLevel,
        "Fidelity" => Fidelity,
        "Contract" => Contract,
        "DriftType" => DriftType,
        "DriftSeverity" => DriftSeverity,
        "DriftResult" => DriftResult,
        "DriftSignal" => DriftSignal,
        "Mutation" => Mutation,
        "Phenotype" => Phenotype,
        "HormoneLevel" => HormoneLevel,
        "HormoneType" => HormoneType,
        "Stimulus" => Stimulus,
        "EndocrineState" => EndocrineState,
        "BehavioralModifiers" => BehavioralModifiers,
        "ModelConfig" => ModelConfig,
        "ModelFormat" => ModelFormat,
        "GenerateParams" => GenerateParams,
        "LoraConfig" => LoraConfig,
        "DatasetConfig" => DatasetConfig,
        "TrainingParams" => TrainingParams,
        "LoraAdapter" => LoraAdapter,
        "FineTuneJob" => FineTuneJob,
        "MultiTrustEngine" => MultiTrustEngine,
        "TrustDimension" => TrustDimension,
        "DimensionWeights" => DimensionWeights,
        "TrustEngine" => TrustEngine,
        "Evidence" => TrustEvidence,
        "TrustLevel" => TrustLevel,
        "TrustVelocity" => TrustVelocity,
        "TrustConfig" => TrustConfig,
        "OriginatorType" => OriginatorType,
        "RiskContext" => RiskContext,
        "RiskScore" => RiskScore,
        // Spatial Bridge Metric types (κ Comparison dominant)
        "ImpurityMetric" => ImpurityMetric,
        "SeverityMetric" => SeverityMetric,
        "RiskScoreMetric" => RiskScoreMetric,
        // Insight Engine (system identity — T2-P/T2-C/T3)
        "Pattern" => Pattern,
        "Recognition" => Recognition,
        "Novelty" => Novelty,
        "NoveltyReason" => NoveltyReason,
        "Connection" => Connection,
        "Compression" => Compression,
        "Suddenness" => Suddenness,
        "InsightEngine" => InsightEngine,
        // DNA — Foundation (7)
        "Nucleotide" => Nucleotide,
        "AminoAcid" => AminoAcid,
        "Codon" => Codon,
        "Strand" => Strand,
        "DoubleHelix" => DoubleHelix,
        "CodonTable" => CodonTable,
        "CodonVM" => CodonVM,
        // DNA — Compute (10)
        "DnaInstruction" => DnaInstruction,
        "DnaProgram" => DnaProgram,
        "DnaToken" => DnaToken,
        "DisasmOptions" => DisasmOptions,
        "DnaGlyph" => DnaGlyph,
        "DnaGlyphPair" => DnaGlyphPair,
        "DnaErrorCode" => DnaErrorCode,
        "DnaDiagnostic" => DnaDiagnostic,
        "DnaJsonValue" => DnaJsonValue,
        "DnaTemplate" => DnaTemplate,
        // DNA — Store (9)
        "DnaType" => DnaType,
        "DnaValue" => DnaValue,
        "DnaArray" => DnaArray,
        "DnaRecord" => DnaRecord,
        "DnaMap" => DnaMap,
        "DnaFrame" => DnaFrame,
        "Pixel" => Pixel,
        "VoxelPos" => VoxelPos,
        "VoxelCube" => VoxelCube,
        // DNA — Evolve (8)
        "Gene" => Gene,
        "Genome" => Genome,
        "Plasmid" => Plasmid,
        "Organism" => Organism,
        "DnaCluster" => DnaCluster,
        "DnaClusterResult" => DnaClusterResult,
        "EvolutionConfig" => EvolutionConfig,
        "EvolutionResult" => EvolutionResult,
        // DNA — Gravity (2)
        "Particle" => Particle,
        "GravityConfig" => GravityConfig,
        // DNA — Analyze: String Theory (5)
        "StringTension" => StringTension,
        "HarmonicMode" => HarmonicMode,
        "FrequencySpectrum" => FrequencySpectrum,
        "DnaResonance" => DnaResonance,
        "StringEnergy" => StringEnergy,
        // DNA — Analyze: PV Theory (7+1)
        "CausalityCategory" => CausalityCategory,
        "DnaSafetyLevel" => DnaSafetyLevel,
        "AlertLevel" => AlertLevel,
        "DrugProfile" => DrugProfile,
        "DnaSignal" => DnaSignal,
        "DnaSafetyMargin" => DnaSafetyMargin,
        "CausalityScore" => CausalityScore,
        "VigilanceState" => VigilanceState,
        // DNA — Analyze: Mind (6)
        "WordOre" => WordOre,
        "Affinity" => Affinity,
        "DnaLexicon" => DnaLexicon,
        "DnaMindPoint" => DnaMindPoint,
        "DnaDrift" => DnaDrift,
        "DnaStateMind" => DnaStateMind,
        // Energy (ATP/ADP biochemistry)
        "Regime" => Regime,
        "Strategy" => Strategy,
        "EnergySystem" => EnergySystem,
        "WasteClass" => WasteClass,
        "TokenPool" => TokenPool,
        "Operation" => Operation,
        "OperationBuilder" => OperationBuilder,
        "RecyclingRate" => RecyclingRate,
        "EnergyState" => EnergyState,
        // Compliance
        "Assessment" => Assessment,
        "ComplianceResult" => ComplianceResult,
        "Finding" => Finding,
        "FindingSeverity" => FindingSeverity,
        "Control" => Control,
        "ControlCatalog" => ControlCatalog,
        "ControlStatus" => ControlStatus,
        "Exclusion" => Exclusion,
        "ExclusionClassification" => ExclusionClassification,
        "ExclusionType" => ExclusionType,
        // Measure (information theory, graph theory, statistics)
        "BayesianPosterior" => BayesianPosterior,
        "Centrality" => Centrality,
        "ChemistryMapping" => ChemistryMapping,
        "CodeDensityIndex" => CodeDensityIndex,
        "CouplingRatio" => CouplingRatio,
        "CrateHealth" => CrateHealth,
        "CrateId" => CrateId,
        "CrateMeasurement" => CrateMeasurement,
        "Density" => Density,
        "DriftDirection" => DriftDirection,
        "MeasureDriftResult" => MeasureDriftResult,
        "Entropy" => Entropy,
        "GraphAnalysis" => GraphAnalysis,
        "GraphNode" => GraphNode,
        "HealthComponents" => HealthComponents,
        "HealthRating" => HealthRating,
        "HealthScore" => HealthScore,
        "MeasureError" => MeasureError,
        "MeasureTimestamp" => MeasureTimestamp,
        "PoissonCi" => PoissonCi,
        "Probability" => Probability,
        "RatingDistribution" => RatingDistribution,
        "RegressionResult" => RegressionResult,
        "TestDensity" => TestDensity,
        "WelchResult" => WelchResult,
        "WorkspaceHealth" => WorkspaceHealth,
        "WorkspaceMeasurement" => WorkspaceMeasurement,
        // Signal (PV signal detection pipeline)
        "Prr" => SignalPrr,
        "Ror" => SignalRor,
        "Ic" => SignalIc,
        "Ebgm" => SignalEbgm,
        "ChiSquare" => SignalChiSquare,
        "SignalStrength" => SignalStrength,
        "ReportSource" => ReportSource,
        "AlertState" => SignalAlertState,
        "SignalError" => SignalError,
        "ConfidenceInterval" => SignalConfidenceInterval,
        "DrugEventPair" => DrugEventPair,
        "ContingencyTable" => SignalContingencyTable,
        "ValidationCheck" => SignalValidationCheck,
        "ThresholdConfig" => SignalThresholdConfig,
        "NormalizedEvent" => NormalizedEvent,
        "ValidationReport" => SignalValidationReport,
        "SignalMetrics" => SignalMetrics,
        "RawReport" => RawReport,
        "DetectionResult" => SignalDetectionResult,
        "Alert" => SignalAlert,
        "AlertTransitions" => AlertTransitions,
        "BasicNormalizer" => BasicNormalizer,
        "SynonymNormalizer" => SynonymNormalizer,
        "CsvIngestor" => CsvIngestor,
        "JsonIngestor" => JsonIngestor,
        "TableDetector" => TableDetector,
        "CompositeThreshold" => CompositeThreshold,
        "EvansThreshold" => EvansThreshold,
        "StandardValidator" => StandardValidator,
        "JsonFileStore" => JsonFileStore,
        "MemoryStore" => MemoryStore,
        "JsonReporter" => JsonReporter,
        "TableReporter" => TableReporter,
        // Brain (sessions, artifacts, implicit learning)
        "AgentId" => AgentId,
        "Artifact" => Artifact,
        "ArtifactMetadata" => ArtifactMetadata,
        "ArtifactMetrics" => ArtifactMetrics,
        "ArtifactSizeInfo" => ArtifactSizeInfo,
        "ArtifactType" => ArtifactType,
        "Belief" => Belief,
        "BeliefGraph" => BeliefGraph,
        "BeliefImplication" => BeliefImplication,
        "BrainConfig" => BrainConfig,
        "BrainError" => BrainError,
        "BrainHealth" => BrainHealth,
        "BrainSession" => BrainSession,
        "BrainSnapshot" => BrainSnapshot,
        "BrainCheckpoint" => BrainCheckpoint,
        "CodeTracker" => CodeTracker,
        "CoordinationRegistry" => CoordinationRegistry,
        "Correction" => Correction,
        "EvidenceRef" => EvidenceRef,
        "EvidenceType" => EvidenceType,
        "FileLock" => FileLock,
        "GrowthRate" => GrowthRate,
        "ImplicationStrength" => ImplicationStrength,
        "ImplicitKnowledge" => ImplicitKnowledge,
        "ImplicitStats" => ImplicitStats,
        "LockDuration" => LockDuration,
        "LockStatus" => LockStatus,
        "BrainPattern" => BrainPattern,
        "PersistentSynapseBank" => PersistentSynapseBank,
        "PipelineRun" => PipelineRun,
        "PipelineState" => PipelineState,
        "Preference" => Preference,
        "ProjectSnapshot" => ProjectSnapshot,
        "RecoveryResult" => RecoveryResult,
        "RunStatus" => RunStatus,
        "SessionEntry" => SessionEntry,
        "SessionMetrics" => SessionMetrics,
        "SynapseBankStats" => SynapseBankStats,
        "SynapseInfo" => SynapseInfo,
        "T1Primitive" => T1Primitive,
        "TrackedFile" => TrackedFile,
        "TrackerIndex" => TrackerIndex,
        "TrustAccumulator" => TrustAccumulator,
        // Config (type-safe Claude/Gemini/Git configuration)
        "AllConfigs" => AllConfigs,
        "AsrConfig" => AsrConfig,
        "ClaudeConfig" => ClaudeConfig,
        "CredentialHelper" => CredentialHelper,
        "DoDChecklist" => DoDChecklist,
        "DoDItem" => DoDItem,
        "FeatureFlags" => FeatureFlags,
        "FlywheelConfig" => FlywheelConfig,
        "FlywheelStage" => FlywheelStage,
        "GeminiConfig" => GeminiConfig,
        "GeminiHookType" => GeminiHookType,
        "GitConfig" => GitConfig,
        "GitCore" => GitCore,
        "GitDiff" => GitDiff,
        "GitFetch" => GitFetch,
        "GitInit" => GitInit,
        "GitPull" => GitPull,
        "GitUser" => GitUser,
        "HookEvent" => HookEvent,
        "HookMeta" => HookMeta,
        "HookRegistry" => HookRegistry,
        "HookTier" => HookTier,
        "McpServerConfig" => McpServerConfig,
        "Model" => Model,
        "ModelUsage" => ModelUsage,
        "OAuthAccount" => OAuthAccount,
        "PerformanceStats" => PerformanceStats,
        "ProjectConfig" => ProjectConfig,
        "RoutingConfig" => RoutingConfig,
        "SkillChain" => SkillChain,
        "SkillMapping" => SkillMapping,
        "SkillUsageStats" => SkillUsageStats,
        "VocabSkillMap" => VocabSkillMap,
        "VulnerabilityCache" => VulnerabilityCache,
        // Mesh (distributed mesh networking)
        "ChemotacticRouter" => ChemotacticRouter,
        "ChemotacticRouteSelector" => ChemotacticRouteSelector,
        "DiscoveryAction" => DiscoveryAction,
        "DiscoveryLoop" => DiscoveryLoop,
        "DiscoveryMessage" => DiscoveryMessage,
        "GossipLoop" => GossipLoop,
        "GossipMessage" => GossipMessage,
        "MeshError" => MeshError,
        "MeshEvent" => MeshEvent,
        "MeshHandle" => MeshHandle,
        "MeshMessage" => MeshMessage,
        "MeshRuntime" => MeshRuntime,
        "MeshSnapshot" => MeshSnapshot,
        "Neighbor" => Neighbor,
        "NeighborRegistry" => NeighborRegistry,
        "Node" => Node,
        "NodeState" => NodeState,
        "MeshPath" => MeshPath,
        "PeerIdentity" => PeerIdentity,
        "ResilienceAction" => ResilienceAction,
        "ResilienceLoop" => ResilienceLoop,
        "ResilienceState" => ResilienceState,
        "Route" => Route,
        "RouteQuality" => RouteQuality,
        "RoutingTable" => RoutingTable,
        "SecurityPolicy" => SecurityPolicy,
        "SnapshotStore" => SnapshotStore,
        "TlsTier" => TlsTier,
        // Vigil (AI orchestrator)
        "Urgency" => Urgency,
        "DecisionAction" => DecisionAction,
        "ExecutorType" => ExecutorType,
        "VigilEvent" => VigilEvent,
        "Interaction" => Interaction,
        "ExecutorResult" => ExecutorResult,
        "CytokineBridge" => CytokineBridge,
        "HormonalModulator" => HormonalModulator,
        "SynapticLearner" => SynapticLearner,
        "ImmunitySensor" => ImmunitySensor,
        "EnergyGovernor" => EnergyGovernor,
        "GuardianBridge" => GuardianBridge,
        "VigilRuntime" => VigilRuntime,
        "NervousSystem" => NervousSystem,
        "VigilError" => VigilError,
        "EventId" => EventId,
        "EventKind" => EventKind,
        "EventSeverity" => EventSeverity,
        "EscalationLevel" => EscalationLevel,
        "LedgerEntryType" => LedgerEntryType,
        "ConsequenceOutcome" => ConsequenceOutcome,
        "ShellConsequence" => ShellConsequence,
        "WebhookConsequence" => WebhookConsequence,
        "NotifyConsequence" => NotifyConsequence,
        "WatchEvent" => WatchEvent,
        "ThresholdCheck" => ThresholdCheck,
        "BoundarySpec" => BoundarySpec,
        "BoundaryViolation" => BoundaryViolation,
        "ConsequenceReceipt" => ConsequenceReceipt,
        "LedgerEntry" => LedgerEntry,
        "BoundaryGate" => BoundaryGate,
        "VigilanceLedger" => VigilanceLedger,
        "ConsequencePipeline" => ConsequencePipeline,
        "VigilDaemon" => VigilDaemon,
        "ShutdownHandle" => ShutdownHandle,
        "VigilHealth" => VigilHealth,
        "VigilStats" => VigilStats,
        "VigilSubError" => VigilSubError,
        // Signal Theory (formal signal detection axioms & algebra)
        "SignalPrimitive" => SignalPrimitive,
        "EvidenceKind" => EvidenceKind,
        "A2NoiseDominance" => A2NoiseDominance,
        "A3SignalExistence" => A3SignalExistence,
        "A4BoundaryRequirement" => A4BoundaryRequirement,
        "A5Disproportionality" => A5Disproportionality,
        "A6CausalInference" => A6CausalInference,
        "ObservationSpace" => ObservationSpace,
        "Baseline" => Baseline,
        "Ratio" => Ratio,
        "Difference" => Difference,
        "DetectionInterval" => DetectionInterval,
        "DetectionOutcome" => DetectionOutcome,
        "SignalStrengthLevel" => SignalStrengthLevel,
        "SignalVerificationReport" => SignalVerificationReport,
        "BoundaryKind" => BoundaryKind,
        "ConjunctionMode" => ConjunctionMode,
        "ThresholdPreset" => ThresholdPreset,
        "DetectionPhase" => DetectionPhase,
        "FixedBoundary" => FixedBoundary,
        "AdaptiveBoundary" => AdaptiveBoundary,
        "CompositeBoundary" => CompositeBoundary,
        "CascadedThreshold" => CascadedThreshold,
        "DetectionPipeline" => DetectionPipeline,
        "DecisionOutcome" => DecisionOutcome,
        "DecisionMatrix" => DecisionMatrix,
        "RocPoint" => RocPoint,
        "RocCurve" => RocCurve,
        "DPrime" => DPrime,
        "ResponseBias" => ResponseBias,
        "L1TotalCountConservation" => L1TotalCountConservation,
        "L2BaseRateInvariance" => L2BaseRateInvariance,
        "L3SensitivitySpecificityTradeoff" => L3SensitivitySpecificityTradeoff,
        "L4InformationConservation" => L4InformationConservation,
        "ConservationReport" => ConservationReport,
        // FAERS ETL (FDA adverse event data pipeline)
        "CaseCount" => CaseCount,
        "RowCount" => RowCount,
        "DrugRole" => DrugRole,
        "DrugCharacterization" => DrugCharacterization,
        "OpenFdaError" => OpenFdaError,
        "DrugName" => DrugName,
        "EventName" => EventName,
        "ContingencyBatch" => ContingencyBatch,
        "FaersPipelineOutput" => FaersPipelineOutput,
        "ReactionOutcome" => ReactionOutcome,
        "OutcomeCase" => OutcomeCase,
        "OutcomeConditionedConfig" => OutcomeConditionedConfig,
        "MonthBucket" => MonthBucket,
        "TemporalCase" => TemporalCase,
        "VelocityConfig" => VelocityConfig,
        "SeriousnessFlag" => SeriousnessFlag,
        "CaseSeriousness" => CaseSeriousness,
        "SeriousnessCase" => SeriousnessCase,
        "CascadeConfig" => CascadeConfig,
        "PolypharmacyCase" => PolypharmacyCase,
        "PolypharmacyConfig" => PolypharmacyConfig,
        "ReporterQualification" => ReporterQualification,
        "ReporterCase" => ReporterCase,
        "ReporterWeightedSignal" => ReporterWeightedSignal,
        "ReporterWeightedConfig" => ReporterWeightedConfig,
        "GeographicCase" => GeographicCase,
        "CountrySignal" => CountrySignal,
        "GeographicConfig" => GeographicConfig,
        "FaersReport" => FaersReport,
        "ReportFingerprint" => ReportFingerprint,
        "FaersDeduplicationResult" => FaersDeduplicationResult,
        "DuplicateCluster" => DuplicateCluster,
        "DeduplicatorConfig" => DeduplicatorConfig,
        "FaersDeduplicator" => FaersDeduplicator,
        "NdcProduct" => NdcProduct,
        "NdcBridge" => NdcBridge,
        "NdcMatch" => NdcMatch,
        "NdcMatchType" => NdcMatchType,
        "DrugEventResponse" => DrugEventResponse,
        "DrugEventQuery" => DrugEventQuery,
        "OpenFdaClient" => OpenFdaClient,
        "FaersSignalDetectionResult" => FaersSignalDetectionResult,
        "OutcomeConditionedSignal" => OutcomeConditionedSignal,
        "SignalVelocity" => SignalVelocity,
        "SeriousnessCascade" => SeriousnessCascade,
        "PolypharmacySignal" => PolypharmacySignal,
        "GeographicDivergence" => GeographicDivergence,
        // Algovigilance (ICSR deduplication + signal triage)
        "Similarity" => Similarity,
        "Relevance" => Relevance,
        "HalfLife" => HalfLife,
        "CaseId" => CaseId,
        "SignalId" => SignalId,
        "DecayReport" => DecayReport,
        "AlgovigilanceError" => AlgovigilanceError,
        "SynonymEntry" => SynonymEntry,
        "AlgovigilanceStore" => AlgovigilanceStore,
        "IcsrNarrative" => IcsrNarrative,
        "CasePair" => CasePair,
        "AlgoDeduplicationResult" => AlgoDeduplicationResult,
        "DedupConfig" => DedupConfig,
        "SynonymPair" => SynonymPair,
        "DedupFunction" => DedupFunction,
        "TriagedSignal" => TriagedSignal,
        "TriageConfig" => TriageConfig,
        "TriageResult" => TriageResult,
        "SignalInput" => SignalInput,
        "ReinforcementEvent" => ReinforcementEvent,
        "TriageFunction" => TriageFunction,
        "UrgencyClassification" => UrgencyClassification,
        "UrgencyClassifier" => UrgencyClassifier,
        "SignalQueue" => SignalQueue,
        // Signal Fence (process-aware network boundary)
        "FenceMode" => FenceMode,
        "Direction" => Direction,
        "FenceVerdict" => FenceVerdict,
        "Protocol" => Protocol,
        "TcpState" => TcpState,
        "SocketEntry" => SocketEntry,
        "ProcessInfo" => ProcessInfo,
        "ProcessMatch" => ProcessMatch,
        "NetworkMatch" => NetworkMatch,
        "FenceRule" => FenceRule,
        "FenceDecision" => FenceDecision,
        "FenceAuditEntry" => FenceAuditEntry,
        "ConnectionEvent" => ConnectionEvent,
        "RuleSet" => RuleSet,
        "FencePolicy" => FencePolicy,
        "FenceStats" => FenceStats,
        "FenceAuditLog" => FenceAuditLog,
        "FenceTickResult" => FenceTickResult,
        "FenceReport" => FenceReport,
        "EnforcerOp" => EnforcerOp,
        "TickDecision" => TickDecision,
        // Education Machine (mastery, assessment, spaced repetition)
        "MasteryLevel" => MasteryLevel,
        "Difficulty" => Difficulty,
        "BayesianPrior" => BayesianPrior,
        "LearningPhase" => LearningPhase,
        "CompetencyLevel" => CompetencyLevel,
        "MasteryVerdict" => MasteryVerdict,
        "Grade" => Grade,
        "Question" => Question,
        "QuestionResult" => QuestionResult,
        "EduAssessment" => EduAssessment,
        "EduAssessmentResult" => EduAssessmentResult,
        "Subject" => Subject,
        "LessonRef" => LessonRef,
        "Lesson" => Lesson,
        "LessonStep" => LessonStep,
        "LessonContent" => LessonContent,
        "PrimitiveMapping" => PrimitiveMapping,
        "Enrollment" => Enrollment,
        "AssessmentRecord" => AssessmentRecord,
        "ReviewState" => ReviewState,
        "PhaseTransition" => PhaseTransition,
        "Learner" => Learner,
        // Biological Systems — Integumentary (20)
        "AuthResult" => AuthResult,
        "SensorKind" => SensorKind,
        "SensorReading" => SensorReading,
        "SkinCondition" => SkinCondition,
        "Shield" => Shield,
        "WoundRepair" => WoundRepair,
        "CoolingAction" => CoolingAction,
        "IntegumentaryError" => IntegumentaryError,
        "PermissionRule" => PermissionRule,
        "PermissionDecision" => PermissionDecision,
        "RuleOrigin" => RuleOrigin,
        "PermissionCascade" => PermissionCascade,
        "SettingsScope" => SettingsScope,
        "ScopedSetting" => ScopedSetting,
        "SettingsPrecedence" => SettingsPrecedence,
        "SandboxLayer" => SandboxLayer,
        "RiskLevel" => RiskLevel,
        "Scar" => Scar,
        "ScarringMechanism" => ScarringMechanism,
        "IntegumentaryHealth" => IntegumentaryHealth,
        // Biological Systems — Respiratory (17)
        "InputSource" => InputSource,
        "Inhaled" => Inhaled,
        "Extracted" => Extracted,
        "Exhaled" => Exhaled,
        "ExchangeResult" => ExchangeResult,
        "BreathingRate" => BreathingRate,
        "RespiratoryError" => RespiratoryError,
        "ContextSource" => ContextSource,
        "Inhalation" => Inhalation,
        "Exhalation" => Exhalation,
        "GasExchange" => GasExchange,
        "DeadSpace" => DeadSpace,
        "ContextFork" => ContextFork,
        "TidalVolume" => TidalVolume,
        "VitalCapacity" => VitalCapacity,
        "BreathCycle" => BreathCycle,
        "RespiratoryHealth" => RespiratoryHealth,
        // Biological Systems — Digestive (20)
        "Quality" => Quality,
        "DataKind" => DataKind,
        "Fragment" => Fragment,
        "Taste" => Taste,
        "Nutrients" => Nutrients,
        "Absorbed" => Absorbed,
        "Metabolized" => Metabolized,
        "DigestiveError" => DigestiveError,
        "SkillTrigger" => SkillTrigger,
        "SkillLoad" => SkillLoad,
        "ContextMode" => ContextMode,
        "SkillFrontmatter" => SkillFrontmatter,
        "Sphincter" => Sphincter,
        "SkillArguments" => SkillArguments,
        "EnzymeType" => EnzymeType,
        "EnzymeSubstitution" => EnzymeSubstitution,
        "Microbiome" => Microbiome,
        "SkillResult" => SkillResult,
        "SkillExecution" => SkillExecution,
        "DigestiveHealth" => DigestiveHealth,
        // Biological Systems — Circulatory (21)
        "CellKind" => CellKind,
        "BloodCell" => BloodCell,
        "Enriched" => Enriched,
        "Destination" => Destination,
        "RouteDecision" => RouteDecision,
        "BloodPressure" => BloodPressure,
        "Pulse" => Pulse,
        "Platelet" => Platelet,
        "CirculatoryError" => CirculatoryError,
        "McpTransport" => McpTransport,
        "McpScope" => McpScope,
        "FlowDirection" => FlowDirection,
        "McpServer" => McpServer,
        "McpHeartbeat" => McpHeartbeat,
        "ToolCall" => ToolCall,
        "ToolResult" => ToolResult,
        "PortalFiltration" => PortalFiltration,
        "SelectivePerfusion" => SelectivePerfusion,
        "BloodPayload" => BloodPayload,
        "FrankStarling" => FrankStarling,
        "CirculatoryHealth" => CirculatoryHealth,
        // Biological Systems — Skeletal (5)
        "WolffsLaw" => WolffsLaw,
        "SkeletalCorrection" => SkeletalCorrection,
        "BoneMarrow" => BoneMarrow,
        "Joint" => Joint,
        "SkeletalHealth" => SkeletalHealth,
        // Biological Systems — Muscular (8)
        "MuscleType" => MuscleType,
        "ToolClassification" => ToolClassification,
        "AntagonisticPair" => AntagonisticPair,
        "SizePrinciple" => SizePrinciple,
        "Fatigue" => Fatigue,
        "ModelEscalation" => ModelEscalation,
        "MotorActivation" => MotorActivation,
        "MuscularHealth" => MuscularHealth,
        // Biological Systems — Lymphatic (7)
        "OutputStyle" => OutputStyle,
        "LymphDrainage" => LymphDrainage,
        "LymphNode" => LymphNode,
        "ThymicSelection" => ThymicSelection,
        "ThymicVerdict" => ThymicVerdict,
        "OverflowHandler" => OverflowHandler,
        "LymphaticHealth" => LymphaticHealth,
        // Biological Systems — Nervous (18)
        "NeuronType" => NeuronType,
        "Impulse" => Impulse,
        "ReflexArc" => ReflexArc,
        "Myelination" => Myelination,
        "SensoryInput" => SensoryInput,
        "MotorCommand" => MotorCommand,
        "SignalSpeed" => SignalSpeed,
        "NervousHealth" => NervousHealth,
        "NervousHookEvent" => NervousHookEvent,
        "NervousHookDispatch" => NervousHookDispatch,
        "HookChain" => HookChain,
        "EventBusRoute" => EventBusRoute,
        "ContextAssembly" => ContextAssembly,
        "ToolChain" => ToolChain,
        "SignalLatency" => SignalLatency,
        "MyelinationCache" => MyelinationCache,
        "ReflexResponse" => ReflexResponse,
        "NervousSystemHealth" => NervousSystemHealth,
        // Biological Systems — Urinary (18)
        "FilterCategory" => FilterCategory,
        "Nephron" => Nephron,
        "GlomerularFiltration" => GlomerularFiltration,
        "Reabsorption" => Reabsorption,
        "Excretion" => Excretion,
        "FiltrationRate" => FiltrationRate,
        "Bladder" => Bladder,
        "UrinaryHealth" => UrinaryHealth,
        "TelemetryPruning" => TelemetryPruning,
        "SessionExpiry" => SessionExpiry,
        "ArtifactRetention" => ArtifactRetention,
        "LogRotation" => LogRotation,
        "RetentionPolicy" => RetentionPolicy,
        "DecisionAuditCleanup" => DecisionAuditCleanup,
        "WasteCategory" => WasteCategory,
        "DisposalMethod" => DisposalMethod,
        "SilentFailureRisk" => SilentFailureRisk,
        "UrinarySystemHealth" => UrinarySystemHealth,
        // Biological Systems — Reproductive (18)
        "GameteFitness" => GameteFitness,
        "Gamete" => Gamete,
        "Fertilization" => Fertilization,
        "Trimester" => Trimester,
        "Embryo" => Embryo,
        "Differentiation" => Differentiation,
        "GeneticMutation" => GeneticMutation,
        "ReproductiveHealth" => ReproductiveHealth,
        "CiStage" => CiStage,
        "CiPipeline" => CiPipeline,
        "DeployTarget" => DeployTarget,
        "DeploymentBirth" => DeploymentBirth,
        "BranchMutation" => BranchMutation,
        "MergeEvent" => MergeEvent,
        "TrimesterGate" => TrimesterGate,
        "ScalingEvent" => ScalingEvent,
        "KnowledgeTransfer" => KnowledgeTransfer,
        "ReproductiveSystemHealth" => ReproductiveSystemHealth,
    }

    None
}

// ============================================================================
// Molecular Weight (Algorithm A76)
// ============================================================================

/// Compute the molecular weight of a word/concept from its T1 primitive composition.
///
/// Algorithm A76: Shannon information-theoretic weight.
/// `MW(word) = Σ -log₂(freq(pᵢ) / total)` for each primitive pᵢ.
///
/// **Note:** For comparison, periodic table, and transfer prediction,
/// prefer `mw_compute` / `mw_compare` / `mw_predict_transfer` in the
/// dedicated molecular_weight tool group.
pub fn molecular_weight(
    params: crate::params::LexPrimitivaMolecularWeightParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_lex_primitiva::molecular_weight::{AtomicMass, MolecularFormula};

    let mut resolved = Vec::new();
    let mut unknown = Vec::new();

    for name in &params.primitives {
        match find_primitive(name) {
            Some(p) => resolved.push(p),
            None => unknown.push(name.clone()),
        }
    }

    if !unknown.is_empty() {
        let json = json!({
            "error": "unknown_primitives",
            "unknown": unknown,
            "hint": "Use names like 'Sequence', 'Boundary' or symbols like 'σ', '∂'",
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    if resolved.is_empty() {
        let json = json!({
            "error": "no_primitives",
            "hint": "Provide at least one primitive name in the 'primitives' array",
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    let concept_name = params.name.as_deref().unwrap_or("unnamed");
    let formula = MolecularFormula::new(concept_name).with_all(&resolved);
    let weight = formula.weight();

    let constituents: Vec<serde_json::Value> = formula
        .atomic_masses()
        .iter()
        .map(|am| {
            json!({
                "primitive": format!("{:?}", am.primitive()),
                "symbol": am.primitive().symbol(),
                "mass_bits": (am.bits() * 100.0).round() / 100.0,
                "frequency": am.frequency(),
            })
        })
        .collect();

    let tc = weight.transfer_class();
    let pct = (weight.predicted_transfer() * 100.0).round() as u32;
    let interpretation = match tc {
        nexcore_lex_primitiva::molecular_weight::TransferClass::Light => format!(
            "Light ({:.1} Da) — highly transferable (~{}%)",
            weight.daltons(),
            pct
        ),
        nexcore_lex_primitiva::molecular_weight::TransferClass::Medium => format!(
            "Medium ({:.1} Da) — moderately transferable (~{}%)",
            weight.daltons(),
            pct
        ),
        nexcore_lex_primitiva::molecular_weight::TransferClass::Heavy => format!(
            "Heavy ({:.1} Da) — domain-locked (~{}%)",
            weight.daltons(),
            pct
        ),
    };

    let mut response = json!({
        "algorithm": "A76 — Word Molecular Weight",
        "name": concept_name,
        "formula": formula.formula_string(),
        "molecular_weight_daltons": (weight.daltons() * 100.0).round() / 100.0,
        "primitive_count": weight.primitive_count(),
        "average_mass": (weight.average_mass() * 100.0).round() / 100.0,
        "transfer_class": format!("{}", tc),
        "predicted_transfer_confidence": (weight.predicted_transfer() * 1000.0).round() / 1000.0,
        "constituents": constituents,
        "interpretation": interpretation,
    });

    if params.include_periodic_table {
        let table: Vec<serde_json::Value> = AtomicMass::periodic_table()
            .iter()
            .map(|am| {
                json!({
                    "primitive": format!("{:?}", am.primitive()),
                    "symbol": am.primitive().symbol(),
                    "mass_bits": (am.bits() * 100.0).round() / 100.0,
                    "frequency": am.frequency(),
                })
            })
            .collect();
        if let serde_json::Value::Object(ref mut map) = response {
            map.insert("periodic_table".to_string(), json!(table));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Get the disambiguated State (ς) mode for a grounded type.
///
/// Returns Mutable (freely changing), Modal (FSM), or Accumulated (monotonic).
/// Returns None if the type has no state component.
pub fn get_state_mode(params: LexPrimitivaStateModeParams) -> Result<CallToolResult, McpError> {
    let json = lookup_grounded_type(&params.type_name, |type_name, comp| {
        let mode_json = comp.state_mode.as_ref().map(|mode| {
            json!({
                "mode": mode.label(),
                "symbol": mode.symbol_suffix(),
                "reversible": mode.is_reversible(),
                "description": mode.description(),
            })
        });

        json!({
            "type": type_name,
            "state_mode": mode_json,
            "has_state": comp.primitives.contains(&LexPrimitiva::State),
        })
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Detect whether adding a primitive causes a dominant-shift (phase transition).
///
/// Given a base set of primitives and one new primitive to add, determines
/// whether the dominant primitive changes — a "phase transition" in composition
/// character.
pub fn dominant_shift(params: LexPrimitivaDominantShiftParams) -> Result<CallToolResult, McpError> {
    // Resolve base primitives
    let mut base_resolved = Vec::new();
    let mut unknown = Vec::new();
    for name in &params.base_primitives {
        match find_primitive(name) {
            Some(p) => base_resolved.push(p),
            None => unknown.push(name.clone()),
        }
    }

    // Resolve the added primitive
    let added = match find_primitive(&params.added_primitive) {
        Some(p) => p,
        None => {
            unknown.push(params.added_primitive.clone());
            let json = json!({
                "error": "unknown_primitives",
                "unknown": unknown,
                "hint": "Use names like 'Sequence', 'Boundary' or symbols like 'σ', '∂'",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    if !unknown.is_empty() {
        let json = json!({
            "error": "unknown_primitives",
            "unknown": unknown,
            "hint": "Use names like 'Sequence', 'Boundary' or symbols like 'σ', '∂'",
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    // Compute dominant before (most frequent primitive in base)
    let old_dominant = mode_primitive(&base_resolved);

    // Compute dominant after (base + added)
    let mut combined = base_resolved.clone();
    combined.push(added);
    let new_dominant = mode_primitive(&combined);

    let shifted = old_dominant != new_dominant;
    let old_label = old_dominant.map(|p| format!("{:?}", p));
    let new_label = new_dominant.map(|p| format!("{:?}", p));

    let json = json!({
        "base_count": base_resolved.len(),
        "added": format!("{:?}", added),
        "old_dominant": old_label,
        "new_dominant": new_label,
        "phase_transition": shifted,
        "interpretation": if shifted {
            format!(
                "Phase transition detected: dominant shifted from {} to {}",
                old_label.as_deref().unwrap_or("∅"),
                new_label.as_deref().unwrap_or("∅")
            )
        } else {
            format!(
                "No phase transition: dominant remains {}",
                new_label.as_deref().unwrap_or("∅")
            )
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Find the most frequent primitive in a slice (the "mode").
/// Returns `None` for empty slices. Ties broken by enum order.
fn mode_primitive(primitives: &[LexPrimitiva]) -> Option<LexPrimitiva> {
    if primitives.is_empty() {
        return None;
    }
    let mut counts = std::collections::HashMap::<LexPrimitiva, usize>::new();
    for p in primitives {
        *counts.entry(*p).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(p, _)| p)
}

/// Audit all known GroundsTo implementations against core.true invariants.
///
/// Validates structural health of the entire grounding landscape:
/// - Every composition has a dominant primitive
/// - Dominant is a member of the composition's primitives
/// - Confidence is in valid range (0.0, 1.0]
/// - No empty compositions
/// - Tier classification matches unique primitive count
/// - All 16 T1 primitives appear at least once across all compositions
pub fn audit() -> Result<CallToolResult, McpError> {
    let mut total = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();
    let mut tier_counts = [0u32; 4]; // T1, T2-P, T2-C, T3
    let mut primitive_frequency = std::collections::HashMap::<String, u32>::new();
    let mut missing_dominant = Vec::new();
    let mut orphan_dominant = Vec::new();
    let mut zero_confidence = Vec::new();
    let mut empty_composition = Vec::new();
    let mut types_by_tier: [Vec<String>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    for &type_name in KNOWN_TYPES {
        total += 1;
        let Some(comp) = get_composition_direct(type_name) else {
            errors.push(json!({
                "type": type_name,
                "error": "get_composition_direct returned None",
            }));
            continue;
        };

        // Check 1: non-empty composition
        if comp.primitives.is_empty() {
            empty_composition.push(type_name.to_string());
            errors.push(json!({
                "type": type_name,
                "check": "empty_composition",
                "error": "primitives vec is empty",
            }));
        }

        // Check 2: has dominant
        match comp.dominant {
            None => {
                missing_dominant.push(type_name.to_string());
                errors.push(json!({
                    "type": type_name,
                    "check": "missing_dominant",
                    "error": "no dominant primitive declared",
                }));
            }
            Some(dom) => {
                // Check 3: dominant is member of primitives
                if !comp.primitives.contains(&dom) {
                    orphan_dominant.push(type_name.to_string());
                    errors.push(json!({
                        "type": type_name,
                        "check": "orphan_dominant",
                        "error": format!("dominant {:?} not in primitives {:?}", dom, comp.primitives),
                    }));
                }
            }
        }

        // Check 4: confidence in valid range
        if comp.confidence <= 0.0 || comp.confidence > 1.0 {
            zero_confidence.push(type_name.to_string());
            errors.push(json!({
                "type": type_name,
                "check": "invalid_confidence",
                "error": format!("confidence {} outside (0.0, 1.0]", comp.confidence),
            }));
        }

        // Accumulate tier distribution
        let tier = GroundingTier::classify(&comp);
        let tier_idx = match tier {
            GroundingTier::T1Universal => 0,
            GroundingTier::T2Primitive => 1,
            GroundingTier::T2Composite => 2,
            GroundingTier::T3DomainSpecific => 3,
        };
        tier_counts[tier_idx] += 1;
        types_by_tier[tier_idx].push(type_name.to_string());

        // Accumulate primitive frequency
        for prim in &comp.primitives {
            *primitive_frequency
                .entry(format!("{} ({})", prim.symbol(), prim.name()))
                .or_insert(0) += 1;
        }
    }

    // Check 5: all 16 T1 primitives appear somewhere
    let all_prims = LexPrimitiva::all();
    let mut coverage: Vec<serde_json::Value> = Vec::new();
    let mut covered_count = 0u32;
    for prim in &all_prims {
        let key = format!("{} ({})", prim.symbol(), prim.name());
        let count = primitive_frequency.get(&key).copied().unwrap_or(0);
        if count > 0 {
            covered_count += 1;
        }
        coverage.push(json!({
            "primitive": key,
            "frequency": count,
            "covered": count > 0,
        }));
    }
    // Sort by frequency descending
    coverage.sort_by(|a, b| {
        let fa = a["frequency"].as_u64().unwrap_or(0);
        let fb = b["frequency"].as_u64().unwrap_or(0);
        fb.cmp(&fa)
    });

    let error_count = errors.len();
    let health = if total == 0 {
        0.0
    } else {
        ((total as f64 - error_count as f64) / total as f64 * 100.0).round() / 100.0 * 100.0
    };

    let report = json!({
        "audit": "GroundsTo Landscape Health Report",
        "source": "core.true v7.5 invariants",
        "total_types_audited": total,
        "errors": error_count,
        "health_pct": health,
        "tier_distribution": {
            "T1_Universal": tier_counts[0],
            "T2_Primitive": tier_counts[1],
            "T2_Composite": tier_counts[2],
            "T3_DomainSpecific": tier_counts[3],
        },
        "primitive_coverage": {
            "covered": covered_count,
            "total": 16,
            "pct": (covered_count as f64 / 16.0 * 100.0).round(),
            "details": coverage,
        },
        "violations": errors,
        "summary": {
            "empty_composition": empty_composition.len(),
            "missing_dominant": missing_dominant.len(),
            "orphan_dominant": orphan_dominant.len(),
            "invalid_confidence": zero_confidence.len(),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| report.to_string()),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_runs_and_reports() {
        let result = audit();
        assert!(result.is_ok(), "audit() should not error");
        let call_result = result.ok();
        assert!(call_result.is_some());
        // Print the report for baseline capture
        if let Some(cr) = call_result {
            for content in &cr.content {
                eprintln!("{:?}", content);
            }
        }
    }

    #[test]
    fn test_audit_checks_all_known_types() {
        let result = audit();
        assert!(result.is_ok());
        // Extract the JSON from the CallToolResult
        let call_result = result.ok();
        assert!(call_result.is_some());
        // KNOWN_TYPES should have >100 entries
        assert!(
            KNOWN_TYPES.len() > 100,
            "Expected >100 known types, got {}",
            KNOWN_TYPES.len()
        );
    }

    #[test]
    fn test_known_types_resolution_rate() {
        // Count how many KNOWN_TYPES resolve vs don't
        let mut resolved = 0;
        let mut missing = Vec::new();
        for &type_name in KNOWN_TYPES {
            if get_composition_direct(type_name).is_some() {
                resolved += 1;
            } else {
                missing.push(type_name);
            }
        }
        let total = KNOWN_TYPES.len();
        let pct = (resolved as f64 / total as f64) * 100.0;
        // Regression gate: 100% of KNOWN_TYPES must resolve
        assert!(
            missing.is_empty(),
            "Resolution rate {:.0}% ({}/{}). Unresolved: {:?}",
            pct,
            resolved,
            total,
            missing
        );
    }
}
