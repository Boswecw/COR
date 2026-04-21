# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_03_repo_crawler_preflight_toml_unblock.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/repo_crawler_toml_unblock_slice
mkdir -p /tmp/repo_crawler_toml_unblock_slice
unzip -o "$ZIP" -d /tmp/repo_crawler_toml_unblock_slice

rsync -av /tmp/repo_crawler_toml_unblock_slice/slice_03_repo_crawler_preflight_toml_unblock/ ./

python3 scripts/ensure_toml_dependency.py
```
