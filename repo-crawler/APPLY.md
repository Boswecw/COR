# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_12_worm_reference_audit_bin.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_reference_audit_slice
mkdir -p /tmp/worm_reference_audit_slice
unzip -o "$ZIP" -d /tmp/worm_reference_audit_slice

rsync -av /tmp/worm_reference_audit_slice/slice_12_worm_reference_audit_bin/ ./
```
