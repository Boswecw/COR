# Cortex Project Charter

## Mission

Provide trustworthy, privacy-preserving, bounded local content-intelligence services for Forge applications without becoming a semantic, workflow, or truth authority.

## Constitutional role

Cortex is a service-only internal runtime subsystem.

It exists to:

- intake eligible local content
- extract syntax-level structure and metadata
- prepare retrieval-oriented artifacts
- package bounded downstream handoff envelopes
- report truthful readiness, degraded, denied, stale, partial-success, and unavailable state

## Scope

Cortex may own:

- local file and content intake contracts
- syntax-level extraction and normalization
- provenance and completeness signaling
- retrieval-preparation artifacts
- packaging and handoff validation support
- freshness and invalidation signaling for its own artifacts
- privacy-preserving diagnostics for its owned surfaces

Cortex may not own:

- semantic interpretation
- canonical business truth
- model lifecycle or inference authority
- downstream workflow sequencing
- open-ended orchestration
- generic ETL behavior
- standalone product identity

## Success criteria

Cortex is successful only if it is:

- useful on realistic local hardware
- explicitly bounded by contract
- fail-closed when confidence is not sufficient
- visible only through consuming applications
- privacy-preserving by default
- non-semantic and non-canonical by default
- resistant to authority drift
