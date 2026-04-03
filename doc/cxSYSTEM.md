# Cortex - System Documentation

**Document version:** 1.14 (2026-03-20) - Type-checking hygiene pass
**Protocol:** Forge Documentation Protocol v1

| Key | Value |
|-----|-------|
| **Project** | Cortex |
| **Prefix** | `cx` |
| **Output** | `doc/cxSYSTEM.md` |

This `doc/system/` tree is the assembled system reference for Cortex as a bounded local file-intelligence service.
It reflects the current repo state through Wave 3 hardening, audit-remediation tightening, the shared source-lane framework, Runtime Slices 1 through 9, the post-Slice-7 hardening and lane-admission-governance pass, bounded ODT lane delivery, bounded EPUB lane delivery, the post-Slice-8 governance execution, the post-Slice-9 governance selection, the bounded special-track Scrivener Stage 1 authority-recon runtime slice, the current Scrivener Stage 2 planning-control packet, the explicit Stage 2 implementation-remains-blocked decision, the PDF lane admission hardening and structured probe pass, and the mypy type-checking hygiene pass.

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

*Last updated: 2026-03-20*

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
- Slice 8 - bounded local ODT source lane
- Slice 9 - bounded local EPUB source lane
- Slice 10 - special-track Scrivener Stage 1 authority recon, status-only only

The currently admitted source lanes remain narrow:

- local `.md`
- local `.txt`
- local text-layer `.pdf`
- local `.docx`
- local `.rtf`
- local `.odt`
- local `.epub`

Scrivener is not admitted as a source lane.
Only the bounded Stage 1 authority-recon runtime slice is implemented.

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
Empty text-like sources are now denied rather than treated as empty success outputs.

Runtime Slice 5 adds one bounded local PDF lane only.
That lane admits text-layer `.pdf` files, remains text-only and non-OCR, allows `ready` only for trustworthy extractable text, allows `partial_success` only when some pages lack extractable text, denies encrypted or text-layer-free PDFs, and marks corrupt or unreadable PDFs unavailable.
The PDF lane has since been hardened for admission truth.
It now exposes a structured host admission probe (`probe_pdf_lane_admission()`) that reports per-tool presence of `pdfinfo` and `pdftotext` separately so downstream consumers can distinguish which tool is absent.
The lane eligibility path now uses probe-derived summaries rather than a static dependency message when tools are missing.
The extraction path now distinguishes malformed or corrupt PDFs (detected via `pdfinfo` or `pdftotext` stderr structural-error patterns) from generic tool invocation failures, giving each a specific operator-visible summary.
No OCR path or external service fallback exists or is implied at any failure state.

The extraction runtime now also uses a shared source-lane framework for lane admission, shared provenance metadata, shared failure posture, and admitted-lane reporting.
Runtime Slice 6 adds one bounded local DOCX lane only.
That lane admits local `.docx` packages, remains syntax-only, recovers headings only from explicit paragraph-style evidence, recovers simple lists and bounded table text only when deterministic, denies comments or tracked changes, and marks corrupt or unreadable packages unavailable.
Runtime Slice 7 adds one bounded local RTF lane only.
That lane admits local `.rtf` files, remains paragraph-only, supports basic escaped character recovery only as needed for honest plain-text extraction, denies annotation, review, field, media, and other rich destinations outside the lane, and marks corrupt or syntactically untrustworthy sources unavailable.
Runtime Slice 8 adds one bounded local ODT lane only.
That lane admits local `.odt` packages, remains syntax-only, recovers headings only from explicit `text:h` structure, recovers simple lists only from explicit `text:list` structure, recovers bounded table text only when row and cell order are deterministic, denies annotations, tracked changes, and embedded object/media structures, and marks corrupt or structurally untrustworthy packages unavailable.
Runtime Slice 9 adds one bounded local EPUB lane only.
That lane admits local `.epub` packages, remains syntax-only, establishes package truth through EPUB mimetype, container, manifest, and spine authority, recovers headings only from explicit XHTML heading structure, recovers simple lists and bounded table text only when deterministic, denies active content, navigation documents in the admitted reading path, and media-bearing content structures, and marks corrupt or structurally untrustworthy packages unavailable.
The text baseline is now documented explicitly alongside the richer lanes rather than remaining only an implicit runtime truth surface.

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
Ready ODT extraction results remain compatible with the same path through section-bounded chunking when explicit heading structure exists and paragraph-bounded fallback otherwise.
Ready EPUB extraction results remain compatible with the same path through section-bounded chunking when explicit heading structure exists and paragraph-bounded fallback otherwise.

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
Special-track runtime slices implemented without lane admission may appear in implemented-slice reporting only; the current example is Scrivener Stage 1 authority recon.
Future lane work is now expected to pass a reusable admission playbook before implementation begins.

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
- `docs/contracts/source-lane-text.md`
- `docs/contracts/source-lane-docx.md`
- `docs/contracts/source-lane-odt.md`
- `docs/contracts/source-lane-epub.md`
- `docs/contracts/source-lane-pdf.md`
- `docs/contracts/source-lane-rtf.md`
- `docs/contracts/retrieval-package.md`
- `docs/contracts/handoff-envelope.md`
- `docs/contracts/service-status.md`
- `docs/contracts/embedded-diagnostics.md`
- `docs/source-lanes/README.md`
- `docs/source-lanes/contract-symmetry-audit.md`
- `docs/source-lanes/lane-admission-playbook.md`

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
