# Worm Reference Audit Bin

Date and Time: 2026-04-22 12:50 AM America/New_York

## Purpose

This slice adds a minimal Rust audit surface that checks cross-file relationships among the governed Worm example documents.

## Audit scope

- issue catalogs
- evidence bundles
- Centipede handoffs

## Current invariants

- every handoff bundle id must resolve to a loaded evidence bundle
- every bundle finding class must exist in a loaded issue catalog
- every bundle finding reason code must be declared for its finding class
