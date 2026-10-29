# Client Build Scripts

## Overview
Contains the Node.js helper used by the client package to build the server sidecar.

## Interface
- `resolveHostTarget` selects the Rust host target and the script's command-line profile selects debug or release (`build-server-sidecar.mjs:resolveHostTarget`, `build-server-sidecar.mjs:profile`).

## Invariants
- The helper accepts `release` as the release profile and defaults other values to debug (`build-server-sidecar.mjs:profile`).

## Side Effects
- Spawns Cargo to build the server and copies the resulting executable into `client/src-tauri/binaries` (`build-server-sidecar.mjs:cargoResult`, `build-server-sidecar.mjs:sidecarPath`).

## Verification
- Run `npm run build:server:debug` or `npm run build:server:release` from `client/` (scripts in `../package.json`).
