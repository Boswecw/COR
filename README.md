# Cortex

Cortex is the bounded local file-intelligence, extraction, retrieval-preparation, and handoff-support service for Forge applications.

This directory starts with constitutional artifacts first:

- project charter
- doctrine
- authority boundaries
- service-domain definitions
- degradation and control-surface rules
- anti-drift ADRs
- architecture boundary references

Current status:

- Wave 1 constitutional scaffold created from `cortex_constitutional_project_plan_v_2.md`
- Wave 2 and Wave 3 contract/schema enforcement in place
- Runtime Slice 1 intake validation path implemented against `schemas/intake-request.schema.json`
- Runtime Slice 2 syntax-only extraction emission path implemented for bounded local `.md` and `.txt` sources
- Runtime Slice 3 governed retrieval-package emission path implemented from ready syntax-only extraction output
- Runtime Slice 4 service-status truth path implemented for current bounded runtime surfaces and admitted source lanes
- Runtime Slice 5 bounded local PDF source lane implemented for text-layer `.pdf` files only
- Shared source-lane framework extracted for admitted-lane registration, failure posture, provenance, and service-status reporting
- Runtime Slice 6 bounded local DOCX source lane implemented for local `.docx` files only

Start here:

- `PROJECT_CHARTER.md`
- `LOCAL_DOCTRINE.md`
- `AUTHORITY_BOUNDARIES.md`
- `PHASE_1_PLAN.md`
- `docs/architecture/boundary-matrix.md`

Runtime Slice 1:

- validate a candidate intake payload: `python -m cortex_runtime.intake_validation tests/contracts/fixtures/valid/intake-request-file-basic.json`
- run runtime tests: `make test-runtime`

Runtime Slice 2:

- emit a syntax-only extraction result from a bounded local file: `python -m cortex_runtime.extraction_emission --source-path tests/runtime/fixtures/sample-note.md --request-id local-001 --source-ref src-local`

Runtime Slice 3:

- emit a retrieval package from the same bounded local file path: `python -m cortex_runtime.retrieval_package_emission --source-path tests/runtime/fixtures/sample-note.md --request-id local-001 --source-ref src-local`

Runtime Slice 4:

- emit current bounded service-status truth: `python -m cortex_runtime.service_status`

Runtime Slice 5:

- emit a syntax-only extraction result from a bounded local text-layer PDF: `python -m cortex_runtime.extraction_emission --source-path tests/runtime/fixtures/sample-note.pdf --request-id pdf-001 --source-ref pdf-local`
- emit a retrieval package from the same bounded local PDF lane: `python -m cortex_runtime.retrieval_package_emission --source-path tests/runtime/fixtures/sample-note.pdf --request-id pdf-001 --source-ref pdf-local`

Runtime Slice 6:

- emit a syntax-only extraction result from a bounded local DOCX source: `python -m cortex_runtime.extraction_emission --source-path tests/runtime/fixtures/sample-note.docx --request-id docx-001 --source-ref docx-local`
- emit a retrieval package from the same bounded local DOCX lane: `python -m cortex_runtime.retrieval_package_emission --source-path tests/runtime/fixtures/sample-note.docx --request-id docx-001 --source-ref docx-local`
