# Gnat Cache Record

`GnatCacheRecord.v1` is the exact-version cache identity for one bounded Cortex
Gnat shard result stored by DF-Local.

The reusable identity is:

```text
source_fingerprint_digest
worker_type
worker_implementation_version
operation_contract_version
lane_contract_version
```

A cache record is reusable only when the identity matches exactly, the record is
`ready`, the record has not expired, and `invalidated_at` is null.

Cached receipts are not reused verbatim across runs. Cortex may reuse the
bounded output and mint a fresh schema-valid receipt for the new run and shard.
DF-Local stores the old immutable receipt and the cache identity; Cortex remains
the validation and syntax-output authority.
