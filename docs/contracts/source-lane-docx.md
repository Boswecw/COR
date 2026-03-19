# Cortex Contract - Source Lane DOCX

## Purpose

This document defines the bounded local DOCX lane for Cortex.

The lane exists to admit local `.docx` files only when Cortex can recover syntax-only structure honestly through the existing extraction-result contract.

## Admission

This lane admits only:

- local file paths
- `.docx` sources
- readable OpenXML packages
- document text and bounded structural markers that are recoverable without semantic interpretation

This lane does not admit:

- remote DOCX sources
- comments or review semantics
- tracked-changes semantics
- embedded object or media interpretation
- layout-faithful rendering claims

## Extraction scope

DOCX lane v1 admits only:

- paragraphs
- headings when recoverable from explicit WordprocessingML paragraph styles
- simple list items when recoverable from explicit numbering or list-style markers
- bounded plain table text when table rows and cell order are deterministic

DOCX lane v1 does not admit:

- comments
- tracked changes
- page layout meaning
- rendering fidelity
- style meaning beyond bounded structural recovery
- semantic labels
- summaries

## Metadata posture

DOCX lane v1 exposes only bounded structure metadata already allowed by the extraction contract:

- file name
- file extension
- source lane identifier

It does not expose broader DOCX core-properties metadata in this slice.

## Completeness posture

`ready` is allowed only when bounded syntax-only structure is recoverable honestly.

`denied` is required when:

- the DOCX contains comments or tracked changes
- the DOCX requires semantics outside the admitted lane
- the DOCX contains boundedly unsupported nested structure

`unavailable` is required when:

- the DOCX package is corrupt
- the DOCX package cannot be read
- required package parts cannot be parsed safely enough to trust extraction

`partial_success` is not introduced by DOCX lane v1.

## Retrieval compatibility

Ready DOCX extraction outputs may flow into the existing retrieval-package path only through the existing extraction-result contract.

Retrieval remains:

- deterministic
- syntax-derived
- non-ranking
- non-semantic
- non-canonical

## Explicit exclusions

This lane explicitly excludes:

- tracked changes semantics
- comments or review semantics
- embedded media interpretation
- chart semantics
- footnote or endnote meaning
- workflow hints
- downstream action language
