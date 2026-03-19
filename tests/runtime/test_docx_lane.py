from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from cortex_runtime.extraction_emission import emit_extraction_result_from_intake_payload
from cortex_runtime.extraction_emission import emit_extraction_result_from_source_file
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_source_file


ROOT = Path(__file__).resolve().parents[2]
EXTRACTION_SCHEMA_PATH = ROOT / "schemas/extraction-result.schema.json"
RETRIEVAL_SCHEMA_PATH = ROOT / "schemas/retrieval-package.schema.json"
VALID_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/valid/intake-request-file-basic.json"
DOCX_READY_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.docx"
DOCX_REVIEWED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-reviewed.docx"
DOCX_CORRUPT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-corrupt.docx"


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


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


def build_docx_intake_payload(path: Path) -> dict[str, object]:
    payload = load_json(VALID_INTAKE_FIXTURE)
    assert isinstance(payload, dict)
    payload = copy.deepcopy(payload)
    payload["sources"][0]["path"] = str(path)
    payload["sources"][0]["media_type"] = (
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    payload["requested_artifact"] = "extraction_result"
    return payload


class DocxLaneRuntimeTests(unittest.TestCase):
    def test_docx_intake_emits_ready_extraction_result(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_docx_intake_payload(DOCX_READY_FIXTURE))

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["completeness"]["status"], "complete")
        self.assertEqual(result["structures"]["metadata_fields"]["source_lane"], "docx_text")
        self.assertEqual(len(result["structures"]["sections"]), 2)
        self.assertEqual(result["structures"]["tables_detected"], 1)
        self.assertEqual(
            {block["block_kind"] for block in result["structures"]["content_blocks"]},
            {"heading", "paragraph", "list", "table"},
        )

    def test_docx_extraction_is_deterministic(self) -> None:
        first = emit_extraction_result_from_source_file(
            DOCX_READY_FIXTURE,
            request_id="docx-det-001",
            source_ref="docx-det",
            media_type="application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )
        second = emit_extraction_result_from_source_file(
            DOCX_READY_FIXTURE,
            request_id="docx-det-001",
            source_ref="docx-det",
            media_type="application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )

        assert_schema_valid(self, first, validator=extraction_validator())
        assert_schema_valid(self, second, validator=extraction_validator())
        self.assertEqual(first["structures"], second["structures"])

    def test_reviewed_docx_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            DOCX_REVIEWED_FIXTURE,
            request_id="docx-denied-001",
            source_ref="docx-reviewed",
            media_type="application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_corrupt_docx_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            DOCX_CORRUPT_FIXTURE,
            request_id="docx-bad-001",
            source_ref="docx-corrupt",
            media_type="application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_ready_docx_extraction_is_retrieval_compatible(self) -> None:
        result = emit_retrieval_package_from_source_file(
            DOCX_READY_FIXTURE,
            request_id="docx-ret-001",
            source_ref="docx-ret",
            media_type="application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )

        assert_schema_valid(self, result, validator=retrieval_validator())
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["retrieval_profile"]["chunking_mode"], "section")
        self.assertEqual([chunk["ordinal"] for chunk in result["chunks"]], [0, 1])
        self.assertEqual({chunk["structure_kind"] for chunk in result["chunks"]}, {"section"})


if __name__ == "__main__":
    unittest.main()
