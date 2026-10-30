# Server Source

## Overview

This directory contains the `server` library entry point and binary entry point. The library exposes the API, database, domain, and state modules; the binary wires the HTTP server and SQLite database ([`lib`](lib.rs), `main.rs::main`).

## Interface

- [`api`](api/README.md) — Axum routes and handlers for bridge dispatch, Codex integration, project operations, execution budgets and access policy, and WebSocket streaming; [`openapi.yaml`](../openapi.yaml) documents the implemented API contract.
- [`db`](db/README.md) — SQLite persistence and serialized project data models.
- [`domain`](domain/README.md) — shared agent, pod, task, event, and execution-policy types.
- [`run`](run/README.md) — durable run coordinator, event replay, recovery decisions, and execution-budget evaluation.
- [`state`](state/README.md) — in-memory world state, task lifecycle operations, and event broadcast.
- The crate publicly exports the API, database, domain, run-service, and state modules from [`lib.rs`](lib.rs).

## Invariants

- The library exposes transport, persistence, domain, run-service, and simulation-state boundaries declared in `lib.rs` ([`lib`](lib.rs)).
- The binary initializes the database and shared state before composing and serving the router (`main.rs::main`).

## Side Effects

- The binary initializes SQLite, binds a TCP listener, and serves HTTP (`main.rs::main`).

## Verification

- Run from `server/`: `cargo test`

