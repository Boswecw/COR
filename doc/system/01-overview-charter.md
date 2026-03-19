# 1. Overview and Charter

## Purpose

Cortex is the bounded local file-intelligence, extraction, retrieval-preparation, and handoff-support service for Forge applications.

Its Phase 1 purpose is narrow:

- intake eligible local content
- extract syntax-level structure and metadata
- prepare one governed retrieval package form
- emit bounded handoff transfer truth
- surface truthful operational state without privacy collapse
- expose redacted embedded diagnostics for Cortex-owned surfaces only

## Constitutional role

Cortex is a service-only internal runtime subsystem.

It must not become:

- a standalone product
- a semantic engine
- a workflow coordinator
- a generic ETL platform
- a canonical truth store

## Success posture

Cortex is only successful if it remains:

- bounded by contract
- fail-closed when trust is insufficient
- privacy-preserving by default
- non-semantic by default
- freshness-bound rather than silently stale
- visible only through consuming applications
- unable to drift into workflow, queue, or executor ownership

## Current bounded runtime baseline

The currently implemented executable runtime surfaces are:

- Slice 1 - intake validation
- Slice 2 - syntax-only extraction-result emission
- Slice 3 - one governed retrieval-package emission path
- Slice 4 - service-status truth

The currently admitted source lanes remain narrow:

- local `.md`
- local `.txt`

This is the current bounded baseline, not a promise of broader source or control-surface expansion.

## Foundational references

This section is grounded in:

- `PROJECT_CHARTER.md`
- `PHASE_1_PLAN.md`
- `README.md`
