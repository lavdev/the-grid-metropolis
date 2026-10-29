# Card Components

## Overview
Renders the card hand, card inspection, card creation, and card catalog dialogs.

## Interface
- Components: `CardHandHUD.svelte`, `CardItem.svelte`, `CardInspectorModal.svelte`, `ForgeCardModal.svelte`, and `GrimoireModal.svelte`.
- `CardItem` accepts `card`, `index`, and `totalInHand` props (`CardItem.svelte:card`, `CardItem.svelte:index`, `CardItem.svelte:totalInHand`).
- Related state and actions are in [the stores contract](../../lib/stores/README.md) (`cardStore.ts:playCard`, `cardStore.ts:forgeCustomCard`).

## Invariants
- The rendered card inputs use the shared `ActionCard` type (`CardItem.svelte:card`, `card.ts:ActionCard`).

## Side Effects
- Components update card inspection and modal stores in response to user actions (`CardItem.svelte:handleInspect`, `ForgeCardModal.svelte:handleForge`, `CardHandHUD.svelte:isForgeModalOpen`).

## Verification
- Run `npm run check` from `client/`.
