# Current COR State Audit

## Repository identity issue

The repository is named `COR`, but several constitutional and runtime identifiers still say `Cortex`, including README headings, project charter, doctrine, authority files, Python package name `cortex_runtime`, and service status identity. This is understandable because COR was copied from Cortex, but it is now a material identity-drift risk.

The Gnats work must not combine identity migration with parallel-runtime implementation in one patch. First record the distinction; then perform identity migration as a separate governed slice.

## Existing strengths

The current codebase already has:

- JSON Schema validation with Draft 2020-12.
- Fail-closed service-status emission.
- Runtime slice discovery.
- A shared admitted source-lane registry.
- Syntax-only extraction.
- Retrieval-package emission.
- Provenance and completeness signaling.
- Runtime tests through `unittest`.
- Schema validation via `scripts/validate_schemas.py`.
- A default-denied watcher doctrine.
- Explicit boundaries with DF Local, NeuronForge Local, FA Local, and consuming applications.

## Existing runtime surfaces

```text
slice1  intake validation
slice2  syntax-only extraction emission
slice3  governed retrieval-package emission
slice4  service-status truth
slice5  PDF text-layer lane
slice6  DOCX lane
slice7  RTF lane
slice8  ODT lane
slice9  EPUB lane
slice10 Scrivener Stage 1 authority recon
```

## Existing architectural limits

COR may:

- intake eligible local content;
- normalize admitted inputs;
- extract syntax-level structure and metadata;
- shape retrieval-preparation artifacts;
- package bounded handoffs;
- report freshness, completeness, degradation, and readiness.

COR may not:

- become a semantic authority;
- own model routing or inference;
- become the general execution host;
- govern open-ended workflows;
- become a generic ETL platform;
- silently watch local files;
- treat possession of an artifact as authority over its meaning.

## Consequence for Gnats

Gnats cannot be general agents inside COR. They must be:

- deterministic;
- bounded;
- read-only by default;
- contract-scoped;
- short-lived;
- receipt-producing;
- replaceable;
- unable to self-authorize additional work.

The coordinating execution authority belongs to FA-Local. COR may expose a bounded batch plan and receive completed shard receipts, but it must not become a hidden workflow governor.

## Present-code opportunities

1. `source_lanes` already provides a natural registry for worker capability admission.
2. `extraction_emission` provides the first deterministic function suitable for sharding.
3. `retrieval_package_emission` can consume reconciled extraction results after the first proof.
4. `service_status` already has a runtime-slice vocabulary that can expose Gnat readiness and degradation.
5. Existing schemas and schema validators provide the pattern for new Gnat contracts.
6. Existing fixtures can be multiplied into deterministic batch fixtures.

## Current gaps

- No explicit batch-run contract.
- No shard identity or lease contract.
- No worker receipt.
- No run-level reconciliation result.
- No bounded concurrency configuration.
- No cache contract keyed by source hash and parser version.
- No partial-success semantics for parallel work.
- No explicit FA-Local dispatch envelope for COR worker jobs.
- No Gnat-specific status fields.
- No performance benchmark harness.

## Required preliminary decision

Choose one of these identity strategies before production release:

### Strategy A — Preserve internal package compatibility

Keep `cortex_runtime` temporarily, but change external service identity to `cor` and document the package name as legacy compatibility.

### Strategy B — Full COR rename

Migrate package and documentation to `cor_runtime` in a dedicated, test-protected change.

Recommended: **Strategy A for GNAT-01**, followed by a separate identity migration. This avoids mixing behavioral and naming changes.
