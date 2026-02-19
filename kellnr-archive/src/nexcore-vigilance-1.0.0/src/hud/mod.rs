//! # HUD Domain: Housing and Urban Development
//!
//! The HUD domain is responsible for the structural integrity of the nexcore
//! and the codification of the 37 Core Capabilities of NexVigilant.
//!
//! Matches 1:1 to the US Department of Housing and Urban Development's role
//! in building and maintaining national infrastructure, here applied to
//! our computational foundation.

pub mod capabilities;
pub mod judicial;
pub mod labor;
pub mod legislative;

use self::judicial::JudicialBranch;
use self::labor::LaborDomain;
use self::legislative::LegislativeBranch;
use serde::{Deserialize, Serialize};

/// T3: HUD - The architectural authority of the Union.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hud {
    /// List of currently active Capability IDs within the Union.
    pub active_capabilities: Vec<String>,
    /// The Labor domain managing workforce and management systems.
    pub labor: LaborDomain,
    /// The Legislative branch for processing resolutions and bills.
    pub legislative: Option<LegislativeBranch>,
    /// The Judicial branch for adjudication and constitutional review.
    pub judicial: Option<JudicialBranch>,
}

impl Hud {
    /// Creates a new instance of the HUD authority.
    pub fn new() -> Self {
        Self {
            active_capabilities: vec![],
            labor: LaborDomain::new(),
            legislative: None,
            judicial: None,
        }
    }
}
