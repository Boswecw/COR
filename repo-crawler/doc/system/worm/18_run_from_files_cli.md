# Worm Run from Files CLI

Date and Time: 2026-04-22 03:05 AM America/New_York

## Purpose

This slice adds the first bounded file-input execution path for Worm.

## Arguments

`cargo run --bin worm_run_from_files -- <source_repo> <gitmodules_path_or_dash> <package_json_path_or_dash> <out_dir>`

Use `-` to skip an input.

## Posture

- operator-supplied inputs only
- no repo walk
- no hidden discovery
- deterministic output writing
