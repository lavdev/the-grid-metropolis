use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunOutcome {
    Succeeded,
    Failed,
    Blocked,
    BudgetExhausted,
    Cancelled,
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
