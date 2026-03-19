# Cortex Shared Source-Lane Failure Taxonomy

## Purpose

This document defines the shared failure posture for Cortex source lanes.

## Shared states

- `ready`: the admitted lane recovered bounded syntax-only structure honestly
- `partial_success`: the lane recovered only the contract-defined subset of structure and the incompleteness is explicit
- `denied`: the source is outside the admitted lane or violates a declared exclusion
- `unavailable`: the source or required bounded dependency could not be read safely enough to trust the lane

## Reason-class posture

Shared reason classes should remain narrow:

- `unsupported_source_type`
- `ineligible_source`
- `dependency_unavailable`

Lane-specific logic may choose among these, but it must not invent workflow or recommendation semantics.

## Examples

- encrypted PDF: `denied`
- scanned PDF without text layer: `denied`
- corrupt PDF or corrupt DOCX package: `unavailable`
- DOCX with comments or tracked changes: `denied`
- oversized literal content beyond bounded limits: `denied`

## Non-goals

The failure surface is not allowed to become:

- retry negotiation
- operator instruction
- workflow sequencing
- parser repair heuristics disguised as success
