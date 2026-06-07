# Gnat Run Plan

Schema: `schemas/gnat-run-plan.schema.json`

`GnatRunPlan.v1` is the immutable Cortex-created plan for a finite Gnat run.
It records request identity, operation, shard count, source fingerprints, execution limits, expected receipt schema, fallback policy, and a stable plan hash.

Cortex creates and validates this plan.
FA-Local owns integrated routing and cancellation when the parallel path is implemented.

The plan may not carry hidden orchestration fields or unrestricted local paths.
