# AGENTS.md

## Repository role

Cortex is a **standalone constitutional workspace/repo** in the Forge ecosystem.

It is a **bounded local file-intelligence service**, not a product, not a semantic engine, not an orchestrator, not a workflow controller, and not a generic ETL platform.

The purpose of this repo is to implement and validate **bounded contract truth** for governed local source intake, syntax-only extraction, and retrieval-package preparation under explicit constitutional limits.

---

## Governing authority

The governing authority for this repository is:

**Cortex — Constitutional Project Plan v2.1**

When there is any tension between:

- this file
- implementation convenience
- inferred product opportunity
- prior prompts
- generated build plans
- runtime ambition
- parser availability
- abstraction preferences

**v2.1 wins.**

Do not reinterpret the repo beyond that authority.

---

## Non-negotiable boundaries

Keep these hard:

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
- a summarizer
- a classifier
- a tagger
- a ranker
- a dispatcher
- a queue manager
- a coordinator
- an executor selector
- an agent host
- a general ingestion platform

If a proposed implementation adds meaning, importance, recommendation, workflow, routing, or executor behavior:

**do not implement it.**

---

## Current runtime status

Completed:

1. **Runtime Slice 1 — intake validation**
2. **Runtime Slice 2 — syntax-only extraction-result emission**
3. **Runtime Slice 3 — one governed retrieval-package emission path**
4. **Runtime Slice 4 — service-status truth path**
5. **Runtime Slice 5 — bounded PDF source lane**
6. **Runtime Slice 6 — bounded DOCX source lane**
7. **Runtime Slice 7 — bounded RTF source lane**
8. **Runtime Slice 8 — bounded ODT source lane**
9. **Runtime Slice 9 — bounded EPUB source lane**

Current runtime posture remains narrow:

- intake validation exists
- syntax-only extraction emission exists
- governed retrieval-package emission exists
- service-status truth exists
- bounded local PDF lane exists for text-layer PDFs only
- bounded local DOCX lane exists for local `.docx` files only
- bounded local RTF lane exists for local `.rtf` files only
- bounded local ODT lane exists for local `.odt` files only
- bounded local EPUB lane exists for local `.epub` files only
- shared source-lane framework now governs admitted-lane registration, failure posture, provenance, and service-status reporting
- supported source paths are currently narrow and governed
- no semantic authority has been added
- no orchestration or workflow behavior has been added

Do not change the completed delivery order without explicit governing-plan updates.

---

## Active implementation target

Unless the user explicitly redirects the task, the active next step is:

**No default runtime expansion target is implied past Runtime Slice 9 without an explicit user request anchored to v2.1.**

This means:

- preserve the bounded slices already implemented
- keep status, extraction, and retrieval surfaces schema-backed and fail-closed
- avoid inventing broader control, orchestration, or semantic ownership
- treat any further runtime slice as explicit new work rather than implied backlog pull-forward

This does **not** mean:

- semantic expansion
- retrieval judgment
- workflow hints
- retry/dispatch behavior
- handoff coordination
- executor or agent behavior

---

## Source support posture

Cortex should support source families only through **explicit governed source lanes**.

Do not treat “documents” as one broad abstraction.

The rule is:

**Cortex supports specific source lanes, not generic document ingestion.**

Each source lane must be:

- explicitly allowed
- contract-bounded
- independently testable
- degradation-aware
- fail-closed
- unable to inject semantic interpretation

Current strong/near-term source posture:

- `.md`
- `.txt`
- text-based `.pdf` as a bounded/degraded lane
- `.docx` as an admitted bounded local lane
- `.rtf` as an admitted bounded local paragraph-only lane
- `.odt` as an admitted bounded local structured-authoring lane
- Scrivener only as a specialized read-only project-source lane if admitted later

Do not add multiple new source families in one pass unless the task explicitly requires it and the implementation remains constitutionally narrow.

---

## Scrivener posture

If Scrivener support is worked on in this repo, it must be treated as a **special governed source family**, not just another file type.

Allowed posture:

- read-only only
- `.scrivx` as structural authority
- eligible internal text artifacts only as bounded content sources
- provenance-preserving extraction only

Not allowed:

- write-back into project internals
- compile/export orchestration
- workflow interpretation
- editorial meaning inference
- broad observation of every internal file

If this boundary cannot be preserved, do not implement the lane.

---

## Implementation style

Prefer:

- small pure functions
- explicit boundary checks
- deterministic ordering
- obvious denial/unavailability paths
- direct schema validation before success return
- narrow extensions over speculative frameworks

Avoid:

- generalized ingestion abstractions
- plugin systems for hypothetical future formats
- semantic repair behavior
- silent fallback chains
- hidden widening of support
- speculative extensibility that expands authority

This repo should stay narrow on purpose.

---

## Validation requirements

Before claiming work complete, run the repo validation/test commands appropriate to the change.

At minimum, use the repo’s existing validation wiring.

Typical required commands:

```bash
make validate
make test-runtime
```

If a command name differs in the current repo state, use the repo’s actual documented equivalent and state exactly what was run.

Do not claim success without reporting command results.

---

## Documentation update requirements

If runtime behavior changes, update the relevant docs narrowly and truthfully.

Likely docs include:

- `README.md`
- `doc/system/03-contract-surface.md`
- `doc/system/04-validation-and-delivery.md`
- rebuilt assembled `doc/cxSYSTEM.md` when system-source docs changed

Do not overstate capability.

Do not document planned behavior as implemented behavior.

---

## Required completion report format

When returning implementation work, use this exact structure:

1. **Executive summary**
2. **Files created**
3. **Files modified**
4. **Runtime behavior added**
5. **Tests added**
6. **Validation result**
7. **Drift check**
8. **Final status**

In **Drift check**, explicitly state whether the change introduced movement toward:

- semantic authority
- retrieval authority
- workflow ownership
- orchestration
- generic ETL
- executor/agent hosting

If yes, explain exactly where.
If no, explain why not.

---

## Final operating rule

Do not redesign Cortex through implementation.

Do not broaden scope because a parser or library makes it easy.

Do not add semantics because they seem useful.

Do not add workflow behavior because it feels practical.

Implement only the **narrowest correct bounded service behavior** consistent with the constitutional posture.
