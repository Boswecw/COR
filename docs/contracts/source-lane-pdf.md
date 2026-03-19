# Cortex Contract - Source Lane PDF

## Purpose

This document defines the bounded local PDF lane for Cortex.

The lane exists to admit local PDF files only when they can be handled as syntax-only, text-layer extraction inputs under the existing Cortex contract posture.

This lane now operates under the shared source-lane framework documented in `docs/source-lanes/`.

## Admission

This lane admits only:

- local file paths
- `.pdf` sources
- PDF inputs with extractable text

This lane does not admit:

- OCR
- scanned-image interpretation
- image understanding
- annotations or comments as semantic signals
- embedded media interpretation
- remote PDFs
- directory-wide PDF crawling

## Extraction scope

The PDF lane is text-only.

Allowed extraction scope:

- literal text obtained from the PDF text layer
- bounded file metadata already allowed by Cortex provenance or structures
- deterministic paragraph ordering
- page-count metadata when reported as bounded structure metadata

Not allowed:

- summaries
- semantic labels
- inferred headings where the PDF does not provide explicit text markers
- importance ranking
- relevance ranking
- recommendations

## Provenance posture

PDF extraction must preserve bounded provenance, including:

- source hash
- extractor version
- source modified time when available
- byte count

The lane may also report bounded PDF metadata such as page count through structure metadata fields.

## Completeness posture

`ready` is allowed only when the PDF text layer is readable and the bounded extraction path can produce complete syntax-only structures for the extracted text.

`partial_success` is allowed only when some PDF pages yield bounded text-layer extraction while one or more pages yield no extractable text.

`denied` is required when:

- the PDF is encrypted
- the PDF has no extractable text layer
- the request would require OCR or image interpretation
- the source crosses other existing bounded-lane rules

`unavailable` is required when:

- the PDF cannot be read
- the bounded PDF tooling is unavailable
- the PDF metadata or text layer cannot be read reliably enough to trust extraction

`stale` is not introduced by this lane directly.
Freshness remains governed by the existing extraction and retrieval contracts rather than by PDF-specific assumptions.

## Retrieval compatibility

Ready PDF extraction outputs may flow into the existing retrieval-package path only through the existing extraction-result contract.

Retrieval packaging must remain:

- deterministic
- syntax-derived
- non-semantic
- non-ranking
- non-canonical

## Explicit exclusions

This lane explicitly excludes:

- OCR
- scanned-image interpretation
- handwritten-text interpretation
- annotation or comment semantics
- form-field semantics
- embedded media interpretation
- attachment extraction
- script execution
- workflow hints
- downstream action language
