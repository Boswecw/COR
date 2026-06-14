#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "doc/system/worm/examples"

ALLOWED_ISSUE_CLASSES = {
    "stale_submodule_pointer",
    "missing_referenced_repo",
    "broken_workspace_member_reference",
    "missing_shared_contract_path",
    "repo_link_drift",
    "ci_points_to_missing_artifact",
    "declared_relationship_target_absent",
    "ambiguous_target_identity",
}
SEVERITY = {"low","medium","high"}
CONFIDENCE = {"high","medium","low","unassessable"}
POSTURE = {"evidence_bound","ambiguous","unresolved","blocked"}

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

def load_catalogs():
    mapping = {}
    files = sorted(EXAMPLES.glob("issue_catalog_*.json"))
    if not files:
        fail(f"No issue catalog files found in {EXAMPLES}")
    for path in files:
        data = json.loads(path.read_text())
        if data.get("kind") != "worm_reason_code_catalog" or data.get("schemaVersion") != 1:
            fail(f"{path.name}: invalid catalog header")
        issue_class = require_nonempty_str(data, "issueClass", path.name)
        if issue_class not in ALLOWED_ISSUE_CLASSES:
            fail(f"{path.name}: invalid issueClass '{issue_class}'")
        codes = require(data, "reasonCodes", path.name)
        if not isinstance(codes, list) or not codes:
            fail(f"{path.name}: reasonCodes must be a non-empty array")
        catalog_codes = set()
        for idx, item in enumerate(codes):
            code = require_nonempty_str(item, "code", f"{path.name}.reasonCodes[{idx}]")
            severity = require_nonempty_str(item, "severity", f"{path.name}.reasonCodes[{idx}]")
            if severity not in SEVERITY:
                fail(f"{path.name}.reasonCodes[{idx}]: invalid severity '{severity}'")
            require_nonempty_str(item, "description", f"{path.name}.reasonCodes[{idx}]")
            catalog_codes.add(code)
        mapping[issue_class] = catalog_codes
        print(f"OK  {path.name}")
    return mapping

def validate_findings(catalogs):
    files = sorted(EXAMPLES.glob("finding_catalog_*.json"))
    if not files:
        fail(f"No finding catalog files found in {EXAMPLES}")
    count = 0
    for path in files:
        data = json.loads(path.read_text())
        if data.get("kind") != "worm_finding" or data.get("schemaVersion") != 1:
            fail(f"{path.name}: invalid finding header")
        finding_class = require_nonempty_str(data, "findingClass", path.name)
        if finding_class not in catalogs:
            fail(f"{path.name}: findingClass '{finding_class}' has no loaded catalog")
        reason_code = require_nonempty_str(data, "reasonCode", path.name)
        if reason_code not in catalogs[finding_class]:
            fail(f"{path.name}: reasonCode '{reason_code}' is not declared for findingClass '{finding_class}'")
        require_nonempty_str(data, "findingId", path.name)
        require_nonempty_str(data, "sourceRepo", path.name)
        src = require(data, "sourceArtifact", path.name)
        require_nonempty_str(src, "path", f"{path.name}.sourceArtifact")
        target = require(data, "target", path.name)
        require_nonempty_str(target, "rawReference", f"{path.name}.target")
        edge_ids = require(data, "relatedEdgeIds", path.name)
        if not isinstance(edge_ids, list) or not edge_ids or not all(isinstance(x, str) and x.strip() for x in edge_ids):
            fail(f"{path.name}: relatedEdgeIds must be a non-empty array of strings")
        confidence = require_nonempty_str(data, "confidence", path.name)
        if confidence not in CONFIDENCE:
            fail(f"{path.name}: invalid confidence '{confidence}'")
        posture = require_nonempty_str(data, "posture", path.name)
        if posture not in POSTURE:
            fail(f"{path.name}: invalid posture '{posture}'")
        require_nonempty_str(data, "crawlScope", path.name)
        require_nonempty_str(data, "timestamp", path.name)
        print(f"OK  {path.name}")
        count += 1
    return count

def main():
    catalogs = load_catalogs()
    findings_count = validate_findings(catalogs)
    total = len(catalogs) + findings_count
    print(f"Validated {total} Worm issue catalog files successfully.")

if __name__ == "__main__":
    main()
