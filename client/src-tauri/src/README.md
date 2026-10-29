# Tauri Runtime

## Overview
Rust entry point and desktop runtime setup for the Tauri application.

## Interface
- `main` delegates to `the_grid_metropolis_lib::run` (`main.rs:main`).
- `run` configures the shell plugin, local backend process, application state, and exit handling (`lib.rs:run`).
- `BackendProcess` stores the optional managed sidecar child (`lib.rs:BackendProcess`).

## Invariants
- Before launching its sidecar, `run` checks whether the local backend health endpoint is ready (`lib.rs:run`, `lib.rs:backend_is_ready`).
- The managed child is taken and killed on `RunEvent::Exit` when present (`lib.rs:run`).

## Side Effects
- Creates the app data directory, probes `127.0.0.1:3000/health`, may spawn the `grid-metropolis-server` sidecar, and kills that child on application exit (`lib.rs:run`, `lib.rs:backend_is_ready`).

## Verification
- Run `cargo check` from `client/src-tauri/` (manifest: `../Cargo.toml`).
