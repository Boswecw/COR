from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator  # type: ignore[import-untyped]


ROOT = Path(__file__).resolve().parents[2]
SCHEMAS_DIR = ROOT / "schemas"


@lru_cache(maxsize=None)
def schema_validator(schema_name: str) -> Draft202012Validator:
    schema_path = SCHEMAS_DIR / schema_name
    with schema_path.open("r", encoding="utf-8") as handle:
        return Draft202012Validator(json.load(handle))


def schema_error_messages(payload: dict[str, Any], *, schema_name: str) -> list[str]:
    validator = schema_validator(schema_name)
    errors = sorted(
        validator.iter_errors(payload),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    return [
        f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}"
        for error in errors
    ]


def require_schema_valid(payload: dict[str, Any], *, schema_name: str) -> None:
    errors = schema_error_messages(payload, schema_name=schema_name)
    if errors:
        raise ValueError(f"{schema_name} validation failed: " + "; ".join(errors))
