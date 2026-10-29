# State

## Overview

This module owns the in-memory world model and its broadcast channel. `SharedState` wraps `WorldState` in `Arc<RwLock<_>>` for sharing across Axum handlers (`world.rs::WorldState`, `world.rs::SharedState`).

## Interface

- Construction and event transport: `WorldState::new`, `subscribe`, and `broadcast`; `create_shared_state` supplies the shared wrapper (`world.rs::WorldState`, `world.rs::create_shared_state`).
- Registration and lifecycle: `register_agent`, `register_pod`, `dispatch_task`, `progress_task`, `complete_task`, and `fail_task` (`world.rs::WorldState`).
- Queries: `get_agent`, `get_pod`, `get_task`, `list_agents`, `list_pods`, and `list_tasks` (`world.rs::WorldState`).
- `seed_default_world` loads the in-code default entities; `StateError` describes state-operation failures (`world.rs::WorldState::seed_default_world`, `error.rs::StateError`).

## Invariants

- Duplicate agent, pod, and task IDs are rejected; dispatch also requires the target agent and project pod to exist (`world.rs::WorldState::register_agent`, `world.rs::WorldState::register_pod`, `world.rs::WorldState::dispatch_task`).
- Task progress, completion, and failure operations return `StateError::TaskNotFound` when the task ID is absent (`world.rs::WorldState::progress_task`, `world.rs::WorldState::complete_task`, `world.rs::WorldState::fail_task`).
- The default event broadcast capacity is 1024 (`world.rs::DEFAULT_BROADCAST_CAPACITY`).

## Side Effects

- Registration and task lifecycle operations mutate in-memory maps and publish spawn, assignment, progress, completion, or error events (`world.rs::WorldState`).
- Completion updates agent metrics/status and pod visual state; failure marks the agent and pod as errored/alerted (`world.rs::WorldState::complete_task`, `world.rs::WorldState::fail_task`).

## Verification

- Run from `server/`: `cargo test`

