# Worm Cargo Manifest Adapter

Date and Time: 2026-04-22 03:35 AM America/New_York

## Purpose

This slice expands Worm repo-root surface coverage to include Rust Cargo manifests.

## Supported tables

- `[dependencies]`
- `[dev-dependencies]`
- `[build-dependencies]`
- `[workspace.dependencies]`

## Supported reference shape

- dependency table entries with `git = "..."`

## Posture

- bounded repo-root extraction
- no crate registry interpretation
- no recursion
- no lockfile analysis yet
