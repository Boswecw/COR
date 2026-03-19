# ADR 0005 - Bounded reverse signaling

## Status

Accepted

## Context

Once handoff exists, downstream consumers tend to ask for richer callback language.
That pressure can turn a packaging boundary into a workflow controller.

## Decision

Phase 1 reverse signaling remains minimal and anti-orchestration in posture.
The initial bounded enum is:

- `accepted`
- `rejected_reason_code`
- `re_prep_required`
- `stale`
- `integrity_failed`

## Consequences

- Cortex can reason about handoff outcomes without owning downstream workflow
- new reverse signals require explicit review because they broaden control language
- retry coordination stays outside Cortex unless architecture changes
