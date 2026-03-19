# Boundary Matrix - Cortex

## Purpose

This matrix is the primary anti-drift artifact for the Cortex project workspace.

It exists to make capability ownership, non-ownership, authority posture, privacy sensitivity, degraded behavior, and cross-service boundary relations explicit before implementation expands.

## Reading rule

Every row must make clear:

- what Cortex owns
- what Cortex explicitly does not own
- what kind of authority the row implies
- what privacy pressure the row introduces
- how reduced-state truth should surface

## Matrix

| Capability | Owned by Cortex | Explicitly not owned by Cortex | Authority status | Privacy sensitivity | Truthful reduced states | Cross-service note |
|---|---|---|---|---|---|---|
| local file intake | yes | DF Local Foundation, NeuronForge Local, FA Local, consuming apps as intake owner for this service boundary | intake authority only within bounded contracts | high | denied, partial_success, unavailable, integrity_failed | may rely on DF Local Foundation for substrate support |
| syntax-level extraction | yes | NeuronForge Local as semantic authority, FA Local as execution authority | syntax authority only, never semantic authority | high | extraction_incomplete, partial_success, stale, integrity_failed | outputs may be handed to NeuronForge Local without transferring semantics |
| structure detection | yes | NeuronForge Local, FA Local, consuming apps as structure owner | structural authority only | high | degraded, extraction_incomplete, stale | limited to syntax-level structure |
| provenance and completeness signaling | yes | downstream systems as truth authority | bounded reporting authority only | medium | extraction_incomplete, stale, partial_success | provenance informs downstream trust but does not decide truth |
| retrieval-preparation support | yes | consuming apps as retrieval-policy authority, NeuronForge Local as semantic authority | infrastructure support only, not retrieval authority | medium to high | stale, partial_success, degraded | governed profiles must stay explicit |
| packaging and handoff support | yes | FA Local as execution authority, consuming apps as workflow authority | handoff-packaging authority only | medium | re_prep_required, stale, integrity_failed, rejected_reason_code | reverse signaling must remain minimal |
| freshness and invalidation signaling | yes | consuming apps as truth authority | freshness-reporting authority only for Cortex-owned artifacts | low to medium | stale, invalidation_triggered, degraded | artifact age does not imply truth authority |
| privacy-preserving diagnostics | yes | all services as broad surveillance authority | diagnostics authority only for Cortex surfaces | high | degraded, stale, unavailable | no raw-content browsing by default |
| contract-scoped observation | limited and opt-in only | Cortex as default broad watcher, consuming apps as invisible watcher owner | observation authority only within explicit app-scoped contract | high | denied, unavailable, stale | watcher presence must be operator-visible |

## Cross-service boundary summary

### DF Local Foundation

Provides storage and lifecycle substrate support.
Does not absorb Cortex file-intelligence logic.

### NeuronForge Local

Consumes Cortex artifacts for semantic work.
Does not transfer semantic authority back into Cortex.

### FA Local

Owns governed execution routing.
Does not delegate execution authority into Cortex.

### Consuming applications

Own business meaning and canonical truth.
Must use app-owned bridging contracts for any truth promotion.
