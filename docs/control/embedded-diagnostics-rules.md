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

## Visibility rule

Diagnostics must describe operational truth about Cortex-owned surfaces.
They must not become a substitute application for reading local content.
