# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_23_worm_pyproject_manifest_adapter.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_pyproject_manifest_slice
mkdir -p /tmp/worm_pyproject_manifest_slice
unzip -o "$ZIP" -d /tmp/worm_pyproject_manifest_slice

rsync -av /tmp/worm_pyproject_manifest_slice/slice_23_worm_pyproject_manifest_adapter/ ./
```
