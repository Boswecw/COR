# VERIFY

Run these commands exactly after applying the slice.

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

python3 scripts/ensure_toml_dependency.py
cargo check
cargo run --bin svelte_probe_self_check
cargo run --bin svelte_probe_cli -- --self-check
cargo run --bin svelte_probe_cli -- tools/svelte-provider/tmp-smoke-test.svelte
```

## Success should look like

- the script prints either:
  - `Inserted toml dependency under [dependencies]`
  - or `toml dependency already present`
- `cargo check` no longer fails on unresolved `toml::*`
- existing Svelte probe bins still pass
