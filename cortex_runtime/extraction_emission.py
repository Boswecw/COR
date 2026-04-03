from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import zipfile
from datetime import UTC, datetime
from functools import lru_cache
from pathlib import Path
from typing import Any
from xml.etree import ElementTree as ET

from jsonschema import Draft202012Validator  # type: ignore[import-untyped]

from cortex_runtime.epub_lane import EpubDeniedError, EpubUnavailableError, extract_epub_surface
from cortex_runtime.intake_validation import validate_intake_payload
from cortex_runtime.odt_lane import OdtDeniedError, OdtUnavailableError, extract_odt_surface
from cortex_runtime.rtf_lane import RtfDeniedError, RtfUnavailableError, extract_rtf_paragraphs
from cortex_runtime.source_lanes import (
    DOCX_TEXT_LANE,
    EPUB_TEXT_LANE,
    MARKDOWN_LANE,
    ODT_TEXT_LANE,
    PDF_INFO_COMMAND,
    PDF_TEXT_LANE,
    PDF_TO_TEXT_COMMAND,
    PLAIN_TEXT_LANE,
    RTF_TEXT_LANE,
    SourceLaneSpec,
    admitted_source_lanes as _admitted_source_lanes,
    configured_supported_media_types,
    configured_supported_suffixes,
    docx_lane_runtime_available as _docx_lane_runtime_available,
    lane_eligibility_for_path,
    pdf_lane_runtime_available as _pdf_lane_runtime_available,
    probe_pdf_lane_admission as _probe_pdf_lane_admission,
)


ROOT = Path(__file__).resolve().parent.parent
EXTRACTION_SCHEMA_PATH = ROOT / "schemas/extraction-result.schema.json"
EXTRACTION_SCHEMA_REF = "schemas/extraction-result.schema.json"
EXTRACTOR_VERSION = "slice9.syntax_only.1"
SUPPORTED_SUFFIXES = configured_supported_suffixes()
SUPPORTED_MEDIA_TYPES = configured_supported_media_types()
SOURCE_LANE_IDS = {
    MARKDOWN_LANE.suffix: MARKDOWN_LANE.lane_id,
    PLAIN_TEXT_LANE.suffix: PLAIN_TEXT_LANE.lane_id,
    PDF_TEXT_LANE.suffix: PDF_TEXT_LANE.lane_id,
    RTF_TEXT_LANE.suffix: RTF_TEXT_LANE.lane_id,
    DOCX_TEXT_LANE.suffix: DOCX_TEXT_LANE.lane_id,
    ODT_TEXT_LANE.suffix: ODT_TEXT_LANE.lane_id,
    EPUB_TEXT_LANE.suffix: EPUB_TEXT_LANE.lane_id,
}
WORD_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
WORD_TAG = f"{{{WORD_NS}}}"
WORD_VALUE = f"{{{WORD_NS}}}val"


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _timestamp_from_epoch(epoch_seconds: float) -> str:
    return datetime.fromtimestamp(epoch_seconds, tz=UTC).isoformat().replace("+00:00", "Z")


def pdf_lane_runtime_available() -> bool:
    return _pdf_lane_runtime_available()


def docx_lane_runtime_available() -> bool:
    return _docx_lane_runtime_available()


def admitted_source_lanes() -> list[str]:
    return _admitted_source_lanes()


def _artifact_id(request_id: str, source_ref: str) -> str:
    digest = hashlib.sha256(f"{request_id}:{source_ref}".encode("utf-8")).hexdigest()[:16]
    return f"extract-{digest}"


def _source_hash(raw_bytes: bytes) -> str:
    return f"sha256:{hashlib.sha256(raw_bytes).hexdigest()}"


def _fallback_source_hash(request_id: str, source_ref: str, reason_class: str) -> str:
    return f"unavailable:{request_id}:{source_ref}:{reason_class}"


def _extract_request_id(payload: Any) -> str:
    if isinstance(payload, dict):
        request_id = payload.get("request_id")
        if isinstance(request_id, str) and request_id:
            return request_id
    return "unknown_request"


def _extract_source_ref(payload: Any) -> str:
    if isinstance(payload, dict):
        sources = payload.get("sources")
        if isinstance(sources, list) and sources:
            first = sources[0]
            if isinstance(first, dict):
                source_id = first.get("source_id")
                if isinstance(source_id, str) and source_id:
                    return source_id
    return "unknown_source"


def _schema_error_messages(result: dict[str, Any]) -> list[str]:
    validator = _extraction_validator()
    errors = sorted(
        validator.iter_errors(result),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    return [
        f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}"
        for error in errors
    ]


def _validate_or_fallback(
    candidate: dict[str, Any],
    *,
    request_id: str,
    source_ref: str,
) -> dict[str, Any]:
    if not _schema_error_messages(candidate):
        return candidate

    fallback = _build_failure_result(
        request_id=request_id,
        source_ref=source_ref,
        state="unavailable",
        reason_class="dependency_unavailable",
        summary="Extraction emission failed closed because the bounded extraction contract could not be satisfied.",
    )
    fallback_errors = _schema_error_messages(fallback)
    if fallback_errors:
        raise RuntimeError(
            "fallback extraction result violated schema: " + "; ".join(fallback_errors)
        )
    return fallback


def _build_failure_result(
    *,
    request_id: str,
    source_ref: str,
    state: str,
    reason_class: str,
    summary: str,
    source_hash: str | None = None,
    source_modified_at: str | None = None,
    byte_count: int | None = None,
) -> dict[str, Any]:
    provenance: dict[str, Any] = {
        "source_hash": source_hash or _fallback_source_hash(request_id, source_ref, reason_class),
        "extractor_version": EXTRACTOR_VERSION,
    }
    if source_modified_at is not None:
        provenance["source_modified_at"] = source_modified_at
    if byte_count is not None:
        provenance["byte_count"] = byte_count

    return {
        "artifact_id": _artifact_id(request_id, source_ref),
        "request_id": request_id,
        "source_ref": source_ref,
        "state": state,
        "syntax_boundary": "syntax_only",
        "semantic_boundary_enforced": True,
        "provenance": provenance,
        "completeness": {
            "status": "failed",
            "operator_visible_summary": summary,
        },
        "refusal": {
            "reason_class": reason_class,
            "operator_visible_summary": summary,
        },
        "extracted_at": _utc_now(),
    }


def _emit_failure_result(
    *,
    request_id: str,
    source_ref: str,
    state: str,
    reason_class: str,
    summary: str,
    source_hash: str | None = None,
    source_modified_at: str | None = None,
    byte_count: int | None = None,
) -> dict[str, Any]:
    return _validate_or_fallback(
        _build_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state=state,
            reason_class=reason_class,
            summary=summary,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        ),
        request_id=request_id,
        source_ref=source_ref,
    )


def _build_success_result(
    *,
    request_id: str,
    source_ref: str,
    state: str,
    completeness_status: str,
    completeness_summary: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
    structures: dict[str, Any],
) -> dict[str, Any]:
    return {
        "artifact_id": _artifact_id(request_id, source_ref),
        "request_id": request_id,
        "source_ref": source_ref,
        "state": state,
        "syntax_boundary": "syntax_only",
        "semantic_boundary_enforced": True,
        "provenance": {
            "source_hash": source_hash,
            "extractor_version": EXTRACTOR_VERSION,
            "source_modified_at": source_modified_at,
            "byte_count": byte_count,
        },
        "completeness": {
            "status": completeness_status,
            "operator_visible_summary": completeness_summary,
        },
        "structures": structures,
        "extracted_at": _utc_now(),
    }


def _emit_success_result(
    *,
    request_id: str,
    source_ref: str,
    state: str,
    completeness_status: str,
    completeness_summary: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
    structures: dict[str, Any],
) -> dict[str, Any]:
    return _validate_or_fallback(
        _build_success_result(
            request_id=request_id,
            source_ref=source_ref,
            state=state,
            completeness_status=completeness_status,
            completeness_summary=completeness_summary,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
            structures=structures,
        ),
        request_id=request_id,
        source_ref=source_ref,
    )


def _metadata_fields(source_path: Path, lane: SourceLaneSpec) -> dict[str, str]:
    return {
        "file_name": source_path.name,
        "file_extension": source_path.suffix or "<none>",
        "source_lane": lane.metadata_id,
    }


def _build_sections_from_heading_positions(
    blocks: list[dict[str, Any]],
    heading_positions: list[tuple[int, int, str]],
) -> list[dict[str, Any]]:
    sections: list[dict[str, Any]] = []
    for index, (block_index, level, title) in enumerate(heading_positions):
        next_index = (
            heading_positions[index + 1][0]
            if index + 1 < len(heading_positions)
            else len(blocks)
        )
        sections.append(
            {
                "section_id": f"sec-{index + 1}",
                "heading": title,
                "level": level,
                "block_count": max(1, next_index - block_index),
            }
        )
    return sections


def _parse_markdown_heading(line: str) -> tuple[int, str] | None:
    if not line.startswith("#"):
        return None

    marker, _, title = line.partition(" ")
    if not title or len(marker) > 8 or any(char != "#" for char in marker):
        return None

    return len(marker), title.strip()


def _build_text_structures(
    text: str,
    source_path: Path,
    *,
    lane: SourceLaneSpec,
    markdown_headings_allowed: bool,
) -> dict[str, Any]:
    blocks: list[dict[str, Any]] = []
    heading_positions: list[tuple[int, int, str]] = []
    paragraph_lines: list[str] = []
    block_counter = 0
    tables_detected = 0

    def flush_paragraph() -> None:
        nonlocal block_counter
        if not paragraph_lines:
            return

        paragraph_text = "\n".join(paragraph_lines).strip()
        paragraph_lines.clear()
        if not paragraph_text:
            return
        if len(paragraph_text) > 20000:
            raise ValueError("literal content exceeds bounded extraction limits")

        block_counter += 1
        blocks.append(
            {
                "block_id": f"blk-{block_counter}",
                "block_kind": "paragraph",
                "text": paragraph_text,
            }
        )

    for raw_line in text.splitlines():
        if raw_line.count("|") >= 2:
            tables_detected += 1

        stripped = raw_line.strip()
        if not stripped:
            flush_paragraph()
            continue

        heading = _parse_markdown_heading(stripped) if markdown_headings_allowed else None
        if heading is not None:
            flush_paragraph()
            level, title = heading
            if len(title) > 300:
                raise ValueError("heading exceeds bounded extraction limits")

            block_counter += 1
            blocks.append(
                {
                    "block_id": f"blk-{block_counter}",
                    "block_kind": "heading",
                    "text": title,
                }
            )
            heading_positions.append((len(blocks) - 1, level, title))
            continue

        paragraph_lines.append(stripped)

    flush_paragraph()

    structures: dict[str, Any] = {
        "tables_detected": tables_detected,
        "metadata_fields": _metadata_fields(source_path, lane),
        "content_blocks": blocks,
    }

    if heading_positions:
        structures["sections"] = _build_sections_from_heading_positions(blocks, heading_positions)

    return structures


def _build_paragraph_only_structures(
    paragraphs: list[str],
    source_path: Path,
    *,
    lane: SourceLaneSpec,
) -> dict[str, Any]:
    blocks: list[dict[str, Any]] = []
    for index, paragraph_text in enumerate(paragraphs, start=1):
        if len(paragraph_text) > 20000:
            raise ValueError("literal content exceeds bounded extraction limits")
        blocks.append(
            {
                "block_id": f"blk-{index}",
                "block_kind": "paragraph",
                "text": paragraph_text,
            }
        )

    return {
        "tables_detected": 0,
        "metadata_fields": _metadata_fields(source_path, lane),
        "content_blocks": blocks,
    }


def _build_odt_structures(odt_surface: dict[str, Any], source_path: Path) -> dict[str, Any]:
    blocks: list[dict[str, Any]] = []
    heading_positions: list[tuple[int, int, str]] = []
    block_counter = 0

    for raw_block in odt_surface.get("blocks", []):
        if not isinstance(raw_block, dict):
            raise ValueError("ODT block surface is malformed")
        block_kind = raw_block.get("block_kind")
        text = raw_block.get("text")
        if block_kind not in {"heading", "paragraph", "list", "table"} or not isinstance(text, str) or not text:
            raise ValueError("ODT block surface is malformed")
        if len(text) > 20000:
            raise ValueError("literal content exceeds bounded extraction limits")

        block_counter += 1
        blocks.append(
            {
                "block_id": f"blk-{block_counter}",
                "block_kind": block_kind,
                "text": text,
            }
        )

        if block_kind == "heading":
            level = raw_block.get("level")
            if not isinstance(level, int) or level < 1 or level > 8:
                raise ValueError("ODT heading level is malformed")
            heading_positions.append((len(blocks) - 1, level, text))

    structures: dict[str, Any] = {
        "tables_detected": int(odt_surface.get("tables_detected", 0)),
        "metadata_fields": _metadata_fields(source_path, ODT_TEXT_LANE),
        "content_blocks": blocks,
    }
    if heading_positions:
        structures["sections"] = _build_sections_from_heading_positions(blocks, heading_positions)
    return structures


def _build_epub_structures(epub_surface: dict[str, Any], source_path: Path) -> dict[str, Any]:
    blocks: list[dict[str, Any]] = []
    heading_positions: list[tuple[int, int, str]] = []
    block_counter = 0

    for raw_block in epub_surface.get("blocks", []):
        if not isinstance(raw_block, dict):
            raise ValueError("EPUB block surface is malformed")
        block_kind = raw_block.get("block_kind")
        text = raw_block.get("text")
        if block_kind not in {"heading", "paragraph", "list", "table"} or not isinstance(text, str) or not text:
            raise ValueError("EPUB block surface is malformed")
        if len(text) > 20000:
            raise ValueError("literal content exceeds bounded extraction limits")

        block_counter += 1
        blocks.append(
            {
                "block_id": f"blk-{block_counter}",
                "block_kind": block_kind,
                "text": text,
            }
        )

        if block_kind == "heading":
            level = raw_block.get("level")
            if not isinstance(level, int) or level < 1 or level > 8:
                raise ValueError("EPUB heading level is malformed")
            heading_positions.append((len(blocks) - 1, level, text))

    structures: dict[str, Any] = {
        "tables_detected": int(epub_surface.get("tables_detected", 0)),
        "metadata_fields": _metadata_fields(source_path, EPUB_TEXT_LANE),
        "content_blocks": blocks,
    }
    if heading_positions:
        structures["sections"] = _build_sections_from_heading_positions(blocks, heading_positions)
    return structures


def _pdf_command_failed_due_to_password(stderr: str) -> bool:
    lowered = stderr.lower()
    return "incorrect password" in lowered or ("password" in lowered and "error" in lowered)


def _pdf_stderr_indicates_malformed(stderr: str) -> bool:
    """Return True when PDF tool stderr indicates a malformed or corrupt PDF structure.

    Matches only patterns unambiguously produced by file-structure errors —
    not generic tool failures, permission issues, or resource errors.
    """
    lowered = stderr.lower()
    return any(
        phrase in lowered
        for phrase in (
            "syntax error",
            "syntax warning",
            "couldn't read xref",
            "invalid xref",
            "damaged",
        )
    )


def _parse_pdfinfo(output: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for line in output.splitlines():
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        parsed[key.strip()] = value.strip()
    return parsed


def _build_pdf_structures(text: str, source_path: Path, page_count: int) -> tuple[dict[str, Any], int]:
    blocks: list[dict[str, Any]] = []
    block_counter = 0
    tables_detected = 0
    extractable_text_pages = 0

    def flush_paragraph(paragraph_lines: list[str]) -> None:
        nonlocal block_counter
        if not paragraph_lines:
            return

        paragraph_text = "\n".join(paragraph_lines).strip()
        paragraph_lines.clear()
        if not paragraph_text:
            return
        if len(paragraph_text) > 20000:
            raise ValueError("literal content exceeds bounded extraction limits")

        block_counter += 1
        blocks.append(
            {
                "block_id": f"blk-{block_counter}",
                "block_kind": "paragraph",
                "text": paragraph_text,
            }
        )

    for page_text in text.split("\f"):
        if not page_text.strip():
            continue

        extractable_text_pages += 1
        paragraph_lines: list[str] = []
        for raw_line in page_text.splitlines():
            if raw_line.count("|") >= 2:
                tables_detected += 1

            stripped = raw_line.strip()
            if not stripped:
                flush_paragraph(paragraph_lines)
                continue

            paragraph_lines.append(stripped)

        flush_paragraph(paragraph_lines)

    metadata_fields = _metadata_fields(source_path, PDF_TEXT_LANE)
    metadata_fields["pdf_page_count"] = str(page_count)
    metadata_fields["extractable_text_pages"] = str(extractable_text_pages)
    return {
        "tables_detected": tables_detected,
        "metadata_fields": metadata_fields,
        "content_blocks": blocks,
    }, extractable_text_pages


def _word_tag(name: str) -> str:
    return f"{WORD_TAG}{name}"


def _docx_literal_text(element: ET.Element) -> str:
    pieces: list[str] = []
    for descendant in element.iter():
        if descendant.tag == _word_tag("t") and descendant.text:
            pieces.append(descendant.text)
        elif descendant.tag == _word_tag("tab"):
            pieces.append("\t")
        elif descendant.tag in {_word_tag("br"), _word_tag("cr")}:
            pieces.append("\n")
    return "".join(pieces).strip()


def _docx_paragraph_style_id(paragraph: ET.Element) -> str | None:
    ppr = paragraph.find(_word_tag("pPr"))
    if ppr is None:
        return None
    pstyle = ppr.find(_word_tag("pStyle"))
    if pstyle is None:
        return None
    style_id = pstyle.get(WORD_VALUE)
    return style_id if style_id else None


def _docx_heading_level(style_id: str | None) -> int | None:
    if not style_id:
        return None
    normalized = style_id.replace(" ", "").lower()
    if not normalized.startswith("heading"):
        return None
    suffix = normalized[len("heading") :]
    if not suffix.isdigit():
        return None
    level = int(suffix)
    if level < 1 or level > 8:
        return None
    return level


def _docx_is_list_paragraph(paragraph: ET.Element, style_id: str | None) -> bool:
    ppr = paragraph.find(_word_tag("pPr"))
    if ppr is not None and ppr.find(_word_tag("numPr")) is not None:
        return True
    if not style_id:
        return False
    normalized = style_id.replace(" ", "").lower()
    return normalized.startswith("list")


def _docx_contains_review_markup(document_root: ET.Element) -> bool:
    denied_tags = (
        "ins",
        "del",
        "moveFrom",
        "moveTo",
        "commentRangeStart",
        "commentRangeEnd",
        "commentReference",
    )
    return any(document_root.find(f".//{_word_tag(tag)}") is not None for tag in denied_tags)


def _docx_contains_comments(docx_archive: zipfile.ZipFile) -> bool:
    try:
        comments_xml = docx_archive.read("word/comments.xml")
    except KeyError:
        return False
    try:
        comments_root = ET.fromstring(comments_xml)
    except ET.ParseError:
        return True
    return comments_root.find(f".//{_word_tag('comment')}") is not None


def _docx_table_text(table: ET.Element) -> str:
    rows: list[str] = []
    for row in table.findall(_word_tag("tr")):
        cells: list[str] = []
        for cell in row.findall(_word_tag("tc")):
            if cell.find(f".//{_word_tag('tbl')}") is not None:
                raise ValueError("nested DOCX tables are outside the bounded lane")
            paragraphs: list[str] = []
            for paragraph in cell.findall(_word_tag("p")):
                text = _docx_literal_text(paragraph)
                if text:
                    paragraphs.append(text)
            cells.append(" / ".join(paragraphs).strip())
        row_text = " | ".join(cells).strip()
        if row_text:
            rows.append(row_text)
    table_text = "\n".join(rows).strip()
    if len(table_text) > 20000:
        raise ValueError("literal content exceeds bounded extraction limits")
    return table_text


def _build_docx_structures(docx_archive: zipfile.ZipFile, source_path: Path) -> dict[str, Any]:
    document_xml = docx_archive.read("word/document.xml")
    document_root = ET.fromstring(document_xml)
    if _docx_contains_review_markup(document_root) or _docx_contains_comments(docx_archive):
        raise PermissionError("review markup is outside the bounded DOCX lane")

    body = document_root.find(_word_tag("body"))
    if body is None:
        raise zipfile.BadZipFile("missing word/document.xml body")

    blocks: list[dict[str, Any]] = []
    heading_positions: list[tuple[int, int, str]] = []
    block_counter = 0
    tables_detected = 0

    for child in body:
        if child.tag == _word_tag("p"):
            style_id = _docx_paragraph_style_id(child)
            paragraph_text = _docx_literal_text(child)
            if not paragraph_text:
                continue
            if len(paragraph_text) > 20000:
                raise ValueError("literal content exceeds bounded extraction limits")

            heading_level = _docx_heading_level(style_id)
            if heading_level is not None:
                block_counter += 1
                blocks.append(
                    {
                        "block_id": f"blk-{block_counter}",
                        "block_kind": "heading",
                        "text": paragraph_text,
                    }
                )
                heading_positions.append((len(blocks) - 1, heading_level, paragraph_text))
                continue

            block_kind = "list" if _docx_is_list_paragraph(child, style_id) else "paragraph"
            block_counter += 1
            blocks.append(
                {
                    "block_id": f"blk-{block_counter}",
                    "block_kind": block_kind,
                    "text": paragraph_text,
                }
            )
            continue

        if child.tag == _word_tag("tbl"):
            table_text = _docx_table_text(child)
            if not table_text:
                continue
            block_counter += 1
            tables_detected += 1
            blocks.append(
                {
                    "block_id": f"blk-{block_counter}",
                    "block_kind": "table",
                    "text": table_text,
                }
            )

    structures: dict[str, Any] = {
        "tables_detected": tables_detected,
        "metadata_fields": _metadata_fields(source_path, DOCX_TEXT_LANE),
        "content_blocks": blocks,
    }
    if heading_positions:
        structures["sections"] = _build_sections_from_heading_positions(blocks, heading_positions)
    return structures


@lru_cache(maxsize=1)
def _extraction_validator() -> Draft202012Validator:
    with EXTRACTION_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        schema = json.load(handle)
    return Draft202012Validator(schema)


def _emit_pdf_extraction_result(
    source_path: Path,
    *,
    request_id: str,
    source_ref: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
) -> dict[str, Any]:
    if not pdf_lane_runtime_available():
        probe = _probe_pdf_lane_admission()
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary=probe.operator_summary,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    pdfinfo_result = subprocess.run(
        [PDF_INFO_COMMAND, str(source_path)],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if pdfinfo_result.returncode != 0:
        if _pdf_command_failed_due_to_password(pdfinfo_result.stderr):
            return _emit_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="unsupported_source_type",
                summary="Extraction is denied because encrypted PDFs are outside the bounded local text-only PDF lane.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            )

        if _pdf_stderr_indicates_malformed(pdfinfo_result.stderr):
            return _emit_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="unavailable",
                reason_class="dependency_unavailable",
                summary="Extraction is unavailable because the PDF file is malformed or corrupt and cannot be read through the bounded local PDF lane.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            )

        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the bounded PDF metadata tool (pdfinfo) failed on this file.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    pdfinfo = _parse_pdfinfo(pdfinfo_result.stdout)
    encrypted = pdfinfo.get("Encrypted", "").lower().startswith("yes")
    if encrypted:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because encrypted PDFs are outside the bounded local text-only PDF lane.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    try:
        page_count = int(pdfinfo.get("Pages", "0"))
    except ValueError:
        page_count = 0
    if page_count <= 0:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the bounded PDF lane could not establish a trustworthy page count.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    pdftotext_result = subprocess.run(
        [PDF_TO_TEXT_COMMAND, "-enc", "UTF-8", str(source_path), "-"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if pdftotext_result.returncode != 0:
        if _pdf_command_failed_due_to_password(pdftotext_result.stderr):
            return _emit_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="unsupported_source_type",
                summary="Extraction is denied because encrypted PDFs are outside the bounded local text-only PDF lane.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            )

        if _pdf_stderr_indicates_malformed(pdftotext_result.stderr):
            return _emit_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="unavailable",
                reason_class="dependency_unavailable",
                summary="Extraction is unavailable because the PDF text layer is malformed or corrupt and cannot be read through the bounded local PDF lane.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            )

        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the bounded PDF text-extraction tool (pdftotext) failed on this file.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    try:
        structures, extractable_text_pages = _build_pdf_structures(
            pdftotext_result.stdout,
            source_path,
            page_count,
        )
    except ValueError:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="ineligible_source",
            summary="Extraction is denied because the PDF exceeds the bounded literal extraction limits for this slice.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if not structures["content_blocks"]:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because the PDF has no extractable text layer and OCR or image interpretation is not allowed.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    completeness_status = "complete"
    state = "ready"
    completeness_summary = "Syntax-only text-layer extraction completed for a bounded local PDF source."
    if extractable_text_pages < page_count:
        completeness_status = "incomplete"
        state = "partial_success"
        completeness_summary = (
            "Syntax-only PDF extraction completed for extractable pages, but one or more pages had no extractable text layer."
        )

    return _emit_success_result(
        request_id=request_id,
        source_ref=source_ref,
        state=state,
        completeness_status=completeness_status,
        completeness_summary=completeness_summary,
        source_hash=source_hash,
        source_modified_at=source_modified_at,
        byte_count=byte_count,
        structures=structures,
    )


def _emit_docx_extraction_result(
    source_path: Path,
    *,
    request_id: str,
    source_ref: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
) -> dict[str, Any]:
    try:
        with zipfile.ZipFile(source_path, "r") as docx_archive:
            try:
                structures = _build_docx_structures(docx_archive, source_path)
            except PermissionError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="denied",
                    reason_class="unsupported_source_type",
                    summary="Extraction is denied because DOCX review markup, comments, or tracked changes are outside the bounded DOCX lane.",
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
            except ValueError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="denied",
                    reason_class="ineligible_source",
                    summary="Extraction is denied because the DOCX structure exceeds the bounded deterministic recovery limits for this slice.",
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
    except (OSError, KeyError, zipfile.BadZipFile, ET.ParseError):
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the DOCX package could not be read through the bounded local DOCX lane.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if not structures["content_blocks"]:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because the DOCX package has no bounded extractable text structures.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    return _emit_success_result(
        request_id=request_id,
        source_ref=source_ref,
        state="ready",
        completeness_status="complete",
        completeness_summary="Syntax-only extraction completed for a bounded local DOCX source.",
        source_hash=source_hash,
        source_modified_at=source_modified_at,
        byte_count=byte_count,
        structures=structures,
    )


def _emit_text_extraction_result(
    source_path: Path,
    raw_bytes: bytes,
    *,
    request_id: str,
    source_ref: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
    lane: SourceLaneSpec,
) -> dict[str, Any]:
    try:
        text = raw_bytes.decode("utf-8")
    except UnicodeDecodeError:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because only UTF-8 text-like input is supported in this bounded text lane.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    try:
        structures = _build_text_structures(
            text,
            source_path,
            lane=lane,
            markdown_headings_allowed=lane.lane_id == MARKDOWN_LANE.lane_id,
        )
    except ValueError:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="ineligible_source",
            summary="Extraction is denied because the source exceeds the bounded literal extraction limits for this slice.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if not structures["content_blocks"]:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because the text source has no bounded extractable text structures.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    return _emit_success_result(
        request_id=request_id,
        source_ref=source_ref,
        state="ready",
        completeness_status="complete",
        completeness_summary="Syntax-only extraction completed for a bounded local text source.",
        source_hash=source_hash,
        source_modified_at=source_modified_at,
        byte_count=byte_count,
        structures=structures,
    )


def _emit_rtf_extraction_result(
    source_path: Path,
    raw_bytes: bytes,
    *,
    request_id: str,
    source_ref: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
) -> dict[str, Any]:
    try:
        paragraphs = extract_rtf_paragraphs(raw_bytes)
    except RtfDeniedError:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because this RTF source uses annotation, review, field, media, or other rich destinations outside the bounded paragraph-only RTF lane.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )
    except RtfUnavailableError:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the RTF source could not be parsed safely enough to trust bounded paragraph recovery.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if not paragraphs:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because the RTF source has no bounded extractable paragraph text.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    try:
        structures = _build_paragraph_only_structures(paragraphs, source_path, lane=RTF_TEXT_LANE)
    except ValueError:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="ineligible_source",
            summary="Extraction is denied because the RTF source exceeds the bounded literal extraction limits for this slice.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    return _emit_success_result(
        request_id=request_id,
        source_ref=source_ref,
        state="ready",
        completeness_status="complete",
        completeness_summary="Syntax-only extraction completed for a bounded local RTF source.",
        source_hash=source_hash,
        source_modified_at=source_modified_at,
        byte_count=byte_count,
        structures=structures,
    )


def _emit_odt_extraction_result(
    source_path: Path,
    *,
    request_id: str,
    source_ref: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
) -> dict[str, Any]:
    try:
        with zipfile.ZipFile(source_path, "r") as odt_archive:
            try:
                odt_surface = extract_odt_surface(odt_archive)
                structures = _build_odt_structures(odt_surface, source_path)
            except OdtDeniedError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="denied",
                    reason_class="unsupported_source_type",
                    summary="Extraction is denied because ODT annotations, tracked changes, embedded objects, or other out-of-lane package structures are outside the bounded ODT lane.",
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
            except OdtUnavailableError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="unavailable",
                    reason_class="dependency_unavailable",
                    summary="Extraction is unavailable because the ODT package could not be read safely enough to trust bounded extraction.",
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
            except ValueError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="denied",
                    reason_class="ineligible_source",
                    summary="Extraction is denied because the ODT structure exceeds the bounded deterministic recovery limits for this slice.",
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
    except (OSError, zipfile.BadZipFile):
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the ODT package could not be opened through the bounded local ODT lane.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if not structures["content_blocks"]:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because the ODT package has no bounded extractable text structures.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    return _emit_success_result(
        request_id=request_id,
        source_ref=source_ref,
        state="ready",
        completeness_status="complete",
        completeness_summary="Syntax-only extraction completed for a bounded local ODT source.",
        source_hash=source_hash,
        source_modified_at=source_modified_at,
        byte_count=byte_count,
        structures=structures,
    )


def _emit_epub_extraction_result(
    source_path: Path,
    *,
    request_id: str,
    source_ref: str,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
) -> dict[str, Any]:
    try:
        with zipfile.ZipFile(source_path, "r") as epub_archive:
            try:
                epub_surface = extract_epub_surface(epub_archive)
                structures = _build_epub_structures(epub_surface, source_path)
            except EpubDeniedError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="denied",
                    reason_class="unsupported_source_type",
                    summary=(
                        "Extraction is denied because this EPUB uses active, media, or other out-of-lane "
                        "content structures outside the bounded EPUB lane."
                    ),
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
            except EpubUnavailableError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="unavailable",
                    reason_class="dependency_unavailable",
                    summary="Extraction is unavailable because the EPUB package truth could not be established safely enough to trust bounded extraction.",
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
            except ValueError:
                return _emit_failure_result(
                    request_id=request_id,
                    source_ref=source_ref,
                    state="denied",
                    reason_class="ineligible_source",
                    summary="Extraction is denied because the EPUB structure exceeds the bounded deterministic recovery limits for this slice.",
                    source_hash=source_hash,
                    source_modified_at=source_modified_at,
                    byte_count=byte_count,
                )
    except (OSError, zipfile.BadZipFile):
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the EPUB package could not be opened through the bounded local EPUB lane.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if not structures["content_blocks"]:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Extraction is denied because the EPUB package has no bounded extractable text structures.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    return _emit_success_result(
        request_id=request_id,
        source_ref=source_ref,
        state="ready",
        completeness_status="complete",
        completeness_summary="Syntax-only extraction completed for a bounded local EPUB source.",
        source_hash=source_hash,
        source_modified_at=source_modified_at,
        byte_count=byte_count,
        structures=structures,
    )


def emit_extraction_result_from_source_file(
    source_path: str | Path,
    *,
    request_id: str = "direct-local-input",
    source_ref: str | None = None,
    media_type: str | None = None,
) -> dict[str, Any]:
    path = Path(source_path)
    source_ref = source_ref or path.name

    try:
        raw_bytes = path.read_bytes()
        file_stat = path.stat()
    except OSError:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Extraction is unavailable because the source file could not be read.",
        )

    source_hash = _source_hash(raw_bytes)
    source_modified_at = _timestamp_from_epoch(file_stat.st_mtime)
    byte_count = len(raw_bytes)

    lane_decision = lane_eligibility_for_path(path, media_type=media_type)
    if not lane_decision.admitted:
        return _emit_failure_result(
            request_id=request_id,
            source_ref=source_ref,
            state=lane_decision.failure_state or "denied",
            reason_class=lane_decision.reason_class or "unsupported_source_type",
            summary=lane_decision.operator_visible_summary
            or "Extraction is denied because the source is outside the bounded source-lane framework.",
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    lane = lane_decision.lane
    if lane is None:
        raise RuntimeError("lane admission succeeded without a source-lane specification")

    if lane.lane_id == PDF_TEXT_LANE.lane_id:
        return _emit_pdf_extraction_result(
            path,
            request_id=request_id,
            source_ref=source_ref,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if lane.lane_id == RTF_TEXT_LANE.lane_id:
        return _emit_rtf_extraction_result(
            path,
            raw_bytes,
            request_id=request_id,
            source_ref=source_ref,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if lane.lane_id == DOCX_TEXT_LANE.lane_id:
        return _emit_docx_extraction_result(
            path,
            request_id=request_id,
            source_ref=source_ref,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if lane.lane_id == ODT_TEXT_LANE.lane_id:
        return _emit_odt_extraction_result(
            path,
            request_id=request_id,
            source_ref=source_ref,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    if lane.lane_id == EPUB_TEXT_LANE.lane_id:
        return _emit_epub_extraction_result(
            path,
            request_id=request_id,
            source_ref=source_ref,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
        )

    return _emit_text_extraction_result(
        path,
        raw_bytes,
        request_id=request_id,
        source_ref=source_ref,
        source_hash=source_hash,
        source_modified_at=source_modified_at,
        byte_count=byte_count,
        lane=lane,
    )


def emit_extraction_result_from_intake_payload(payload: Any) -> dict[str, Any]:
    request_id = _extract_request_id(payload)
    source_ref = _extract_source_ref(payload)

    validation_result = validate_intake_payload(payload)
    if not validation_result.accepted:
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because the intake payload is not valid for bounded extraction.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    assert isinstance(payload, dict)
    if payload.get("requested_artifact") != "extraction_result":
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because the intake request does not target extraction-result emission.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    if payload.get("source_type") != "file_path":
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because only bounded local file-path intake is supported in this slice.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    sources = payload.get("sources")
    if not isinstance(sources, list) or len(sources) != 1 or not isinstance(sources[0], dict):
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because this slice accepts exactly one bounded local file source per request.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    source = sources[0]
    path = source.get("path")
    media_type = source.get("media_type")

    if not isinstance(path, str) or not path:
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because the intake source path is not present.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    return emit_extraction_result_from_source_file(
        path,
        request_id=request_id,
        source_ref=source_ref,
        media_type=media_type if isinstance(media_type, str) else None,
    )


def emit_extraction_result_from_intake_json_text(payload_text: str) -> dict[str, Any]:
    try:
        payload = json.loads(payload_text)
    except json.JSONDecodeError:
        return _validate_or_fallback(
            _build_failure_result(
                request_id="unknown_request",
                source_ref="unknown_source",
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because the intake payload is not valid JSON.",
            ),
            request_id="unknown_request",
            source_ref="unknown_source",
        )

    return emit_extraction_result_from_intake_payload(payload)


def emit_extraction_result_from_intake_file(path: str | Path) -> dict[str, Any]:
    try:
        payload_text = Path(path).read_text(encoding="utf-8")
    except OSError:
        return _validate_or_fallback(
            _build_failure_result(
                request_id="unknown_request",
                source_ref="unknown_source",
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because the intake payload could not be read.",
            ),
            request_id="unknown_request",
            source_ref="unknown_source",
        )

    return emit_extraction_result_from_intake_json_text(payload_text)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Emit a bounded Cortex extraction-result from intake JSON or a direct local text, PDF, DOCX, RTF, ODT, or EPUB source."
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--input",
        help="Path to an intake-request JSON payload, or '-' to read intake JSON from stdin.",
    )
    group.add_argument(
        "--source-path",
        help="Path to a bounded local text-like, PDF, DOCX, RTF, ODT, or EPUB source file for direct extraction emission.",
    )
    parser.add_argument(
        "--request-id",
        default="direct-local-input",
        help="Request id to use with --source-path. Ignored when --input is used.",
    )
    parser.add_argument(
        "--source-ref",
        default=None,
        help="Source reference to use with --source-path. Defaults to the file name.",
    )
    parser.add_argument(
        "--media-type",
        default=None,
        help="Declared media type to apply to --source-path lane admission. Ignored when --input is used.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)

    if args.input is not None:
        if args.input == "-":
            result = emit_extraction_result_from_intake_json_text(sys.stdin.read())
        else:
            result = emit_extraction_result_from_intake_file(args.input)
    else:
        result = emit_extraction_result_from_source_file(
            source_path=args.source_path,
            request_id=args.request_id,
            source_ref=args.source_ref,
            media_type=args.media_type,
        )

    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["state"] == "ready" else 1


if __name__ == "__main__":
    raise SystemExit(main())
