use std::path::PathBuf;

use tracing::{info, warn};

use crate::{
    db::{Database, sqlite::RunStoreError},
    domain::run::{RecoveryDecision, RecoveryEvidence, RunStatus},
};

use super::{
    codex_app_server::CodexAppServerClient, coordinator::RunCoordinator, worker::RunPolicySnapshot,
};

#[derive(Debug, Default)]
pub struct StartupRecoveryReport {
    pub runs_inspected: usize,
    pub queued_runs_preserved: usize,
    pub pending_permissions_preserved: usize,
    pub threads_resumed: usize,
    pub thread_resume_failures: usize,
    pub completed_turns_verified: usize,
    pub steps_recovered: usize,
    pub runs_blocked: usize,
}

/// Reconciles durable run state before HTTP routes begin accepting requests.
/// This pass never starts a model turn or executes a tool operation.
pub async fn recover_at_startup(
    database: &Database,
    coordinator: &RunCoordinator,
) -> Result<StartupRecoveryReport, RunStoreError> {
    let runs = coordinator.nonterminal_runs()?;
    let mut report = StartupRecoveryReport {
        runs_inspected: runs.len(),
        ..StartupRecoveryReport::default()
    };

    for run in runs {
        let unresolved_permissions = coordinator.unresolved_permissions(&run.run_id)?;
        let in_progress_steps = run
            .steps
            .iter()
            .filter(|step| step.status == crate::domain::run::RunStepStatus::InProgress)
            .collect::<Vec<_>>();
        let recoverable_turn_step = (in_progress_steps.len() == 1)
            .then(|| in_progress_steps[0])
            .filter(|step| {
                step.stage != "REVIEWER"
                    && step.app_server_turn_id.is_some()
                    && step.recovery_next_status.is_some()
            });
        let evidence = if run.status == RunStatus::Reviewing {
            let approved_review = run.steps.iter().rev().find(|step| step.stage == "REVIEWER");
            if let Some(step) = approved_review
                && step.status == crate::domain::run::RunStepStatus::Succeeded
                && step
                    .verification
                    .as_ref()
                    .and_then(|verification| verification.get("verdict"))
                    .and_then(serde_json::Value::as_str)
                    == Some("APPROVE")
            {
                RecoveryEvidence::PersistedCompletion {
                    step_id: step.step_id.clone(),
                    next_status: RunStatus::Succeeded,
                }
            } else {
                RecoveryEvidence::UncertainMutation
            }
        } else if run.status == RunStatus::Queued
            && run.thread_id.is_none()
            && run.steps.is_empty()
            && run.pending_permissions.is_empty()
        {
            report.queued_runs_preserved += 1;
            RecoveryEvidence::NoInFlightOperation
        } else if run.status == RunStatus::WaitingForPermission
            && unresolved_permissions.len() == 1
            && !unresolved_permissions[0].response_sent
        {
            let permission = &unresolved_permissions[0];
            if let Some(thread_id) = run.thread_id.as_deref() {
                let working_directory = database.project_path_for_run(&run.project_id)?;
                let input = coordinator.execution_input(&run.run_id)?;
                let policy: RunPolicySnapshot = serde_json::from_str(&input.policy_snapshot_json)
                    .map_err(|error| {
                    RunStoreError::InvalidRequest(format!(
                        "stored policy snapshot is invalid: {error}"
                    ))
                })?;
                match resume_saved_thread(
                    thread_id,
                    &working_directory,
                    &policy.reference_project_roots,
                    &policy.effective_policy,
                )
                .await
                {
                    Ok(()) => report.threads_resumed += 1,
                    Err(error) => {
                        report.thread_resume_failures += 1;
                        warn!(
                            run_id = %run.run_id,
                            error = %error,
                            "Could not resume app-server thread; keeping unsent permission pending"
                        );
                    }
                }
            }
            report.pending_permissions_preserved += 1;
            RecoveryEvidence::PersistedUnsentApproval {
                request_id: permission.request_id.clone(),
            }
        } else if let Some(step) = recoverable_turn_step {
            let recovery = if let (Some(thread_id), Some(turn_id), Some(next_status)) = (
                run.thread_id.as_deref(),
                step.app_server_turn_id.as_deref(),
                step.recovery_next_status,
            ) {
                let working_directory = database.project_path_for_run(&run.project_id)?;
                match inspect_saved_turn(thread_id, turn_id, &working_directory).await {
                    Ok(Some(evidence)) => {
                        report.threads_resumed += 1;
                        report.completed_turns_verified += 1;
                        coordinator.complete_app_server_step(
                            &step.step_id,
                            &evidence.turn_id,
                            &evidence.final_item_id,
                            "Recovered from a completed Codex app-server turn.",
                        )?;
                        report.steps_recovered += 1;
                        Some(RecoveryEvidence::PersistedAppServerCompletion {
                            step_id: step.step_id.clone(),
                            thread_id: thread_id.to_owned(),
                            turn_id: evidence.turn_id,
                            item_id: evidence.final_item_id,
                            completed_operation_item_ids: evidence.completed_operation_item_ids,
                            next_status,
                            terminal_outcome: step.recovery_terminal_outcome,
                        })
                    }
                    Ok(None) => None,
                    Err(error) => {
                        report.thread_resume_failures += 1;
                        warn!(
                            run_id = %run.run_id,
                            step_id = %step.step_id,
                            error = %error,
                            "Could not verify completed app-server turn during startup recovery"
                        );
                        None
                    }
                }
            } else {
                None
            };
            recovery.unwrap_or(RecoveryEvidence::UncertainMutation)
        } else if let Some(step) = run.steps.iter().rev().find(|step| {
            step.status == crate::domain::run::RunStepStatus::Succeeded
                && step.recovery_next_status.is_some()
        }) {
            let next_status = step.recovery_next_status.unwrap_or(RunStatus::Blocked);
            if run.status == next_status {
                RecoveryEvidence::NoInFlightOperation
            } else {
                RecoveryEvidence::PersistedCompletion {
                    step_id: step.step_id.clone(),
                    next_status,
                }
            }
        } else {
            RecoveryEvidence::UncertainMutation
        };

        let (decision, recovered) = coordinator.recover(&run.run_id, &evidence)?;
        if matches!(decision, RecoveryDecision::Block(_)) {
            report.runs_blocked += 1;
            warn!(
                run_id = %recovered.run_id,
                reason = ?decision,
                "Blocked nonterminal run because startup recovery could not prove a safe continuation"
            );
        } else {
            info!(run_id = %recovered.run_id, decision = ?decision, "Startup run recovery completed");
        }
    }

    Ok(report)
}

async fn resume_saved_thread(
    thread_id: &str,
    working_directory: &PathBuf,
    reference_project_roots: &[PathBuf],
    policy: &crate::domain::execution_policy::EffectiveExecutionPolicy,
) -> Result<(), super::codex_app_server::AppServerError> {
    let mut app_server =
        CodexAppServerClient::start_with_policy(working_directory, reference_project_roots, policy)
            .await?;
    let resume_result = app_server.resume_thread(thread_id, working_directory).await;
    app_server.shutdown().await;
    resume_result
}

async fn inspect_saved_turn(
    thread_id: &str,
    turn_id: &str,
    working_directory: &PathBuf,
) -> Result<
    Option<super::codex_app_server::CompletedTurnEvidence>,
    super::codex_app_server::AppServerError,
> {
    let mut app_server = CodexAppServerClient::start(working_directory).await?;
    let result = async {
        app_server
            .resume_thread(thread_id, working_directory)
            .await?;
        app_server.inspect_completed_turn(thread_id, turn_id).await
    }
    .await;
    app_server.shutdown().await;
    result
}
