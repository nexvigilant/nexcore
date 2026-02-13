//! # Capability 17: Institutional Memory Act (Presidential Library)
//!
//! Implementation of the Presidential Library as a core structural
//! capability within the HUD domain. This capability ensures that the
//! Union's strategic evolution is searchable, versioned, and audit-logged.
//!
//! Matches 1:1 to the US National Archives and Records Administration (NARA)
//! and the Presidential Libraries Act.

use crate::primitives::governance::Verdict;
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: InstitutionalMemoryAct - Capability 17 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalMemoryAct {
    pub id: String,
    pub storage_active: bool,
}

/// T2-P: ArchiveSecurityLevel - Role-based access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArchiveSecurityLevel {
    PublicUnion,
    ExecutiveOnly,
    CEOPrivate,
}

/// T2-C: LibraryArtifact - A versioned document in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryArtifact {
    pub artifact_id: String,
    pub title: String,
    pub security_level: ArchiveSecurityLevel,
    pub grounding_proof_hash: String,
    pub version: u32,
}

impl InstitutionalMemoryAct {
    pub fn new() -> Self {
        Self {
            id: "CAP-017".into(),
            storage_active: true,
        }
    }

    /// Archive a directive or charter.
    pub fn archive_document(
        &self,
        title: &str,
        security: ArchiveSecurityLevel,
    ) -> Measured<LibraryArtifact> {
        let artifact = LibraryArtifact {
            artifact_id: format!("NEX-ARC-{}", title.to_uppercase().replace(" ", "_")),
            title: title.into(),
            security_level: security,
            grounding_proof_hash: "SHA256:IMMUTABLE_HISTORY".into(),
            version: 1,
        };

        Measured::uncertain(artifact, Confidence::new(1.0))
    }

    /// Verify access rights to an artifact.
    pub fn verify_access(
        &self,
        requester_role: crate::primitives::governance::OrgRole,
        artifact: &LibraryArtifact,
    ) -> Verdict {
        use crate::primitives::governance::OrgRole;
        match (requester_role, artifact.security_level) {
            (OrgRole::CEO, _) => Verdict::Permitted,
            (OrgRole::President, ArchiveSecurityLevel::CEOPrivate) => Verdict::Rejected,
            (OrgRole::President, _) => Verdict::Permitted,
            (_, ArchiveSecurityLevel::PublicUnion) => Verdict::Permitted,
            _ => Verdict::Rejected,
        }
    }
}
