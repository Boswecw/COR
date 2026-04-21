# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_11_worm_contract_loader_scaffold.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_contract_loader_slice
mkdir -p /tmp/worm_contract_loader_slice
unzip -o "$ZIP" -d /tmp/worm_contract_loader_slice

rsync -av /tmp/worm_contract_loader_slice/slice_11_worm_contract_loader_scaffold/ ./
```
