# Worm Run Repo Surface CLI

Date and Time: 2026-04-22 03:18 AM America/New_York

## Purpose

This slice introduces bounded repo-root surface discovery for Worm.

## Arguments

`cargo run --bin worm_run_repo_surface -- <source_repo> <repo_root> <out_dir>`

## Current discovery scope

- `<repo_root>/.gitmodules`
- `<repo_root>/package.json`

## Doctrine

- bounded root-only discovery
- no recursion
- fail closed when files are absent
- deterministic output writing
