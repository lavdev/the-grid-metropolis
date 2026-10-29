# Game Agents

## Overview
Implements Phaser entities and management for agents and drones, including procedural sprite generation.

## Interface
- `AgentEntity`, `DroneEntity`, and `AgentManager` define the rendered entities and roster manager (`AgentEntity.ts:AgentEntity`, `DroneEntity.ts:DroneEntity`, `AgentManager.ts:AgentManager`).
- `AgentData`, `AgentRole`, and `AgentStatus` define entity inputs (`AgentEntity.ts:AgentData`, `AgentEntity.ts:AgentRole`, `AgentEntity.ts:AgentStatus`).
- `ProceduralSpriteGenerator`, `AgentVisualTheme`, `AGENT_THEMES`, and `ROLE_FALLBACKS` define sprite generation and theme configuration (`spritesheetGenerator.ts:ProceduralSpriteGenerator`, `spritesheetGenerator.ts:AgentVisualTheme`, `spritesheetGenerator.ts:AGENT_THEMES`, `spritesheetGenerator.ts:ROLE_FALLBACKS`).

## Invariants
- Agent role and status values are constrained by string literal unions (`AgentEntity.ts:AgentRole`, `AgentEntity.ts:AgentStatus`).

## Side Effects
- Generates canvas textures and creates/updates Phaser display objects (`spritesheetGenerator.ts:ProceduralSpriteGenerator`, `AgentEntity.ts:AgentEntity`, `DroneEntity.ts:DroneEntity`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
