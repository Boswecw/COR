#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "doc/system/worm/examples"

POSTURE = {"resolved","ambiguous","unresolved"}
METHODS = {"ssh_remote_parse","https_remote_parse","relative_path_inference","owner_repo_direct","manual_mapping","none"}

def fail(msg: str) -> None:
    print(msg, file=sys.stderr)
    raise SystemExit(1)

def require(obj: dict, key: str, label: str):
    if key not in obj:
        fail(f"{label}: missing required key '{key}'")
    return obj[key]

def require_nonempty_str(obj: dict, key: str, label: str):
    value = require(obj, key, label)
    if not isinstance(value, str) or not value.strip():
        fail(f"{label}: key '{key}' must be a non-empty string")
    return value

def validate_identity(identity: dict, label: str):
    require_nonempty_str(identity, "host", label)
    require_nonempty_str(identity, "owner", label)
    require_nonempty_str(identity, "repo", label)
    require_nonempty_str(identity, "display", label)

def validate_resolution(data: dict, label: str):
    if data.get("kind") != "worm_target_resolution":
        fail(f"{label}: kind must be 'worm_target_resolution'")
    if data.get("schemaVersion") != 1:
        fail(f"{label}: schemaVersion must be 1")

    require_nonempty_str(data, "resolutionId", label)
    require_nonempty_str(data, "rawReference", label)

    posture = require_nonempty_str(data, "resolutionPosture", label)
    if posture not in POSTURE:
        fail(f"{label}: invalid resolutionPosture '{posture}'")

    method = require_nonempty_str(data, "resolutionMethod", label)
    if method not in METHODS:
        fail(f"{label}: invalid resolutionMethod '{method}'")

    evidence = data.get("evidence", [])
    if not isinstance(evidence, list) or not all(isinstance(x, str) and x.strip() for x in evidence):
        fail(f"{label}: evidence must be an array of non-empty strings")

    if posture == "resolved":
        identity = require(data, "canonicalIdentity", label)
        validate_identity(identity, f"{label}.canonicalIdentity")
        if method == "none":
            fail(f"{label}: resolved posture cannot use resolutionMethod 'none'")
    else:
        if "canonicalIdentity" in data:
            fail(f"{label}: non-resolved posture must not include canonicalIdentity")

    require_nonempty_str(data, "timestamp", label)

def main():
    files = sorted(EXAMPLES.glob("target_resolution_*.json"))
    if not files:
        fail(f"No target resolution examples found in {EXAMPLES}")

    count = 0
    for path in files:
        data = json.loads(path.read_text())
        validate_resolution(data, path.name)
        print(f"OK  {path.name}")
        count += 1

    print(f"Validated {count} Worm target resolution files successfully.")

if __name__ == "__main__":
    main()
