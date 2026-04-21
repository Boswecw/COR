# Slice 03 — Repo-Crawler Preflight TOML Unblock

Date: 2026-04-21

## Why this slice exists

Before I can safely package the actual pipeline-wiring tranche, the repo's library crate has to
compile cleanly again.

The proof logs show a real blocker in the current repo state:

- `src/error.rs` references `toml::de::Error` and `toml::ser::Error`
- `src/config.rs` uses `toml::from_str(...)` and `toml::to_string_pretty(...)`
- `src/parser.rs` parses `toml::Value`
- but Cargo resolves `toml` as missing in the active crate dependency graph

That means the next pipeline-wiring slice would be built on a broken preflight baseline.

## What this slice does

- adds a robust helper script that ensures `toml = "0.8.23"` exists under `[dependencies]`
- avoids assuming single-line formatting in `Cargo.toml`
- gives you exact apply and verify commands

## What this slice does not do

- it does not yet wire Svelte probing into the repo-crawler library pipeline
- it does not edit your unknown live `src/lib.rs` or parser pipeline blindly

Once this preflight blocker is cleared and `cargo check` is green, the next slice can target
actual pipeline wiring with much lower risk.
