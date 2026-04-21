# Slice 29 — Worm Nested Requirements Follow

Date and Time: 2026-04-22 04:47 AM America/New_York

## Why this slice

The new summary output showed that requirements parsing is active and measurable.

The next high-value Python hardening step is to follow nested requirements references in the runner instead of only skipping them inside each single-file parse.

## What this slice does

- updates `worm_run_repo_surface`
- follows:
  - `-r other-file.txt`
  - `--requirement other-file.txt`
- attributes nested files in `surface_summary.json`
- adds `cargo run --bin worm_nested_requirements_smoke`

## Current posture

- bounded local file follow only
- relative-path include resolution only
- no network fetch
- no pip constraint solving
