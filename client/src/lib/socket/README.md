# WebSocket Client

## Overview
Owns the browser-side WebSocket connection, event subscriptions, buffering, and reconnection behavior.

## Interface
- `GridWebSocketClient` exposes `connect`, `disconnect`, `subscribe`, and state-change subscription methods (`client.ts:GridWebSocketClient`).
- `ConnectionState` defines `CONNECTING`, `CONNECTED`, and `DISCONNECTED` (`client.ts:ConnectionState`); event payloads use the shared `WebSocketEvent` type (`client.ts:GridWebSocketClient`).

## Invariants
- Buffered events are bounded by `MAX_BUFFERED_EVENTS` (100) (`client.ts:MAX_BUFFERED_EVENTS`).
- `connect` does not create another socket while the current socket is open or connecting (`client.ts:GridWebSocketClient.connect`).

## Side Effects
- Opens and closes a browser `WebSocket`, parses incoming JSON events, notifies listeners, and schedules reconnect timers (`client.ts:GridWebSocketClient.connect`, `client.ts:GridWebSocketClient.scheduleReconnect`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
