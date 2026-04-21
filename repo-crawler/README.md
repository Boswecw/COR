# Slice 28 — Worm Repo Surface Summary Evidence

Date and Time: 2026-04-22 04:40 AM America/New_York

## Why this slice

Your last run showed why total edge count is a weak proof surface.
The UV parser smoke passed, but the total repo-surface edge count changed because the temp `pyproject.toml` itself was replaced.

That means we need per-surface attribution, not just total counts.

## What this slice does

- updates `worm_run_repo_surface`
- writes `surface_summary.json`
- adds `cargo run --bin worm_repo_surface_summary_smoke`

## Summary evidence produced

- `adapterEdgeCounts`
- `sourceArtifactEdgeCounts`
- `edgesBeforeResolution`
- `resolutions`

## Why this matters

This gives Centipede and later self-healing wiring a stable evidence surface for:
- what Worm actually inspected
- which adapter emitted which edges
- whether a total-count swing came from parser behavior or fixture drift
