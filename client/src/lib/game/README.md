# Isometric Game Engine

## Overview
Contains the Phaser game setup and client-side systems for isometric coordinates, city topology, rendering, depth, district structures, agents, and world effects.

## Interface
- `createGridMetropolisGame` and `destroyGridMetropolisGame` manage the Phaser game (`gameManager.ts:createGridMetropolisGame`, `gameManager.ts:destroyGridMetropolisGame`).
- `GameBridge` and `gameBridge` connect UI actions to the scene and agents (`gameBridge.ts:GameBridge`, `gameBridge.ts:gameBridge`).
- `GridPathfinder`, `TileType`, `buildCityGridMatrix`, isometric conversion helpers, and `DepthManager` provide navigation and coordinate/depth operations (`pathfinding.ts:GridPathfinder`, `pathfinding.ts:buildCityGridMatrix`, `isometric.ts:cartesianToIsometric`, `isometric.ts:isometricToCartesian`, `depth.ts:DepthManager`).
- `DataOcean`, `CityTilesetRenderer`, `DepthLayer`, and `calculateIsometricDepth` provide world rendering and depth-order operations (`dataOcean.ts:DataOcean`, `tileset.ts:CityTilesetRenderer`, `depth.ts:DepthLayer`, `depth.ts:calculateIsometricDepth`).
- Child contracts: [agents](./agents/README.md), [districts](./districts/README.md), [scenes](./scenes/README.md), [world](./world/README.md).

## Invariants
- City topology is a 36-by-36 matrix; pathfinding accepts the declared navigable tile types and disables diagonal movement (`pathfinding.ts:CITY_GRID_WIDTH`, `pathfinding.ts:CITY_GRID_HEIGHT`, `GridPathfinder.constructor`).
- Isometric tile dimensions are 64 by 32 (`isometric.ts:TILE_WIDTH`, `isometric.ts:TILE_HEIGHT`).

## Side Effects
- Creates/destroys a Phaser WebGL game and mutates Phaser scene, texture, display, and graphics state (`gameManager.ts:createGridMetropolisGame`, `gameManager.ts:destroyGridMetropolisGame`, `tileset.ts:CityTilesetRenderer`, `gameBridge.ts:GameBridge`).

## Verification
- Run `npm run check` from `client/`.
