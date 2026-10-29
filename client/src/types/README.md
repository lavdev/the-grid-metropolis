# Client Types

## Overview
TypeScript contracts shared by the client UI, state stores, and WebSocket event handling.

## Interface
- `SpriteConfig`, `AgentLocation`, `AgentMetrics`, `Agent`, `VisualState`, `InteriorStation`, `Pod`, `TaskPriority`, `TaskStatus`, `TaskPayload`, `Task`, `EventTopic`, and generic `WebSocketEvent` are declared in [index.ts](./index.ts) (`index.ts:Agent`, `index.ts:Pod`, `index.ts:Task`, `index.ts:WebSocketEvent`).
- Card-domain types `CardRarity`, `CardCategory`, `TargetAIModel`, `ActionCard`, and `PlayedCardRecord` are declared in [card.ts](./card.ts) (`card.ts:ActionCard`).

## Invariants
- Agent roles and statuses, task priorities and statuses, and event topics are represented by string literal unions in `index.ts` (`index.ts:AgentRole`, `index.ts:AgentStatus`, `index.ts:TaskPriority`, `index.ts:TaskStatus`, `index.ts:EventTopic`).
- Card rarity, category, and target model values are bounded by string literal unions (`card.ts:CardRarity`, `card.ts:CardCategory`, `card.ts:TargetAIModel`).

## Side Effects
- None. These files declare types only (`index.ts:Agent`, `card.ts:ActionCard`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
