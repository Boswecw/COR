# VERIFY

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

cargo run --bin worm_nested_requirements_smoke

mkdir -p /tmp/worm-repo-surface-pyproject/requirements
cat > /tmp/worm-repo-surface-pyproject/requirements.txt <<'EOF'
-r requirements/dev.txt
fastapi==0.115.0
EOF

cat > /tmp/worm-repo-surface-pyproject/requirements/dev.txt <<'EOF'
git+https://github.com/Boswecw/test-tooling.git
EOF

rm -rf /tmp/worm-repo-surface-pyproject-out
cargo run --bin worm_run_repo_surface -- Boswecw/Cortex /tmp/worm-repo-surface-pyproject /tmp/worm-repo-surface-pyproject-out

python3 - <<'PY'
from pathlib import Path
import json

root = Path("/tmp/worm-repo-surface-pyproject-out")
summary = json.loads((root / "surface_summary.json").read_text())

print("filesProcessed:", summary["filesProcessed"])
print("adapterEdgeCounts:", summary["adapterEdgeCounts"])
print("sourceArtifactEdgeCounts:", summary["sourceArtifactEdgeCounts"])
print("edgesBeforeResolution:", summary["totals"]["edgesBeforeResolution"])
print("resolutions:", summary["totals"]["resolutions"])
PY
```

## Expected result

`requirements/dev.txt` should appear in `filesProcessed` and `sourceArtifactEdgeCounts`.
