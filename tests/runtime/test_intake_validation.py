from __future__ import annotations

import copy
import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path

from cortex_runtime.intake_validation import (
    main,
    validate_intake_file,
    validate_intake_json_text,
    validate_intake_payload,
)


ROOT = Path(__file__).resolve().parents[2]
VALID_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/valid/intake-request-file-basic.json"
INVALID_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/invalid/intake-request-watcher-not-visible.json"
ORCHESTRATION_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/invalid/intake-request-orchestration-field.json"


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


class IntakeValidationRuntimeTests(unittest.TestCase):
    def test_valid_intake_fixture_passes_runtime_validation(self) -> None:
        payload = load_json(VALID_INTAKE_FIXTURE)

        result = validate_intake_payload(payload)

        self.assertTrue(result.accepted)
        self.assertEqual(result.validation_state, "accepted")
        self.assertEqual(result.request_id, "intake-001")
        self.assertIsNone(result.refusal_reason)
        self.assertEqual(result.error_count, 0)
        self.assertFalse(result.errors_truncated)
        self.assertEqual(result.errors, ())

    def test_invalid_intake_fixture_fails_runtime_validation(self) -> None:
        payload = load_json(INVALID_INTAKE_FIXTURE)

        result = validate_intake_payload(payload)

        self.assertFalse(result.accepted)
        self.assertEqual(result.validation_state, "denied")
        self.assertEqual(result.refusal_reason, "contract_invalid")
        self.assertGreaterEqual(result.error_count, 1)
        self.assertTrue(
            any(issue.path == "observation_policy.operator_visible" for issue in result.errors),
            result.to_dict(),
        )

    def test_orchestration_shaped_input_fails_closed_with_bounded_output(self) -> None:
        payload = load_json(VALID_INTAKE_FIXTURE)
        self.assertIsInstance(payload, dict)
        payload = copy.deepcopy(payload)
        payload["workflow_id"] = "wf-001"

        result = validate_intake_payload(payload)
        result_dict = result.to_dict()

        self.assertFalse(result.accepted)
        self.assertEqual(result.validation_state, "denied")
        self.assertEqual(
            set(result_dict.keys()),
            {
                "surface",
                "schema_ref",
                "validation_state",
                "accepted",
                "request_id",
                "refusal_reason",
                "error_count",
                "errors_truncated",
                "errors",
            },
        )
        self.assertNotIn("workflow_id", result_dict)
        self.assertNotIn("retry_after", result_dict)

    def test_invalid_json_text_is_explicitly_denied(self) -> None:
        result = validate_intake_json_text("{")

        self.assertFalse(result.accepted)
        self.assertEqual(result.validation_state, "denied")
        self.assertEqual(result.refusal_reason, "contract_invalid")
        self.assertEqual(result.error_count, 1)
        self.assertEqual(result.errors[0].path, "<root>")
        self.assertEqual(result.errors[0].message, "payload is not valid JSON")

    def test_file_entrypoint_emits_bounded_json_result(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            exit_code = main([str(VALID_INTAKE_FIXTURE)])

        payload = json.loads(output.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertTrue(payload["accepted"])
        self.assertEqual(payload["validation_state"], "accepted")
        self.assertEqual(payload["schema_ref"], "schemas/intake-request.schema.json")
        self.assertEqual(payload["errors"], [])

    def test_contract_fixture_for_orchestration_field_is_invalid(self) -> None:
        payload = load_json(ORCHESTRATION_INTAKE_FIXTURE)

        result = validate_intake_payload(payload)

        self.assertFalse(result.accepted)
        self.assertEqual(result.validation_state, "denied")

    def test_unreadable_file_fails_closed(self) -> None:
        missing_path = ROOT / "tests/contracts/fixtures/invalid/not-present.json"

        result = validate_intake_file(missing_path)

        self.assertFalse(result.accepted)
        self.assertEqual(result.validation_state, "denied")
        self.assertEqual(result.error_count, 1)
        self.assertEqual(result.errors[0].message, "payload could not be read")


if __name__ == "__main__":
    unittest.main()
