# Cortex Source Lanes

This directory defines the shared admission framework for governed Cortex source lanes.

Cortex does not support "documents" as one broad ingestion category.
Cortex admits only explicit local source lanes with:

- bounded eligibility
- bounded failure posture
- bounded provenance
- deterministic syntax-only recovery
- independent tests

Current admitted lanes:

- local Markdown files
- local plain-text files
- local text-layer PDF files
- local DOCX files
- local RTF files
- local ODT files

Lane-specific rules remain in `docs/contracts/source-lane-*.md`.
Shared lane rules live in this directory so future lane admission does not become a series of one-off branches.

Governance references in this directory:

- `lane-admission-criteria.md`
- `shared-failure-taxonomy.md`
- `shared-provenance-model.md`
- `lane-admission-playbook.md`
- `contract-symmetry-audit.md`
- `next-lane-candidate-matrix.md`
- `odt-admission-draft.md` (historical)
