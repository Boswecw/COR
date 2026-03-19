# Cortex Contract - Extraction Result

## Purpose

This contract defines the syntax-level output produced by Cortex extraction.

## Required fields

- `artifact_id`
- `request_id`
- `source_ref`
- `state`
- `syntax_boundary`
- `semantic_boundary_enforced`
- `provenance`
- `extracted_at`

## Required posture

Extraction results must make explicit:

- that the result is syntax-only
- what provenance basis supports the result
- whether completeness is full, incomplete, or failed
- whether refusal occurred because a request crossed into semantics or another denied boundary

## Allowed structures

Phase 1 extraction may include bounded syntax-level structures such as:

- sections
- tables detected counts
- metadata fields
- content blocks

## Disallowed output drift

The extraction result must not claim:

- summary authority
- thematic labeling
- sentiment or implication judgments
- business conclusions

## State rules

- `ready` requires complete extraction posture
- `partial_success` requires explicit incompleteness signaling
- `stale` means an extraction artifact exists but freshness can no longer be asserted
- `denied` and `unavailable` require a refusal object

## Semantic refusal rule

If a caller asks Cortex for semantic interpretation, the extraction result must refuse that request explicitly rather than silently approximating it.
