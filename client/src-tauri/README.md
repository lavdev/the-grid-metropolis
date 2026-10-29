# Tauri Desktop Wrapper

## Overview
Rust build configuration and desktop runtime for the client application.

## Interface
- `build.rs` calls `tauri_build::build` (`build.rs:main`).
- [Rust runtime](./src/README.md) exposes the library entry point `run` and owns the local backend sidecar lifecycle (`src/lib.rs:run`).

## Invariants
- The crate builds as `staticlib`, `cdylib`, and `rlib` (`Cargo.toml:[lib].crate-type`).

## Side Effects
- The build script invokes Tauri's build-time context generation (`build.rs:main`).

## Verification
- Run `cargo check` from `client/src-tauri/` (`Cargo.toml`).

