# Slice 27 — Worm Pyproject UV Sources Adapter

Date and Time: 2026-04-22 04:31 AM America/New_York

## Why this slice

I checked the current Cortex repo for `workflow_call`, `action.yml`, and extra workflow surfaces through GitHub search and got no hits.
So the next bounded hardening move is to deepen an already-supported surface: `pyproject.toml`.

This slice adds support for:

- `[tool.uv.sources]`
- git-backed uv source entries inside `pyproject.toml`

## What this slice does

- extends `parse_pyproject_manifest`
- adds `cargo run --bin worm_pyproject_uv_sources_smoke`

## Current posture

- only extracts git-backed uv sources
- no lockfile solving
- no index resolution
- no network calls
