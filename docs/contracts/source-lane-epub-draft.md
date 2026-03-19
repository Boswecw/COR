# Cortex Source Lane Draft - EPUB

Governance note only. This draft defines planning posture and does not authorize implementation by itself.

## Status

Draft for governance review only.

## Purpose

This document defines the proposed admission boundary for a bounded Cortex EPUB lane.

It does not authorize implementation by itself.

## 1. Lane identity

Lane name: `epub`

Admitted family:

- local `.epub`

Declared media type:

- `application/epub+zip`

EPUB is treated as a bounded composite document package lane, not as a rendering surface.

## 2. Authority model

The EPUB lane must define explicit package authority before admission.

At minimum, implementation must identify and honor bounded package truth for:

- package and container discovery
- package manifest membership
- spine or equivalent declared reading-order authority
- textual content documents actually admitted for extraction

Cortex must not invent authority from presentation behavior or browser-style interpretation.

## 3. Proposed admitted input boundary

Admitted inputs:

- local `.epub` files
- declared media type `application/epub+zip`
- readable package containers whose package truth can be validated honestly

Not admitted:

- unpacked loose EPUB directories presented as arbitrary folders
- EPUB-adjacent web bundles
- non-EPUB zip containers
- package variants whose structure cannot be verified truthfully

## 4. Proposed ready extraction surface

Ready extraction may include only content that can be recovered deterministically from admitted package truth.

Candidate ready surface:

- bounded textual content from admitted spine or content documents
- paragraphs
- explicit headings when honestly recoverable
- explicit lists when honestly recoverable
- limited deterministic table text only if package or source truth supports it clearly
- limited core metadata only if explicitly admitted by contract

All extraction must remain syntax-only.

Cortex must not infer semantic section meaning, chapter intent, narrative role, or editorial significance.

## 5. Explicit exclusions

The EPUB lane must explicitly exclude any behavior outside bounded syntax-only extraction.

Excluded by default:

- rendering or layout reconstruction
- CSS or style meaning beyond narrowly allowed structural recovery
- script or active-content semantics
- audio, video, or media interpretation
- annotation, comment, or review semantics
- cover-image or artwork interpretation
- navigation semantics beyond narrowly defined package authority needs
- semantic labels, summaries, or editorial inference

## 6. Denial conditions

Inputs should be denied when they are syntactically recognizable as candidate EPUB inputs but contain out-of-lane or disallowed structures that make honest ready extraction inappropriate under the lane contract.

Candidate denial cases to refine during planning:

- active-content or script constructs in the admitted textual path if the contract excludes them absolutely
- package structures that require unsupported semantics to proceed honestly
- package features whose presence places the item outside the admitted v1 extraction boundary

Denial conditions must be explicit and test-backed.

## 7. Unavailable conditions

Inputs should be unavailable when truthful extraction cannot proceed because package or parsing truth cannot be trusted.

Candidate unavailable cases:

- corrupt or unreadable zip or package container
- missing required package authority files
- malformed package or content XML beyond bounded recovery policy
- manifest or spine truth too broken to establish the admitted reading path honestly

Unavailable conditions must be distinguished cleanly from denied conditions.

## 8. `partial_success` posture

`partial_success` is not admitted by default for EPUB v1.

It may be introduced only if contract truth later demonstrates a stable honest degraded posture that does not hide uncertainty.

Absent that proof, EPUB v1 should follow ready, denied, and unavailable discipline only.

## 9. Provenance model

EPUB extraction provenance must remain explicit and bounded.

At minimum, provenance should truthfully identify:

- source lane = `epub`
- local package source
- admitted package or content authority path used for extraction
- any bounded structural subdocument origin necessary to explain emitted sections

Cortex must not emit provenance claims it cannot justify from package truth.

## 10. Admission requirements

Before EPUB implementation begins, the lane must have:

- a fixture-first candidate set
- explicit ugly-case fixtures
- contract tests for ready, denied, and unavailable separation
- invariant coverage showing no cross-lane drift
- service-status truth only if new status signaling is genuinely required

## 11. Anti-drift reminders

EPUB admission must not be allowed to pull Cortex toward:

- browser behavior
- ebook-reader behavior
- document-platform ambitions
- semantic chapter interpretation
- media or archive generalization
- generic package parsing as an end in itself

The lane exists only to admit bounded local EPUB content into Cortex's syntax-only extraction and retrieval-preparation surface.

## 12. Governance recommendation

Recommendation: approve EPUB as the next planning target and use this draft as the starting boundary note for the implementation plan.
