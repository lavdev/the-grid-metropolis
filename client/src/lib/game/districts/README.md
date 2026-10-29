# City Districts

## Overview
Contains Phaser structures for the Nexus, Project Pod, Glitch Alley, and Battery Lounge districts.

## Interface
- `TheNexusSector`, `ProjectPodStructure`, `GlitchAlleySector`, and `BatteryLoungeSector` construct and update district visuals (`nexus.ts:TheNexusSector`, `projectPod.ts:ProjectPodStructure`, `glitchAlley.ts:GlitchAlleySector`, `batteryLounge.ts:BatteryLoungeSector`).
- `PodVisualState` is the visual-state input contract (`projectPod.ts:PodVisualState`).

## Invariants
- Pod visual modes and fields are represented by `PodVisualState` (`projectPod.ts:PodVisualState`).

## Side Effects
- Creates and updates Phaser graphics for structures and visual effects (`nexus.ts:TheNexusSector`, `projectPod.ts:ProjectPodStructure`, `glitchAlley.ts:GlitchAlleySector`, `batteryLounge.ts:BatteryLoungeSector`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
