from __future__ import annotations

import unittest
from pathlib import Path

from cortex_runtime.extraction_emission import (
    admitted_source_lanes,
    emit_extraction_result_from_source_file,
    pdf_lane_runtime_available,
)
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_source_file
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid


def ready_lane_cases() -> list[tuple[Path, str, str]]:
    cases = [
        (
            ROOT / "tests/runtime/fixtures/sample-note.md",
            "text/markdown",
            "markdown_text",
        ),
        (
            ROOT / "tests/runtime/fixtures/sample-note.txt",
            "text/plain",
            "plain_text",
        ),
        (
            ROOT / "tests/runtime/fixtures/sample-note.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "docx_text",
        ),
        (
            ROOT / "tests/runtime/fixtures/sample-note.rtf",
            "application/rtf",
            "rtf_text",
        ),
    ]
    if pdf_lane_runtime_available():
        cases.append(
            (
                ROOT / "tests/runtime/fixtures/sample-note.pdf",
                "application/pdf",
                "pdf_text",
            )
        )
    return cases


class SourceLaneFrameworkRuntimeTests(unittest.TestCase):
    def test_admitted_source_lanes_are_reported_from_shared_model(self) -> None:
        expected = ["local_file_markdown", "local_file_plain_text"]
        if pdf_lane_runtime_available():
            expected.append("local_file_pdf_text")
        expected.append("local_file_docx_text")
        expected.append("local_file_rtf_text")
        self.assertEqual(admitted_source_lanes(), expected)

    def test_ready_lanes_emit_common_schema_shape(self) -> None:
        for index, (fixture, media_type, expected_lane) in enumerate(ready_lane_cases()):
            result = emit_extraction_result_from_source_file(
                fixture,
                request_id=f"lane-shape-{index}",
                source_ref=f"lane-{index}",
                media_type=media_type,
            )

            assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
            self.assertEqual(result["state"], "ready")
            self.assertEqual(result["syntax_boundary"], "syntax_only")
            self.assertTrue(result["semantic_boundary_enforced"])
            self.assertEqual(result["completeness"]["status"], "complete")
            self.assertEqual(result["structures"]["metadata_fields"]["source_lane"], expected_lane)
            self.assertIn("source_hash", result["provenance"])
            self.assertIn("extractor_version", result["provenance"])
            self.assertNotIn("summary", result)
            self.assertNotIn("tags", result)
            self.assertNotIn("workflow_id", result)

    def test_ready_lanes_gate_retrieval_consistently(self) -> None:
        for index, (fixture, media_type, _expected_lane) in enumerate(ready_lane_cases()):
            result = emit_retrieval_package_from_source_file(
                fixture,
                request_id=f"lane-ret-{index}",
                source_ref=f"lane-ret-{index}",
                media_type=media_type,
            )

            assert_schema_valid(self, result, schema_name="retrieval-package.schema.json")
            self.assertEqual(result["state"], "ready")
            self.assertTrue(result["non_canonical"])
            self.assertTrue(result["non_semantic_default"])
            self.assertNotIn("ranking", result)
            self.assertNotIn("best_chunk", result)
            self.assertNotIn("recommendations", result)

    def test_media_type_mismatch_is_denied_for_each_ready_lane(self) -> None:
        for index, (fixture, _media_type, _expected_lane) in enumerate(ready_lane_cases()):
            result = emit_extraction_result_from_source_file(
                fixture,
                request_id=f"lane-mismatch-{index}",
                source_ref=f"lane-mismatch-{index}",
                media_type="application/octet-stream",
            )

            assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
            self.assertEqual(result["state"], "denied")
            self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_unreadable_paths_fail_closed_across_admitted_suffixes(self) -> None:
        suffixes = [".md", ".txt", ".docx", ".rtf"]
        if pdf_lane_runtime_available():
            suffixes.append(".pdf")

        for index, suffix in enumerate(suffixes):
            result = emit_extraction_result_from_source_file(
                ROOT / "tests/runtime/fixtures" / f"not-present-{index}{suffix}",
                request_id=f"lane-missing-{index}",
                source_ref=f"lane-missing-{index}",
            )

            assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
            self.assertEqual(result["state"], "unavailable")
            self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_non_pdf_ready_lanes_never_emit_partial_success(self) -> None:
        for index, (fixture, media_type, expected_lane) in enumerate(ready_lane_cases()):
            result = emit_extraction_result_from_source_file(
                fixture,
                request_id=f"lane-partial-{index}",
                source_ref=f"lane-partial-{index}",
                media_type=media_type,
            )

            assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
            if expected_lane != "pdf_text":
                self.assertNotEqual(result["state"], "partial_success")


if __name__ == "__main__":
    unittest.main()
