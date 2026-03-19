from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from cortex_runtime.extraction_emission import (
    emit_extraction_result_from_intake_payload,
    emit_extraction_result_from_source_file,
    pdf_lane_runtime_available,
)
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_source_file


ROOT = Path(__file__).resolve().parents[2]
EXTRACTION_SCHEMA_PATH = ROOT / "schemas/extraction-result.schema.json"
RETRIEVAL_SCHEMA_PATH = ROOT / "schemas/retrieval-package.schema.json"
VALID_INTAKE_FIXTURE = ROOT / "tests/contracts/fixtures/valid/intake-request-file-basic.json"
PDF_TEXT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.pdf"
PDF_ENCRYPTED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-encrypted.pdf"
PDF_SCANNED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-scanned.pdf"
PDF_PARTIAL_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-partial.pdf"
PDF_CORRUPT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-corrupt.pdf"


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


def build_pdf_intake_payload(path: Path) -> dict[str, object]:
    payload = load_json(VALID_INTAKE_FIXTURE)
    assert isinstance(payload, dict)
    payload = copy.deepcopy(payload)
    payload["sources"][0]["path"] = str(path)
    payload["sources"][0]["media_type"] = "application/pdf"
    payload["requested_artifact"] = "extraction_result"
    return payload


@unittest.skipUnless(pdf_lane_runtime_available(), "bounded local PDF tooling is not available")
class PdfLaneRuntimeTests(unittest.TestCase):
    def test_text_pdf_intake_emits_ready_extraction_result(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_pdf_intake_payload(PDF_TEXT_FIXTURE))

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["completeness"]["status"], "complete")
        self.assertEqual(result["structures"]["metadata_fields"]["source_lane"], "pdf_text")
        self.assertEqual(result["structures"]["metadata_fields"]["pdf_page_count"], "1")
        self.assertEqual(len(result["structures"]["content_blocks"]), 2)
        self.assertNotIn("sections", result["structures"])

    def test_pdf_extraction_is_deterministic(self) -> None:
        first = emit_extraction_result_from_source_file(
            PDF_TEXT_FIXTURE,
            request_id="pdf-det-001",
            source_ref="pdf-det",
            media_type="application/pdf",
        )
        second = emit_extraction_result_from_source_file(
            PDF_TEXT_FIXTURE,
            request_id="pdf-det-001",
            source_ref="pdf-det",
            media_type="application/pdf",
        )

        assert_schema_valid(self, first, validator=extraction_validator())
        assert_schema_valid(self, second, validator=extraction_validator())
        self.assertEqual(first["structures"], second["structures"])

    def test_partial_pdf_emits_partial_success(self) -> None:
        result = emit_extraction_result_from_source_file(
            PDF_PARTIAL_FIXTURE,
            request_id="pdf-partial-001",
            source_ref="pdf-partial",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "partial_success")
        self.assertEqual(result["completeness"]["status"], "incomplete")
        self.assertEqual(result["structures"]["metadata_fields"]["pdf_page_count"], "2")
        self.assertEqual(result["structures"]["metadata_fields"]["extractable_text_pages"], "1")

    def test_encrypted_pdf_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            PDF_ENCRYPTED_FIXTURE,
            request_id="pdf-enc-001",
            source_ref="pdf-encrypted",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_scanned_pdf_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            PDF_SCANNED_FIXTURE,
            request_id="pdf-scan-001",
            source_ref="pdf-scanned",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_corrupt_pdf_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            PDF_CORRUPT_FIXTURE,
            request_id="pdf-bad-001",
            source_ref="pdf-corrupt",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, validator=extraction_validator())
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_ready_pdf_extraction_is_retrieval_compatible(self) -> None:
        result = emit_retrieval_package_from_source_file(
            PDF_TEXT_FIXTURE,
            request_id="pdf-ret-001",
            source_ref="pdf-ret",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, validator=retrieval_validator())
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["retrieval_profile"]["chunking_mode"], "paragraph")
        self.assertEqual([chunk["ordinal"] for chunk in result["chunks"]], [0, 1])
        self.assertEqual({chunk["structure_kind"] for chunk in result["chunks"]}, {"paragraph"})


if __name__ == "__main__":
    unittest.main()
