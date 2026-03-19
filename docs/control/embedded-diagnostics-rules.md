# Embedded Diagnostics Rules

## Purpose

This document names the hard limits on Cortex diagnostics visibility.

## Allowed by default

- counts
- hashes and fingerprints
- bounded metadata
- redacted summaries
- state, reason, and freshness indicators
- watcher status for explicit contract scopes

## Forbidden by default

- raw content preview panes
- full-text search or browse through Cortex diagnostics
- hidden operator backdoors into local content
- surveillance-style observation surfaces

## Redaction rule

If a field could expose meaningful source content, the default posture is redaction unless a higher-order consuming application authority explicitly governs a narrower surface.
In the current Cortex schema-backed layer, embedded diagnostics remain redacted by default and `details_redacted` must be `true`.

## Visibility rule

Diagnostics must describe operational truth about Cortex-owned surfaces.
They must not become a substitute application for reading local content.

## Schema alignment

The machine-checked boundary is:

- `schemas/embedded-diagnostics.schema.json`

Explicitly rejected field shapes include:

- `raw_content_preview`
- `full_text_preview`
- `full_text_search`
- `full_text_browse`
- `content_browser`
- `raw_artifact_dump`
- `unbounded_artifact_dump`
