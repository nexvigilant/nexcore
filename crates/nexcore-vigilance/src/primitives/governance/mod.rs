//! # Governance Primitives
//!
//! Computational patterns for simulating the United States Government structure
//! within the NexVigilant framework. These primitives ground high-level political
//! and administrative concepts into T1/T2 universals.
//!
//! ## Tier Classification
//!
//! | Concept | Tier | Grounding |
//! |---------|------|-----------|
//! | Action | T1 | Axiom: Exchange |
//! | Rule | T1 | Axiom: Cause |
//! | Verdict | T1 | Axiom: Detect |
//! | VoteWeight | T2-P | T1: Quantity (0-100) |
//! | Confidence | T2-P | T1: Ratio (0.0-1.0) |
//! | Term | T2-P | T1: Duration |
//! | Treasury | T2-C | T1: Resource Set |

pub mod administrative_act;

pub mod agents;

pub mod bayesian_trial;

pub mod business_simulation;

pub mod cabinet;

pub mod communication;

pub mod congress;

pub mod due_process;

pub mod election;

pub mod executive;

pub mod executive_unity;

pub mod factional_stability;

pub mod federalist;

pub mod grounding_treaty;

pub mod hierarchy;

pub mod human_commandments;

pub mod locality;

pub mod judicial;

pub mod judicial_review;

pub mod market_integration;

pub mod national_strategy;

pub mod oracle_protocol;

pub mod partnership_charter;

pub mod promotion;

pub mod reserved_powers;

pub mod resource_limits;

pub mod security;

pub mod sovereignty;

pub mod tooling;

pub mod treasury_act;

pub mod unenumerated;

pub mod union;

pub mod validation_speed;

pub use administrative_act::{AdministrativeRule, AgencyLog, Compliance, Procedure};

pub use agents::{AgentRole, GovernanceAgent};

pub use business_simulation::{
    AnySimulation, CapabilityBuild, Foundation, GrowthPhase, GrowthPhaseSnapshot, GrowthPhaseState,
    MarketEntry, MarketLeadership, Scaling, SimulationParameters, SimulationSnapshot,
};

pub use cabinet::{
    AttorneyGeneral, Cabinet, SecretaryOfAgriculture, SecretaryOfCommerce, SecretaryOfDefense,
    SecretaryOfEducation, SecretaryOfEnergy, SecretaryOfHealthAndHumanServices,
    SecretaryOfHomelandSecurity, SecretaryOfHousingAndUrbanDevelopment, SecretaryOfLabor,
    SecretaryOfState, SecretaryOfTheInterior, SecretaryOfTheTreasury, SecretaryOfTransportation,
    SecretaryOfVeteransAffairs,
};

pub use congress::{Congress, EscalationLevel, HouseOfT1, SenateOfT2, T1Representative, T2Senator};

pub use election::{
    CertificateOfAscertainment, ContextUtilization, ElectionCommission, ElectoralCollege,
    HandoffArtifact,
};

pub use executive::{Agent, Guardrail, Orchestrator, RiskMinimizationLevel, RiskMinimizer};

pub use executive_unity::{Dispatch, Energy, ExecutivePower, Responsibility};

pub use factional_stability::{
    Adversity, Faction as InterestFaction, FactionDensity, Interest, StabilityAudit,
};

pub use federalist::FederalistPipeline;

pub use grounding_treaty::{AlignmentScore, ExternalRelationsOffice, ForeignType, TranslationBond};

pub use hierarchy::{Division, DivisionAssignment, ExecutiveHierarchy, OrgRole};

pub use human_commandments::{
    Commandment, CommandmentAudit, CommandmentCategory, CompilationProof, FalsifiabilityProof,
    HumanCommandments, OracleCount, PrecedentHash, ProvenanceChain, QuorumRatio, SourceId,
    VerificationContext,
};

pub use judicial::SupremeCompiler;

pub use judicial_review::{JudicialOpinion, JudicialReviewEngine, Nullification, Precedent};

pub use market_integration::{MarketIntegration, MarketSimulation};

pub use national_strategy::{DepartmentalMandate, NationalStrategy, StrategicFocus};

pub use oracle_protocol::{
    OracleIntegrationLayer, OracleQuery, OracleReputation, OracleResponse, RequestID,
};

pub use partnership_charter::{
    DissolutionProtocol, DualSignatureTreasury, PartnershipBoard, Share,
};

pub use promotion::{AgentRank, PromotionCriteria};

// === Bill of Rights Re-exports (Amendments I-X) ===

pub use bayesian_trial::BayesianTrial;

pub use communication::{GroundingRight, LoggingFreedom, RedressOfInconsistency, RegistryAccess};

pub use due_process::{DueProcess, DueProcessViolation, QuotaCompensation};

pub use locality::{ExternalLogic, ModulePrivacy};

pub use reserved_powers::{DomainReservation, PowerHolder, ReservedPower};

pub use resource_limits::{ResourceGuard, ResourceViolation};

pub use security::{Clearance, SecurityAlert, StateGuardian};

pub use sovereignty::{Faction as DomainFaction, SovereignDomain};

pub use tooling::{AgentPool, ToolRight};

pub use treasury_act::{Asymmetry, Liquidity, MarketState, Odds, Position, Side, Token};

pub use unenumerated::{AxiomRetention, UnenumeratedRight};

pub use union::Union;

pub use validation_speed::SpeedyValidation;

use nexcore_primitives::measurement::{Confidence, Measured};

use serde::{Deserialize, Serialize};

// ============================================================================

// T1: UNIVERSAL PRIMITIVES (AXIOMS)

// ============================================================================

/// T1: Action - The fundamental executive act.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]

pub struct Action;

/// T1: Rule - The fundamental legislative constraint.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]

pub struct Rule;

/// T1: Verdict - The fundamental judicial determination.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]

pub enum Verdict {
    Rejected = 0,

    Flagged = 1,

    Permitted = 2,
}

// ============================================================================

// T2-P: GOVERNANCE QUANTITIES (NEWTYPES)

// ============================================================================

/// T2-P: VoteWeight - Quantified influence (0-100).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]

pub struct VoteWeight(u8);

impl VoteWeight {
    pub const fn new(value: u8) -> Self {
        Self(if value > 100 { 100 } else { value })
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// T2-P: Term - Duration of office or validity in cycles.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]

pub struct Term(u32);

impl Term {
    pub const fn new(cycles: u32) -> Self {
        Self(cycles)
    }

    pub const fn cycles(self) -> u32 {
        self.0
    }
}

// ============================================================================

// T2-C: GOVERNANCE COMPOSITES

// ============================================================================

/// T2-C: Resolution - A proposed change to the system.

/// Composes Rule and Uncertainty via Measured wrapper.

pub type Resolution = Measured<Rule>;

/// Extension trait for Resolution (Measured<Rule>)

pub trait ResolutionExt {
    fn support(&self) -> VoteWeight;

    fn with_support(self, support: VoteWeight) -> ResolutionWithSupport;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct ResolutionWithSupport {
    pub resolution: Resolution,

    pub support: VoteWeight,
}

/// T2-C: Treasury - Resource quota management.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub struct Treasury {
    pub compute_quota: u64,

    pub memory_quota: u64,
}

impl Treasury {
    pub const fn can_afford(&self, cost: &Treasury) -> bool {
        self.compute_quota >= cost.compute_quota && self.memory_quota >= cost.memory_quota
    }

    pub fn spend(&mut self, cost: &Treasury) -> Result<(), &'static str> {
        if self.can_afford(cost) {
            self.compute_quota -= cost.compute_quota;

            self.memory_quota -= cost.memory_quota;

            Ok(())
        } else {
            Err("Insufficient funds in treasury")
        }
    }
}
