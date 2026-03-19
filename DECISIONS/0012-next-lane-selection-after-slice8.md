# Decision 0012 - Next lane selection after Slice 8

This record is governance-only and does not authorize implementation by itself.

## Status

Accepted

## Date

2026-03-19

## Context

Cortex is at a clean post-Slice-8 baseline. The following are now true:

- runtime slices 1 through 8 are delivered
- admitted lanes include `.md`, `.txt`, text-layer `.pdf`, `.docx`, `.rtf`, and `.odt`
- the shared source-lane framework is proven across multiple governed families
- ODT was admitted without semantic, workflow, orchestration, or document-platform drift

With ODT complete, Cortex requires a new explicit governance decision before any further lane work begins.

The candidate set considered at this point is:

- `HTML`
- `EPUB`
- special-track `Scrivener`

## Decision

Cortex selects `EPUB` as the next planning target after Slice 8.

This decision is governance-only.

Implementation may begin only after a bounded EPUB admission draft and implementation plan are written and accepted.

## Why EPUB was selected

EPUB is the cleanest next routine source-lane candidate because:

- it remains document-centered rather than browser-centered
- its main pressure is composite package governance, not rendering drift
- it fits the existing source-lane framework better than HTML
- it avoids collapsing routine lane expansion into the separate project-source question raised by Scrivener

## Why HTML was not selected

HTML is deferred because it continues to carry the strongest rendering and web-behavior drift pressure in the current candidate set.

Selecting HTML next would force Cortex to confront browser-adjacent ambiguity and style and layout interpretation pressures sooner than necessary.

That is not the preferred next step after a clean ODT admission.

## Why Scrivener was not selected

Scrivener is deferred because it remains a special-track project-source lane, not a routine format peer.

Opening Scrivener next would change the type of expansion being undertaken. That move may be valid later, but it should occur only through an explicit decision to open the project-source track.

## Consequences

### Immediate consequences

- a bounded EPUB admission draft should be created
- an EPUB implementation plan may be prepared after the admission draft
- no HTML or Scrivener parser work is authorized by this decision

### Boundary consequences

This decision does not authorize:

- generic rich-document abstraction
- browser or rendering interpretation behavior
- editorial, workflow, or project semantics
- hidden schema widening beyond truthful contract need

### Deferred items

- HTML remains a future candidate for a deliberate rendering-pressure governance pass
- Scrivener remains a future special-track candidate for a deliberate project-source governance pass
