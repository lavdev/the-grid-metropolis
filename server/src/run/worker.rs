use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::time::timeout;
use tracing::error;

use crate::{
    db::Database,
    domain::{execution_policy::EffectiveExecutionPolicy, run::*},
};

use super::{
    codex_app_server::{CodexAppServerClient, turn_completion_from_message},
    coordinator::RunCoordinator,
    dispatcher::RunDispatcher,
};

static PERMISSION_ID_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const MAX_REPAIR_ATTEMPTS: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPolicySnapshot {
    pub effective_policy: EffectiveExecutionPolicy,
    pub reference_project_roots: Vec<PathBuf>,
    #[serde(default)]
    pub reviewer: Option<ReviewPolicySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPolicySnapshot {
    pub effective_policy: EffectiveExecutionPolicy,
    pub reference_project_roots: Vec<PathBuf>,
}

#[derive(Clone)]
pub struct RunWorker {
    db: Database,
    enabled: bool,
    active: Arc<Mutex<HashSet<String>>>,
    dispatcher: RunDispatcher,
}

impl RunWorker {
    pub fn new(db: Database) -> Self {
        let enabled = std::env::var("GRID_METROPOLIS_RUNS_ENABLED")
            .is_ok_and(|value| value.eq_ignore_ascii_case("true"));
        Self::with_enabled(db, enabled)
    }

    pub fn new_with_dispatcher(db: Database, dispatcher: RunDispatcher) -> Self {
        let enabled = std::env::var("GRID_METROPOLIS_RUNS_ENABLED")
            .is_ok_and(|value| value.eq_ignore_ascii_case("true"));
        Self::with_dispatcher(db, enabled, dispatcher)
    }

    pub fn with_enabled(db: Database, enabled: bool) -> Self {
        Self::with_dispatcher(db, enabled, RunDispatcher::default())
    }

    pub fn with_dispatcher(db: Database, enabled: bool, dispatcher: RunDispatcher) -> Self {
        Self {
            db,
            enabled,
            active: Arc::default(),
            dispatcher,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn supports_verification_preset(preset: &str) -> bool {
        verification_command(preset).is_some()
    }

    pub fn submit(
        &self,
        record: &NewRunRecord,
    ) -> Result<RunSnapshot, crate::db::sqlite::RunStoreError> {
        let snapshot = RunCoordinator::new(self.db.clone()).create(record)?;
        if snapshot.status == RunStatus::Queued {
            self.enqueue(&snapshot.run_id);
        }
        Ok(snapshot)
    }

    pub fn enqueue(&self, run_id: &str) -> bool {
        if !self.enabled {
            return false;
        }
        let Ok(mut active) = self.active.lock() else {
            return false;
        };
        if !active.insert(run_id.to_owned()) {
            return false;
        }
        drop(active);

        let worker = self.clone();
        let run_id = run_id.to_owned();
        tokio::spawn(async move {
            if let Err(error) = worker.execute(&run_id).await {
                error!(run_id = %run_id, error = %error, "Run worker stopped with an error");
                worker.block_run(&run_id, "WORKER_ERROR");
            }
            if let Ok(mut active) = worker.active.lock() {
                active.remove(&run_id);
            }
        });
        true
    }

    pub fn enqueue_queued_runs(&self) -> Result<usize, crate::db::sqlite::RunStoreError> {
        if !self.enabled {
            return Ok(0);
        }
        let queued = self
            .db
            .list_nonterminal_runs()?
            .into_iter()
            .filter(|run| run.status == RunStatus::Queued)
            .filter(|run| self.enqueue(&run.run_id))
            .count();
        Ok(queued)
    }

    pub fn enqueue_pending_approval_runs(&self) -> Result<usize, crate::db::sqlite::RunStoreError> {
        if !self.enabled {
            return Ok(0);
        }
        let coordinator = RunCoordinator::new(self.db.clone());
        let mut enqueued = 0;
        for run in coordinator.nonterminal_runs()? {
            if run.status != RunStatus::WaitingForPermission {
                continue;
            }
            let unresolved = coordinator.unresolved_permissions(&run.run_id)?;
            if unresolved.len() != 1 || unresolved[0].response_sent {
                continue;
            }
            let Ok(mut active) = self.active.lock() else {
                continue;
            };
            if !active.insert(run.run_id.clone()) {
                continue;
            }
            drop(active);
            let worker = self.clone();
            let run_id = run.run_id.clone();
            tokio::spawn(async move {
                if let Err(error) = worker.resume_pending_approval(&run_id).await {
                    error!(run_id = %run_id, error = %error, "Run approval recovery stopped with an error");
                    worker.block_run(&run_id, "APPROVAL_RECOVERY_ERROR");
                }
                if let Ok(mut active) = worker.active.lock() {
                    active.remove(&run_id);
                }
            });
            enqueued += 1;
        }
        Ok(enqueued)
    }

    async fn resume_pending_approval(&self, run_id: &str) -> Result<(), String> {
        let started = Instant::now();
        let coordinator = RunCoordinator::new(self.db.clone());
        let run = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?;
        if run.status != RunStatus::WaitingForPermission {
            return Ok(());
        }
        let mut unresolved = coordinator
            .unresolved_permissions(run_id)
            .map_err(|error| error.to_string())?;
        if unresolved.len() != 1 || unresolved[0].response_sent {
            self.block_run(run_id, "AMBIGUOUS_PENDING_APPROVALS");
            return Ok(());
        }
        let permission = unresolved.remove(0);
        let saved_thread_id = run
            .thread_id
            .clone()
            .ok_or_else(|| "pending approval has no saved thread".to_owned())?;
        if permission.thread_id.as_deref() != Some(&saved_thread_id) {
            self.block_run(run_id, "APPROVAL_THREAD_MISMATCH");
            return Ok(());
        }
        let input = coordinator
            .execution_input(run_id)
            .map_err(|error| error.to_string())?;
        let policy: RunPolicySnapshot = serde_json::from_str(&input.policy_snapshot_json)
            .map_err(|error| format!("stored policy snapshot is invalid: {error}"))?;
        let root = self
            .db
            .project_path_for_run(&run.project_id)
            .map_err(|error| error.to_string())?;
        let root = std::fs::canonicalize(root).map_err(|error| error.to_string())?;
        let original_step = run
            .steps
            .iter()
            .find(|step| Some(step.step_id.as_str()) == permission.step_id.as_deref())
            .ok_or_else(|| "pending approval step is missing".to_owned())?;
        let droid_id = permission.droid_id.clone();
        let step_id = next_id("step");

        let mut app_server = CodexAppServerClient::start_with_policy(
            &root,
            &policy.reference_project_roots,
            &policy.effective_policy,
        )
        .await
        .map_err(|error| error.to_string())?;
        app_server
            .resume_thread(&saved_thread_id, &root)
            .await
            .map_err(|error| error.to_string())?;
        let (Some(previous_turn_id), Some(pending_item_id)) =
            (permission.turn_id.as_deref(), permission.item_id.as_deref())
        else {
            app_server.shutdown().await;
            self.block_run(run_id, "APPROVAL_RECOVERY_EVIDENCE_MISSING");
            return Ok(());
        };
        let safe_to_reissue = app_server
            .can_reissue_pending_approval(&saved_thread_id, previous_turn_id, pending_item_id)
            .await
            .map_err(|error| error.to_string())?;
        if !safe_to_reissue {
            app_server.shutdown().await;
            self.block_run(run_id, "APPROVAL_RECOVERY_HAS_COMPLETED_OPERATIONS");
            return Ok(());
        }

        let resume_status = permission.resume_status;
        coordinator
            .transition(
                run_id,
                resume_status,
                None,
                "RUN_APPROVAL_RECOVERY_STARTED",
                &json!({ "threadId": saved_thread_id, "permissionRequestId": permission.request_id, "resumeStatus": resume_status.as_str() }),
            )
            .map_err(|error| error.to_string())?;
        coordinator
            .create_step(&NewRunStep {
                step_id: step_id.clone(),
                run_id: run_id.to_owned(),
                ordinal: original_step.ordinal,
                stage: original_step.stage.clone(),
                droid_id: droid_id.clone(),
                attempt: original_step.attempt.saturating_add(1),
            })
            .map_err(|error| error.to_string())?;
        coordinator
            .start_step(&step_id)
            .map_err(|error| error.to_string())?;
        coordinator
            .reserve_model_turn(run_id)
            .map_err(|error| error.to_string())?;
        let prompt = if original_step.stage == "REPAIR" {
            let failed_verification = run
                .steps
                .iter()
                .filter(|step| step.stage == "VERIFICATION" && step.status == RunStepStatus::Failed)
                .max_by_key(|step| step.attempt)
                .and_then(|step| step.verification.as_ref())
                .ok_or_else(|| {
                    "repair recovery lacks persisted verification evidence".to_owned()
                })?;
            repair_prompt(
                &input.task_title,
                &input.prompt,
                &input.verification_preset_id,
                failed_verification,
            )
        } else {
            initial_task_prompt(&input.task_title, &input.prompt)
        };
        let turn_id = app_server
            .start_turn(&saved_thread_id, &root, &prompt, None, "on-request")
            .await
            .map_err(|error| error.to_string())?;
        coordinator
            .persist_step_turn(&step_id, &turn_id, RunStatus::Verifying, None)
            .map_err(|error| error.to_string())?;

        let previous_elapsed = run.usage.elapsed_seconds;
        let mut matching_approval_reissued = false;
        let completion = loop {
            let elapsed = previous_elapsed.saturating_add(started.elapsed().as_secs());
            let max_elapsed = run.budget_limits.max_elapsed_seconds_per_run;
            if elapsed >= max_elapsed {
                let _ = app_server
                    .request_turn_interrupt(&saved_thread_id, &turn_id)
                    .await;
                app_server.shutdown().await;
                self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_DURING_APPROVAL_RECOVERY");
                return Ok(());
            }
            let remaining = Duration::from_secs(max_elapsed.saturating_sub(elapsed));
            let message = timeout(remaining, app_server.next_message())
                .await
                .map_err(|_| "recovered turn exceeded elapsed-time budget".to_owned())?
                .map_err(|error| error.to_string())?;
            if let Some(done) = turn_completion_from_message(&message, &turn_id) {
                break done;
            }
            if let Some(reported_tokens) = token_usage_from_message(&message) {
                let current = coordinator
                    .snapshot(run_id)
                    .map_err(|error| error.to_string())?;
                coordinator
                    .record_usage(
                        run_id,
                        RunBudgetUsage {
                            model_turns_started: current.usage.model_turns_started,
                            elapsed_seconds: previous_elapsed
                                .saturating_add(started.elapsed().as_secs()),
                            reported_tokens,
                        },
                    )
                    .map_err(|error| error.to_string())?;
                if reported_tokens >= run.budget_limits.max_tokens_per_run {
                    let _ = app_server
                        .request_turn_interrupt(&saved_thread_id, &turn_id)
                        .await;
                    app_server.shutdown().await;
                    self.block_budget_run(run_id, "TOKEN_BUDGET_REACHED_DURING_APPROVAL_RECOVERY");
                    return Ok(());
                }
            }
            let super::codex_app_server::AppServerMessage::Request {
                request_id: rpc_request_id,
                method,
                params,
            } = message
            else {
                continue;
            };
            if !is_supported_approval_method(&method) {
                app_server
                    .respond_to_server_request(rpc_request_id, json!({ "decision": "decline" }))
                    .await
                    .map_err(|error| error.to_string())?;
                continue;
            }
            let payload = json!({ "method": method, "params": params });
            let payload_json =
                serde_json::to_string(&payload).map_err(|error| error.to_string())?;
            let fingerprint = permission_fingerprint(&method, &params);
            if payload_json.len() > 32 * 1024 || fingerprint != permission.payload_fingerprint {
                let _ = app_server
                    .respond_to_server_request_and_wait(
                        rpc_request_id,
                        json!({ "decision": "decline" }),
                    )
                    .await;
                app_server.shutdown().await;
                coordinator
                    .complete_step(
                        &step_id,
                        RunStepStatus::Failed,
                        "Recovery approval fingerprint did not match the persisted request.",
                        None,
                    )
                    .map_err(|error| error.to_string())?;
                self.block_run(run_id, "APPROVAL_FINGERPRINT_MISMATCH");
                return Ok(());
            }

            let Some(mut decision_receiver) = self
                .dispatcher
                .register_pending_approval(run_id, &permission.request_id)
            else {
                app_server
                    .respond_to_server_request(rpc_request_id, json!({ "decision": "decline" }))
                    .await
                    .map_err(|error| error.to_string())?;
                self.block_run(run_id, "APPROVAL_SESSION_CONFLICT");
                return Ok(());
            };
            coordinator
                .refresh_unsent_permission(
                    &permission.request_id,
                    &saved_thread_id,
                    &turn_id,
                    params.get("itemId").and_then(Value::as_str),
                    &rpc_request_id.to_string(),
                    params.get("approvalId").and_then(Value::as_str),
                    &fingerprint,
                )
                .map_err(|error| error.to_string())?;
            matching_approval_reissued = true;

            let decision = if permission.status == PermissionStatus::Pending {
                Some(
                    decision_receiver
                        .recv()
                        .await
                        .ok_or_else(|| "permission decision channel closed".to_owned())?,
                )
            } else {
                None
            };
            let granted = decision
                .as_ref()
                .map(|decision| decision.granted)
                .unwrap_or(permission.status == PermissionStatus::Granted);
            let result = if granted {
                json!({ "decision": "accept" })
            } else {
                json!({ "decision": "decline" })
            };
            let response = app_server
                .respond_to_server_request_and_wait(rpc_request_id, result)
                .await;
            let acknowledged = if response.is_ok() {
                coordinator
                    .mark_permission_response_sent(&permission.request_id)
                    .is_ok()
            } else {
                false
            };
            if let Some(decision) = decision {
                let _ = decision.acknowledge.send(acknowledged);
            }
            self.dispatcher
                .unregister_pending_approval(run_id, &permission.request_id);
            response.map_err(|error| error.to_string())?;
            if !acknowledged {
                app_server.shutdown().await;
                return Err("recovered permission acknowledgement was not persisted".into());
            }
        };

        if !matching_approval_reissued {
            app_server.shutdown().await;
            coordinator
                .complete_step(
                    &step_id,
                    RunStepStatus::Failed,
                    "The resumed turn did not reproduce the pending permission request.",
                    None,
                )
                .map_err(|error| error.to_string())?;
            self.block_run(run_id, "APPROVAL_NOT_REISSUED");
            return Ok(());
        }
        if completion.status != "completed" {
            app_server.shutdown().await;
            coordinator
                .complete_step(
                    &step_id,
                    RunStepStatus::Failed,
                    "Recovered Codex turn did not complete successfully.",
                    Some(
                        &json!({ "status": completion.status, "error": completion.error_message })
                            .to_string(),
                    ),
                )
                .map_err(|error| error.to_string())?;
            self.block_run(run_id, "RECOVERED_TURN_FAILED");
            return Ok(());
        }
        let evidence = app_server
            .inspect_completed_turn(&saved_thread_id, &turn_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "recovered turn lacks persisted final-answer evidence".to_owned())?;
        app_server.shutdown().await;
        coordinator
            .complete_app_server_step(
                &step_id,
                &turn_id,
                &evidence.final_item_id,
                "Recovered after matching the persisted permission fingerprint.",
            )
            .map_err(|error| error.to_string())?;
        coordinator
            .complete_step(
                &original_step.step_id,
                RunStepStatus::Failed,
                "Interrupted by process restart; resumed in a new attempt after permission fingerprint matched.",
                None,
            )
            .map_err(|error| error.to_string())?;
        coordinator
            .transition(
                run_id,
                RunStatus::Verifying,
                None,
                "RUN_VERIFYING",
                &json!({ "stepId": step_id, "recoveredPermissionRequestId": permission.request_id }),
            )
            .map_err(|error| error.to_string())?;
        self.verify_run_after_recovery(
            run_id,
            &step_id,
            &input,
            &root,
            &droid_id,
            started,
            previous_elapsed,
        )
        .await
    }

    async fn verify_run_after_recovery(
        &self,
        run_id: &str,
        coder_step_id: &str,
        input: &RunExecutionInput,
        root: &PathBuf,
        droid_id: &str,
        started: Instant,
        previous_elapsed: u64,
    ) -> Result<(), String> {
        let coordinator = RunCoordinator::new(self.db.clone());
        let run = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?;
        let elapsed = previous_elapsed.saturating_add(started.elapsed().as_secs());
        if elapsed >= run.budget_limits.max_elapsed_seconds_per_run {
            self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_BEFORE_VERIFICATION");
            return Ok(());
        }
        let (argv, preset_timeout_ms) = verification_command(&input.verification_preset_id)
            .ok_or_else(|| "verification preset is unsupported".to_owned())?;
        let remaining_ms = run
            .budget_limits
            .max_elapsed_seconds_per_run
            .saturating_sub(elapsed)
            .saturating_mul(1000);
        let verification_step_id = next_id("step");
        coordinator
            .create_step(&NewRunStep {
                step_id: verification_step_id.clone(),
                run_id: run_id.to_owned(),
                ordinal: 1,
                stage: "VERIFICATION".into(),
                droid_id: droid_id.to_owned(),
                attempt: 1,
            })
            .map_err(|error| error.to_string())?;
        coordinator
            .start_step(&verification_step_id)
            .map_err(|error| error.to_string())?;
        let policy: RunPolicySnapshot = serde_json::from_str(&input.policy_snapshot_json)
            .map_err(|error| format!("stored policy snapshot is invalid: {error}"))?;
        let mut app_server = CodexAppServerClient::start_with_policy(
            root,
            &policy.reference_project_roots,
            &policy.effective_policy,
        )
        .await
        .map_err(|error| error.to_string())?;
        let result = app_server
            .execute_command(&argv, root, preset_timeout_ms.min(remaining_ms))
            .await
            .map_err(|error| error.to_string())?;
        app_server.shutdown().await;
        let elapsed = previous_elapsed.saturating_add(started.elapsed().as_secs());
        let latest = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?;
        coordinator
            .record_usage(
                run_id,
                RunBudgetUsage {
                    model_turns_started: latest.usage.model_turns_started,
                    elapsed_seconds: elapsed,
                    reported_tokens: latest.usage.reported_tokens,
                },
            )
            .map_err(|error| error.to_string())?;
        if elapsed >= run.budget_limits.max_elapsed_seconds_per_run {
            self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_DURING_VERIFICATION");
            return Ok(());
        }
        let evidence = bounded_verification_evidence(&input.verification_preset_id, &result);
        let passed = result.get("exitCode").and_then(Value::as_i64) == Some(0);
        coordinator
            .complete_step(
                &verification_step_id,
                if passed {
                    RunStepStatus::Succeeded
                } else {
                    RunStepStatus::Failed
                },
                if passed {
                    "Verification passed."
                } else {
                    "Verification failed."
                },
                Some(&evidence.to_string()),
            )
            .map_err(|error| error.to_string())?;
        if passed {
            coordinator
                .transition(
                    run_id,
                    RunStatus::Reviewing,
                    None,
                    "RUN_REVIEWING",
                    &json!({ "coderStepId": coder_step_id }),
                )
                .map_err(|error| error.to_string())?;
            self.run_independent_review(
                run_id,
                input,
                root,
                &policy,
                started,
                previous_elapsed,
                &evidence,
            )
            .await?;
        } else {
            coordinator
                .transition(run_id, RunStatus::Failed, Some(RunOutcome::Failed), "RUN_FAILED", &json!({ "reason": "VERIFICATION_FAILED", "verificationStepId": verification_step_id }))
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    async fn repair_after_failed_verification(
        &self,
        run_id: &str,
        input: &RunExecutionInput,
        root: &PathBuf,
        policy: &RunPolicySnapshot,
        thread_id: &str,
        droid_id: &str,
        started: Instant,
        failed_verification: &Value,
    ) -> Result<(), String> {
        let coordinator = RunCoordinator::new(self.db.clone());
        let run = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?;
        let elapsed = started.elapsed().as_secs();
        if run.usage.model_turns_started >= run.budget_limits.max_model_turns_per_run
            || run.usage.reported_tokens >= run.budget_limits.max_tokens_per_run
            || elapsed >= run.budget_limits.max_elapsed_seconds_per_run
        {
            self.block_budget_run(run_id, "BUDGET_EXHAUSTED_BEFORE_REPAIR");
            return Ok(());
        }
        if MAX_REPAIR_ATTEMPTS == 0 {
            return Err("repair attempts are disabled".into());
        }

        coordinator
            .transition(
                run_id,
                RunStatus::Repairing,
                None,
                "RUN_REPAIRING",
                &json!({
                    "repairAttempt": 1,
                    "maxRepairAttempts": MAX_REPAIR_ATTEMPTS,
                    "verification": failed_verification,
                }),
            )
            .map_err(|error| error.to_string())?;
        let repair_step_id = next_id("step");
        coordinator
            .create_step(&NewRunStep {
                step_id: repair_step_id.clone(),
                run_id: run_id.to_owned(),
                ordinal: 2,
                stage: "REPAIR".into(),
                droid_id: droid_id.to_owned(),
                attempt: 1,
            })
            .map_err(|error| error.to_string())?;
        coordinator
            .start_step(&repair_step_id)
            .map_err(|error| error.to_string())?;
        coordinator
            .reserve_model_turn(run_id)
            .map_err(|error| error.to_string())?;
        let prompt = repair_prompt(
            &input.task_title,
            &input.prompt,
            &input.verification_preset_id,
            failed_verification,
        );
        let mut app_server = CodexAppServerClient::start_with_policy(
            root,
            &policy.reference_project_roots,
            &policy.effective_policy,
        )
        .await
        .map_err(|error| error.to_string())?;
        app_server
            .resume_thread(thread_id, root)
            .await
            .map_err(|error| error.to_string())?;
        let repair_turn_id = app_server
            .start_turn(thread_id, root, &prompt, None, "on-request")
            .await
            .map_err(|error| error.to_string())?;
        coordinator
            .persist_step_turn(&repair_step_id, &repair_turn_id, RunStatus::Verifying, None)
            .map_err(|error| error.to_string())?;

        let completion = loop {
            let elapsed = started.elapsed();
            let remaining = Duration::from_secs(
                run.budget_limits
                    .max_elapsed_seconds_per_run
                    .saturating_sub(elapsed.as_secs()),
            );
            if remaining.is_zero() {
                let _ = app_server
                    .request_turn_interrupt(thread_id, &repair_turn_id)
                    .await;
                app_server.shutdown().await;
                self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_DURING_REPAIR");
                return Ok(());
            }
            let message = timeout(remaining, app_server.next_message())
                .await
                .map_err(|_| "repair turn exceeded elapsed-time budget".to_owned())?
                .map_err(|error| error.to_string())?;
            if let Some(done) = turn_completion_from_message(&message, &repair_turn_id) {
                break done;
            }
            if let Some(reported_tokens) = token_usage_from_message(&message) {
                let current = coordinator
                    .snapshot(run_id)
                    .map_err(|error| error.to_string())?;
                coordinator
                    .record_usage(
                        run_id,
                        RunBudgetUsage {
                            model_turns_started: current.usage.model_turns_started,
                            elapsed_seconds: started.elapsed().as_secs(),
                            reported_tokens,
                        },
                    )
                    .map_err(|error| error.to_string())?;
                if reported_tokens >= run.budget_limits.max_tokens_per_run {
                    let _ = app_server
                        .request_turn_interrupt(thread_id, &repair_turn_id)
                        .await;
                    app_server.shutdown().await;
                    self.block_budget_run(run_id, "TOKEN_BUDGET_REACHED_DURING_REPAIR");
                    return Ok(());
                }
            }
            if let super::codex_app_server::AppServerMessage::Request {
                request_id,
                method,
                params,
            } = message
            {
                if is_supported_approval_method(&method) {
                    self.await_live_permission(
                        &coordinator,
                        &mut app_server,
                        run_id,
                        &repair_step_id,
                        droid_id,
                        thread_id,
                        &repair_turn_id,
                        request_id,
                        &method,
                        &params,
                    )
                    .await?;
                } else {
                    app_server
                        .respond_to_server_request(request_id, json!({ "decision": "decline" }))
                        .await
                        .map_err(|error| error.to_string())?;
                }
            }
        };
        let current_usage = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?
            .usage;
        coordinator
            .record_usage(
                run_id,
                RunBudgetUsage {
                    model_turns_started: current_usage.model_turns_started,
                    elapsed_seconds: started.elapsed().as_secs(),
                    reported_tokens: current_usage.reported_tokens,
                },
            )
            .map_err(|error| error.to_string())?;
        if completion.status != "completed" {
            app_server.shutdown().await;
            coordinator
                .complete_step(
                    &repair_step_id,
                    RunStepStatus::Failed,
                    "The repair turn did not complete successfully.",
                    Some(
                        &json!({ "status": completion.status, "error": completion.error_message })
                            .to_string(),
                    ),
                )
                .map_err(|error| error.to_string())?;
            coordinator
                .transition(
                    run_id,
                    RunStatus::Failed,
                    Some(RunOutcome::Failed),
                    "RUN_FAILED",
                    &json!({ "reason": "REPAIR_TURN_FAILED", "repairStepId": repair_step_id }),
                )
                .map_err(|error| error.to_string())?;
            return Ok(());
        }
        let app_evidence = app_server
            .inspect_completed_turn(thread_id, &repair_turn_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "repair turn lacks persisted final-answer evidence".to_owned())?;
        coordinator
            .complete_app_server_step(
                &repair_step_id,
                &repair_turn_id,
                &app_evidence.final_item_id,
                "One bounded repair pass completed; verification will run again.",
            )
            .map_err(|error| error.to_string())?;
        app_server.shutdown().await;
        coordinator
            .transition(
                run_id,
                RunStatus::Verifying,
                None,
                "RUN_REVERIFYING",
                &json!({ "repairStepId": repair_step_id, "repairAttempt": 1 }),
            )
            .map_err(|error| error.to_string())?;

        let (argv, preset_timeout_ms) = verification_command(&input.verification_preset_id)
            .ok_or_else(|| "verification preset is unsupported".to_owned())?;
        let elapsed = started.elapsed().as_secs();
        if elapsed >= run.budget_limits.max_elapsed_seconds_per_run {
            self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_BEFORE_REVERIFY");
            return Ok(());
        }
        let remaining_ms = run
            .budget_limits
            .max_elapsed_seconds_per_run
            .saturating_sub(elapsed)
            .saturating_mul(1000);
        let verify_step_id = next_id("step");
        coordinator
            .create_step(&NewRunStep {
                step_id: verify_step_id.clone(),
                run_id: run_id.to_owned(),
                ordinal: 1,
                stage: "VERIFICATION".into(),
                droid_id: droid_id.to_owned(),
                attempt: 2,
            })
            .map_err(|error| error.to_string())?;
        coordinator
            .start_step(&verify_step_id)
            .map_err(|error| error.to_string())?;
        let mut verifier = CodexAppServerClient::start_with_policy(
            root,
            &policy.reference_project_roots,
            &policy.effective_policy,
        )
        .await
        .map_err(|error| error.to_string())?;
        let result = verifier
            .execute_command(&argv, root, preset_timeout_ms.min(remaining_ms))
            .await
            .map_err(|error| error.to_string())?;
        verifier.shutdown().await;
        let elapsed = started.elapsed().as_secs();
        let latest = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?
            .usage;
        coordinator
            .record_usage(
                run_id,
                RunBudgetUsage {
                    model_turns_started: latest.model_turns_started,
                    elapsed_seconds: elapsed,
                    reported_tokens: latest.reported_tokens,
                },
            )
            .map_err(|error| error.to_string())?;
        if elapsed >= run.budget_limits.max_elapsed_seconds_per_run {
            self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_DURING_REVERIFY");
            return Ok(());
        }
        let verification = bounded_verification_evidence(&input.verification_preset_id, &result);
        let passed = result.get("exitCode").and_then(Value::as_i64) == Some(0);
        coordinator
            .complete_step(
                &verify_step_id,
                if passed {
                    RunStepStatus::Succeeded
                } else {
                    RunStepStatus::Failed
                },
                if passed {
                    "Verification passed after repair."
                } else {
                    "Verification still fails after the single repair pass."
                },
                Some(&verification.to_string()),
            )
            .map_err(|error| error.to_string())?;
        if passed {
            coordinator
                .transition(
                    run_id,
                    RunStatus::Reviewing,
                    None,
                    "RUN_REVIEWING",
                    &json!({ "verificationStepId": verify_step_id }),
                )
                .map_err(|error| error.to_string())?;
            self.run_independent_review(run_id, input, root, policy, started, 0, &verification)
                .await?;
        } else {
            coordinator
                .transition(run_id, RunStatus::Failed, Some(RunOutcome::Failed), "RUN_FAILED", &json!({ "reason": "VERIFICATION_FAILED_AFTER_REPAIR", "verificationStepId": verify_step_id, "repairStepId": repair_step_id }))
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    async fn await_live_permission(
        &self,
        coordinator: &RunCoordinator,
        app_server: &mut CodexAppServerClient,
        run_id: &str,
        step_id: &str,
        droid_id: &str,
        thread_id: &str,
        turn_id: &str,
        rpc_request_id: Value,
        method: &str,
        params: &Value,
    ) -> Result<(), String> {
        let payload = json!({ "method": method, "params": params });
        let payload_json = serde_json::to_string(&payload).map_err(|error| error.to_string())?;
        if payload_json.len() > 32 * 1024 {
            app_server
                .respond_to_server_request(rpc_request_id, json!({ "decision": "decline" }))
                .await
                .map_err(|error| error.to_string())?;
            return Ok(());
        }
        let request_id = format!(
            "permission-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            PERMISSION_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let Some(mut receiver) = self
            .dispatcher
            .register_pending_approval(run_id, &request_id)
        else {
            app_server
                .respond_to_server_request(rpc_request_id, json!({ "decision": "decline" }))
                .await
                .map_err(|error| error.to_string())?;
            return Err("could not register the repair approval session".into());
        };
        if let Err(error) = coordinator.request_permission(&NewRunPermissionRequest {
            request_id: request_id.clone(),
            run_id: run_id.to_owned(),
            step_id: Some(step_id.to_owned()),
            droid_id: droid_id.to_owned(),
            capability: approval_capability(method).to_owned(),
            action: approval_action(method).to_owned(),
            resource: approval_resource(method, params),
            rationale:
                "Codex requested an operation outside the saved execution policy during repair."
                    .into(),
            alternative_index: 0,
            app_server_generation: None,
            thread_id: Some(thread_id.to_owned()),
            turn_id: Some(turn_id.to_owned()),
            item_id: params
                .get("itemId")
                .and_then(Value::as_str)
                .map(str::to_owned),
            json_rpc_request_id: Some(rpc_request_id.to_string()),
            app_server_approval_id: params
                .get("approvalId")
                .and_then(Value::as_str)
                .map(str::to_owned),
            payload_json,
            payload_fingerprint: permission_fingerprint(method, params),
        }) {
            self.dispatcher
                .unregister_pending_approval(run_id, &request_id);
            app_server
                .respond_to_server_request(rpc_request_id, json!({ "decision": "decline" }))
                .await
                .map_err(|response_error| response_error.to_string())?;
            return Err(error.to_string());
        }
        let decision = receiver
            .recv()
            .await
            .ok_or_else(|| "repair permission decision channel closed".to_owned())?;
        let result = if decision.granted {
            json!({ "decision": "accept" })
        } else {
            json!({ "decision": "decline" })
        };
        let response = app_server
            .respond_to_server_request_and_wait(rpc_request_id, result)
            .await;
        let acknowledged = response.is_ok()
            && coordinator
                .mark_permission_response_sent(&request_id)
                .is_ok();
        let _ = decision.acknowledge.send(acknowledged);
        self.dispatcher
            .unregister_pending_approval(run_id, &request_id);
        response.map_err(|error| error.to_string())?;
        if !acknowledged {
            return Err("repair permission acknowledgement was not persisted".into());
        }
        Ok(())
    }

    async fn run_independent_review(
        &self,
        run_id: &str,
        input: &RunExecutionInput,
        root: &PathBuf,
        policy: &RunPolicySnapshot,
        started: Instant,
        elapsed_before_run: u64,
        verification: &Value,
    ) -> Result<(), String> {
        let coordinator = RunCoordinator::new(self.db.clone());
        let run = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?;
        let Some(reviewer_droid_id) = run.droid_ids.get(1) else {
            self.block_run(run_id, "REVIEWER_DROID_MISSING");
            return Ok(());
        };
        if run.droid_ids.first() == Some(reviewer_droid_id) {
            self.block_run(run_id, "REVIEWER_MUST_BE_INDEPENDENT");
            return Ok(());
        }
        let Some(review_policy) = policy.reviewer.as_ref() else {
            self.block_run(run_id, "REVIEWER_POLICY_MISSING");
            return Ok(());
        };
        if !review_policy.effective_policy.read_project
            || !review_policy.effective_policy.write_roots.is_empty()
            || review_policy.effective_policy.web_search
            || review_policy.effective_policy.shell_network
        {
            self.block_run(run_id, "REVIEWER_POLICY_NOT_READ_ONLY");
            return Ok(());
        }
        let elapsed = elapsed_before_run.saturating_add(started.elapsed().as_secs());
        if run.usage.model_turns_started >= run.budget_limits.max_model_turns_per_run
            || run.usage.reported_tokens >= run.budget_limits.max_tokens_per_run
            || elapsed >= run.budget_limits.max_elapsed_seconds_per_run
        {
            self.block_budget_run(run_id, "BUDGET_EXHAUSTED_BEFORE_REVIEW");
            return Ok(());
        }
        let ordinal = run
            .steps
            .iter()
            .map(|step| step.ordinal)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let review_step_id = next_id("step");
        coordinator
            .create_step(&NewRunStep {
                step_id: review_step_id.clone(),
                run_id: run_id.to_owned(),
                ordinal,
                stage: "REVIEWER".into(),
                droid_id: reviewer_droid_id.clone(),
                attempt: 1,
            })
            .map_err(|error| error.to_string())?;
        coordinator
            .start_step(&review_step_id)
            .map_err(|error| error.to_string())?;
        let mut reviewer = CodexAppServerClient::start_with_policy(
            root,
            &review_policy.reference_project_roots,
            &review_policy.effective_policy,
        )
        .await
        .map_err(|error| error.to_string())?;
        let thread_id = reviewer
            .start_thread(root, "never")
            .await
            .map_err(|error| error.to_string())?;
        coordinator
            .persist_step_thread(&review_step_id, &thread_id)
            .map_err(|error| error.to_string())?;
        coordinator
            .reserve_model_turn(run_id)
            .map_err(|error| error.to_string())?;
        let prompt = format!(
            "You are the independent code reviewer for this run. This is a separate read-only session. Do not edit files or claim to run tests. Inspect the current project and compare it with the task and recorded verification evidence. Identify concrete correctness, security, regression, and completeness issues. Reply with exactly APPROVE or REJECT as the first line, followed by concise findings. Approve only when you find no blocking issue.\n\nTask title: {}\n\nTask request:\n{}\n\nVerification evidence:\n{}",
            input.task_title,
            input.prompt,
            serde_json::to_string(verification).unwrap_or_else(|_| "{}".into())
        );
        let turn_id = reviewer
            .start_turn(&thread_id, root, &prompt, None, "never")
            .await
            .map_err(|error| error.to_string())?;
        coordinator
            .persist_step_turn(
                &review_step_id,
                &turn_id,
                RunStatus::Succeeded,
                Some(RunOutcome::Succeeded),
            )
            .map_err(|error| error.to_string())?;
        let token_baseline = run.usage.reported_tokens;
        let completion = loop {
            let elapsed = elapsed_before_run.saturating_add(started.elapsed().as_secs());
            let remaining = Duration::from_secs(
                run.budget_limits
                    .max_elapsed_seconds_per_run
                    .saturating_sub(elapsed),
            );
            if remaining.is_zero() {
                let _ = reviewer.request_turn_interrupt(&thread_id, &turn_id).await;
                reviewer.shutdown().await;
                self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_DURING_REVIEW");
                return Ok(());
            }
            let message = timeout(remaining, reviewer.next_message())
                .await
                .map_err(|_| "review turn exceeded elapsed-time budget".to_owned())?
                .map_err(|error| error.to_string())?;
            if let Some(done) = turn_completion_from_message(&message, &turn_id) {
                break done;
            }
            if let Some(reported_tokens) = token_usage_from_message(&message) {
                let total_tokens = token_baseline.saturating_add(reported_tokens);
                let current = coordinator
                    .snapshot(run_id)
                    .map_err(|error| error.to_string())?;
                coordinator
                    .record_usage(
                        run_id,
                        RunBudgetUsage {
                            model_turns_started: current.usage.model_turns_started,
                            elapsed_seconds: elapsed,
                            reported_tokens: total_tokens,
                        },
                    )
                    .map_err(|error| error.to_string())?;
                if total_tokens >= run.budget_limits.max_tokens_per_run {
                    let _ = reviewer.request_turn_interrupt(&thread_id, &turn_id).await;
                    reviewer.shutdown().await;
                    self.block_budget_run(run_id, "TOKEN_BUDGET_REACHED_DURING_REVIEW");
                    return Ok(());
                }
            }
            if let super::codex_app_server::AppServerMessage::Request { request_id, .. } = message {
                reviewer
                    .respond_to_server_request(request_id, json!({ "decision": "decline" }))
                    .await
                    .map_err(|error| error.to_string())?;
            }
        };
        if completion.status != "completed" {
            reviewer.shutdown().await;
            coordinator
                .complete_step(
                    &review_step_id,
                    RunStepStatus::Failed,
                    "Independent reviewer turn did not complete.",
                    Some(
                        &json!({ "status": completion.status, "error": completion.error_message })
                            .to_string(),
                    ),
                )
                .map_err(|error| error.to_string())?;
            coordinator
                .transition(
                    run_id,
                    RunStatus::Failed,
                    Some(RunOutcome::Failed),
                    "RUN_REVIEW_FAILED",
                    &json!({ "reviewStepId": review_step_id }),
                )
                .map_err(|error| error.to_string())?;
            return Ok(());
        }
        let history = reviewer
            .inspect_completed_turn(&thread_id, &turn_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "review turn lacks persisted final-answer evidence".to_owned())?;
        reviewer.shutdown().await;
        let verdict = history
            .final_answer
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_uppercase();
        let approved = verdict == "APPROVE";
        let review_evidence = json!({
            "verdict": verdict,
            "finalItemId": history.final_item_id,
            "findings": history.final_answer.chars().take(8_000).collect::<String>(),
            "verification": verification,
        });
        coordinator
            .complete_step(
                &review_step_id,
                if approved {
                    RunStepStatus::Succeeded
                } else {
                    RunStepStatus::Failed
                },
                if approved {
                    "Independent review approved the run."
                } else {
                    "Independent review rejected or returned an invalid verdict."
                },
                Some(&review_evidence.to_string()),
            )
            .map_err(|error| error.to_string())?;
        let current = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?;
        coordinator
            .record_usage(
                run_id,
                RunBudgetUsage {
                    model_turns_started: current.usage.model_turns_started,
                    elapsed_seconds: elapsed_before_run.saturating_add(started.elapsed().as_secs()),
                    reported_tokens: current.usage.reported_tokens,
                },
            )
            .map_err(|error| error.to_string())?;
        if approved {
            coordinator
                .transition(
                    run_id,
                    RunStatus::Succeeded,
                    Some(RunOutcome::Succeeded),
                    "RUN_SUCCEEDED",
                    &json!({ "reviewStepId": review_step_id, "verification": verification }),
                )
                .map_err(|error| error.to_string())?;
        } else {
            coordinator
                .transition(
                    run_id,
                    RunStatus::Failed,
                    Some(RunOutcome::Failed),
                    "RUN_REVIEW_REJECTED",
                    &json!({ "reviewStepId": review_step_id }),
                )
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    async fn execute(&self, run_id: &str) -> Result<(), String> {
        let started = Instant::now();
        let coordinator = RunCoordinator::new(self.db.clone());
        let run = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?;
        if run.status != RunStatus::Queued {
            return Ok(());
        }
        let input = coordinator
            .execution_input(run_id)
            .map_err(|error| error.to_string())?;
        let policy: RunPolicySnapshot = serde_json::from_str(&input.policy_snapshot_json)
            .map_err(|error| format!("stored policy snapshot is invalid: {error}"))?;
        let root = self
            .db
            .project_path_for_run(&run.project_id)
            .map_err(|error| error.to_string())?;
        let root = std::fs::canonicalize(root).map_err(|error| error.to_string())?;
        let droid_id = run
            .droid_ids
            .first()
            .ok_or_else(|| "run has no assigned droid".to_owned())?;

        coordinator
            .transition(
                run_id,
                RunStatus::Planning,
                None,
                "RUN_PLANNING",
                &json!({ "worker": "codex-app-server" }),
            )
            .map_err(|error| error.to_string())?;

        let mut app_server = CodexAppServerClient::start_with_policy(
            &root,
            &policy.reference_project_roots,
            &policy.effective_policy,
        )
        .await
        .map_err(|error| error.to_string())?;
        let thread_id = app_server
            .start_thread(&root, "on-request")
            .await
            .map_err(|error| error.to_string())?;
        coordinator
            .persist_thread_id(run_id, &thread_id)
            .map_err(|error| error.to_string())?;
        coordinator
            .transition(
                run_id,
                RunStatus::Executing,
                None,
                "RUN_EXECUTING",
                &json!({ "threadId": thread_id }),
            )
            .map_err(|error| error.to_string())?;

        let step_id = next_id("step");
        coordinator
            .create_step(&NewRunStep {
                step_id: step_id.clone(),
                run_id: run_id.to_owned(),
                ordinal: 0,
                stage: "CODER".into(),
                droid_id: droid_id.clone(),
                attempt: 1,
            })
            .map_err(|error| error.to_string())?;
        coordinator
            .start_step(&step_id)
            .map_err(|error| error.to_string())?;
        coordinator
            .reserve_model_turn(run_id)
            .map_err(|error| error.to_string())?;
        let prompt = initial_task_prompt(&input.task_title, &input.prompt);
        let turn_id = app_server
            .start_turn(&thread_id, &root, &prompt, None, "on-request")
            .await
            .map_err(|error| error.to_string())?;
        coordinator
            .persist_step_turn(&step_id, &turn_id, RunStatus::Verifying, None)
            .map_err(|error| error.to_string())?;

        let completion = loop {
            let elapsed = started.elapsed();
            if elapsed >= Duration::from_secs(run.budget_limits.max_elapsed_seconds_per_run) {
                let _ = app_server
                    .request_turn_interrupt(&thread_id, &turn_id)
                    .await;
                app_server.shutdown().await;
                self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED");
                return Ok(());
            }
            let remaining = Duration::from_secs(run.budget_limits.max_elapsed_seconds_per_run)
                .saturating_sub(elapsed);
            let message = timeout(remaining, app_server.next_message())
                .await
                .map_err(|_| "turn exceeded elapsed-time budget".to_owned())?
                .map_err(|error| error.to_string())?;
            if let Some(done) = turn_completion_from_message(&message, &turn_id) {
                break done;
            }
            if let Some(reported_tokens) = token_usage_from_message(&message) {
                let current_usage = coordinator
                    .snapshot(run_id)
                    .map_err(|error| error.to_string())?
                    .usage;
                coordinator
                    .record_usage(
                        run_id,
                        RunBudgetUsage {
                            model_turns_started: current_usage.model_turns_started,
                            elapsed_seconds: started.elapsed().as_secs(),
                            reported_tokens,
                        },
                    )
                    .map_err(|error| error.to_string())?;
                if reported_tokens >= run.budget_limits.max_tokens_per_run {
                    let _ = app_server
                        .request_turn_interrupt(&thread_id, &turn_id)
                        .await;
                    app_server.shutdown().await;
                    self.block_budget_run(run_id, "TOKEN_BUDGET_REACHED");
                    return Ok(());
                }
            }
            if let super::codex_app_server::AppServerMessage::Request {
                request_id,
                method,
                params,
            } = message
            {
                if is_supported_approval_method(&method) {
                    let resource = approval_resource(&method, &params);
                    let payload = json!({ "method": method, "params": params });
                    let payload_json =
                        serde_json::to_string(&payload).map_err(|error| error.to_string())?;
                    if payload_json.len() > 32 * 1024 {
                        app_server
                            .respond_to_server_request(request_id, json!({ "decision": "decline" }))
                            .await
                            .map_err(|error| error.to_string())?;
                        continue;
                    }
                    let request_id_string = format!(
                        "permission-{}-{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos(),
                        PERMISSION_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed)
                    );
                    let fingerprint = permission_fingerprint(&method, &params);
                    let Some(mut decision_receiver) = self
                        .dispatcher
                        .register_pending_approval(run_id, &request_id_string)
                    else {
                        app_server
                            .respond_to_server_request(request_id, json!({ "decision": "decline" }))
                            .await
                            .map_err(|error| error.to_string())?;
                        continue;
                    };
                    if let Err(error) = coordinator.request_permission(&NewRunPermissionRequest {
                        request_id: request_id_string.clone(),
                        run_id: run_id.to_owned(),
                        step_id: Some(step_id.clone()),
                        droid_id: droid_id.clone(),
                        capability: approval_capability(&method).to_owned(),
                        action: approval_action(&method).to_owned(),
                        resource,
                        rationale:
                            "Codex requested an operation outside the saved execution policy."
                                .into(),
                        alternative_index: 0,
                        app_server_generation: None,
                        thread_id: Some(thread_id.clone()),
                        turn_id: Some(turn_id.clone()),
                        item_id: params
                            .get("itemId")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        json_rpc_request_id: Some(request_id.to_string()),
                        app_server_approval_id: params
                            .get("approvalId")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        payload_json,
                        payload_fingerprint: fingerprint,
                    }) {
                        self.dispatcher
                            .unregister_pending_approval(run_id, &request_id_string);
                        app_server
                            .respond_to_server_request(request_id, json!({ "decision": "decline" }))
                            .await
                            .map_err(|response_error| response_error.to_string())?;
                        return Err(error.to_string());
                    }
                    let decision = decision_receiver
                        .recv()
                        .await
                        .ok_or_else(|| "permission decision channel closed".to_owned())?;
                    let result = if decision.granted {
                        json!({ "decision": "accept" })
                    } else {
                        json!({ "decision": "decline" })
                    };
                    let response = app_server
                        .respond_to_server_request_and_wait(request_id, result)
                        .await;
                    let acknowledged = if response.is_ok() {
                        coordinator
                            .mark_permission_response_sent(&request_id_string)
                            .is_ok()
                    } else {
                        false
                    };
                    let _ = decision.acknowledge.send(acknowledged);
                    self.dispatcher
                        .unregister_pending_approval(run_id, &request_id_string);
                    response.map_err(|error| error.to_string())?;
                    if !acknowledged {
                        return Err("app-server approval acknowledgement was not persisted".into());
                    }
                } else {
                    app_server
                        .respond_to_server_request(request_id, json!({ "decision": "decline" }))
                        .await
                        .map_err(|error| error.to_string())?;
                }
            }
        };
        let current_usage = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?
            .usage;
        let elapsed_seconds = started.elapsed().as_secs();
        coordinator
            .record_usage(
                run_id,
                RunBudgetUsage {
                    model_turns_started: current_usage.model_turns_started,
                    elapsed_seconds,
                    reported_tokens: current_usage.reported_tokens,
                },
            )
            .map_err(|error| error.to_string())?;
        if elapsed_seconds >= run.budget_limits.max_elapsed_seconds_per_run {
            let _ = app_server
                .request_turn_interrupt(&thread_id, &turn_id)
                .await;
            app_server.shutdown().await;
            self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED");
            return Ok(());
        }
        if completion.status != "completed" {
            app_server.shutdown().await;
            coordinator
                .complete_step(
                    &step_id,
                    RunStepStatus::Failed,
                    "Codex turn did not complete successfully.",
                    Some(
                        &json!({ "status": completion.status, "error": completion.error_message })
                            .to_string(),
                    ),
                )
                .map_err(|error| error.to_string())?;
            coordinator
                .transition(
                    run_id,
                    RunStatus::Failed,
                    Some(RunOutcome::Failed),
                    "RUN_FAILED",
                    &json!({ "reason": "CODEX_TURN_FAILED" }),
                )
                .map_err(|error| error.to_string())?;
            return Ok(());
        }

        let evidence = app_server
            .inspect_completed_turn(&thread_id, &turn_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "completed turn lacks persisted final-answer evidence".to_owned())?;
        coordinator
            .complete_app_server_step(
                &step_id,
                &turn_id,
                &evidence.final_item_id,
                "Codex turn completed; verification follows.",
            )
            .map_err(|error| error.to_string())?;
        coordinator
            .transition(
                run_id,
                RunStatus::Verifying,
                None,
                "RUN_VERIFYING",
                &json!({ "stepId": step_id }),
            )
            .map_err(|error| error.to_string())?;

        let (argv, preset_timeout_ms) = verification_command(&input.verification_preset_id)
            .ok_or_else(|| "verification preset is not supported by this worker".to_owned())?;
        let elapsed_before_verification = started.elapsed().as_secs();
        if elapsed_before_verification >= run.budget_limits.max_elapsed_seconds_per_run {
            app_server.shutdown().await;
            self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_BEFORE_VERIFICATION");
            return Ok(());
        }
        let remaining_ms = run
            .budget_limits
            .max_elapsed_seconds_per_run
            .saturating_sub(elapsed_before_verification)
            .saturating_mul(1000);
        let timeout_ms = preset_timeout_ms.min(remaining_ms);
        let verification_step_id = next_id("step");
        coordinator
            .create_step(&NewRunStep {
                step_id: verification_step_id.clone(),
                run_id: run_id.to_owned(),
                ordinal: 1,
                stage: "VERIFICATION".into(),
                droid_id: droid_id.clone(),
                attempt: 1,
            })
            .map_err(|error| error.to_string())?;
        coordinator
            .start_step(&verification_step_id)
            .map_err(|error| error.to_string())?;
        let result = app_server
            .execute_command(&argv, &root, timeout_ms)
            .await
            .map_err(|error| error.to_string())?;
        app_server.shutdown().await;
        let elapsed_seconds = started.elapsed().as_secs();
        let latest_usage = coordinator
            .snapshot(run_id)
            .map_err(|error| error.to_string())?
            .usage;
        coordinator
            .record_usage(
                run_id,
                RunBudgetUsage {
                    model_turns_started: latest_usage.model_turns_started,
                    elapsed_seconds,
                    reported_tokens: latest_usage.reported_tokens,
                },
            )
            .map_err(|error| error.to_string())?;
        if elapsed_seconds >= run.budget_limits.max_elapsed_seconds_per_run {
            self.block_budget_run(run_id, "ELAPSED_BUDGET_EXCEEDED_DURING_VERIFICATION");
            return Ok(());
        }
        let exit_code = result.get("exitCode").and_then(Value::as_i64);
        let evidence = bounded_verification_evidence(&input.verification_preset_id, &result);
        let passed = exit_code == Some(0);
        coordinator
            .complete_step(
                &verification_step_id,
                if passed {
                    RunStepStatus::Succeeded
                } else {
                    RunStepStatus::Failed
                },
                if passed {
                    "Verification passed."
                } else {
                    "Verification failed."
                },
                Some(&evidence.to_string()),
            )
            .map_err(|error| error.to_string())?;
        if passed {
            coordinator
                .transition(
                    run_id,
                    RunStatus::Reviewing,
                    None,
                    "RUN_REVIEWING",
                    &json!({ "verificationStepId": verification_step_id }),
                )
                .map_err(|error| error.to_string())?;
            self.run_independent_review(run_id, &input, &root, &policy, started, 0, &evidence)
                .await?;
        } else {
            self.repair_after_failed_verification(
                run_id, &input, &root, &policy, &thread_id, &droid_id, started, &evidence,
            )
            .await?;
        }
        Ok(())
    }

    fn block_run(&self, run_id: &str, reason: &str) {
        let coordinator = RunCoordinator::new(self.db.clone());
        if let Ok(snapshot) = coordinator.snapshot(run_id)
            && !snapshot.status.is_terminal()
            && snapshot.status.can_transition_to(RunStatus::Blocked)
        {
            let _ = coordinator.transition(
                run_id,
                RunStatus::Blocked,
                Some(RunOutcome::Blocked),
                "RUN_BLOCKED",
                &json!({ "reason": reason }),
            );
        }
    }

    fn block_budget_run(&self, run_id: &str, reason: &str) {
        let coordinator = RunCoordinator::new(self.db.clone());
        if let Ok(snapshot) = coordinator.snapshot(run_id)
            && !snapshot.status.is_terminal()
            && snapshot.status.can_transition_to(RunStatus::Blocked)
        {
            let _ = coordinator.transition(
                run_id,
                RunStatus::Blocked,
                Some(RunOutcome::BudgetExhausted),
                "RUN_BUDGET_EXHAUSTED",
                &json!({ "reason": reason }),
            );
        }
    }
}

fn is_supported_approval_method(method: &str) -> bool {
    matches!(
        method,
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
    )
}

fn approval_capability(method: &str) -> &'static str {
    match method {
        "item/commandExecution/requestApproval" => "SHELL",
        "item/fileChange/requestApproval" => "FILESYSTEM",
        _ => "UNKNOWN",
    }
}

fn approval_action(method: &str) -> &'static str {
    match method {
        "item/commandExecution/requestApproval" => "EXECUTE_COMMAND",
        "item/fileChange/requestApproval" => "CHANGE_FILES",
        _ => "UNKNOWN",
    }
}

fn approval_resource(method: &str, params: &Value) -> String {
    let value = match method {
        "item/commandExecution/requestApproval" => json!({
            "command": params.get("command"),
            "cwd": params.get("cwd"),
            "commandActions": params.get("commandActions"),
            "networkApprovalContext": params.get("networkApprovalContext"),
        }),
        "item/fileChange/requestApproval" => json!({
            "grantRoot": params.get("grantRoot"),
            "reason": params.get("reason"),
        }),
        _ => Value::Null,
    };
    let text = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_owned());
    text.chars().take(2048).collect()
}

fn permission_fingerprint(method: &str, params: &Value) -> String {
    let action = json!({
        "method": method,
        "command": params.get("command").cloned().unwrap_or(Value::Null),
        "cwd": params.get("cwd").cloned().unwrap_or(Value::Null),
        "commandActions": params.get("commandActions").cloned().unwrap_or(Value::Null),
        "networkApprovalContext": params
            .get("networkApprovalContext")
            .cloned()
            .unwrap_or(Value::Null),
        "grantRoot": params.get("grantRoot").cloned().unwrap_or(Value::Null),
    });
    let canonical = serde_json::to_vec(&action).unwrap_or_default();
    let digest = Sha256::digest(canonical);
    format!("{digest:x}")
}

fn token_usage_from_message(message: &super::codex_app_server::AppServerMessage) -> Option<u64> {
    let super::codex_app_server::AppServerMessage::Notification { method, params } = message else {
        return None;
    };
    if method != "thread/tokenUsage/updated" {
        return None;
    }
    let usage = params.get("tokenUsage").unwrap_or(params);
    let total = usage.get("total").unwrap_or(usage);
    total
        .get("totalTokens")
        .and_then(Value::as_u64)
        .or_else(|| {
            Some(
                total
                    .get("inputTokens")?
                    .as_u64()?
                    .saturating_add(total.get("outputTokens")?.as_u64()?),
            )
        })
}

fn verification_command(preset: &str) -> Option<(Vec<String>, u64)> {
    let argv = match preset {
        "cargo-test" => vec!["cargo".into(), "test".into()],
        "npm-check" => vec!["npm".into(), "run".into(), "check".into()],
        _ => return None,
    };
    Some((argv, 10 * 60 * 1000))
}

fn initial_task_prompt(title: &str, prompt: &str) -> String {
    format!(
        "Task: {title}\n\n{prompt}\n\nMake the requested project changes. Do not claim completion until the configured verification runs."
    )
}

fn repair_prompt(
    title: &str,
    original_prompt: &str,
    verification_preset: &str,
    evidence: &Value,
) -> String {
    format!(
        "Task: {title}\n\nOriginal request:\n{original_prompt}\n\nThe single configured verification attempt `{verification_preset}` failed. Make one focused repair to address the recorded failure, do not broaden scope, and do not claim success. The Harness will rerun the same verification once.\n\nRecorded verification evidence:\n{}",
        serde_json::to_string(evidence).unwrap_or_else(|_| "{}".into())
    )
}

fn bounded_verification_evidence(preset: &str, result: &Value) -> Value {
    let text = |key| {
        result
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .chars()
            .take(2_500)
            .collect::<String>()
    };
    json!({
        "presetId": preset,
        "exitCode": result.get("exitCode"),
        "durationMs": result.get("durationMs"),
        "stdout": text("stdout"),
        "stderr": text("stderr"),
    })
}

fn next_id(prefix: &str) -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{time}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_accepts_only_server_owned_verification_presets() {
        assert!(RunWorker::supports_verification_preset("cargo-test"));
        assert!(RunWorker::supports_verification_preset("npm-check"));
        assert!(!RunWorker::supports_verification_preset("cmd /c del *"));
    }

    #[test]
    fn token_usage_parser_reads_cumulative_totals_only() {
        use super::super::codex_app_server::AppServerMessage;
        let message = AppServerMessage::Notification {
            method: "thread/tokenUsage/updated".into(),
            params: json!({
                "tokenUsage": {
                    "total": { "inputTokens": 12, "outputTokens": 7 }
                }
            }),
        };
        assert_eq!(token_usage_from_message(&message), Some(19));
    }
}
