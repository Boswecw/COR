from __future__ import annotations

import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from cortex_runtime.extraction_emission import (
    admitted_source_lanes,
    emit_extraction_result_from_source_file,
    pdf_lane_runtime_available,
)
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_source_file


ROOT = Path(__file__).resolve().parents[2]
EXTRACTION_SCHEMA_PATH = ROOT / "schemas/extraction-result.schema.json"
RETRIEVAL_SCHEMA_PATH = ROOT / "schemas/retrieval-package.schema.json"


def extraction_validator() -> Draft202012Validator:
    with EXTRACTION_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        return Draft202012Validator(json.load(handle))


def retrieval_validator() -> Draft202012Validator:
    with RETRIEVAL_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        return Draft202012Validator(json.load(handle))


def assert_schema_valid(
    testcase: unittest.TestCase,
    payload: dict[str, object],
    *,
    validator: Draft202012Validator,
) -> None:
    errors = sorted(
        validator.iter_errors(payload),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    testcase.assertEqual(
        [],
        [f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}" for error in errors],
    )


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
        expected = [
            "local_file_markdown",
            "local_file_plain_text",
        ]
        if pdf_lane_runtime_available():
            expected.append("local_file_pdf_text")
        expected.append("local_file_docx_text")
        self.assertEqual(admitted_source_lanes(), expected)

    def test_ready_lanes_emit_common_schema_shape(self) -> None:
        for index, (fixture, media_type, expected_lane) in enumerate(ready_lane_cases()):
            result = emit_extraction_result_from_source_file(
                fixture,
                request_id=f"lane-shape-{index}",
                source_ref=f"lane-{index}",
                media_type=media_type,
            )

            assert_schema_valid(self, result, validator=extraction_validator())
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

            assert_schema_valid(self, result, validator=retrieval_validator())
            self.assertEqual(result["state"], "ready")
            self.assertTrue(result["non_canonical"])
            self.assertTrue(result["non_semantic_default"])
            self.assertNotIn("ranking", result)
            self.assertNotIn("best_chunk", result)
            self.assertNotIn("recommendations", result)


if __name__ == "__main__":
    unittest.main()
