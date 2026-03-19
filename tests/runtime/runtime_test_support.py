from __future__ import annotations

import copy
import json
import unittest
from functools import lru_cache
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[2]
VALID_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/valid/intake-request-file-basic.json"


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


@lru_cache(maxsize=None)
def schema_validator(schema_name: str) -> Draft202012Validator:
    schema_path = ROOT / "schemas" / schema_name
    with schema_path.open("r", encoding="utf-8") as handle:
        return Draft202012Validator(json.load(handle))


def assert_schema_valid(
    testcase: unittest.TestCase,
    payload: dict[str, Any],
    *,
    schema_name: str,
) -> None:
    validator = schema_validator(schema_name)
    errors = sorted(
        validator.iter_errors(payload),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    testcase.assertEqual(
        [],
        [f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}" for error in errors],
    )


def build_file_intake_payload(path: Path, media_type: str) -> dict[str, Any]:
    payload = load_json(VALID_INTAKE_FIXTURE)
    assert isinstance(payload, dict)
    payload = copy.deepcopy(payload)
    payload["sources"][0]["path"] = str(path)
    payload["sources"][0]["media_type"] = media_type
    payload["requested_artifact"] = "extraction_result"
    return payload
