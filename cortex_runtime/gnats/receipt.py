from __future__ import annotations

import time
from datetime import UTC, datetime
from typing import Any

from cortex_runtime.gnats.models import GNAT_WORKER_IMPLEMENTATION_VERSION, GNAT_WORKER_RECEIPT_VERSION, GnatShard, SourceFingerprint
from cortex_runtime.gnats.schema_validation import require_schema_valid, schema_error_messages


def utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def monotonic_ms() -> int:
    return int(time.monotonic() * 1000)


def _finding_counts(output: dict[str, Any] | None) -> dict[str, int]:
    if not output:
        return {"content_blocks": 0, "sections": 0, "tables_detected": 0}
    structures = output.get("structures")
    if not isinstance(structures, dict):
        return {"content_blocks": 0, "sections": 0, "tables_detected": 0}
    content_blocks = structures.get("content_blocks")
    sections = structures.get("sections")
    tables_detected = structures.get("tables_detected")
    return {
        "content_blocks": len(content_blocks) if isinstance(content_blocks, list) else 0,
        "sections": len(sections) if isinstance(sections, list) else 0,
        "tables_detected": int(tables_detected) if isinstance(tables_detected, int) else 0,
    }


def _base_receipt(
    shard: GnatShard,
    *,
    source_fingerprint_before: SourceFingerprint,
    source_fingerprint_after: SourceFingerprint,
    state: str,
    started_at: str,
    completed_at: str,
    duration_ms: int,
) -> dict[str, Any]:
    return {
        "contract_version": GNAT_WORKER_RECEIPT_VERSION,
        "run_id": shard.run_id,
        "shard_id": shard.shard_id,
        "attempt_id": f"{shard.shard_id}-attempt-1",
        "worker_id": f"{shard.worker_type}:serial",
        "worker_type": shard.worker_type,
        "implementation_version": GNAT_WORKER_IMPLEMENTATION_VERSION,
        "source_fingerprint_before": source_fingerprint_before.to_contract(),
        "source_fingerprint_after": source_fingerprint_after.to_contract(),
        "state": state,
        "started_at": started_at,
        "completed_at": completed_at,
        "duration_ms": duration_ms,
        "finding_counts": {"content_blocks": 0, "sections": 0, "tables_detected": 0},
        "redaction_applied": True,
    }


def error_reason_for_extraction(output: dict[str, Any]) -> str:
    refusal = output.get("refusal")
    reason_class = refusal.get("reason_class") if isinstance(refusal, dict) else None
    if reason_class in {"unsupported_source_type", "ineligible_source"}:
        return "source_ineligible"
    if reason_class == "dependency_unavailable":
        return "worker_unavailable"
    if reason_class == "privacy_scope_violation":
        return "source_ineligible"
    return "internal_error_redacted"


def state_for_extraction(output: dict[str, Any]) -> str:
    state = output.get("state")
    if state in {"ready", "partial_success"}:
        return "complete"
    if state == "denied":
        return "denied"
    if state == "stale":
        return "stale"
    return "failed"


def build_receipt_from_extraction(
    shard: GnatShard,
    *,
    output: dict[str, Any],
    source_fingerprint_before: SourceFingerprint,
    source_fingerprint_after: SourceFingerprint,
    started_at: str,
    completed_at: str,
    duration_ms: int,
) -> dict[str, Any]:
    state = state_for_extraction(output)
    receipt = _base_receipt(
        shard,
        source_fingerprint_before=source_fingerprint_before,
        source_fingerprint_after=source_fingerprint_after,
        state=state,
        started_at=started_at,
        completed_at=completed_at,
        duration_ms=duration_ms,
    )
    receipt["finding_counts"] = _finding_counts(output)
    if state == "complete":
        receipt["bounded_output"] = output
    else:
        receipt["error_reason_code"] = error_reason_for_extraction(output)
        refusal = output.get("refusal")
        summary = refusal.get("operator_visible_summary") if isinstance(refusal, dict) else None
        receipt["operator_visible_summary"] = (
            summary if isinstance(summary, str) and summary else "Gnat worker did not complete the shard."
        )

    if schema_error_messages(output, schema_name="extraction-result.schema.json"):
        receipt["state"] = "failed"
        receipt.pop("bounded_output", None)
        receipt["error_reason_code"] = "output_contract_violation"
        receipt["operator_visible_summary"] = "Gnat worker output violated the extraction contract."

    require_schema_valid(receipt, schema_name="gnat-worker-receipt.schema.json")
    return receipt


def build_failure_receipt(
    shard: GnatShard,
    *,
    reason_code: str,
    operator_visible_summary: str,
    source_fingerprint_before: SourceFingerprint,
    source_fingerprint_after: SourceFingerprint,
    state: str,
    started_at: str,
    completed_at: str,
    duration_ms: int,
) -> dict[str, Any]:
    receipt = _base_receipt(
        shard,
        source_fingerprint_before=source_fingerprint_before,
        source_fingerprint_after=source_fingerprint_after,
        state=state,
        started_at=started_at,
        completed_at=completed_at,
        duration_ms=duration_ms,
    )
    receipt["error_reason_code"] = reason_code
    receipt["operator_visible_summary"] = operator_visible_summary
    require_schema_valid(receipt, schema_name="gnat-worker-receipt.schema.json")
    return receipt
