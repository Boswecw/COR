# Gnat Retrieval Preparation

Output schema: `schemas/retrieval-package.schema.json`

Phase 8 retrieval preparation consumes validated `GnatWorkerReceipt.v1`
extraction receipts from an immutable `GnatRunPlan.v1` and emits one governed
retrieval package.

It does not introduce a semantic ranking surface, embedding surface, model call,
or canonical meaning claim. Chunk order is derived only from plan shard order and
each source's bounded syntax chunk order.

States:

- `ready` when every planned shard has a complete, chunkable extraction receipt.
- `partial_success` when at least one completed shard is chunkable and at least
  one planned shard is missing, failed, denied, stale, or not chunkable.
- `denied` when no completed shard can produce a chunkable retrieval package.

Cortex owns receipt validation, deterministic merge order, and retrieval-package
schema validation. FA-Local remains the execution router for extraction GNATs;
this phase does not give FA-Local retrieval semantics.
