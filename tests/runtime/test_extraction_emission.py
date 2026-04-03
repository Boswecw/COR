from __future__ import annotations

import copy
import unittest
from typing import Any

from cortex_runtime.extraction_emission import (
    emit_extraction_result_from_intake_json_text,
    emit_extraction_result_from_intake_payload,
    emit_extraction_result_from_source_file,
    main,
)
from tests.runtime.runtime_test_support import (
    ROOT,
    assert_schema_valid,
    build_file_intake_payload,
    capture_cli_result,
    load_json,
)

VALID_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/valid/intake-request-file-basic.json"
INVALID_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/invalid/intake-request-watcher-not-visible.json"
SUPPORTED_SOURCE_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.md"
EMPTY_TEXT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-empty.txt"
UNSUPPORTED_SOURCE_FIXTURE = ROOT / "tests/runtime/fixtures/sample-unsupported.bin"


def build_supported_intake_payload() -> dict[str, Any]:
    return build_file_intake_payload(SUPPORTED_SOURCE_FIXTURE, "text/markdown")


class ExtractionEmissionRuntimeTests(unittest.TestCase):
    def test_supported_valid_intake_emits_ready_extraction_result(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_supported_intake_payload())

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["syntax_boundary"], "syntax_only")
        self.assertTrue(result["semantic_boundary_enforced"])
        self.assertEqual(result["completeness"]["status"], "complete")
        self.assertNotIn("refusal", result)
        self.assertTrue(result["structures"]["content_blocks"])

    def test_direct_source_file_emits_schema_valid_ready_result(self) -> None:
        result = emit_extraction_result_from_source_file(
            SUPPORTED_SOURCE_FIXTURE,
            request_id="direct-001",
            source_ref="src-direct",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["source_ref"], "src-direct")

    def test_empty_text_source_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            EMPTY_TEXT_FIXTURE,
            request_id="empty-text-001",
            source_ref="empty-text",
            media_type="text/plain",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_output_remains_syntax_only_and_bounded(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_supported_intake_payload())

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertNotIn("summary", result)
        self.assertNotIn("tags", result)
        self.assertNotIn("workflow_id", result)
        self.assertNotIn("dispatch_plan", result)
        self.assertEqual(result["syntax_boundary"], "syntax_only")
        self.assertTrue(all(block["block_kind"] in {"heading", "paragraph"} for block in result["structures"]["content_blocks"]))

    def test_unsupported_input_fails_closed(self) -> None:
        payload = build_supported_intake_payload()
        payload["sources"][0]["path"] = str(UNSUPPORTED_SOURCE_FIXTURE)
        payload["sources"][0]["media_type"] = "application/octet-stream"

        result = emit_extraction_result_from_intake_payload(payload)

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_unreadable_input_fails_closed(self) -> None:
        payload = build_supported_intake_payload()
        payload["sources"][0]["path"] = str(ROOT / "tests/runtime/fixtures/not-present.md")

        result = emit_extraction_result_from_intake_payload(payload)

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_invalid_intake_payload_fails_closed(self) -> None:
        payload = load_json(INVALID_INTAKE_FIXTURE)

        result = emit_extraction_result_from_intake_payload(payload)

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "ineligible_source")

    def test_malformed_intake_json_fails_closed(self) -> None:
        result = emit_extraction_result_from_intake_json_text("{")

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "ineligible_source")

    def test_cli_entrypoint_emits_ready_json_for_direct_source(self) -> None:
        exit_code, result = capture_cli_result(
            main,
            [
                "--source-path",
                str(SUPPORTED_SOURCE_FIXTURE),
                "--request-id",
                "cli-001",
                "--source-ref",
                "src-cli",
                "--media-type",
                "text/markdown",
            ],
        )
        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")

    def test_cli_media_type_mismatch_fails_closed(self) -> None:
        exit_code, result = capture_cli_result(
            main,
            [
                "--source-path",
                str(SUPPORTED_SOURCE_FIXTURE),
                "--request-id",
                "cli-mismatch-001",
                "--source-ref",
                "src-cli-mismatch",
                "--media-type",
                "application/octet-stream",
            ],
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(exit_code, 1)
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_cli_unreadable_source_fails_closed(self) -> None:
        exit_code, result = capture_cli_result(
            main,
            [
                "--source-path",
                str(ROOT / "tests/runtime/fixtures/not-present.md"),
                "--request-id",
                "cli-missing-001",
                "--source-ref",
                "src-cli-missing",
            ],
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(exit_code, 1)
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")


if __name__ == "__main__":
    unittest.main()
