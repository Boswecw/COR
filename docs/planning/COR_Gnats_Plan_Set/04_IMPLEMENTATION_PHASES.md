# Implementation Phases

## Phase 0 — Identity and boundary guardrail

Deliverables:

- Add an ADR stating that this repository is COR, copied from Cortex for the internal/business-side stack.
- State whether `cortex_runtime` is temporarily retained for compatibility.
- Add a COR/Cortex deployment-role matrix.
- Prohibit semantic or workflow expansion during Gnats implementation.

Exit gate:

- Documentation validation passes.
- No runtime behavior changes.

## Phase 1 — Contract scaffold

Deliverables:

- Add the first five Gnat JSON Schemas.
- Add valid and invalid fixtures.
- Extend `scripts/validate_schemas.py` coverage.
- Add contract tests.
- Add reason-code vocabulary.

Exit gate:

- Every schema has at least one valid and three invalid fixtures.
- `make validate` passes.
- Existing runtime tests remain green.

## Phase 2 — Serial Gnat compatibility path

Purpose: prove the contracts before adding concurrency.

Deliverables:

- Add `cortex_runtime/gnats/` package or future `cor_runtime/gnats/` package.
- Add planner, shard model, worker registry, receipt builder, and reconciler.
- Wrap existing Markdown and plain-text extraction functions as Gnat workers.
- Execute shards serially inside COR.
- Emit `GnatRunSummary.v1`.

Exit gate:

- Serial Gnat output matches existing extraction behavior for equivalent inputs.
- Golden fixtures are byte-for-byte stable after normalized timestamps.
- Failure receipts validate.

## Phase 3 — FA-Local dispatch integration

Deliverables:

- Define `GnatDispatchEnvelope.v1`.
- Implement a COR adapter client for FA-Local.
- Add capability negotiation: worker types, concurrency, deadlines, cancellation.
- Preserve serial fallback when FA-Local is unavailable and policy allows it.
- Add correlation IDs end to end.

Exit gate:

- COR never owns worker scheduling in integrated mode.
- FA-Local can cancel a run.
- COR validates all returned receipts.
- An unavailable FA-Local state is reported truthfully.

## Phase 4 — Bounded parallel execution

Deliverables:

- Enable `.md` and `.txt` shards concurrently.
- Default max concurrency 4; hard cap 8.
- Add run and shard deadlines.
- Add retry policy for infrastructure failures only.
- Do not retry deterministic contract failures.
- Add stale-source detection before and after extraction.

Exit gate:

- Results are deterministic independent of completion order.
- No duplicate accepted shard IDs.
- Worker crash produces a visible failed or partial run.
- Parallel run shows meaningful wall-clock improvement on a multi-file fixture.

## Phase 5 — DF-Local persistence and cache

Deliverables:

- Persist run plans, receipts, summaries, and accepted artifact references.
- Add hash/version-based cache records.
- Reuse only exact compatible cache entries.
- Add invalidation on source or worker-version change.
- Add bounded retention policy.

Exit gate:

- Unchanged second run reuses cache.
- Changed source invalidates only affected shards.
- Cache failure never silently changes truth.

## Phase 6 — Operator-Local controls

Deliverables:

- Run progress.
- Worker-count and resource-limit display.
- Cancel control.
- Partial-success and stale-source display.
- Bounded failure reasons.
- No raw-content surveillance surface.

Exit gate:

- All non-ready states are distinguishable.
- Operator can tell parallel, serial-fallback, and cache-reuse modes apart.
- Accessibility and keyboard navigation pass.

## Phase 7 — Expand existing source lanes

Order:

1. PDF text-layer
2. DOCX
3. RTF
4. ODT
5. EPUB

Each lane requires its own benchmark and memory gate. Scrivener remains on its existing special-track governance path and is not admitted merely because Gnats exist.

## Phase 8 — Retrieval-package parallel preparation

Only after extraction receipts are stable:

- create retrieval-preparation workers;
- consume validated extraction artifacts;
- preserve deterministic ordering;
- emit one governed retrieval package;
- avoid semantic ranking.

## Phase 9 — Optional NeuronForge handoff

NeuronForge may receive reconciled syntax artifacts for semantic candidate generation.

Requirements:

- separate contract family;
- explicit user/app request;
- model/resource disclosure;
- semantic result labeled non-canonical;
- no modification of the underlying COR receipts.

## Phase 10 — Shared-core extraction

After two successful application implementations, extract generic scheduling-neutral interfaces into a shared Gnat core. Do not extract prematurely.
