# Codex Implementation Prompts

These prompts are intentionally sliced. Run them one at a time and require a review after each.

## Prompt 1 — Identity and Gnats authorization ADRs

```text
Repository: Boswecw/COR, branch main.

Create a documentation-only change that:
1. Adds an ADR clarifying that COR is the internal/business-side lineage copied from Cortex, while Cortex is used for public-facing application deployments.
2. Records that the Python package name `cortex_runtime` is temporarily retained for compatibility and will be migrated separately.
3. Adds an ADR authorizing bounded deterministic Gnat workers while explicitly preserving syntax-before-semantics, default-denied watchers, fail-closed behavior, and the rule that FA-Local owns execution routing.
4. Adds a boundary matrix for COR, FA-Local, NeuronForge, DF-Local, Operator-Local, and consuming apps.
5. Do not modify runtime code.
6. Run existing documentation/schema validation and report results.
```

## Prompt 2 — Gnat contract scaffold

```text
Add versioned JSON Schemas and fixtures for:
- GnatRunRequest.v1
- GnatRunPlan.v1
- GnatShard.v1
- GnatWorkerReceipt.v1
- GnatRunSummary.v1

Follow the repository's Draft 2020-12 schema conventions and fail-closed validation style. Add valid and invalid fixtures and update schema-validation tooling. Do not implement runtime execution. Preserve existing contracts unchanged. Run `make validate` and `make test-runtime`.
```

## Prompt 3 — Serial compatibility proof

```text
Implement a new `cortex_runtime/gnats/` package that can:
- plan a finite batch of admitted Markdown/plain-text syntax extraction shards;
- execute them serially through wrappers around the existing deterministic extraction code;
- emit one schema-valid receipt per shard;
- reconcile receipts into GnatRunSummary.v1;
- detect source fingerprint changes;
- preserve existing extraction CLI output and tests.

No concurrency, FA-Local, DF-Local, NeuronForge, watchers, or mutation in this slice. Add comprehensive unit and runtime tests.
```

## Prompt 4 — FA-Local dispatch adapter

```text
Add the COR side of GnatDispatchEnvelope.v1 and a narrow FA-Local client interface. The implementation must:
- negotiate supported contract versions, worker types, max concurrency, deadlines, and cancellation;
- submit an immutable hashed run plan;
- accept receipts asynchronously or as a completed batch through an abstract transport interface;
- validate every receipt inside COR;
- support a configured serial fallback;
- report FA-Local unavailable/degraded truth in service status.

Do not implement a general scheduler in COR.
```

## Prompt 5 — Parallel execution proof

```text
Using the approved FA-Local interface, enable bounded parallel execution for Markdown and plain-text syntax extraction only. Default max concurrency is 4 and hard cap is 8. Add cancellation, run/shard deadlines, stale-source detection, infrastructure-only retries, deterministic reconciliation, and partial-success reporting. Benchmark against the existing serial path and include results in a Markdown report.
```

## Prompt 6 — DF-Local receipts and cache

```text
Add contract-bound DF-Local persistence for Gnat plans, receipts, summaries, and cache records. Exact cache reuse requires source fingerprint, worker version, operation contract version, and lane contract version to match. Cache/persistence failure must be reported separately and must not fabricate extraction failure or success. Add invalidation and retention tests.
```

## Prompt 7 — Operator-Local integration packet

```text
Produce the COR-side API/status/event contracts needed for Operator-Local to show Gnat run progress, cancellation, concurrency, serial fallback, cache reuse, partial success, stale shards, and bounded failure reason codes. Do not add raw-content preview or surveillance surfaces. Include accessibility-oriented state labels and error copy.
```
