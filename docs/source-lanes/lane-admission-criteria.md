# Cortex Source-Lane Admission Criteria

## Purpose

This document defines the shared admission rule for Cortex source lanes.

## Admission requirements

A source lane may be admitted only if all of the following are true:

- the lane is local-path bounded
- the lane can emit the existing extraction-result contract honestly
- the lane can remain syntax-only
- the lane can fail closed on unreadable, malformed, or excluded inputs
- the lane can report provenance without leaking raw-content diagnostics
- the lane can be validated with bounded fixtures and runtime tests

## Required lane declaration

Each admitted lane must declare:

- exact suffix or media-type admission
- explicit deny conditions
- explicit unavailable conditions
- whether partial-success is allowed
- literal structures that may be emitted
- structures or behaviors that are explicitly excluded

## Non-admission rule

Do not admit a lane if honest contract truth would require:

- OCR
- image interpretation
- semantic labels
- summarization
- workflow hints
- best-effort silent widening
- layout-faithful rendering claims

If trustworthy contract truth cannot be maintained, the lane must remain denied or unavailable.
