# Isometric Scene

## Overview
Contains the Phaser scene that creates and updates the city map, sectors, agents, and world effects.

## Interface
- `MainIsometricScene` is the game scene (`MainIsometricScene.ts:MainIsometricScene`).

## Invariants
- The scene update loop advances the data ocean, living overlay, agent manager, and other scene systems (`MainIsometricScene.ts:MainIsometricScene.update`).

## Side Effects
- Creates Phaser display objects and delegates frame updates to scene systems (`MainIsometricScene.ts:MainIsometricScene.create`, `MainIsometricScene.ts:MainIsometricScene.update`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
