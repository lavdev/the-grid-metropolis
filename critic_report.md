# Stage 7 — Critic Review

**Target:** `docs/designs/harness-pro.md` (FeatureRange, lines 1–280)
**Reviewers:** `egm-design`, `egm-structural`, and `egm-style` — all completed.
**Standards:** `feature-spec-standard.md` for design and structural reviewers; no separate style standard assigned.

## Prioritized findings

### Major — Policy enforcement is not shown for operations that need no approval

**Reviewer lens:** `egm-design`
**Affected scope:** Lines 50, 192–194.

The design describes checking app-server approval requests, but this does not establish that every operation is checked. An operation allowed by the configured sandbox may execute without producing an approval request. The per-turn sandbox mapping is not demonstrated to enforce both the project ceiling and the narrower droid policy for such operations.

**Recommendation:** Define and prototype enforcement for both approval-triggering and already-permitted operations. Include both cases in the acceptance gate before enabling real runs.

### Major — Reconnect recovery has a snapshot/replay/subscription race

**Reviewer lens:** `egm-design`
**Affected scope:** Lines 110, 211.

An event committed and published after replay completes but before WebSocket subscription can be missed until another reconnect.

**Recommendation:** Define an atomic high-water sequence and a subscribe/replay handshake that closes the gap; clients should deduplicate by sequence.

### Major — Run events have no WebSocket authentication or project filtering contract

**Reviewer lens:** `egm-design`
**Affected scope:** Lines 114, 142, 211.

REST run routes are stated to require authentication, but the existing WebSocket accepts connections without authentication and broadcasts world events to subscribers. The design does not say how private run events are protected or filtered by project.

**Recommendation:** Specify WebSocket authentication and authorization/filtering for run events, how the desktop client supplies credentials, and add these requirements to the API contract.

### Major — Risk coverage is scattered rather than explicit

**Reviewer lens:** `egm-structural`
**Affected scope:** Lines 1–280, especially Open Questions, Technical Plan, and Alternatives Considered.

The feature-spec standard expects risks, constraints, failure modes, security implications, scalability limits, and mitigations. These appear in decision trade-offs and failure conditions, but there is no consolidated `Risks & Mitigations` section.

**Recommendation:** Add a clearly labeled section consolidating risks and mitigations, or establish a clear index to their locations.

### Major — Alternatives repeat decision rationales

**Reviewer lens:** `egm-style`
**Affected scope:** Lines 245–279.

The alternatives section repeats rationales already detailed under Decisions 1–5, making the design harder to scan.

**Recommendation:** Consolidate the rationale or make the alternatives section a short index.

## Nits

* **Reviewer lens:** `egm-style`; **Scope:** Lines 131, 209. Authentication and path-validation migration caveats appear twice. State this requirement once.
* **Reviewer lens:** `egm-style`; **Scope:** Lines 9, 13, 35. “Trustworthy,” “truthful,” and “reliably” are broad evaluative terms; use the observable conditions already described.
* **Reviewer lens:** `egm-style`; **Scope:** Lines 64, 79. “Closest supported integration” and “costs more than the control it adds” lack explicit comparison criteria; name the capability or cost being compared.

## Structural check

The structural reviewer found the problem, goals/non-goals, architecture, and alternatives areas present. No heading hierarchy or Markdown formatting violations were found in the reviewed range.
