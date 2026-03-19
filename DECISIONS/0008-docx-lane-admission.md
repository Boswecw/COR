# ADR 0008 - DOCX Lane Admission

## Status

Accepted

## Context

DOCX is a common local authoring format whose structure is more recoverable than PDF for syntax-first extraction.

Cortex needs a next admitted lane after PDF that strengthens structural honesty without adding semantics or office-suite sprawl.

## Decision

Cortex admits a bounded local DOCX lane with these limits:

- local `.docx` only
- syntax-only extraction only
- headings only when explicit paragraph-style evidence exists
- lists only when explicit numbering or list-style evidence exists
- bounded table text only when deterministic
- comments and tracked changes are denied
- corrupt or unreadable packages are unavailable

## Consequences

Positive:

- expands governed capability to a common authoring format
- strengthens the shared source-lane framework
- preserves deterministic retrieval-package compatibility

Negative:

- DOCX support remains intentionally incomplete
- review and layout-heavy documents fail closed rather than degrading into best-effort output

## Rejected alternatives

- full office-document semantics
- tracked-changes recovery
- comment extraction
- rendering-faithful layout reconstruction
