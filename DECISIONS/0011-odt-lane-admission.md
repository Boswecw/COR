# ADR 0011 - ODT Lane Admission

## Status

Accepted

## Context

After the formal next-lane evaluation phase, ODT was selected as the strongest remaining conventional authoring-document candidate for Cortex.

It offers clearer package-level structural authority than HTML or EPUB and does not change the authority model the way a Scrivener project-source lane would.

## Decision

Cortex admits a bounded local ODT lane with these limits:

- local `.odt` only
- declared media type `application/vnd.oasis.opendocument.text` only when media type is present
- syntax-only extraction only
- headings only from explicit `text:h` structure with bounded outline levels
- lists only from explicit `text:list` structure
- bounded table text only when row and cell order are deterministic
- no `partial_success` in v1
- comments, annotations, tracked changes, and embedded object/media structures are denied
- corrupt, unreadable, or structurally untrustworthy packages are unavailable

## Consequences

Positive:

- extends the governed source-lane framework to another structured authoring format
- preserves retrieval-package neutrality by staying section-bounded when explicit heading structure exists and paragraph-bounded otherwise
- keeps ODT inside zip/XML package truth rather than render-driven behavior

Negative:

- many richer ODT documents will fail closed rather than degrade into best-effort output
- ODT does not authorize broader office-suite or compound-document abstractions

## Rejected alternatives

- office-rendering or conversion workflows as the lane authority
- comments or tracked-change recovery
- embedded object or media interpretation
- generic rich-document abstractions spanning PDF, DOCX, RTF, and ODT
