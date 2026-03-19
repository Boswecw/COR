from __future__ import annotations

import copy
import unittest
from pathlib import Path

from cortex_runtime.extraction_emission import emit_extraction_result_from_intake_payload
from cortex_runtime.extraction_emission import emit_extraction_result_from_source_file
from cortex_runtime.extraction_emission import main as extraction_main
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_source_file
from cortex_runtime.retrieval_package_emission import main as retrieval_main
from tests.runtime.runtime_test_support import (
    ROOT,
    assert_schema_valid,
    build_file_intake_payload,
    capture_cli_result,
)


ODT_READY_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.odt"
ODT_REVIEWED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-reviewed.odt"
ODT_MISSING_CONTENT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-missing-content.odt"
ODT_CORRUPT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-corrupt.odt"


def build_odt_intake_payload(path: Path) -> dict[str, object]:
    return copy.deepcopy(
        build_file_intake_payload(
            path,
            "application/vnd.oasis.opendocument.text",
        )
    )


class OdtLaneRuntimeTests(unittest.TestCase):
    def test_odt_intake_emits_ready_extraction_result(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_odt_intake_payload(ODT_READY_FIXTURE))

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["completeness"]["status"], "complete")
        self.assertEqual(result["structures"]["metadata_fields"]["source_lane"], "odt_text")
        self.assertEqual(len(result["structures"]["sections"]), 2)
        self.assertEqual(result["structures"]["tables_detected"], 1)
        self.assertEqual(
            {block["block_kind"] for block in result["structures"]["content_blocks"]},
            {"heading", "paragraph", "list", "table"},
        )

    def test_odt_extraction_is_deterministic(self) -> None:
        first = emit_extraction_result_from_source_file(
            ODT_READY_FIXTURE,
            request_id="odt-det-001",
            source_ref="odt-det",
            media_type="application/vnd.oasis.opendocument.text",
        )
        second = emit_extraction_result_from_source_file(
            ODT_READY_FIXTURE,
            request_id="odt-det-001",
            source_ref="odt-det",
            media_type="application/vnd.oasis.opendocument.text",
        )

        assert_schema_valid(self, first, schema_name="extraction-result.schema.json")
        assert_schema_valid(self, second, schema_name="extraction-result.schema.json")
        self.assertEqual(first["structures"], second["structures"])

    def test_reviewed_odt_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            ODT_REVIEWED_FIXTURE,
            request_id="odt-denied-001",
            source_ref="odt-reviewed",
            media_type="application/vnd.oasis.opendocument.text",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_missing_content_odt_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            ODT_MISSING_CONTENT_FIXTURE,
            request_id="odt-missing-001",
            source_ref="odt-missing",
            media_type="application/vnd.oasis.opendocument.text",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_corrupt_odt_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            ODT_CORRUPT_FIXTURE,
            request_id="odt-corrupt-001",
            source_ref="odt-corrupt",
            media_type="application/vnd.oasis.opendocument.text",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_ready_odt_extraction_is_retrieval_compatible(self) -> None:
        result = emit_retrieval_package_from_source_file(
            ODT_READY_FIXTURE,
            request_id="odt-ret-001",
            source_ref="odt-ret",
            media_type="application/vnd.oasis.opendocument.text",
        )

        assert_schema_valid(self, result, schema_name="retrieval-package.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["retrieval_profile"]["chunking_mode"], "section")
        self.assertEqual([chunk["ordinal"] for chunk in result["chunks"]], [0, 1])
        self.assertEqual({chunk["structure_kind"] for chunk in result["chunks"]}, {"section"})

    def test_odt_extraction_cli_is_ready_with_declared_media_type(self) -> None:
        exit_code, result = capture_cli_result(
            extraction_main,
            [
                "--source-path",
                str(ODT_READY_FIXTURE),
                "--request-id",
                "odt-cli-001",
                "--source-ref",
                "odt-cli",
                "--media-type",
                "application/vnd.oasis.opendocument.text",
            ],
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")

    def test_odt_retrieval_cli_is_ready_with_declared_media_type(self) -> None:
        exit_code, result = capture_cli_result(
            retrieval_main,
            [
                "--source-path",
                str(ODT_READY_FIXTURE),
                "--request-id",
                "odt-ret-cli-001",
                "--source-ref",
                "odt-ret-cli",
                "--media-type",
                "application/vnd.oasis.opendocument.text",
            ],
        )

        assert_schema_valid(self, result, schema_name="retrieval-package.schema.json")
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")


if __name__ == "__main__":
    unittest.main()
