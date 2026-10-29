use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};
use serde::{Deserialize, Serialize};

use crate::{
    api::{auth::RequireRunAuth, run_error::RunApiError},
    db::Database,
    domain::run::{ProjectExecutionSettings, RunBudgetLimits},
};

#[derive(Clone)]
pub struct ExecutionSettingsApiState {
    pub db: Database,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateExecutionSettingsRequest {
    pub budgets: RunBudgetLimits,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSettingsResponse {
    pub configured: bool,
    pub budgets: Option<RunBudgetLimits>,
}

impl From<Option<ProjectExecutionSettings>> for ExecutionSettingsResponse {
    fn from(settings: Option<ProjectExecutionSettings>) -> Self {
        match settings {
            Some(settings) => Self {
                configured: true,
                budgets: Some(settings.budgets),
            },
            None => Self {
                configured: false,
                budgets: None,
            },
        }
    }
}

/// GET /api/v1/projects/{id}/execution-settings
pub async fn get_execution_settings(
    _auth: RequireRunAuth,
    State(state): State<ExecutionSettingsApiState>,
    Path(project_id): Path<String>,
) -> Result<Json<ExecutionSettingsResponse>, RunApiError> {
    let settings = state.db.get_project_execution_settings(&project_id)?;
    Ok(Json(settings.into()))
}

/// PUT /api/v1/projects/{id}/execution-settings
pub async fn put_execution_settings(
    _auth: RequireRunAuth,
    State(state): State<ExecutionSettingsApiState>,
    Path(project_id): Path<String>,
    request: Result<Json<UpdateExecutionSettingsRequest>, JsonRejection>,
) -> Result<Json<ExecutionSettingsResponse>, RunApiError> {
    let Json(request) = request.map_err(|_| RunApiError::InvalidRequest)?;
    let settings = state
        .db
        .save_project_execution_settings(&project_id, request.budgets)?;
    Ok(Json(Some(settings).into()))
}
