# VERIFY

Run these commands exactly after applying the slice.

```bash
cd ~/Forge/ecosystem/local-systems/cortex/repo-crawler || exit 1

cargo run --bin worm_pyproject_adapter_smoke

mkdir -p /tmp/worm-repo-surface-pyproject
cat > /tmp/worm-repo-surface-pyproject/.gitmodules <<'EOF'
[submodule "linked-repo"]
    path = linked-repo
    url = ../linked-repo
EOF

cat > /tmp/worm-repo-surface-pyproject/package.json <<'EOF'
{
  "dependencies": {
    "shared-lib": "git+ssh://git@github.com/Boswecw/shared-lib.git",
    "forge-contract-core": "Boswecw/forge-contract-core",
    "lodash": "^4.17.21"
  }
}
EOF

cat > /tmp/worm-repo-surface-pyproject/Cargo.toml <<'EOF'
[package]
name = "demo"
version = "0.1.0"

[dependencies]
forge-contract-core = { git = "https://github.com/Boswecw/forge-contract-core.git" }
serde = "1"

[workspace.dependencies]
dataforge = { git = "https://github.com/Boswecw/DataForge.git" }
EOF

cat > /tmp/worm-repo-surface-pyproject/pyproject.toml <<'EOF'
[project]
dependencies = [
  "fastapi>=0.110.0",
  "my-lib @ git+https://github.com/Boswecw/my-lib.git"
]

[project.optional-dependencies]
dev = [
  "another-lib @ git+ssh://git@github.com/Boswecw/another-lib.git"
]

[tool.poetry.dependencies]
python = "^3.12"
sharedkit = { git = "https://github.com/Boswecw/sharedkit.git" }

[tool.poetry.group.dev.dependencies]
tooling = { git = "https://github.com/Boswecw/tooling.git" }
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
