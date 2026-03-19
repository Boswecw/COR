# Cortex Contract - Source Lane RTF

## Purpose

This document defines the bounded local RTF lane for Cortex.

The lane exists to admit local `.rtf` files only when Cortex can recover bounded paragraph text honestly through the existing extraction-result contract.

## Admission

This lane admits only:

- local file paths
- `.rtf` sources
- declared media type `application/rtf` or `text/rtf` when media type is present
- RTF sources that can be parsed safely enough to trust paragraph recovery

This lane does not admit:

- remote RTF sources
- review or annotation semantics
- embedded object or media interpretation
- field or layout semantics
- rendering-faithful output

## Extraction scope

RTF lane v1 admits only:

- paragraph text
- explicit paragraph boundaries
- limited literal character recovery from basic escaped hex and unicode forms

RTF lane v1 does not admit:

- headings
- lists
- tables
- comments or annotations
- layout semantics
- semantic styling claims
- summaries

## Metadata posture

RTF lane v1 exposes only bounded structure metadata already allowed by the extraction contract:

- file name
- file extension
- source lane identifier

## Completeness posture

`ready` is allowed only when bounded paragraph text is recoverable honestly.

`denied` is required when:

- the RTF contains annotation, comment, field, media, or other rich destinations outside the bounded paragraph-only lane
- the RTF has no bounded extractable paragraph text
- the source crosses other existing bounded-lane rules

`unavailable` is required when:

- the RTF file cannot be read
- the RTF syntax cannot be trusted well enough to support bounded extraction

`partial_success` is not introduced by RTF lane v1.

## Retrieval compatibility

Ready RTF extraction outputs may flow into the existing retrieval-package path only through the existing extraction-result contract.

Retrieval remains:

- deterministic
- paragraph-bounded
- non-ranking
- non-semantic
- non-canonical

## Explicit exclusions

This lane explicitly excludes:

- annotation or comment semantics
- embedded object interpretation
- image interpretation
- field semantics
- layout-faithful reconstruction
- workflow hints
- downstream action language
