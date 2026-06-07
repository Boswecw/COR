# File-Level Change Map

This map is grounded in the current repository conventions shown by `README.md`, constitutional files, `cortex_runtime`, `schemas`, `scripts`, and `tests/runtime`.

## New documentation

```text
DECISIONS/0017-cor-identity-and-cortex-lineage.md
DECISIONS/0018-gnat-bounded-parallel-worker-authorization.md
DECISIONS/0019-fa-local-owns-gnat-execution-routing.md
docs/architecture/gnats-boundary-matrix.md
docs/contracts/gnat-run-request.md
docs/contracts/gnat-run-plan.md
docs/contracts/gnat-worker-receipt.md
docs/contracts/gnat-run-summary.md
docs/integration/fa-local-gnat-dispatch.md
```

Use the next available ADR numbers if the repository has advanced beyond 0016.

## New schemas

```text
schemas/gnat-run-request.schema.json
schemas/gnat-run-plan.schema.json
schemas/gnat-shard.schema.json
schemas/gnat-worker-receipt.schema.json
schemas/gnat-run-summary.schema.json
schemas/gnat-dispatch-envelope.schema.json
```

Deferred:

```text
schemas/gnat-cache-record.schema.json
schemas/gnat-semantic-handoff.schema.json
```

## Runtime package

Retaining the current package name for GNAT-01:

```text
cortex_runtime/gnats/__init__.py
cortex_runtime/gnats/models.py
cortex_runtime/gnats/schema_validation.py
cortex_runtime/gnats/planner.py
cortex_runtime/gnats/registry.py
cortex_runtime/gnats/receipt.py
cortex_runtime/gnats/reconcile.py
cortex_runtime/gnats/serial_runner.py
cortex_runtime/gnats/fa_local_client.py
cortex_runtime/gnats/status.py
cortex_runtime/gnats/workers/__init__.py
cortex_runtime/gnats/workers/markdown_text.py
cortex_runtime/gnats/workers/plain_text.py
```

When the repository completes identity migration, move these to `cor_runtime/gnats/` in a separate refactor.

## Existing files to modify

### `cortex_runtime/service_status.py`

- Change external `SERVICE_ID` to `cor` after identity ADR approval.
- Add a bounded Gnat capability summary.
- Do not claim ready parallel execution unless FA-Local capability negotiation succeeds.
- Preserve fail-closed fallback behavior.

### `cortex_runtime/source_lanes.py`

- Add worker-type metadata to admitted lane declarations, or add a companion Gnat registry.
- Do not make every admitted source lane automatically parallel-enabled.
- Add explicit `gnat_parallel_admitted` truth per lane.

### `cortex_runtime/extraction_emission.py`

- Extract a pure deterministic function that can be invoked by both the legacy serial CLI and Gnat workers.
- Keep CLI behavior compatible.
- Avoid embedding scheduling logic here.

### `cortex_runtime/retrieval_package_emission.py`

- No Phase 2 behavior change beyond accepting validated reconciled extraction references later.

### `scripts/validate_schemas.py`

- Register Gnat schemas and fixtures.
- Add cross-schema `$ref` validation if not already present.

### `README.md`

- Correct COR identity.
- Add Gnat status only after a slice is complete.
- Preserve snapshot-vs-doctrine wording.

### Constitutional files

- Update identity carefully.
- Preserve the existing syntax-before-semantics, fail-closed, privacy, watcher, and anti-orchestration rules.
- Add a narrow exception clarifying that finite batch planning is not workflow ownership when FA-Local owns execution.

## Tests

```text
tests/contracts/fixtures/valid/gnat-run-request-basic.json
tests/contracts/fixtures/valid/gnat-run-plan-two-text-files.json
tests/contracts/fixtures/valid/gnat-worker-receipt-complete.json
tests/contracts/fixtures/valid/gnat-worker-receipt-failed.json
tests/contracts/fixtures/valid/gnat-run-summary-ready.json
tests/contracts/fixtures/valid/gnat-run-summary-partial.json

tests/contracts/fixtures/invalid/gnat-*.json

tests/runtime/test_gnat_planner.py
tests/runtime/test_gnat_registry.py
tests/runtime/test_gnat_receipts.py
tests/runtime/test_gnat_reconcile.py
tests/runtime/test_gnat_serial_runner.py
tests/runtime/test_gnat_fa_local_client.py
tests/runtime/test_gnat_status.py
tests/runtime/test_gnat_determinism.py
tests/runtime/test_gnat_stale_source.py
tests/runtime/test_gnat_privacy.py
```

## Fixtures

```text
tests/runtime/fixtures/gnats/text-batch-small/
tests/runtime/fixtures/gnats/text-batch-medium/
tests/runtime/fixtures/gnats/mixed-valid-invalid/
tests/runtime/fixtures/gnats/source-mutates-during-run/
```

## Commands

Add Make targets:

```make
.PHONY: test-gnats benchmark-gnats

test-gnats:
	$(PYTHON) -m unittest discover -s tests/runtime -p 'test_gnat_*.py' -t .

benchmark-gnats:
	$(PYTHON) scripts/benchmark_gnats.py
```
