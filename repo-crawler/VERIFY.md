# VERIFY

Run these commands exactly after applying the slice.

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

python3 scripts/validate_worm_issue_catalog.py
```

## Success should look like

- each issue catalog file is reported as `OK`
- each finding catalog file is reported as `OK`
- the script ends with:
  - `Validated 4 Worm issue catalog files successfully.`
