# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_08_worm_issue_classes_catalog.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_issue_catalog_slice
mkdir -p /tmp/worm_issue_catalog_slice
unzip -o "$ZIP" -d /tmp/worm_issue_catalog_slice

rsync -av /tmp/worm_issue_catalog_slice/slice_08_worm_issue_classes_catalog/ ./
```
