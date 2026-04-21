# VERIFY

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

cargo run --bin worm_repo_surface_summary_smoke

rm -rf /tmp/worm-repo-surface-pyproject-out
cargo run --bin worm_run_repo_surface -- Boswecw/Cortex /tmp/worm-repo-surface-pyproject /tmp/worm-repo-surface-pyproject-out

python3 - <<'PY'
from pathlib import Path
import json

root = Path("/tmp/worm-repo-surface-pyproject-out")
summary = json.loads((root / "surface_summary.json").read_text())

print("kind:", summary["kind"])
print("filesProcessed:", len(summary["filesProcessed"]))
print("adapterEdgeCounts:", summary["adapterEdgeCounts"])
print("sourceArtifactEdgeCounts:", summary["sourceArtifactEdgeCounts"])
print("edgesBeforeResolution:", summary["totals"]["edgesBeforeResolution"])
print("resolutions:", summary["totals"]["resolutions"])
PY
```
