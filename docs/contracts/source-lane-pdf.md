# Cortex Contract - Source Lane PDF

## Purpose

This document defines the bounded local PDF lane for Cortex.

The lane exists to admit local PDF files only when they can be handled as syntax-only, text-layer extraction inputs under the existing Cortex contract posture.

This lane now operates under the shared source-lane framework documented in `docs/source-lanes/`.

## Admission-readiness status

**The PDF lane is runtime-admissible on hosts where `pdfinfo` and `pdftotext` are present.**

Downstream consumers (such as AuthorForge) can determine runtime admissibility via the structured probe described below. The lane fails closed with explicit reasons when tools are absent, the file is unreadable, or the PDF has no extractable text layer. No guessing or silent fallback occurs.

## Runtime dependency truth

The PDF lane requires two host-local tools from the **poppler-utils** package:

| Tool | Purpose | Required |
| ---- | ------- | -------- |
| `pdfinfo` | Read PDF metadata, page count, encryption status | Yes |
| `pdftotext` | Extract the text layer from PDF pages | Yes |

Both tools must be present and executable. The lane does not install, fetch, or substitute for these tools.

**The lane is unavailable if either tool is absent.** See `probe_pdf_lane_admission()` below.

## Host admission probe

Use `probe_pdf_lane_admission()` from `cortex_runtime.source_lanes` to get a structured truth probe for the current host:

```python
from cortex_runtime.source_lanes import probe_pdf_lane_admission

probe = probe_pdf_lane_admission()
# probe.admitted           — bool: True only if both tools are present
# probe.pdfinfo_present    — bool: presence of pdfinfo binary
# probe.pdftotext_present  — bool: presence of pdftotext binary
# probe.operator_summary   — str: human-readable truth with specific tool names
```

The probe checks tool presence only (`shutil.which`). It does not invoke any tool or open any file. Calling it is safe with no I/O side effects beyond PATH scanning.

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

- the PDF is encrypted (detected via `pdfinfo` output or password-error stderr)
- the PDF has no extractable text layer (image-only / scanned PDF)
- the request would require OCR or image interpretation
- the source crosses other existing bounded-lane rules

`unavailable` is required when:

- `pdfinfo` or `pdftotext` is not present on the host
- `pdfinfo` or `pdftotext` is present but fails on invocation
- the PDF file is malformed or corrupt (syntax errors or unreadable xref table reported by `pdfinfo`)
- the PDF metadata or text layer cannot be read reliably enough to trust extraction

`stale` is not introduced by this lane directly.
Freshness remains governed by the existing extraction and retrieval contracts rather than by PDF-specific assumptions.

## Denial and unavailability taxonomy

The following table maps PDF-lane-specific conditions to extraction-result contract states:

| Condition | State | reason_class | operator_summary content |
| --------- | ----- | ------------ | ------------------------ |
| `pdfinfo` binary absent | `unavailable` | `dependency_unavailable` | names "pdfinfo" |
| `pdftotext` binary absent | `unavailable` | `dependency_unavailable` | names "pdftotext" |
| Both tools absent | `unavailable` | `dependency_unavailable` | names both tools |
| `pdfinfo` failed — password/encrypted | `denied` | `unsupported_source_type` | mentions encrypted |
| `pdfinfo` failed — malformed PDF | `unavailable` | `dependency_unavailable` | mentions "malformed or corrupt" |
| `pdfinfo` failed — other | `unavailable` | `dependency_unavailable` | mentions pdfinfo failure |
| Encrypted flag in pdfinfo output | `denied` | `unsupported_source_type` | mentions encrypted |
| Page count untrustworthy | `unavailable` | `dependency_unavailable` | mentions page count |
| `pdftotext` failed — password/encrypted | `denied` | `unsupported_source_type` | mentions encrypted |
| `pdftotext` failed — malformed PDF | `unavailable` | `dependency_unavailable` | mentions "malformed or corrupt" |
| `pdftotext` failed — other | `unavailable` | `dependency_unavailable` | mentions pdftotext failure |
| No extractable text blocks | `denied` | `unsupported_source_type` | mentions no text layer / no OCR |
| Content exceeds bounded limits | `denied` | `ineligible_source` | mentions extraction limits |
| Partial text (some pages empty) | `partial_success` | — | completeness.status = incomplete |

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
