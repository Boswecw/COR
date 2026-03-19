# ADR 0001 - Service-only framing

## Status

Accepted

## Context

Cortex can easily drift into a pseudo-product because it handles content, diagnostics, and operational state that users may want to inspect directly.

## Decision

Cortex is a service-only subsystem.
It may be visible only through consuming application surfaces and bounded operator-facing controls.

## Consequences

- Cortex does not get a standalone product identity by default.
- Control surfaces must remain embedded and bounded.
- Architecture and roadmap work must treat Cortex as runtime infrastructure, not a parallel app.
