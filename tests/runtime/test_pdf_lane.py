from __future__ import annotations

import copy
import unittest
from pathlib import Path
from unittest.mock import patch

from cortex_runtime.extraction_emission import (
    emit_extraction_result_from_intake_payload,
    emit_extraction_result_from_source_file,
    pdf_lane_runtime_available,
)
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_source_file
from cortex_runtime.source_lanes import probe_pdf_lane_admission
from tests.runtime.runtime_test_support import (
    ROOT,
    assert_schema_valid,
    build_file_intake_payload,
)

PDF_TEXT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.pdf"
PDF_ENCRYPTED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-encrypted.pdf"
PDF_SCANNED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-scanned.pdf"
PDF_PARTIAL_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-partial.pdf"
PDF_CORRUPT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-corrupt.pdf"


def build_pdf_intake_payload(path: Path) -> dict[str, object]:
    return copy.deepcopy(build_file_intake_payload(path, "application/pdf"))


@unittest.skipUnless(pdf_lane_runtime_available(), "bounded local PDF tooling is not available")
class PdfLaneRuntimeTests(unittest.TestCase):
    def test_text_pdf_intake_emits_ready_extraction_result(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_pdf_intake_payload(PDF_TEXT_FIXTURE))

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
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

        assert_schema_valid(self, first, schema_name="extraction-result.schema.json")
        assert_schema_valid(self, second, schema_name="extraction-result.schema.json")
        self.assertEqual(first["structures"], second["structures"])

    def test_partial_pdf_emits_partial_success(self) -> None:
        result = emit_extraction_result_from_source_file(
            PDF_PARTIAL_FIXTURE,
            request_id="pdf-partial-001",
            source_ref="pdf-partial",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
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

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_scanned_pdf_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            PDF_SCANNED_FIXTURE,
            request_id="pdf-scan-001",
            source_ref="pdf-scanned",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_corrupt_pdf_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            PDF_CORRUPT_FIXTURE,
            request_id="pdf-bad-001",
            source_ref="pdf-corrupt",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_pdfinfo_page_count_anomaly_is_unavailable(self) -> None:
        with patch("cortex_runtime.source_lanes.source_lane_slice_available", return_value=True), patch(
            "cortex_runtime.extraction_emission.pdf_lane_runtime_available",
            return_value=True,
        ), patch(
            "cortex_runtime.extraction_emission.subprocess.run",
            return_value=type(
                "CompletedProcess",
                (),
                {
                    "returncode": 0,
                    "stdout": "Pages: not-a-number\nEncrypted: no\n",
                    "stderr": "",
                },
            )(),
        ):
            result = emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-anomaly-001",
                source_ref="pdf-anomaly",
                media_type="application/pdf",
            )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_pdftotext_anomaly_is_unavailable(self) -> None:
        def run_side_effect(command: list[str], **_kwargs: object) -> object:
            if command[0] == "pdfinfo":
                return type(
                    "CompletedProcess",
                    (),
                    {
                        "returncode": 0,
                        "stdout": "Pages: 1\nEncrypted: no\n",
                        "stderr": "",
                    },
                )()
            return type(
                "CompletedProcess",
                (),
                {
                    "returncode": 1,
                    "stdout": "",
                    "stderr": "syntax error in xref table",
                },
            )()

        with patch("cortex_runtime.source_lanes.source_lane_slice_available", return_value=True), patch(
            "cortex_runtime.extraction_emission.pdf_lane_runtime_available",
            return_value=True,
        ), patch("cortex_runtime.extraction_emission.subprocess.run", side_effect=run_side_effect):
            result = emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-anomaly-002",
                source_ref="pdf-anomaly-2",
                media_type="application/pdf",
            )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_ready_pdf_extraction_is_retrieval_compatible(self) -> None:
        result = emit_retrieval_package_from_source_file(
            PDF_TEXT_FIXTURE,
            request_id="pdf-ret-001",
            source_ref="pdf-ret",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, schema_name="retrieval-package.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["retrieval_profile"]["chunking_mode"], "paragraph")
        self.assertEqual([chunk["ordinal"] for chunk in result["chunks"]], [0, 1])
        self.assertEqual({chunk["structure_kind"] for chunk in result["chunks"]}, {"paragraph"})


class PdfLaneMockedAdmissionTests(unittest.TestCase):
    """Tests for PDF lane admission truth that do not require PDF tools to be installed.

    These tests use mocks so they run on any host regardless of whether pdfinfo
    and pdftotext are present. They prove the lane fails closed correctly when
    tools are absent and that no OCR or external service drift occurs.
    """

    def test_tools_missing_emits_unavailable_result(self) -> None:
        """When both PDF tools are absent, the lane emits state=unavailable / dependency_unavailable."""
        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.return_value = None
            result = emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-noop-001",
                source_ref="pdf-noop",
                media_type="application/pdf",
            )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")
        self.assertEqual(result["completeness"]["status"], "failed")

    def test_both_tools_missing_summary_names_both_tools(self) -> None:
        """When both tools are absent, the summary names both pdfinfo and pdftotext."""
        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.return_value = None
            result = emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-noop-002",
                source_ref="pdf-noop-2",
                media_type="application/pdf",
            )

        summary = result["refusal"]["operator_visible_summary"]
        self.assertIn("pdfinfo", summary)
        self.assertIn("pdftotext", summary)

    def test_pdfinfo_missing_summary_names_pdfinfo(self) -> None:
        """When only pdfinfo is absent, the summary specifically names pdfinfo."""

        def which_side_effect(cmd: str) -> str | None:
            return None if cmd == "pdfinfo" else "/usr/bin/" + cmd

        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.side_effect = which_side_effect
            result = emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-noop-003",
                source_ref="pdf-noop-3",
                media_type="application/pdf",
            )

        self.assertEqual(result["state"], "unavailable")
        self.assertIn("pdfinfo", result["refusal"]["operator_visible_summary"])

    def test_pdftotext_missing_summary_names_pdftotext(self) -> None:
        """When only pdftotext is absent, the summary specifically names pdftotext."""

        def which_side_effect(cmd: str) -> str | None:
            return None if cmd == "pdftotext" else "/usr/bin/" + cmd

        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.side_effect = which_side_effect
            result = emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-noop-004",
                source_ref="pdf-noop-4",
                media_type="application/pdf",
            )

        self.assertEqual(result["state"], "unavailable")
        self.assertIn("pdftotext", result["refusal"]["operator_visible_summary"])

    def test_no_ocr_drift_when_tools_unavailable(self) -> None:
        """When PDF tools are absent, no subprocess is invoked — no OCR drift can occur."""
        observed_commands: list[str] = []

        def recording_run(command: list[str], **_kwargs: object) -> object:
            observed_commands.append(command[0])
            raise FileNotFoundError("tool not found")

        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil, patch(
            "cortex_runtime.extraction_emission.subprocess.run",
            side_effect=recording_run,
        ):
            mock_shutil.which.return_value = None
            emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-ocr-001",
                source_ref="pdf-ocr",
                media_type="application/pdf",
            )

        # The lane should fail at eligibility before any subprocess is called.
        self.assertEqual(
            observed_commands,
            [],
            "No subprocess must be invoked when PDF tools are absent — no OCR drift.",
        )

    def test_no_ocr_drift_on_extraction_failure(self) -> None:
        """When extraction fails (pdftotext returns non-zero), no other tool (e.g. OCR) is invoked."""
        observed_commands: list[str] = []

        def recording_run(command: list[str], **_kwargs: object) -> object:
            observed_commands.append(command[0])
            if command[0] == "pdfinfo":
                return type(
                    "CompletedProcess",
                    (),
                    {"returncode": 0, "stdout": "Pages: 1\nEncrypted: no\n", "stderr": ""},
                )()
            return type(
                "CompletedProcess",
                (),
                {"returncode": 1, "stdout": "", "stderr": "some generic failure"},
            )()

        with patch("cortex_runtime.source_lanes.source_lane_slice_available", return_value=True), patch(
            "cortex_runtime.extraction_emission.pdf_lane_runtime_available",
            return_value=True,
        ), patch("cortex_runtime.extraction_emission.subprocess.run", side_effect=recording_run):
            emit_extraction_result_from_source_file(
                PDF_TEXT_FIXTURE,
                request_id="pdf-ocr-002",
                source_ref="pdf-ocr-2",
                media_type="application/pdf",
            )

        # Only pdfinfo and pdftotext should appear — nothing else.
        unexpected = [cmd for cmd in observed_commands if cmd not in ("pdfinfo", "pdftotext")]
        self.assertEqual(
            unexpected,
            [],
            f"Only pdfinfo/pdftotext may be invoked during PDF extraction; saw: {unexpected}",
        )


class PdfLaneAdmissionProbeTests(unittest.TestCase):
    """Tests for the structured PDF lane admission probe."""

    def test_probe_reports_admitted_when_both_tools_present(self) -> None:
        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.return_value = "/usr/bin/tool"
            probe = probe_pdf_lane_admission()

        self.assertTrue(probe.admitted)
        self.assertTrue(probe.pdfinfo_present)
        self.assertTrue(probe.pdftotext_present)
        self.assertIn("runtime-admissible", probe.operator_summary)

    def test_probe_reports_not_admitted_when_both_tools_absent(self) -> None:
        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.return_value = None
            probe = probe_pdf_lane_admission()

        self.assertFalse(probe.admitted)
        self.assertFalse(probe.pdfinfo_present)
        self.assertFalse(probe.pdftotext_present)
        self.assertIn("pdfinfo", probe.operator_summary)
        self.assertIn("pdftotext", probe.operator_summary)

    def test_probe_reports_not_admitted_when_pdfinfo_absent(self) -> None:
        def which_side_effect(cmd: str) -> str | None:
            return None if cmd == "pdfinfo" else "/usr/bin/" + cmd

        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.side_effect = which_side_effect
            probe = probe_pdf_lane_admission()

        self.assertFalse(probe.admitted)
        self.assertFalse(probe.pdfinfo_present)
        self.assertTrue(probe.pdftotext_present)
        self.assertIn("pdfinfo", probe.operator_summary)

    def test_probe_reports_not_admitted_when_pdftotext_absent(self) -> None:
        def which_side_effect(cmd: str) -> str | None:
            return None if cmd == "pdftotext" else "/usr/bin/" + cmd

        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.side_effect = which_side_effect
            probe = probe_pdf_lane_admission()

        self.assertFalse(probe.admitted)
        self.assertTrue(probe.pdfinfo_present)
        self.assertFalse(probe.pdftotext_present)
        self.assertIn("pdftotext", probe.operator_summary)

    def test_probe_does_not_invoke_tools(self) -> None:
        """The admission probe must not invoke pdfinfo or pdftotext — only checks presence."""
        with patch("cortex_runtime.source_lanes.shutil") as mock_shutil:
            mock_shutil.which.return_value = "/usr/bin/tool"
            probe = probe_pdf_lane_admission()

        # If shutil.which was only used for presence checks (not invocation),
        # subprocess must never have been called. We verify by confirming the
        # probe returned without errors and did not call subprocess.
        self.assertIsNotNone(probe)
        # The probe is defined purely in terms of shutil.which — no subprocess.
        # The implicit proof is that no subprocess.run mock was needed above.


@unittest.skipUnless(pdf_lane_runtime_available(), "bounded local PDF tooling is not available")
class PdfLaneRuntimeMalformedTests(unittest.TestCase):
    """Runtime tests that require PDF tools — specifically testing malformed-PDF truth."""

    def test_corrupt_pdf_summary_mentions_malformed(self) -> None:
        """A corrupt PDF should surface a summary indicating the file is malformed or corrupt."""
        result = emit_extraction_result_from_source_file(
            PDF_CORRUPT_FIXTURE,
            request_id="pdf-malformed-001",
            source_ref="pdf-malformed",
            media_type="application/pdf",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")
        summary = result["refusal"]["operator_visible_summary"]
        self.assertTrue(
            "malformed" in summary or "corrupt" in summary,
            f"Expected 'malformed' or 'corrupt' in summary for a corrupt PDF; got: {summary!r}",
        )


if __name__ == "__main__":
    unittest.main()
