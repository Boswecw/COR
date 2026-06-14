#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "doc/system/worm/examples"

POSTURE = {"evidence_bound","ambiguous","unresolved","blocked"}
ADAPTERS = {"gitmodules_parse","package_manifest_parse"}
EDGE_TYPES = {
    "git_submodule","git_remote_reference","workspace_member","dependency_repo_reference",
    "ci_pipeline_reference","docs_repo_reference","shared_contract_reference","unknown_external_reference"
}
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

def validate_edge(edge: dict, label: str, expected_adapter: str):
    if edge.get("kind") != "worm_edge":
        fail(f"{label}: edge kind must be 'worm_edge'")
    if edge.get("schemaVersion") != 1:
        fail(f"{label}: edge schemaVersion must be 1")
    require_nonempty_str(edge, "edgeId", label)
    relation = require_nonempty_str(edge, "relationType", label)
    if relation not in EDGE_TYPES:
        fail(f"{label}: invalid relationType '{relation}'")
    method = require_nonempty_str(edge, "discoveryMethod", label)
    if method != expected_adapter:
        fail(f"{label}: discoveryMethod '{method}' does not match adapter '{expected_adapter}'")
    require_nonempty_str(edge, "sourceRepo", label)
    src = require(edge, "sourceArtifact", label)
    require_nonempty_str(src, "path", f"{label}.sourceArtifact")
    tgt = require(edge, "target", label)
    require_nonempty_str(tgt, "rawReference", f"{label}.target")
    confidence = require_nonempty_str(edge, "confidence", label)
    if confidence not in CONFIDENCE:
        fail(f"{label}: invalid confidence '{confidence}'")
    posture = require_nonempty_str(edge, "posture", label)
    if posture not in POSTURE:
        fail(f"{label}: invalid posture '{posture}'")
    require_nonempty_str(edge, "crawlScope", label)
    require_nonempty_str(edge, "timestamp", label)

def validate_adapter(data: dict, label: str):
    if data.get("kind") != "worm_adapter_emission":
        fail(f"{label}: kind must be 'worm_adapter_emission'")
    if data.get("schemaVersion") != 1:
        fail(f"{label}: schemaVersion must be 1")

    adapter_name = require_nonempty_str(data, "adapterName", label)
    if adapter_name not in ADAPTERS:
        fail(f"{label}: invalid adapterName '{adapter_name}'")

    require_nonempty_str(data, "sourceRepo", label)
    src = require(data, "sourceArtifact", label)
    require_nonempty_str(src, "path", f"{label}.sourceArtifact")

    emitted = require(data, "emittedEdges", label)
    if not isinstance(emitted, list) or not emitted:
        fail(f"{label}: emittedEdges must be a non-empty array")
    for idx, edge in enumerate(emitted):
        validate_edge(edge, f"{label}.emittedEdges[{idx}]", adapter_name)

    skipped = require(data, "skippedReferences", label)
    if not isinstance(skipped, list):
        fail(f"{label}: skippedReferences must be an array")
    for idx, item in enumerate(skipped):
        require_nonempty_str(item, "rawReference", f"{label}.skippedReferences[{idx}]")
        require_nonempty_str(item, "reasonCode", f"{label}.skippedReferences[{idx}]")

    posture = require_nonempty_str(data, "posture", label)
    if posture not in POSTURE:
        fail(f"{label}: invalid posture '{posture}'")
    require_nonempty_str(data, "timestamp", label)

def main():
    files = sorted(EXAMPLES.glob("adapter_emit_*.json"))
    if not files:
        fail(f"No adapter emission examples found in {EXAMPLES}")

    count = 0
    for path in files:
        data = json.loads(path.read_text())
        validate_adapter(data, path.name)
        print(f"OK  {path.name}")
        count += 1

    print(f"Validated {count} Worm discovery adapter emission files successfully.")

if __name__ == "__main__":
    main()
