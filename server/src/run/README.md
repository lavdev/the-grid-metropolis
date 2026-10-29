# Run

## Overview

The run module contains application services for durable droid runs. Its budget service applies project ceilings and returns whether the coordinator may start a turn, must interrupt an active turn, or must stop before the next turn ([`budget`](budget.rs)).

## Invariants

- Project budgets are required; a droid or run override may only lower the project values.
- Model-turn and elapsed-time limits are checked before each new turn.
- Token usage is best-effort: when provider telemetry reaches the configured value, the active turn is interrupted when possible and no later turn may start. A turn can overshoot before the telemetry arrives.
- This module does not enable live execution by itself; the run feature remains gated until persistence, approvals, auth, and the end-to-end flow are ready.

## Verification

- Budget unit coverage lives in `budget.rs`.
