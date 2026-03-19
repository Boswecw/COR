# Cortex — ODT Lane Implementation Plan

## Immediate objective

Implement **ODT** as the next bounded source lane in Cortex, following the completed lane-selection governance work.

This implementation must preserve Cortex as a **bounded local file-intelligence service** and admit ODT only through a narrow, explicit, fixture-first contract.

ODT is the selected next target.
That does **not** authorize broad office-suite support, semantic extraction, rendering behavior, or generalized compound-document abstractions.

---

## Required sequencing

Current baseline status:

1. the post-hardening / post-selection baseline is already committed and pushed at `955c4af`
2. this plan file may be normalized and committed with the ODT lane work
3. implement ODT from this plan only

Do not reopen baseline-stabilization work inside the ODT lane change set.

---

## Why ODT is next

ODT won the formal next-lane evaluation because it is the strongest remaining conventional authoring-document candidate under Cortex doctrine:

- clearer structural authority than HTML
- lower rendering/web drift pressure than HTML
- lower composite publication-package pressure than EPUB
- still a normal document-family lane unlike Scrivener

So ODT is the right next lane **only if admitted narrowly**.

---

## Execution target

Ship a **bounded ODT lane** that:

- admits only local `.odt`
- emits schema-valid syntax-only extraction results
- feeds ready outputs into the existing retrieval-package path without semantic special-casing
- reports truthful provenance, denial, unavailable, and any contract-defined degraded posture
- preserves lane-specific honesty limits instead of flattening ODT into a generic rich-document model

---

## Scope for this implementation

### In scope

1. ODT lane contract doc
2. ODT lane admission ADR
3. runtime implementation through the existing shared source-lane framework
4. fixture-first lane admission
5. tests for ready / denied / unavailable cases
6. retrieval-path verification for ready ODT outputs
7. service-status update only where needed to represent shipped runtime truth
8. README / system-doc refresh reflecting shipped reality only

### Out of scope

- HTML work
- EPUB work
- Scrivener work
- semantic labeling
- summarization
- ranking
- recommendations
- rendering/viewer behavior
- page-faithful layout reconstruction
- comments/review/editorial semantics
- embedded object/media interpretation
- workflow/orchestration behavior
- retry/queue/executor behavior
- generic office-suite abstraction
- schema expansion unless contract truth truly requires it

---

## Explicit guardrails

### Guardrail 1 — No generic office / rich-document abstraction

Do **not** create a generalized compound-document or rich-document model to make ODT implementation easier.

Allowed:

- use of the existing shared source-lane framework
- shared status / provenance / failure handling
- shared retrieval gating
- shared invariant testing

Not allowed:

- a new abstraction that pressures PDF / DOCX / RTF / ODT into the same structural truth surface
- convenience APIs that erase lane-specific honesty limits

### Guardrail 2 — No best-effort fallback behavior that hides uncertainty

Do **not** coerce unsupported or ambiguous ODT states into apparent success.

Examples to reject:

- silently dropping unsupported structures and still claiming ready when contract truth is broken
- masking corrupt package states with fallback text extraction
- converting uncertainty about document structure into optimistic heading/list/table claims

### Guardrail 3 — No schema expansion unless contract truth truly requires it

ODT should use the existing schemas unless there is a genuine truth gap.
Do not add fields because package internals are tempting to expose.

### Guardrail 4 — Preserve syntax-only posture

ODT admission must remain strictly syntax-only.
No content classification, summarization, editorial interpretation, semantic labeling, or authoring-intent inference.

---

## ODT authority model

### Structural authority

Treat the ODT package as a bounded OpenDocument text package where:

- the package/container defines the admitted artifact boundary
- the document content XML provides the primary structural extraction source
- admitted metadata is limited and explicit
- style semantics are not automatically treated as meaning unless the contract explicitly allows a narrow deterministic interpretation

### Important consequence

ODT is **not** admitted as “whatever LibreOffice can display.”
It is admitted only as the narrow syntax surface Cortex can recover honestly from the package structure.

---

## ODT admitted input boundary

Admit only:

- local `.odt`
- media type consistent with ODT expectations when supplied
- readable package/container
- readable primary document content required for the lane contract

Deny or fail closed when:

- suffix/media-type mismatch violates lane eligibility
- container/package shape is inconsistent with honest ODT admission
- required document content is missing or structurally unusable

---

## ODT admitted extraction surface

Admit only what can be recovered honestly and deterministically.

### Initial target surface

1. **paragraphs**
2. **headings when honestly recoverable from explicit structure or narrowly allowed style evidence**
3. **simple list structure when honestly recoverable**
4. **bounded plain table text only if deterministic and contract-safe**
5. **limited core metadata only if explicitly admitted and consistently recoverable**

### Important constraint

If a structure type cannot be recovered honestly enough for cross-lane consistency, keep the ODT lane narrower rather than inflating claims.

---

## Explicit exclusions

Do **not** admit:

- rendering/layout reconstruction
- page-faithful output claims
- comments/review/editorial semantics
- tracked changes or revision semantics
- embedded media/object interpretation
- macro/script semantics
- style semantics beyond narrowly contract-allowed structural recovery
- semantic labels or inferred intent
- packaging artifacts as user-facing extraction truth unless explicitly contract-admitted

---

## Expected posture behavior

### `ready`
Use only when the ODT package is valid for the lane, the admitted extraction surface is recoverable honestly, and output is schema-valid.

### `denied`
Use when:

- the source is outside the ODT lane boundary
- suffix/media-type mismatch makes admission ineligible
- the package contains contract-disallowed review/comment/revision semantics if the lane contract defines them as denial conditions
- the document shape requests unsupported semantics that the lane explicitly refuses

### `unavailable`
Use when:

- the container is corrupt or unreadable
- required package members are missing
- parsing/tooling needed for the bounded lane cannot be completed honestly
- syntax is too damaged to trust extraction

### `partial_success`
Allow only if the contract explicitly proves it is honest.

Default posture: **do not use `partial_success` unless clearly necessary and explicitly justified**.
If the lane can fail closed instead, prefer fail closed.

---

## Provenance requirements

ODT extraction results must report truthful provenance through the shared model, including only what the current contract already supports.

The provenance posture should make clear that:

- this is ODT lane extraction
- extraction was bounded and syntax-only
- structure claims are only those the lane contract admits
- no semantic or rendering interpretation occurred

Do not expand provenance fields unless contract truth genuinely requires it.

---

## Tooling posture

Prefer bounded local parsing that preserves honest package-level inspection and deterministic extraction.

Selection criteria for any parsing dependency:

- local-only
- stable and auditable
- does not force Cortex into render-driven interpretation
- supports fail-closed handling for corrupt/unreadable packages
- does not introduce hidden semantic transforms

Do **not** rely on a heavyweight office-rendering workflow as the lane authority.

---

## Fixture-first admission requirements

ODT must not be admitted without fixtures that prove the lane contract.

### Required ready fixtures

At minimum:

- simple paragraph-only ODT
- heading-bearing ODT
- simple list ODT if lists are admitted
- simple table ODT if tables are admitted

### Required denied fixtures

At minimum:

- suffix/media-type mismatch case
- contract-disallowed review/comment/revision case if applicable
- unsupported structure case if the contract defines a denial boundary there

### Required unavailable fixtures

At minimum:

- corrupt package/container
- missing required content member
- unreadable/untrustworthy syntax case

### Cross-lane invariant fixtures/tests

Strengthen tests proving ODT now conforms to the same governed system behavior as:

- `.md`
- `.txt`
- text-layer `.pdf`
- `.docx`
- `.rtf`

---

## Runtime implementation tasks

### 1. Contract docs

Create:

- `docs/contracts/source-lane-odt.md`
- ADR for ODT lane admission

### 2. Runtime lane implementation

Implement ODT through the existing shared source-lane framework.

Possible structure:

- extend lane registry / admission map
- add ODT lane helper/module if needed
- add extraction path in `extraction_emission.py` only through the shared lane flow

### 3. Retrieval integration

Verify ready ODT extraction feeds the existing retrieval-package path with no semantic special-casing.

### 4. Service-status truth

Update service status only as needed to truthfully reflect the admitted ODT lane and any newly delivered runtime slice if the repo’s naming/versioning requires it.

### 5. Docs refresh

Update only shipped-reality docs:

- `README.md`
- relevant `doc/system/` sources
- rebuilt `doc/cxSYSTEM.md`

---

## Cross-lane invariants to preserve

ODT admission must not break the governed lane system.

Add or preserve tests proving that ODT:

- emits the same extraction-result schema shape
- never emits semantic fields
- respects privacy-bounded diagnostics
- uses uniform denial/unavailable semantics
- uses `partial_success` only if explicitly contract-defined
- gates retrieval packaging consistently
- reports admitted-lane truth consistently via service status / lane registry
- fails closed on unsupported suffix/media-type mismatch

---

## Suggested implementation sequence

### Step 1
Treat `955c4af` as the clean post-hardening / post-selection baseline.

### Step 2
Write `source-lane-odt.md` and the ODT admission ADR before code changes.

### Step 3
Create ODT fixtures for ready / denied / unavailable cases.

### Step 4
Implement bounded ODT extraction through the shared lane framework.

### Step 5
Add runtime tests and cross-lane invariant tests.

### Step 6
Verify extraction CLI and retrieval CLI/operator surfaces behave consistently for ODT, including media-type handling.

### Step 7
Update docs and rebuild `cxSYSTEM.md`.

### Step 8
Validate with:

- `make validate`
- `make test-runtime`
- `bash doc/system/BUILD.sh`
- direct ODT extraction CLI verification against a ready fixture

---

## Acceptance criteria

This implementation is complete only if:

1. ODT is admitted through the existing governed lane framework
2. ODT extraction remains syntax-only
3. no generic office/rich-document abstraction was introduced
4. no best-effort fallback behavior hides uncertainty
5. no schema expansion occurred unless contract truth required it
6. fixture-first admission proves ready / denied / unavailable behavior
7. retrieval integration remains lane-neutral
8. service-status truth reflects shipped reality only
9. docs reflect shipped repo truth only

---

## Anti-drift reminders

Reject any change that tries to turn ODT admission into:

- office-suite support momentum
- rendering/viewer behavior
- semantic extraction
- editorial/review semantics
- generalized compound-document abstractions
- parser convenience over contract truth
- Scrivener-by-proxy package work

This step is **bounded ODT lane admission only**.

---

## Prompt for VS Code Codex

Use the current Cortex repo as the baseline: slices 1–7 delivered, post-Slice-7 hardening complete, next-lane governance evaluation complete, ODT selected as the next target, and no ODT implementation yet.

Your task is to implement **ODT as a bounded source lane** in Cortex.

Requirements:

- preserve Cortex constitutional posture
- syntax-only extraction only
- no semantic enrichment
- no workflow/orchestration behavior
- no generic office-suite or rich-document abstraction
- no best-effort fallback behavior that hides uncertainty
- no schema expansion unless contract truth truly requires it
- fixture-first lane admission

Implement:

1. `docs/contracts/source-lane-odt.md`
2. ODT lane admission ADR
3. ready / denied / unavailable ODT fixtures
4. bounded ODT extraction through the shared lane framework
5. runtime and cross-lane invariant tests
6. retrieval integration verification
7. service-status update only where needed for shipped truth
8. README / system-doc refresh and rebuilt `cxSYSTEM.md`

Target admitted ODT surface:

- paragraphs
- headings when honestly recoverable
- simple lists when honestly recoverable
- bounded plain table text only if deterministic
- limited core metadata only if explicitly admitted

Exclude:

- rendering/layout reconstruction
- comments/review/revision semantics
- embedded object/media interpretation
- macro/script semantics
- style meaning beyond narrowly allowed structural recovery
- semantic labels or summaries

Use `partial_success` only if explicitly justified by the lane contract; otherwise fail closed.

The goal is to leave Cortex with ODT admitted cleanly as the next governed lane, without changing the system’s bounded role.
