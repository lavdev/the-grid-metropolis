# Droid Components

## Overview
Provides the droid roster editor dialog.

## Interface
- `DroidForgeModal.svelte` edits and saves roster entries through `handleSave` (`DroidForgeModal.svelte:handleSave`).
- Roster state contract: [client stores](../../lib/stores/README.md) (`droidRosterStore.ts:RosterDroid`, `droidRosterStore.ts:saveDroid`).

## Invariants
- Saved roster data is represented by `RosterDroid` (`droidRosterStore.ts:RosterDroid`).

## Side Effects
- Updates roster and forge-modal stores (`DroidForgeModal.svelte:handleSave`, `DroidForgeModal.svelte:handleOpenCreate`, `droidRosterStore.ts:saveDroid`, `droidRosterStore.ts:closeDroidForge`).

## Verification
- Run `npm run check` from `client/`.
