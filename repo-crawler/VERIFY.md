# VERIFY

Run these commands exactly after applying the slice.

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

cargo run --bin svelte_probe_self_check
cargo run --bin svelte_probe_cli -- --self-check
cargo run --bin svelte_probe_cli -- tools/svelte-provider/tmp-smoke-test.svelte
```

## Success should look like

- `svelte_probe_self_check` prints pretty JSON with:
  - `"provider": "svelte-probe"`
  - `"version": 1`
  - `"mode": "self-check"`
- `svelte_probe_cli -- --self-check` prints the same contract JSON
- `svelte_probe_cli -- tools/svelte-provider/tmp-smoke-test.svelte` prints the parsed file JSON
- both bins compile, proving the shared module is reusable by multiple call sites
