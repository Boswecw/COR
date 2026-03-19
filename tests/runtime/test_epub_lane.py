from __future__ import annotations

import copy
import tempfile
import unittest
import zipfile
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


EPUB_READY_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.epub"
EPUB_SCRIPTED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-scripted.epub"
EPUB_MISSING_PACKAGE_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-missing-package.epub"
EPUB_MALFORMED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-malformed.epub"
EPUB_SPINE_MISMATCH_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-spine-mismatch.epub"
EPUB_CORRUPT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-corrupt.epub"


def build_epub_intake_payload(path: Path) -> dict[str, object]:
    return copy.deepcopy(
        build_file_intake_payload(
            path,
            "application/epub+zip",
        )
    )


def write_epub(path: Path, members: dict[str, str]) -> None:
    with zipfile.ZipFile(path, "w") as archive:
        archive.writestr("mimetype", "application/epub+zip")
        for name, content in members.items():
            archive.writestr(name, content.lstrip())


class EpubLaneRuntimeTests(unittest.TestCase):
    def test_epub_intake_emits_ready_extraction_result(self) -> None:
        result = emit_extraction_result_from_intake_payload(build_epub_intake_payload(EPUB_READY_FIXTURE))

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["completeness"]["status"], "complete")
        self.assertEqual(result["structures"]["metadata_fields"]["source_lane"], "epub_text")
        self.assertEqual(len(result["structures"]["sections"]), 2)
        self.assertEqual(result["structures"]["tables_detected"], 1)
        self.assertEqual(
            {block["block_kind"] for block in result["structures"]["content_blocks"]},
            {"heading", "paragraph", "list", "table"},
        )

    def test_epub_extraction_is_deterministic(self) -> None:
        first = emit_extraction_result_from_source_file(
            EPUB_READY_FIXTURE,
            request_id="epub-det-001",
            source_ref="epub-det",
            media_type="application/epub+zip",
        )
        second = emit_extraction_result_from_source_file(
            EPUB_READY_FIXTURE,
            request_id="epub-det-001",
            source_ref="epub-det",
            media_type="application/epub+zip",
        )

        assert_schema_valid(self, first, schema_name="extraction-result.schema.json")
        assert_schema_valid(self, second, schema_name="extraction-result.schema.json")
        self.assertEqual(first["structures"], second["structures"])

    def test_scripted_epub_is_denied(self) -> None:
        result = emit_extraction_result_from_source_file(
            EPUB_SCRIPTED_FIXTURE,
            request_id="epub-denied-001",
            source_ref="epub-scripted",
            media_type="application/epub+zip",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")

    def test_nested_list_epub_is_ineligible(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            source_path = Path(tmpdir) / "nested-list.epub"
            write_epub(
                source_path,
                {
                    "META-INF/container.xml": """
                    <?xml version="1.0" encoding="UTF-8"?>
                    <container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
                      <rootfiles>
                        <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
                      </rootfiles>
                    </container>
                    """,
                    "OEBPS/content.opf": """
                    <?xml version="1.0" encoding="UTF-8"?>
                    <package xmlns="http://www.idpf.org/2007/opf" version="3.0">
                      <manifest>
                        <item id="chap1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
                      </manifest>
                      <spine>
                        <itemref idref="chap1"/>
                      </spine>
                    </package>
                    """,
                    "OEBPS/chapter1.xhtml": """
                    <?xml version="1.0" encoding="UTF-8"?>
                    <html xmlns="http://www.w3.org/1999/xhtml">
                      <body>
                        <ul>
                          <li>Outer item<ul><li>Nested item</li></ul></li>
                        </ul>
                      </body>
                    </html>
                    """,
                },
            )

            result = emit_extraction_result_from_source_file(
                source_path,
                request_id="epub-ineligible-001",
                source_ref="epub-ineligible",
                media_type="application/epub+zip",
            )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "ineligible_source")

    def test_non_linear_nav_document_is_skipped(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            source_path = Path(tmpdir) / "non-linear-nav.epub"
            write_epub(
                source_path,
                {
                    "META-INF/container.xml": """
                    <?xml version="1.0" encoding="UTF-8"?>
                    <container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
                      <rootfiles>
                        <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
                      </rootfiles>
                    </container>
                    """,
                    "OEBPS/content.opf": """
                    <?xml version="1.0" encoding="UTF-8"?>
                    <package xmlns="http://www.idpf.org/2007/opf" version="3.0">
                      <manifest>
                        <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
                        <item id="chap1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
                      </manifest>
                      <spine>
                        <itemref idref="nav" linear="no"/>
                        <itemref idref="chap1"/>
                      </spine>
                    </package>
                    """,
                    "OEBPS/nav.xhtml": """
                    <?xml version="1.0" encoding="UTF-8"?>
                    <html xmlns="http://www.w3.org/1999/xhtml">
                      <body>
                        <nav>
                          <ol>
                            <li>Table of Contents</li>
                          </ol>
                        </nav>
                      </body>
                    </html>
                    """,
                    "OEBPS/chapter1.xhtml": """
                    <?xml version="1.0" encoding="UTF-8"?>
                    <html xmlns="http://www.w3.org/1999/xhtml">
                      <body>
                        <h1>Chapter One</h1>
                        <p>Body paragraph.</p>
                      </body>
                    </html>
                    """,
                },
            )

            result = emit_extraction_result_from_source_file(
                source_path,
                request_id="epub-nav-001",
                source_ref="epub-nav",
                media_type="application/epub+zip",
            )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(
            [block["text"] for block in result["structures"]["content_blocks"]],
            ["Chapter One", "Body paragraph."],
        )

    def test_missing_package_epub_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            EPUB_MISSING_PACKAGE_FIXTURE,
            request_id="epub-missing-package-001",
            source_ref="epub-missing-package",
            media_type="application/epub+zip",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_malformed_epub_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            EPUB_MALFORMED_FIXTURE,
            request_id="epub-malformed-001",
            source_ref="epub-malformed",
            media_type="application/epub+zip",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_spine_mismatch_epub_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            EPUB_SPINE_MISMATCH_FIXTURE,
            request_id="epub-spine-001",
            source_ref="epub-spine",
            media_type="application/epub+zip",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_corrupt_epub_is_unavailable(self) -> None:
        result = emit_extraction_result_from_source_file(
            EPUB_CORRUPT_FIXTURE,
            request_id="epub-corrupt-001",
            source_ref="epub-corrupt",
            media_type="application/epub+zip",
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_ready_epub_extraction_is_retrieval_compatible(self) -> None:
        result = emit_retrieval_package_from_source_file(
            EPUB_READY_FIXTURE,
            request_id="epub-ret-001",
            source_ref="epub-ret",
            media_type="application/epub+zip",
        )

        assert_schema_valid(self, result, schema_name="retrieval-package.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["retrieval_profile"]["chunking_mode"], "section")
        self.assertEqual([chunk["ordinal"] for chunk in result["chunks"]], [0, 1])
        self.assertEqual({chunk["structure_kind"] for chunk in result["chunks"]}, {"section"})

    def test_epub_extraction_cli_is_ready_with_declared_media_type(self) -> None:
        exit_code, result = capture_cli_result(
            extraction_main,
            [
                "--source-path",
                str(EPUB_READY_FIXTURE),
                "--request-id",
                "epub-cli-001",
                "--source-ref",
                "epub-cli",
                "--media-type",
                "application/epub+zip",
            ],
        )

        assert_schema_valid(self, result, schema_name="extraction-result.schema.json")
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")

    def test_epub_retrieval_cli_is_ready_with_declared_media_type(self) -> None:
        exit_code, result = capture_cli_result(
            retrieval_main,
            [
                "--source-path",
                str(EPUB_READY_FIXTURE),
                "--request-id",
                "epub-ret-cli-001",
                "--source-ref",
                "epub-ret-cli",
                "--media-type",
                "application/epub+zip",
            ],
        )

        assert_schema_valid(self, result, schema_name="retrieval-package.schema.json")
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")


if __name__ == "__main__":
    unittest.main()
