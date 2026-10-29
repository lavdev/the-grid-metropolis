# Client State Stores

## Overview
Holds reactive client state for the metropolis view, projects, conversations, droids, cards, files, AI settings, theme, and UI feedback.

## Interface
- Grid event state and event reducer: `agents`, `pods`, `tasks`, `selectedAgent`, `eventLogs`, `handleIncomingEvent` (`gridStore.ts:handleIncomingEvent`).
- Project and conversation state/actions: `projects`, `activeProjectId`, `activeProject`, `selectProject`, `createNewProject`, `updateDroidState` (`projectStore.ts:selectProject`, `projectStore.ts:createNewProject`, `projectStore.ts:updateDroidState`).
- Roster, card, chat, workspace-file, AI configuration, theme, and UI feedback stores/actions are exported by `droidRosterStore.ts`, `cardStore.ts`, `chatStore.ts`, `workspaceFilesStore.ts`, `aiConfigStore.ts`, `themeStore.ts`, and `uiFeedbackStore.ts` (for example `droidRosterStore.ts:saveDroid`, `cardStore.ts:playCard`, `chatStore.ts:sendInstruction`, `workspaceFilesStore.ts:loadWorkspaceFile`, `aiConfigStore.ts:updateAiConfig`, `uiFeedbackStore.ts:notify`).
- Related package contracts: [AI](../ai/README.md), [audio](../audio/README.md), [socket](../socket/README.md).

## Invariants
- Project, roster, and AI configuration persistence uses distinct local-storage keys and guards browser access (`projectStore.ts:ACTIVE_PROJECT_STORAGE_KEY`, `droidRosterStore.ts:ROSTER_STORAGE_KEY`, `aiConfigStore.ts:STORAGE_KEY`).
- Project message writes are delayed per project/message key by `queueProjectMessagePersistence` (`projectStore.ts:queueProjectMessagePersistence`).

## Side Effects
- Reads/writes browser local storage, calls project and file API routes through `fetch`, and schedules timers for persistence and chat actions (`projectStore.ts:loadProjectsFromDatabase`, `projectStore.ts:queueProjectMessagePersistence`, `droidRosterStore.ts:loadInitialRoster`, `aiConfigStore.ts:loadInitialConfig`, `workspaceFilesStore.ts:loadWorkspaceFile`, `chatStore.ts:sendInstruction`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
