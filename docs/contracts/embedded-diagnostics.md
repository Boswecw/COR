# Cortex Contract - Embedded Diagnostics

## Purpose

This contract governs what consuming applications may expose about Cortex through embedded diagnostics surfaces.

## Allowed diagnostics content

Applications may expose bounded Cortex diagnostics such as:

- service state
- degraded subtype
- denial summaries
- freshness summaries
- watcher scope counts
- integrity and completeness reason codes
- redacted provenance

## Forbidden diagnostics content

Applications must not expose by default:

- raw file content browsing
- full-text preview panes for convenience
- ad hoc content inspection detached from explicit app authority
- unbounded artifact dumps

## Operator controls

Allowed controls remain bounded and attributable, such as:

- re-run extraction for a bounded source
- clear or invalidate a bounded artifact
- disable an explicitly scoped watcher
- inspect redacted reason codes

## Required posture

Every embedded diagnostics surface must make clear:

- which Cortex surface is being diagnosed
- what scope is affected
- whether details are redacted
- whether the current state is ready, degraded, stale, denied, unavailable, or partial success
