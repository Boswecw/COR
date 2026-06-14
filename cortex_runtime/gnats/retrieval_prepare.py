from __future__ import annotations

import hashlib
from datetime import UTC, datetime
from typing import Any, Iterable

from cortex_runtime.gnats.models import GnatParallelResult, GnatPersistentResult, GnatRunPlan, GnatSerialResult
from cortex_runtime.gnats.schema_validation import require_schema_valid, schema_error_messages
from cortex_runtime.retrieval_package_emission import emit_retrieval_package_from_extraction_result


DEFAULT_TTL_SECONDS = 3600


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _short_hash(payload: str) -> str:
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:16]


def _package_id(plan: GnatRunPlan) -> str:
    return f"pkg-gnat-{_short_hash(plan.run_id + ':retrieval-prepare')}"


def _chunk_id(plan: GnatRunPlan, source_ref: str, source_ordinal: int, local_ordinal: int, global_ordinal: int) -> str:
    digest = _short_hash(f"{plan.run_id}:{source_ref}:{source_ordinal}:{local_ordinal}:{global_ordinal}")
    return f"chunk-gnat-{digest}"


def emit_retrieval_package_from_gnat_result(
    result: GnatParallelResult | GnatPersistentResult | GnatSerialResult,
) -> dict[str, Any]:
    return emit_retrieval_package_from_gnat_receipts(result.plan, result.receipts)


def emit_retrieval_package_from_gnat_receipts(
    plan: GnatRunPlan,
    receipts: Iterable[dict[str, Any]],
) -> dict[str, Any]:
    receipts_by_shard = {str(receipt.get("shard_id")): receipt for receipt in receipts}
    chunks: list[dict[str, Any]] = []
    source_refs: list[str] = []
    failure_count = 0
    chunking_modes: set[str] = set()

    for shard in plan.shards:
        if shard.source_ref not in source_refs:
            source_refs.append(shard.source_ref)
        receipt = receipts_by_shard.get(shard.shard_id)
        if receipt is None or schema_error_messages(receipt, schema_name="gnat-worker-receipt.schema.json"):
            failure_count += 1
            continue
        if receipt.get("state") != "complete":
            failure_count += 1
            continue
        extraction_result = receipt.get("bounded_output")
        retrieval_package = emit_retrieval_package_from_extraction_result(extraction_result)
        if retrieval_package.get("state") != "ready":
            failure_count += 1
            continue
        profile = retrieval_package.get("retrieval_profile")
        if isinstance(profile, dict) and isinstance(profile.get("chunking_mode"), str):
            chunking_modes.add(str(profile["chunking_mode"]))
        for local_chunk in retrieval_package.get("chunks", []):
            if not isinstance(local_chunk, dict):
                failure_count += 1
                continue
            global_ordinal = len(chunks)
            chunks.append(
                {
                    "chunk_id": _chunk_id(
                        plan,
                        shard.source_ref,
                        shard.ordinal,
                        int(local_chunk.get("ordinal", 0)),
                        global_ordinal,
                    ),
                    "source_ref": shard.source_ref,
                    "structure_kind": str(local_chunk.get("structure_kind", "paragraph")),
                    "text": str(local_chunk.get("text", "")),
                    "ordinal": global_ordinal,
                }
            )

    if not chunks:
        return _validate_retrieval_package(_denied_package(plan, source_refs=source_refs))

    state = "partial_success" if failure_count else "ready"
    chunking_mode = next(iter(chunking_modes)) if len(chunking_modes) == 1 else "hybrid"
    package = {
        "package_id": _package_id(plan),
        "request_id": plan.request_id,
        "source_refs": source_refs,
        "retrieval_profile": {
            "profile_id": "gnat.retrieval-prep-bounded.v1",
            "chunking_mode": chunking_mode,
            "max_chunk_chars": 1200,
            "overlap_chars": 0,
        },
        "state": state,
        "freshness": {
            "state": "fresh" if state == "ready" else "unknown",
            "asserted_at": _utc_now(),
            "ttl_seconds": DEFAULT_TTL_SECONDS,
            "source_dependency_marker": plan.plan_hash,
            "operator_visible_summary": (
                "Fresh against the validated GNAT extraction plan."
                if state == "ready"
                else "Partially prepared from validated GNAT extraction receipts; one or more shards did not complete."
            ),
        },
        "invalidation": {
            "policy": "compound",
            "stale_if_source_changes": True,
            "stale_if_profile_changes": True,
            "manual_invalidation_allowed": True,
        },
        "chunks": chunks,
        "completeness": {
            "status": "complete" if state == "ready" else "incomplete",
            "operator_visible_summary": (
                "Retrieval package prepared from validated GNAT extraction receipts."
                if state == "ready"
                else "Retrieval package prepared from completed GNAT shards only; failed shards are omitted."
            ),
        },
        "non_canonical": True,
        "non_semantic_default": True,
        "details_redacted": True,
        "created_at": _utc_now(),
    }
    return _validate_retrieval_package(package)


def _denied_package(plan: GnatRunPlan, *, source_refs: list[str]) -> dict[str, Any]:
    return {
        "package_id": _package_id(plan),
        "request_id": plan.request_id,
        "source_refs": source_refs or ["unknown_source"],
        "retrieval_profile": {
            "profile_id": "gnat.retrieval-prep-bounded.v1",
            "chunking_mode": "hybrid",
            "max_chunk_chars": 1200,
            "overlap_chars": 0,
        },
        "state": "denied",
        "freshness": {
            "state": "unknown",
            "asserted_at": _utc_now(),
            "ttl_seconds": DEFAULT_TTL_SECONDS,
            "source_dependency_marker": plan.plan_hash,
            "operator_visible_summary": "Retrieval preparation denied because no completed GNAT extraction receipt was chunkable.",
        },
        "invalidation": {
            "policy": "compound",
            "stale_if_source_changes": True,
            "stale_if_profile_changes": True,
            "manual_invalidation_allowed": True,
        },
        "refusal": {
            "reason_class": "missing_required_structure",
            "operator_visible_summary": "Retrieval preparation denied because no completed GNAT extraction receipt was chunkable.",
        },
        "non_canonical": True,
        "non_semantic_default": True,
        "details_redacted": True,
        "created_at": _utc_now(),
    }


def _validate_retrieval_package(package: dict[str, Any]) -> dict[str, Any]:
    require_schema_valid(package, schema_name="retrieval-package.schema.json")
    return package
