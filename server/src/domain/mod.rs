pub mod agent;
pub mod event;
pub mod pod;
pub mod task;

pub use agent::{Agent, AgentLocation, AgentMetrics, AgentRole, AgentStatus, SpriteConfig};
pub use event::{EventTopic, WebSocketEvent};
pub use pod::{InteriorStation, Pod, ScreenStatus, VisualState};
pub use task::{DispatchPayload, DispatchTaskInput, Task, TaskPayload, TaskPriority, TaskStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_serialization_contract() {
        let agent = Agent::new(
            "agent-rust-dev",
            "Rusty",
            AgentRole::Developer,
            SpriteConfig {
                texture_key: "hacker_sprite".into(),
                skin_theme: "cyber_punk_lab".into(),
                color_accent: "#00f0ff".into(),
            },
            "pod-notex",
            AgentLocation {
                x: 12.0,
                y: 8.5,
                current_zone: "HOME".into(),
            },
        );

        let json = serde_json::to_string_pretty(&agent).expect("Falha ao serializar Agent");
        assert!(json.contains("\"role\": \"DEVELOPER\""));
        assert!(json.contains("\"status\": \"IDLE\""));
        assert!(json.contains("\"colorAccent\": \"#00f0ff\""));

        let deserialized: Agent = serde_json::from_str(&json).expect("Falha ao desserializar Agent");
        assert_eq!(agent, deserialized);
    }

    #[test]
    fn test_dispatch_payload_contract() {
        let raw_json = r#"{
            "source": "codex-cloud-runner",
            "projectId": "proj-notex-backend",
            "targetAgentId": "agent-rust-dev",
            "task": {
                "taskId": "task-8841",
                "title": "Implementar driver SQLite OPFS",
                "priority": "HIGH",
                "payload": {
                    "prompt": "Escreva a camada de persistencia...",
                    "contextFiles": ["src/db/schema.rs"]
                }
            },
            "callbackWebhook": "https://cloud.engine.io/tasks/task-8841/callback"
        }"#;

        let payload: DispatchPayload =
            serde_json::from_str(raw_json).expect("Falha ao desserializar DispatchPayload");
        assert_eq!(payload.source, "codex-cloud-runner");
        assert_eq!(payload.task.priority, TaskPriority::High);

        let task = Task::from_dispatch(&payload, "2026-10-24T12:00:00Z");
        assert_eq!(task.task_id, "task-8841");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_websocket_event_constructors() {
        let event = WebSocketEvent::task_assigned("agent-01", "pod-01", "pod-02", "task-100");
        assert_eq!(event.topic, EventTopic::TaskAssigned);
        assert!(event.timestamp > 0);
        assert_eq!(event.payload["agentId"], "agent-01");
        assert_eq!(event.payload["targetPod"], "pod-02");
    }

    #[test]
    fn test_pod_capacity_and_states() {
        let mut pod = Pod::new("pod-notex", "Notex Core", 1, None);
        assert!(pod.assign_agent("agent-1").is_ok());
        assert!(pod.assign_agent("agent-2").is_err()); // Excede capacidade

        pod.set_executing();
        assert_eq!(pod.visual_state.screen_status, ScreenStatus::CodeStream);
        assert_eq!(pod.visual_state.power_level, 100);
    }
}
