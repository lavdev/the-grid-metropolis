# Project Sidebar

## Overview
Renders the project and droid navigation sidebar.

## Interface
- `ProjectSidebar.svelte` reads project and roster state; `selectProject` is imported from the project store and used by the component (`ProjectSidebar.svelte:selectProject`).

## Invariants
- Project expansion preferences are parsed by `loadProjectExpansionState` before use (`ProjectSidebar.svelte:loadProjectExpansionState`).

## Side Effects
- Reads and writes browser local storage for project expansion state, and registers viewport behavior on mount (`ProjectSidebar.svelte:loadProjectExpansionState`, `ProjectSidebar.svelte:toggleProjectExpansion`, `ProjectSidebar.svelte:onMount`).

## Verification
- Run `npm run check` from `client/`.
