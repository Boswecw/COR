#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator  # type: ignore[import-untyped]


ROOT = Path(__file__).resolve().parent.parent
SCHEMAS_DIR = ROOT / "schemas"
TESTS_DIR = ROOT / "tests"
VALID_FIXTURES_DIR = TESTS_DIR / "contracts/fixtures/valid"
INVALID_FIXTURES_DIR = TESTS_DIR / "contracts/fixtures/invalid"

EXPECTED_HANDOFF_TRANSFER_STATES = {
    "ready_for_transfer",
    "denied",
    "stale",
    "integrity_failed",
    "re_prep_required",
}

EXPECTED_HANDOFF_REVERSE_SIGNALS = {
    "accepted",
    "rejected_reason_code",
    "re_prep_required",
    "stale",
    "integrity_failed",
}

EXPECTED_HANDOFF_DENIAL_REASONS = {
    "contract_invalid",
    "unsupported_destination",
    "privacy_scope_violation",
    "missing_required_structure",
}

EXPECTED_EMBEDDED_DIAGNOSTIC_STATES = {
    "ready",
    "degraded",
    "unavailable",
    "denied",
    "stale",
    "partial_success",
}

EXPECTED_EMBEDDED_DIAGNOSTIC_CONTROLS = {
    "re_run_extraction",
    "invalidate_artifact",
    "disable_watcher",
    "inspect_reason_codes",
}

FORBIDDEN_ORCHESTRATION_FIELDS = {
    "retry_count",
    "retry_policy",
    "workflow_id",
    "queue_name",
    "executor",
    "dispatch_plan",
    "orchestration_state",
    "agent_assignment",
}

FORBIDDEN_DIAGNOSTIC_FIELDS = {
    "raw_content_preview",
    "full_text_preview",
    "full_text_search",
    "full_text_browse",
    "content_browser",
    "raw_artifact_dump",
    "unbounded_artifact_dump",
}


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def validate_file(instance_path: Path, schema_path: Path) -> list[str]:
    schema = load_json(schema_path)
    instance = load_json(instance_path)

    validator = Draft202012Validator(schema)
    errors = sorted(validator.iter_errors(instance), key=lambda e: list(e.path))

    messages: list[str] = []
    for error in errors:
        loc = ".".join(str(part) for part in error.path) or "<root>"
        messages.append(f"{instance_path}: {loc}: {error.message}")

    return messages


def expect_valid(instance_path: Path, schema_path: Path) -> list[str]:
    return validate_file(instance_path, schema_path)


def expect_invalid(instance_path: Path, schema_path: Path) -> list[str]:
    errors = validate_file(instance_path, schema_path)
    if errors:
        return []
    return [f"{instance_path}: expected invalid but validation passed"]


def discover_schemas() -> dict[str, Path]:
    return {
        path.name.removesuffix(".schema.json"): path
        for path in sorted(SCHEMAS_DIR.glob("*.schema.json"))
    }


def match_schema_key(fixture_path: Path, schema_keys: list[str]) -> str:
    fixture_key = fixture_path.name.removesuffix(".json")
    matches = [
        schema_key
        for schema_key in schema_keys
        if fixture_key == schema_key or fixture_key.startswith(f"{schema_key}-")
    ]

    if not matches:
        raise ValueError(f"{fixture_path}: no matching schema found for fixture name")

    return max(matches, key=len)


def discover_cases(fixtures_dir: Path, schema_map: dict[str, Path]) -> tuple[list[tuple[Path, Path]], dict[str, int], list[str]]:
    cases: list[tuple[Path, Path]] = []
    coverage = {schema_key: 0 for schema_key in schema_map}
    failures: list[str] = []
    schema_keys = sorted(schema_map.keys(), key=len, reverse=True)

    for fixture_path in sorted(fixtures_dir.rglob("*.json")):
        try:
            schema_key = match_schema_key(fixture_path, schema_keys)
        except ValueError as exc:
            failures.append(str(exc))
            continue

        coverage[schema_key] += 1
        cases.append((fixture_path, schema_map[schema_key]))

    return cases, coverage, failures


def verify_fixture_coverage(valid_coverage: dict[str, int], invalid_coverage: dict[str, int]) -> list[str]:
    failures: list[str] = []
    for schema_key in sorted(valid_coverage):
        if valid_coverage[schema_key] == 0:
            failures.append(f"{schema_key}: missing valid fixture coverage")
        if invalid_coverage[schema_key] == 0:
            failures.append(f"{schema_key}: missing invalid fixture coverage")
    return failures


def extract_forbidden_required_fields(schema: dict[str, Any]) -> set[str]:
    fields: set[str] = set()
    for clause in schema.get("allOf", []):
        not_clause = clause.get("not")
        if not isinstance(not_clause, dict):
            continue
        for candidate in not_clause.get("anyOf", []):
            required = candidate.get("required")
            if isinstance(required, list) and len(required) == 1:
                field_name = required[0]
                if isinstance(field_name, str):
                    fields.add(field_name)
    return fields


def verify_handoff_schema_contract() -> list[str]:
    schema_path = SCHEMAS_DIR / "handoff-envelope.schema.json"
    if not schema_path.exists():
        return [f"{schema_path}: handoff schema missing"]

    schema = load_json(schema_path)
    failures: list[str] = []

    if schema.get("additionalProperties") is not False:
        failures.append(f"{schema_path}: root additionalProperties must be false")

    required_fields = set(schema.get("required", []))
    if "reverse_signal" in required_fields:
        failures.append(f"{schema_path}: reverse_signal must remain optional")

    properties = schema.get("properties", {})
    forbidden_properties = FORBIDDEN_ORCHESTRATION_FIELDS.intersection(properties.keys())
    if forbidden_properties:
        failures.append(
            f"{schema_path}: forbidden orchestration fields present in schema properties: {sorted(forbidden_properties)}"
        )

    transfer_state_enum = set(properties.get("transfer_state", {}).get("enum", []))
    if transfer_state_enum != EXPECTED_HANDOFF_TRANSFER_STATES:
        failures.append(
            f"{schema_path}: transfer_state enum mismatch: expected {sorted(EXPECTED_HANDOFF_TRANSFER_STATES)}, got {sorted(transfer_state_enum)}"
        )

    reverse_signal_enum = set(properties.get("reverse_signal", {}).get("enum", []))
    if reverse_signal_enum != EXPECTED_HANDOFF_REVERSE_SIGNALS:
        failures.append(
            f"{schema_path}: reverse_signal enum mismatch: expected {sorted(EXPECTED_HANDOFF_REVERSE_SIGNALS)}, got {sorted(reverse_signal_enum)}"
        )

    denial_reason_enum = set(
        properties.get("denial", {}).get("properties", {}).get("reason_class", {}).get("enum", [])
    )
    if denial_reason_enum != EXPECTED_HANDOFF_DENIAL_REASONS:
        failures.append(
            f"{schema_path}: denial reason enum mismatch: expected {sorted(EXPECTED_HANDOFF_DENIAL_REASONS)}, got {sorted(denial_reason_enum)}"
        )

    forbidden_required_fields = extract_forbidden_required_fields(schema)
    if forbidden_required_fields != FORBIDDEN_ORCHESTRATION_FIELDS:
        failures.append(
            f"{schema_path}: forbidden orchestration guard mismatch: expected {sorted(FORBIDDEN_ORCHESTRATION_FIELDS)}, got {sorted(forbidden_required_fields)}"
        )

    return failures


def verify_embedded_diagnostics_schema_contract() -> list[str]:
    schema_path = SCHEMAS_DIR / "embedded-diagnostics.schema.json"
    if not schema_path.exists():
        return [f"{schema_path}: embedded diagnostics schema missing"]

    schema = load_json(schema_path)
    failures: list[str] = []

    if schema.get("additionalProperties") is not False:
        failures.append(f"{schema_path}: root additionalProperties must be false")

    properties = schema.get("properties", {})
    forbidden_properties = FORBIDDEN_DIAGNOSTIC_FIELDS.intersection(properties.keys())
    if forbidden_properties:
        failures.append(
            f"{schema_path}: forbidden diagnostics fields present in schema properties: {sorted(forbidden_properties)}"
        )

    state_enum = set(properties.get("state", {}).get("enum", []))
    if state_enum != EXPECTED_EMBEDDED_DIAGNOSTIC_STATES:
        failures.append(
            f"{schema_path}: state enum mismatch: expected {sorted(EXPECTED_EMBEDDED_DIAGNOSTIC_STATES)}, got {sorted(state_enum)}"
        )

    control_enum = set(properties.get("allowed_controls", {}).get("items", {}).get("enum", []))
    if control_enum != EXPECTED_EMBEDDED_DIAGNOSTIC_CONTROLS:
        failures.append(
            f"{schema_path}: allowed_controls enum mismatch: expected {sorted(EXPECTED_EMBEDDED_DIAGNOSTIC_CONTROLS)}, got {sorted(control_enum)}"
        )

    details_redacted = properties.get("details_redacted", {})
    if details_redacted.get("const") is not True:
        failures.append(f"{schema_path}: details_redacted must be const true")

    watcher_contract_scoped_only = (
        properties.get("watcher_summary", {})
        .get("properties", {})
        .get("contract_scoped_only", {})
        .get("const")
    )
    if watcher_contract_scoped_only is not True:
        failures.append(f"{schema_path}: watcher_summary.contract_scoped_only must be const true")

    forbidden_required_fields = extract_forbidden_required_fields(schema)
    if forbidden_required_fields != FORBIDDEN_DIAGNOSTIC_FIELDS:
        failures.append(
            f"{schema_path}: forbidden diagnostics guard mismatch: expected {sorted(FORBIDDEN_DIAGNOSTIC_FIELDS)}, got {sorted(forbidden_required_fields)}"
        )

    return failures


def main() -> int:
    failures: list[str] = []
    schema_map = discover_schemas()

    valid_cases, valid_coverage, discovery_failures = discover_cases(VALID_FIXTURES_DIR, schema_map)
    failures.extend(discovery_failures)

    invalid_cases, invalid_coverage, discovery_failures = discover_cases(INVALID_FIXTURES_DIR, schema_map)
    failures.extend(discovery_failures)

    failures.extend(verify_fixture_coverage(valid_coverage, invalid_coverage))
    failures.extend(verify_handoff_schema_contract())
    failures.extend(verify_embedded_diagnostics_schema_contract())

    for instance_path, schema_path in valid_cases:
        failures.extend(expect_valid(instance_path, schema_path))

    for instance_path, schema_path in invalid_cases:
        failures.extend(expect_invalid(instance_path, schema_path))

    if failures:
        print("VALIDATION FAILED")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print(
        f"VALIDATION PASSED ({len(valid_cases)} valid fixtures, {len(invalid_cases)} invalid fixtures, {len(schema_map)} schemas)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
