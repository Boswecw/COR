# Cortex Contract - Source Lane ODT

## Purpose

This document defines the bounded local ODT lane for Cortex.

The lane exists to admit local `.odt` files only when Cortex can recover syntax-only OpenDocument text structure honestly through the existing extraction-result contract.

## Admission

This lane admits only:

- local file paths
- `.odt` sources
- readable OpenDocument text packages
- declared media type `application/vnd.oasis.opendocument.text` when media type is present
- package contents whose bounded syntax surface can be recovered without rendering or semantic interpretation

This lane does not admit:

- remote ODT sources
- render-derived output
- comments, annotations, or revision semantics
- embedded object or media interpretation
- macro or script semantics

## Extraction scope

ODT lane v1 admits only:

- paragraphs
- headings when explicit `text:h` structure carries a bounded outline level
- simple list items when explicit `text:list` structure is present
- bounded plain table text when row and cell order are deterministic

ODT lane v1 does not admit:

- layout or page-faithful reconstruction
- comments or annotations
- tracked changes or revision meaning
- embedded objects or images
- style meaning beyond narrow structural recovery
- semantic labels
- summaries

## Metadata posture

ODT lane v1 exposes only bounded structure metadata already allowed by the extraction contract:

- file name
- file extension
- source lane identifier

It does not expose package internals or broader office metadata in this slice.

## Completeness posture

`ready` is allowed only when bounded syntax-only structure is recoverable honestly.

`denied` is required when:

- the source is outside the ODT lane boundary
- the declared media type is outside the admitted ODT lane
- the package includes comments, annotations, tracked changes, or embedded object/media structures outside the bounded lane
- the document shape exceeds the bounded deterministic recovery surface
- the source has no bounded extractable text structures

`unavailable` is required when:

- the ODT file cannot be read
- the package is corrupt
- `content.xml` is missing
- required XML cannot be parsed safely enough to trust extraction

`partial_success` is not introduced by ODT lane v1.

## Retrieval compatibility

Ready ODT extraction outputs may flow into the existing retrieval-package path only through the existing extraction-result contract.

Retrieval remains:

- deterministic
- syntax-derived
- non-ranking
- non-semantic
- non-canonical

## Explicit exclusions

This lane explicitly excludes:

- rendering or layout reconstruction
- comments, annotations, or review semantics
- tracked changes or revision semantics
- embedded object or media interpretation
- macro or script semantics
- workflow hints
- downstream action language
