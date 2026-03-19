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
