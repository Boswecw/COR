# VERIFY

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

cargo run --bin worm_pyproject_uv_sources_smoke

cat > /tmp/worm-repo-surface-pyproject/pyproject.toml <<'EOF'
[project]
name = "repo-surface-test"
version = "0.1.0"
dependencies = [
  "fastapi>=0.110.0",
  "sharedpkg @ git+https://github.com/Boswecw/sharedpkg.git"
]

[tool.uv.sources]
sharedlib = { git = "https://github.com/Boswecw/sharedlib.git" }
mytool = { git = "ssh://git@github.com/Boswecw/mytool.git" }
EOF

rm -rf /tmp/worm-repo-surface-pyproject-out
cargo run --bin worm_run_repo_surface -- Boswecw/Cortex /tmp/worm-repo-surface-pyproject /tmp/worm-repo-surface-pyproject-out

python3 - <<'PY'
from pathlib import Path
import json

root = Path("/tmp/worm-repo-surface-pyproject-out")
bundle = json.loads((root / "bundle.json").read_text())
handoff = json.loads((root / "handoff.json").read_text())

print("bundleId:", bundle["bundleId"])
print("handoffId:", handoff["handoffId"])
print("edges:", len(bundle["edges"]))
print("resolutions:", len(bundle["resolutions"]))
print("findings:", len(bundle["findings"]))
print("candidateIssueKeys:", len(handoff["candidateIssueKeys"]))
PY
```

## Expected result

The repo-surface edge count should increase by 2 compared with the previous run on the same temp repo.
