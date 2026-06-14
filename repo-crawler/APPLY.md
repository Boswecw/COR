# APPLY

```bash
ZIP=~/Downloads/slice_29_worm_nested_requirements_follow.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_nested_requirements_slice
mkdir -p /tmp/worm_nested_requirements_slice
unzip -o "$ZIP" -d /tmp/worm_nested_requirements_slice

rsync -av /tmp/worm_nested_requirements_slice/slice_29_worm_nested_requirements_follow/ ./
```
