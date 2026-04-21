# VERIFY

Run these commands exactly after applying the slice.

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

python3 scripts/validate_worm_discovery_adapters.py
```

## Success should look like

- each adapter emission file is reported as `OK`
- the script ends with:
  - `Validated 2 Worm discovery adapter emission files successfully.`
