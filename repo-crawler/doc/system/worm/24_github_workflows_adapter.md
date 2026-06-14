# Worm GitHub Workflows Adapter

Date and Time: 2026-04-22 04:18 AM America/New_York

## Purpose

Extend Worm cross-repo discovery into CI wiring surfaces.

## Included scope

- `.github/workflows/*.yml`
- `.github/workflows/*.yaml`

## Current posture

- bounded text extraction
- only `uses:` repo references
- no action metadata fetch
- no network calls
- local and docker actions skipped
