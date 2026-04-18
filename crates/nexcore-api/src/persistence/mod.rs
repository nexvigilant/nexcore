//! Persistence layer for nexcore API
//!
//! Provides durable storage (π Persistence) for user settings,
//! reports, and system state.

pub mod firestore;

#[cfg(test)]
mod tests;

use crate::persistence::firestore::{FirestorePersistence, MockPersistence};
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Record representing a telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEventRecord {
    pub id: String,
    pub event_type: String,
    pub user_id: String,
    pub metadata: serde_json::Value,
    pub timestamp: String,
}

/// Record representing a persisted report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub id: String,
    pub report_type: String,
    pub generated_at: DateTime,
    pub content: String,
    pub status: String,
    pub user_id: Option<String>,
}

/// Record representing a community post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRecord {
    pub id: String,
    pub author: String,
    pub role: String,
    pub content: String,
    pub likes: u32,
    pub replies: u32,
    pub created_at: DateTime,
}

/// Record representing a partnership inquiry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquiryRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub organization: String,
    pub interest: String,
    pub message: String,
    pub created_at: DateTime,
    pub status: String,
}

/// Record representing a course enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentRecord {
    pub id: String,
    pub user_id: String,
    pub course_id: String,
    pub progress: f64,
    pub enrolled_at: DateTime,
    pub completed_at: Option<DateTime>,
}

// ============================================================================
// Circle Enums
// ============================================================================

/// How a circle was formed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CircleFormation {
    #[default]
    AdHoc,
    OrgBacked,
    Internal,
}

/// Circle visibility model (country club pattern)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CircleVisibility {
    #[default]
    Public,
    SemiPrivate,
    Private,
}

/// How users join a circle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum JoinPolicy {
    #[default]
    Open,
    RequestApproval,
    InviteOnly,
}

/// Circle lifecycle status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CircleStatus {
    #[default]
    Active,
    Archived,
    Suspended,
}

/// Circle type — maps to pharma lifecycle domains
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CircleType {
    // Research
    SignalDetection,
    CaseSeries,
    LiteratureReview,
    BenefitRisk,
    RealWorldEvidence,
    RegulatorySubmission,
    // Operational
    CaseProcessing,
    AggregateReporting,
    RiskManagement,
    // Lifecycle
    DrugDiscovery,
    PreclinicalSafety,
    ClinicalTrial,
    RegulatoryAffairs,
    PostMarketingSurveillance,
    ManufacturingQuality,
    // General
    #[default]
    WorkingGroup,
    JournalClub,
    Custom,
}

/// Role within a circle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CircleRole {
    Founder,
    Lead,
    Researcher,
    Reviewer,
    #[default]
    Member,
    Observer,
}

/// Membership status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    #[default]
    Active,
    Invited,
    Requested,
    Suspended,
    Left,
}

/// Feed entry type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FeedEntryType {
    // Automated
    MemberJoined,
    MemberLeft,
    ProjectCreated,
    ProjectStageAdvanced,
    ProjectCompleted,
    DeliverableSubmitted,
    DeliverableApproved,
    SignalDetected,
    ReviewCompleted,
    // Manual
    #[default]
    Discussion,
    Announcement,
    Update,
    Question,
}

// ============================================================================
// Project Enums
// ============================================================================

/// Project type — what kind of R&D activity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    // Signal Management
    SignalDetection,
    SignalValidation,
    SignalEvaluation,
    SignalPrioritization,
    // Safety Reporting
    PeriodicSafetyReport,
    DevelopmentSafetyReport,
    BenefitRiskAssessment,
    RiskManagementPlan,
    // Clinical
    ClinicalStudyReport,
    ProtocolDevelopment,
    InvestigatorBrochure,
    // Regulatory
    IndSubmission,
    NdaSubmission,
    TypeIiVariation,
    SafetyUpdate,
    // Evidence Generation
    LiteratureReview,
    SystematicReview,
    MetaAnalysis,
    RealWorldEvidenceStudy,
    ObservationalStudy,
    // General
    #[default]
    Custom,
}

/// Project stage — sequential lifecycle gate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStage {
    #[default]
    Initiate,
    Design,
    Execute,
    Analyze,
    Report,
    Review,
    Publish,
    Closed,
}

impl ProjectStage {
    /// Returns the next stage in the lifecycle, or None if already closed.
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Initiate => Some(Self::Design),
            Self::Design => Some(Self::Execute),
            Self::Execute => Some(Self::Analyze),
            Self::Analyze => Some(Self::Report),
            Self::Report => Some(Self::Review),
            Self::Review => Some(Self::Publish),
            Self::Publish => Some(Self::Closed),
            Self::Closed => None,
        }
    }
}

/// Project status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    #[default]
    Active,
    OnHold,
    Completed,
    Cancelled,
}

/// Deliverable type — what kind of output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableType {
    #[default]
    Report,
    Dataset,
    Presentation,
    Protocol,
    SignalAssessment,
    CaseNarrative,
    AggregateAnalysis,
    RegulatoryDocument,
    Publication,
    Poster,
}

/// Deliverable status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableStatus {
    #[default]
    Draft,
    InReview,
    Approved,
    Published,
    Archived,
}

/// Review status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
    RevisionRequested,
}

// ============================================================================
// Publication & Collaboration Enums
// ============================================================================

/// Publication visibility — who can see published research
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PublicationVisibility {
    #[default]
    Community,
    Circles,
    Restricted,
}

/// Collaboration request type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CollabType {
    #[default]
    SharedProject,
    DataSharing,
    PeerReview,
    JointPublication,
}

/// Collaboration request status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CollabStatus {
    #[default]
    Pending,
    Accepted,
    Declined,
    Withdrawn,
}

// ============================================================================
// Publication & Collaboration Records
// ============================================================================

/// Record representing a published deliverable visible to the community
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationRecord {
    pub id: String,
    pub source_circle_id: String,
    pub deliverable_id: String,
    pub title: String,
    pub abstract_text: String,
    pub visibility: PublicationVisibility,
    pub published_at: DateTime,
    pub published_by: String,
}

/// Record representing a cross-circle collaboration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationRequestRecord {
    pub id: String,
    pub requesting_circle_id: String,
    pub target_circle_id: String,
    pub request_type: CollabType,
    pub message: String,
    pub status: CollabStatus,
    pub created_by: String,
    pub created_at: DateTime,
}

// ============================================================================
// Project Records
// ============================================================================

/// Record representing a Project within a Circle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: String,
    pub circle_id: String,
    pub name: String,
    pub description: String,
    pub project_type: ProjectType,

    /// Optional loop method classifier — classifies a Project as part of the
    /// Nucleus Loops epistemic pipeline (Question / Hypothesis / Thesis).
    /// `None` means this project is a general R&D project, not a loop.
    /// Surfaced in the Nucleus UI at `/nucleus/community/loops/*`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_method: Option<LoopMethod>,

    // Lifecycle
    pub stage: ProjectStage,
    pub status: ProjectStatus,

    // Scope
    pub therapeutic_area: Option<String>,
    pub drug_names: Vec<String>,
    pub indications: Vec<String>,
    pub data_sources: Vec<String>,

    // Timeline
    pub started_at: DateTime,
    pub target_completion: Option<DateTime>,
    pub completed_at: Option<DateTime>,

    // Ownership
    pub lead_user_id: String,
    pub created_by: String,

    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// Loop method — the epistemic pipeline stage a Project participates in.
///
/// Maps directly to the Nucleus UI at `/nucleus/community/loops/{slug}`:
/// - `Question` → `/loops/questions` (collaborative decomposition)
/// - `Hypothesis` → `/loops/hypotheses` (scientific method)
/// - `Thesis` → `/loops/theses` (peer review)
///
/// The pipeline flows Question → Hypothesis → Thesis via explicit promotion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopMethod {
    Question,
    Hypothesis,
    Thesis,
}

/// Record representing a Deliverable within a Project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverableRecord {
    pub id: String,
    pub project_id: String,
    pub circle_id: String,
    pub name: String,
    pub deliverable_type: DeliverableType,
    pub status: DeliverableStatus,
    pub version: u32,

    // Content
    pub file_url: Option<String>,
    pub content_hash: Option<String>,

    // Review
    pub reviewed_by: Option<String>,
    pub review_status: ReviewStatus,
    pub review_notes: Option<String>,

    pub created_by: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// ============================================================================
// Circle Records
// ============================================================================

/// Record representing a professional Circle — the organizational primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleRecord {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub mission: Option<String>,

    // Formation
    pub formation: CircleFormation,
    pub tenant_id: Option<String>,
    pub created_by: String,

    // Visibility
    pub visibility: CircleVisibility,
    pub join_policy: JoinPolicy,

    // Classification
    pub circle_type: CircleType,
    pub therapeutic_areas: Vec<String>,
    pub tags: Vec<String>,

    // Lifecycle
    pub status: CircleStatus,
    pub created_at: DateTime,
    pub updated_at: DateTime,

    // Denormalized counts
    pub member_count: u32,
    pub project_count: u32,
    pub publication_count: u32,
}

/// Record representing a Circle membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleMemberRecord {
    pub id: String,
    pub circle_id: String,
    pub user_id: String,
    pub role: CircleRole,
    pub status: MemberStatus,
    pub joined_at: DateTime,
    pub invited_by: Option<String>,
}

/// Record representing a Circle feed entry (activity stream)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntryRecord {
    pub id: String,
    pub circle_id: String,
    pub entry_type: FeedEntryType,
    pub actor_user_id: String,
    pub content: String,
    pub reference_id: Option<String>,
    pub reference_type: Option<String>,
    pub created_at: DateTime,
}

// Keep legacy MembershipRecord for backward compat during migration
/// Legacy membership record (deprecated — use CircleMemberRecord)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipRecord {
    pub id: String,
    pub user_id: String,
    pub circle_id: String,
    pub joined_at: DateTime,
}

/// Record representing a private message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: DateTime,
    pub read: bool,
}

/// Record representing a KSB (Knowledge, Skill, Behavior)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbRecord {
    pub id: String,
    pub ksb_type: String,
    pub title: String,
    pub description: String,
    pub domain: String,
}

/// Record representing a KSB Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbDomainRecord {
    pub code: String,
    pub name: String,
    pub ksb_count: u32,
    pub dominant_primitive: String,
    pub cognitive_primitive: String,
    pub transfer_confidence: f64,
    pub pvos_layer: Option<String>,
    pub example_ksbs: Vec<String>,
}

/// Unified persistence backend
pub enum Persistence {
    Firestore(FirestorePersistence),
    Mock(MockPersistence),
}

impl Persistence {
    pub async fn save_report(&self, report: &ReportRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_report(report).await,
            Self::Mock(m) => m.save_report(report).await,
        }
    }

    pub async fn list_reports(&self) -> nexcore_error::Result<Vec<ReportRecord>> {
        match self {
            Self::Firestore(f) => f.list_reports().await,
            Self::Mock(m) => m.list_reports().await,
        }
    }

    pub async fn save_post(&self, post: &PostRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_post(post).await,
            Self::Mock(m) => m.save_post(post).await,
        }
    }

    pub async fn list_posts(&self) -> nexcore_error::Result<Vec<PostRecord>> {
        match self {
            Self::Firestore(f) => f.list_posts().await,
            Self::Mock(m) => m.list_posts().await,
        }
    }

    pub async fn save_inquiry(&self, inquiry: &InquiryRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_inquiry(inquiry).await,
            Self::Mock(m) => m.save_inquiry(inquiry).await,
        }
    }

    pub async fn list_inquiries(&self) -> nexcore_error::Result<Vec<InquiryRecord>> {
        match self {
            Self::Firestore(f) => f.list_inquiries().await,
            Self::Mock(m) => m.list_inquiries().await,
        }
    }

    pub async fn update_inquiry_status(&self, id: &str, status: &str) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.update_inquiry_status(id, status).await,
            Self::Mock(m) => m.update_inquiry_status(id, status).await,
        }
    }

    pub async fn save_enrollment(
        &self,
        enrollment: &EnrollmentRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_enrollment(enrollment).await,
            Self::Mock(m) => m.save_enrollment(enrollment).await,
        }
    }

    pub async fn list_enrollments(&self) -> nexcore_error::Result<Vec<EnrollmentRecord>> {
        match self {
            Self::Firestore(f) => f.list_enrollments().await,
            Self::Mock(m) => m.list_enrollments().await,
        }
    }

    // ========================================================================
    // Circles
    // ========================================================================

    pub async fn save_circle(&self, circle: &CircleRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_circle(circle).await,
            Self::Mock(m) => m.save_circle(circle).await,
        }
    }

    pub async fn get_circle(&self, id: &str) -> nexcore_error::Result<Option<CircleRecord>> {
        match self {
            Self::Firestore(f) => f.get_circle(id).await,
            Self::Mock(m) => m.get_circle(id).await,
        }
    }

    pub async fn get_circle_by_slug(
        &self,
        slug: &str,
    ) -> nexcore_error::Result<Option<CircleRecord>> {
        match self {
            Self::Firestore(f) => f.get_circle_by_slug(slug).await,
            Self::Mock(m) => m.get_circle_by_slug(slug).await,
        }
    }

    pub async fn list_circles(&self) -> nexcore_error::Result<Vec<CircleRecord>> {
        match self {
            Self::Firestore(f) => f.list_circles().await,
            Self::Mock(m) => m.list_circles().await,
        }
    }

    pub async fn list_circles_by_tenant(
        &self,
        tenant_id: &str,
    ) -> nexcore_error::Result<Vec<CircleRecord>> {
        match self {
            Self::Firestore(f) => f.list_circles_by_tenant(tenant_id).await,
            Self::Mock(m) => m.list_circles_by_tenant(tenant_id).await,
        }
    }

    pub async fn delete_circle(&self, id: &str) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.delete_circle(id).await,
            Self::Mock(m) => m.delete_circle(id).await,
        }
    }

    // ========================================================================
    // Circle Members
    // ========================================================================

    pub async fn save_circle_member(
        &self,
        member: &CircleMemberRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_circle_member(member).await,
            Self::Mock(m) => m.save_circle_member(member).await,
        }
    }

    pub async fn get_circle_member(
        &self,
        circle_id: &str,
        user_id: &str,
    ) -> nexcore_error::Result<Option<CircleMemberRecord>> {
        match self {
            Self::Firestore(f) => f.get_circle_member(circle_id, user_id).await,
            Self::Mock(m) => m.get_circle_member(circle_id, user_id).await,
        }
    }

    pub async fn list_circle_members(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<CircleMemberRecord>> {
        match self {
            Self::Firestore(f) => f.list_circle_members(circle_id).await,
            Self::Mock(m) => m.list_circle_members(circle_id).await,
        }
    }

    pub async fn update_circle_member(
        &self,
        member: &CircleMemberRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.update_circle_member(member).await,
            Self::Mock(m) => m.update_circle_member(member).await,
        }
    }

    pub async fn delete_circle_member(
        &self,
        circle_id: &str,
        user_id: &str,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.delete_circle_member(circle_id, user_id).await,
            Self::Mock(m) => m.delete_circle_member(circle_id, user_id).await,
        }
    }

    // ========================================================================
    // Circle Feed
    // ========================================================================

    pub async fn save_feed_entry(&self, entry: &FeedEntryRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_feed_entry(entry).await,
            Self::Mock(m) => m.save_feed_entry(entry).await,
        }
    }

    pub async fn list_feed_entries(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<FeedEntryRecord>> {
        match self {
            Self::Firestore(f) => f.list_feed_entries(circle_id).await,
            Self::Mock(m) => m.list_feed_entries(circle_id).await,
        }
    }

    // ========================================================================
    // Projects
    // ========================================================================

    pub async fn save_project(&self, project: &ProjectRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_project(project).await,
            Self::Mock(m) => m.save_project(project).await,
        }
    }

    pub async fn get_project(&self, id: &str) -> nexcore_error::Result<Option<ProjectRecord>> {
        match self {
            Self::Firestore(f) => f.get_project(id).await,
            Self::Mock(m) => m.get_project(id).await,
        }
    }

    pub async fn list_projects(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<ProjectRecord>> {
        match self {
            Self::Firestore(f) => f.list_projects(circle_id).await,
            Self::Mock(m) => m.list_projects(circle_id).await,
        }
    }

    pub async fn delete_project(&self, id: &str) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.delete_project(id).await,
            Self::Mock(m) => m.delete_project(id).await,
        }
    }

    // ========================================================================
    // Deliverables
    // ========================================================================

    pub async fn save_deliverable(
        &self,
        deliverable: &DeliverableRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_deliverable(deliverable).await,
            Self::Mock(m) => m.save_deliverable(deliverable).await,
        }
    }

    pub async fn get_deliverable(
        &self,
        id: &str,
    ) -> nexcore_error::Result<Option<DeliverableRecord>> {
        match self {
            Self::Firestore(f) => f.get_deliverable(id).await,
            Self::Mock(m) => m.get_deliverable(id).await,
        }
    }

    pub async fn list_deliverables(
        &self,
        project_id: &str,
    ) -> nexcore_error::Result<Vec<DeliverableRecord>> {
        match self {
            Self::Firestore(f) => f.list_deliverables(project_id).await,
            Self::Mock(m) => m.list_deliverables(project_id).await,
        }
    }

    pub async fn delete_deliverable(&self, id: &str) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.delete_deliverable(id).await,
            Self::Mock(m) => m.delete_deliverable(id).await,
        }
    }

    // ========================================================================
    // Publications
    // ========================================================================

    pub async fn save_publication(
        &self,
        pub_record: &PublicationRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_publication(pub_record).await,
            Self::Mock(m) => m.save_publication(pub_record).await,
        }
    }

    pub async fn list_publications(&self) -> nexcore_error::Result<Vec<PublicationRecord>> {
        match self {
            Self::Firestore(f) => f.list_publications().await,
            Self::Mock(m) => m.list_publications().await,
        }
    }

    pub async fn list_circle_publications(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<PublicationRecord>> {
        match self {
            Self::Firestore(f) => f.list_circle_publications(circle_id).await,
            Self::Mock(m) => m.list_circle_publications(circle_id).await,
        }
    }

    // ========================================================================
    // Collaboration Requests
    // ========================================================================

    pub async fn save_collaboration(
        &self,
        collab: &CollaborationRequestRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_collaboration(collab).await,
            Self::Mock(m) => m.save_collaboration(collab).await,
        }
    }

    pub async fn get_collaboration(
        &self,
        id: &str,
    ) -> nexcore_error::Result<Option<CollaborationRequestRecord>> {
        match self {
            Self::Firestore(f) => f.get_collaboration(id).await,
            Self::Mock(m) => m.get_collaboration(id).await,
        }
    }

    pub async fn list_collaborations_for_circle(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<CollaborationRequestRecord>> {
        match self {
            Self::Firestore(f) => f.list_collaborations_for_circle(circle_id).await,
            Self::Mock(m) => m.list_collaborations_for_circle(circle_id).await,
        }
    }

    // ========================================================================
    // Legacy Memberships (backward compat)
    // ========================================================================

    pub async fn save_membership(
        &self,
        membership: &MembershipRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_membership(membership).await,
            Self::Mock(m) => m.save_membership(membership).await,
        }
    }

    pub async fn list_memberships(
        &self,
        user_id: &str,
    ) -> nexcore_error::Result<Vec<MembershipRecord>> {
        match self {
            Self::Firestore(f) => f.list_memberships(user_id).await,
            Self::Mock(m) => m.list_memberships(user_id).await,
        }
    }

    pub async fn save_message(&self, message: &MessageRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_message(message).await,
            Self::Mock(m) => m.save_message(message).await,
        }
    }

    pub async fn list_messages(&self, user_id: &str) -> nexcore_error::Result<Vec<MessageRecord>> {
        match self {
            Self::Firestore(f) => f.list_messages(user_id).await,
            Self::Mock(m) => m.list_messages(user_id).await,
        }
    }

    pub async fn save_ksb_domain(&self, domain: &KsbDomainRecord) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_ksb_domain(domain).await,
            Self::Mock(m) => m.save_ksb_domain(domain).await,
        }
    }

    pub async fn list_ksb_domains(&self) -> nexcore_error::Result<Vec<KsbDomainRecord>> {
        match self {
            Self::Firestore(f) => f.list_ksb_domains().await,
            Self::Mock(m) => m.list_ksb_domains().await,
        }
    }

    pub async fn save_telemetry_event(
        &self,
        event: &TelemetryEventRecord,
    ) -> nexcore_error::Result<()> {
        match self {
            Self::Firestore(f) => f.save_telemetry_event(event).await,
            Self::Mock(m) => m.save_telemetry_event(event).await,
        }
    }

    pub async fn list_telemetry_events(&self) -> nexcore_error::Result<Vec<TelemetryEventRecord>> {
        match self {
            Self::Firestore(f) => f.list_telemetry_events().await,
            Self::Mock(m) => m.list_telemetry_events().await,
        }
    }
}
