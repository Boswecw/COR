# APPLY

Run these commands exactly.

```bash
ZIP=~/Downloads/slice_10_worm_centipede_handoff_contract.zip

cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1
rm -rf /tmp/worm_centipede_handoff_slice
mkdir -p /tmp/worm_centipede_handoff_slice
unzip -o "$ZIP" -d /tmp/worm_centipede_handoff_slice

rsync -av /tmp/worm_centipede_handoff_slice/slice_10_worm_centipede_handoff_contract/ ./
```
