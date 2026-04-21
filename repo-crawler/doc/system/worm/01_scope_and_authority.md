# Worm Scope and Authority

Date and Time: 2026-04-21 10:45 PM America/New_York

## Purpose

Worm is the **deep-dive cross-repo expansion crawler** inside Cortex.

Worm exists to discover and emit **structured cross-repo evidence** that goes beyond the bounded
local-repo truth established by `repo-crawler`.

## Authority boundaries

Worm **is authoritative for**:
- discovery of repo-to-repo references
- discovery of repo-adjacent links and relationships
- controlled expansion from a source repo into governed related targets
- emission of structured relation edges with provenance
- emission of structured cross-repo findings when bounded issue classes match

Worm **is not authoritative for**:
- arbitrary web crawling
- unrestricted network traversal
- semantic truth claims beyond evidence
- patch generation
- patch application
- execution of repair actions
- bypass of Centipede, rak-adatsik, or YellowJacket

## System position

Within the larger system:

- `repo-crawler` = bounded local repo truth
- `Worm` = deep-dive cross-repo expansion truth
- `Cortex` = host and normalize both lanes
- `Centipede` = reconcile weighted evidence across lanes
- `rak-adatsik` = governed operator/control-plane intent path
- `Self-Healing` = later governed remediation stage

## Required operating posture

Worm must be:

- bounded
- deterministic
- provenance-preserving
- fail-closed
- relation-typed
- evidence-first
- non-executing

## Allowed discovery sources

Initial allowed sources may include:

- `.gitmodules`
- git remotes
- workspace manifests
- package manifests
- CI pipeline references
- explicit contract references
- explicit docs/config repo references

## Forbidden behaviors

Worm must not:

- follow arbitrary internet links
- guess repo identity without marking ambiguity
- infer human intent without evidence
- emit narrative-heavy findings without typed classes
- modify source repos directly
- trigger direct execution or bypass review queues

## Required downstream contract discipline

All Worm outputs must carry enough structure for downstream systems to know:

- what relation or finding was emitted
- why it was emitted
- what source artifact produced it
- what target was implicated
- how certain the crawler is
- what evidence source can be re-checked later
