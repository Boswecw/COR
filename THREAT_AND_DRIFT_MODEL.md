# Cortex Threat and Drift Model

## Purpose

This document names the primary ways Cortex could become dangerous or misleading if left under-constrained.

## Threat 1 - authority drift

Failure mode:

- retrieval or extraction artifacts begin to be treated as truth authority

Primary controls:

- explicit non-canonical artifact marking
- app-owned bridging contracts for truth promotion
- boundary-matrix review

## Threat 2 - semantic creep

Failure mode:

- helpful metadata grows into classification, summarization, or interpretation

Primary controls:

- syntax-only doctrine
- semantic refusal conditions in contracts
- ADR coverage for boundary changes

## Threat 3 - orchestration creep

Failure mode:

- handoff support expands into downstream coordination, retry ownership, or execution routing

Primary controls:

- bounded reverse signaling
- explicit FA Local boundary
- refusal of workflow ownership

## Threat 4 - privacy collapse

Failure mode:

- diagnostics and debugging convenience expose raw content or broad previews

Primary controls:

- privacy-preserving diagnostics defaults
- bounded operator surfaces
- prohibition on raw-content browsing by default

## Threat 5 - uncontrolled observation

Failure mode:

- watchers become always-on, invisible, or too broad in scope

Primary controls:

- default-denied observation
- app-scoped watcher contracts
- operator-visible watcher status

## Threat 6 - ETL creep

Failure mode:

- Cortex becomes the default place for generalized extract-transform-load behavior

Primary controls:

- hard non-goals against generic sinks
- service-domain scope discipline
- explicit rejection of open-ended transforms

## Threat 7 - silent freshness drift

Failure mode:

- old artifacts continue to circulate without valid freshness posture

Primary controls:

- explicit invalidation rule
- stale-state signaling
- source-change dependency markers

## Anti-drift review questions

1. Does this proposal broaden syntax into semantics?
2. Does it broaden handoff into workflow control?
3. Does it expand observation power without explicit scope and visibility?
4. Does it imply Cortex is a truth authority rather than infrastructure support?
5. Does it turn a bounded service into a generic file or ETL platform?
