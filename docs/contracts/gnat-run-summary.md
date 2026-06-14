# Gnat Run Summary

Schema: `schemas/gnat-run-summary.schema.json`

`GnatRunSummary.v1` is Cortex's authoritative reconciliation result for validated receipts.
It reports expected, complete, failed, stale, cancelled, and missing shard counts plus accepted receipt hashes and rejected receipt reasons.

The reconciler is order-independent and does not invent missing success.
Partial success, stale sources, duplicate receipts, schema-invalid receipts, and missing receipts remain visible.
