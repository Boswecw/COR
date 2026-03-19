# ADR 0007 - Shared Source-Lane Framework

## Status

Accepted

## Context

Cortex began with bounded local Markdown and plain-text support, then admitted a bounded text-layer PDF lane.

Without a shared lane model, future source-family admission would drift toward one-off branches, duplicated failure logic, and inconsistent service-status reporting.

## Decision

Cortex will treat source support as a governed lane framework with:

- explicit lane registration
- shared admission checks
- shared failure taxonomy
- shared provenance posture
- shared service-status reporting of admitted lanes

Lane-specific parsing remains separate, but lane admission and runtime truth must come from a shared bounded model.

## Consequences

Positive:

- new lanes enter through one bounded framework
- service-status can report admitted lanes from one source of truth
- fail-closed behavior is more consistent across lanes

Negative:

- adding a lane now requires both runtime and contract registration work

## Rejected alternatives

- ad hoc per-format branches in extraction runtime
- generic plugin-oriented document ingestion
- broad "document" capability reporting in service status
