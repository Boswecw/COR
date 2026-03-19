# ADR 0002 - Syntax not semantics

## Status

Accepted

## Context

File intelligence often drifts from structure detection into interpretation because the same raw inputs can support both.

## Decision

Cortex owns syntax-level extraction, structure, provenance, completeness, and packaging posture.
It does not own summarization, classification, meaning assignment, or business interpretation.

## Consequences

- extraction contracts must refuse semantic requests
- schemas must keep syntax and semantics separate
- downstream semantic consumers do not retroactively convert Cortex into a semantic authority
