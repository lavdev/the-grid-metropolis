# Client Components

## Overview
Svelte presentation components for the HUD, agent inspection, canvas host, global feedback, terminal, cards, mission backlog, droids, project sidebar, and chat.

## Interface
- Direct components: `UiFeedback.svelte`, `TopBarHUD.svelte`, `AgentDrawer.svelte`, and `CanvasContainer.svelte`; their source handlers and props include `handleKeydown`, `connectionStatus`, `close`, and the `onMount` lifecycle callback (`UiFeedback.svelte:handleKeydown`, `TopBarHUD.svelte:connectionStatus`, `AgentDrawer.svelte:close`, `CanvasContainer.svelte:onMount`).
- Child contracts: [chat](./chat/README.md), [cards](./cards/README.md), [droids](./droids/README.md), [sidebar](./sidebar/README.md), [tasks](./tasks/README.md), and [terminal](./terminal/README.md).

## Invariants
- `TopBarHUD` receives the shared `ConnectionState` and a terminal-switch callback (`TopBarHUD.svelte:connectionStatus`, `TopBarHUD.svelte:onSwitchToTerminal`).
- `CanvasContainer` destroys the game instance when the Svelte component is destroyed (`CanvasContainer.svelte:onDestroy`).

## Side Effects
- Components update shared UI stores and create/destroy the Phaser game canvas (`AgentDrawer.svelte:close`, `CanvasContainer.svelte:onMount`, `CanvasContainer.svelte:onDestroy`, `UiFeedback.svelte:handleKeydown`).

## Verification
- Run `npm run check` from `client/`.
