# Cortex Shared Source-Lane Provenance Model

## Purpose

This document defines the shared provenance posture for admitted Cortex source lanes.

## Required provenance

Every extraction-result emitted through a source lane must preserve:

- `source_hash`
- `extractor_version`
- `source_modified_at` when available
- `byte_count` when available

## Shared structure metadata

Each admitted lane may also report bounded structure metadata fields that remain literal and non-semantic.

At minimum, the runtime now uses:

- `file_name`
- `file_extension`
- `source_lane`

Lane-specific metadata may be added only when:

- it is deterministic
- it does not imply semantic meaning
- it does not expose raw content or privacy-sensitive detail beyond the existing contract

## Forbidden provenance drift

Do not add:

- semantic certainty markers
- importance or relevance claims
- workflow ownership hints
- raw-content previews in provenance or diagnostics
