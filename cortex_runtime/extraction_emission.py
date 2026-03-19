from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from datetime import UTC, datetime
from functools import lru_cache
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from cortex_runtime.intake_validation import validate_intake_payload


ROOT = Path(__file__).resolve().parent.parent
EXTRACTION_SCHEMA_PATH = ROOT / "schemas/extraction-result.schema.json"
EXTRACTION_SCHEMA_REF = "schemas/extraction-result.schema.json"
EXTRACTOR_VERSION = "slice2.syntax_only.1"
SUPPORTED_SUFFIXES = {".md", ".txt"}
SUPPORTED_MEDIA_TYPES = {"text/markdown", "text/plain"}


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _timestamp_from_epoch(epoch_seconds: float) -> str:
    return datetime.fromtimestamp(epoch_seconds, tz=UTC).isoformat().replace("+00:00", "Z")


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


def _parse_markdown_heading(line: str) -> tuple[int, str] | None:
    if not line.startswith("#"):
        return None

    marker, _, title = line.partition(" ")
    if not title or len(marker) > 8 or any(char != "#" for char in marker):
        return None

    return len(marker), title.strip()


def _build_structures(text: str, source_path: Path) -> dict[str, Any]:
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

        heading = _parse_markdown_heading(stripped)
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
        "metadata_fields": {
            "file_name": source_path.name,
            "file_extension": source_path.suffix or "<none>",
        },
        "content_blocks": blocks,
    }

    if heading_positions:
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
        structures["sections"] = sections

    return structures


@lru_cache(maxsize=1)
def _extraction_validator() -> Draft202012Validator:
    with EXTRACTION_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        schema = json.load(handle)
    return Draft202012Validator(schema)


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
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="unavailable",
                reason_class="dependency_unavailable",
                summary="Extraction is unavailable because the source file could not be read.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    source_hash = _source_hash(raw_bytes)
    source_modified_at = _timestamp_from_epoch(file_stat.st_mtime)
    byte_count = len(raw_bytes)

    if path.suffix.lower() not in SUPPORTED_SUFFIXES:
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="unsupported_source_type",
                summary="Extraction is denied because only bounded local .md and .txt sources are supported in this slice.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    if media_type is not None and media_type not in SUPPORTED_MEDIA_TYPES:
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="unsupported_source_type",
                summary="Extraction is denied because the source media type is outside the bounded text-only slice.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    try:
        text = raw_bytes.decode("utf-8")
    except UnicodeDecodeError:
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="unsupported_source_type",
                summary="Extraction is denied because only UTF-8 text-like input is supported in this slice.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    try:
        structures = _build_structures(text, path)
    except ValueError:
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because the source exceeds the bounded literal extraction limits for this slice.",
                source_hash=source_hash,
                source_modified_at=source_modified_at,
                byte_count=byte_count,
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    ready_result = {
        "artifact_id": _artifact_id(request_id, source_ref),
        "request_id": request_id,
        "source_ref": source_ref,
        "state": "ready",
        "syntax_boundary": "syntax_only",
        "semantic_boundary_enforced": True,
        "provenance": {
            "source_hash": source_hash,
            "extractor_version": EXTRACTOR_VERSION,
            "source_modified_at": source_modified_at,
            "byte_count": byte_count,
        },
        "completeness": {
            "status": "complete",
            "operator_visible_summary": "Syntax-only extraction completed for a bounded local text source.",
        },
        "structures": structures,
        "extracted_at": _utc_now(),
    }

    return _validate_or_fallback(
        ready_result,
        request_id=request_id,
        source_ref=source_ref,
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
                reason_class="unsupported_source_type",
                summary="Extraction is denied because this slice supports only bounded file-path intake.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    sources = payload.get("sources", [])
    if not isinstance(sources, list) or len(sources) != 1 or not isinstance(sources[0], dict):
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="ineligible_source",
                summary="Extraction is denied because this slice accepts exactly one bounded source.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    source = copy.deepcopy(sources[0])
    if source.get("source_class") != "file":
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                state="denied",
                reason_class="unsupported_source_type",
                summary="Extraction is denied because this slice supports only file-class sources.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    return emit_extraction_result_from_source_file(
        source_path=source["path"],
        request_id=request_id,
        source_ref=source_ref,
        media_type=source.get("media_type"),
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
                state="unavailable",
                reason_class="dependency_unavailable",
                summary="Extraction is unavailable because the intake payload file could not be read.",
            ),
            request_id="unknown_request",
            source_ref="unknown_source",
        )

    return emit_extraction_result_from_intake_json_text(payload_text)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Emit a bounded syntax-only Cortex extraction result from intake JSON or a direct local text source."
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--input",
        help="Path to an intake request JSON payload, or '-' to read intake JSON from stdin.",
    )
    group.add_argument(
        "--source-path",
        help="Path to a bounded local text-like source file for direct extraction emission.",
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
            args.source_path,
            request_id=args.request_id,
            source_ref=args.source_ref,
        )

    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["state"] == "ready" else 1


if __name__ == "__main__":
    raise SystemExit(main())
