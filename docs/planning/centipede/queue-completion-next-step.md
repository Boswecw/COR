# Centipede Queue Completion Next Step

**Status:** Draft Planning Note  
**Date and Time:** 2026-04-21 America/New_York  
**Repository:** Boswecw/COR

---

## Why this exists

The bounded local queue reclaim proof is now green in the `repo-crawler` workstream.

That proof established:

- claim writes `processingState.status = "claimed"`
- expired claim reclaims back to `queued`
- reclaimed item can be claimed again
- no-expired-claim runs degrade cleanly as a no-op

The next step is to close the lifecycle with a governed completion surface.

---

## Immediate objective

Implement queue completion so a queue item that has gone through:

1. initial claim
2. stale expiry
3. reclaim
4. second active claim

can be completed deterministically without losing historical claim evidence.

---

## Required behavior

### 1. Active-claim-only completion

Completion must bind only to the **currently active claim episode**.

A reclaimed or stale claim episode must fail closed and cannot complete the queue item.

### 2. Deterministic queue item final state

On valid completion:

- queue item `processingState.status` becomes `completed`
- queue item retains auditable linkage to the active completion episode
- prior reclaimed claim history remains preserved

### 3. Deterministic claim record continuity

Because the deterministic claim file is reused across claim episodes for the same queue item, completion must:

- preserve `claimHistory`
- record final completion against the latest active claim episode
- avoid overwriting stale-episode evidence

### 4. Failed-closed invalid completion attempts

Completion must reject:

- already reclaimed claim state
- stale or expired claim state
- mismatched active-claim identity
- already-completed queue item state

---

## Proposed bounded implementation surface

- update `centipede_queue_complete.rs`
- add any supporting receipt/index material strictly needed for deterministic completion replay
- add one bounded smoke proving:
  - claim
  - reclaim
  - re-claim
  - complete
  - final completed item state
  - preserved claim history
  - rejection of stale completion attempt

---

## Acceptance gate

This next step is complete when:

- reclaimed-then-reclaimed-again items complete cleanly
- stale/non-active completion attempts fail closed
- historical claim evidence remains preserved
- final queue item state is deterministic and operator-readable
- smoke proof passes end to end

---

## Explicitly out of scope

This step does **not** include:

- proposal actuation
- remediation execution
- auto-healing authority
- downstream autonomous handoff
- multi-repo weighted reconciliation

Those remain later Centipede program surfaces.

---

## Operator note

Use this file as the repo-local planning anchor instead of an issue-thread workflow for this next bounded step.