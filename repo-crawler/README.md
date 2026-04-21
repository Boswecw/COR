# Slice 02 — Svelte Probe Library Extraction

Date: 2026-04-21

## Slice boundary

This slice extracts the Bun/Svelte subprocess logic out of the smoke-test bin and into
a reusable Rust module.

## Included in this slice

- shared Rust module at `src/svelte_probe.rs`
- thin CLI harness at `src/bin/svelte_probe_cli.rs`
- second consumer bin at `src/bin/svelte_probe_self_check.rs` to prove reuse

## Design choice in this slice

This slice does **not** assume an existing crate-level `src/lib.rs` contract, because the current
repo state for that file was not provided here. To avoid clobbering unknown crate wiring, both bins
import the shared module with:

```rust
#[path = "../svelte_probe.rs"]
mod svelte_probe;
```

That still gives you one reusable source module now, without making a blind edit to `src/lib.rs`.

## Not in this slice

- repo-crawler pipeline integration
- crate-level `pub mod svelte_probe;` export via `src/lib.rs`
- fixture suite for multiple failure cases

Those should be the next repo-aware slice after this extraction proves clean.
