# Chat Components

## Overview
Contains the global chat input and mission-launch UI.

## Interface
- `GlobalChatBar.svelte` renders the chat composer; `handleSubmit` and `openBacklog` connect it to chat and backlog actions (`GlobalChatBar.svelte:handleSubmit`, `GlobalChatBar.svelte:openBacklog`).

## Invariants
- Chat submission delegates to the `sendInstruction` store action (`GlobalChatBar.svelte:sendInstruction`).

## Side Effects
- Updates chat state and opens the mission backlog via client stores (`GlobalChatBar.svelte:sendInstruction`, `GlobalChatBar.svelte:isBacklogOpen`).

## Verification
- Run `npm run check` from `client/`.
