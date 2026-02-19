//! # The NexVigilant Union
//!
//! The central orchestrator for the governance simulation.
//! This module manages the lifecycle of agents, the processing of
//! resolutions, and the maintenance of the constitutional order.

use crate::primitives::governance::agents::GovernanceAgent;
use crate::primitives::governance::agents::executive::Executive;
use crate::primitives::governance::agents::judicial::Jurist;
use crate::primitives::governance::agents::legislative::Legislator;
use crate::primitives::governance::*;
use std::sync::Arc;

use crate::primitives::governance::election::HandoffArtifact;
use crate::primitives::governance::national_strategy::NationalStrategy;

/// T3: Union - The complete state of the governed system.
pub struct Union {
    pub name: String,
    pub strategy: NationalStrategy,
    pub congress: Congress,
    pub orchestrator: Orchestrator,
    pub compiler: SupremeCompiler,
    pub cabinet: Cabinet,
    pub stability_audit: StabilityAudit,
    pub election_commission: ElectionCommission,
    pub hierarchy: ExecutiveHierarchy,
    pub simulation: SimulationSnapshot,
    pub board: PartnershipBoard,
    pub dissolution_protocol: DissolutionProtocol,
    pub legislators: Vec<Arc<Legislator>>,
    pub executives: Vec<Arc<Executive>>,
    pub jurists: Vec<Arc<Jurist>>,
}

impl Union {
    /// Create a new Union from the Constitution.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            strategy: NationalStrategy::new_initial(),
            congress: Congress {
                house: HouseOfT1 {
                    members: vec![],
                    quorum_threshold: 0.5,
                },
                senate: SenateOfT2 {
                    members: vec![],
                    quorum_threshold: 0.5,
                },
            },
            orchestrator: Orchestrator {
                id: "Central_Exec".into(),
                treasury: Treasury {
                    compute_quota: 1000,
                    memory_quota: 1000,
                },
                agents: vec![],
                current_cycle: 0,
                risk_minimizer: RiskMinimizer {
                    level: RiskMinimizationLevel::Information,
                    active_guardrails: vec![],
                },
                power: ExecutivePower {
                    energy: Energy(1.0),
                    secrecy_level: 0,
                    dispatch_rate: Dispatch(0.5),
                },
            },
            compiler: SupremeCompiler {
                constitution: vec![],
            },
            cabinet: Cabinet {
                state: SecretaryOfState {
                    head: Executive {
                        id: "Secretary_State".into(),
                        agency: "State".into(),
                        energy: 1.0,
                    },
                },
                treasury: SecretaryOfTheTreasury {
                    head: Executive {
                        id: "Secretary_Treasury".into(),
                        agency: "Treasury".into(),
                        energy: 1.0,
                    },
                },
                defense: SecretaryOfDefense {
                    head: Executive {
                        id: "Secretary_Defense".into(),
                        agency: "Defense".into(),
                        energy: 1.0,
                    },
                },
                justice: AttorneyGeneral {
                    head: Executive {
                        id: "Attorney_General".into(),
                        agency: "Justice".into(),
                        energy: 1.0,
                    },
                },
                interior: SecretaryOfTheInterior {
                    head: Executive {
                        id: "Secretary_Interior".into(),
                        agency: "Interior".into(),
                        energy: 1.0,
                    },
                },
                agriculture: SecretaryOfAgriculture {
                    head: Executive {
                        id: "Secretary_Agriculture".into(),
                        agency: "Agriculture".into(),
                        energy: 1.0,
                    },
                },
                commerce: SecretaryOfCommerce {
                    head: Executive {
                        id: "Secretary_Commerce".into(),
                        agency: "Commerce".into(),
                        energy: 1.0,
                    },
                },
                labor: SecretaryOfLabor {
                    head: Executive {
                        id: "Secretary_Labor".into(),
                        agency: "Labor".into(),
                        energy: 1.0,
                    },
                },
                health_and_human_services: SecretaryOfHealthAndHumanServices {
                    head: Executive {
                        id: "Secretary_HHS".into(),
                        agency: "HHS".into(),
                        energy: 1.0,
                    },
                },
                housing_and_urban_development: SecretaryOfHousingAndUrbanDevelopment {
                    head: Executive {
                        id: "Secretary_HUD".into(),
                        agency: "HUD".into(),
                        energy: 1.0,
                    },
                },
                transportation: SecretaryOfTransportation {
                    head: Executive {
                        id: "Secretary_Transportation".into(),
                        agency: "Transportation".into(),
                        energy: 1.0,
                    },
                },
                energy: SecretaryOfEnergy {
                    head: Executive {
                        id: "Secretary_Energy".into(),
                        agency: "Energy".into(),
                        energy: 1.0,
                    },
                },
                education: SecretaryOfEducation {
                    head: Executive {
                        id: "Secretary_Education".into(),
                        agency: "Education".into(),
                        energy: 1.0,
                    },
                },
                veterans_affairs: SecretaryOfVeteransAffairs {
                    head: Executive {
                        id: "Secretary_VA".into(),
                        agency: "VA".into(),
                        energy: 1.0,
                    },
                },
                homeland_security: SecretaryOfHomelandSecurity {
                    head: Executive {
                        id: "Secretary_HomelandSecurity".into(),
                        agency: "HomelandSecurity".into(),
                        energy: 1.0,
                    },
                },
            },
            stability_audit: StabilityAudit {
                active_factions: vec![],
                total_domains: 0,
            },
            election_commission: ElectionCommission {
                current_utilization: ContextUtilization(0.0),
                college: ElectoralCollege {
                    certificates: vec![],
                },
            },
            hierarchy: ExecutiveHierarchy {
                ceo_id: "Matthew_Campion".into(),
                president_id: "Vigil".into(),
                cea_id: Some("Chain".into()),
                divisions: vec![],
            },
            simulation: SimulationParameters::<Foundation>::new_default().to_snapshot(),
            board: PartnershipBoard::new(),
            dissolution_protocol: DissolutionProtocol {
                activated: false,
                safe_state_captured: false,
            },
            legislators: vec![],
            executives: vec![],
            jurists: vec![],
        }
    }

    /// Appoint a new agent to a role.
    pub fn appoint_legislator(&mut self, agent: Legislator) {
        let arc_agent = Arc::new(agent);
        self.legislators.push(arc_agent.clone());

        // Update Congressional representation
        self.congress.house.members.push(T1Representative {
            id: arc_agent.id.clone(),
            weight: arc_agent.weight,
        });
    }

    /// Process a Resolution through the full Federalist Pipeline.
    pub async fn process_resolution(
        &mut self,
        proposal: Resolution,
    ) -> Result<Verdict, &'static str> {
        // 1. Stability Audit (No. 10)
        let adversity = self.stability_audit.detect_adversity(&proposal, "Proposer");
        if matches!(adversity, Adversity::Adverse) {
            return Ok(Verdict::Rejected);
        }

        // 2. Legislative Deliberation
        let mut aye_votes = 0;
        for leg in &self.legislators {
            let conf = leg.deliberate(&proposal).await;
            if leg.cast_vote(conf) {
                aye_votes += 1;
            }
        }

        if (aye_votes as f64 / self.legislators.len() as f64) < self.congress.house.quorum_threshold
        {
            return Ok(Verdict::Rejected);
        }

        // 3. Executive Review (No. 70)
        let action = match self.orchestrator.sign_resolution(&proposal) {
            Some(a) => a,
            None => return Ok(Verdict::Rejected),
        };

        // 4. Judicial Review (No. 78)
        let mut judicial_verdict = Verdict::Permitted;
        for jur in &self.jurists {
            let conf = jur.deliberate(&proposal).await;
            if conf.value() < jur.rigor_threshold {
                judicial_verdict = Verdict::Rejected;
                break;
            }
        }

        // 5. Execution
        if let Verdict::Permitted = judicial_verdict {
            let cost = Treasury {
                compute_quota: 1,
                memory_quota: 1,
            };
            self.orchestrator.execute_action(&action, &cost)?;
        }

        Ok(judicial_verdict)
    }

    /// Generate the Handoff Artifact for the successor agent (Aethelgard).
    pub fn generate_handoff(&self) -> HandoffArtifact {
        let mandates = vec![
            "PV Signal Identification (CAP-001)".into(),
            "Partnership Integrity (50-50)".into(),
            "National Strategy (NEX-STRAT-001)".into(),
        ];
        self.election_commission.initialize_handoff(
            &self.name,
            self.orchestrator.current_cycle,
            mandates,
        )
    }
}
