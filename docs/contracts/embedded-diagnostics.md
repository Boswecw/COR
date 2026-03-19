# Cortex Contract - Embedded Diagnostics

## Purpose

This contract governs what consuming applications may expose about Cortex through embedded diagnostics surfaces.

Phase 1 treats this as a real contract surface with schema-backed privacy defaults.

## Allowed diagnostics content

Applications may expose bounded Cortex diagnostics such as:

- service state
- degraded subtype
- denial summaries
- freshness summaries
- watcher scope counts
- integrity and completeness reason codes
- redacted provenance

## Required posture

Every embedded diagnostics surface must make clear:

- which Cortex surface is being diagnosed
- what bounded scope is affected
- whether the current state is ready, degraded, stale, denied, unavailable, or partial success
- whether details are redacted

Phase 1 schema-backed diagnostics are redacted by default.
`details_redacted` is required and currently enforced as `true`.

## Forbidden diagnostics content

Applications must not expose by default:

- raw file content browsing
- full-text preview panes for convenience
- ad hoc content inspection detached from explicit app authority
- unbounded artifact dumps

Explicitly rejected examples include:

- `raw_content_preview`
- `full_text_preview`
- `full_text_search`
- `full_text_browse`
- `content_browser`
- `raw_artifact_dump`
- `unbounded_artifact_dump`

## Operator controls

Allowed controls remain bounded and attributable, such as:

- re-run extraction for a bounded source
- clear or invalidate a bounded artifact
- disable an explicitly scoped watcher
- inspect redacted reason codes

## Validation expectations

Validation must prove that:

1. redacted bounded diagnostics shapes pass
2. unredacted diagnostics shapes fail
3. raw-content and preview-style fields fail
4. watcher diagnostics remain contract-scoped only
