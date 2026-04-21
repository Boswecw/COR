# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_06_worm_discovery_adapters_contract.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_discovery_adapters_slice
mkdir -p /tmp/worm_discovery_adapters_slice
unzip -o "$ZIP" -d /tmp/worm_discovery_adapters_slice

rsync -av /tmp/worm_discovery_adapters_slice/slice_06_worm_discovery_adapters_contract/ ./
```
