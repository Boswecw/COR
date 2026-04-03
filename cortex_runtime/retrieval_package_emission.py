from __future__ import annotations

import argparse
import hashlib
import json
import sys
from datetime import UTC, datetime
from functools import lru_cache
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator  # type: ignore[import-untyped]

from cortex_runtime.extraction_emission import (
    emit_extraction_result_from_source_file,
)


ROOT = Path(__file__).resolve().parent.parent
RETRIEVAL_SCHEMA_PATH = ROOT / "schemas/retrieval-package.schema.json"
RETRIEVAL_SCHEMA_REF = "schemas/retrieval-package.schema.json"
DEFAULT_TTL_SECONDS = 3600


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _package_id(request_id: str, source_ref: str) -> str:
    digest = hashlib.sha256(f"{request_id}:{source_ref}:retrieval".encode("utf-8")).hexdigest()[:16]
    return f"pkg-{digest}"


def _chunk_id(request_id: str, source_ref: str, ordinal: int, structure_kind: str) -> str:
    digest = hashlib.sha256(
        f"{request_id}:{source_ref}:{ordinal}:{structure_kind}".encode("utf-8")
    ).hexdigest()[:12]
    return f"chunk-{digest}"


def _schema_error_messages(result: dict[str, Any]) -> list[str]:
    validator = _retrieval_validator()
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
        reason_class="unsupported_route",
        summary="Retrieval-package emission failed closed because the bounded retrieval contract could not be satisfied.",
    )
    fallback_errors = _schema_error_messages(fallback)
    if fallback_errors:
        raise RuntimeError(
            "fallback retrieval package violated schema: " + "; ".join(fallback_errors)
        )
    return fallback


def _build_failure_result(
    *,
    request_id: str,
    source_ref: str,
    reason_class: str,
    summary: str,
    source_dependency_marker: str | None = None,
) -> dict[str, Any]:
    return {
        "package_id": _package_id(request_id, source_ref),
        "request_id": request_id,
        "source_refs": [source_ref],
        "retrieval_profile": {
            "profile_id": "slice3.route-denied-v1",
            "chunking_mode": "paragraph",
            "max_chunk_chars": 1200,
            "overlap_chars": 0,
        },
        "state": "denied",
        "freshness": {
            "state": "unknown",
            "asserted_at": _utc_now(),
            "ttl_seconds": DEFAULT_TTL_SECONDS,
            "source_dependency_marker": source_dependency_marker or f"unavailable:{source_ref}",
            "operator_visible_summary": summary,
        },
        "invalidation": {
            "policy": "compound",
            "stale_if_source_changes": True,
            "stale_if_profile_changes": True,
            "manual_invalidation_allowed": True,
        },
        "refusal": {
            "reason_class": reason_class,
            "operator_visible_summary": summary,
        },
        "non_canonical": True,
        "non_semantic_default": True,
        "details_redacted": True,
        "created_at": _utc_now(),
    }


def _extract_request_id(payload: Any) -> str:
    if isinstance(payload, dict):
        request_id = payload.get("request_id")
        if isinstance(request_id, str) and request_id:
            return request_id
    return "unknown_request"


def _extract_source_ref(payload: Any) -> str:
    if isinstance(payload, dict):
        source_ref = payload.get("source_ref")
        if isinstance(source_ref, str) and source_ref:
            return source_ref
    return "unknown_source"


@lru_cache(maxsize=1)
def _retrieval_validator() -> Draft202012Validator:
    with RETRIEVAL_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        schema = json.load(handle)
    return Draft202012Validator(schema)


def _section_chunks(extraction_result: dict[str, Any]) -> list[dict[str, Any]]:
    request_id = extraction_result["request_id"]
    source_ref = extraction_result["source_ref"]
    structures = extraction_result["structures"]
    content_blocks = structures.get("content_blocks", [])
    sections = structures.get("sections", [])

    chunks: list[dict[str, Any]] = []
    block_start = 0
    for ordinal, section in enumerate(sections):
        block_count = section["block_count"]
        block_slice = content_blocks[block_start : block_start + block_count]
        block_start += block_count

        chunk_text = "\n\n".join(block["text"] for block in block_slice if block.get("text"))
        if not chunk_text or len(chunk_text) > 20000:
            raise ValueError("bounded retrieval chunk limits exceeded")

        chunks.append(
            {
                "chunk_id": _chunk_id(request_id, source_ref, ordinal, "section"),
                "source_ref": source_ref,
                "structure_kind": "section",
                "text": chunk_text,
                "ordinal": ordinal,
            }
        )

    return chunks


def _paragraph_chunks(extraction_result: dict[str, Any]) -> list[dict[str, Any]]:
    request_id = extraction_result["request_id"]
    source_ref = extraction_result["source_ref"]
    content_blocks = extraction_result["structures"].get("content_blocks", [])

    paragraph_blocks = [
        block for block in content_blocks if isinstance(block, dict) and block.get("block_kind") == "paragraph"
    ]

    chunks: list[dict[str, Any]] = []
    for ordinal, block in enumerate(paragraph_blocks):
        text = block["text"]
        if not text or len(text) > 20000:
            raise ValueError("bounded retrieval chunk limits exceeded")

        chunks.append(
            {
                "chunk_id": _chunk_id(request_id, source_ref, ordinal, "paragraph"),
                "source_ref": source_ref,
                "structure_kind": "paragraph",
                "text": text,
                "ordinal": ordinal,
            }
        )

    return chunks


def _build_ready_result(extraction_result: dict[str, Any]) -> dict[str, Any]:
    request_id = extraction_result["request_id"]
    source_ref = extraction_result["source_ref"]
    structures = extraction_result["structures"]
    sections = structures.get("sections")

    if isinstance(sections, list) and sections:
        chunks = _section_chunks(extraction_result)
        chunking_mode = "section"
        profile_id = "slice3.section-bounded.v1"
    else:
        chunks = _paragraph_chunks(extraction_result)
        chunking_mode = "paragraph"
        profile_id = "slice3.paragraph-bounded.v1"

    if not chunks:
        raise ValueError("missing chunkable syntax structure")

    source_hash = extraction_result["provenance"]["source_hash"]
    return {
        "package_id": _package_id(request_id, source_ref),
        "request_id": request_id,
        "source_refs": [source_ref],
        "retrieval_profile": {
            "profile_id": profile_id,
            "chunking_mode": chunking_mode,
            "max_chunk_chars": 1200,
            "overlap_chars": 0,
        },
        "state": "ready",
        "freshness": {
            "state": "fresh",
            "asserted_at": extraction_result["extracted_at"],
            "ttl_seconds": DEFAULT_TTL_SECONDS,
            "source_dependency_marker": source_hash,
            "operator_visible_summary": "Fresh against the current extraction provenance hash.",
        },
        "invalidation": {
            "policy": "compound",
            "stale_if_source_changes": True,
            "stale_if_profile_changes": True,
            "manual_invalidation_allowed": True,
        },
        "chunks": chunks,
        "completeness": {
            "status": "complete",
            "operator_visible_summary": "Retrieval package emission completed from bounded syntax-only extraction output.",
        },
        "non_canonical": True,
        "non_semantic_default": True,
        "details_redacted": True,
        "created_at": _utc_now(),
    }


def emit_retrieval_package_from_extraction_result(extraction_result: Any) -> dict[str, Any]:
    request_id = _extract_request_id(extraction_result)
    source_ref = _extract_source_ref(extraction_result)

    if not isinstance(extraction_result, dict):
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                reason_class="unsupported_route",
                summary="Retrieval-package emission is denied because the upstream extraction result is not a bounded object.",
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    state = extraction_result.get("state")
    if state == "stale":
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                reason_class="stale_input",
                summary="Retrieval-package emission is denied because the upstream extraction result is stale.",
                source_dependency_marker=extraction_result.get("provenance", {}).get("source_hash"),
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    if state != "ready":
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                reason_class="unsupported_route",
                summary="Retrieval-package emission is denied because this slice accepts only ready extraction-result upstream input.",
                source_dependency_marker=extraction_result.get("provenance", {}).get("source_hash"),
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    if extraction_result.get("syntax_boundary") != "syntax_only" or extraction_result.get(
        "semantic_boundary_enforced"
    ) is not True:
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                reason_class="unsupported_route",
                summary="Retrieval-package emission is denied because the upstream extraction result is not syntax-only.",
                source_dependency_marker=extraction_result.get("provenance", {}).get("source_hash"),
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    completeness = extraction_result.get("completeness", {})
    if completeness.get("status") != "complete":
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                reason_class="missing_required_structure",
                summary="Retrieval-package emission is denied because the upstream extraction result is not complete.",
                source_dependency_marker=extraction_result.get("provenance", {}).get("source_hash"),
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    structures = extraction_result.get("structures")
    if not isinstance(structures, dict):
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                reason_class="missing_required_structure",
                summary="Retrieval-package emission is denied because chunkable syntax structure is missing.",
                source_dependency_marker=extraction_result.get("provenance", {}).get("source_hash"),
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    try:
        ready_result = _build_ready_result(extraction_result)
    except (KeyError, TypeError, ValueError):
        return _validate_or_fallback(
            _build_failure_result(
                request_id=request_id,
                source_ref=source_ref,
                reason_class="missing_required_structure",
                summary="Retrieval-package emission is denied because the upstream extraction structure is not chunkable in this slice.",
                source_dependency_marker=extraction_result.get("provenance", {}).get("source_hash"),
            ),
            request_id=request_id,
            source_ref=source_ref,
        )

    return _validate_or_fallback(
        ready_result,
        request_id=request_id,
        source_ref=source_ref,
    )


def emit_retrieval_package_from_extraction_json_text(payload_text: str) -> dict[str, Any]:
    try:
        extraction_result = json.loads(payload_text)
    except json.JSONDecodeError:
        return _validate_or_fallback(
            _build_failure_result(
                request_id="unknown_request",
                source_ref="unknown_source",
                reason_class="unsupported_route",
                summary="Retrieval-package emission is denied because the upstream extraction payload is not valid JSON.",
            ),
            request_id="unknown_request",
            source_ref="unknown_source",
        )

    return emit_retrieval_package_from_extraction_result(extraction_result)


def emit_retrieval_package_from_extraction_file(path: str | Path) -> dict[str, Any]:
    try:
        payload_text = Path(path).read_text(encoding="utf-8")
    except OSError:
        return _validate_or_fallback(
            _build_failure_result(
                request_id="unknown_request",
                source_ref="unknown_source",
                reason_class="unsupported_route",
                summary="Retrieval-package emission is denied because the upstream extraction payload could not be read.",
            ),
            request_id="unknown_request",
            source_ref="unknown_source",
        )

    return emit_retrieval_package_from_extraction_json_text(payload_text)


def emit_retrieval_package_from_source_file(
    source_path: str | Path,
    *,
    request_id: str = "direct-local-input",
    source_ref: str | None = None,
    media_type: str | None = None,
) -> dict[str, Any]:
    extraction_result = emit_extraction_result_from_source_file(
        source_path=source_path,
        request_id=request_id,
        source_ref=source_ref,
        media_type=media_type,
    )
    return emit_retrieval_package_from_extraction_result(extraction_result)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Emit a bounded Cortex retrieval-package from a ready extraction result or a direct local text, PDF, DOCX, RTF, ODT, or EPUB source."
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--input",
        help="Path to an extraction-result JSON payload, or '-' to read extraction JSON from stdin.",
    )
    group.add_argument(
        "--source-path",
        help="Path to a bounded local text-like, PDF, DOCX, RTF, ODT, or EPUB source file for direct retrieval-package emission.",
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
            result = emit_retrieval_package_from_extraction_json_text(sys.stdin.read())
        else:
            result = emit_retrieval_package_from_extraction_file(args.input)
    else:
        result = emit_retrieval_package_from_source_file(
            source_path=args.source_path,
            request_id=args.request_id,
            source_ref=args.source_ref,
            media_type=args.media_type,
        )

    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["state"] == "ready" else 1


if __name__ == "__main__":
    raise SystemExit(main())
