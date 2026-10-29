# Client Source

## Overview
Svelte application entry point and presentation, library, and type packages.

## Interface
- `main.ts` mounts `App` at the element with id `app` (`main.ts:app`). `App.svelte` loads projects, owns the `GridWebSocketClient` lifecycle, and sends incoming events to `handleIncomingEvent` (`App.svelte:onMount`, `App.svelte:reconnectBridge`).
- Child contracts: [components](./components/README.md), [libraries](./lib/README.md), and [types](./types/README.md).

## Invariants
- `App.svelte` unsubscribes both listeners and disconnects its WebSocket client in the `onMount` cleanup (`App.svelte:onMount`).

## Side Effects
- Mounts the Svelte application, loads project state, establishes a WebSocket connection, and updates reactive grid state from received events (`main.ts:app`, `App.svelte:onMount`, `gridStore.ts:handleIncomingEvent`).

## Verification
- Run `npm run check` from `client/`.
