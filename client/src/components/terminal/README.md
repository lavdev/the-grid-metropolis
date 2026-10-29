# Terminal Components

## Overview
Contains the CRT terminal, execution and file inspectors, AI configuration controls, model selector, arcade cabinet, and Spark mascot.

## Interface
- Components: `CrtTerminal.svelte`, `ExecutionInspector.svelte`, `VimInspector.svelte`, `AiSettingsModal.svelte`, `ModelSelectorDropdown.svelte`, `ArcadeCabinet.svelte`, `PipelineStepper.svelte`, and `SparkMascot.svelte`.
- Component props and handlers include `connectionStatus`, `onReconnect`, `project`, `pipeline`, `isOpen`, and `triggerSleepNow` (`CrtTerminal.svelte:connectionStatus`, `CrtTerminal.svelte:onReconnect`, `ExecutionInspector.svelte:project`, `PipelineStepper.svelte:pipeline`, `ModelSelectorDropdown.svelte:isOpen`, `SparkMascot.svelte:triggerSleepNow`).
- `CrtTerminal` accepts the connection status and reconnect callback used by `App.svelte` (`CrtTerminal.svelte:connectionStatus`, `CrtTerminal.svelte:onReconnect`).
- State and streaming contracts: [client stores](../../lib/stores/README.md) and [AI service](../../lib/ai/README.md) (`projectStore.ts:sendCommandToActiveProject`, `aiService.ts:streamAiCompletion`).

## Invariants
- Terminal view selection is restricted to `CONSOLE` or `VIM_INSPECTOR` (`workspaceFilesStore.ts:terminalViewTab`).
- `SparkMascot` clears its active timers in the `onMount` cleanup callback (`SparkMascot.svelte:onMount`).

## Side Effects
- UI actions update shared stores; AI settings may issue a provider discovery `fetch`; terminal and mascot components schedule and clear browser timers (`AiSettingsModal.svelte:onMount`, `CrtTerminal.svelte:onMount`, `SparkMascot.svelte:onMount`).

## Verification
- Run `npm run check` from `client/`.
