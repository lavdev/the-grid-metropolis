# Task Components

## Overview
Contains the mission backlog dialog.

## Interface
- `MissionBacklogModal.svelte` displays the mission backlog and dispatches its UI actions (`MissionBacklogModal.svelte:sendInstruction`, `MissionBacklogModal.svelte:deleteMission`).
- Backlog state and task actions are defined in [client stores](../../lib/stores/README.md) (`chatStore.ts:missionBacklog`, `chatStore.ts:deleteMission`).

## Invariants
- Backlog entries use the `MissionTask` shape (`chatStore.ts:MissionTask`).

## Side Effects
- Updates backlog and visibility stores in response to dialog actions (`MissionBacklogModal.svelte:isBacklogOpen`, `chatStore.ts:deleteMission`).

## Verification
- Run `npm run check` from `client/`.
