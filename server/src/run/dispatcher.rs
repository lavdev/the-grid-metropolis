use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::{mpsc, oneshot};

const DECISION_ACK_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Default)]
pub struct RunDispatcher {
    pending_approvals: Arc<Mutex<HashMap<ApprovalKey, mpsc::UnboundedSender<ApprovalDecision>>>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ApprovalKey {
    run_id: String,
    request_id: String,
}

pub struct ApprovalDecision {
    pub granted: bool,
    pub acknowledge: oneshot::Sender<bool>,
}

impl RunDispatcher {
    /// Registers an approval owned by a live app-server session.
    ///
    /// The returned receiver is held by that session while it waits for the UI
    /// decision. Only one live request with a given run/request ID is permitted.
    pub fn register_pending_approval(
        &self,
        run_id: &str,
        request_id: &str,
    ) -> Option<mpsc::UnboundedReceiver<ApprovalDecision>> {
        let key = ApprovalKey {
            run_id: run_id.to_owned(),
            request_id: request_id.to_owned(),
        };
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut approvals = self.pending_approvals.lock().ok()?;
        if approvals.contains_key(&key) {
            return None;
        }
        approvals.insert(key, sender);
        Some(receiver)
    }

    /// Sends the persisted user choice to an active session and waits for its
    /// confirmation that app-server resolved the JSON-RPC request.
    pub async fn deliver_decision(&self, run_id: &str, request_id: &str, granted: bool) -> bool {
        let key = ApprovalKey {
            run_id: run_id.to_owned(),
            request_id: request_id.to_owned(),
        };
        let Some(sender) = self
            .pending_approvals
            .lock()
            .ok()
            .and_then(|approvals| approvals.get(&key).cloned())
        else {
            return false;
        };
        let (acknowledge, acknowledged) = oneshot::channel();
        if sender
            .send(ApprovalDecision {
                granted,
                acknowledge,
            })
            .is_err()
        {
            return false;
        }
        matches!(
            tokio::time::timeout(DECISION_ACK_TIMEOUT, acknowledged).await,
            Ok(Ok(true))
        )
    }

    pub fn unregister_pending_approval(&self, run_id: &str, request_id: &str) {
        let key = ApprovalKey {
            run_id: run_id.to_owned(),
            request_id: request_id.to_owned(),
        };
        if let Ok(mut approvals) = self.pending_approvals.lock() {
            approvals.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn decision_is_delivered_only_to_the_matching_active_session() {
        let dispatcher = RunDispatcher::default();
        assert!(
            !dispatcher
                .deliver_decision("run-1", "approval-1", true)
                .await
        );

        let mut receiver = dispatcher
            .register_pending_approval("run-1", "approval-1")
            .expect("register live approval");
        assert!(
            dispatcher
                .register_pending_approval("run-1", "approval-1")
                .is_none()
        );

        let delivery = tokio::spawn({
            let dispatcher = dispatcher.clone();
            async move {
                dispatcher
                    .deliver_decision("run-1", "approval-1", true)
                    .await
            }
        });
        let decision = receiver.recv().await.expect("receive persisted decision");
        assert!(decision.granted);
        assert!(
            !dispatcher
                .deliver_decision("run-2", "approval-1", false)
                .await
        );
        decision
            .acknowledge
            .send(true)
            .expect("acknowledge app-server result");
        assert!(delivery.await.expect("decision delivery task"));

        dispatcher.unregister_pending_approval("run-1", "approval-1");
        assert!(
            !dispatcher
                .deliver_decision("run-1", "approval-1", true)
                .await
        );
    }
}
