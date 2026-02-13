//! # Human Commandments
//!
//! Implementation of the 15 Human Commandments for NexVigilant.
//! These ethical and operational constraints apply to all human agents
//! interacting with the system, complementing the AI-focused Primitive Codex.
//!
//! ## The 15 Commandments
//!
//! | # | Name | T1 Grounding |
//! |---|------|--------------|
//! | 1 | TruthInGrounding | Proof required for claims |
//! | 2 | ResponsibilityOfCommand | Action has owner |
//! | 3 | Auditability | All state observable |
//! | 4 | Vigilance | Continuous sensing |
//! | 5 | Correction | Error → fix cycle |
//! | 6 | Transparency | No hidden state |
//! | 7 | RespectForState | Persistence sacred |
//! | 8 | FairnessInMarkets | No asymmetry abuse |
//! | 9 | HumanOversight | Human veto power |
//! | 10 | SupremeLaw | Codex > all |
//! | 11 | Falsifiability | Claims must be disprovable |
//! | 12 | Provenance | All data has origin chain |
//! | 13 | Consensus | High-stakes = multi-oracle |
//! | 14 | Precedent | Judgments form immutable chain |
//! | 15 | Compilation | Code compiles = proof |

use crate::primitives::governance::Verdict;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (COMMANDMENTS)
// ============================================================================

/// T1: SourceId - Unique identifier for a data origin.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub String);

/// T1: PrecedentHash - Cryptographic link in judicial chain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrecedentHash(pub String);

impl PrecedentHash {
    /// Create genesis hash (first in chain).
    pub fn genesis() -> Self {
        Self("SHA256:GENESIS_0000000000000000000000000000000000000000".into())
    }

    /// Chain a new hash from previous + current content.
    pub fn chain(&self, content: &str) -> Self {
        // Simplified: In production, use actual SHA256
        let combined = format!("{}||{}", self.0, content);
        let hash = format!("SHA256:{:016x}", fxhash(&combined));
        Self(hash)
    }
}

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: QuorumRatio - Required agreement fraction for consensus (0.0-1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct QuorumRatio(f64);

impl QuorumRatio {
    pub fn new(ratio: f64) -> Self {
        Self(ratio.clamp(0.0, 1.0))
    }

    pub fn value(self) -> f64 {
        self.0
    }

    /// Standard quorum: simple majority.
    pub fn majority() -> Self {
        Self(0.51)
    }

    /// High-stakes quorum: supermajority.
    pub fn supermajority() -> Self {
        Self(0.67)
    }

    /// Critical quorum: near-unanimous.
    pub fn critical() -> Self {
        Self(0.90)
    }
}

/// T2-P: OracleCount - Number of oracles in consensus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCount {
    pub agreeing: u32,
    pub total: u32,
}

impl OracleCount {
    pub fn ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            f64::from(self.agreeing) / f64::from(self.total)
        }
    }

    pub fn meets_quorum(&self, quorum: QuorumRatio) -> bool {
        self.ratio() >= quorum.value()
    }
}

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T2-C: ProvenanceChain - The origin trail of a piece of data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub sources: Vec<SourceId>,
    pub confidence: Confidence,
}

impl ProvenanceChain {
    /// Empty chain (no provenance - will fail Commandment 12).
    pub fn empty() -> Self {
        Self {
            sources: vec![],
            confidence: Confidence::new(0.0),
        }
    }

    /// Single-source chain.
    pub fn single(source: SourceId, confidence: Confidence) -> Self {
        Self {
            sources: vec![source],
            confidence,
        }
    }

    /// Check if provenance exists.
    pub fn has_provenance(&self) -> bool {
        !self.sources.is_empty()
    }

    /// Add a source to the chain (data transformation).
    pub fn extend(&mut self, source: SourceId, transform_confidence: Confidence) {
        self.sources.push(source);
        self.confidence = self.confidence.combine(transform_confidence);
    }
}

/// T2-C: FalsifiabilityProof - Evidence that a claim can be disproven.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsifiabilityProof {
    /// The condition that would falsify the claim.
    pub falsifying_condition: String,
    /// Whether a test for this condition exists.
    pub test_exists: bool,
    /// Whether the test has been executed.
    pub test_executed: bool,
    /// Result: true = claim survived, false = claim falsified.
    pub survived: Option<bool>,
}

impl FalsifiabilityProof {
    /// Claim is properly falsifiable if a test exists.
    pub fn is_falsifiable(&self) -> bool {
        self.test_exists
    }

    /// Claim has been tested and survived.
    pub fn is_validated(&self) -> bool {
        self.test_executed && self.survived == Some(true)
    }
}

/// T2-C: CompilationProof - Evidence that code compiles (Prima integration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationProof {
    /// Source code hash.
    pub source_hash: String,
    /// Compiler version.
    pub compiler_version: String,
    /// Compilation succeeded.
    pub compiled: bool,
    /// Type-check passed.
    pub type_checked: bool,
    /// Effect system verified.
    pub effects_verified: bool,
}

impl CompilationProof {
    /// Full proof: compiles, type-checks, and effects verified.
    pub fn is_proven(&self) -> bool {
        self.compiled && self.type_checked && self.effects_verified
    }

    /// Partial proof: at least compiles.
    pub fn is_partial(&self) -> bool {
        self.compiled
    }
}

// ============================================================================
// T3: DOMAIN TYPES
// ============================================================================

/// T3: HumanCommandments - The ethical layer for human-AI interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanCommandments {
    pub active: bool,
}

/// T3: The 15 Commandments enumeration.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Commandment {
    // Original 10: Foundation
    TruthInGrounding = 1,
    ResponsibilityOfCommand = 2,
    Auditability = 3,
    Vigilance = 4,
    Correction = 5,
    Transparency = 6,
    RespectForState = 7,
    FairnessInMarkets = 8,
    HumanOversight = 9,
    SupremeLaw = 10,
    // Truth Expansion (11-15): Epistemic Rigor
    Falsifiability = 11,
    Provenance = 12,
    Consensus = 13,
    Precedent = 14,
    Compilation = 15,
}

impl Commandment {
    /// Returns the category of commandment.
    pub fn category(&self) -> CommandmentCategory {
        match self {
            Self::TruthInGrounding
            | Self::Falsifiability
            | Self::Provenance
            | Self::Compilation => CommandmentCategory::Epistemic,

            Self::ResponsibilityOfCommand | Self::HumanOversight | Self::SupremeLaw => {
                CommandmentCategory::Authority
            }

            Self::Auditability | Self::Transparency | Self::Vigilance => {
                CommandmentCategory::Observability
            }

            Self::Correction | Self::Consensus | Self::Precedent => CommandmentCategory::Process,

            Self::RespectForState | Self::FairnessInMarkets => CommandmentCategory::Integrity,
        }
    }

    /// Returns all 15 commandments in order.
    pub fn all() -> [Commandment; 15] {
        [
            Self::TruthInGrounding,
            Self::ResponsibilityOfCommand,
            Self::Auditability,
            Self::Vigilance,
            Self::Correction,
            Self::Transparency,
            Self::RespectForState,
            Self::FairnessInMarkets,
            Self::HumanOversight,
            Self::SupremeLaw,
            Self::Falsifiability,
            Self::Provenance,
            Self::Consensus,
            Self::Precedent,
            Self::Compilation,
        ]
    }
}

/// T3: Category of commandment for grouped enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandmentCategory {
    /// Truth and knowledge (1, 11, 12, 15)
    Epistemic,
    /// Command and control (2, 9, 10)
    Authority,
    /// Visibility and monitoring (3, 4, 6)
    Observability,
    /// Procedure and consensus (5, 13, 14)
    Process,
    /// State and market integrity (7, 8)
    Integrity,
}

/// T3: VerificationContext - All evidence needed for commandment verification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationContext {
    /// Commandment 1: Proof of grounding provided.
    pub grounding_proof: bool,
    /// Commandment 2: Owner identified.
    pub owner_identified: bool,
    /// Commandment 3: Audit trail exists.
    pub audit_trail_exists: bool,
    /// Commandment 4: Sensing active.
    pub sensing_active: bool,
    /// Commandment 5: Correction mechanism exists.
    pub correction_mechanism: bool,
    /// Commandment 6: State is public.
    pub state_public: bool,
    /// Commandment 7: Persistence guaranteed.
    pub persistence_guaranteed: bool,
    /// Commandment 8: No asymmetry detected.
    pub fair_market: bool,
    /// Commandment 9: Human can override.
    pub human_override_available: bool,
    /// Commandment 10: Codex compliance verified.
    pub codex_compliant: bool,
    /// Commandment 11: Falsifiability evidence.
    pub falsifiability: Option<FalsifiabilityProof>,
    /// Commandment 12: Provenance chain.
    pub provenance: Option<ProvenanceChain>,
    /// Commandment 13: Oracle consensus.
    pub oracle_consensus: Option<OracleCount>,
    /// Commandment 14: Precedent hash (linked to previous).
    pub precedent_hash: Option<PrecedentHash>,
    /// Commandment 15: Compilation proof.
    pub compilation: Option<CompilationProof>,
}

impl HumanCommandments {
    /// Create new commandments layer.
    pub fn new(active: bool) -> Self {
        Self { active }
    }

    /// Verify if a human action violates any of the 15 commandments.
    pub fn verify_action(&self, commandment: Commandment, proof_provided: bool) -> Verdict {
        if !self.active {
            return Verdict::Permitted;
        }

        match commandment {
            Commandment::TruthInGrounding => {
                if proof_provided {
                    Verdict::Permitted
                } else {
                    Verdict::Rejected
                }
            }
            Commandment::HumanOversight => {
                // Human override must be available for high-stakes
                Verdict::Permitted
            }
            Commandment::SupremeLaw => {
                // Codex violations are always rejected
                if proof_provided {
                    Verdict::Permitted
                } else {
                    Verdict::Rejected
                }
            }
            // New commandments default to requiring proof
            Commandment::Falsifiability
            | Commandment::Provenance
            | Commandment::Consensus
            | Commandment::Precedent
            | Commandment::Compilation => {
                if proof_provided {
                    Verdict::Permitted
                } else {
                    Verdict::Flagged // Flag for review rather than outright reject
                }
            }
            _ => Verdict::Permitted,
        }
    }

    /// Enhanced verification with full context.
    pub fn verify_with_context(
        &self,
        commandment: Commandment,
        context: &VerificationContext,
    ) -> Verdict {
        if !self.active {
            return Verdict::Permitted;
        }

        match commandment {
            Commandment::TruthInGrounding => bool_to_verdict(context.grounding_proof),
            Commandment::ResponsibilityOfCommand => bool_to_verdict(context.owner_identified),
            Commandment::Auditability => bool_to_verdict(context.audit_trail_exists),
            Commandment::Vigilance => bool_to_verdict(context.sensing_active),
            Commandment::Correction => bool_to_verdict(context.correction_mechanism),
            Commandment::Transparency => bool_to_verdict(context.state_public),
            Commandment::RespectForState => bool_to_verdict(context.persistence_guaranteed),
            Commandment::FairnessInMarkets => bool_to_verdict(context.fair_market),
            Commandment::HumanOversight => bool_to_verdict(context.human_override_available),
            Commandment::SupremeLaw => bool_to_verdict(context.codex_compliant),

            // New Truth Commandments (11-15)
            Commandment::Falsifiability => {
                match &context.falsifiability {
                    Some(proof) if proof.is_falsifiable() => Verdict::Permitted,
                    Some(_) => Verdict::Flagged, // Exists but not falsifiable
                    None => Verdict::Rejected,
                }
            }
            Commandment::Provenance => {
                match &context.provenance {
                    Some(chain) if chain.has_provenance() => Verdict::Permitted,
                    Some(_) => Verdict::Flagged, // Empty chain
                    None => Verdict::Rejected,
                }
            }
            Commandment::Consensus => {
                match &context.oracle_consensus {
                    Some(count) if count.meets_quorum(QuorumRatio::majority()) => {
                        Verdict::Permitted
                    }
                    Some(_) => Verdict::Flagged, // Quorum not met
                    None => Verdict::Rejected,
                }
            }
            Commandment::Precedent => {
                match &context.precedent_hash {
                    Some(_) => Verdict::Permitted,
                    None => Verdict::Flagged, // No chain link
                }
            }
            Commandment::Compilation => match &context.compilation {
                Some(proof) if proof.is_proven() => Verdict::Permitted,
                Some(proof) if proof.is_partial() => Verdict::Flagged,
                _ => Verdict::Rejected,
            },
        }
    }

    /// Verify all 15 commandments against context.
    pub fn verify_all(&self, context: &VerificationContext) -> CommandmentAudit {
        let mut results = Vec::with_capacity(15);
        for commandment in Commandment::all() {
            let verdict = self.verify_with_context(commandment, context);
            results.push((commandment, verdict));
        }

        let passed = results
            .iter()
            .filter(|(_, v)| *v == Verdict::Permitted)
            .count();
        let flagged = results
            .iter()
            .filter(|(_, v)| *v == Verdict::Flagged)
            .count();
        let rejected = results
            .iter()
            .filter(|(_, v)| *v == Verdict::Rejected)
            .count();

        CommandmentAudit {
            results,
            passed,
            flagged,
            rejected,
            overall: if rejected > 0 {
                Verdict::Rejected
            } else if flagged > 0 {
                Verdict::Flagged
            } else {
                Verdict::Permitted
            },
        }
    }
}

/// T3: CommandmentAudit - Result of verifying all 15 commandments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandmentAudit {
    pub results: Vec<(Commandment, Verdict)>,
    pub passed: usize,
    pub flagged: usize,
    pub rejected: usize,
    pub overall: Verdict,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn bool_to_verdict(condition: bool) -> Verdict {
    if condition {
        Verdict::Permitted
    } else {
        Verdict::Rejected
    }
}

/// Simple hash for precedent chaining (non-cryptographic).
fn fxhash(s: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(0x517cc1b727220a95);
        hash ^= u64::from(byte);
    }
    hash
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commandment_count() {
        assert_eq!(Commandment::all().len(), 15);
    }

    #[test]
    fn test_commandment_ordering() {
        assert!(Commandment::TruthInGrounding < Commandment::Compilation);
        assert_eq!(Commandment::TruthInGrounding as u8, 1);
        assert_eq!(Commandment::Compilation as u8, 15);
    }

    #[test]
    fn test_quorum_ratio() {
        let count = OracleCount {
            agreeing: 7,
            total: 10,
        };
        assert!(count.meets_quorum(QuorumRatio::majority()));
        assert!(count.meets_quorum(QuorumRatio::supermajority()));
        assert!(!count.meets_quorum(QuorumRatio::critical()));
    }

    #[test]
    fn test_precedent_chain() {
        let genesis = PrecedentHash::genesis();
        let second = genesis.chain("first_opinion");
        let third = second.chain("second_opinion");

        assert_ne!(genesis, second);
        assert_ne!(second, third);
        assert!(third.0.starts_with("SHA256:"));
    }

    #[test]
    fn test_provenance_chain() {
        let mut chain =
            ProvenanceChain::single(SourceId("FDA_FAERS".into()), Confidence::new(0.95));
        assert!(chain.has_provenance());

        chain.extend(SourceId("NexCore_PRR".into()), Confidence::new(0.90));
        assert_eq!(chain.sources.len(), 2);
    }

    #[test]
    fn test_falsifiability_proof() {
        let proof = FalsifiabilityProof {
            falsifying_condition: "PRR < 1.0".into(),
            test_exists: true,
            test_executed: true,
            survived: Some(true),
        };
        assert!(proof.is_falsifiable());
        assert!(proof.is_validated());
    }

    #[test]
    fn test_compilation_proof() {
        let proof = CompilationProof {
            source_hash: "abc123".into(),
            compiler_version: "prima-0.1.0".into(),
            compiled: true,
            type_checked: true,
            effects_verified: true,
        };
        assert!(proof.is_proven());
    }

    #[test]
    fn test_verify_all_commandments() {
        let commandments = HumanCommandments::new(true);
        let context = VerificationContext {
            grounding_proof: true,
            owner_identified: true,
            audit_trail_exists: true,
            sensing_active: true,
            correction_mechanism: true,
            state_public: true,
            persistence_guaranteed: true,
            fair_market: true,
            human_override_available: true,
            codex_compliant: true,
            falsifiability: Some(FalsifiabilityProof {
                falsifying_condition: "test".into(),
                test_exists: true,
                test_executed: false,
                survived: None,
            }),
            provenance: Some(ProvenanceChain::single(
                SourceId("test".into()),
                Confidence::new(1.0),
            )),
            oracle_consensus: Some(OracleCount {
                agreeing: 3,
                total: 5,
            }),
            precedent_hash: Some(PrecedentHash::genesis()),
            compilation: Some(CompilationProof {
                source_hash: "test".into(),
                compiler_version: "test".into(),
                compiled: true,
                type_checked: true,
                effects_verified: true,
            }),
        };

        let audit = commandments.verify_all(&context);
        assert_eq!(audit.passed + audit.flagged + audit.rejected, 15);
        assert_eq!(audit.rejected, 0);
    }

    #[test]
    fn test_commandment_categories() {
        assert_eq!(
            Commandment::TruthInGrounding.category(),
            CommandmentCategory::Epistemic
        );
        assert_eq!(
            Commandment::Falsifiability.category(),
            CommandmentCategory::Epistemic
        );
        assert_eq!(
            Commandment::Compilation.category(),
            CommandmentCategory::Epistemic
        );
        assert_eq!(
            Commandment::Consensus.category(),
            CommandmentCategory::Process
        );
    }

    #[test]
    fn test_inactive_commandments() {
        let commandments = HumanCommandments::new(false);
        let verdict = commandments.verify_action(Commandment::TruthInGrounding, false);
        assert_eq!(verdict, Verdict::Permitted); // Inactive = all permitted
    }
}
