# ADR 0004 - No agent-host drift

## Status

Accepted

## Context

Services that prepare content for downstream systems are often pressured to absorb task routing, retries, execution loops, or autonomous coordination.

## Decision

Cortex must not become a general local execution host, agent platform, or workflow coordinator.
Execution authority remains outside Cortex.

## Consequences

- handoff support stays bounded to validation and packaging
- downstream retries and execution sequencing remain out of scope
- Cortex cannot be used as a convenience home for orchestration logic
