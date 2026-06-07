# COR Gnats Master Checklist

## Implementation note

This Cortex local-system implementation completed the contract scaffold and serial proving slice.
FA-Local dispatch, bounded parallel routing, DF-Local persistence, Operator-Local controls, later source-lane expansion, and shared-core extraction remain deferred.

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
- [ ] `GnatDispatchEnvelope.v1`
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

- [ ] Capability negotiation.
- [ ] Contract-version negotiation.
- [ ] Worker-type admission.
- [ ] Concurrency clamp.
- [ ] Cancellation.
- [ ] Deadlines.
- [ ] Infrastructure-only retry rules.
- [ ] Serial fallback policy.
- [ ] FA-Local status truth.

## Parallel proof

- [ ] Two-worker test.
- [ ] Four-worker test.
- [ ] Eight-worker hard-cap test.
- [ ] Worker crash test.
- [ ] Duplicate receipt test.
- [ ] Missing receipt test.
- [ ] Stale source test.
- [ ] Partial-success test.
- [ ] Deterministic output test.
- [ ] Performance report.

## DF-Local

- [ ] Plan persistence.
- [ ] Immutable receipt persistence.
- [ ] Summary persistence.
- [ ] Exact-version cache key.
- [ ] Targeted invalidation.
- [ ] Retention policy.
- [ ] Persistence-degraded state.

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
