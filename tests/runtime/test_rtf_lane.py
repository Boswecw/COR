from __future__ import annotations

import copy
import unittest
from pathlib import Path

from cortex_runtime.extraction_emission import emit_extraction_result_from_intake_payload
from cortex_runtime.extraction_emission import emit_extraction_result_from_source_file
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_source_file
from tests.runtime.runtime_test_support import (
    ROOT,
    assert_schema_valid,
    build_file_intake_payload,
)


RTF_READY_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.rtf"
RTF_ANNOTATED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-annotated.rtf"
RTF_CORRUPT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-corrupt.rtf"


def build_rtf_intake_payload(path: Path) -> dict[str, object]:
    return copy.deepcopy(build_file_intake_payload(path, "application/rtf"))


class RtfLaneRuntimeTests(unittest.TestCase):
    def test_rtf_intake_emits_ready_extraction_result(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_rtf_intake_payload(RTF_READY_FIXTURE))

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["completeness"]["status"], "complete")
        self.assertEqual(result["structures"]["metadata_fields"]["source_lane"], "rtf_text")
        self.assertEqual(len(result["structures"]["content_blocks"]), 3)
        self.assertNotIn("sections", result["structures"])
        self.assertEqual({block["block_kind"] for block in result["structures"]["content_blocks"]}, {"paragraph"})

    def test_rtf_extraction_is_deterministic(self) -> None:
        first = emit_extraction_result_from_source_file(
            RTF_READY_FIXTURE,
            request_id="rtf-det-001",
            source_ref="rtf-det",
            media_type="application/rtf",
        )
        second = emit_extraction_result_from_source_file(
            RTF_READY_FIXTURE,
            request_id="rtf-det-001",
            source_ref="rtf-det",
            media_type="application/rtf",
        )

        assert_schema_valid(self, first, schema_name="extraction-result.schema.json")
        assert_schema_valid(self, second, schema_name="extraction-result.schema.json")
        self.assertEqual(first["structures"], second["structures"])

    def test_annotated_rtf_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            RTF_ANNOTATED_FIXTURE,
            request_id="rtf-denied-001",
            source_ref="rtf-annotated",
            media_type="text/rtf",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_corrupt_rtf_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            RTF_CORRUPT_FIXTURE,
            request_id="rtf-bad-001",
            source_ref="rtf-corrupt",
            media_type="application/rtf",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_ready_rtf_extraction_is_retrieval_compatible(self) -> None:
        result = emit_retrieval_package_from_source_file(
            RTF_READY_FIXTURE,
            request_id="rtf-ret-001",
            source_ref="rtf-ret",
            media_type="text/rtf",
        )

        assert_schema_valid(self, result, schema_name="retrieval-package.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["retrieval_profile"]["chunking_mode"], "paragraph")
        self.assertEqual([chunk["ordinal"] for chunk in result["chunks"]], [0, 1, 2])
        self.assertEqual({chunk["structure_kind"] for chunk in result["chunks"]}, {"paragraph"})


if __name__ == "__main__":
    unittest.main()
