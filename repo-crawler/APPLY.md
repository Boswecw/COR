# APPLY

```bash
ZIP=~/Downloads/slice_27_worm_pyproject_uv_sources_adapter.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_pyproject_uv_sources_slice
mkdir -p /tmp/worm_pyproject_uv_sources_slice
unzip -o "$ZIP" -d /tmp/worm_pyproject_uv_sources_slice

rsync -av /tmp/worm_pyproject_uv_sources_slice/slice_27_worm_pyproject_uv_sources_adapter/ ./
```
