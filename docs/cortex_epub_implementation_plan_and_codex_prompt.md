# Cortex — EPUB Implementation Plan + Codex Prompt

## Status

Planning artifact only  
Governance-aligned follow-on to post-Slice-8 selection  
No implementation is assumed complete by this document

---

# Part 1 — Bounded EPUB implementation plan

## 1. Purpose

This plan defines the next bounded Cortex implementation target after Slice 8.

Selected target:
- **EPUB**

This plan exists to admit EPUB as a **governed composite document package lane** using the existing shared source-lane framework, while preserving Cortex’s constitutional boundaries.

This is not a generic archive/parser effort.
This is not ebook-reader behavior.
This is not rendering reconstruction.
This is not semantic interpretation.

---

## 2. Locked constitutional posture

EPUB must be implemented only under the already-established Cortex doctrine:

- syntax before semantics
- fail closed over convenience
- retrieval infrastructure, not retrieval authority
- privacy-preserving diagnostics
- explicit invalidation over assumed freshness
- no hidden control-plane or workflow behavior
- no generic rich-document or package abstraction beyond what the lane contract truly requires

EPUB must fit Cortex.
Cortex must not bend around EPUB.

---

## 3. Lane identity

**Lane name:** `epub`  
**Admitted family:** local `.epub`  
**Declared media type:** `application/epub+zip`

EPUB is treated as a **bounded composite package lane**.

The implementation must read only enough package truth to support honest syntax-only extraction.

---

## 4. Implementation objective

Deliver a first bounded EPUB lane that can:

- admit valid local `.epub` inputs
- validate declared media type when supplied
- establish package/container truth honestly
- identify admitted textual package members through bounded package authority
- emit syntax-only extraction results through the existing extraction path
- feed the existing retrieval packaging path unchanged when extraction is `ready`
- report truthful service-status state only if contract truth requires new status surface

---

## 5. Required authority model

The implementation must define and honor a narrow package authority chain.

At minimum, EPUB v1 should establish:

1. package/container discovery truth
2. package document truth
3. manifest membership truth
4. spine or equivalent reading-order truth for admitted textual documents
5. per-content-document textual extraction truth

The lane must never derive authority from presentation intent, styling, or reader behavior.

---

## 6. Proposed ready surface for EPUB v1

Ready extraction may include only bounded syntax-level structure that is honestly recoverable from admitted textual package members.

Admit for v1:

- paragraphs
- explicit headings when honestly recoverable
- explicit list items when honestly recoverable
- bounded deterministic plain table text only if clearly recoverable without layout interpretation
- limited core metadata only if explicitly admitted and directly recoverable from package truth

Do not admit for v1:

- chapter meaning
- narrative role
- editorial significance
- style semantics beyond narrow structural recovery
- rendering/layout reconstruction
- image/media interpretation
- annotations/comments/review semantics
- scripting/active content semantics

---

## 7. Contract posture

EPUB v1 should preserve the existing Cortex contract discipline:

- `ready` when bounded extraction is honest and complete enough for admitted surface
- `denied` when the item is recognizable but outside lane policy
- `unavailable` when package/parsing truth is too broken to trust
- no `partial_success` unless a truly honest degraded posture is explicitly proven and added later

Default assumption:
- **do not introduce `partial_success` in EPUB v1**

---

## 8. Expected package boundary for v1

The lane should support only the bounded normal EPUB package path needed for truthful v1 extraction.

Expected minimum package surfaces:

- zip container readability
- container/package file discovery
- package manifest parsing
- package spine parsing
- admitted textual content document parsing

Everything beyond that should be treated as excluded unless explicitly required by contract.

---

## 9. Explicit exclusions

The implementation must fail closed against drift into:

- browser or ebook-reader behavior
- CSS/layout reconstruction
- cover/media interpretation
- embedded audio/video/object handling
- JavaScript or active content meaning
- annotation/comment/review features
- accessibility semantics beyond plainly admitted syntax recovery
- generic zip/package framework ambitions
- semantic chapter or section labeling

---

## 10. Slice contents

The EPUB slice should be limited to these implementation surfaces.

### A. Contract and governance artifacts

Create/update:

- `docs/contracts/source-lane-epub.md`
- decision record if needed only to finalize admission truth from the already-selected governance posture
- system documentation sections affected by admitted EPUB truth

### B. Runtime lane implementation

Add lane implementation files through the existing framework, likely including:

- `cortex_runtime/epub_lane.py`
- registration/update in `cortex_runtime/source_lanes.py`
- narrow extraction-path integration in `cortex_runtime/extraction_emission.py` only where lane wiring requires it

### C. Retrieval compatibility

Do **not** redesign retrieval packaging.

Only ensure that a `ready` EPUB extraction result can feed the existing retrieval path unchanged through:

- `cortex_runtime/retrieval_package_emission.py`

### D. Service-status truth

Update service-status schema or reporting only if EPUB introduces a genuinely necessary new truthful status surface.

No ornamental schema expansion.

### E. Fixtures and tests

Add fixture-first coverage for:

- normal valid EPUB
- structural valid-but-denied EPUB
- corrupt/unreadable package
- missing authority file(s)
- malformed package/content XML
- ugly cases around manifest/spine disagreement or unsupported structures

Add/extend:

- lane tests for `ready` / `denied` / `unavailable`
- cross-lane invariant tests
- service-status truth tests if status surface changes
- CLI smoke coverage if already present for peer lanes

---

## 11. Proposed implementation sequence

### Step 1 — Lock the lane contract

Finalize `docs/contracts/source-lane-epub.md` from the draft.

Must explicitly define:
- authority model
- admitted input boundary
- ready surface
- exclusions
- denial conditions
- unavailable conditions
- provenance posture
- `partial_success` posture

### Step 2 — Build fixture set first

Before runtime work, assemble a bounded fixture set.

Minimum fixture categories:
- valid simple EPUB
- valid EPUB with clear headings/lists
- valid EPUB with deterministic plain table text if admitted
- EPUB with unsupported active/media structures for denial testing
- corrupt EPUB archive
- EPUB missing required authority/package files
- malformed XML in package/content documents
- manifest/spine inconsistency ugly case

### Step 3 — Implement bounded package authority walk

Implement the smallest possible authority walk needed to establish honest extraction:
- open zip
- discover package/container truth
- locate package file
- parse manifest/spine
- identify admitted textual members in bounded reading order

No convenience fallbacks that hide uncertainty.

### Step 4 — Implement content extraction

For admitted textual members only:
- parse bounded text structure
- recover paragraphs
- recover explicit headings when honest
- recover explicit lists when honest
- recover deterministic plain table text only if contract allows

Reject or exclude anything outside the lane contract.

### Step 5 — Wire extraction emission

Integrate lane output into the shared extraction emission path with no cross-lane drift.

### Step 6 — Verify retrieval compatibility

Confirm `ready` EPUB results feed existing retrieval packaging unchanged.

### Step 7 — Update truth surfaces only if needed

Update service-status and docs only where required to report shipped truth honestly.

### Step 8 — Rebuild and validate

Run repo validation, runtime tests, and system-doc rebuild.

---

## 12. Acceptance criteria

EPUB implementation is complete only if all are true:

- EPUB is admitted through the shared source-lane framework
- contract artifacts explicitly define v1 boundaries
- ready extraction is syntax-only and bounded
- denied and unavailable are distinct and test-backed
- no `partial_success` unless explicitly justified and documented
- retrieval path works unchanged for `ready` EPUB results
- invariant coverage shows no cross-lane drift
- service-status truth remains narrow and honest
- docs/system build succeeds
- worktree is left clean after commit

---

## 13. Non-goals

This slice must not:

- implement HTML
- open special-track Scrivener
- create a shared office/package mega-abstraction
- generalize Cortex into archive intelligence
- infer semantic structure from styling
- introduce workflow/editorial semantics
- add browser-like recovery behavior

---

## 14. Deliverable summary

Expected outputs from the slice:

- final EPUB lane contract doc
- bounded EPUB runtime lane
- fixture set
- lane + invariant tests
- truthful status/doc updates only if needed
- rebuilt `cxSYSTEM.md`
- clean commit boundary

---

# Part 2 — VS Code Codex build prompt

## Prompt

You are implementing the **next bounded Cortex runtime slice** in the existing repository.

Repository/project context:
- Project: **Cortex**
- Cortex is a **bounded local file-intelligence service**
- It owns only intake of eligible local content, syntax-only extraction, retrieval-preparation support, handoff packaging support, freshness/invalidation signaling, truthful operational status, and privacy-preserving diagnostics
- It does **not** own semantic authority, retrieval authority, workflow/orchestration behavior, canonical truth storage, generic ETL behavior, or document-platform ambitions

Current repo baseline:
- clean post-Slice-8 baseline
- admitted lanes already exist for `.md`, `.txt`, text-layer `.pdf`, `.docx`, `.rtf`, and `.odt`
- shared source-lane framework already exists
- ODT is implemented and committed
- next governance-selected planning target is **EPUB**

Your task:
Implement a **bounded EPUB lane** only.

You must follow these non-negotiables:

1. **Syntax before semantics**
   - recover only bounded syntax-level structure
   - do not infer meaning, chapter semantics, narrative role, or editorial intent

2. **Fail closed over convenience**
   - no best-effort ambiguity masking
   - no hidden fallbacks that pretend extraction succeeded when package truth is unclear

3. **Retrieval infrastructure, not retrieval authority**
   - EPUB may feed existing retrieval packaging only when extraction is honestly `ready`
   - do not redesign retrieval behavior

4. **No drift**
   - do not introduce browser behavior
   - do not introduce ebook-reader behavior
   - do not introduce generic office/package/archive abstractions unless truly unavoidable and tightly bounded
   - do not add workflow/orchestration/editorial semantics

5. **Truthful status**
   - update service-status only if new truthful surface is genuinely required
   - no ornamental schema widening

Implementation target:
- Admit local `.epub`
- Support declared media type `application/epub+zip`
- Treat EPUB as a **bounded composite package lane**
- Establish package/container truth honestly
- Parse only enough package authority to identify admitted textual content documents
- Extract only bounded syntax-level structure from admitted textual members

Admitted v1 ready surface:
- paragraphs
- explicit headings when honestly recoverable
- explicit list items when honestly recoverable
- bounded deterministic plain table text only if clearly recoverable without layout interpretation
- limited core metadata only if explicitly admitted by the final lane contract

Excluded v1 surface:
- rendering/layout reconstruction
- CSS/style semantics beyond narrow structural recovery
- image/media interpretation
- scripting/active content semantics
- annotations/comments/review semantics
- semantic labels/summaries
- browser or ebook-reader behavior

Contract posture:
- `ready` when bounded extraction is honest
- `denied` when recognizable candidate EPUB is outside lane policy
- `unavailable` when package/parsing truth cannot be trusted
- do **not** introduce `partial_success` in v1 unless you can prove an honest degraded posture and encode it explicitly

Expected authority chain:
- zip container readability
- container/package discovery
- package manifest parsing
- spine/reading-order authority
- admitted textual member parsing

Work in this order:

### 1. Finalize governance/contract artifact
Create or finalize:
- `docs/contracts/source-lane-epub.md`

It must explicitly define:
- authority model
- admitted input boundary
- ready surface
- exclusions
- denial conditions
- unavailable conditions
- provenance posture
- `partial_success` posture

### 2. Build fixtures first
Add a bounded fixture set for at least:
- simple valid EPUB
- valid EPUB with headings/lists
- valid EPUB with deterministic plain table text if admitted
- structurally valid-but-denied EPUB if applicable
- corrupt archive
- missing required package authority files
- malformed XML
- manifest/spine inconsistency ugly case

### 3. Implement the lane
Likely files to add/update:
- `cortex_runtime/epub_lane.py`
- `cortex_runtime/source_lanes.py`
- `cortex_runtime/extraction_emission.py`
- any narrowly required shared helpers only if unavoidable

Use the existing ODT / DOCX / RTF lane posture as structural reference points, but do not broaden abstractions casually.

### 4. Preserve retrieval compatibility
Ensure `ready` EPUB extraction results feed the existing retrieval path unchanged through:
- `cortex_runtime/retrieval_package_emission.py`

### 5. Add tests
Add/extend tests for:
- `ready`
- `denied`
- `unavailable`
- cross-lane invariants
- service-status truth only if status changes
- direct extraction/retrieval smoke if that pattern already exists

### 6. Rebuild and validate
Run the repo’s normal validation/test/doc rebuild flow.
Target outcomes should include:
- validation passing
- runtime tests passing
- system docs rebuilt successfully

Deliverable format required from you:

1. **Executive summary**
   - what was implemented
   - what was intentionally not implemented

2. **Files changed**
   - grouped by purpose

3. **Contract summary**
   - admitted boundary
   - ready surface
   - denied posture
   - unavailable posture
   - `partial_success` posture

4. **Validation results**
   - exact commands run
   - concise outcomes

5. **Drift check**
   - explicitly state whether any semantic, workflow, retrieval-authority, browser, or generic package drift was introduced

Important constraints:
- do not implement HTML
- do not implement Scrivener
- do not create a generic rich-document or archive platform
- do not change schemas unless truthful contract/status needs require it
- prefer the smallest bounded implementation that honestly admits EPUB into the existing Cortex lane framework

Success condition:
A clean, governed EPUB lane lands without changing what Cortex fundamentally is.

