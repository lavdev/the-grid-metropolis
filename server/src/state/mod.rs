pub mod error;
pub mod world;

pub use error::StateError;
pub use world::{DEFAULT_BROADCAST_CAPACITY, SharedState, WorldState, create_shared_state};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Agent, AgentLocation, AgentRole, AgentStatus, DispatchPayload, DispatchTaskInput, Pod,
        ScreenStatus, SpriteConfig, TaskPayload, TaskPriority,
    };

    fn make_test_agent(id: &str) -> Agent {
        Agent::new(
            id,
            "Test Agent",
            AgentRole::Developer,
            SpriteConfig {
                texture_key: "sprite".into(),
                skin_theme: "theme".into(),
                color_accent: "#ffffff".into(),
            },
            "pod-01",
            AgentLocation {
                x: 0.0,
                y: 0.0,
                current_zone: "TEST".into(),
            },
        )
    }

    #[tokio::test]
    async fn test_state_registration_and_queries() {
        let mut world = WorldState::new(32);
        let agent = make_test_agent("agent-01");
        let pod = Pod::new("pod-01", "Pod 1", 2, None);

        assert!(world.register_agent(agent).is_ok());
        assert!(world.register_pod(pod).is_ok());

        assert_eq!(world.list_agents().len(), 1);
        assert_eq!(world.list_pods().len(), 1);
        assert!(world.get_agent("agent-01").is_some());
        assert!(world.get_pod("pod-01").is_some());
    }

    #[tokio::test]
    async fn test_dispatch_lifecycle_and_broadcast() {
        let mut world = WorldState::new(32);
        let mut rx = world.subscribe();

        let agent = make_test_agent("agent-01");
        let pod = Pod::new("proj-01", "Project 1", 2, None);

        world.register_agent(agent).unwrap();
        world.register_pod(pod).unwrap();

        // Limpa o evento AGENT_SPAWN
        let _ = rx.recv().await.unwrap();

        let dispatch = DispatchPayload {
            source: "test-runner".into(),
            project_id: "proj-01".into(),
            target_agent_id: "agent-01".into(),
            task: DispatchTaskInput {
                task_id: "task-01".into(),
                title: "Test Task".into(),
                priority: TaskPriority::High,
                payload: TaskPayload::default(),
            },
            callback_webhook: None,
        };

        // 1. Dispatch
        let task = world.dispatch_task(dispatch).expect("Falha ao despachar");
        assert_eq!(task.task_id, "task-01");

        // Verifica evento TASK_ASSIGNED recebido no broadcast
        let event = rx.recv().await.unwrap();
        assert_eq!(event.topic, crate::domain::EventTopic::TaskAssigned);

        let agent = world.get_agent("agent-01").unwrap();
        assert_eq!(agent.status, AgentStatus::Walking);

        let pod = world.get_pod("proj-01").unwrap();
        assert_eq!(pod.visual_state.screen_status, ScreenStatus::CodeStream);

        // 2. Complete Task
        world
            .complete_task("task-01", 500, vec!["artifact.rs".into()])
            .expect("Falha ao completar");

        let event = rx.recv().await.unwrap();
        assert_eq!(event.topic, crate::domain::EventTopic::TaskCompleted);

        let agent = world.get_agent("agent-01").unwrap();
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.metrics.tokens_consumed, 500);
        assert_eq!(agent.metrics.tasks_completed, 1);
    }
}
