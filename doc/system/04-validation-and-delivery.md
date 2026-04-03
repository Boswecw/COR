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

## Scrivener Stage 1 authority-recon delivery

The current special-track implementation phase adds:

- one bounded Scrivener Stage 1 authority-recon runtime slice only
- one status-only contract and schema surface for `ready`, `denied`, and `unavailable` outcomes
- bounded runtime coverage for singular-authority, missing-authority, multi-authority, malformed-authority, unsupported-source, operator-disabled, and directly incomplete-correspondence cases
- service-status reporting for the implemented special-track slice without promoting Scrivener into the admitted source-lane set

This phase does not add Scrivener extraction, manuscript-policy handling, generalized Scrivener schemas, or lane admission.

## Scrivener Stage 2 planning-control packet

The current post-Stage-1 governance phase adds:

- one canonical Scrivener Stage 2 planning packet
- companion Stage 2 evidence-and-gate, contract, manuscript-boundary, degraded-truth, and item-type control notes
- one bounded fixture acquisition and evidence expansion plan that sequences compatibility, boundary, and degraded-case evidence growth

This phase opens Stage 2 planning only.
It does not add runtime behavior, extraction authority, schema changes, or lane admission.

## PDF lane admission hardening delivered

The current hardening pass strengthens the PDF lane for honest downstream admission by AuthorForge or equivalent consumers.

It adds:

- a structured host admission probe (`probe_pdf_lane_admission()`) that reports `pdfinfo` and `pdftotext` presence separately so consumers can distinguish which tool is missing rather than receiving a boolean only
- updated lane eligibility truth so the tools-missing message is derived from the probe rather than a static string, giving per-tool specificity in the operator-visible summary
- a malformed-PDF detection helper (`_pdf_stderr_indicates_malformed()`) that matches unambiguous file-structure error patterns from `pdfinfo` and `pdftotext` stderr and gates a "malformed or corrupt" summary distinct from generic tool invocation failure
- per-tool-specific summaries across all PDF failure paths so operators can distinguish `pdfinfo` absent, `pdftotext` absent, both absent, malformed PDF, and tool invocation failure without collapsing them into a single vague message
- a `PdfLaneMockedAdmissionTests` class that runs on any host without requiring PDF tools: proves tools-missing emits `unavailable`, proves per-tool summary specificity, proves no subprocess is invoked when tools are absent, and proves no OCR or unexpected subprocess fires during extraction failure
- a `PdfLaneAdmissionProbeTests` class that proves the probe accurately reports each combination of tool presence and that the probe itself never invokes any tool
- a `PdfLaneRuntimeMalformedTests` class that proves a corrupt PDF produces a summary explicitly naming "malformed" or "corrupt" rather than a generic failure message
- an updated `docs/contracts/source-lane-pdf.md` with admission-readiness status, runtime dependency table, probe usage example, and a complete denial/unavailability taxonomy table mapping every failure condition to its state, reason_class, and summary content

This pass does not add OCR, cloud services, new format support, or AuthorForge integration.
The PDF lane remains bounded to local text-layer extraction only.

## Scrivener Stage 2 implementation-remains-blocked decision

The current governance response adds:

- one explicit decision reviewing the current Scrivener Stage 2 planning-control packet
- one explicit determination that Stage 2 implementation remains blocked
- explicit preservation of fail-closed posture until compatibility, mapping, boundary, degraded-truth, and dependency evidence are materially broader

This phase does not add runtime behavior, extraction authority, schema changes, or lane admission.

## Type-checking hygiene pass delivered

The current pass resolves mypy errors across the runtime and test surfaces without changing runtime behavior.

It adds:

- `mypy.ini` at repo root with `[mypy-jsonschema.*] ignore_missing_imports = true` as a project-level override
- `# type: ignore[import-untyped]` on every `from jsonschema import Draft202012Validator` line across `cortex_runtime/` and `tests/runtime/` so the suppression is source-of-truth regardless of how the mypy extension discovers config
- an `isinstance(uuid, str)` guard at the top of the `direct_missing_targets` loop in `scrivener_authority_recon.py` so mypy can narrow the dict key from `str | None` to `str` before the `Path /` operator and `list.append` calls
- corrected return type on `build_supported_intake_payload` in `test_extraction_emission.py` from `dict[str, object]` to `dict[str, Any]` so nested subscript operations type-check
- corrected return type on local `load_json` in `test_intake_validation.py` from `object` to `Any` so indexed assignment on the returned payload type-checks

This pass does not change runtime behavior, schemas, contracts, or extraction logic.

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

Slices 1 through 9 plus the special-track Scrivener Stage 1 authority-recon slice now form the current bounded runtime baseline.
This baseline has also been hardened for contract symmetry, operator consistency, and future lane-admission governance.
EPUB is now admitted as a bounded local source lane.
The PDF lane has been hardened for admission truth with a structured host admission probe, per-tool unavailability distinction, malformed-PDF detection, and explicit no-OCR test coverage.
Scrivener remains a special-track project-source opening rather than an admitted source lane.
Only the bounded Stage 1 authority-recon runtime slice is implemented.
The current Scrivener Stage 2 packet is planning and control only.
Decision 0016 keeps Stage 2 implementation blocked pending materially broader evidence.
HTML remains deferred.
No further implementation target is implied by this system reference alone beyond that bounded Stage 1 authorization.
Any future lane work must still be explicit, narrow, and anchored to the governing plan rather than inferred from planning momentum alone.

This assembled system doc is therefore a control reference, not a product or roadmap document.

## Assembly purpose

`doc/cxSYSTEM.md` is intended to give a single assembled system reference without replacing the canonical source files that define the actual doctrine and contracts.
