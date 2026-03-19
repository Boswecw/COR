# Scrivener Fixtures

## Purpose

This directory contains Scrivener project fixtures for Cortex Scrivener Phase 0 reconnaissance and any later contract-grade validation.

Fixtures are expected to be sanitized by default.

These fixtures are governance and evidence assets first.

Their presence does not authorize:

- parser implementation
- runtime lane admission
- schema changes
- convenience-based scope broadening

## Allowed uses

Fixtures in this directory may be used for:

- structural reconnaissance
- authority proof work
- admission-policy validation
- compatibility characterization
- same-source clean vs malformed-authority comparison when provenance is explicit
- future contract tests after explicit lane admission

Fixtures in this directory may not be used to justify:

- support for unobserved structures
- hidden fallback rules
- claims of cross-version compatibility beyond observed evidence
- broad research or project ingestion by convenience

## Required preserved artifacts

A valid Scrivener fixture should preserve enough internal structure to support all of the following where applicable:

- `.scriv` container layout
- candidate `*.scrivx` authority file
- binder hierarchy evidence
- binder item ordering evidence
- manuscript or draft subtree evidence if present
- binder-node to content-file mapping evidence
- representative content storage artifacts
- representative metadata artifacts only when needed for authority proof

## Sanitization requirements

Fixtures must be sanitized before inclusion unless they are synthetic from creation or explicitly retained as restricted archive-backed negative evidence.

Sanitization expectations:

- remove or replace real manuscript prose where possible
- remove private names, emails, addresses, and identifying annotations
- sanitize project titles where needed
- remove non-essential attachments and media
- sanitize notes, comments, synopsis, labels, keywords, custom metadata, and inspector content where needed
- preserve structural relationships needed for proof
- preserve ambiguity when that ambiguity itself is evidence

Restricted exception:

- an intentionally malformed-authority negative packet may retain a derived real-project archive when the evidence question is fail-closed authority handling rather than content semantics
- when a sanitized derivative exists for that source, the canonical malformed-authority packet should be derived from the sanitized derivative first
- a raw-derived malformed-authority packet may be retained only as superseded provenance history and must not remain the canonical negative packet once a sanitized-derived replacement exists
- in that case, keep the packet archive-backed and restricted rather than expanding the project tree again
- such packets are governance evidence only and must not be treated as support proof or future automated test fixtures

## Fixture classes

Recommended fixture classes:

- `positive/`
- `negative/`
- `compat/`
- `unsupported/`
- `ambiguous/`

## Canonical layout

Store each fixture as its own directory beneath the class directory.

Recommended per-fixture shape:

- `<class>/<fixture-id>/README.md`
- `<class>/<fixture-id>/sanitization-report.md` when applicable
- `<class>/<fixture-id>/<fixture-id>.scriv/`
- `<class>/<fixture-id>/<fixture-id>.zip` only when retaining the supplied archive materially helps provenance or reinspection

This keeps provenance, extracted structure, and any retained delivery artifact in one bounded packet.

If a retained delivered archive has a provenance-significant filename, it may keep that original name instead of being forced to `<fixture-id>.zip`.

When that happens, the packet `README.md` must tie the retained archive name to the fixture id explicitly.

For restricted negative packets derived from real-project copies, an archive-backed packet with `README.md` plus the retained archive is acceptable when that avoids unnecessary duplication of unsanitized surrounding material.

## Naming convention

Use stable descriptive names such as:

- `scriv-simple-manuscript-v1`
- `scriv-manuscript-research-nested-v1`
- `scriv-missing-content-negative-v1`
- `scriv-legacy-compat-v1`
- `scriv-authority-ambiguous-v1`

Do not use names that imply support beyond observed evidence.

## Per-fixture minimum metadata

Each fixture must have an index entry recording:

- fixture id
- class
- source provenance
- platform if known
- Scrivener version if known
- sanitization status
- key structural observations
- intended use
- known limitations

## Future test posture

If Scrivener is admitted later, these fixtures may be promoted into contract tests for:

- authority detection
- admission decisions
- content-location correctness
- denied and unsupported diagnostics
- malformed-structure fail-closed behavior

Until then, they remain evidence assets only.
