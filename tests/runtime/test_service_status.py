from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

from cortex_runtime.extraction_emission import pdf_lane_runtime_available
from cortex_runtime.service_status import emit_service_status, main
from tests.runtime.runtime_test_support import assert_schema_valid


class ServiceStatusRuntimeTests(unittest.TestCase):
    def test_service_status_emits_schema_valid_ready_output(self) -> None:
        result = emit_service_status()

        assert_schema_valid(self, result, schema_name="service-status.schema.json")
        expected_state = "ready" if pdf_lane_runtime_available() else "degraded"
        self.assertEqual(result["state"], expected_state)
        self.assertEqual(result["service_id"], "cortex")
        self.assertEqual(result["service_class"], "file_intelligence")

    def test_implemented_slices_and_admitted_source_lanes_are_reported(self) -> None:
        result = emit_service_status()

        assert_schema_valid(self, result, schema_name="service-status.schema.json")
        expected_slices = [
            "slice1_intake_validation",
            "slice2_extraction_emission",
            "slice3_retrieval_package_emission",
            "slice4_service_status_truth",
            "slice6_docx_source_lane",
            "slice7_rtf_source_lane",
            "slice8_odt_source_lane",
        ]
        expected_lanes = [
            "local_file_markdown",
            "local_file_plain_text",
        ]
        if pdf_lane_runtime_available():
            expected_slices.insert(4, "slice5_pdf_source_lane")
        expected_lanes.append("local_file_pdf_text")
        expected_lanes.append("local_file_docx_text")
        expected_lanes.append("local_file_rtf_text")
        expected_lanes.append("local_file_odt_text")
        self.assertEqual(
            result["runtime_surface_summary"]["implemented_slices"],
            expected_slices,
        )
        self.assertEqual(result["runtime_surface_summary"]["admitted_source_lanes"], expected_lanes)
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

        assert_schema_valid(self, result, schema_name="service-status.schema.json")
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

        assert_schema_valid(self, result, schema_name="service-status.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["runtime_surface_summary"]["admitted_source_lanes"], [])

    def test_unavailable_status_is_reported_when_registry_exceeds_bounded_vocabulary(self) -> None:
        with patch(
            "cortex_runtime.service_status._admitted_source_lanes",
            return_value=(
                [],
                "Cortex source-lane truth is unavailable because extraction support exceeds the bounded service-status vocabulary.",
            ),
        ):
            result = emit_service_status()

        assert_schema_valid(self, result, schema_name="service-status.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertIn("bounded service-status vocabulary", result["operator_visible_message"])

    def test_output_remains_informational_only(self) -> None:
        result = emit_service_status()

        assert_schema_valid(self, result, schema_name="service-status.schema.json")
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
        assert_schema_valid(self, result, schema_name="service-status.schema.json")
        expected_state = "ready" if pdf_lane_runtime_available() else "degraded"
        self.assertEqual(exit_code, 0 if expected_state == "ready" else 1)
        self.assertEqual(result["state"], expected_state)


if __name__ == "__main__":
    unittest.main()
