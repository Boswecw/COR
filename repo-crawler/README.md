# Slice 12 — Worm Reference Audit Bin

Date and Time: 2026-04-22 12:50 AM America/New_York

## Slice boundary

This is the next implementation slice after the Worm contract smoke loader.

It adds a second Rust code path:
- `cargo run --bin worm_reference_audit`

## What this slice does

This slice performs a bounded referential audit across the governed Worm example surfaces:
- loads issue catalogs
- loads evidence bundles
- loads Centipede handoffs
- verifies bundle references in handoffs exist
- verifies finding class / reason code pairs resolve against loaded issue catalogs

## What this slice does not do

- live crawl behavior
- repo graph walking
- live Centipede integration
- mutation of any repo data
