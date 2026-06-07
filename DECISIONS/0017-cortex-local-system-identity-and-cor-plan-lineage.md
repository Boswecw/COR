# Decision 0017 - Cortex local-system identity and COR plan lineage

This record is governance-only.
It does not rename the repository or runtime package.
It does not change external service status.
It does not authorize semantic or workflow expansion.

## Status

Accepted

## Date

2026-06-07

## Context

This repository is Cortex under `ecosystem/local-systems`.
The Gnat planning packet uses COR terminology, but this implementation target is the Cortex local-system codebase.

The package name `cortex_runtime` remains correct for this repository.
Changing repository or package identity at the same time as new Gnat worker contracts would combine identity risk with runtime behavior risk.

## Decision

Cortex remains the repository identity and local-system service name for this implementation.

For GNAT-01, the project will retain `cortex_runtime`.
COR terminology in `docs/planning/COR_Gnats_Plan_Set/` is treated as planning lineage only and does not create a service rename.

## Deployment-role matrix

| Role | Current name | Responsibility |
|---|---|---|
| Local-system service | Cortex | Governed local intake, syntax extraction, retrieval packaging, and service truth |
| Package/import surface | `cortex_runtime` | Stable import path during GNAT-01 |
| Plan packet terminology | COR | Planning artifact wording; not a service rename in this repo |

## Boundary

This decision does not authorize:

- semantic interpretation;
- model routing;
- a general execution host;
- workflow ownership;
- automatic file watching;
- mutation of source artifacts;
- package or service rename work during GNAT-01.

## Consequences

Code may add Cortex-owned Gnat contracts and runtime modules under `cortex_runtime/gnats`.
Operator-facing documentation should remain explicit that the local-system implementation is Cortex even when plan filenames use COR terminology.
