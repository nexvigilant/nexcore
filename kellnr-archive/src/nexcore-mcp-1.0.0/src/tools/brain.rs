//! Brain MCP tools - Antigravity-style working memory for Claude Code
//!
//! Provides session management, artifact versioning, code tracking, implicit knowledge,
//! and health metrics.

use nexcore_brain::{
    AgentId, Artifact, ArtifactType, Belief, BrainSession, CodeTracker, CoordinationRegistry,
    EvidenceRef, EvidenceType, ImplicationStrength, ImplicitKnowledge, LockDuration, T1Primitive,
    attempt_recovery, check_brain_availability, check_index_health, detect_partial_writes,
    rebuild_index_from_sessions, repair_partial_writes,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use std::path::{Path, PathBuf};

use crate::params::{
    BrainArtifactDiffParams, BrainArtifactGetParams, BrainArtifactResolveParams,
    BrainArtifactSaveParams, BrainCodeTrackerChangedParams, BrainCodeTrackerOriginalParams,
    BrainCodeTrackerTrackParams, BrainCoordinationAcquireParams, BrainCoordinationReleaseParams,
    BrainCoordinationStatusParams, BrainImplicitGetParams, BrainImplicitSetParams,
    BrainSessionCreateParams, BrainSessionLoadParams, BrainSessionsListParams,
};

// ============================================================================
// Coordination Tools (ACS)
// ============================================================================

/// Acquire a file lock for agent coordination
pub fn coordination_acquire(
    params: BrainCoordinationAcquireParams,
) -> Result<CallToolResult, McpError> {
    let mut registry =
        CoordinationRegistry::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let path = Path::new(&params.path);
    let agent_id = AgentId(params.agent_id);
    let ttl = LockDuration(params.ttl);

    let success = registry
        .acquire_lock(path, agent_id.clone(), ttl)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if success {
        let _ = nexcore_brain::coordination::log_access(&agent_id, path, "acquire_success");
    } else {
        let _ = nexcore_brain::coordination::log_access(&agent_id, path, "acquire_failed_occupied");
    }

    let result = serde_json::json!({
        "success": success,
        "path": params.path,
        "agent_id": agent_id.0,
        "status": if success { "Occupied" } else { "Conflict" }
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Release a file lock
pub fn coordination_release(
    params: BrainCoordinationReleaseParams,
) -> Result<CallToolResult, McpError> {
    let mut registry =
        CoordinationRegistry::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let path = Path::new(&params.path);
    let agent_id = AgentId(params.agent_id);

    let success = registry
        .release_lock(path, &agent_id)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if success {
        let _ = nexcore_brain::coordination::log_access(&agent_id, path, "release_success");
    }

    let result = serde_json::json!({
        "success": success,
        "path": params.path,
        "agent_id": agent_id.0,
        "status": "Vacant"
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Check lock status of a file
pub fn coordination_status(
    params: BrainCoordinationStatusParams,
) -> Result<CallToolResult, McpError> {
    let registry =
        CoordinationRegistry::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let path = Path::new(&params.path);
    let status = registry
        .check_status(path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "path": params.path,
        "status": format!("{:?}", status)
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Session Tools
// ============================================================================

/// Create a new brain session
pub fn session_create(params: BrainSessionCreateParams) -> Result<CallToolResult, McpError> {
    let session =
        BrainSession::create_with_options(params.project, params.git_commit, params.description)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "session_id": session.id.to_string(),
        "created_at": session.created_at.to_rfc3339(),
        "project": session.project,
        "git_commit": session.git_commit,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Load an existing brain session
pub fn session_load(params: BrainSessionLoadParams) -> Result<CallToolResult, McpError> {
    let session = BrainSession::load_str(&params.session_id)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let artifacts = session.list_artifacts().unwrap_or_default();

    let result = serde_json::json!({
        "session_id": session.id.to_string(),
        "created_at": session.created_at.to_rfc3339(),
        "project": session.project,
        "git_commit": session.git_commit,
        "session_dir": session.dir().to_string_lossy(),
        "artifacts": artifacts,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// List all brain sessions
pub fn sessions_list(params: BrainSessionsListParams) -> Result<CallToolResult, McpError> {
    let mut sessions =
        BrainSession::list_all().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Sort by created_at descending (most recent first)
    sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Apply limit
    let limit = params.limit.unwrap_or(20) as usize;
    sessions.truncate(limit);

    let result: Vec<serde_json::Value> = sessions
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "session_id": s.id.to_string(),
                "created_at": s.created_at.to_rfc3339(),
                "project": s.project,
                "description": s.description,
            })
        })
        .collect();

    let count = result.len();
    let json_output = serde_json::json!({
        "sessions": result,
        "count": count
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json_output.to_string(),
    )]))
}

// ============================================================================
// Artifact Tools
// ============================================================================

/// Save an artifact to a session
pub fn artifact_save(params: BrainArtifactSaveParams) -> Result<CallToolResult, McpError> {
    let session = get_session(params.session_id.as_deref())?;

    let artifact_type = params
        .artifact_type
        .map(|t| t.parse::<ArtifactType>())
        .transpose()
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?
        .unwrap_or_else(|| ArtifactType::from_filename(&params.name));

    let artifact = Artifact::new(&params.name, artifact_type, &params.content);
    session
        .save_artifact(&artifact)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "status": "saved",
        "name": params.name,
        "artifact_type": artifact_type.to_string(),
        "session_id": session.id.to_string(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Resolve an artifact (create immutable snapshot)
pub fn artifact_resolve(params: BrainArtifactResolveParams) -> Result<CallToolResult, McpError> {
    let session = get_session(params.session_id.as_deref())?;

    let version = session
        .resolve_artifact(&params.name)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "status": "resolved",
        "name": params.name,
        "version": version,
        "session_id": session.id.to_string(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get an artifact (current or specific version)
pub fn artifact_get(params: BrainArtifactGetParams) -> Result<CallToolResult, McpError> {
    let session = get_session(params.session_id.as_deref())?;

    let artifact = session
        .get_artifact(&params.name, params.version)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "name": artifact.name,
        "artifact_type": artifact.artifact_type.to_string(),
        "version": params.version.unwrap_or(0),
        "content": artifact.content,
        "updated_at": artifact.updated_at.to_rfc3339(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Diff two versions of an artifact
pub fn artifact_diff(params: BrainArtifactDiffParams) -> Result<CallToolResult, McpError> {
    let session = get_session(params.session_id.as_deref())?;

    let diff = session
        .diff_versions(&params.name, params.v1, params.v2)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "name": params.name,
        "v1": params.v1,
        "v2": params.v2,
        "diff": diff,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Code Tracker Tools
// ============================================================================

/// Track a file for change detection
pub fn code_tracker_track(params: BrainCodeTrackerTrackParams) -> Result<CallToolResult, McpError> {
    let path = PathBuf::from(&params.path);
    let project = params.project.unwrap_or_else(get_default_project);

    let mut tracker = CodeTracker::new(&project, None)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let tracked = tracker
        .track_file(&path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "status": "tracked",
        "path": params.path,
        "content_hash": tracked.content_hash,
        "size": tracked.size,
        "tracked_at": tracked.tracked_at.to_rfc3339(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Check if a tracked file has changed
pub fn code_tracker_changed(
    params: BrainCodeTrackerChangedParams,
) -> Result<CallToolResult, McpError> {
    let path = PathBuf::from(&params.path);
    let project = params.project.unwrap_or_else(get_default_project);

    let tracker =
        CodeTracker::load(&project).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let changed = tracker
        .has_changed(&path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "path": params.path,
        "changed": changed,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get original content of a tracked file
pub fn code_tracker_original(
    params: BrainCodeTrackerOriginalParams,
) -> Result<CallToolResult, McpError> {
    let path = PathBuf::from(&params.path);
    let project = params.project.unwrap_or_else(get_default_project);

    let tracker =
        CodeTracker::load(&project).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let content = tracker
        .get_original(&path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "path": params.path,
        "original_content": content,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Implicit Knowledge Tools
// ============================================================================

/// Get a preference from implicit knowledge
pub fn implicit_get(params: BrainImplicitGetParams) -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = if let Some(pref) = knowledge.get_preference(&params.key) {
        serde_json::json!({
            "key": pref.key,
            "value": pref.value,
            "confidence": pref.confidence,
            "reinforcement_count": pref.reinforcement_count,
            "updated_at": pref.updated_at.to_rfc3339(),
        })
    } else {
        serde_json::json!({
            "key": params.key,
            "found": false,
        })
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Set a preference in implicit knowledge
pub fn implicit_set(params: BrainImplicitSetParams) -> Result<CallToolResult, McpError> {
    let mut knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Parse value as JSON
    let value: serde_json::Value = serde_json::from_str(&params.value)
        .map_err(|e| McpError::invalid_params(format!("Invalid JSON value: {e}"), None))?;

    knowledge.set_preference_value(&params.key, value.clone());
    knowledge
        .save()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "status": "set",
        "key": params.key,
        "value": value,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Implicit Knowledge - Extended Tools (v2)
// ============================================================================

/// Get implicit knowledge statistics including decay and grounding metrics
pub fn implicit_stats() -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let stats = knowledge.stats();
    let result = serde_json::json!({
        "total_preferences": stats.total_preferences,
        "total_patterns": stats.total_patterns,
        "total_corrections": stats.total_corrections,
        "high_confidence_preferences": stats.high_confidence_preferences,
        "high_confidence_patterns": stats.high_confidence_patterns,
        "decayed_patterns": stats.decayed_patterns,
        "ungrounded_patterns": stats.ungrounded_patterns,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Find corrections using fuzzy token matching (Jaccard similarity)
pub fn implicit_find_corrections(
    params: crate::params::BrainImplicitFindCorrectionsParams,
) -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let threshold = params.threshold.unwrap_or(0.3);
    let corrections = knowledge.find_corrections_fuzzy(&params.query, threshold);

    let results: Vec<serde_json::Value> = corrections
        .iter()
        .map(|c| {
            serde_json::json!({
                "mistake": c.mistake,
                "correction": c.correction,
                "context": c.context,
                "application_count": c.application_count,
            })
        })
        .collect();

    let result = serde_json::json!({
        "query": params.query,
        "threshold": threshold,
        "matches": results,
        "count": results.len(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// List patterns filtered by T1 primitive grounding
pub fn implicit_patterns_by_grounding(
    params: crate::params::BrainImplicitPatternsByGroundingParams,
) -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let primitive: T1Primitive = serde_json::from_value(serde_json::Value::String(
        params.primitive.clone(),
    ))
    .map_err(|_| {
        McpError::invalid_params(
            format!(
                "Invalid primitive '{}'. Use: sequence, mapping, recursion, state, void",
                params.primitive
            ),
            None,
        )
    })?;

    let patterns = knowledge.list_patterns_by_grounding(primitive);

    let results: Vec<serde_json::Value> = patterns
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "pattern_type": p.pattern_type,
                "description": p.description,
                "confidence": p.confidence,
                "effective_confidence": p.effective_confidence(),
                "occurrence_count": p.occurrence_count,
                "t1_grounding": p.t1_grounding,
            })
        })
        .collect();

    let result = serde_json::json!({
        "primitive": params.primitive,
        "patterns": results,
        "count": results.len(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// List all patterns sorted by effective confidence (decay-adjusted)
pub fn implicit_patterns_by_relevance() -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let scored = knowledge.list_patterns_by_relevance();

    let results: Vec<serde_json::Value> = scored
        .iter()
        .map(|(p, eff)| {
            serde_json::json!({
                "id": p.id,
                "pattern_type": p.pattern_type,
                "description": p.description,
                "raw_confidence": p.confidence,
                "effective_confidence": eff,
                "t1_grounding": p.t1_grounding,
                "occurrence_count": p.occurrence_count,
            })
        })
        .collect();

    let result = serde_json::json!({
        "patterns": results,
        "count": results.len(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Belief Tools (PROJECT GROUNDED)
// ============================================================================

/// Create a new belief or hypothesis
pub fn belief_save(
    params: crate::params::BrainBeliefSaveParams,
) -> Result<CallToolResult, McpError> {
    let mut knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let belief = if params.is_hypothesis.unwrap_or(false) {
        Belief::from_hypothesis(&params.id, &params.proposition, &params.category)
    } else {
        Belief::new(&params.id, &params.proposition, &params.category)
    };

    knowledge.add_belief(belief);
    knowledge
        .save()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "status": "saved",
        "id": params.id,
        "proposition": params.proposition,
        "category": params.category,
        "is_hypothesis": params.is_hypothesis.unwrap_or(false),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get a belief by ID
pub fn belief_get(params: crate::params::BrainBeliefGetParams) -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = if let Some(belief) = knowledge.get_belief(&params.id) {
        serde_json::json!({
            "id": belief.id,
            "proposition": belief.proposition,
            "category": belief.category,
            "confidence": belief.confidence,
            "effective_confidence": belief.effective_confidence(),
            "evidence_count": belief.evidence.len(),
            "validation_count": belief.validation_count,
            "user_confirmed": belief.user_confirmed,
            "t1_grounding": belief.t1_grounding,
            "formed_at": belief.formed_at.to_rfc3339(),
            "updated_at": belief.updated_at.to_rfc3339(),
        })
    } else {
        serde_json::json!({ "id": params.id, "found": false })
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// List all beliefs sorted by effective confidence
pub fn belief_list() -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let scored = knowledge.list_beliefs_by_confidence();
    let results: Vec<serde_json::Value> = scored
        .iter()
        .map(|(b, eff)| {
            serde_json::json!({
                "id": b.id,
                "proposition": b.proposition,
                "category": b.category,
                "effective_confidence": eff,
                "user_confirmed": b.user_confirmed,
            })
        })
        .collect();

    let result = serde_json::json!({ "beliefs": results, "count": results.len() });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Add evidence to a belief
pub fn belief_add_evidence(
    params: crate::params::BrainBeliefAddEvidenceParams,
) -> Result<CallToolResult, McpError> {
    let mut knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let evidence_type: EvidenceType =
        serde_json::from_value(serde_json::Value::String(params.evidence_type.clone()))
            .map_err(|_| McpError::invalid_params("Invalid evidence_type", None))?;

    let mut evidence = EvidenceRef::weighted(
        &params.evidence_id,
        evidence_type,
        &params.description,
        params.weight,
        &params.source,
    );

    if let Some(exec_id) = &params.execution_id {
        evidence = evidence.with_execution(exec_id);
    }
    if let Some(hyp_id) = &params.hypothesis_id {
        evidence = evidence.with_hypothesis(hyp_id);
    }

    let success = knowledge.add_evidence_to_belief(&params.belief_id, evidence);
    if success {
        knowledge
            .save()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    }

    let result = serde_json::json!({
        "belief_id": params.belief_id,
        "evidence_id": params.evidence_id,
        "added": success,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Record a validation attempt on a belief
pub fn belief_validate(
    params: crate::params::BrainBeliefValidateParams,
) -> Result<CallToolResult, McpError> {
    let mut knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let success = knowledge.validate_belief(&params.belief_id, params.passed);
    if success {
        knowledge
            .save()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    }

    let new_confidence = knowledge
        .get_belief(&params.belief_id)
        .map(|b| b.effective_confidence());

    let result = serde_json::json!({
        "belief_id": params.belief_id,
        "validation_result": params.passed,
        "recorded": success,
        "new_effective_confidence": new_confidence,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Trust Tools (PROJECT GROUNDED)
// ============================================================================

/// Record a trust demonstration (success or failure) in a domain
pub fn trust_record(
    params: crate::params::BrainTrustRecordParams,
) -> Result<CallToolResult, McpError> {
    let mut knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if params.success {
        knowledge.record_trust_success(&params.domain);
    } else {
        knowledge.record_trust_failure(&params.domain);
    }
    knowledge
        .save()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let trust = knowledge.get_trust(&params.domain);
    let result = serde_json::json!({
        "domain": params.domain,
        "recorded": params.success,
        "new_score": trust.map(|t| t.score()),
        "demonstrations": trust.map(|t| t.demonstrations),
        "failures": trust.map(|t| t.failures),
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get trust score for a domain
pub fn trust_get(params: crate::params::BrainTrustGetParams) -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = if let Some(trust) = knowledge.get_trust(&params.domain) {
        serde_json::json!({
            "domain": trust.domain,
            "score": trust.score(),
            "demonstrations": trust.demonstrations,
            "failures": trust.failures,
            "total_interactions": trust.total_interactions(),
            "created_at": trust.created_at.to_rfc3339(),
            "updated_at": trust.updated_at.to_rfc3339(),
        })
    } else {
        serde_json::json!({ "domain": params.domain, "found": false })
    };
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get global trust score across all domains
pub fn trust_global() -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let domains: Vec<serde_json::Value> = knowledge
        .list_trust()
        .iter()
        .map(|t| serde_json::json!({ "domain": t.domain, "score": t.score() }))
        .collect();

    let result = serde_json::json!({
        "global_score": knowledge.global_trust_score(),
        "domain_count": domains.len(),
        "domains": domains,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Belief Graph Tools (PROJECT GROUNDED)
// ============================================================================

/// Add an implication between beliefs (with cycle detection)
pub fn belief_graph_add_implication(
    params: crate::params::BrainBeliefGraphAddImplicationParams,
) -> Result<CallToolResult, McpError> {
    let mut knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let strength: ImplicationStrength =
        serde_json::from_value(serde_json::Value::String(params.strength.clone())).map_err(
            |_| McpError::invalid_params("Invalid strength (use: strong, moderate, weak)", None),
        )?;

    let added = knowledge.add_belief_implication(&params.from, &params.to, strength);

    if added {
        knowledge
            .save()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    }

    let result = serde_json::json!({
        "from": params.from,
        "to": params.to,
        "strength": params.strength,
        "added": added,
        "reason": if added { "success" } else { "would_create_cycle" },
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get beliefs implied by a given belief
pub fn belief_graph_implied_by(
    params: crate::params::BrainBeliefGraphQueryParams,
) -> Result<CallToolResult, McpError> {
    let knowledge =
        ImplicitKnowledge::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let implied: Vec<serde_json::Value> = knowledge
        .belief_graph()
        .implied_by(&params.belief_id)
        .iter()
        .map(|(id, strength)| serde_json::json!({ "belief_id": id, "strength": format!("{:?}", strength) }))
        .collect();

    let result = serde_json::json!({
        "source": params.belief_id,
        "implies": implied,
        "count": implied.len(),
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Recovery Tools
// ============================================================================

/// Check brain health status
pub fn recovery_check() -> Result<CallToolResult, McpError> {
    let availability = check_brain_availability();
    let index_health = check_index_health();

    // Check partial writes in latest session
    let partial_writes = BrainSession::load_latest()
        .ok()
        .map(|s| detect_partial_writes(&s.dir()))
        .unwrap_or_default();

    let status = if availability.is_none() {
        "healthy"
    } else {
        "degraded"
    };

    let result = serde_json::json!({
        "status": status,
        "availability": availability,
        "index_health": index_health,
        "partial_writes": partial_writes,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Repair partial writes in a session
pub fn recovery_repair(session_id: Option<&str>) -> Result<CallToolResult, McpError> {
    let session = get_session(session_id)?;

    let repair_result = repair_partial_writes(&session.dir())
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "success": repair_result.success,
        "recovered_count": repair_result.recovered_count,
        "failed_count": repair_result.failed_count,
        "details": repair_result.details,
        "warnings": repair_result.warnings,
        "session_id": session.id.to_string(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Rebuild index from session directories
pub fn recovery_rebuild_index() -> Result<CallToolResult, McpError> {
    let rebuild_result =
        rebuild_index_from_sessions().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "success": rebuild_result.success,
        "recovered_count": rebuild_result.recovered_count,
        "failed_count": rebuild_result.failed_count,
        "details": rebuild_result.details,
        "warnings": rebuild_result.warnings,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Attempt automatic recovery
pub fn recovery_auto() -> Result<CallToolResult, McpError> {
    let auto_result =
        attempt_recovery().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "success": auto_result.success,
        "recovered_count": auto_result.recovered_count,
        "failed_count": auto_result.failed_count,
        "details": auto_result.details,
        "warnings": auto_result.warnings,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Health Metrics Tools
// ============================================================================

/// Get comprehensive brain health report
pub fn health() -> Result<CallToolResult, McpError> {
    let health = nexcore_brain::BrainHealth::collect()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "collected_at": health.collected_at.to_rfc3339(),
        "status": health.status,
        "artifacts": {
            "total": health.artifacts.total,
            "by_type": health.artifacts.by_type,
        },
        "total_bytes": health.total_bytes,
        "total_bytes_human": format_bytes(health.total_bytes),
        "sessions": {
            "active": health.sessions.active,
            "total": health.sessions.total,
        },
        "implicit_entries": health.implicit_entries,
        "tracked_files": health.tracked_files,
        "brain_dir": health.brain_dir.to_string_lossy(),
        "telemetry_dir": health.telemetry_dir.to_string_lossy(),
        "warnings": health.warnings,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get growth rate statistics
pub fn growth_rate(
    params: crate::params::BrainGrowthRateParams,
) -> Result<CallToolResult, McpError> {
    let rate = nexcore_brain::GrowthRate::calculate(params.days)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "period_start": rate.period_start.to_rfc3339(),
        "period_end": rate.period_end.to_rfc3339(),
        "days_analyzed": rate.days_analyzed,
        "artifacts_per_day": rate.artifacts_per_day,
        "sessions_per_day": rate.sessions_per_day,
        "total_artifacts_created": rate.total_artifacts_created,
        "total_sessions_created": rate.total_sessions_created,
        "bytes_per_day": rate.bytes_per_day,
        "bytes_per_day_human": format_bytes(rate.bytes_per_day as u64),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get the N largest artifacts across all sessions
pub fn largest_artifacts(
    params: crate::params::BrainLargestArtifactsParams,
) -> Result<CallToolResult, McpError> {
    let artifacts = nexcore_brain::largest_artifacts(params.n)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let results: Vec<serde_json::Value> = artifacts
        .iter()
        .map(|a| {
            serde_json::json!({
                "session_id": a.session_id,
                "name": a.name,
                "artifact_type": a.artifact_type,
                "size_bytes": a.size_bytes,
                "size_human": format_bytes(a.size_bytes),
                "modified_at": a.modified_at.to_rfc3339(),
            })
        })
        .collect();

    let result = serde_json::json!({
        "artifacts": results,
        "count": results.len(),
        "requested": params.n,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Save a metrics snapshot to telemetry
pub fn snapshot() -> Result<CallToolResult, McpError> {
    let snapshot = nexcore_brain::BrainSnapshot::collect()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    nexcore_brain::save_snapshot(&snapshot)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "status": "saved",
        "timestamp": snapshot.timestamp.to_rfc3339(),
        "artifacts_total": snapshot.artifacts.total,
        "total_bytes": snapshot.total_bytes,
        "sessions_total": snapshot.sessions.total,
        "implicit_entries": snapshot.implicit_entries,
        "tracked_files": snapshot.tracked_files,
        "telemetry_dir": nexcore_brain::telemetry_dir().to_string_lossy().to_string(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Helpers
// ============================================================================

fn get_session(id: Option<&str>) -> Result<BrainSession, McpError> {
    match id {
        Some(id) => {
            BrainSession::load_str(id).map_err(|e| McpError::internal_error(e.to_string(), None))
        }
        None => {
            BrainSession::load_latest().map_err(|e| McpError::internal_error(e.to_string(), None))
        }
    }
}

fn get_default_project() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "default".to_string())
}

/// Format bytes into human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
