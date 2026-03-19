from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from jsonschema import Draft202012Validator

from cortex_runtime.service_status import emit_service_status, main


ROOT = Path(__file__).resolve().parents[2]
SERVICE_STATUS_SCHEMA_PATH = ROOT / "schemas/service-status.schema.json"


def service_status_validator() -> Draft202012Validator:
    with SERVICE_STATUS_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        return Draft202012Validator(json.load(handle))


def assert_schema_valid(testcase: unittest.TestCase, payload: dict[str, object]) -> None:
    errors = sorted(
        service_status_validator().iter_errors(payload),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    testcase.assertEqual(
        [],
        [f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}" for error in errors],
    )


class ServiceStatusRuntimeTests(unittest.TestCase):
    def test_service_status_emits_schema_valid_ready_output(self) -> None:
        result = emit_service_status()

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["service_id"], "cortex")
        self.assertEqual(result["service_class"], "file_intelligence")

    def test_implemented_slices_and_admitted_source_lanes_are_reported(self) -> None:
        result = emit_service_status()

        assert_schema_valid(self, result)
        self.assertEqual(
            result["runtime_surface_summary"]["implemented_slices"],
            [
                "slice1_intake_validation",
                "slice2_extraction_emission",
                "slice3_retrieval_package_emission",
                "slice4_service_status_truth",
            ],
        )
        self.assertEqual(
            result["runtime_surface_summary"]["admitted_source_lanes"],
            ["local_file_markdown", "local_file_plain_text"],
        )
        self.assertEqual(result["watcher_summary"]["active_watch_scope_count"], 0)

    def test_degraded_status_is_reported_when_runtime_slice_is_missing(self) -> None:
        with patch(
            "cortex_runtime.service_status._implemented_runtime_slices",
            return_value=[
                "slice1_intake_validation",
                "slice2_extraction_emission",
                "slice4_service_status_truth",
            ],
        ):
            result = emit_service_status()

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "degraded")
        self.assertEqual(result["degraded_subtype"], "dependency_unavailable")
        self.assertNotIn("slice3_retrieval_package_emission", result["runtime_surface_summary"]["implemented_slices"])

    def test_unavailable_status_is_reported_when_no_source_lanes_are_admitted(self) -> None:
        with patch(
            "cortex_runtime.service_status._admitted_source_lanes",
            return_value=(
                [],
                "Cortex is unavailable because no governed local source lanes are currently admitted.",
            ),
        ):
            result = emit_service_status()

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["runtime_surface_summary"]["admitted_source_lanes"], [])

    def test_output_remains_informational_only(self) -> None:
        result = emit_service_status()

        assert_schema_valid(self, result)
        self.assertNotIn("next_action", result)
        self.assertNotIn("recommendation", result)
        self.assertNotIn("workflow_id", result)
        self.assertNotIn("dispatch_plan", result)
        self.assertNotIn("executor", result)
        self.assertNotIn("queue_name", result)

    def test_cli_entrypoint_emits_schema_valid_json(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            exit_code = main([])

        result = json.loads(output.getvalue())
        assert_schema_valid(self, result)
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")


if __name__ == "__main__":
    unittest.main()
