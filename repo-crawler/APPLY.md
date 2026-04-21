# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_02_svelte_probe_library_extraction.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/svelte_probe_library_slice
mkdir -p /tmp/svelte_probe_library_slice
unzip -o "$ZIP" -d /tmp/svelte_probe_library_slice

rsync -av /tmp/svelte_probe_library_slice/slice_02_svelte_probe_library_extraction/ ./
```
