# Slice 11 — Worm Contract Loader Scaffold

Date and Time: 2026-04-22 12:35 AM America/New_York

## Slice boundary

This is the **WRM-09** slice.

This is the first code-oriented Worm slice.
It adds a minimal Rust loader/smoke path that reads the governed Worm JSON example surfaces already added in prior slices.

## What this slice does

- adds a small shared Rust helper for Worm contract example loading
- adds a smoke binary:
  - `cargo run --bin worm_contract_smoke`
- validates presence and basic shape of Worm example files

## Why this slice comes now

The contract set is now broad enough that Worm needs a minimal code foothold.
This slice stays low-risk:
- no live crawl behavior
- no traversal logic
- no Centipede integration
- no mutation of existing repo-crawler flow

## Included

- `src/worm_contracts.rs`
- `src/bin/worm_contract_smoke.rs`
- `doc/system/worm/09_contract_loader_scaffold.md`
