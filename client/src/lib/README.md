# Client Libraries

## Overview
Application services, stores, networking, and the Phaser game engine used by the Svelte client.

## Interface
- `apiUrl` resolves API paths for browser and Tauri contexts (`api.ts:apiUrl`).
- `renderMarkdown` and `highlightCode` render terminal markdown content (`markdown.ts:renderMarkdown`, `markdown.ts:highlightCode`).
- `Counter.svelte` is the local counter example component (`Counter.svelte:increment`).
- Child contracts: [AI](./ai/README.md), [audio](./audio/README.md), [game](./game/README.md), [socket](./socket/README.md), and [stores](./stores/README.md).

## Invariants
- `safeHref` rejects URL schemes outside its explicit allowlist before inline links are rendered (`markdown.ts:safeHref`).

## Side Effects
- `apiUrl` reads browser location to select the local Tauri API origin (`api.ts:apiUrl`). The libraries' network, storage, and rendering effects are documented in their child contracts.

## Verification
- Run `npm run check` from `client/`.
