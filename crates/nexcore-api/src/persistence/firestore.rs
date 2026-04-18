//! Firestore persistence implementation

use crate::persistence::{
    CircleFormation, CircleMemberRecord, CircleRecord, CollaborationRequestRecord,
    DeliverableRecord, EnrollmentRecord, FeedEntryRecord, InquiryRecord, KsbDomainRecord,
    MembershipRecord, MessageRecord, PostRecord, ProjectRecord, PublicationRecord, ReportRecord,
    TelemetryEventRecord,
};
use serde_json::json;

/// Firestore REST API base URL
const FIRESTORE_BASE: &str = "https://firestore.googleapis.com/v1";

pub struct FirestorePersistence {
    project_id: String,
    collection: String, // Base collection, but we'll use specific ones below
    http: reqwest::Client,
    api_key: Option<String>,
}

impl FirestorePersistence {
    pub fn new(project_id: String, _collection: String, api_key: Option<String>) -> Self {
        Self {
            project_id,
            collection: "nexcore".to_string(), // Root collection
            http: reqwest::Client::new(),
            api_key,
        }
    }

    fn url(&self, collection: &str, doc_id: Option<&str>) -> String {
        let base = format!(
            "{FIRESTORE_BASE}/projects/{}/databases/(default)/documents/{}",
            self.project_id, collection
        );
        match doc_id {
            Some(id) => format!("{}/{}", base, id),
            None => base,
        }
    }

    pub async fn save_report(&self, report: &ReportRecord) -> nexcore_error::Result<()> {
        let url = self.url("reports", Some(&report.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": report.id },
                "report_type": { "stringValue": report.report_type },
                "generated_at": { "stringValue": report.generated_at.to_rfc3339() },
                "content": { "stringValue": report.content },
                "status": { "stringValue": report.status },
                "user_id": { "stringValue": report.user_id.clone().unwrap_or_default() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_reports(&self) -> nexcore_error::Result<Vec<ReportRecord>> {
        let url = self.url("reports", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_report_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_post(&self, post: &PostRecord) -> nexcore_error::Result<()> {
        let url = self.url("posts", Some(&post.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": post.id },
                "author": { "stringValue": post.author },
                "role": { "stringValue": post.role },
                "content": { "stringValue": post.content },
                "likes": { "integerValue": post.likes.to_string() },
                "replies": { "integerValue": post.replies.to_string() },
                "created_at": { "stringValue": post.created_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_posts(&self) -> nexcore_error::Result<Vec<PostRecord>> {
        let url = self.url("posts", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_post_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_inquiry(&self, inquiry: &InquiryRecord) -> nexcore_error::Result<()> {
        let url = self.url("inquiries", Some(&inquiry.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": inquiry.id },
                "name": { "stringValue": inquiry.name },
                "email": { "stringValue": inquiry.email },
                "organization": { "stringValue": inquiry.organization },
                "interest": { "stringValue": inquiry.interest },
                "message": { "stringValue": inquiry.message },
                "created_at": { "stringValue": inquiry.created_at.to_rfc3339() },
                "status": { "stringValue": inquiry.status },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_inquiries(&self) -> nexcore_error::Result<Vec<InquiryRecord>> {
        let url = self.url("inquiries", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_inquiry_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn update_inquiry_status(&self, id: &str, status: &str) -> nexcore_error::Result<()> {
        let url = self.url("inquiries", Some(id));
        let body = json!({
            "fields": {
                "status": { "stringValue": status },
            }
        });

        // Use mask to only update the status field
        let url_with_mask = format!("{}?updateMask.fieldPaths=status", url);
        self.patch(&url_with_mask, &body).await
    }

    pub async fn save_enrollment(
        &self,
        enrollment: &EnrollmentRecord,
    ) -> nexcore_error::Result<()> {
        let url = self.url("enrollments", Some(&enrollment.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": enrollment.id },
                "user_id": { "stringValue": enrollment.user_id },
                "course_id": { "stringValue": enrollment.course_id },
                "progress": { "doubleValue": enrollment.progress },
                "enrolled_at": { "stringValue": enrollment.enrolled_at.to_rfc3339() },
                "completed_at": { "stringValue": enrollment.completed_at.map(|d| d.to_rfc3339()).unwrap_or_default() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_enrollments(&self) -> nexcore_error::Result<Vec<EnrollmentRecord>> {
        let url = self.url("enrollments", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_enrollment_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    // ========================================================================
    // Circles (expanded)
    // ========================================================================

    pub async fn save_circle(&self, circle: &CircleRecord) -> nexcore_error::Result<()> {
        let url = self.url("circles", Some(&circle.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": circle.id },
                "name": { "stringValue": circle.name },
                "slug": { "stringValue": circle.slug },
                "description": { "stringValue": circle.description },
                "mission": { "stringValue": circle.mission.clone().unwrap_or_default() },
                "formation": { "stringValue": serde_json::to_string(&circle.formation).unwrap_or_default() },
                "tenant_id": { "stringValue": circle.tenant_id.clone().unwrap_or_default() },
                "created_by": { "stringValue": circle.created_by },
                "visibility": { "stringValue": serde_json::to_string(&circle.visibility).unwrap_or_default() },
                "join_policy": { "stringValue": serde_json::to_string(&circle.join_policy).unwrap_or_default() },
                "circle_type": { "stringValue": serde_json::to_string(&circle.circle_type).unwrap_or_default() },
                "therapeutic_areas": { "arrayValue": { "values": circle.therapeutic_areas.iter().map(|t| json!({"stringValue": t})).collect::<Vec<_>>() } },
                "tags": { "arrayValue": { "values": circle.tags.iter().map(|t| json!({"stringValue": t})).collect::<Vec<_>>() } },
                "status": { "stringValue": serde_json::to_string(&circle.status).unwrap_or_default() },
                "created_at": { "stringValue": circle.created_at.to_rfc3339() },
                "updated_at": { "stringValue": circle.updated_at.to_rfc3339() },
                "member_count": { "integerValue": circle.member_count.to_string() },
                "project_count": { "integerValue": circle.project_count.to_string() },
                "publication_count": { "integerValue": circle.publication_count.to_string() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn get_circle(&self, id: &str) -> nexcore_error::Result<Option<CircleRecord>> {
        let url = self.url("circles", Some(id));
        let doc = self.get_doc(&url).await?;
        match doc {
            Some(d) => Ok(Some(parse_circle_doc(&d)?)),
            None => Ok(None),
        }
    }

    pub async fn get_circle_by_slug(
        &self,
        slug: &str,
    ) -> nexcore_error::Result<Option<CircleRecord>> {
        let circles = self.list_circles().await?;
        Ok(circles.into_iter().find(|c| c.slug == slug))
    }

    pub async fn list_circles(&self) -> nexcore_error::Result<Vec<CircleRecord>> {
        let url = self.url("circles", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_circle_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn list_circles_by_tenant(
        &self,
        tenant_id: &str,
    ) -> nexcore_error::Result<Vec<CircleRecord>> {
        let all = self.list_circles().await?;
        Ok(all
            .into_iter()
            .filter(|c| {
                c.formation == CircleFormation::OrgBacked
                    && c.tenant_id.as_deref() == Some(tenant_id)
            })
            .collect())
    }

    pub async fn delete_circle(&self, id: &str) -> nexcore_error::Result<()> {
        let url = self.url("circles", Some(id));
        self.delete(&url).await
    }

    // ========================================================================
    // Circle Members
    // ========================================================================

    pub async fn save_circle_member(
        &self,
        member: &CircleMemberRecord,
    ) -> nexcore_error::Result<()> {
        let url = self.url("circle_members", Some(&member.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": member.id },
                "circle_id": { "stringValue": member.circle_id },
                "user_id": { "stringValue": member.user_id },
                "role": { "stringValue": serde_json::to_string(&member.role).unwrap_or_default() },
                "status": { "stringValue": serde_json::to_string(&member.status).unwrap_or_default() },
                "joined_at": { "stringValue": member.joined_at.to_rfc3339() },
                "invited_by": { "stringValue": member.invited_by.clone().unwrap_or_default() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn get_circle_member(
        &self,
        circle_id: &str,
        user_id: &str,
    ) -> nexcore_error::Result<Option<CircleMemberRecord>> {
        let members = self.list_circle_members(circle_id).await?;
        Ok(members.into_iter().find(|m| m.user_id == user_id))
    }

    pub async fn list_circle_members(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<CircleMemberRecord>> {
        let url = self.url("circle_members", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_circle_member_doc(&doc) {
                if record.circle_id == circle_id {
                    results.push(record);
                }
            }
        }
        Ok(results)
    }

    pub async fn update_circle_member(
        &self,
        member: &CircleMemberRecord,
    ) -> nexcore_error::Result<()> {
        self.save_circle_member(member).await
    }

    pub async fn delete_circle_member(
        &self,
        circle_id: &str,
        user_id: &str,
    ) -> nexcore_error::Result<()> {
        let members = self.list_circle_members(circle_id).await?;
        if let Some(member) = members.iter().find(|m| m.user_id == user_id) {
            let url = self.url("circle_members", Some(&member.id));
            self.delete(&url).await
        } else {
            Ok(())
        }
    }

    // ========================================================================
    // Circle Feed
    // ========================================================================

    pub async fn save_feed_entry(&self, entry: &FeedEntryRecord) -> nexcore_error::Result<()> {
        let url = self.url("circle_feed", Some(&entry.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": entry.id },
                "circle_id": { "stringValue": entry.circle_id },
                "entry_type": { "stringValue": serde_json::to_string(&entry.entry_type).unwrap_or_default() },
                "actor_user_id": { "stringValue": entry.actor_user_id },
                "content": { "stringValue": entry.content },
                "reference_id": { "stringValue": entry.reference_id.clone().unwrap_or_default() },
                "reference_type": { "stringValue": entry.reference_type.clone().unwrap_or_default() },
                "created_at": { "stringValue": entry.created_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_feed_entries(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<FeedEntryRecord>> {
        let url = self.url("circle_feed", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_feed_entry_doc(&doc) {
                if record.circle_id == circle_id {
                    results.push(record);
                }
            }
        }
        Ok(results)
    }

    // ========================================================================
    // Projects
    // ========================================================================

    pub async fn save_project(&self, project: &ProjectRecord) -> nexcore_error::Result<()> {
        let url = self.url("projects", Some(&project.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": project.id },
                "circle_id": { "stringValue": project.circle_id },
                "name": { "stringValue": project.name },
                "description": { "stringValue": project.description },
                "project_type": { "stringValue": serde_json::to_string(&project.project_type).unwrap_or_default() },
                "loop_method": { "stringValue": project.loop_method.as_ref().map(|m| match m {
                    crate::persistence::LoopMethod::Question => "question",
                    crate::persistence::LoopMethod::Hypothesis => "hypothesis",
                    crate::persistence::LoopMethod::Thesis => "thesis",
                }).unwrap_or("") },
                "stage": { "stringValue": serde_json::to_string(&project.stage).unwrap_or_default() },
                "status": { "stringValue": serde_json::to_string(&project.status).unwrap_or_default() },
                "therapeutic_area": { "stringValue": project.therapeutic_area.clone().unwrap_or_default() },
                "drug_names": { "arrayValue": { "values": project.drug_names.iter().map(|d| json!({"stringValue": d})).collect::<Vec<_>>() } },
                "indications": { "arrayValue": { "values": project.indications.iter().map(|i| json!({"stringValue": i})).collect::<Vec<_>>() } },
                "data_sources": { "arrayValue": { "values": project.data_sources.iter().map(|s| json!({"stringValue": s})).collect::<Vec<_>>() } },
                "started_at": { "stringValue": project.started_at.to_rfc3339() },
                "target_completion": { "stringValue": project.target_completion.map(|d| d.to_rfc3339()).unwrap_or_default() },
                "completed_at": { "stringValue": project.completed_at.map(|d| d.to_rfc3339()).unwrap_or_default() },
                "lead_user_id": { "stringValue": project.lead_user_id },
                "created_by": { "stringValue": project.created_by },
                "created_at": { "stringValue": project.created_at.to_rfc3339() },
                "updated_at": { "stringValue": project.updated_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn get_project(&self, id: &str) -> nexcore_error::Result<Option<ProjectRecord>> {
        let url = self.url("projects", Some(id));
        let doc = self.get_doc(&url).await?;
        match doc {
            Some(d) => Ok(Some(parse_project_doc(&d)?)),
            None => Ok(None),
        }
    }

    pub async fn list_projects(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<ProjectRecord>> {
        let url = self.url("projects", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_project_doc(&doc) {
                if record.circle_id == circle_id {
                    results.push(record);
                }
            }
        }
        Ok(results)
    }

    pub async fn delete_project(&self, id: &str) -> nexcore_error::Result<()> {
        let url = self.url("projects", Some(id));
        self.delete(&url).await
    }

    // ========================================================================
    // Deliverables
    // ========================================================================

    pub async fn save_deliverable(
        &self,
        deliverable: &DeliverableRecord,
    ) -> nexcore_error::Result<()> {
        let url = self.url("deliverables", Some(&deliverable.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": deliverable.id },
                "project_id": { "stringValue": deliverable.project_id },
                "circle_id": { "stringValue": deliverable.circle_id },
                "name": { "stringValue": deliverable.name },
                "deliverable_type": { "stringValue": serde_json::to_string(&deliverable.deliverable_type).unwrap_or_default() },
                "status": { "stringValue": serde_json::to_string(&deliverable.status).unwrap_or_default() },
                "version": { "integerValue": deliverable.version.to_string() },
                "file_url": { "stringValue": deliverable.file_url.clone().unwrap_or_default() },
                "content_hash": { "stringValue": deliverable.content_hash.clone().unwrap_or_default() },
                "reviewed_by": { "stringValue": deliverable.reviewed_by.clone().unwrap_or_default() },
                "review_status": { "stringValue": serde_json::to_string(&deliverable.review_status).unwrap_or_default() },
                "review_notes": { "stringValue": deliverable.review_notes.clone().unwrap_or_default() },
                "created_by": { "stringValue": deliverable.created_by },
                "created_at": { "stringValue": deliverable.created_at.to_rfc3339() },
                "updated_at": { "stringValue": deliverable.updated_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn get_deliverable(
        &self,
        id: &str,
    ) -> nexcore_error::Result<Option<DeliverableRecord>> {
        let url = self.url("deliverables", Some(id));
        let doc = self.get_doc(&url).await?;
        match doc {
            Some(d) => Ok(Some(parse_deliverable_doc(&d)?)),
            None => Ok(None),
        }
    }

    pub async fn list_deliverables(
        &self,
        project_id: &str,
    ) -> nexcore_error::Result<Vec<DeliverableRecord>> {
        let url = self.url("deliverables", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_deliverable_doc(&doc) {
                if record.project_id == project_id {
                    results.push(record);
                }
            }
        }
        Ok(results)
    }

    pub async fn delete_deliverable(&self, id: &str) -> nexcore_error::Result<()> {
        let url = self.url("deliverables", Some(id));
        self.delete(&url).await
    }

    // ========================================================================
    // Publications
    // ========================================================================

    pub async fn save_publication(
        &self,
        pub_record: &PublicationRecord,
    ) -> nexcore_error::Result<()> {
        let url = self.url("publications", Some(&pub_record.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": pub_record.id },
                "source_circle_id": { "stringValue": pub_record.source_circle_id },
                "deliverable_id": { "stringValue": pub_record.deliverable_id },
                "title": { "stringValue": pub_record.title },
                "abstract_text": { "stringValue": pub_record.abstract_text },
                "visibility": { "stringValue": serde_json::to_string(&pub_record.visibility).unwrap_or_default() },
                "published_at": { "stringValue": pub_record.published_at.to_rfc3339() },
                "published_by": { "stringValue": pub_record.published_by },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_publications(&self) -> nexcore_error::Result<Vec<PublicationRecord>> {
        let url = self.url("publications", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_publication_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn list_circle_publications(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<PublicationRecord>> {
        let all = self.list_publications().await?;
        Ok(all
            .into_iter()
            .filter(|p| p.source_circle_id == circle_id)
            .collect())
    }

    // ========================================================================
    // Collaboration Requests
    // ========================================================================

    pub async fn save_collaboration(
        &self,
        collab: &CollaborationRequestRecord,
    ) -> nexcore_error::Result<()> {
        let url = self.url("collaboration_requests", Some(&collab.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": collab.id },
                "requesting_circle_id": { "stringValue": collab.requesting_circle_id },
                "target_circle_id": { "stringValue": collab.target_circle_id },
                "request_type": { "stringValue": serde_json::to_string(&collab.request_type).unwrap_or_default() },
                "message": { "stringValue": collab.message },
                "status": { "stringValue": serde_json::to_string(&collab.status).unwrap_or_default() },
                "created_by": { "stringValue": collab.created_by },
                "created_at": { "stringValue": collab.created_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn get_collaboration(
        &self,
        id: &str,
    ) -> nexcore_error::Result<Option<CollaborationRequestRecord>> {
        let url = self.url("collaboration_requests", Some(id));
        let doc = self.get_doc(&url).await?;
        match doc {
            Some(d) => Ok(Some(parse_collaboration_doc(&d)?)),
            None => Ok(None),
        }
    }

    pub async fn list_collaborations_for_circle(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<CollaborationRequestRecord>> {
        let url = self.url("collaboration_requests", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_collaboration_doc(&doc) {
                if record.requesting_circle_id == circle_id || record.target_circle_id == circle_id
                {
                    results.push(record);
                }
            }
        }
        Ok(results)
    }

    // ========================================================================
    // Legacy Memberships
    // ========================================================================

    pub async fn save_membership(
        &self,
        membership: &MembershipRecord,
    ) -> nexcore_error::Result<()> {
        let url = self.url("memberships", Some(&membership.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": membership.id },
                "user_id": { "stringValue": membership.user_id },
                "circle_id": { "stringValue": membership.circle_id },
                "joined_at": { "stringValue": membership.joined_at.to_rfc3339() },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_memberships(
        &self,
        _user_id: &str,
    ) -> nexcore_error::Result<Vec<MembershipRecord>> {
        let url = self.url("memberships", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_membership_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_message(&self, message: &MessageRecord) -> nexcore_error::Result<()> {
        let url = self.url("messages", Some(&message.id));
        let body = json!({
            "fields": {
                "id": { "stringValue": message.id },
                "sender_id": { "stringValue": message.sender_id },
                "recipient_id": { "stringValue": message.recipient_id },
                "content": { "stringValue": message.content },
                "created_at": { "stringValue": message.created_at.to_rfc3339() },
                "read": { "booleanValue": message.read },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_messages(&self, _user_id: &str) -> nexcore_error::Result<Vec<MessageRecord>> {
        let url = self.url("messages", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_message_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_ksb_domain(&self, domain: &KsbDomainRecord) -> nexcore_error::Result<()> {
        let url = self.url("ksb_domains", Some(&domain.code));
        let body = json!({
            "fields": {
                "code": { "stringValue": domain.code },
                "name": { "stringValue": domain.name },
                "ksb_count": { "integerValue": domain.ksb_count.to_string() },
                "dominant_primitive": { "stringValue": domain.dominant_primitive },
                "cognitive_primitive": { "stringValue": domain.cognitive_primitive },
                "transfer_confidence": { "doubleValue": domain.transfer_confidence },
                "pvos_layer": { "stringValue": domain.pvos_layer.clone().unwrap_or_default() },
                "example_ksbs": { "arrayValue": { "values": domain.example_ksbs.iter().map(|k| json!({ "stringValue": k })).collect::<Vec<_>>() } }
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_ksb_domains(&self) -> nexcore_error::Result<Vec<KsbDomainRecord>> {
        let url = self.url("ksb_domains", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_ksb_domain_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    pub async fn save_telemetry_event(
        &self,
        event: &TelemetryEventRecord,
    ) -> nexcore_error::Result<()> {
        let url = self.url("telemetry_events", Some(&event.id));
        let body = serde_json::json!({
            "fields": {
                "id": { "stringValue": event.id },
                "event_type": { "stringValue": event.event_type },
                "user_id": { "stringValue": event.user_id },
                "metadata": { "stringValue": event.metadata.to_string() },
                "timestamp": { "stringValue": event.timestamp },
            }
        });
        self.patch(&url, &body).await
    }

    pub async fn list_telemetry_events(&self) -> nexcore_error::Result<Vec<TelemetryEventRecord>> {
        let url = self.url("telemetry_events", None);
        let documents = self.get_list(&url).await?;
        let mut results = Vec::new();
        for doc in documents {
            if let Ok(record) = parse_telemetry_event_doc(&doc) {
                results.push(record);
            }
        }
        Ok(results)
    }

    async fn get_doc(&self, url: &str) -> nexcore_error::Result<Option<serde_json::Value>> {
        let mut req = self.http.get(url);
        if let Some(ref key) = self.api_key {
            req = req.query(&[("key", key)]);
        }
        let resp = req.send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            nexcore_error::bail!("Firestore error: {}", err_text);
        }
        let body: serde_json::Value = resp.json().await?;
        Ok(Some(body))
    }

    async fn delete(&self, url: &str) -> nexcore_error::Result<()> {
        let mut req = self.http.delete(url);
        if let Some(ref key) = self.api_key {
            req = req.query(&[("key", key)]);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::NOT_FOUND {
            let err_text = resp.text().await?;
            nexcore_error::bail!("Firestore error: {}", err_text);
        }
        Ok(())
    }

    async fn patch(&self, url: &str, body: &serde_json::Value) -> nexcore_error::Result<()> {
        let mut req = self.http.patch(url).json(body);
        if let Some(ref key) = self.api_key {
            req = req.query(&[("key", key)]);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            nexcore_error::bail!("Firestore error: {}", err_text);
        }
        Ok(())
    }

    async fn get_list(&self, url: &str) -> nexcore_error::Result<Vec<serde_json::Value>> {
        let mut req = self.http.get(url);
        if let Some(ref key) = self.api_key {
            req = req.query(&[("key", key)]);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            nexcore_error::bail!("Firestore error: {}", err_text);
        }
        let body: serde_json::Value = resp.json().await?;
        Ok(body
            .get("documents")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default())
    }
}

fn parse_report_doc(doc: &serde_json::Value) -> nexcore_error::Result<ReportRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(ReportRecord {
        id: get_string(fields, "id"),
        report_type: get_string(fields, "report_type"),
        generated_at: get_timestamp(fields, "generated_at"),
        content: get_string(fields, "content"),
        status: get_string(fields, "status"),
        user_id: fields
            .get("user_id")
            .and_then(|v| v.get("stringValue"))
            .and_then(|s| s.as_str())
            .map(String::from),
    })
}

fn parse_post_doc(doc: &serde_json::Value) -> nexcore_error::Result<PostRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(PostRecord {
        id: get_string(fields, "id"),
        author: get_string(fields, "author"),
        role: get_string(fields, "role"),
        content: get_string(fields, "content"),
        likes: get_int(fields, "likes"),
        replies: get_int(fields, "replies"),
        created_at: get_timestamp(fields, "created_at"),
    })
}

fn parse_inquiry_doc(doc: &serde_json::Value) -> nexcore_error::Result<InquiryRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(InquiryRecord {
        id: get_string(fields, "id"),
        name: get_string(fields, "name"),
        email: get_string(fields, "email"),
        organization: get_string(fields, "organization"),
        interest: get_string(fields, "interest"),
        message: get_string(fields, "message"),
        created_at: get_timestamp(fields, "created_at"),
        status: get_string(fields, "status"),
    })
}

fn parse_enrollment_doc(doc: &serde_json::Value) -> nexcore_error::Result<EnrollmentRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(EnrollmentRecord {
        id: get_string(fields, "id"),
        user_id: get_string(fields, "user_id"),
        course_id: get_string(fields, "course_id"),
        progress: fields
            .get("progress")
            .and_then(|v| v.get("doubleValue"))
            .and_then(|n| n.as_f64())
            .unwrap_or(0.0),
        enrolled_at: get_timestamp(fields, "enrolled_at"),
        completed_at: fields
            .get("completed_at")
            .and_then(|v| v.get("stringValue"))
            .and_then(|s| s.as_str())
            .and_then(|s| s.parse().ok()),
    })
}

fn parse_circle_doc(doc: &serde_json::Value) -> nexcore_error::Result<CircleRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(CircleRecord {
        id: get_string(fields, "id"),
        name: get_string(fields, "name"),
        slug: get_string(fields, "slug"),
        description: get_string(fields, "description"),
        mission: get_optional_string(fields, "mission"),
        formation: parse_enum_field(&get_string(fields, "formation")),
        tenant_id: get_optional_string(fields, "tenant_id"),
        created_by: get_string(fields, "created_by"),
        visibility: parse_enum_field(&get_string(fields, "visibility")),
        join_policy: parse_enum_field(&get_string(fields, "join_policy")),
        circle_type: parse_enum_field(&get_string(fields, "circle_type")),
        therapeutic_areas: get_string_array(fields, "therapeutic_areas"),
        tags: get_string_array(fields, "tags"),
        status: parse_enum_field(&get_string(fields, "status")),
        created_at: get_timestamp(fields, "created_at"),
        updated_at: get_timestamp(fields, "updated_at"),
        member_count: get_int(fields, "member_count"),
        project_count: get_int(fields, "project_count"),
        publication_count: get_int(fields, "publication_count"),
    })
}

fn parse_circle_member_doc(doc: &serde_json::Value) -> nexcore_error::Result<CircleMemberRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(CircleMemberRecord {
        id: get_string(fields, "id"),
        circle_id: get_string(fields, "circle_id"),
        user_id: get_string(fields, "user_id"),
        role: parse_enum_field(&get_string(fields, "role")),
        status: parse_enum_field(&get_string(fields, "status")),
        joined_at: get_timestamp(fields, "joined_at"),
        invited_by: get_optional_string(fields, "invited_by"),
    })
}

fn parse_feed_entry_doc(doc: &serde_json::Value) -> nexcore_error::Result<FeedEntryRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(FeedEntryRecord {
        id: get_string(fields, "id"),
        circle_id: get_string(fields, "circle_id"),
        entry_type: parse_enum_field(&get_string(fields, "entry_type")),
        actor_user_id: get_string(fields, "actor_user_id"),
        content: get_string(fields, "content"),
        reference_id: get_optional_string(fields, "reference_id"),
        reference_type: get_optional_string(fields, "reference_type"),
        created_at: get_timestamp(fields, "created_at"),
    })
}

fn parse_project_doc(doc: &serde_json::Value) -> nexcore_error::Result<ProjectRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(ProjectRecord {
        id: get_string(fields, "id"),
        circle_id: get_string(fields, "circle_id"),
        name: get_string(fields, "name"),
        description: get_string(fields, "description"),
        project_type: parse_enum_field(&get_string(fields, "project_type")),
        loop_method: get_optional_string(fields, "loop_method").and_then(|s| {
            match s.to_lowercase().trim() {
                "question" => Some(crate::persistence::LoopMethod::Question),
                "hypothesis" => Some(crate::persistence::LoopMethod::Hypothesis),
                "thesis" => Some(crate::persistence::LoopMethod::Thesis),
                _ => None,
            }
        }),
        stage: parse_enum_field(&get_string(fields, "stage")),
        status: parse_enum_field(&get_string(fields, "status")),
        therapeutic_area: get_optional_string(fields, "therapeutic_area"),
        drug_names: get_string_array(fields, "drug_names"),
        indications: get_string_array(fields, "indications"),
        data_sources: get_string_array(fields, "data_sources"),
        started_at: get_timestamp(fields, "started_at"),
        target_completion: get_optional_string(fields, "target_completion")
            .and_then(|s| s.parse().ok()),
        completed_at: get_optional_string(fields, "completed_at").and_then(|s| s.parse().ok()),
        lead_user_id: get_string(fields, "lead_user_id"),
        created_by: get_string(fields, "created_by"),
        created_at: get_timestamp(fields, "created_at"),
        updated_at: get_timestamp(fields, "updated_at"),
    })
}

fn parse_deliverable_doc(doc: &serde_json::Value) -> nexcore_error::Result<DeliverableRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(DeliverableRecord {
        id: get_string(fields, "id"),
        project_id: get_string(fields, "project_id"),
        circle_id: get_string(fields, "circle_id"),
        name: get_string(fields, "name"),
        deliverable_type: parse_enum_field(&get_string(fields, "deliverable_type")),
        status: parse_enum_field(&get_string(fields, "status")),
        version: get_int(fields, "version"),
        file_url: get_optional_string(fields, "file_url"),
        content_hash: get_optional_string(fields, "content_hash"),
        reviewed_by: get_optional_string(fields, "reviewed_by"),
        review_status: parse_enum_field(&get_string(fields, "review_status")),
        review_notes: get_optional_string(fields, "review_notes"),
        created_by: get_string(fields, "created_by"),
        created_at: get_timestamp(fields, "created_at"),
        updated_at: get_timestamp(fields, "updated_at"),
    })
}

fn parse_publication_doc(doc: &serde_json::Value) -> nexcore_error::Result<PublicationRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(PublicationRecord {
        id: get_string(fields, "id"),
        source_circle_id: get_string(fields, "source_circle_id"),
        deliverable_id: get_string(fields, "deliverable_id"),
        title: get_string(fields, "title"),
        abstract_text: get_string(fields, "abstract_text"),
        visibility: parse_enum_field(&get_string(fields, "visibility")),
        published_at: get_timestamp(fields, "published_at"),
        published_by: get_string(fields, "published_by"),
    })
}

fn parse_collaboration_doc(
    doc: &serde_json::Value,
) -> nexcore_error::Result<CollaborationRequestRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(CollaborationRequestRecord {
        id: get_string(fields, "id"),
        requesting_circle_id: get_string(fields, "requesting_circle_id"),
        target_circle_id: get_string(fields, "target_circle_id"),
        request_type: parse_enum_field(&get_string(fields, "request_type")),
        message: get_string(fields, "message"),
        status: parse_enum_field(&get_string(fields, "status")),
        created_by: get_string(fields, "created_by"),
        created_at: get_timestamp(fields, "created_at"),
    })
}

fn parse_membership_doc(doc: &serde_json::Value) -> nexcore_error::Result<MembershipRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(MembershipRecord {
        id: get_string(fields, "id"),
        user_id: get_string(fields, "user_id"),
        circle_id: get_string(fields, "circle_id"),
        joined_at: get_timestamp(fields, "joined_at"),
    })
}

fn parse_message_doc(doc: &serde_json::Value) -> nexcore_error::Result<MessageRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    Ok(MessageRecord {
        id: get_string(fields, "id"),
        sender_id: get_string(fields, "sender_id"),
        recipient_id: get_string(fields, "recipient_id"),
        content: get_string(fields, "content"),
        created_at: get_timestamp(fields, "created_at"),
        read: fields
            .get("read")
            .and_then(|v| v.get("booleanValue"))
            .and_then(|b| b.as_bool())
            .unwrap_or(false),
    })
}

fn parse_ksb_domain_doc(doc: &serde_json::Value) -> nexcore_error::Result<KsbDomainRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    let examples = fields
        .get("example_ksbs")
        .and_then(|v| v.get("arrayValue"))
        .and_then(|a| a.get("values"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|val| {
                    val.get("stringValue")
                        .and_then(|s| s.as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(KsbDomainRecord {
        code: get_string(fields, "code"),
        name: get_string(fields, "name"),
        ksb_count: get_int(fields, "ksb_count"),
        dominant_primitive: get_string(fields, "dominant_primitive"),
        cognitive_primitive: get_string(fields, "cognitive_primitive"),
        transfer_confidence: fields
            .get("transfer_confidence")
            .and_then(|v| v.get("doubleValue"))
            .and_then(|n| n.as_f64())
            .unwrap_or(0.0),
        pvos_layer: fields
            .get("pvos_layer")
            .and_then(|v| v.get("stringValue"))
            .and_then(|s| s.as_str())
            .map(String::from),
        example_ksbs: examples,
    })
}

fn parse_telemetry_event_doc(
    doc: &serde_json::Value,
) -> nexcore_error::Result<TelemetryEventRecord> {
    let fields = doc
        .get("fields")
        .ok_or_else(|| nexcore_error::nexerror!("Missing fields"))?;
    let metadata_str = get_string(fields, "metadata");
    let metadata = serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null);
    Ok(TelemetryEventRecord {
        id: get_string(fields, "id"),
        event_type: get_string(fields, "event_type"),
        user_id: get_string(fields, "user_id"),
        metadata,
        timestamp: get_string(fields, "timestamp"),
    })
}

fn get_optional_string(fields: &serde_json::Value, key: &str) -> Option<String> {
    fields
        .get(key)
        .and_then(|v| v.get("stringValue"))
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn get_string_array(fields: &serde_json::Value, key: &str) -> Vec<String> {
    fields
        .get(key)
        .and_then(|v| v.get("arrayValue"))
        .and_then(|a| a.get("values"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|val| {
                    val.get("stringValue")
                        .and_then(|s| s.as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_enum_field<T: serde::de::DeserializeOwned + Default>(json_str: &str) -> T {
    if json_str.is_empty() {
        return T::default();
    }
    serde_json::from_str(json_str).unwrap_or_default()
}

fn get_string(fields: &serde_json::Value, key: &str) -> String {
    fields
        .get(key)
        .and_then(|v| v.get("stringValue"))
        .and_then(|s| s.as_str())
        .unwrap_or_default()
        .to_string()
}

fn get_timestamp(fields: &serde_json::Value, key: &str) -> nexcore_chrono::DateTime {
    fields
        .get(key)
        .and_then(|v| v.get("stringValue"))
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| nexcore_chrono::DateTime::now())
}

fn get_int(fields: &serde_json::Value, key: &str) -> u32 {
    fields
        .get(key)
        .and_then(|v| v.get("integerValue"))
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Mock implementation for local development
pub struct MockPersistence {
    reports: std::sync::Arc<std::sync::Mutex<Vec<ReportRecord>>>,
    posts: std::sync::Arc<std::sync::Mutex<Vec<PostRecord>>>,
    inquiries: std::sync::Arc<std::sync::Mutex<Vec<InquiryRecord>>>,
    enrollments: std::sync::Arc<std::sync::Mutex<Vec<EnrollmentRecord>>>,
    circles: std::sync::Arc<std::sync::Mutex<Vec<CircleRecord>>>,
    circle_members: std::sync::Arc<std::sync::Mutex<Vec<CircleMemberRecord>>>,
    feed_entries: std::sync::Arc<std::sync::Mutex<Vec<FeedEntryRecord>>>,
    projects: std::sync::Arc<std::sync::Mutex<Vec<ProjectRecord>>>,
    deliverables: std::sync::Arc<std::sync::Mutex<Vec<DeliverableRecord>>>,
    publications: std::sync::Arc<std::sync::Mutex<Vec<PublicationRecord>>>,
    collaborations: std::sync::Arc<std::sync::Mutex<Vec<CollaborationRequestRecord>>>,
    memberships: std::sync::Arc<std::sync::Mutex<Vec<MembershipRecord>>>,
    messages: std::sync::Arc<std::sync::Mutex<Vec<MessageRecord>>>,
    ksb_domains: std::sync::Arc<std::sync::Mutex<Vec<KsbDomainRecord>>>,
    telemetry_events: std::sync::Arc<std::sync::Mutex<Vec<TelemetryEventRecord>>>,
}

impl MockPersistence {
    pub fn new() -> Self {
        Self {
            reports: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            posts: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            inquiries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            enrollments: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            circles: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            circle_members: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            feed_entries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            projects: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            deliverables: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            publications: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            collaborations: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            memberships: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            messages: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            ksb_domains: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            telemetry_events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn save_report(&self, report: &ReportRecord) -> nexcore_error::Result<()> {
        let mut reports = self
            .reports
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        reports.push(report.clone());
        Ok(())
    }

    pub async fn list_reports(&self) -> nexcore_error::Result<Vec<ReportRecord>> {
        let reports = self
            .reports
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(reports.clone())
    }

    pub async fn save_post(&self, post: &PostRecord) -> nexcore_error::Result<()> {
        let mut posts = self
            .posts
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        posts.push(post.clone());
        Ok(())
    }

    pub async fn list_posts(&self) -> nexcore_error::Result<Vec<PostRecord>> {
        let posts = self
            .posts
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(posts.clone())
    }

    pub async fn save_inquiry(&self, inquiry: &InquiryRecord) -> nexcore_error::Result<()> {
        let mut inquiries = self
            .inquiries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        inquiries.push(inquiry.clone());
        Ok(())
    }

    pub async fn list_inquiries(&self) -> nexcore_error::Result<Vec<InquiryRecord>> {
        let inquiries = self
            .inquiries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(inquiries.clone())
    }

    pub async fn update_inquiry_status(&self, id: &str, status: &str) -> nexcore_error::Result<()> {
        let mut inquiries = self
            .inquiries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        if let Some(inquiry) = inquiries.iter_mut().find(|i| i.id == id) {
            inquiry.status = status.to_string();
            Ok(())
        } else {
            nexcore_error::bail!("Inquiry not found")
        }
    }

    pub async fn save_enrollment(
        &self,
        enrollment: &EnrollmentRecord,
    ) -> nexcore_error::Result<()> {
        let mut enrollments = self
            .enrollments
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        enrollments.push(enrollment.clone());
        Ok(())
    }

    pub async fn list_enrollments(&self) -> nexcore_error::Result<Vec<EnrollmentRecord>> {
        let enrollments = self
            .enrollments
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(enrollments.clone())
    }

    // ========================================================================
    // Circles (Mock)
    // ========================================================================

    pub async fn save_circle(&self, circle: &CircleRecord) -> nexcore_error::Result<()> {
        let mut circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        // Upsert: replace if exists, push if new
        if let Some(pos) = circles.iter().position(|c| c.id == circle.id) {
            circles[pos] = circle.clone();
        } else {
            circles.push(circle.clone());
        }
        Ok(())
    }

    pub async fn get_circle(&self, id: &str) -> nexcore_error::Result<Option<CircleRecord>> {
        let circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(circles.iter().find(|c| c.id == id).cloned())
    }

    pub async fn get_circle_by_slug(
        &self,
        slug: &str,
    ) -> nexcore_error::Result<Option<CircleRecord>> {
        let circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(circles.iter().find(|c| c.slug == slug).cloned())
    }

    pub async fn list_circles(&self) -> nexcore_error::Result<Vec<CircleRecord>> {
        let circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(circles.clone())
    }

    pub async fn list_circles_by_tenant(
        &self,
        tenant_id: &str,
    ) -> nexcore_error::Result<Vec<CircleRecord>> {
        let circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(circles
            .iter()
            .filter(|c| {
                c.formation == CircleFormation::OrgBacked
                    && c.tenant_id.as_deref() == Some(tenant_id)
            })
            .cloned()
            .collect())
    }

    pub async fn delete_circle(&self, id: &str) -> nexcore_error::Result<()> {
        let mut circles = self
            .circles
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        circles.retain(|c| c.id != id);
        Ok(())
    }

    // ========================================================================
    // Circle Members (Mock)
    // ========================================================================

    pub async fn save_circle_member(
        &self,
        member: &CircleMemberRecord,
    ) -> nexcore_error::Result<()> {
        let mut members = self
            .circle_members
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        if let Some(pos) = members.iter().position(|m| m.id == member.id) {
            members[pos] = member.clone();
        } else {
            members.push(member.clone());
        }
        Ok(())
    }

    pub async fn get_circle_member(
        &self,
        circle_id: &str,
        user_id: &str,
    ) -> nexcore_error::Result<Option<CircleMemberRecord>> {
        let members = self
            .circle_members
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(members
            .iter()
            .find(|m| m.circle_id == circle_id && m.user_id == user_id)
            .cloned())
    }

    pub async fn list_circle_members(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<CircleMemberRecord>> {
        let members = self
            .circle_members
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(members
            .iter()
            .filter(|m| m.circle_id == circle_id)
            .cloned()
            .collect())
    }

    pub async fn update_circle_member(
        &self,
        member: &CircleMemberRecord,
    ) -> nexcore_error::Result<()> {
        self.save_circle_member(member).await
    }

    pub async fn delete_circle_member(
        &self,
        circle_id: &str,
        user_id: &str,
    ) -> nexcore_error::Result<()> {
        let mut members = self
            .circle_members
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        members.retain(|m| !(m.circle_id == circle_id && m.user_id == user_id));
        Ok(())
    }

    // ========================================================================
    // Circle Feed (Mock)
    // ========================================================================

    pub async fn save_feed_entry(&self, entry: &FeedEntryRecord) -> nexcore_error::Result<()> {
        let mut entries = self
            .feed_entries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        entries.push(entry.clone());
        Ok(())
    }

    pub async fn list_feed_entries(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<FeedEntryRecord>> {
        let entries = self
            .feed_entries
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(entries
            .iter()
            .filter(|e| e.circle_id == circle_id)
            .cloned()
            .collect())
    }

    // ========================================================================
    // Projects (Mock)
    // ========================================================================

    pub async fn save_project(&self, project: &ProjectRecord) -> nexcore_error::Result<()> {
        let mut projects = self
            .projects
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        if let Some(pos) = projects.iter().position(|p| p.id == project.id) {
            projects[pos] = project.clone();
        } else {
            projects.push(project.clone());
        }
        Ok(())
    }

    pub async fn get_project(&self, id: &str) -> nexcore_error::Result<Option<ProjectRecord>> {
        let projects = self
            .projects
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(projects.iter().find(|p| p.id == id).cloned())
    }

    pub async fn list_projects(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<ProjectRecord>> {
        let projects = self
            .projects
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(projects
            .iter()
            .filter(|p| p.circle_id == circle_id)
            .cloned()
            .collect())
    }

    pub async fn delete_project(&self, id: &str) -> nexcore_error::Result<()> {
        let mut projects = self
            .projects
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        projects.retain(|p| p.id != id);
        Ok(())
    }

    // ========================================================================
    // Deliverables (Mock)
    // ========================================================================

    pub async fn save_deliverable(
        &self,
        deliverable: &DeliverableRecord,
    ) -> nexcore_error::Result<()> {
        let mut deliverables = self
            .deliverables
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        if let Some(pos) = deliverables.iter().position(|d| d.id == deliverable.id) {
            deliverables[pos] = deliverable.clone();
        } else {
            deliverables.push(deliverable.clone());
        }
        Ok(())
    }

    pub async fn get_deliverable(
        &self,
        id: &str,
    ) -> nexcore_error::Result<Option<DeliverableRecord>> {
        let deliverables = self
            .deliverables
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(deliverables.iter().find(|d| d.id == id).cloned())
    }

    pub async fn list_deliverables(
        &self,
        project_id: &str,
    ) -> nexcore_error::Result<Vec<DeliverableRecord>> {
        let deliverables = self
            .deliverables
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(deliverables
            .iter()
            .filter(|d| d.project_id == project_id)
            .cloned()
            .collect())
    }

    pub async fn delete_deliverable(&self, id: &str) -> nexcore_error::Result<()> {
        let mut deliverables = self
            .deliverables
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        deliverables.retain(|d| d.id != id);
        Ok(())
    }

    // ========================================================================
    // Publications (Mock)
    // ========================================================================

    pub async fn save_publication(
        &self,
        pub_record: &PublicationRecord,
    ) -> nexcore_error::Result<()> {
        let mut pubs = self
            .publications
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        if let Some(pos) = pubs.iter().position(|p| p.id == pub_record.id) {
            pubs[pos] = pub_record.clone();
        } else {
            pubs.push(pub_record.clone());
        }
        Ok(())
    }

    pub async fn list_publications(&self) -> nexcore_error::Result<Vec<PublicationRecord>> {
        let pubs = self
            .publications
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(pubs.clone())
    }

    pub async fn list_circle_publications(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<PublicationRecord>> {
        let pubs = self
            .publications
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(pubs
            .iter()
            .filter(|p| p.source_circle_id == circle_id)
            .cloned()
            .collect())
    }

    // ========================================================================
    // Collaboration Requests (Mock)
    // ========================================================================

    pub async fn save_collaboration(
        &self,
        collab: &CollaborationRequestRecord,
    ) -> nexcore_error::Result<()> {
        let mut collabs = self
            .collaborations
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        if let Some(pos) = collabs.iter().position(|c| c.id == collab.id) {
            collabs[pos] = collab.clone();
        } else {
            collabs.push(collab.clone());
        }
        Ok(())
    }

    pub async fn get_collaboration(
        &self,
        id: &str,
    ) -> nexcore_error::Result<Option<CollaborationRequestRecord>> {
        let collabs = self
            .collaborations
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(collabs.iter().find(|c| c.id == id).cloned())
    }

    pub async fn list_collaborations_for_circle(
        &self,
        circle_id: &str,
    ) -> nexcore_error::Result<Vec<CollaborationRequestRecord>> {
        let collabs = self
            .collaborations
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(collabs
            .iter()
            .filter(|c| c.requesting_circle_id == circle_id || c.target_circle_id == circle_id)
            .cloned()
            .collect())
    }

    // ========================================================================
    // Legacy Memberships (Mock)
    // ========================================================================

    pub async fn save_membership(
        &self,
        membership: &MembershipRecord,
    ) -> nexcore_error::Result<()> {
        let mut memberships = self
            .memberships
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        memberships.push(membership.clone());
        Ok(())
    }

    pub async fn list_memberships(
        &self,
        user_id: &str,
    ) -> nexcore_error::Result<Vec<MembershipRecord>> {
        let memberships = self
            .memberships
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(memberships
            .iter()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    pub async fn save_message(&self, message: &MessageRecord) -> nexcore_error::Result<()> {
        let mut messages = self
            .messages
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        messages.push(message.clone());
        Ok(())
    }

    pub async fn list_messages(&self, user_id: &str) -> nexcore_error::Result<Vec<MessageRecord>> {
        let messages = self
            .messages
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(messages
            .iter()
            .filter(|m| m.sender_id == user_id || m.recipient_id == user_id)
            .cloned()
            .collect())
    }

    pub async fn save_ksb_domain(&self, domain: &KsbDomainRecord) -> nexcore_error::Result<()> {
        let mut domains = self
            .ksb_domains
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        domains.push(domain.clone());
        Ok(())
    }

    pub async fn list_ksb_domains(&self) -> nexcore_error::Result<Vec<KsbDomainRecord>> {
        let domains = self
            .ksb_domains
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(domains.clone())
    }

    pub async fn save_telemetry_event(
        &self,
        event: &TelemetryEventRecord,
    ) -> nexcore_error::Result<()> {
        let mut events = self
            .telemetry_events
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        events.push(event.clone());
        Ok(())
    }

    pub async fn list_telemetry_events(&self) -> nexcore_error::Result<Vec<TelemetryEventRecord>> {
        let events = self
            .telemetry_events
            .lock()
            .map_err(|_| nexcore_error::nexerror!("Lock poisoned"))?;
        Ok(events.clone())
    }
}

impl Default for MockPersistence {
    fn default() -> Self {
        Self::new()
    }
}
