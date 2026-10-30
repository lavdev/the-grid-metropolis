use crate::{
    db::{Database, sqlite::RunStoreError},
    domain::run::{
        NewRunPermissionRequest, NewRunRecord, NewRunStep, RecoveryDecision, RecoveryEvidence,
        RunBudgetUsage, RunEvent, RunExecutionInput, RunOutcome, RunPermissionRequest, RunSnapshot,
        RunStatus, RunStep, RunStepStatus,
    },
};

#[derive(Clone)]
pub struct RunCoordinator {
    db: Database,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunEventPage {
    pub events: Vec<RunEvent>,
    pub next_sequence: u64,
    pub has_more: bool,
}

impl RunCoordinator {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, run: &NewRunRecord) -> Result<RunSnapshot, RunStoreError> {
        self.db.create_run_record(run)
    }

    pub fn snapshot(&self, run_id: &str) -> Result<RunSnapshot, RunStoreError> {
        self.db.get_run_snapshot(run_id)
    }

    pub fn execution_input(&self, run_id: &str) -> Result<RunExecutionInput, RunStoreError> {
        self.db.get_run_execution_input(run_id)
    }

    pub fn reserve_model_turn(&self, run_id: &str) -> Result<RunSnapshot, RunStoreError> {
        self.db.reserve_run_model_turn(run_id)
    }

    pub fn record_usage(
        &self,
        run_id: &str,
        usage: RunBudgetUsage,
    ) -> Result<RunSnapshot, RunStoreError> {
        self.db.record_run_usage(run_id, usage)
    }

    pub fn nonterminal_runs(&self) -> Result<Vec<RunSnapshot>, RunStoreError> {
        self.db.list_nonterminal_runs()
    }

    pub fn persist_thread_id(
        &self,
        run_id: &str,
        thread_id: &str,
    ) -> Result<RunSnapshot, RunStoreError> {
        self.db.persist_run_thread_id(run_id, thread_id)
    }

    pub fn transition(
        &self,
        run_id: &str,
        next_status: RunStatus,
        terminal_outcome: Option<RunOutcome>,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<RunSnapshot, RunStoreError> {
        self.db
            .transition_run(run_id, next_status, terminal_outcome, event_type, payload)
    }

    pub fn create_step(&self, step: &NewRunStep) -> Result<RunStep, RunStoreError> {
        self.db.create_run_step(step)
    }

    pub fn start_step(&self, step_id: &str) -> Result<RunStep, RunStoreError> {
        self.db.start_run_step(step_id)
    }

    pub fn persist_step_thread(
        &self,
        step_id: &str,
        thread_id: &str,
    ) -> Result<RunStep, RunStoreError> {
        self.db.persist_run_step_thread_id(step_id, thread_id)
    }

    pub fn persist_step_turn(
        &self,
        step_id: &str,
        turn_id: &str,
        next_status: RunStatus,
        terminal_outcome: Option<RunOutcome>,
    ) -> Result<RunStep, RunStoreError> {
        self.db
            .persist_run_step_turn(step_id, turn_id, next_status, terminal_outcome)
    }

    pub fn complete_app_server_step(
        &self,
        step_id: &str,
        turn_id: &str,
        item_id: &str,
        summary: &str,
    ) -> Result<RunStep, RunStoreError> {
        self.db
            .complete_app_server_run_step(step_id, turn_id, item_id, summary)
    }

    pub fn complete_step(
        &self,
        step_id: &str,
        status: RunStepStatus,
        summary: &str,
        verification_json: Option<&str>,
    ) -> Result<RunStep, RunStoreError> {
        self.db
            .complete_run_step(step_id, status, summary, verification_json)
    }

    pub fn request_permission(
        &self,
        request: &NewRunPermissionRequest,
    ) -> Result<RunPermissionRequest, RunStoreError> {
        self.db.create_pending_permission(request)
    }

    pub fn refresh_unsent_permission(
        &self,
        request_id: &str,
        thread_id: &str,
        turn_id: &str,
        item_id: Option<&str>,
        json_rpc_request_id: &str,
        app_server_approval_id: Option<&str>,
        resumed_fingerprint: &str,
    ) -> Result<RunPermissionRequest, RunStoreError> {
        self.db.refresh_unsent_permission_request(
            request_id,
            thread_id,
            turn_id,
            item_id,
            json_rpc_request_id,
            app_server_approval_id,
            resumed_fingerprint,
        )
    }

    pub fn decide_permission(
        &self,
        request_id: &str,
        granted: bool,
        idempotency_key: &str,
        grant_scope: Option<&str>,
    ) -> Result<(RunPermissionRequest, RunSnapshot), RunStoreError> {
        self.db
            .decide_run_permission(request_id, granted, idempotency_key, grant_scope)
    }

    pub fn permission_request(
        &self,
        request_id: &str,
    ) -> Result<Option<RunPermissionRequest>, RunStoreError> {
        self.db.get_permission_request(request_id)
    }

    pub fn unresolved_permissions(
        &self,
        run_id: &str,
    ) -> Result<Vec<RunPermissionRequest>, RunStoreError> {
        self.db.list_unresolved_permission_requests(run_id)
    }

    pub fn mark_permission_response_sent(
        &self,
        request_id: &str,
    ) -> Result<RunPermissionRequest, RunStoreError> {
        self.db.mark_permission_response_sent(request_id)
    }

    pub fn replay_events(
        &self,
        run_id: &str,
        after_sequence: u64,
        limit: usize,
    ) -> Result<RunEventPage, RunStoreError> {
        if limit == 0 || limit > 500 {
            return Err(RunStoreError::InvalidRequest(
                "run event page size must be between 1 and 500".into(),
            ));
        }
        let mut events = self
            .db
            .get_run_events(run_id, after_sequence, limit.saturating_add(1))?;
        let has_more = events.len() > limit;
        if has_more {
            events.pop();
        }
        let next_sequence = events.last().map_or(after_sequence, |event| event.sequence);
        Ok(RunEventPage {
            events,
            next_sequence,
            has_more,
        })
    }

    pub fn recover(
        &self,
        run_id: &str,
        evidence: &RecoveryEvidence,
    ) -> Result<(RecoveryDecision, RunSnapshot), RunStoreError> {
        self.db.recover_run(run_id, evidence)
    }
}
