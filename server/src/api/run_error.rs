use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::db::sqlite::{ExecutionPolicyStoreError, ExecutionSettingsStoreError, RunStoreError};
use crate::domain::execution_policy::ExecutionPolicyError;
use crate::domain::run::BudgetConfigError;

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub enum RunApiError {
    AuthNotConfigured,
    Unauthorized,
    InvalidRequest,
    ProjectNotFound,
    DroidNotFound,
    RunNotFound,
    PermissionNotFound,
    InvalidBudget(BudgetConfigError),
    RunConflict,
    BudgetExhausted,
    ExecutionBudgetsRequired,
    ExecutionPolicyRequired,
    InvalidExecutionPolicy(ExecutionPolicyError),
    ExistingDroidPolicyConflict,
    InvalidRunRequest,
    ExecutionDisabled,
    UnsupportedVerificationPreset,
    ReviewerDroidRequired,
    ReviewerReadAccessRequired,
    Storage,
}

impl From<ExecutionSettingsStoreError> for RunApiError {
    fn from(error: ExecutionSettingsStoreError) -> Self {
        match error {
            ExecutionSettingsStoreError::ProjectNotFound => Self::ProjectNotFound,
            ExecutionSettingsStoreError::InvalidBudget(error) => Self::InvalidBudget(error),
            ExecutionSettingsStoreError::Storage(_) => Self::Storage,
        }
    }
}

impl From<ExecutionPolicyStoreError> for RunApiError {
    fn from(error: ExecutionPolicyStoreError) -> Self {
        match error {
            ExecutionPolicyStoreError::ProjectNotFound => Self::ProjectNotFound,
            ExecutionPolicyStoreError::DroidNotFound => Self::DroidNotFound,
            ExecutionPolicyStoreError::ProjectPolicyRequired => Self::ExecutionPolicyRequired,
            ExecutionPolicyStoreError::ExistingDroidPolicyConflict => {
                Self::ExistingDroidPolicyConflict
            }
            ExecutionPolicyStoreError::InvalidPolicy(error) => Self::InvalidExecutionPolicy(error),
            ExecutionPolicyStoreError::Storage(_) => Self::Storage,
        }
    }
}

impl From<RunStoreError> for RunApiError {
    fn from(error: RunStoreError) -> Self {
        match error {
            RunStoreError::ProjectNotFound => Self::ProjectNotFound,
            RunStoreError::RunNotFound => Self::RunNotFound,
            RunStoreError::PermissionNotFound => Self::PermissionNotFound,
            RunStoreError::ExecutionBudgetsRequired => Self::ExecutionBudgetsRequired,
            RunStoreError::BudgetExhausted => Self::BudgetExhausted,
            RunStoreError::IdempotencyConflict
            | RunStoreError::InvalidTransition { .. }
            | RunStoreError::StepConflict
            | RunStoreError::ThreadConflict
            | RunStoreError::PermissionDecisionConflict => Self::RunConflict,
            RunStoreError::InvalidRequest(_) => Self::InvalidRunRequest,
            RunStoreError::Storage(_) => Self::Storage,
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetails,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorDetails {
    code: &'static str,
    message: &'static str,
    request_id: String,
}

impl IntoResponse for RunApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::AuthNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "AUTH_NOT_CONFIGURED",
                "Run API authentication is not configured",
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "A valid bearer token is required",
            ),
            Self::InvalidRequest => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_REQUEST",
                "The request body is invalid",
            ),
            Self::ProjectNotFound => (
                StatusCode::NOT_FOUND,
                "PROJECT_NOT_FOUND",
                "Project does not exist",
            ),
            Self::DroidNotFound => (
                StatusCode::NOT_FOUND,
                "DROID_NOT_FOUND",
                "Droid does not exist in this project",
            ),
            Self::RunNotFound => (StatusCode::NOT_FOUND, "RUN_NOT_FOUND", "Run does not exist"),
            Self::PermissionNotFound => (
                StatusCode::NOT_FOUND,
                "PERMISSION_REQUEST_NOT_FOUND",
                "Permission request does not exist",
            ),
            Self::InvalidBudget(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_BUDGET",
                "Budget limits must be positive and within the supported storage range",
            ),
            Self::RunConflict => (
                StatusCode::CONFLICT,
                "RUN_CONFLICT",
                "The requested run state transition conflicts with its current state",
            ),
            Self::BudgetExhausted => (
                StatusCode::CONFLICT,
                "RUN_BUDGET_EXHAUSTED",
                "The run cannot start another operation within its configured budgets",
            ),
            Self::ExecutionBudgetsRequired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "EXECUTION_BUDGETS_REQUIRED",
                "Project execution budgets must be configured before a run can be created",
            ),
            Self::ExecutionPolicyRequired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "EXECUTION_POLICY_REQUIRED",
                "Project execution policy must be configured before droid overrides",
            ),
            Self::InvalidExecutionPolicy(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_EXECUTION_POLICY",
                "Execution policy contains an invalid path or reference",
            ),
            Self::ExistingDroidPolicyConflict => (
                StatusCode::CONFLICT,
                "DROID_POLICY_CONFLICT",
                "Project policy change would invalidate a saved droid policy",
            ),
            Self::InvalidRunRequest => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                "The run request or event cursor is invalid",
            ),
            Self::ExecutionDisabled => (
                StatusCode::SERVICE_UNAVAILABLE,
                "RUN_EXECUTION_DISABLED",
                "Run execution is disabled in this server configuration",
            ),
            Self::UnsupportedVerificationPreset => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UNSUPPORTED_VERIFICATION_PRESET",
                "The verification preset is not available to the run worker",
            ),
            Self::ReviewerDroidRequired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "REVIEWER_DROID_REQUIRED",
                "A distinct code reviewer droid must be configured for this project or selected in the request",
            ),
            Self::ReviewerReadAccessRequired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "REVIEWER_READ_ACCESS_REQUIRED",
                "The reviewer droid must be allowed to read the project",
            ),
            Self::Storage => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "The request could not be completed",
            ),
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let body = ErrorBody {
            error: ErrorDetails {
                code,
                message,
                request_id: format!("run-{now}-{sequence}"),
            },
        };

        (status, Json(body)).into_response()
    }
}
