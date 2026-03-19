# ADR 0006 - Explicit invalidation over assumed freshness

## Status

Accepted

## Context

Extraction and retrieval artifacts lose trust silently when their freshness posture is implicit or forgotten.

## Decision

Cortex will require explicit freshness carriers and invalidation posture for governed artifacts.
When freshness cannot be asserted, Cortex must mark artifacts stale rather than imply readiness.

## Consequences

- contracts and schemas must encode freshness and invalidation fields
- stale becomes a first-class operational truth, not a soft warning
- implementation convenience cannot override truthful freshness posture
