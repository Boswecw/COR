# Decision 0019 - FA-Local owns Gnat execution routing

This record preserves execution ownership boundaries for Gnat work.
It does not implement FA-Local.
It does not authorize Cortex to become a scheduler.

## Status

Accepted

## Date

2026-06-07

## Context

The Gnat target architecture separates Cortex planning and validation from execution routing.
Cortex owns source eligibility, shard construction, worker implementation, receipt validation, and reconciliation.
FA-Local owns policy-gated execution routing, scheduling, cancellation, concurrency limits, and retry decisions in integrated mode.

## Decision

In integrated Gnat mode, FA-Local owns execution routing.

Cortex may:

- validate a `GnatRunRequest.v1`;
- create an immutable `GnatRunPlan.v1`;
- expose admitted worker types and contract versions;
- validate returned `GnatWorkerReceipt.v1` payloads;
- reconcile accepted receipts into `GnatRunSummary.v1`;
- execute a serial compatibility path only when the contract explicitly permits fallback.

Cortex may not:

- run a hidden long-lived scheduler;
- silently retry deterministic worker failures;
- fabricate missing receipts;
- treat partial success as readiness;
- delegate semantic interpretation into the Gnat path;
- make FA-Local unavailable states invisible.

## Consequences

Cortex GNAT-01 may include an in-process serial runner to prove contracts and preserve fallback behavior.
Bounded parallel execution requires a later FA-Local dispatch adapter and capability negotiation.
