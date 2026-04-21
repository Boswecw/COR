#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "doc/system/worm/examples"

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

def validate_bundle(data: dict, label: str):
    if data.get("kind") != "worm_evidence_bundle":
        fail(f"{label}: kind must be 'worm_evidence_bundle'")
    if data.get("schemaVersion") != 1:
        fail(f"{label}: schemaVersion must be 1")
    require_nonempty_str(data, "bundleId", label)
    require_nonempty_str(data, "runId", label)
    if data.get("lane") != "worm":
        fail(f"{label}: lane must be 'worm'")
    require_nonempty_str(data, "sourceRepo", label)
    source_artifacts = require(data, "sourceArtifacts", label)
    if not isinstance(source_artifacts, list) or not source_artifacts:
        fail(f"{label}: sourceArtifacts must be a non-empty array")
    for idx, item in enumerate(source_artifacts):
        require_nonempty_str(item, "path", f"{label}.sourceArtifacts[{idx}]")
        adapter = item.get("adapterName")
        if adapter is not None and (not isinstance(adapter, str) or not adapter.strip()):
            fail(f"{label}.sourceArtifacts[{idx}]: adapterName must be a non-empty string when present")

    edges = require(data, "edges", label)
    findings = require(data, "findings", label)
    resolutions = require(data, "targetResolutions", label)

    if not isinstance(edges, list) or not edges:
        fail(f"{label}: edges must be a non-empty array")
    if not isinstance(findings, list) or not findings:
        fail(f"{label}: findings must be a non-empty array")
    if not isinstance(resolutions, list) or not resolutions:
        fail(f"{label}: targetResolutions must be a non-empty array")

    edge_ids = set()
    resolution_ids = set()

    for idx, edge in enumerate(edges):
        if edge.get("kind") != "worm_edge" or edge.get("schemaVersion") != 1:
            fail(f"{label}.edges[{idx}]: invalid edge header")
        edge_id = require_nonempty_str(edge, "edgeId", f"{label}.edges[{idx}]")
        edge_ids.add(edge_id)
        src = require(edge, "sourceArtifact", f"{label}.edges[{idx}]")
        require_nonempty_str(src, "path", f"{label}.edges[{idx}].sourceArtifact")
        require_nonempty_str(edge, "timestamp", f"{label}.edges[{idx}]")

    for idx, res in enumerate(resolutions):
        if res.get("kind") != "worm_target_resolution" or res.get("schemaVersion") != 1:
            fail(f"{label}.targetResolutions[{idx}]: invalid resolution header")
        res_id = require_nonempty_str(res, "resolutionId", f"{label}.targetResolutions[{idx}]")
        resolution_ids.add(res_id)
        require_nonempty_str(res, "rawReference", f"{label}.targetResolutions[{idx}]")
        require_nonempty_str(res, "resolutionPosture", f"{label}.targetResolutions[{idx}]")
        require_nonempty_str(res, "resolutionMethod", f"{label}.targetResolutions[{idx}]")
        require_nonempty_str(res, "timestamp", f"{label}.targetResolutions[{idx}]")

    for idx, finding in enumerate(findings):
        if finding.get("kind") != "worm_finding" or finding.get("schemaVersion") != 1:
            fail(f"{label}.findings[{idx}]: invalid finding header")
        require_nonempty_str(finding, "findingId", f"{label}.findings[{idx}]")
        related = require(finding, "relatedEdgeIds", f"{label}.findings[{idx}]")
        if not isinstance(related, list) or not related:
            fail(f"{label}.findings[{idx}]: relatedEdgeIds must be a non-empty array")
        for edge_id in related:
            if edge_id not in edge_ids:
                fail(f"{label}.findings[{idx}]: related edge '{edge_id}' not present in bundle edges")
        require_nonempty_str(finding, "reasonCode", f"{label}.findings[{idx}]")
        require_nonempty_str(finding, "timestamp", f"{label}.findings[{idx}]")

    require_nonempty_str(data, "timestamp", label)

def main():
    files = sorted(EXAMPLES.glob("evidence_bundle_*.json"))
    if not files:
        fail(f"No evidence bundle examples found in {EXAMPLES}")
    count = 0
    for path in files:
        data = json.loads(path.read_text())
        validate_bundle(data, path.name)
        print(f"OK  {path.name}")
        count += 1
    print(f"Validated {count} Worm evidence bundle files successfully.")

if __name__ == "__main__":
    main()
