from __future__ import annotations

from cortex_runtime.extraction_emission import emit_extraction_result_from_source_file
from cortex_runtime.gnats.models import GnatShard
from cortex_runtime.gnats.planner import fingerprint_source
from cortex_runtime.gnats.receipt import (
    build_failure_receipt,
    build_receipt_from_extraction,
    monotonic_ms,
    utc_now,
)


def _fingerprints_match(shard: GnatShard, fingerprint_digest: str) -> bool:
    return shard.source_fingerprint.digest == fingerprint_digest


def run_text_worker(shard: GnatShard, *, expected_worker_type: str) -> dict[str, object]:
    started_at = utc_now()
    start_ms = monotonic_ms()
    try:
        before = fingerprint_source(shard.local_path)
    except Exception:
        completed_at = utc_now()
        duration_ms = max(0, monotonic_ms() - start_ms)
        return build_failure_receipt(
            shard,
            reason_code="worker_unavailable",
            operator_visible_summary="Gnat worker could not read the source file.",
            source_fingerprint_before=shard.source_fingerprint,
            source_fingerprint_after=shard.source_fingerprint,
            state="failed",
            started_at=started_at,
            completed_at=completed_at,
            duration_ms=duration_ms,
        )

    if shard.worker_type != expected_worker_type:
        completed_at = utc_now()
        duration_ms = max(0, monotonic_ms() - start_ms)
        return build_failure_receipt(
            shard,
            reason_code="unsupported_lane",
            operator_visible_summary="Gnat worker type does not match the planned shard.",
            source_fingerprint_before=before,
            source_fingerprint_after=before,
            state="failed",
            started_at=started_at,
            completed_at=completed_at,
            duration_ms=duration_ms,
        )

    if not _fingerprints_match(shard, before.digest):
        completed_at = utc_now()
        duration_ms = max(0, monotonic_ms() - start_ms)
        return build_failure_receipt(
            shard,
            reason_code="source_changed",
            operator_visible_summary="Gnat worker rejected stale output because the source changed after planning.",
            source_fingerprint_before=before,
            source_fingerprint_after=before,
            state="stale",
            started_at=started_at,
            completed_at=completed_at,
            duration_ms=duration_ms,
        )

    output = emit_extraction_result_from_source_file(
        shard.local_path,
        request_id=shard.run_id,
        source_ref=shard.source_ref,
        media_type=shard.media_type,
    )
    try:
        after = fingerprint_source(shard.local_path)
    except Exception:
        after = before
        output = {
            "state": "unavailable",
            "refusal": {
                "reason_class": "dependency_unavailable",
                "operator_visible_summary": "Gnat worker could not re-read the source after extraction.",
            },
        }

    completed_at = utc_now()
    duration_ms = max(0, monotonic_ms() - start_ms)
    if not _fingerprints_match(shard, after.digest):
        return build_failure_receipt(
            shard,
            reason_code="source_changed",
            operator_visible_summary="Gnat worker rejected stale output because the source changed during extraction.",
            source_fingerprint_before=before,
            source_fingerprint_after=after,
            state="stale",
            started_at=started_at,
            completed_at=completed_at,
            duration_ms=duration_ms,
        )

    return build_receipt_from_extraction(
        shard,
        output=output,
        source_fingerprint_before=before,
        source_fingerprint_after=after,
        started_at=started_at,
        completed_at=completed_at,
        duration_ms=duration_ms,
    )
