use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DroidInfo {
    pub id: String,
    pub name: String,
    pub role: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    pub plan_badge: String,
    pub avatar_type: String,
    pub status: String,
    pub status_detail: String,
    pub tokens: u64,
    pub accent_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub sender: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_model: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_modified: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetrics {
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms: u64,
    pub cpu: String,
    pub mem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineState {
    pub is_active: bool,
    pub stage: String,
    pub current_droid_id: String,
    pub prompt: String,
    pub progress_percent: u32,
    pub status_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTerminal {
    pub id: String,
    pub name: String,
    pub path: String,
    pub model: String,
    pub status: String,
    pub accent_color: String,
    pub metrics: ProjectMetrics,
    pub droids: Vec<DroidInfo>,
    pub active_droid_id: String,
    pub messages: Vec<ChatMessage>,
    pub pipeline: PipelineState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterDroid {
    pub id: String,
    pub name: String,
    pub role: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    pub plan_badge: String,
    pub avatar_type: String,
    pub accent_color: String,
    pub system_prompt: String,
    pub skills: Vec<String>,
    pub max_tokens: u64,
    pub temperature: f64,
    #[serde(default)]
    pub is_custom: bool,
}
