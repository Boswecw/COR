# ADR 0009 - RTF Lane Admission

## Status

Accepted

## Context

After the shared source-lane framework and the DOCX lane were shipped, Cortex needed one more narrow authoring-text lane that could reuse the same admission, provenance, and fail-closed posture without broadening into a generic office-document platform.

RTF is useful as a local authoring-text format, but its reliable structural truth is weaker than DOCX.

## Decision

Cortex admits a bounded local RTF lane with these limits:

- local `.rtf` only
- media types `application/rtf` and `text/rtf` only when declared
- paragraph-only extraction in v1
- no headings, lists, or tables in v1
- no `partial_success` in v1
- annotation, comment, field, object, media, and other rich destinations are denied
- corrupt or syntactically untrustworthy sources are unavailable

## Consequences

Positive:

- extends the shared source-lane framework to another bounded authoring-text lane
- preserves lane honesty by keeping RTF narrower than DOCX
- keeps retrieval-package emission lane-neutral and paragraph-bounded

Negative:

- many rich RTF documents will fail closed rather than degrade into best-effort output
- RTF does not inherit DOCX-like structure claims

## Rejected alternatives

- headless office conversion as a required runtime dependency
- generic rich-document recovery shared across PDF, DOCX, and RTF
- heading/list/table recovery without explicit trustworthy RTF evidence
