# COR Gnats Master Checklist

## Implementation note

This Cortex local-system implementation completed the contract scaffold, serial proving slice,
FA-Local dispatch negotiation, FA-Local-gated bounded parallel proving slice,
and DF-Local GNAT persistence/cache contracts.
Integrated FA-Local lifecycle/routing, Operator-Local controls, later source-lane expansion,
and shared-core extraction remain deferred.

## Governance

- [x] COR/Cortex identity ADR approved.
- [x] Temporary package-name strategy recorded.
- [x] Gnat authorization ADR approved.
- [x] FA-Local execution ownership recorded.
- [x] Privacy/evidence posture recorded.
- [ ] Shared-core extraction threshold recorded.

## Contracts

- [x] `GnatRunRequest.v1`
- [x] `GnatRunPlan.v1`
- [x] `GnatShard.v1`
- [x] `GnatWorkerReceipt.v1`
- [x] `GnatRunSummary.v1`
- [x] `GnatDispatchEnvelope.v1`
- [x] `GnatCacheRecord.v1`
- [x] Valid fixtures.
- [x] Invalid fixtures.
- [x] Cross-schema validation.

## Serial proving slice

- [x] Markdown worker wrapper.
- [x] Plain-text worker wrapper.
- [x] Deterministic shard IDs.
- [x] Immutable plan hash.
- [x] Source fingerprint before/after.
- [x] Success receipts.
- [x] Failure receipts.
- [x] Order-independent reconciliation.
- [x] Existing CLI compatibility.

## FA-Local integration

- [x] Capability negotiation.
- [x] Contract-version negotiation.
- [x] Worker-type admission.
- [x] Concurrency clamp.
- [x] Cancellation.
- [x] Deadlines.
- [x] Infrastructure-only retry rules.
- [x] Serial fallback policy.
- [x] FA-Local status truth.

## Parallel proof

- [x] Two-worker test.
- [x] Four-worker test.
- [x] Eight-worker hard-cap test.
- [x] Worker crash test.
- [x] Duplicate receipt test.
- [x] Missing receipt test.
- [x] Stale source test.
- [x] Partial-success test.
- [x] Deterministic output test.
- [x] Performance report.

## DF-Local

- [x] Plan persistence.
- [x] Immutable receipt persistence.
- [x] Summary persistence.
- [x] Exact-version cache key.
- [x] Targeted invalidation.
- [x] Retention policy.
- [x] Persistence-degraded state.

## Operator-Local

- [ ] Run progress.
- [ ] Concurrency display.
- [ ] Cancel control.
- [ ] Serial fallback indicator.
- [ ] Cache reuse indicator.
- [ ] Partial-success display.
- [ ] Stale-source display.
- [ ] Bounded reason codes.
- [ ] No raw-content surveillance.
- [ ] Accessibility review.

## Later lane expansion

- [ ] PDF benchmark/admission.
- [ ] DOCX benchmark/admission.
- [ ] RTF benchmark/admission.
- [ ] ODT benchmark/admission.
- [ ] EPUB benchmark/admission.
- [ ] Scrivener remains separately governed.

## Shared-core extraction

- [ ] COR production proof complete.
- [ ] Second application requirements documented.
- [ ] Stable common interfaces identified.
- [ ] Domain code excluded from core.
- [ ] Versioning and compatibility tests established.

## Completion definition

GNAT-01 is complete only when:

1. Existing COR extraction behavior remains compatible.
2. A finite multi-file Markdown/plain-text request can execute through bounded FA-Local parallel workers.
3. Every shard produces accepted evidence or an explicit missing/failure state.
4. Reconciliation is deterministic regardless of worker completion order.
5. Source changes are detected and stale output is rejected.
6. Serial fallback remains available according to contract.
7. Service status truthfully reports capability and dependency state.
8. No NeuronForge call, watcher, mutation, or raw-content diagnostics are introduced.
9. Schema validation, runtime tests, privacy tests, and benchmark gates pass.
10. Operator-Local can display and cancel the run without becoming a content-surveillance surface.
