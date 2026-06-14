# Slice 40 Plan — Queue Operator / Evidence Surface

**Date:** 2026-04-21 22:20 America/New_York
**Program:** Centipede
**Repo:** `repo-crawler`

## Slice boundary

After the queue-core tranche is checkpointed and captured, the next code slice should stay **read-only** and **non-actuating**.

That slice should add a bounded queue operator/evidence surface that explains the current queue state without requiring raw file archaeology.

## Recommended target

Build a first queue operator surface with these outputs:

1. queue item summary
2. active-vs-terminal episode summary
3. claim episode list in chronological order
4. completion/failure/reclaim evidence pointers
5. stale-attempt rejection visibility

## Why this is the right next move

The queue core is already green locally for claim, reclaim, complete, heartbeat, and fail.

The next value is not more mutation logic. The next value is a clean read model over what already exists.

That keeps the next slice aligned with the larger Centipede doctrine:

- read-only first
- operator-safe projection
- evidence visibility
- no silent actuation

## Recommended acceptance for Slice 40

A bounded first pass is enough if one command can:

- load the queue artifact state for a target item or queue root
- show terminal vs non-terminal status clearly
- identify the active claim attempt if one exists
- show whether an item is `queued`, `claimed`, `completed`, or `failed`
- show reclaim/completion/failure episode evidence refs
- reject stale claim-attempt views cleanly

## Do not add yet

- retry/requeue mutation logic
- auto-repair
- proposal generation
- score math
- ForgeCommand integration

## Proof target for Slice 40

Prefer a deterministic smoke that:

1. creates a fixture queue root
2. runs claim/reclaim/complete/fail state setup
3. invokes the new read-only queue operator/report command
4. proves the report matches the fixture truth
