# Slice 23 — Worm Pyproject Manifest Adapter

Date and Time: 2026-04-22 03:49 AM America/New_York

## Slice boundary

This is the next Worm implementation slice.

It adds bounded `pyproject.toml` extraction so Worm can understand Python repo-root dependency references.

## What this slice does

- adds `parse_pyproject_manifest`
- adds `cargo run --bin worm_pyproject_adapter_smoke`
- updates `worm_run_repo_surface` to include `pyproject.toml`

## Current pyproject support

- `[project.dependencies]`
- `[project.optional-dependencies.*]`
- `[tool.poetry.dependencies]`
- `[tool.poetry.group.<name>.dependencies]`

Only git or GitHub-style references are emitted.
