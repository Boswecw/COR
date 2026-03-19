# Cortex Authority Boundaries

## Cortex owns

Cortex owns:

- intake contracts for eligible local content
- normalization for admitted inputs
- syntax-level extraction and structural shaping
- provenance, completeness, and integrity signaling
- retrieval-preparation support artifacts
- handoff-envelope validation and packaging support
- freshness and invalidation signaling for Cortex-owned artifacts
- service status and privacy-preserving diagnostics for Cortex surfaces

## Cortex does not own

Cortex does not own:

- semantic interpretation
- app meaning or product policy
- model weights, model routing, or inference parameters
- downstream execution ownership
- workflow sequencing or retry coordination outside contract
- canonical business truth
- broad observability or surveillance authority
- generic ETL responsibilities

## Boundary with DF Local Foundation

DF Local Foundation owns substrate mechanics such as storage support, migrations, and readiness foundations.

Cortex may depend on DF Local Foundation by contract.
That dependency does not transfer syntax, retrieval, or file-intelligence authority to DF Local Foundation.

## Boundary with NeuronForge Local

NeuronForge Local owns semantic interpretation, inference execution, and candidate generation.

Cortex may provide syntax-level extraction outputs and retrieval-ready packages to NeuronForge Local.
That handoff does not make Cortex a semantic authority.

## Boundary with FA Local

FA Local owns policy-gated execution routing and approved task execution.

Cortex may expose bounded callable contracts and handoff envelopes.
It must not become the execution host or workflow governor.

## Boundary with consuming applications

Consuming applications own:

- business meaning
- product decisions
- workflow policy
- canonical truth
- user-facing interpretation

Applications may promote a Cortex artifact into app truth only through an app-owned bridging contract.

## Transfer rule

Possession is not authority.

If Cortex stores, shapes, or packages an artifact, that alone does not make Cortex the authority over meaning, ranking, acceptance, or truth.
