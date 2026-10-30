use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::timeout,
};

use crate::{
    domain::execution_policy::EffectiveExecutionPolicy,
    run::policy::{CodexProfileError, codex_profile_config_args},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RECOVERY_HISTORY_PAGES: usize = 10;

#[derive(Debug, thiserror::Error)]
pub enum AppServerError {
    #[error("could not start Codex app-server: {0}")]
    Spawn(String),
    #[error("Codex app-server stdio is unavailable")]
    MissingStdio,
    #[error("Codex app-server request {0} timed out")]
    Timeout(&'static str),
    #[error("Codex app-server exited before replying to {0}")]
    Closed(&'static str),
    #[error("Codex app-server returned an invalid JSON-RPC response")]
    InvalidResponse,
    #[error("Codex app-server rejected {method}: {message}")]
    Rpc {
        method: &'static str,
        message: String,
    },
    #[error("Codex app-server resumed a different thread")]
    ThreadMismatch,
    #[error("could not construct a restrictive Codex permission profile: {0}")]
    InvalidExecutionProfile(#[from] CodexProfileError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedTurnEvidence {
    pub turn_id: String,
    pub final_item_id: String,
    pub final_answer: String,
    pub completed_operation_item_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppServerMessage {
    Notification {
        method: String,
        params: Value,
    },
    Request {
        request_id: Value,
        method: String,
        params: Value,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCompletion {
    pub turn_id: String,
    pub status: String,
    pub error_message: Option<String>,
}

/// Small stdio client for local app-server JSON-RPC transport.
/// Run orchestration owns when its thread and turn methods may be called.
pub struct CodexAppServerClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    queued_messages: VecDeque<AppServerMessage>,
}

impl CodexAppServerClient {
    pub async fn start(working_directory: &Path) -> Result<Self, AppServerError> {
        let restricted = EffectiveExecutionPolicy {
            read_project: false,
            write_roots: Vec::new(),
            reference_project_ids: Vec::new(),
            web_search: false,
            shell_network: false,
        };
        Self::start_with_policy(working_directory, &[], &restricted).await
    }

    pub async fn start_with_policy(
        working_directory: &Path,
        reference_project_roots: &[PathBuf],
        policy: &EffectiveExecutionPolicy,
    ) -> Result<Self, AppServerError> {
        let executable =
            std::env::var_os("GRID_METROPOLIS_CODEX_COMMAND").unwrap_or_else(|| "codex".into());
        let config_args =
            codex_profile_config_args(Some(working_directory), reference_project_roots, policy)?;
        let mut child = Command::new(executable)
            .args(config_args)
            .args(["app-server", "--stdio"])
            .current_dir(working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| AppServerError::Spawn(error.to_string()))?;
        let stdin = child.stdin.take().ok_or(AppServerError::MissingStdio)?;
        let stdout = child.stdout.take().ok_or(AppServerError::MissingStdio)?;
        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            queued_messages: VecDeque::new(),
        };

        client
            .request(
                "initialize",
                json!({
                    "clientInfo": {
                        "name": "the-grid-metropolis",
                        "title": "The Grid Metropolis",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {}
                }),
            )
            .await?;
        client.notify("initialized", json!({})).await?;
        Ok(client)
    }

    pub async fn resume_thread(
        &mut self,
        thread_id: &str,
        working_directory: &Path,
    ) -> Result<(), AppServerError> {
        let result = self
            .request(
                "thread/resume",
                json!({
                    "threadId": thread_id,
                    "cwd": working_directory.to_string_lossy(),
                    "excludeTurns": true
                }),
            )
            .await?;
        let resumed_id = result
            .pointer("/thread/id")
            .and_then(Value::as_str)
            .ok_or(AppServerError::InvalidResponse)?;
        if resumed_id != thread_id {
            return Err(AppServerError::ThreadMismatch);
        }
        Ok(())
    }

    pub async fn start_thread(
        &mut self,
        working_directory: &Path,
        approval_policy: &str,
    ) -> Result<String, AppServerError> {
        let result = self
            .request(
                "thread/start",
                json!({
                    "cwd": working_directory.to_string_lossy(),
                    "approvalPolicy": approval_policy
                }),
            )
            .await?;
        result
            .pointer("/thread/id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(AppServerError::InvalidResponse)
    }

    pub async fn start_turn(
        &mut self,
        thread_id: &str,
        working_directory: &Path,
        prompt: &str,
        sandbox_policy: Option<Value>,
        approval_policy: &str,
    ) -> Result<String, AppServerError> {
        let mut params = json!({
            "threadId": thread_id,
            "cwd": working_directory.to_string_lossy(),
            "input": [{ "type": "text", "text": prompt }],
            "approvalPolicy": approval_policy
        });
        if let Some(sandbox_policy) = sandbox_policy {
            params["sandboxPolicy"] = sandbox_policy;
        }
        let result = self.request("turn/start", params).await?;
        result
            .pointer("/turn/id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(AppServerError::InvalidResponse)
    }

    /// Run a server-owned argv under the app-server process's active permission profile.
    /// The caller must resolve the executable and arguments from trusted configuration.
    pub async fn execute_command(
        &mut self,
        argv: &[String],
        working_directory: &Path,
        timeout_ms: u64,
    ) -> Result<Value, AppServerError> {
        if argv.is_empty() || timeout_ms == 0 {
            return Err(AppServerError::InvalidResponse);
        }
        self.request(
            "command/exec",
            json!({
                "command": argv,
                "cwd": working_directory.to_string_lossy(),
                "timeoutMs": timeout_ms,
                "streamStdoutStderr": false
            }),
        )
        .await
    }

    pub async fn request_turn_interrupt(
        &mut self,
        thread_id: &str,
        turn_id: &str,
    ) -> Result<(), AppServerError> {
        self.request(
            "turn/interrupt",
            json!({ "threadId": thread_id, "turnId": turn_id }),
        )
        .await?;
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<AppServerMessage, AppServerError> {
        if let Some(message) = self.queued_messages.pop_front() {
            return Ok(message);
        }
        loop {
            let response = self.read_message("event stream").await?;
            if let Some(method) = response.get("method").and_then(Value::as_str) {
                let params = response.get("params").cloned().unwrap_or_else(|| json!({}));
                if let Some(request_id) = response.get("id").cloned() {
                    return Ok(AppServerMessage::Request {
                        request_id,
                        method: method.to_owned(),
                        params,
                    });
                }
                return Ok(AppServerMessage::Notification {
                    method: method.to_owned(),
                    params,
                });
            }
        }
    }

    pub async fn respond_to_server_request(
        &mut self,
        request_id: Value,
        result: Value,
    ) -> Result<(), AppServerError> {
        let message = json!({ "jsonrpc": "2.0", "id": request_id, "result": result });
        let mut bytes =
            serde_json::to_vec(&message).map_err(|_| AppServerError::InvalidResponse)?;
        bytes.push(b'\n');
        self.stdin
            .write_all(&bytes)
            .await
            .map_err(|_| AppServerError::Closed("server response"))?;
        self.stdin
            .flush()
            .await
            .map_err(|_| AppServerError::Closed("server response"))
    }

    pub async fn respond_to_server_request_and_wait(
        &mut self,
        request_id: Value,
        result: Value,
    ) -> Result<(), AppServerError> {
        let expected_id = request_id.clone();
        self.respond_to_server_request(request_id, result).await?;
        loop {
            let response = timeout(
                REQUEST_TIMEOUT,
                self.read_message("server request resolution"),
            )
            .await
            .map_err(|_| AppServerError::Timeout("server request resolution"))??;
            let Some(method) = response.get("method").and_then(Value::as_str) else {
                continue;
            };
            let params = response.get("params").cloned().unwrap_or_else(|| json!({}));
            let message = if let Some(request_id) = response.get("id").cloned() {
                AppServerMessage::Request {
                    request_id,
                    method: method.to_owned(),
                    params,
                }
            } else {
                AppServerMessage::Notification {
                    method: method.to_owned(),
                    params,
                }
            };
            if let AppServerMessage::Notification { method, params } = &message
                && method == "serverRequest/resolved"
                && params.get("requestId") == Some(&expected_id)
            {
                return Ok(());
            }
            self.queued_messages.push_back(message);
        }
    }

    pub async fn inspect_completed_turn(
        &mut self,
        thread_id: &str,
        turn_id: &str,
    ) -> Result<Option<CompletedTurnEvidence>, AppServerError> {
        let mut turns_cursor: Option<String> = None;
        let mut matching_turn = None;
        for _ in 0..MAX_RECOVERY_HISTORY_PAGES {
            let mut params = json!({
                "threadId": thread_id,
                "limit": 100,
                "sortDirection": "desc",
                "itemsView": "notLoaded"
            });
            if let Some(cursor) = turns_cursor.as_deref() {
                params["cursor"] = json!(cursor);
            }
            let page = self.request("thread/turns/list", params).await?;
            matching_turn = page.get("data").and_then(Value::as_array).and_then(|data| {
                data.iter()
                    .find(|turn| turn.get("id").and_then(Value::as_str) == Some(turn_id))
                    .cloned()
            });
            if matching_turn.is_some() {
                break;
            }
            turns_cursor = page
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_owned);
            if turns_cursor.is_none() {
                break;
            }
        }
        let Some(turn) = matching_turn else {
            return Ok(None);
        };
        if turn.get("status").and_then(Value::as_str) != Some("completed") {
            return Ok(None);
        }

        let mut items_cursor: Option<String> = None;
        let mut entries = Vec::new();
        let mut items_history_complete = false;
        for _ in 0..MAX_RECOVERY_HISTORY_PAGES {
            let mut params = json!({
                "threadId": thread_id,
                "turnId": turn_id,
                "limit": 500,
                "sortDirection": "desc"
            });
            if let Some(cursor) = items_cursor.as_deref() {
                params["cursor"] = json!(cursor);
            }
            let page = self.request("thread/items/list", params).await?;
            if let Some(page_entries) = page.get("data").and_then(Value::as_array) {
                entries.extend(page_entries.iter().cloned());
            } else {
                return Ok(None);
            }
            items_cursor = page
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_owned);
            if items_cursor.is_none() {
                items_history_complete = true;
                break;
            }
        }
        if !items_history_complete {
            return Ok(None);
        }
        let completed_operation_item_ids = entries
            .iter()
            .filter_map(|entry| {
                let item = entry.get("item")?;
                let operation_type = matches!(
                    item.get("type").and_then(Value::as_str),
                    Some("commandExecution" | "fileChange")
                );
                let completed = item.get("status").and_then(Value::as_str) == Some("completed")
                    && entry
                        .get("completedAt")
                        .is_some_and(|value| !value.is_null());
                (operation_type && completed)
                    .then(|| item.get("id").and_then(Value::as_str).map(str::to_owned))
                    .flatten()
            })
            .collect::<Vec<_>>();
        let final_answer = entries.iter().find_map(|entry| {
            let item = entry.get("item")?;
            let final_answer = item.get("type").and_then(Value::as_str) == Some("agentMessage")
                && item.get("phase").and_then(Value::as_str) == Some("final_answer");
            let has_completion_timestamp = entry
                .get("completedAt")
                .is_some_and(|value| !value.is_null());
            if final_answer && has_completion_timestamp {
                Some((
                    item.get("id")?.as_str()?.to_owned(),
                    extract_final_answer_text(item),
                ))
            } else {
                None
            }
        });
        Ok(
            final_answer.map(|(final_item_id, final_answer)| CompletedTurnEvidence {
                turn_id: turn_id.to_owned(),
                final_item_id,
                final_answer,
                completed_operation_item_ids,
            }),
        )
    }

    pub async fn can_reissue_pending_approval(
        &mut self,
        thread_id: &str,
        turn_id: &str,
        pending_item_id: &str,
    ) -> Result<bool, AppServerError> {
        let turn_page = self
            .request(
                "thread/turns/list",
                json!({
                    "threadId": thread_id,
                    "limit": 100,
                    "sortDirection": "desc",
                    "itemsView": "notLoaded"
                }),
            )
            .await?;
        let turn = turn_page
            .get("data")
            .and_then(Value::as_array)
            .and_then(|turns| {
                turns
                    .iter()
                    .find(|turn| turn.get("id").and_then(Value::as_str) == Some(turn_id))
            });
        if turn.is_none_or(|turn| turn.get("status").and_then(Value::as_str) == Some("completed")) {
            return Ok(false);
        }

        let mut cursor: Option<String> = None;
        let mut found_pending_item = false;
        let mut history_complete = false;
        for _ in 0..MAX_RECOVERY_HISTORY_PAGES {
            let mut params = json!({
                "threadId": thread_id,
                "turnId": turn_id,
                "limit": 500,
                "sortDirection": "desc"
            });
            if let Some(cursor) = cursor.as_deref() {
                params["cursor"] = json!(cursor);
            }
            let page = self.request("thread/items/list", params).await?;
            let Some(entries) = page.get("data").and_then(Value::as_array) else {
                return Ok(false);
            };
            for entry in entries {
                let Some(item) = entry.get("item") else {
                    continue;
                };
                if item.get("id").and_then(Value::as_str) == Some(pending_item_id) {
                    found_pending_item = true;
                }
                let mutating = matches!(
                    item.get("type").and_then(Value::as_str),
                    Some("commandExecution" | "fileChange")
                );
                let completed = item.get("status").and_then(Value::as_str) == Some("completed")
                    && entry
                        .get("completedAt")
                        .is_some_and(|value| !value.is_null());
                if mutating && completed {
                    return Ok(false);
                }
            }
            cursor = page
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_owned);
            if cursor.is_none() {
                history_complete = true;
                break;
            }
        }
        Ok(history_complete && found_pending_item)
    }

    pub async fn shutdown(mut self) {
        drop(self.stdin);
        if timeout(Duration::from_secs(2), self.child.wait())
            .await
            .is_err()
        {
            let _ = self.child.kill().await;
        }
    }

    async fn notify(&mut self, method: &'static str, params: Value) -> Result<(), AppServerError> {
        let message = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        let mut bytes =
            serde_json::to_vec(&message).map_err(|_| AppServerError::InvalidResponse)?;
        bytes.push(b'\n');
        self.stdin
            .write_all(&bytes)
            .await
            .map_err(|_| AppServerError::Closed(method))?;
        self.stdin
            .flush()
            .await
            .map_err(|_| AppServerError::Closed(method))
    }

    async fn request(
        &mut self,
        method: &'static str,
        params: Value,
    ) -> Result<Value, AppServerError> {
        let request_id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let message = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        });
        let mut bytes =
            serde_json::to_vec(&message).map_err(|_| AppServerError::InvalidResponse)?;
        bytes.push(b'\n');
        self.stdin
            .write_all(&bytes)
            .await
            .map_err(|_| AppServerError::Closed(method))?;
        self.stdin
            .flush()
            .await
            .map_err(|_| AppServerError::Closed(method))?;

        let read_response = async {
            loop {
                let response = self.read_message(method).await?;
                if let Some(incoming_method) = response.get("method").and_then(Value::as_str) {
                    let params = response.get("params").cloned().unwrap_or_else(|| json!({}));
                    let incoming = if let Some(server_request_id) = response.get("id").cloned() {
                        AppServerMessage::Request {
                            request_id: server_request_id,
                            method: incoming_method.to_owned(),
                            params,
                        }
                    } else {
                        AppServerMessage::Notification {
                            method: incoming_method.to_owned(),
                            params,
                        }
                    };
                    self.queued_messages.push_back(incoming);
                    continue;
                }
                if response.get("id").and_then(Value::as_u64) != Some(request_id) {
                    continue;
                }
                if let Some(error) = response.get("error") {
                    return Err(AppServerError::Rpc {
                        method,
                        message: error
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown JSON-RPC error")
                            .to_owned(),
                    });
                }
                return response
                    .get("result")
                    .cloned()
                    .ok_or(AppServerError::InvalidResponse);
            }
        };
        timeout(REQUEST_TIMEOUT, read_response)
            .await
            .map_err(|_| AppServerError::Timeout(method))?
    }

    async fn read_message(&mut self, method: &'static str) -> Result<Value, AppServerError> {
        let mut line = String::new();
        let count = self
            .stdout
            .read_line(&mut line)
            .await
            .map_err(|_| AppServerError::Closed(method))?;
        if count == 0 {
            return Err(AppServerError::Closed(method));
        }
        serde_json::from_str(&line).map_err(|_| AppServerError::InvalidResponse)
    }
}

pub fn turn_completion_from_message(
    message: &AppServerMessage,
    expected_turn_id: &str,
) -> Option<TurnCompletion> {
    let AppServerMessage::Notification { method, params } = message else {
        return None;
    };
    if method != "turn/completed" {
        return None;
    }
    let turn = params.get("turn")?;
    let turn_id = turn.get("id")?.as_str()?;
    if turn_id != expected_turn_id {
        return None;
    }
    let status = turn.get("status")?.as_str()?.to_owned();
    let error_message = turn
        .pointer("/error/message")
        .and_then(Value::as_str)
        .map(str::to_owned);
    Some(TurnCompletion {
        turn_id: turn_id.to_owned(),
        status,
        error_message,
    })
}

fn extract_final_answer_text(item: &Value) -> String {
    let mut chunks = Vec::new();
    if let Some(text) = item.get("text").and_then(Value::as_str) {
        chunks.push(text.to_owned());
    }
    if let Some(content) = item.get("content") {
        match content {
            Value::String(text) => chunks.push(text.clone()),
            Value::Array(parts) => {
                for part in parts {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        chunks.push(text.to_owned());
                    }
                }
            }
            _ => {}
        }
    }
    chunks.join("\n").chars().take(8_000).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_completion_parser_matches_id_and_keeps_failure_details() {
        let message = AppServerMessage::Notification {
            method: "turn/completed".into(),
            params: json!({
                "turn": {
                    "id": "turn-42",
                    "status": "failed",
                    "error": { "message": "model request failed" }
                }
            }),
        };

        assert_eq!(turn_completion_from_message(&message, "turn-other"), None);
        assert_eq!(
            turn_completion_from_message(&message, "turn-42"),
            Some(TurnCompletion {
                turn_id: "turn-42".into(),
                status: "failed".into(),
                error_message: Some("model request failed".into()),
            })
        );
    }

    #[test]
    fn turn_completion_parser_does_not_consume_server_requests() {
        let message = AppServerMessage::Request {
            request_id: json!(7),
            method: "item/commandExecution/requestApproval".into(),
            params: json!({ "threadId": "thread-1" }),
        };
        assert_eq!(turn_completion_from_message(&message, "turn-1"), None);
    }
}
