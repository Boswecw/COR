# 3. Contract Surface

## Phase 1 contract set

The current contract surface covers:

- intake request
- extraction result
- retrieval package
- handoff envelope
- service status
- embedded diagnostics

## Intake request

The intake contract requires explicit source identity, source class, artifact request type, normalization mode, and observation posture.

Observation defaults to denied.
If a watcher is requested, it must be contract-scoped, operator-visible, removable, and bounded by source class.

Runtime Slice 1 now executes this contract mechanically through schema-backed intake validation.
The runtime output remains limited to accepted or denied validation truth plus bounded contract-error paths.

## Extraction result

Extraction results are syntax-only by contract.

They must expose:

- provenance
- completeness posture
- freshness posture when relevant
- refusal posture when requests cross into semantics or other denied boundaries

Runtime Slice 2 now emits bounded extraction-result outputs for supported local `.md` and `.txt` sources only.
Unsupported, unreadable, malformed, or intake-invalid inputs fail closed through `denied` or `unavailable` extraction results.

Runtime Slice 5 adds one bounded local PDF lane only.
That lane admits text-layer `.pdf` files, remains text-only and non-OCR, allows `ready` only for trustworthy extractable text, allows `partial_success` only when some pages lack extractable text, denies encrypted or text-layer-free PDFs, and marks corrupt or unreadable PDFs unavailable.

The extraction runtime now also uses a shared source-lane framework for lane admission, shared provenance metadata, shared failure posture, and admitted-lane reporting.
Runtime Slice 6 adds one bounded local DOCX lane only.
That lane admits local `.docx` packages, remains syntax-only, recovers headings only from explicit paragraph-style evidence, recovers simple lists and bounded table text only when deterministic, denies comments or tracked changes, and marks corrupt or unreadable packages unavailable.

## Retrieval package

Retrieval packages are:

- non-canonical
- non-semantic by default
- freshness-bound

They require explicit retrieval profile, freshness, invalidation, and completeness fields.
They do not decide ranking, canonical truth, or downstream semantic acceptance.

Runtime Slice 3 now emits one governed retrieval-package path from ready syntax-only extraction output only.
Chunking remains deterministic and syntax-derived, using section-bounded chunks when available and paragraph fallback only when no section structure exists.
Ready PDF extraction results remain compatible with this path through the same paragraph-bounded fallback rather than any PDF-specific semantic shaping.
Ready DOCX extraction results remain compatible with the same path through section-bounded chunking when explicit heading structure exists.

## Service status

Service status exposes truthful operator-visible state with the narrow Phase 1 vocabulary:

- `ready`
- `degraded`
- `unavailable`
- `denied`
- `stale`
- `partial_success`

It remains an operational truth surface only.
It does not become a raw-content channel or downstream coordination surface.

Runtime Slice 4 now emits one governed service-status path from bounded local runtime truth only.
It reports implemented runtime slices, admitted source lanes, zero active watcher scopes, and ready/degraded/unavailable posture without adding recommendation or control-plane behavior.
The admitted-source-lane report is now driven from the shared lane registry rather than ad hoc extraction-module inspection.

## Handoff envelope

The handoff envelope is a bounded transfer-truth surface only.

It may express:

- `ready_for_transfer`
- `denied`
- `stale`
- `integrity_failed`
- `re_prep_required`

`reverse_signal` is optional.
When present, it must stay within the bounded reverse-signaling enum and must not become a workflow protocol.

It must reject orchestration-shaped fields such as:

- `retry_count`
- `workflow_id`
- `queue_name`
- `dispatch_plan`
- `agent_assignment`

## Embedded diagnostics

Embedded diagnostics are now schema-backed rather than prose-only.

They are:

- redacted by default
- limited to Cortex-owned operational truth
- allowed to expose bounded state, freshness, integrity, completeness, watcher scope, and denial summaries

They must reject content-exposure shapes such as:

- `raw_content_preview`
- `full_text_preview`
- `full_text_search`
- `content_browser`
- `raw_artifact_dump`

## Implemented schema layer

The current machine-checked schema inventory is:

- `schemas/intake-request.schema.json`
- `schemas/extraction-result.schema.json`
- `schemas/retrieval-package.schema.json`
- `schemas/service-status.schema.json`
- `schemas/handoff-envelope.schema.json`
- `schemas/embedded-diagnostics.schema.json`

## Supporting references

This section is grounded in:

- `docs/contracts/intake-request.md`
- `docs/contracts/extraction-result.md`
- `docs/contracts/source-lane-docx.md`
- `docs/contracts/source-lane-pdf.md`
- `docs/contracts/retrieval-package.md`
- `docs/contracts/handoff-envelope.md`
- `docs/contracts/service-status.md`
- `docs/contracts/embedded-diagnostics.md`
- `docs/source-lanes/README.md`
