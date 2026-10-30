use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};
use serde::Serialize;

use crate::{
    api::{auth::RequireRunAuth, run_error::RunApiError},
    db::Database,
    domain::execution_policy::{DroidExecutionPolicyOverrides, ProjectExecutionPolicy},
};

#[derive(Clone)]
pub struct ExecutionPolicyApiState {
    pub db: Database,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExecutionPolicyResponse {
    pub configured: bool,
    pub policy: Option<ProjectExecutionPolicy>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DroidExecutionPolicyResponse {
    pub configured: bool,
    pub inherits_project_policy: bool,
    pub overrides: DroidExecutionPolicyOverrides,
}

impl From<Option<ProjectExecutionPolicy>> for ProjectExecutionPolicyResponse {
    fn from(policy: Option<ProjectExecutionPolicy>) -> Self {
        Self {
            configured: policy.is_some(),
            policy,
        }
    }
}

impl From<Option<DroidExecutionPolicyOverrides>> for DroidExecutionPolicyResponse {
    fn from(overrides: Option<DroidExecutionPolicyOverrides>) -> Self {
        let inherits_project_policy = overrides
            .as_ref()
            .is_none_or(DroidExecutionPolicyOverrides::inherits_all);
        Self {
            configured: overrides.is_some(),
            inherits_project_policy,
            overrides: overrides.unwrap_or_default(),
        }
    }
}

/// GET /api/v1/projects/{project_id}/execution-policy
pub async fn get_project_execution_policy(
    _auth: RequireRunAuth,
    State(state): State<ExecutionPolicyApiState>,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectExecutionPolicyResponse>, RunApiError> {
    Ok(Json(
        state.db.get_project_execution_policy(&project_id)?.into(),
    ))
}

/// PUT /api/v1/projects/{project_id}/execution-policy
pub async fn put_project_execution_policy(
    _auth: RequireRunAuth,
    State(state): State<ExecutionPolicyApiState>,
    Path(project_id): Path<String>,
    request: Result<Json<ProjectExecutionPolicy>, JsonRejection>,
) -> Result<Json<ProjectExecutionPolicyResponse>, RunApiError> {
    let Json(policy) = request.map_err(|_| RunApiError::InvalidRequest)?;
    Ok(Json(
        Some(
            state
                .db
                .save_project_execution_policy(&project_id, &policy)?,
        )
        .into(),
    ))
}

/// GET /api/v1/projects/{project_id}/droids/{droid_id}/execution-policy
pub async fn get_droid_execution_policy(
    _auth: RequireRunAuth,
    State(state): State<ExecutionPolicyApiState>,
    Path((project_id, droid_id)): Path<(String, String)>,
) -> Result<Json<DroidExecutionPolicyResponse>, RunApiError> {
    Ok(Json(
        state
            .db
            .get_droid_execution_policy_overrides(&project_id, &droid_id)?
            .into(),
    ))
}

/// PUT /api/v1/projects/{project_id}/droids/{droid_id}/execution-policy
pub async fn put_droid_execution_policy(
    _auth: RequireRunAuth,
    State(state): State<ExecutionPolicyApiState>,
    Path((project_id, droid_id)): Path<(String, String)>,
    request: Result<Json<DroidExecutionPolicyOverrides>, JsonRejection>,
) -> Result<Json<DroidExecutionPolicyResponse>, RunApiError> {
    let Json(overrides) = request.map_err(|_| RunApiError::InvalidRequest)?;
    Ok(Json(
        Some(
            state
                .db
                .save_droid_execution_policy_overrides(&project_id, &droid_id, &overrides)?,
        )
        .into(),
    ))
}
