# Decision 0014 - Next lane selection after Slice 9

This record is governance-only and does not authorize implementation by itself.

## Status

Accepted

## Date

2026-03-19

## Context

Cortex is now at a clean post-Slice-9 baseline. The following are now true:

- runtime slices 1 through 9 are delivered
- admitted lanes include `.md`, `.txt`, text-layer `.pdf`, `.docx`, `.rtf`, `.odt`, and `.epub`
- post-Slice-8 governance has now been executed rather than merely recorded
- EPUB was admitted without browser, semantic, workflow, or generic-package drift
- HTML remains deferred
- Scrivener remains a special-track project-source candidate

With EPUB complete, Cortex requires a new explicit governance decision before any further target is allowed to gather parser momentum.

The candidate set considered at this point is:

- `HTML`
- special-track `Scrivener`

## Decision

Cortex selects special-track `Scrivener` as the next planning target after Slice 9.

This decision is governance-only.

Implementation may begin only after a bounded Scrivener admission draft and implementation plan are written and accepted.

## Why Scrivener was selected

Scrivener is selected because:

- continuing routine lane momentum now effectively means confronting HTML next
- HTML remains the highest browser and rendering drift-pressure option in the current state
- Scrivener offers higher strategic value for the wider Forge ecosystem if opened intentionally as a read-only project-source track
- selecting Scrivener now creates a cleaner governance reset than letting EPUB momentum push Cortex toward browser-shaped behavior

## Why HTML was not selected

HTML is deferred because it continues to carry the strongest rendering, boilerplate, style, and browser-adjacent drift pressure in the current candidate set.

Selecting HTML next would force Cortex into web-content ambiguity sooner than necessary and would create stronger pressure toward generic document or browser behavior than the repo should take on next.

## Consequences

### Immediate consequences

- a bounded Scrivener admission draft should be created
- a Scrivener implementation plan may be prepared only after the draft is accepted
- no HTML parser work is authorized by this decision

### Boundary consequences

This decision does not authorize:

- generic project or package abstractions
- compile or export semantics
- editorial or workflow semantics
- application-host or sync behavior
- hidden schema widening beyond truthful contract need

### Deferred items

- HTML remains a future candidate for a deliberate browser-drift governance pass
- Scrivener remains special-track and must not be treated as routine file-lane expansion
