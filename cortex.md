# Cortex

## Purpose

Cortex is a **bounded local file-intelligence service** inside the Forge ecosystem.

It is a **standalone constitutional workspace/repo**, not a product, not a semantic engine, not an orchestrator, not a workflow controller, and not a generic ETL platform.

Its role is to transform eligible local source material into **bounded contract truth** through tightly governed runtime lanes.

The governing authority remains:

**Cortex — Constitutional Project Plan v2.1**

When there is tension between convenience, build speed, future ambition, parser availability, or implementation preference:

**v2.1 wins.**

---

## Core constitutional posture

Keep these boundaries hard:

- service-only visibility
- syntax before semantics
- retrieval infrastructure, not retrieval authority
- fail closed over convenience
- privacy-preserving operational truth
- explicit invalidation over assumed freshness
- default-denied observation
- no standalone product identity
- no canonical business truth ownership
- no workflow sequencing ownership
- no general executor / agent-host drift
- no generic ETL drift

Cortex may validate and emit bounded contract truth.

Cortex may not become:

- a semantic interpreter
- a retrieval judge
- a dispatcher
- a queue manager
- a coordinator
- an executor selector
- an agent host
- a general ingestion platform

---

## What Cortex is for

Cortex exists to provide **governed local source intake, syntax-only extraction, and bounded retrieval-package preparation** for supported source families.

It should make local source material structurally legible to the rest of the Forge ecosystem **without claiming meaning, importance, or action authority**.

That means Cortex can own:

- intake validation
- source eligibility checks
- syntax-only extraction
- deterministic structural chunking
- provenance capture
- invalidation posture
- bounded contract emission

That does **not** mean Cortex owns:

- summarization
- tagging
- classification
- entity meaning
- topic inference
- sentiment
- ranking
- relevance scoring
- recommendation logic
- workflow routing
- retry/dispatch logic
- executor choice

---

## Runtime slices

Current runtime progression:

1. **Runtime Slice 1 — intake validation**
2. **Runtime Slice 2 — syntax-only extraction-result emission**
3. **Runtime Slice 3 — one governed retrieval-package emission path**
4. **Runtime Slice 4 — service-status truth path**

This order matters.
Slices 1 through 4 are now implemented in bounded form.

Do not broaden source ecosystems before the narrow runtime path is stable across these slices.

---

## Source support posture

Cortex should support multiple source types over time, but only through **explicit governed source lanes**.

The rule is:

**Cortex does not “support documents” in the abstract. Cortex supports specific source families through bounded, testable, fail-closed lanes.**

Each source lane must be:

- explicitly allowed
- contract-bounded
- independently testable
- degradation-aware
- fail-closed
- unable to inject semantic interpretation

---

## Recommended source-lane roadmap

### Tier 1 — strong support candidates

These fit Cortex’s syntax-first posture best.

- `.md`
- `.txt`
- `.docx`

Why:

- locally common
- structurally recoverable
- relatively deterministic
- low ambiguity compared with layout-first formats

### Tier 2 — bounded / degraded support

- text-based `.pdf`

Why:

- useful and common
- but structurally messier
- text order may be unstable
- headings are often inferred rather than explicit
- multi-column/page furniture can distort extraction

PDF should enter as a **degraded lane**, not as a parity format with Markdown or DOCX.

Initial PDF posture should be:

- text-based PDFs only
- no OCR in first lane
- explicit degraded completeness posture where appropriate
- fail closed on unreadable or unsupported PDFs

### Tier 3 — specialized governed lane

- Scrivener projects

Scrivener is important, especially for author-facing ecosystem value, but it is not “just another file type.”

A Scrivener project is a **structured project container** with internal hierarchy and multiple artifacts.

Cortex should therefore support Scrivener only as a **special governed source family** with a narrow lane, for example:

- project structure discovery
- draft/research/document node identification
- literal text extraction from supported internal documents
- provenance-preserving structure emission

Cortex should **not** own:

- project interpretation
- writing workflow logic
- manuscript meaning
- compile/export behavior
- editorial guidance

### Tier 4 — later only if justified

- `.rtf`
- `.odt`
- `.html`
- `.epub`

These should enter only if there is a clear operational reason and a bounded contract path.

---

## Source-lane rules

Every new source family must enter through a dedicated lane with explicit rules.

For each new lane, define:

1. **Eligibility**
   - exact supported extensions/media types
   - bounded local source assumptions
   - explicit deny conditions

2. **Extraction posture**
   - what literal structures may be emitted
   - what completeness/degradation flags are allowed
   - what invalidation posture is required

3. **Failure posture**
   - malformed
   - unreadable
   - unsupported
   - dependency unavailable
   - schema-invalid intermediate/final result

4. **Non-goals**
   - specific semantic or workflow behaviors the lane must never pick up

5. **Tests and fixtures**
   - minimal valid fixtures
   - minimal invalid fixtures
   - schema-validation coverage
   - runtime fail-closed coverage

If a source family cannot be supported cleanly under these rules, it should not be admitted yet.

---

## Format-specific guardrails

### Markdown / plain text

Allowed posture:

- literal headings
- paragraphs
- simple ordered structure
- deterministic segmentation

Avoid:

- inferred topic structure
- semantic rewriting

### DOCX

Allowed posture:

- document properties if already available through parser output
- headings
- paragraphs
- lists
- simple tables later if explicitly added

Avoid:

- semantic use of styling beyond bounded structural mapping
- broad style interpretation
- hidden editorial meaning

### PDF

Allowed posture:

- text extraction from supported text-based PDFs
- bounded page-aware or block-aware segmentation if deterministic
- explicit degraded completeness when structure is uncertain

Avoid:

- OCR in the initial lane
- semantic repair
- layout guessing that pretends to be reliable structure

### Scrivener

Allowed posture:

- binder/project structure visibility only where explicitly supported
- literal extraction from supported internal text nodes
- provenance linking to project/container paths

Avoid:

- compile behavior
- workflow orchestration
- editorial interpretation
- project-state inference beyond bounded structural truth

---

## Drift warnings

The biggest risk to Cortex is not failure.

It is **capability drift through parser expansion**.

Warning signs include:

- “just add another format” behavior without lane design
- format support bundled into one broad abstraction
- parser fallback chains that widen support silently
- inferred semantics justified as “helpful extraction”
- chunk ranking or relevance hints appearing inside retrieval packaging
- project/container formats treated as flat files
- source support expanding faster than contracts, fixtures, and tests

If any of those appear, stop and narrow the design.

---

## Practical implementation rule

When adding support for a new source family, use this decision rule:

**Is this a new governed source lane, or is this broadening Cortex into a general ingestion surface?**

If it is the second, do not implement it.

---

## Current recommendation

Near-term source expansion should proceed in this order:

1. `.docx`
2. text-based `.pdf`
3. Scrivener project lane

Do **not** add all formats in one pass.

Do **not** collapse them behind a generic “document parser” abstraction before their constitutional posture is separately defined.

---

## Working principle

Cortex becomes valuable not by reading everything.

Cortex becomes valuable by reading the **right local sources** through **governed, deterministic, fail-closed lanes** that preserve structural truth without claiming semantic authority.
