use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunStatus {
    Queued,
    Planning,
    Executing,
    Verifying,
    Repairing,
    Reviewing,
    WaitingForPermission,
    Succeeded,
    Failed,
    Blocked,
    Cancelled,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "QUEUED",
            Self::Planning => "PLANNING",
            Self::Executing => "EXECUTING",
            Self::Verifying => "VERIFYING",
            Self::Repairing => "REPAIRING",
            Self::Reviewing => "REVIEWING",
            Self::WaitingForPermission => "WAITING_FOR_PERMISSION",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Blocked => "BLOCKED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Blocked | Self::Cancelled
        )
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        use RunStatus::*;
        match self {
            Queued => matches!(next, Planning | Cancelled | Blocked),
            Planning => matches!(
                next,
                Executing | WaitingForPermission | Failed | Blocked | Cancelled
            ),
            Executing => matches!(
                next,
                Verifying | WaitingForPermission | Failed | Blocked | Cancelled
            ),
            Verifying => matches!(
                next,
                Repairing | Reviewing | WaitingForPermission | Failed | Blocked | Cancelled
            ),
            Repairing => matches!(
                next,
                Verifying | WaitingForPermission | Failed | Blocked | Cancelled
            ),
            Reviewing => matches!(
                next,
                Succeeded | WaitingForPermission | Failed | Blocked | Cancelled
            ),
            WaitingForPermission => matches!(
                next,
                Planning
                    | Executing
                    | Verifying
                    | Repairing
                    | Reviewing
                    | Failed
                    | Blocked
                    | Cancelled
            ),
            Succeeded | Failed | Blocked | Cancelled => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunOutcome {
    Succeeded,
    Failed,
    Blocked,
    BudgetExhausted,
    Cancelled,
}

impl RunOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Blocked => "BLOCKED",
            Self::BudgetExhausted => "BUDGET_EXHAUSTED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBudgetLimits {
    pub max_model_turns_per_run: u64,
    pub max_elapsed_seconds_per_run: u64,
    pub max_tokens_per_run: u64,
}

impl RunBudgetLimits {
    pub fn validate(&self) -> Result<(), BudgetConfigError> {
        let sqlite_max = i64::MAX as u64;
        if self.max_model_turns_per_run == 0 {
            return Err(BudgetConfigError::ZeroLimit("maxModelTurnsPerRun"));
        }
        if self.max_model_turns_per_run > sqlite_max {
            return Err(BudgetConfigError::ExceedsStorageRange(
                "maxModelTurnsPerRun",
            ));
        }
        if self.max_elapsed_seconds_per_run == 0 {
            return Err(BudgetConfigError::ZeroLimit("maxElapsedSecondsPerRun"));
        }
        if self.max_elapsed_seconds_per_run > sqlite_max {
            return Err(BudgetConfigError::ExceedsStorageRange(
                "maxElapsedSecondsPerRun",
            ));
        }
        if self.max_tokens_per_run == 0 {
            return Err(BudgetConfigError::ZeroLimit("maxTokensPerRun"));
        }
        if self.max_tokens_per_run > sqlite_max {
            return Err(BudgetConfigError::ExceedsStorageRange("maxTokensPerRun"));
        }
        Ok(())
    }

    pub fn tighten_with(self, override_limits: Option<Self>) -> Result<Self, BudgetConfigError> {
        let Some(override_limits) = override_limits else {
            return Ok(self);
        };

        override_limits.validate()?;
        if override_limits.max_model_turns_per_run > self.max_model_turns_per_run
            || override_limits.max_elapsed_seconds_per_run > self.max_elapsed_seconds_per_run
            || override_limits.max_tokens_per_run > self.max_tokens_per_run
        {
            return Err(BudgetConfigError::OverrideRaisesProjectLimit);
        }

        Ok(override_limits)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RunBudgetUsage {
    pub model_turns_started: u64,
    pub elapsed_seconds: u64,
    pub reported_tokens: u64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BudgetConfigError {
    #[error("budget field {0} must be greater than zero")]
    ZeroLimit(&'static str),
    #[error("budget field {0} exceeds the supported storage range")]
    ExceedsStorageRange(&'static str),
    #[error("a droid or run budget override cannot exceed the project limit")]
    OverrideRaisesProjectLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExecutionSettings {
    pub budgets: RunBudgetLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSnapshot {
    pub run_id: String,
    pub project_id: String,
    pub droid_ids: Vec<String>,
    pub reviewer_droid_id: Option<String>,
    pub idempotency_key: String,
    pub task_title: String,
    pub verification_preset_id: String,
    pub status: RunStatus,
    pub terminal_outcome: Option<RunOutcome>,
    pub budget_limits: RunBudgetLimits,
    pub usage: RunBudgetUsage,
    pub thread_id: Option<String>,
    pub steps: Vec<RunStep>,
    pub pending_permissions: Vec<RunPermissionSummary>,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    pub last_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub run_id: String,
    pub sequence: u64,
    pub event_type: String,
    pub timestamp: i64,
    pub project_id: String,
    pub step_id: Option<String>,
    pub droid_id: Option<String>,
    pub attempt: Option<u32>,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRunRecord {
    pub run_id: String,
    pub project_id: String,
    pub droid_ids: Vec<String>,
    pub idempotency_key: String,
    pub task_title: String,
    pub prompt: String,
    pub verification_preset_id: String,
    pub policy_snapshot_json: String,
    pub budget_limits: RunBudgetLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunExecutionInput {
    pub task_title: String,
    pub prompt: String,
    pub verification_preset_id: String,
    pub policy_snapshot_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunStepStatus {
    Pending,
    InProgress,
    Succeeded,
    Failed,
    Skipped,
}

impl RunStepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::InProgress => "IN_PROGRESS",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Skipped => "SKIPPED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStep {
    pub step_id: String,
    pub run_id: String,
    pub ordinal: u32,
    pub stage: String,
    pub droid_id: String,
    pub attempt: u32,
    pub status: RunStepStatus,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub result_summary: Option<String>,
    pub verification: Option<Value>,
    pub app_server_thread_id: Option<String>,
    pub app_server_turn_id: Option<String>,
    pub app_server_item_id: Option<String>,
    pub recovery_next_status: Option<RunStatus>,
    pub recovery_terminal_outcome: Option<RunOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRunStep {
    pub step_id: String,
    pub run_id: String,
    pub ordinal: u32,
    pub stage: String,
    pub droid_id: String,
    pub attempt: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionStatus {
    Pending,
    Granted,
    Denied,
}

impl PermissionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Granted => "GRANTED",
            Self::Denied => "DENIED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPermissionRequest {
    pub request_id: String,
    pub run_id: String,
    pub project_id: String,
    pub step_id: Option<String>,
    pub droid_id: String,
    pub capability: String,
    pub action: String,
    pub resource: String,
    pub rationale: String,
    pub status: PermissionStatus,
    pub alternative_index: u32,
    pub created_at: i64,
    pub decided_at: Option<i64>,
    pub grant_scope: Option<String>,
    pub resume_status: RunStatus,
    pub app_server_generation: Option<String>,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub item_id: Option<String>,
    pub json_rpc_request_id: Option<String>,
    pub app_server_approval_id: Option<String>,
    pub payload: Value,
    pub payload_fingerprint: String,
    pub response_sent: bool,
    pub decision_idempotency_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPermissionSummary {
    pub request_id: String,
    pub step_id: Option<String>,
    pub droid_id: String,
    pub capability: String,
    pub action: String,
    pub resource: String,
    pub rationale: String,
    pub status: PermissionStatus,
    pub alternative_index: u32,
    pub created_at: i64,
    pub payload_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRunPermissionRequest {
    pub request_id: String,
    pub run_id: String,
    pub step_id: Option<String>,
    pub droid_id: String,
    pub capability: String,
    pub action: String,
    pub resource: String,
    pub rationale: String,
    pub alternative_index: u32,
    pub app_server_generation: Option<String>,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub item_id: Option<String>,
    pub json_rpc_request_id: Option<String>,
    pub app_server_approval_id: Option<String>,
    pub payload_json: String,
    pub payload_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryEvidence {
    PersistedUnsentApproval {
        request_id: String,
    },
    PendingUnsentApproval {
        request_id: String,
        stored_fingerprint: String,
        resumed_fingerprint: String,
    },
    GrantSentWithoutResult {
        request_id: String,
    },
    PersistedCompletion {
        step_id: String,
        next_status: RunStatus,
    },
    PersistedAppServerCompletion {
        step_id: String,
        thread_id: String,
        turn_id: String,
        item_id: String,
        completed_operation_item_ids: Vec<String>,
        next_status: RunStatus,
        terminal_outcome: Option<RunOutcome>,
    },
    UncertainMutation,
    NoInFlightOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryDecision {
    KeepPendingApproval,
    Advance(RunStatus),
    Block(&'static str),
    Continue,
}

pub fn evaluate_recovery(evidence: &RecoveryEvidence) -> RecoveryDecision {
    match evidence {
        RecoveryEvidence::PersistedUnsentApproval { .. } => RecoveryDecision::KeepPendingApproval,
        RecoveryEvidence::PendingUnsentApproval {
            stored_fingerprint,
            resumed_fingerprint,
            ..
        } if stored_fingerprint == resumed_fingerprint => RecoveryDecision::KeepPendingApproval,
        RecoveryEvidence::PendingUnsentApproval { .. } => {
            RecoveryDecision::Block("APPROVAL_FINGERPRINT_MISMATCH")
        }
        RecoveryEvidence::GrantSentWithoutResult { .. } => {
            RecoveryDecision::Block("GRANT_RESULT_NOT_PERSISTED")
        }
        RecoveryEvidence::PersistedCompletion { next_status, .. }
        | RecoveryEvidence::PersistedAppServerCompletion { next_status, .. } => {
            RecoveryDecision::Advance(*next_status)
        }
        RecoveryEvidence::UncertainMutation => RecoveryDecision::Block("UNCERTAIN_MUTATION"),
        RecoveryEvidence::NoInFlightOperation => RecoveryDecision::Continue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_transitions_allow_only_the_sequential_lifecycle() {
        assert!(RunStatus::Queued.can_transition_to(RunStatus::Planning));
        assert!(RunStatus::Verifying.can_transition_to(RunStatus::Repairing));
        assert!(RunStatus::Reviewing.can_transition_to(RunStatus::Succeeded));
        assert!(!RunStatus::Queued.can_transition_to(RunStatus::Succeeded));
        assert!(!RunStatus::Succeeded.can_transition_to(RunStatus::Planning));
    }

    #[test]
    fn recovery_requires_matching_approval_and_never_replays_uncertain_mutation() {
        assert_eq!(
            evaluate_recovery(&RecoveryEvidence::PendingUnsentApproval {
                request_id: "approval-1".into(),
                stored_fingerprint: "same".into(),
                resumed_fingerprint: "same".into(),
            }),
            RecoveryDecision::KeepPendingApproval,
        );
        assert_eq!(
            evaluate_recovery(&RecoveryEvidence::PendingUnsentApproval {
                request_id: "approval-2".into(),
                stored_fingerprint: "old".into(),
                resumed_fingerprint: "new".into(),
            }),
            RecoveryDecision::Block("APPROVAL_FINGERPRINT_MISMATCH"),
        );
        assert_eq!(
            evaluate_recovery(&RecoveryEvidence::GrantSentWithoutResult {
                request_id: "approval-3".into(),
            }),
            RecoveryDecision::Block("GRANT_RESULT_NOT_PERSISTED"),
        );
        assert_eq!(
            evaluate_recovery(&RecoveryEvidence::UncertainMutation),
            RecoveryDecision::Block("UNCERTAIN_MUTATION"),
        );
        assert_eq!(
            evaluate_recovery(&RecoveryEvidence::PersistedCompletion {
                step_id: "step-1".into(),
                next_status: RunStatus::Verifying,
            }),
            RecoveryDecision::Advance(RunStatus::Verifying),
        );
    }
}
