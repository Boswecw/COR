# Cortex — Scrivener Phase 0 Recon Plan

## Document control

- Project: Cortex
- Status: Draft for execution
- Scope: Governance + evidence only
- Runtime impact: None allowed in this phase
- Source-lane posture: Special-track project-source candidate

## Purpose

This phase exists to prove whether Scrivener can be admitted into Cortex as a bounded, constitutional source lane without violating Cortex doctrine.

This is not an implementation phase.

This phase must not add:

- runtime parser code
- extraction pipeline code
- schema expansion
- handoff surface changes
- implicit lane admission
- hidden fallback behavior

The output of this phase is decision-grade evidence, not working ingestion.

---

## Why this phase exists

Scrivener is not a routine document peer like PDF, DOCX, ODT, or EPUB.

It is a project-source format with:

- package/folder structure
- internal hierarchy authority
- binder ordering semantics
- manuscript vs research/workspace ambiguity
- RTF-heavy content storage
- higher risk of authority drift if admitted casually

Because of that, Scrivener must be treated as a special-track source lane and must earn admission through structural proof before runtime work is authorized.

---

## Phase objective

Produce enough bounded evidence to answer these questions truthfully:

1. What file or files are structurally authoritative inside a Scrivener project?
2. How is binder hierarchy represented?
3. How are binder nodes mapped to content files?
4. How can manuscript content be distinguished from research, notes, templates, trash, or metadata-only nodes?
5. What version and compatibility risks exist across realistic Scrivener projects?
6. Can Cortex admit Scrivener without becoming a project manager, editorial engine, or semantic authority?

If the answer to any of these remains materially uncertain, implementation must not begin.

---

## Constitutional constraints

All work in this phase must obey the following:

### 1. Syntax before semantics
This phase may study structure, identifiers, hierarchy, file locations, and bounded content storage patterns.

It must not infer meaning, summarize content, rank importance, or interpret writing intent.

### 2. Fail closed
Unknown project structures, ambiguous mappings, unsupported versions, or unresolved node classifications must be treated as denied or unsupported, not guessed through.

### 3. No hidden authority shift
Scrivener support must not transfer manuscript truth, workflow truth, editorial truth, or semantic truth into Cortex.

### 4. Deterministic admission only
Node inclusion rules must be explicit, testable, and policy-based.

### 5. Privacy-preserving diagnostics
Recon artifacts may describe structure and failure classes, but must not expose user manuscript content beyond the minimum needed for fixture validation.

---

## Non-goals

This phase does not:

- implement Scrivener parsing
- export Scrivener to markdown/docx/pdf
- preserve editor state for round-tripping
- reconstruct compile behavior
- replicate Scrivener project management behavior
- ingest arbitrary research material under a broad convenience rule
- create a generic package-ingestion abstraction
- define semantic chunking or retrieval meaning

---

## Required deliverables

## 1. Fixture reconnaissance packet
Create a bounded fixture packet of sanitized Scrivener projects representing realistic structure diversity.

Minimum target fixture set:

- simple manuscript-only project
- manuscript + research project
- manuscript + notes/synopsis-heavy project
- project with nested folders and reordered binder items
- project with missing or irregular content files if obtainable safely
- project created from a newer Scrivener version if available

Expected artifact:

- `docs/source-lanes/scrivener/fixture-recon.md`

This document should record:

- fixture identifier
- project version if observable
- top-level package structure
- authoritative internal files observed
- binder/content mapping observations
- notable ambiguities
- unsupported or unstable patterns discovered

## 2. Structural authority note
Prove and document which artifact is authoritative for:

- project hierarchy
- item identity
- ordering
- manuscript inclusion posture if represented
- links from binder items to stored content

Expected artifact:

- `docs/source-lanes/scrivener/structural-authority.md`

This must explicitly answer whether `*.scrivx` is the sole structural authority or whether any additional artifact is required for bounded truth.

## 3. Node admission policy draft
Define draft policy for what a future Scrivener lane may admit.

Minimum policy categories:

- admitted by default
- admitted only under explicit bounded rule
- denied by default
- unsupported pending proof

Candidate categories to classify:

- manuscript documents
- manuscript folders
- research documents
- notes
- synopsis-only nodes
- templates
- trash
- snapshots/backups if present
- metadata-only entries

Expected artifact:

- `docs/source-lanes/scrivener/admission-policy.md`

## 4. Content mapping proof
Document how binder nodes map to physical content storage.

This should identify:

- path conventions
- identifier relationships
- missing-content behavior
- duplicate-reference risk
- whether content storage is always RTF or only commonly RTF
- conditions where a node exists structurally but has no extractable body

Expected artifact:

- `docs/source-lanes/scrivener/content-mapping.md`

## 5. Compatibility and risk note
Capture compatibility realities before implementation.

Minimum categories:

- version drift
- platform differences
- package irregularities
- unsupported project states
- partial project copies
- external file references if any appear
- corrupted or incomplete project structures

Expected artifact:

- `docs/source-lanes/scrivener/compatibility.md`

## 6. Implementation gate memo
Summarize whether implementation may begin.

Expected artifact:

- `docs/source-lanes/scrivener/implementation-gate.md`

This memo must conclude one of:

- authorized to draft admission ADR
- blocked pending more fixtures
- blocked pending unresolved authority ambiguity
- blocked pending bounded RTF dependency decision

---

## Evidence standards

The phase is complete only if the evidence is strong enough that another engineer could read the packet and understand:

- what is authoritative
- what is admissible
- what is denied
- what remains uncertain
- what would count as a contract violation in implementation

Each artifact should prefer tables, enumerated observations, and explicit denied cases over prose-heavy narrative.

Every ambiguity should be surfaced directly.

No artifact may smooth over uncertainty using phrases like “probably,” “should be fine,” or “best effort.”

---

## Working method

## Step 1 — Gather and sanitize fixtures
Create or collect a small bounded set of representative `.scriv` projects.

Rules:

- sanitize names and content where needed
- do not rely on a single toy project
- preserve enough internal structure to observe real binder/content behavior
- store fixtures under a dedicated fixture path with a README describing provenance and sanitization level

## Step 2 — Inspect package structure only
Perform read-only inspection of the package tree.

Record:

- top-level files/directories
- recurring structural patterns
- candidate authority artifacts
- clear evidence of project version markers if present

No parser code should be written at this step.

## Step 3 — Prove structural authority
Compare fixtures to determine what consistently governs binder structure and ordering.

The burden here is not “what seems likely.”
The burden is “what can be defended as authoritative in a contract.”

## Step 4 — Prove node-to-content mapping
Track how representative binder nodes correspond to stored content.

The goal is to identify deterministic mapping rules and failure cases before any implementation is considered.

## Step 5 — Draft admission boundaries
Once structural proof exists, draft what the future lane will and will not admit.

This step must stay conservative.
When uncertain, default to deny or unsupported.

## Step 6 — Decide whether implementation is authorized
Only after the above is complete should the gate memo decide whether to proceed to an admission ADR and implementation plan.

---

## Explicit anti-drift guardrails

This phase fails if it drifts into any of the following:

### Drift 1 — “Just parse everything in the package”
Reject this. Scrivener admission must be policy-based, not convenience-based.

### Drift 2 — “Support research too because it is there”
Reject this unless a bounded admission rule is explicitly justified and documented.

### Drift 3 — “Use compile/export semantics as truth”
Reject this. Cortex must not depend on app-export behavior as structural authority unless governance explicitly authorizes that path.

### Drift 4 — “Treat RTF conversion as a helper detail”
Reject this. RTF dependency posture is a first-class admission concern and must be bounded before implementation.

### Drift 5 — “Allow partial success by default”
Reject this. Partial behavior must be contract-defined, not implementation-invented.

### Drift 6 — “Turn Scrivener into a generic package-source template”
Reject this. Scrivener is a special-track candidate, not a shortcut to broad package ingestion.

---

## Exit criteria

Phase 0 is complete only when all of the following are true:

- a representative sanitized fixture packet exists
- structural authority is explicitly documented
- node admission policy draft exists
- node-to-content mapping is documented with denied and failure cases
- compatibility risks are documented
- implementation gate memo gives a clear go/no-go result
- no runtime code, schemas, or extraction surfaces were added during the phase

---

## Recommended next phase after completion

If Phase 0 succeeds, the next artifact should be:

- an admission ADR for Scrivener lane authorization

Only after that should a bounded implementation plan exist.

That future implementation plan should likely be structured as:

1. structural reader for authoritative project metadata
2. binder admission evaluator
3. deterministic content locator
4. bounded RTF extraction adapter
5. contract tests using fixtures
6. denied/unsupported diagnostics with privacy-preserving output

But none of that belongs in this phase.

---

## Codex handoff note

If this document is handed to Codex or another repo agent, the task must be constrained to governance and evidence artifacts only.

Agent instruction posture:

- do not add runtime code
- do not edit schemas
- do not introduce new CLI behavior
- do not create parser modules
- do not infer unsupported behavior from a single fixture
- do surface uncertainty explicitly
- do produce evidence documents that support a later admission decision

---

## Execution summary

The immediate next move for Cortex is not Scrivener implementation.

It is a disciplined Scrivener Phase 0 reconnaissance pass that proves whether the lane can be admitted without violating Cortex’s constitutional boundaries.

