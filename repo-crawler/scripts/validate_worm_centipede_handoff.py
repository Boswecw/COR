#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "doc/system/worm/examples"

WEIGHT = {"supporting","strong","blocking"}
CONFIDENCE = {"high","medium","low","unassessable"}

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

def validate_handoff(data: dict, label: str):
    if data.get("kind") != "worm_centipede_handoff":
        fail(f"{label}: kind must be 'worm_centipede_handoff'")
    if data.get("schemaVersion") != 1:
        fail(f"{label}: schemaVersion must be 1")
    require_nonempty_str(data, "handoffId", label)
    if data.get("sourceLane") != "worm":
        fail(f"{label}: sourceLane must be 'worm'")
    require_nonempty_str(data, "sourceRepo", label)
    require_nonempty_str(data, "runId", label)

    bundle_ids = require(data, "bundleIds", label)
    if not isinstance(bundle_ids, list) or not bundle_ids or not all(isinstance(x, str) and x.strip() for x in bundle_ids):
        fail(f"{label}: bundleIds must be a non-empty array of strings")

    issues = require(data, "candidateIssueKeys", label)
    if not isinstance(issues, list) or not issues:
        fail(f"{label}: candidateIssueKeys must be a non-empty array")
    for idx, item in enumerate(issues):
        require_nonempty_str(item, "issueKey", f"{label}.candidateIssueKeys[{idx}]")
        require_nonempty_str(item, "findingClass", f"{label}.candidateIssueKeys[{idx}]")
        weight = require_nonempty_str(item, "proposedWeightClass", f"{label}.candidateIssueKeys[{idx}]")
        if weight not in WEIGHT:
            fail(f"{label}.candidateIssueKeys[{idx}]: invalid proposedWeightClass '{weight}'")
        confidence = require_nonempty_str(item, "confidence", f"{label}.candidateIssueKeys[{idx}]")
        if confidence not in CONFIDENCE:
            fail(f"{label}.candidateIssueKeys[{idx}]: invalid confidence '{confidence}'")

    require_nonempty_str(data, "timestamp", label)

def main():
    files = sorted(EXAMPLES.glob("centipede_handoff_*.json"))
    if not files:
        fail(f"No Centipede handoff examples found in {EXAMPLES}")

    count = 0
    for path in files:
        data = json.loads(path.read_text())
        validate_handoff(data, path.name)
        print(f"OK  {path.name}")
        count += 1

    print(f"Validated {count} Worm-to-Centipede handoff files successfully.")

if __name__ == "__main__":
    main()
