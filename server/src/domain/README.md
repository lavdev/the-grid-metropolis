# Domain

## Overview

This module defines the serializable domain model used by the simulation and API: agents, pods, tasks, runs, budgets, and WebSocket events ([`domain`](mod.rs)).

## Interface

- Agent model and transitions: `Agent`, `AgentRole`, `AgentStatus`, `AgentLocation`, `AgentMetrics`, and `SpriteConfig` ([`agent`](agent.rs)).
- Pod model and visual state: `Pod`, `ScreenStatus`, `VisualState`, and `InteriorStation` ([`pod`](pod.rs)).
- Task model and dispatch input: `Task`, `TaskPayload`, `TaskPriority`, `TaskStatus`, `DispatchTaskInput`, and `DispatchPayload` ([`task`](task.rs)).
- Run and budget configuration: `RunStatus`, `RunOutcome`, `RunBudgetLimits`, `RunBudgetUsage`, and project execution settings ([`run`](run.rs)).
- Event model and constructors: `EventTopic` and `WebSocketEvent` ([`event`](event.rs)).
- The module re-exports these types from [`mod.rs`](mod.rs).

## Invariants

- Role, status, priority, and event topic enums serialize using screaming snake case; model structs use camel case ([`agent`](agent.rs), [`task`](task.rs), [`event`](event.rs), [`pod`](pod.rs)).
- `Pod::assign_agent` rejects assignment when capacity is full; pod state setters update its visual status (`pod.rs::Pod::assign_agent`, `pod.rs::Pod::set_executing`, `pod.rs::Pod::set_completed`, `pod.rs::Pod::set_alert`).
- Project budgets must be positive and fit SQLite's signed integer range; droid/run overrides may only tighten the project ceilings (`run.rs::RunBudgetLimits`).
- Task lifecycle methods set status and completion time through `mark_in_progress`, `mark_completed`, and `mark_failed` ([`Task`](task.rs)).

## Side Effects

- Domain transition methods mutate the supplied `Agent`, `Pod`, or `Task` value; event constructors create payload values and timestamps ([`Agent`](agent.rs), [`Pod`](pod.rs), [`Task`](task.rs), [`WebSocketEvent`](event.rs)).

## Verification

- Run from `server/`: `cargo test`

