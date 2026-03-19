# Cortex Control Surface

## Purpose

This document defines how Cortex may be seen and controlled through consuming applications.

## Allowed surface classes

Consuming applications may expose bounded Cortex surfaces such as:

- readiness and degraded-state indicators
- freshness and stale-state indicators
- intake and extraction counts
- denial and reason-code summaries
- bounded re-run or re-prep actions
- watcher status for explicitly enabled scopes
- privacy-preserving diagnostics panels

## Allowed operator controls

Allowed controls must remain narrow and attributable.
Examples:

- enable or disable a contract-scoped watcher
- request re-extraction for a bounded source
- clear or invalidate a bounded retrieval artifact
- acknowledge a stale or degraded condition
- inspect bounded integrity or completeness reason codes

## Forbidden surface drift

The following are forbidden by default:

- standalone Cortex application identity
- free-form file browsing through Cortex diagnostics
- raw content preview panes
- open-ended workflow coordination surfaces
- controls that imply app-level truth authority
- controls that imply execution-host authority

## Visibility rule

Every meaningful Cortex control surface must show:

- current service state
- scope of the action
- degradation or denial reason when relevant
- whether the surface is exposing watcher status, freshness state, or handoff state

## Diagnostics rule

Diagnostics are allowed only when they remain privacy-preserving and tied to Cortex-owned surfaces.

Preferred diagnostics include:

- counts
- timestamps
- bounded file-class identifiers
- hashes or fingerprints
- reason codes
- redacted provenance

## Default operator question

Does this surface help an application understand or govern Cortex within bounded authority, or does it make Cortex look like a hidden product or control plane?
