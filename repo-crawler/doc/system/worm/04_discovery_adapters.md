# Worm Discovery Adapters

Date and Time: 2026-04-21 11:20 PM America/New_York

## Purpose

Worm uses bounded discovery adapters to turn source artifacts into structured cross-repo edges.

This slice locks the first adapter contracts:

- `gitmodules_parse`
- `package_manifest_parse`

## Adapter requirements

Every adapter must emit:
- adapter name
- source repo
- source artifact path
- emitted edge list
- skipped references list
- posture
- timestamp

## Initial adapter set

### `gitmodules_parse`
Consumes `.gitmodules` and emits `git_submodule` edges.

### `package_manifest_parse`
Consumes package manifests and emits `dependency_repo_reference` edges when a repo-style dependency reference is present.

## Contract rules

- adapters must emit typed edges only
- adapters must preserve source artifact provenance
- adapters must not guess target identity silently
- skipped or ambiguous references must be recorded explicitly
- adapter emissions must be bounded and machine-checkable
