use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    api::{auth::RequireRunAuth, run_error::RunApiError},
    db::Database,
    domain::{
        execution_policy::DroidExecutionPolicyOverrides,
        run::{NewRunRecord, RunBudgetLimits, RunEvent, RunPermissionSummary, RunSnapshot},
    },
    run::coordinator::RunEventPage,
    run::{
        coordinator::RunCoordinator,
        dispatcher::RunDispatcher,
        worker::{ReviewPolicySnapshot, RunPolicySnapshot, RunWorker},
    },
};

static RUN_ID_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct RunReadApiState {
    coordinator: RunCoordinator,
    dispatcher: RunDispatcher,
    db: Database,
    worker: RunWorker,
}

impl RunReadApiState {
    pub fn new(db: Database) -> Self {
        let dispatcher = RunDispatcher::default();
        let worker = RunWorker::new(db.clone());
        Self {
            coordinator: RunCoordinator::new(db.clone()),
            dispatcher,
            db,
            worker,
        }
    }

    pub fn with_dispatcher(db: Database, dispatcher: RunDispatcher) -> Self {
        let worker = RunWorker::new(db.clone());
        Self::with_worker(db, dispatcher, worker)
    }

    pub fn with_worker(db: Database, dispatcher: RunDispatcher, worker: RunWorker) -> Self {
        Self {
            coordinator: RunCoordinator::new(db.clone()),
            dispatcher,
            db,
            worker,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRunRequest {
    pub target_droid_id: String,
    #[serde(default)]
    pub reviewer_droid_id: Option<String>,
    pub task_title: String,
    pub prompt: String,
    pub verification_preset_id: String,
    #[serde(default)]
    pub budget_override: Option<RunBudgetLimits>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRunResponse {
    pub run: RunSnapshot,
    pub worker_enabled: bool,
}

/// POST /api/v1/projects/{project_id}/runs
pub async fn create_project_run(
    _auth: RequireRunAuth,
    State(state): State<RunReadApiState>,
    Path(project_id): Path<String>,
    headers: HeaderMap,
    body: Result<Json<CreateRunRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<CreateRunResponse>), RunApiError> {
    if !state.worker.enabled() {
        return Err(RunApiError::ExecutionDisabled);
    }
    let Json(body) = body.map_err(|_| RunApiError::InvalidRequest)?;
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty() && value.len() <= 128)
        .ok_or(RunApiError::InvalidRunRequest)?;
    if body.task_title.trim().is_empty()
        || body.task_title.len() > 512
        || body.prompt.trim().is_empty()
        || body.prompt.len() > 32 * 1024
        || body.target_droid_id.trim().is_empty()
    {
        return Err(RunApiError::InvalidRunRequest);
    }
    if !RunWorker::supports_verification_preset(&body.verification_preset_id) {
        return Err(RunApiError::UnsupportedVerificationPreset);
    }

    let projects = state.db.list_projects().map_err(|_| RunApiError::Storage)?;
    let project = projects
        .iter()
        .find(|project| project.id == project_id)
        .ok_or(RunApiError::ProjectNotFound)?;
    if !project
        .droids
        .iter()
        .any(|droid| droid.id == body.target_droid_id)
    {
        return Err(RunApiError::DroidNotFound);
    }
    let reviewer_droid_id = if let Some(reviewer_id) = body.reviewer_droid_id.as_deref() {
        if reviewer_id == body.target_droid_id {
            return Err(RunApiError::InvalidRunRequest);
        }
        if !project.droids.iter().any(|droid| droid.id == reviewer_id) {
            return Err(RunApiError::DroidNotFound);
        }
        reviewer_id.to_owned()
    } else {
        project
            .droids
            .iter()
            .find(|droid| {
                droid.id != body.target_droid_id
                    && droid.role.eq_ignore_ascii_case("REVISOR_DE_CODIGO")
            })
            .map(|droid| droid.id.clone())
            .ok_or(RunApiError::ReviewerDroidRequired)?
    };

    let project_policy = state
        .db
        .get_project_execution_policy(&project_id)?
        .ok_or(RunApiError::ExecutionPolicyRequired)?;
    let droid_overrides = state
        .db
        .get_droid_execution_policy_overrides(&project_id, &body.target_droid_id)?
        .unwrap_or_else(DroidExecutionPolicyOverrides::default);
    let effective_policy = project_policy
        .effective_for(&droid_overrides)
        .map_err(RunApiError::InvalidExecutionPolicy)?;
    let mut reference_project_roots = Vec::new();
    for reference_id in &effective_policy.reference_project_ids {
        let path = state
            .db
            .project_root(reference_id)
            .map_err(|_| RunApiError::Storage)?
            .ok_or(RunApiError::ProjectNotFound)?;
        reference_project_roots.push(PathBuf::from(path));
    }
    let reviewer_overrides = state
        .db
        .get_droid_execution_policy_overrides(&project_id, &reviewer_droid_id)?
        .unwrap_or_else(DroidExecutionPolicyOverrides::default);
    let mut reviewer_policy = project_policy
        .effective_for(&reviewer_overrides)
        .map_err(RunApiError::InvalidExecutionPolicy)?;
    if !reviewer_policy.read_project {
        return Err(RunApiError::ReviewerReadAccessRequired);
    }
    reviewer_policy.write_roots.clear();
    reviewer_policy.web_search = false;
    reviewer_policy.shell_network = false;
    let mut reviewer_reference_roots = Vec::new();
    for reference_id in &reviewer_policy.reference_project_ids {
        let path = state
            .db
            .project_root(reference_id)
            .map_err(|_| RunApiError::Storage)?
            .ok_or(RunApiError::ProjectNotFound)?;
        reviewer_reference_roots.push(PathBuf::from(path));
    }
    let policy_snapshot_json = serde_json::to_string(&RunPolicySnapshot {
        effective_policy,
        reference_project_roots,
        reviewer: Some(ReviewPolicySnapshot {
            effective_policy: reviewer_policy,
            reference_project_roots: reviewer_reference_roots,
        }),
    })
    .map_err(|_| RunApiError::Storage)?;
    let budgets = state
        .db
        .get_project_execution_settings(&project_id)?
        .ok_or(RunApiError::ExecutionBudgetsRequired)?
        .budgets
        .tighten_with(body.budget_override)
        .map_err(RunApiError::InvalidBudget)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = RUN_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let record = NewRunRecord {
        run_id: format!("run-{now}-{sequence}"),
        project_id,
        droid_ids: vec![body.target_droid_id, reviewer_droid_id],
        idempotency_key: idempotency_key.to_owned(),
        task_title: body.task_title,
        prompt: body.prompt,
        verification_preset_id: body.verification_preset_id,
        policy_snapshot_json,
        budget_limits: budgets,
    };
    let run = state.worker.submit(&record)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(CreateRunResponse {
            run,
            worker_enabled: true,
        }),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunEventsQuery {
    #[serde(default)]
    pub after_sequence: u64,
    #[serde(default = "default_event_limit")]
    pub limit: usize,
}

fn default_event_limit() -> usize {
    100
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEventsResponse {
    pub run_id: String,
    pub events: Vec<RunEvent>,
    pub next_sequence: u64,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PermissionDecisionRequest {
    pub decision: PermissionDecisionChoice,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionDecisionChoice {
    Grant,
    Deny,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDecisionResponse {
    pub permission: RunPermissionSummary,
    pub response_sent: bool,
    pub run: RunSnapshot,
}

impl From<(String, RunEventPage)> for RunEventsResponse {
    fn from((run_id, page): (String, RunEventPage)) -> Self {
        Self {
            run_id,
            events: page.events,
            next_sequence: page.next_sequence,
            has_more: page.has_more,
        }
    }
}

/// GET /api/v1/runs/{run_id}
pub async fn get_run_snapshot(
    _auth: RequireRunAuth,
    State(state): State<RunReadApiState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunSnapshot>, RunApiError> {
    Ok(Json(state.coordinator.snapshot(&run_id)?))
}

/// GET /api/v1/runs/{run_id}/events?afterSequence=<n>&limit=<n>
pub async fn get_run_events(
    _auth: RequireRunAuth,
    State(state): State<RunReadApiState>,
    Path(run_id): Path<String>,
    query: Result<Query<RunEventsQuery>, QueryRejection>,
) -> Result<Json<RunEventsResponse>, RunApiError> {
    let Query(query) = query.map_err(|_| RunApiError::InvalidRunRequest)?;
    if query.limit == 0 || query.limit > 500 || query.after_sequence > i64::MAX as u64 {
        return Err(RunApiError::InvalidRunRequest);
    }
    let page = state
        .coordinator
        .replay_events(&run_id, query.after_sequence, query.limit)?;
    Ok(Json((run_id, page).into()))
}

/// POST /api/v1/runs/{run_id}/permission-requests/{request_id}/decisions
pub async fn decide_run_permission(
    _auth: RequireRunAuth,
    State(state): State<RunReadApiState>,
    Path((run_id, request_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: Result<Json<PermissionDecisionRequest>, JsonRejection>,
) -> Result<Json<PermissionDecisionResponse>, RunApiError> {
    let Json(body) = body.map_err(|_| RunApiError::InvalidRunRequest)?;
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty() && value.len() <= 128)
        .ok_or(RunApiError::InvalidRunRequest)?;
    state.coordinator.snapshot(&run_id)?;
    let stored = state
        .coordinator
        .permission_request(&request_id)?
        .filter(|permission| permission.run_id == run_id)
        .ok_or(RunApiError::PermissionNotFound)?;
    let granted = matches!(body.decision, PermissionDecisionChoice::Grant);
    let grant_scope = granted.then(|| {
        format!(
            "{}:{}:{}",
            stored.capability, stored.action, stored.resource
        )
    });
    let (mut permission, mut run) = state.coordinator.decide_permission(
        &request_id,
        granted,
        idempotency_key,
        grant_scope.as_deref(),
    )?;
    if !permission.response_sent
        && state
            .dispatcher
            .deliver_decision(&run_id, &request_id, granted)
            .await
    {
        permission = state
            .coordinator
            .mark_permission_response_sent(&request_id)?;
        run = state.coordinator.snapshot(&run_id)?;
    }

    Ok(Json(PermissionDecisionResponse {
        permission: RunPermissionSummary {
            request_id: permission.request_id,
            step_id: permission.step_id,
            droid_id: permission.droid_id,
            capability: permission.capability,
            action: permission.action,
            resource: permission.resource,
            rationale: permission.rationale,
            status: permission.status,
            alternative_index: permission.alternative_index,
            created_at: permission.created_at,
            payload_fingerprint: permission.payload_fingerprint,
        },
        response_sent: permission.response_sent,
        run,
    }))
}
