# ADR 0013 - EPUB Lane Admission

## Status

Accepted

## Context

After the post-Slice-8 governance pass, EPUB was selected as the next bounded source-lane target for Cortex.

It introduces more package authority than ODT, but it still fits the existing source-lane framework better than HTML and does not open the distinct project-source questions raised by Scrivener.

## Decision

Cortex admits a bounded local EPUB lane with these limits:

- local `.epub` only
- declared media type `application/epub+zip` only when media type is present
- syntax-only extraction only
- package authority only through the EPUB mimetype declaration, `META-INF/container.xml`, one package document, manifest membership, and spine membership
- admitted textual members are XHTML spine members only
- headings only from explicit XHTML heading elements
- lists only from explicit XHTML list elements without nested-list recovery
- bounded table text only when row and cell order are deterministic
- no `partial_success` in v1
- active content, navigation documents in the admitted textual path, and media-bearing content structures are denied
- corrupt, unreadable, or structurally untrustworthy packages are unavailable

## Consequences

Positive:

- extends the governed source-lane framework to a bounded composite publication package
- preserves retrieval-package neutrality by staying section-bounded when explicit heading structure exists and paragraph-bounded otherwise
- keeps EPUB inside explicit package authority rather than browser or reader behavior

Negative:

- many richer EPUB documents will fail closed rather than degrade into best-effort output
- EPUB admission does not authorize generic archive, browser, or ebook-platform abstractions

## Rejected alternatives

- browser-style HTML recovery as the EPUB lane authority
- navigation, scripting, or active-content interpretation
- image, audio, video, or embedded-object interpretation
- generic package abstractions spanning EPUB, ODT, DOCX, and future composite formats
