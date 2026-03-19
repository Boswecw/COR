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

## Runtime slice 8 delivered

The eighth executable runtime slice is now present for one bounded local ODT source lane only.

It adds:

- a bounded local `.odt` extraction path using zip/XML package parsing only
- deterministic recovery of paragraphs, explicit headings, simple lists, and bounded table text
- explicit deny behavior for annotations, tracked changes, embedded objects, and other out-of-lane package structures
- explicit unavailable behavior for corrupt, unreadable, or missing-content ODT packages
- retrieval-package compatibility for ready ODT extraction outputs through the existing deterministic section path
- focused runtime tests for ready, denied, unavailable, deterministic, retrieval-compatible, and cross-lane invariant behavior

## Runtime slice 9 delivered

The ninth executable runtime slice is now present for one bounded local EPUB source lane only.

It adds:

- a bounded local `.epub` extraction path using zip/XML package parsing only
- explicit package authority recovery through EPUB mimetype, container, package document, manifest, and spine truth
- deterministic recovery of paragraphs, explicit headings, simple lists, and bounded table text from admitted XHTML spine members
- explicit deny behavior for active, scripted, media, and other out-of-lane EPUB content structures
- explicit unavailable behavior for corrupt, unreadable, missing-authority, malformed-XML, or manifest/spine-broken EPUB packages
- retrieval-package compatibility for ready EPUB extraction outputs through the existing deterministic section path
- focused runtime tests for ready, denied, unavailable, deterministic, retrieval-compatible, and cross-lane invariant behavior

## Post-slice-7 hardening delivered

The current hardening pass adds:

- an explicit text-lane contract so `.md` and `.txt` no longer rely on implicit runtime truth alone
- a contract-symmetry audit note covering schemas, lane docs, ADRs, service-status truth, and retrieval truth
- deeper ugly-case runtime coverage for empty text inputs, CRLF text normalization, PDF tooling anomalies, DOCX structural edge cases, and RTF escape-path recovery
- direct-source CLI media-type support so suffix and media-type mismatch behavior is visible through operator entrypoints
- a formal lane-admission playbook for future format evaluation without format-momentum drift

## Next-lane evaluation delivered

The current governance phase adds:

- a formal candidate comparison across ODT, HTML, EPUB, and Scrivener
- an explicit recommendation of ODT as the next candidate target only
- a selection ADR recording why HTML and EPUB were deferred and why Scrivener remains a special project-source candidate
- a draft ODT admission posture without admitting or implementing the lane yet

This phase does not add runtime behavior, schema changes, or a new admitted lane.
That selection has now been executed through Runtime Slice 8.

## Post-slice-8 governance selection delivered

The current governance phase adds:

- a fresh post-ODT candidate comparison across HTML, EPUB, and special-track Scrivener
- an explicit selection of EPUB as the next planning target only
- a selection ADR recording why HTML remains deferred and why Scrivener remains special-track
- a draft EPUB admission note for planning only

That governance selection has now been executed through Runtime Slice 9.

## Post-slice-9 governance selection delivered

The current governance phase adds:

- a fresh post-EPUB candidate comparison across HTML and special-track Scrivener
- an explicit selection of Scrivener as the next planning target only
- a selection ADR recording why HTML remains deferred and why Scrivener must be treated as a special-track project-source opening rather than routine lane expansion
- a draft Scrivener admission note for planning only

This phase does not add runtime behavior, schema changes, or a new admitted lane.

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

Slices 1 through 9 now form the current bounded runtime baseline.
This baseline has also been hardened for contract symmetry, operator consistency, and future lane-admission governance.
EPUB is now admitted as a bounded local source lane.
Scrivener is now the next governance-selected planning target through an explicit special-track project-source path.
HTML remains deferred.
No further implementation target is implied by this system reference alone beyond that planning authorization.
Any future lane work must still be explicit, narrow, and anchored to the governing plan rather than inferred from planning momentum alone.

This assembled system doc is therefore a control reference, not a product or roadmap document.

## Assembly purpose

`doc/cxSYSTEM.md` is intended to give a single assembled system reference without replacing the canonical source files that define the actual doctrine and contracts.
