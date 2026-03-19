from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parent.parent
INTAKE_SCHEMA_PATH = ROOT / "schemas/intake-request.schema.json"
INTAKE_SCHEMA_REF = "schemas/intake-request.schema.json"
MAX_ERROR_COUNT = 10


@dataclass(frozen=True, slots=True)
class IntakeValidationIssue:
    path: str
    message: str

    def to_dict(self) -> dict[str, str]:
        return {
            "path": self.path,
            "message": self.message,
        }


@dataclass(frozen=True, slots=True)
class IntakeValidationResult:
    surface: str
    schema_ref: str
    validation_state: str
    accepted: bool
    request_id: str | None
    refusal_reason: str | None
    error_count: int
    errors_truncated: bool
    errors: tuple[IntakeValidationIssue, ...]

    def to_dict(self) -> dict[str, Any]:
        return {
            "surface": self.surface,
            "schema_ref": self.schema_ref,
            "validation_state": self.validation_state,
            "accepted": self.accepted,
            "request_id": self.request_id,
            "refusal_reason": self.refusal_reason,
            "error_count": self.error_count,
            "errors_truncated": self.errors_truncated,
            "errors": [issue.to_dict() for issue in self.errors],
        }


def _extract_request_id(payload: Any) -> str | None:
    if not isinstance(payload, dict):
        return None

    request_id = payload.get("request_id")
    if isinstance(request_id, str) and request_id:
        return request_id

    return None


def _build_result(
    *,
    accepted: bool,
    request_id: str | None,
    issues: list[IntakeValidationIssue],
) -> IntakeValidationResult:
    return IntakeValidationResult(
        surface="intake_request_validation",
        schema_ref=INTAKE_SCHEMA_REF,
        validation_state="accepted" if accepted else "denied",
        accepted=accepted,
        request_id=request_id,
        refusal_reason=None if accepted else "contract_invalid",
        error_count=len(issues),
        errors_truncated=len(issues) > MAX_ERROR_COUNT,
        errors=tuple(issues[:MAX_ERROR_COUNT]),
    )


def _issue(path: str, message: str) -> IntakeValidationIssue:
    return IntakeValidationIssue(path=path, message=message)


@lru_cache(maxsize=1)
def _intake_validator() -> Draft202012Validator:
    with INTAKE_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        schema = json.load(handle)
    return Draft202012Validator(schema)


def validate_intake_payload(payload: Any) -> IntakeValidationResult:
    request_id = _extract_request_id(payload)
    validator = _intake_validator()
    errors = sorted(
        validator.iter_errors(payload),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    issues = [
        _issue(
            ".".join(str(part) for part in error.path) or "<root>",
            error.message,
        )
        for error in errors
    ]

    return _build_result(
        accepted=not issues,
        request_id=request_id,
        issues=issues,
    )


def validate_intake_json_text(payload_text: str) -> IntakeValidationResult:
    try:
        payload = json.loads(payload_text)
    except json.JSONDecodeError:
        return _build_result(
            accepted=False,
            request_id=None,
            issues=[_issue("<root>", "payload is not valid JSON")],
        )

    return validate_intake_payload(payload)


def validate_intake_file(path: str | Path) -> IntakeValidationResult:
    try:
        payload_text = Path(path).read_text(encoding="utf-8")
    except OSError:
        return _build_result(
            accepted=False,
            request_id=None,
            issues=[_issue("<root>", "payload could not be read")],
        )

    return validate_intake_json_text(payload_text)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate a Cortex intake request payload against the bounded intake contract."
    )
    parser.add_argument(
        "input",
        help="Path to a JSON payload file, or '-' to read JSON from stdin.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)

    if args.input == "-":
        result = validate_intake_json_text(sys.stdin.read())
    else:
        result = validate_intake_file(args.input)

    print(json.dumps(result.to_dict(), indent=2, sort_keys=True))
    return 0 if result.accepted else 1


if __name__ == "__main__":
    raise SystemExit(main())
