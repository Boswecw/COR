# Cortex Contract - Source Lane EPUB

## Purpose

This document defines the bounded local EPUB lane for Cortex.

The lane exists to admit local `.epub` files only when Cortex can establish package truth honestly and recover syntax-only textual structure through the existing extraction-result contract.

## Admission

This lane admits only:

- local file paths
- `.epub` sources
- declared media type `application/epub+zip` when media type is present
- readable EPUB packages with a valid EPUB mimetype declaration
- packages whose container, package document, manifest, and spine truth can be established without fallback behavior
- spine members whose declared media type is `application/xhtml+xml`

This lane does not admit:

- remote EPUB sources
- unpacked loose EPUB directory structures
- non-EPUB zip containers
- package variants whose authority chain cannot be verified truthfully
- active-content or media-bearing content documents in the admitted textual path

## Authority model

EPUB lane v1 recovers authority only through this bounded package chain:

1. archive readability
2. EPUB mimetype declaration
3. `META-INF/container.xml`
4. a single package document
5. manifest membership
6. spine reading-order membership
7. admitted XHTML content documents

Cortex does not derive authority from rendering behavior, CSS, reader heuristics, or semantic interpretation.

## Extraction scope

EPUB lane v1 admits only:

- paragraphs
- headings when recoverable from explicit XHTML heading elements
- simple list items when recoverable from explicit XHTML list elements without nested lists
- bounded plain table text when row and cell order are deterministic and nested table/list recovery is not required

EPUB lane v1 does not admit:

- rendering or layout reconstruction
- CSS or style semantics beyond narrow structural recovery
- chapter meaning, narrative role, or editorial significance
- image, audio, video, or embedded-object interpretation
- script or active-content semantics
- annotation, comment, or review semantics
- semantic labels or summaries

## Metadata posture

EPUB lane v1 exposes only bounded structure metadata already allowed by the extraction contract:

- file name
- file extension
- source lane identifier

It does not expose package metadata beyond that bounded surface in this slice.

## Completeness posture

`ready` is allowed only when bounded syntax-only structure is recoverable honestly across the admitted spine members.

`denied` is required when:

- the source is outside the EPUB lane boundary
- the declared media type is outside the admitted EPUB lane
- the EPUB mimetype declaration is not `application/epub+zip`
- the admitted textual path includes active, scripted, media, or other explicitly excluded content structures
- the XHTML structure exceeds the bounded deterministic recovery surface
- the package has no bounded extractable text structures

`unavailable` is required when:

- the EPUB file cannot be read
- the package is corrupt
- required package authority files are missing
- container, package, or content XML cannot be parsed safely enough to trust extraction
- manifest or spine truth is too broken to establish an honest admitted reading path

`partial_success` is not introduced by EPUB lane v1.

## Retrieval compatibility

Ready EPUB extraction outputs may flow into the existing retrieval-package path only through the existing extraction-result contract.

Retrieval remains:

- deterministic
- syntax-derived
- non-ranking
- non-semantic
- non-canonical

## Explicit exclusions

This lane explicitly excludes:

- browser behavior
- ebook-reader behavior
- generic archive or package abstractions beyond the lane contract
- rendering or layout reconstruction
- media interpretation
- script or active-content meaning
- workflow hints
- downstream action language
