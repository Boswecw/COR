# Worm Library Wiring Warning Cleanup

Date and Time: 2026-04-22 04:01 AM America/New_York

## Purpose

This slice converts the active Worm bins to consume shared modules via the package library crate.

## Current scope

- adds `src/lib.rs`
- rewires:
  - `worm_cargo_adapter_smoke`
  - `worm_pyproject_adapter_smoke`
  - `worm_run_repo_surface`

## Benefit

- cleaner compile posture
- less warning noise
- stronger internal structure for future slice growth
