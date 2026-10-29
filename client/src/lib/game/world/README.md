# Living World Overlay

## Overview
Draws animated environmental overlays for the isometric world.

## Interface
- `LivingWorldOverlay` owns overlay graphics and per-frame animation (`LivingWorldOverlay.ts:LivingWorldOverlay`).

## Invariants
- `update` advances animation state and redraws its graphics through the Phaser scene (`LivingWorldOverlay.ts:LivingWorldOverlay.update`).

## Side Effects
- Creates and updates Phaser graphics attached to the scene (`LivingWorldOverlay.ts:LivingWorldOverlay`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
