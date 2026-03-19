# Cortex Phase 1 Plan

## Objective

Prove Cortex as a trustworthy bounded local intake, extraction, and retrieval-preparation substrate.

## Include in Phase 1

- project charter
- local doctrine
- authority boundaries
- service domains
- control-surface rules
- degradation model
- threat and drift model
- intake request contract
- extraction result contract
- retrieval package contract
- service status contract
- embedded diagnostics contract
- syntax-only extraction guardrails
- explicit invalidation rule
- minimal reverse signaling enum
- boundary matrix
- founding ADR set

## Exclude from Phase 1

- broad source connector ecosystems
- always-on observation by default
- semantic enrichment
- app-specific retrieval-policy engines
- broad packaging or export automation
- downstream orchestration logic
- standalone UI identity
- generic ETL sink behavior

## Contract-first delivery order

1. constitutional base docs
2. doctrine and boundary ADRs
3. boundary matrix and architecture references
4. Phase 1 contracts and schemas
5. fixtures and validation

## Exit criteria

Phase 1 is complete only when Cortex can:

- truthfully intake eligible local content
- extract syntax-level structure
- produce one governed retrieval package form
- attach freshness and invalidation posture
- surface bounded operational truth through consuming applications
- refuse boundary-crossing requests explicitly

## Acceptance evidence rule

No Phase 1 claim is accepted by prose alone.
Each claimed capability must be backed by one or more of:

- a doctrine or ADR
- a contract or schema
- a fixture or validation test
- an operator-visible bounded status surface
