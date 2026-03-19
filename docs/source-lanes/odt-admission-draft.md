# Cortex Draft Admission Posture - ODT Candidate

## Status

Historical draft.

ODT is now admitted through `docs/contracts/source-lane-odt.md` and `DECISIONS/0011-odt-lane-admission.md`.
This file remains as the pre-admission selection artifact.

## Authority model

The ODT candidate lane would treat OpenDocument text as a bounded local package-source lane.

Structural authority would come from explicit OpenDocument XML package contents, not from rendering output and not from semantic interpretation.

## Admitted input boundary

The draft ODT lane would admit only:

- local `.odt` file paths
- readable OpenDocument packages
- parseable text-bearing package content sufficient for bounded syntax-only recovery
- declared media type `application/vnd.oasis.opendocument.text` when media type is present

The draft lane would not admit:

- remote ODT sources
- viewer-rendered output
- office-suite conversion dependencies as a required truth path

## Draft admitted extraction surface

ODT v1 would admit only what can be recovered honestly and deterministically:

- paragraphs
- headings only when explicit structural or style-family evidence is trustworthy
- simple list items only when explicit list markup exists
- bounded table text only when table and cell order are deterministic
- bounded lane metadata already allowed by the extraction contract

## Explicit exclusions

ODT v1 would exclude:

- comments or annotation semantics
- tracked changes or review semantics
- layout-faithful rendering claims
- embedded object or media interpretation
- script or macro behavior
- semantic labels
- summaries
- workflow hints

## Draft denial conditions

The draft lane should deny when:

- the package is encrypted or otherwise outside the bounded local package lane
- comments, tracked changes, annotations, or embedded objects are required to interpret the document honestly
- the structure would require style or layout semantics beyond the bounded syntax-only surface
- the source has no bounded extractable text structures
- literal structures exceed bounded extraction limits

## Draft unavailable conditions

The draft lane should be unavailable when:

- the `.odt` file cannot be read
- the package is corrupt
- required XML parts cannot be parsed safely enough to trust extraction
- package truth is too broken to establish bounded structure honestly

## `partial_success` posture

`partial_success` is not recommended for ODT v1.

The cleaner posture is:

- `ready` for fully honest bounded recovery
- `denied` when the source exceeds the admitted lane
- `unavailable` when package truth cannot be trusted

## Expected provenance needs

The draft ODT lane should preserve the existing shared provenance model:

- `source_hash`
- `extractor_version`
- `source_modified_at` when available
- `byte_count` when available
- bounded structure metadata for file name, extension, and source lane

## Expected fixture families before implementation

Required fixture families would include:

- ready structured text ODT
- denied tracked-change or annotation ODT
- denied embedded-object-heavy ODT
- corrupt or unreadable ODT
- suffix and media-type mismatch cases
- retrieval-compatibility cases
- awkward but valid structure cases that must not over-promote headings or layout meaning
