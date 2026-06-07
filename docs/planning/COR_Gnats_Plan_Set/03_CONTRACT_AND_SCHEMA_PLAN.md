# Contract and Schema Plan

## Contract location

Canonical shared contracts should ultimately live in `forge_contract_core`. COR may carry local mirrored schemas during the proving slice, following its current repository pattern, but contract authority should migrate to the shared contract owner after validation.

## Initial contract sequence

### 1. `gnat-run-request.schema.json`

Purpose: app/Operator-Local request to COR for a bounded parallel operation.

Required concepts:

- contract version;
- request ID and correlation ID;
- operation type;
- source references;
- declared media types;
- requested concurrency;
- deadline;
- privacy mode;
- result mode;
- caller authority context reference.

### 2. `gnat-run-plan.schema.json`

Purpose: immutable COR-generated finite plan for FA-Local.

Required concepts:

- run ID;
- planner version;
- operation;
- shard count;
- worker type per shard;
- source fingerprint per shard;
- execution limits;
- expected receipt schema;
- plan hash;
- serial fallback permission.

### 3. `gnat-shard.schema.json`

Purpose: one bounded unit of work.

Required concepts:

```text
run_id
shard_id
ordinal
worker_type
source_ref
source_path_token or approved local reference
media_type
source_fingerprint
operation
limits
output_contract
```

Raw unrestricted filesystem paths should not be persisted beyond the local boundary when a scoped token/reference can be used.

### 4. `gnat-worker-receipt.schema.json`

Purpose: evidence for every attempt.

Required concepts:

```text
run_id
shard_id
attempt_id
worker_id
worker_type
implementation_version
source_fingerprint_before
source_fingerprint_after
state
started_at
completed_at
duration_ms
output_ref or bounded_output
finding_counts
error_reason_code
redaction_applied
```

### 5. `gnat-run-summary.schema.json`

Purpose: authoritative COR summary of valid receipts.

Required concepts:

- run state;
- expected/completed/failed/stale/cancelled counts;
- accepted receipt hashes;
- rejected receipt hashes and reason codes;
- aggregate timing;
- concurrency used;
- fallback used;
- output artifact references;
- operator-visible bounded message.

### 6. `gnat-dispatch-envelope.schema.json`

Purpose: COR-to-FA-Local handoff.

Must state explicitly that FA-Local owns execution routing and COR owns result-contract validation.

### 7. `gnat-cache-record.schema.json`

Added after the GNAT-01 correctness and bounded parallel proof.

Key:

```text
source_fingerprint
worker_type
worker_implementation_version
operation_contract_version
lane_contract_version
```

### 8. `gnat-operator-run-status.schema.json`

Added for the COR-side Operator-Local status/control contract.

Required concepts:

- distinguishable run state and display label;
- execution mode;
- shard counts;
- concurrency used;
- serial fallback indicator;
- cache reuse indicator;
- persistence state;
- bounded failure reasons;
- accessible controls and keyboard labels;
- no raw-content or source-browser surface.

## Enums

### `GnatOperation`

```text
syntax_extract
structure_index
retrieval_prepare
contract_inspect
```

Only `syntax_extract` is admitted in GNAT-01.

### `GnatWorkerState`

```text
queued
claimed
running
complete
failed
cancelled
timed_out
stale
denied
```

### `GnatRunState`

```text
planned
submitted
running
ready
partial_success
degraded
cancelled
denied
failed
stale
```

### `GnatFailureReason`

```text
source_ineligible
source_changed
unsupported_lane
schema_invalid
worker_unavailable
deadline_exceeded
resource_limit
cancelled_by_operator
fa_local_unavailable
output_contract_violation
internal_error_redacted
```

## Contract invariants

1. A run plan is immutable after hashing.
2. A shard ID is deterministic within one run plan.
3. A receipt is invalid when its source fingerprint does not match.
4. Failed workers still emit receipts where technically possible.
5. Missing receipts remain missing; the reconciler may not fabricate success.
6. Partial success must be visible as partial success.
7. Content bodies do not appear in operational diagnostics by default.
8. A result cannot be promoted to app truth without an app-owned bridge.
9. NeuronForge output is a separate semantic-candidate contract.
10. Schema validation failure is fail-closed.

## Versioning

Use explicit version identifiers from the first commit:

```text
GnatRunRequest.v1
GnatRunPlan.v1
GnatShard.v1
GnatWorkerReceipt.v1
GnatRunSummary.v1
GnatDispatchEnvelope.v1
```

No unversioned JSON payloads should enter FA-Local or DF-Local.
