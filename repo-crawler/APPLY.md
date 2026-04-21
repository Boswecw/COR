# APPLY

```bash
ZIP=~/Downloads/slice_28_worm_repo_surface_summary_evidence.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_repo_surface_summary_slice
mkdir -p /tmp/worm_repo_surface_summary_slice
unzip -o "$ZIP" -d /tmp/worm_repo_surface_summary_slice

rsync -av /tmp/worm_repo_surface_summary_slice/slice_28_worm_repo_surface_summary_evidence/ ./
```
