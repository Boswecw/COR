# Test, Validation, and Benchmark Plan

## Test pyramid

### Contract tests

Validate:

- required fields;
- enum closure;
- timestamp format;
- nonnegative counts;
- shard uniqueness;
- receipt/source fingerprint requirements;
- conditional error fields;
- partial-success invariants;
- bounded diagnostics.

### Unit tests

Test pure functions:

- shard ID generation;
- plan hashing;
- source fingerprinting;
- registry lookup;
- receipt building;
- completion-order-independent reconciliation;
- stale detection;
- cache-key generation;
- concurrency clamping.

### Integration tests

Scenarios:

1. Two valid text files complete.
2. One valid and one invalid source produce partial success.
3. Source changes after planning.
4. Worker crashes before receipt; FA-Local synthesizes infrastructure-failure evidence according to contract.
5. Receipt violates schema and is rejected.
6. Duplicate receipt is rejected.
7. Cancellation during run.
8. FA-Local unavailable with serial fallback allowed.
9. FA-Local unavailable with fallback denied.
10. DF-Local unavailable after computation; result remains visible but persistence state is degraded.

### Property-style tests

Without requiring a new framework initially, generate permutations for:

- receipt completion order;
- duplicate delivery;
- retry attempt ordering;
- missing receipts;
- source list order.

Reconciled output must remain deterministic.

## Regression gate

Every phase must pass:

```text
make validate
make test-runtime
make test-gnats
```

No existing source-lane fixture may change output without an explicit contract-version decision.

## Performance benchmarks

### Benchmark sets

```text
Small:   20 files, 10-50 KB each
Medium:  200 files, 10-100 KB each
Large:   1,000 files, bounded total size
Mixed:   Markdown and plain text with controlled failures
```

### Compare

- legacy serial extraction;
- serial Gnat compatibility path;
- parallel Gnat path at 2, 4, and 8 workers;
- warm-cache run;
- cold-cache run.

### Record

```text
wall-clock duration
CPU time
peak resident memory
files per second
bytes per second
receipt validation overhead
reconciliation overhead
failure/cancellation latency
cache hit rate
```

### Acceptance targets for GNAT-01

- Serial Gnat overhead no more than 15% over legacy serial on medium fixture.
- Four-worker path at least 1.7x faster than serial on a suitable multi-file workload.
- Output equivalence for supported extraction fields.
- Peak memory remains within a documented bounded limit.
- Cancellation visible within two seconds for cooperative workers.
- Zero silent shard loss.

Targets are provisional and should be adjusted after baseline measurement on the intended local hardware.

## Resource-pressure tests

- Low available memory.
- Worker count above cap.
- Huge individual source rejected by lane rules.
- Too many files rejected or split by run limit.
- Deadline already expired.
- Slow filesystem.
- Read permission denied.

Expected behavior: reduce concurrency, degrade, deny, or fail closed—never pretend full readiness.

## Privacy tests

Assert that logs/status/receipts do not include:

- unrestricted raw source content;
- full path disclosure when a scoped reference suffices;
- document snippets by default;
- secrets from environment variables;
- model prompts, since GNAT-01 uses no model.

## Security tests

- path traversal;
- symlink escape;
- archive member traversal for later document lanes;
- forged worker ID;
- forged receipt hash;
- mismatched run ID;
- stale source replacement;
- malformed media type;
- unadmitted worker type;
- unauthorized cancellation.
