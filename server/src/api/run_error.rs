use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::db::sqlite::ExecutionSettingsStoreError;
use crate::domain::run::BudgetConfigError;

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub enum RunApiError {
    AuthNotConfigured,
    Unauthorized,
    InvalidRequest,
    ProjectNotFound,
    InvalidBudget(BudgetConfigError),
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
            Self::InvalidBudget(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_BUDGET",
                "Budget limits must be positive and within the supported storage range",
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
