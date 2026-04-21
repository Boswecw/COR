# VERIFY

Run these commands exactly after applying the slice.

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

cargo run --bin worm_contract_smoke
```

## Success should look like

The output should end with something like:

`Validated Worm contract example sets successfully.`
