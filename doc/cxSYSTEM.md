# Cortex - System Documentation

**Document version:** 1.5 (2026-03-19) - Aligned to current constitutional and runtime repo state
**Protocol:** Forge Documentation Protocol v1

| Key | Value |
|-----|-------|
| **Project** | Cortex |
| **Prefix** | `cx` |
| **Output** | `doc/cxSYSTEM.md` |

This `doc/system/` tree is the assembled system reference for Cortex as a bounded local file-intelligence service.
It reflects the current repo state through Wave 3 hardening, audit-remediation tightening, the shared source-lane framework, and Runtime Slices 1 through 7.

Assembly contract:

- Command: `bash doc/system/BUILD.sh`
- Output: `doc/cxSYSTEM.md`

| Part | File | Contents |
|------|------|----------|
| SS1 | [01-overview-charter.md](01-overview-charter.md) | Mission, role, success posture, and current bounded runtime baseline |
| SS2 | [02-boundaries-and-doctrine.md](02-boundaries-and-doctrine.md) | Authority boundaries, syntax-before-semantics doctrine, and anti-control-plane posture |
| SS3 | [03-contract-surface.md](03-contract-surface.md) | Intake, extraction, retrieval, handoff, service-status, and diagnostics surfaces |
| SS4 | [04-validation-and-delivery.md](04-validation-and-delivery.md) | Validation wiring, schema-backed enforcement, delivered slices, and current delivery posture |

## Quick Assembly

```bash
bash doc/system/BUILD.sh
```

*Last updated: 2026-03-19*

---

# 1. Overview and Charter

## Purpose

Cortex is the bounded local file-intelligence, extraction, retrieval-preparation, and handoff-support service for Forge applications.

Its Phase 1 purpose is narrow:

- intake eligible local content
- extract syntax-level structure and metadata
- prepare one governed retrieval package form
- emit bounded handoff transfer truth
- surface truthful operational state without privacy collapse
- expose redacted embedded diagnostics for Cortex-owned surfaces only

## Constitutional role

Cortex is a service-only internal runtime subsystem.

It must not become:

- a standalone product
- a semantic engine
- a workflow coordinator
- a generic ETL platform
- a canonical truth store

## Success posture

Cortex is only successful if it remains:

- bounded by contract
- fail-closed when trust is insufficient
- privacy-preserving by default
- non-semantic by default
- freshness-bound rather than silently stale
- visible only through consuming applications
- unable to drift into workflow, queue, or executor ownership

## Current bounded runtime baseline

The currently implemented executable runtime surfaces are:

- Slice 1 - intake validation
- Slice 2 - syntax-only extraction-result emission
- Slice 3 - one governed retrieval-package emission path
- Slice 4 - service-status truth
- Slice 5 - bounded local PDF source lane
- Slice 6 - bounded local DOCX source lane
- Slice 7 - bounded local RTF source lane

The currently admitted source lanes remain narrow:

- local `.md`
- local `.txt`
- local text-layer `.pdf`
- local `.docx`
- local `.rtf`

This is the current bounded baseline, not a promise of broader source or control-surface expansion.

## Foundational references

This section is grounded in:

- `PROJECT_CHARTER.md`
- `PHASE_1_PLAN.md`
- `README.md`

---

# 2. Boundaries and Doctrine

## Authority line

Cortex owns:

- intake contracts
- syntax-level extraction
- provenance and completeness signaling
- retrieval-preparation support
- handoff packaging support
- freshness and invalidation signaling for Cortex-owned artifacts
- privacy-preserving diagnostics for Cortex-owned surfaces

Cortex does not own:

- semantic interpretation
- model authority
- workflow sequencing
- retry or queue semantics
- executor selection
- downstream execution ownership
- canonical business truth
- broad surveillance authority

## Doctrine line

The governing doctrines are:

- service-only visibility
- syntax before semantics
- fail closed over convenience
- retrieval infrastructure, not retrieval authority
- explicit invalidation over assumed freshness
- default-denied observation
- bounded reverse signaling only
- informational service status, not control-plane behavior

## Cross-service boundaries

### DF Local Foundation

Provides substrate support.
Does not absorb Cortex file-intelligence logic.

### NeuronForge Local

Consumes syntax-level packages for semantic work.
Does not make Cortex a semantic authority.

### FA Local

Owns policy-gated execution routing.
Does not delegate execution authority into Cortex.

## Anti-drift warning

Any proposal that turns Cortex into a semantic surface, workflow router, retry coordinator, surveillance surface, status-control plane, or generalized transform sink should be rejected unless the architecture is explicitly reworked.

No automatic next slice is implied by the current runtime baseline.
Further runtime expansion must be explicit, narrow, and grounded back to the constitutional plan.

---

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
Runtime Slice 7 adds one bounded local RTF lane only.
That lane admits local `.rtf` files, remains paragraph-only, supports basic escaped character recovery only as needed for honest plain-text extraction, denies annotation, review, field, media, and other rich destinations outside the lane, and marks corrupt or syntactically untrustworthy sources unavailable.

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
Ready RTF extraction results remain compatible with the same path through paragraph-bounded chunking only.

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
- `docs/contracts/source-lane-rtf.md`
- `docs/contracts/retrieval-package.md`
- `docs/contracts/handoff-envelope.md`
- `docs/contracts/service-status.md`
- `docs/contracts/embedded-diagnostics.md`
- `docs/source-lanes/README.md`

---

# 4. Validation and Delivery

## Validation surface

Cortex now includes:

- JSON schemas in `schemas/`
- valid fixtures in `tests/contracts/fixtures/valid/`
- invalid fixtures in `tests/contracts/fixtures/invalid/`
- a lightweight validator at `scripts/validate_schemas.py`
- repo-level validation through `make validate`
- automatic fixture discovery by schema-prefix naming
- explicit schema-contract checks for handoff reverse signaling, denial taxonomy, anti-orchestration guards, and embedded diagnostics privacy boundaries

The current machine-checked contract layer covers:

- intake request
- extraction result
- retrieval package
- service status
- handoff envelope
- embedded diagnostics

## Runtime slice 1 delivered

The first executable runtime slice is now present for intake validation only.

It adds:

- a minimal in-process intake validation module
- a local CLI path for validating a candidate intake payload
- focused runtime tests that reuse contract fixtures
- explicit fail-closed handling for malformed JSON and unreadable payload files

## Runtime slice 2 delivered

The second executable runtime slice is now present for syntax-only extraction emission only.

It adds:

- a bounded extraction emitter for local `.md` and `.txt` sources
- reuse of the intake-validation slice before extraction emission
- schema-valid `ready`, `denied`, and `unavailable` extraction-result outputs
- focused runtime tests for supported, unsupported, unreadable, and malformed-input paths

## Runtime slice 3 delivered

The third executable runtime slice is now present for one governed retrieval-package emission path only.

It adds:

- a retrieval-package emitter driven by ready syntax-only extraction-result input
- deterministic section-bounded chunking with paragraph fallback only when section structure is absent
- schema-valid `ready` and fail-closed `denied` retrieval-package outputs
- focused runtime tests for deterministic ordering, unsupported paths, stale upstream input, malformed upstream input, and infrastructure-only output

## Runtime slice 4 delivered

The fourth executable runtime slice is now present for one governed service-status truth path only.

It adds:

- a service-status emitter driven by bounded local runtime truth rather than broad environment probing
- schema-valid `ready`, `degraded`, and `unavailable` service-status outputs
- explicit reporting of implemented runtime slices and admitted source lanes
- focused runtime tests for ready, degraded, unavailable, CLI, and informational-only output posture

## Runtime slice 5 delivered

The fifth executable runtime slice is now present for one bounded local PDF source lane only.

It adds:

- a text-layer-only PDF extraction path using bounded local PDF tooling already present on the host
- explicit deny behavior for encrypted PDFs and PDFs with no extractable text layer
- explicit unavailable behavior for corrupt PDFs or unavailable PDF tooling
- optional `partial_success` when some PDF pages are extractable and others are text-layer-free
- retrieval-package compatibility for ready PDF extraction outputs through the existing deterministic paragraph path
- focused runtime tests for text, encrypted, scanned, partial, corrupt, and retrieval-compatible PDF paths

## Shared lane framework delivered

The runtime now exposes a shared source-lane model rather than only format-specific branches.

It adds:

- explicit admitted-lane registration
- shared admission checks
- shared failure taxonomy wiring
- shared provenance metadata for lane identity
- shared service-status lane reporting

## Runtime slice 6 delivered

The sixth executable runtime slice is now present for one bounded local DOCX source lane only.

It adds:

- a bounded local `.docx` extraction path using OpenXML package reads only
- deterministic recovery of headings, paragraphs, simple lists, and bounded table text
- explicit deny behavior for comments and tracked changes
- explicit unavailable behavior for corrupt or unreadable DOCX packages
- retrieval-package compatibility for ready DOCX extraction outputs through the existing deterministic section path
- focused runtime tests for ready, denied, unavailable, deterministic, retrieval-compatible, and cross-lane invariant behavior

## Runtime slice 7 delivered

The seventh executable runtime slice is now present for one bounded local RTF source lane only.

It adds:

- a bounded local `.rtf` extraction path using an in-repo stdlib parser rather than external conversion tooling
- paragraph-only recovery with basic escaped character support only as needed for honest plain-text extraction
- explicit deny behavior for annotation, review, field, object, media, and other rich destinations outside the lane
- explicit unavailable behavior for corrupt or syntactically untrustworthy RTF sources
- retrieval-package compatibility for ready RTF extraction outputs through the existing deterministic paragraph path
- focused runtime tests for ready, denied, unavailable, deterministic, retrieval-compatible, and cross-lane invariant behavior

## Delivery order

The current delivery order remains:

1. constitutional base docs
2. doctrine and boundary ADRs
3. architecture boundary matrix
4. contracts and schemas
5. fixtures and validation

## Wave 3 hardening delivered

Wave 3 adds:

- the handoff envelope contract and schema
- valid handoff fixtures for basic, stale, and denied paths
- invalid handoff fixtures for missing integrity context, invalid reverse signaling, invalid denial taxonomy, and orchestration creep
- automatic validator wiring so new schema-prefixed fixtures are picked up without manual script edits

## Audit remediation tightening

The current remediation pass adds:

- a strict embedded diagnostics schema with privacy-preserving defaults
- boundary fixtures for service status, retrieval package, and extraction result branches that were previously under-exercised
- handoff alignment so reverse signaling remains optional rather than forced on every forward transfer envelope
- doctrine alignment so invalidation is represented through stale posture and invalidation policy rather than a separate workflow-like wire state

## Current repo posture

The repo is currently strongest where constitutional claims are backed by schemas, invalid fixtures, and validator guard checks.

Slices 1 through 7 now form the current bounded runtime baseline.
No further implementation target is implied by this system reference alone.
Any next step should be explicit, narrow, and anchored to the governing plan rather than inferred from momentum.

This assembled system doc is therefore a control reference, not a product or roadmap document.

## Assembly purpose

`doc/cxSYSTEM.md` is intended to give a single assembled system reference without replacing the canonical source files that define the actual doctrine and contracts.
