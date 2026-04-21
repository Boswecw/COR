#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "doc/system/worm/examples"

EDGE_TYPES = {
    "git_submodule",
    "git_remote_reference",
    "workspace_member",
    "dependency_repo_reference",
    "ci_pipeline_reference",
    "docs_repo_reference",
    "shared_contract_reference",
    "unknown_external_reference",
}

FINDING_CLASSES = {
    "stale_submodule_pointer",
    "missing_referenced_repo",
    "broken_workspace_member_reference",
    "missing_shared_contract_path",
    "repo_link_drift",
    "ci_points_to_missing_artifact",
    "declared_relationship_target_absent",
    "ambiguous_target_identity",
}

CONFIDENCE = {"high", "medium", "low", "unassessable"}
POSTURE = {"evidence_bound", "ambiguous", "unresolved", "blocked"}
SCOPE = {"local_repo", "cross_repo", "external_reference"}
IDENTITY_POSTURE = {"resolved", "ambiguous", "unresolved"}

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

def validate_source_artifact(obj: dict, label: str):
    path = require_nonempty_str(obj, "path", label)
    line = obj.get("line")
    if line is not None and (not isinstance(line, int) or line < 1):
        fail(f"{label}: 'line' must be an integer >= 1 when present")
    sha = obj.get("artifactSha256")
    if sha is not None and (not isinstance(sha, str) or not sha.strip()):
        fail(f"{label}: 'artifactSha256' must be a non-empty string when present")
    return path

def validate_target(obj: dict, label: str):
    require_nonempty_str(obj, "rawReference", label)
    identity_posture = obj.get("identityPosture")
    if identity_posture is not None and identity_posture not in IDENTITY_POSTURE:
        fail(f"{label}: invalid identityPosture '{identity_posture}'")

def validate_common(data: dict, label: str):
    schema_version = require(data, "schemaVersion", label)
    if schema_version != 1:
        fail(f"{label}: schemaVersion must be 1")
    require_nonempty_str(data, "sourceRepo", label)
    validate_source_artifact(require(data, "sourceArtifact", label), f"{label}.sourceArtifact")
    validate_target(require(data, "target", label), f"{label}.target")
    confidence = require_nonempty_str(data, "confidence", label)
    if confidence not in CONFIDENCE:
        fail(f"{label}: invalid confidence '{confidence}'")
    posture = require_nonempty_str(data, "posture", label)
    if posture not in POSTURE:
        fail(f"{label}: invalid posture '{posture}'")
    scope = require_nonempty_str(data, "crawlScope", label)
    if scope not in SCOPE:
        fail(f"{label}: invalid crawlScope '{scope}'")
    require_nonempty_str(data, "timestamp", label)

def validate_edge(data: dict, label: str):
    if data.get("kind") != "worm_edge":
        fail(f"{label}: kind must be 'worm_edge'")
    require_nonempty_str(data, "edgeId", label)
    relation_type = require_nonempty_str(data, "relationType", label)
    if relation_type not in EDGE_TYPES:
        fail(f"{label}: invalid relationType '{relation_type}'")
    require_nonempty_str(data, "discoveryMethod", label)
    validate_common(data, label)

def validate_finding(data: dict, label: str):
    if data.get("kind") != "worm_finding":
        fail(f"{label}: kind must be 'worm_finding'")
    require_nonempty_str(data, "findingId", label)
    finding_class = require_nonempty_str(data, "findingClass", label)
    if finding_class not in FINDING_CLASSES:
        fail(f"{label}: invalid findingClass '{finding_class}'")
    reason_code = require_nonempty_str(data, "reasonCode", label)
    edge_ids = require(data, "relatedEdgeIds", label)
    if not isinstance(edge_ids, list) or not edge_ids or not all(isinstance(x, str) and x.strip() for x in edge_ids):
        fail(f"{label}: relatedEdgeIds must be a non-empty array of strings")
    validate_common(data, label)

def main():
    if not EXAMPLES.exists():
        fail(f"Examples directory not found: {EXAMPLES}")

    files = sorted(EXAMPLES.glob("*.json"))
    if not files:
        fail("No example JSON files found")

    count = 0
    for path in files:
        data = json.loads(path.read_text())
        label = path.name
        kind = data.get("kind")
        if kind == "worm_edge":
            validate_edge(data, label)
        elif kind == "worm_finding":
            validate_finding(data, label)
        else:
            fail(f"{label}: unknown kind '{kind}'")
        print(f"OK  {label}")
        count += 1

    print(f"Validated {count} example files successfully.")

if __name__ == "__main__":
    main()
