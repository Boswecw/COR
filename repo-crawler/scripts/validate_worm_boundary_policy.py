#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "doc/system/worm/examples"

MODES = {"local_repo_only","same_org_governed","governed_external_reference"}
FALLBACK = {"blocked","ambiguous","unresolved"}
IDENTITY_KEYS = {"normalized_repo","raw_reference"}

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

def require_bool(obj: dict, key: str, label: str):
    value = require(obj, key, label)
    if not isinstance(value, bool):
        fail(f"{label}: key '{key}' must be a boolean")
    return value

def require_int(obj: dict, key: str, label: str, minimum: int):
    value = require(obj, key, label)
    if not isinstance(value, int) or value < minimum:
        fail(f"{label}: key '{key}' must be an integer >= {minimum}")
    return value

def validate_policy(data: dict, label: str):
    if data.get("kind") != "worm_traversal_policy":
        fail(f"{label}: kind must be 'worm_traversal_policy'")
    if data.get("schemaVersion") != 1:
        fail(f"{label}: schemaVersion must be 1")

    require_nonempty_str(data, "policyId", label)
    mode = require_nonempty_str(data, "mode", label)
    if mode not in MODES:
        fail(f"{label}: invalid mode '{mode}'")

    max_depth = require_int(data, "maxDepth", label, 0)
    require_int(data, "maxTargetsPerSourceRepo", label, 1)
    deterministic = require_bool(data, "deterministicTraversal", label)

    cycle = require(data, "cycleDetection", label)
    require_bool(cycle, "enabled", f"{label}.cycleDetection")
    identity_key = require_nonempty_str(cycle, "identityKey", f"{label}.cycleDetection")
    if identity_key not in IDENTITY_KEYS:
        fail(f"{label}.cycleDetection: invalid identityKey '{identity_key}'")

    scope = require(data, "scopePolicy", label)
    allow_same_org = require_bool(scope, "allowSameOrg", f"{label}.scopePolicy")
    allow_external = require_bool(scope, "allowExternalReferences", f"{label}.scopePolicy")
    require_explicit = require_bool(scope, "requireExplicitAllowlistForExpansion", f"{label}.scopePolicy")

    repo_allowlist = data.get("repoAllowlist", [])
    repo_denylist = data.get("repoDenylist", [])
    if not isinstance(repo_allowlist, list) or not all(isinstance(x, str) and x.strip() for x in repo_allowlist):
        fail(f"{label}: repoAllowlist must be an array of non-empty strings")
    if not isinstance(repo_denylist, list) or not all(isinstance(x, str) and x.strip() for x in repo_denylist):
        fail(f"{label}: repoDenylist must be an array of non-empty strings")

    network = require(data, "networkPolicy", label)
    allow_unknown_hosts = require_bool(network, "allowUnknownHosts", f"{label}.networkPolicy")
    allowed_hosts = require(network, "allowedHosts", f"{label}.networkPolicy")
    if not isinstance(allowed_hosts, list) or not all(isinstance(x, str) and x.strip() for x in allowed_hosts):
        fail(f"{label}.networkPolicy: allowedHosts must be an array of non-empty strings")

    fallback = require_nonempty_str(data, "fallbackPosture", label)
    if fallback not in FALLBACK:
        fail(f"{label}: invalid fallbackPosture '{fallback}'")

    if mode == "local_repo_only":
        if max_depth != 0:
            fail(f"{label}: local_repo_only requires maxDepth == 0")
        if allow_same_org or allow_external:
            fail(f"{label}: local_repo_only cannot allow same-org or external expansion")
        if allowed_hosts:
            fail(f"{label}: local_repo_only should not include allowedHosts")

    if mode == "same_org_governed":
        if max_depth < 1:
            fail(f"{label}: same_org_governed requires maxDepth >= 1")
        if not allow_same_org or allow_external:
            fail(f"{label}: same_org_governed must allow same-org only")
        if require_explicit and not repo_allowlist:
            fail(f"{label}: same_org_governed with explicit allowlist requires repoAllowlist entries")

    if mode == "governed_external_reference":
        if max_depth < 1:
            fail(f"{label}: governed_external_reference requires maxDepth >= 1")
        if not allow_external:
            fail(f"{label}: governed_external_reference must allow external references")
        if require_explicit and not repo_allowlist:
            fail(f"{label}: governed_external_reference with explicit allowlist requires repoAllowlist entries")
        if not allow_unknown_hosts and not allowed_hosts:
            fail(f"{label}: governed_external_reference with allowUnknownHosts=false requires allowedHosts entries")

    if not deterministic:
        fail(f"{label}: deterministicTraversal must remain true in this contract set")

def main():
    files = sorted(EXAMPLES.glob("boundary_policy_*.json"))
    if not files:
        fail(f"No boundary policy examples found in {EXAMPLES}")

    count = 0
    for path in files:
        data = json.loads(path.read_text())
        validate_policy(data, path.name)
        print(f"OK  {path.name}")
        count += 1

    print(f"Validated {count} Worm traversal policy files successfully.")

if __name__ == "__main__":
    main()
